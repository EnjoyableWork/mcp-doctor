use std::collections::BTreeSet;

use serde_json::Value;

use super::model::{Finding, Location, LocationField, RuleViolation};
use crate::transport::MirroredField;
use crate::transport::http::mirrored_primitive;

// A tools/call request always owns six explicit transport fields, Mcp-Name,
// Host, and Content-Length. Mirrored annotations cannot consume those slots.
const MAX_MIRRORED_FIELDS: usize = 55;
const MAX_HTTP_FIELD_NAME_BYTES: usize = 256;
const MCP_PARAM_PREFIX: &str = "Mcp-Param-";

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum PrimitiveKind {
    String,
    Integer,
    Boolean,
}

#[derive(Clone)]
pub(super) struct HeaderAnnotation {
    suffix: String,
    path: Vec<String>,
    kind: PrimitiveKind,
}

impl std::fmt::Debug for HeaderAnnotation {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("HeaderAnnotation")
            .field("suffix", &"[REDACTED]")
            .field("path_depth", &self.path.len())
            .field("kind", &self.kind)
            .finish()
    }
}

pub(super) fn validate_annotations(
    schema: &Value,
    location: Location,
) -> Result<Vec<HeaderAnnotation>, Finding> {
    let mut annotations = Vec::new();
    let mut names = BTreeSet::new();
    if scan_schema(schema, None, &mut annotations, &mut names).is_err()
        || annotations.len() > MAX_MIRRORED_FIELDS
    {
        return Err(Finding::http_header_mapping_invalid(
            location.field(LocationField::Properties).wildcard(),
            RuleViolation::InvalidHttpHeaderAnnotation,
        ));
    }
    Ok(annotations)
}

fn scan_schema(
    schema: &Value,
    property_path: Option<Vec<String>>,
    annotations: &mut Vec<HeaderAnnotation>,
    names: &mut BTreeSet<String>,
) -> Result<(), ()> {
    let Some(object) = schema.as_object() else {
        return scan_disallowed(schema);
    };

    if let Some(annotation) = object.get("x-mcp-header") {
        let path = property_path.as_ref().ok_or(())?;
        let suffix = annotation.as_str().filter(|suffix| {
            valid_token(suffix)
                && MCP_PARAM_PREFIX.len().saturating_add(suffix.len()) <= MAX_HTTP_FIELD_NAME_BYTES
        });
        let suffix = suffix.ok_or(())?;
        let normalized = suffix.to_ascii_lowercase();
        if !names.insert(normalized) {
            return Err(());
        }
        let kind = match object.get("type").and_then(Value::as_str) {
            Some("string") => PrimitiveKind::String,
            Some("integer") => PrimitiveKind::Integer,
            Some("boolean") => PrimitiveKind::Boolean,
            _ => return Err(()),
        };
        annotations.push(HeaderAnnotation {
            suffix: suffix.to_owned(),
            path: path.clone(),
            kind,
        });
    }

    for (key, value) in object {
        if key == "x-mcp-header" {
            continue;
        }
        if key == "properties" {
            let properties = value.as_object().ok_or(())?;
            let prefix = property_path.clone().unwrap_or_default();
            for (name, property_schema) in properties {
                let mut path = prefix.clone();
                path.push(name.clone());
                scan_schema(property_schema, Some(path), annotations, names)?;
            }
        } else {
            scan_disallowed(value)?;
        }
    }
    Ok(())
}

fn scan_disallowed(value: &Value) -> Result<(), ()> {
    match value {
        Value::Object(object) => {
            if object.contains_key("x-mcp-header") {
                return Err(());
            }
            for value in object.values() {
                scan_disallowed(value)?;
            }
        }
        Value::Array(values) => {
            for value in values {
                scan_disallowed(value)?;
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
    }
    Ok(())
}

impl HeaderAnnotation {
    pub(super) fn extract(&self, arguments: &Value) -> Result<Option<MirroredField>, ()> {
        let mut value = arguments;
        for segment in &self.path {
            let Some(next) = value.as_object().and_then(|object| object.get(segment)) else {
                return Ok(None);
            };
            value = next;
        }
        if !matches!(
            (self.kind, value),
            (PrimitiveKind::String, Value::String(_))
                | (PrimitiveKind::Integer, Value::Number(_))
                | (PrimitiveKind::Boolean, Value::Bool(_))
                | (_, Value::Null)
        ) {
            return Err(());
        }
        let value = mirrored_primitive(value)?;
        let Some(value) = value else {
            return Ok(None);
        };
        Ok(Some(MirroredField::new(self.suffix.clone(), value)))
    }
}

fn valid_token(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric()
                || matches!(
                    byte,
                    b'!' | b'#'
                        | b'$'
                        | b'%'
                        | b'&'
                        | b'\''
                        | b'*'
                        | b'+'
                        | b'-'
                        | b'.'
                        | b'^'
                        | b'_'
                        | b'`'
                        | b'|'
                        | b'~'
                )
        })
}

#[cfg(test)]
mod tests {
    use serde_json::{Map, Value, json};

    use super::validate_annotations;
    use crate::contract::model::{Location, LocationField};

    #[test]
    fn annotations_are_only_valid_on_unique_primitive_property_chains() {
        let valid = json!({
            "type": "object",
            "properties": {
                "region": {"type": "string", "x-mcp-header": "Region"},
                "nested": {
                    "type": "object",
                    "properties": {
                        "enabled": {"type": "boolean", "x-mcp-header": "Enabled"}
                    }
                }
            }
        });
        assert_eq!(
            validate_annotations(&valid, Location::root(LocationField::InputSchema))
                .expect("the property-chain annotations are valid")
                .len(),
            2
        );

        for invalid in [
            json!({"type":"object","x-mcp-header":"Root"}),
            json!({"type":"object","properties":{"v":{"type":"number","x-mcp-header":"V"}}}),
            json!({"type":"object","properties":{"a":{"type":"string","x-mcp-header":"Same"},"b":{"type":"string","x-mcp-header":"same"}}}),
            json!({"type":"object","oneOf":[{"properties":{"a":{"type":"string","x-mcp-header":"A"}}}]}),
        ] {
            assert!(
                validate_annotations(&invalid, Location::root(LocationField::InputSchema)).is_err()
            );
        }

        let properties = (0..56)
            .map(|index| {
                (
                    format!("p{index}"),
                    json!({"type":"string","x-mcp-header":format!("P{index}")}),
                )
            })
            .collect::<Map<String, Value>>();
        assert!(
            validate_annotations(
                &json!({"type":"object","properties":properties}),
                Location::root(LocationField::InputSchema),
            )
            .is_err()
        );
    }
}
