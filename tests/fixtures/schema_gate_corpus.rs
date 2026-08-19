use serde_json::{Map, Value, json};

pub const CASES: &[&str] = &[
    "nested-required",
    "array-items",
    "enums",
    "combinators",
    "formats",
    "local-ref-pattern",
];

pub fn schema(case_id: &str) -> Option<Value> {
    let schema = match case_id {
        "nested-required" => json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "type": "object",
            "properties": {
                "account": {
                    "type": "object",
                    "properties": {
                        "identity": {"type": "string", "description": "Synthetic stable account identity."},
                        "enabled": {"type": "boolean", "description": "Synthetic stable account state."},
                        "profile": {
                            "type": "object",
                            "properties": {
                                "label": {"type": "string", "description": "Synthetic stable profile label."}
                            },
                            "required": ["label"],
                            "additionalProperties": false
                        }
                    },
                    "required": ["identity", "enabled", "profile"],
                    "additionalProperties": false
                }
            },
            "required": ["account"],
            "additionalProperties": false
        }),
        "array-items" => json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "type": "object",
            "properties": {
                "records": {
                    "type": "array",
                    "minItems": 1,
                    "maxItems": 32,
                    "items": {
                        "type": "object",
                        "properties": {
                            "label": {"type": "string", "description": "Synthetic stable record label."},
                            "quantity": {"type": "integer", "minimum": 0, "description": "Synthetic stable record quantity."}
                        },
                        "required": ["label", "quantity"],
                        "additionalProperties": false
                    }
                }
            },
            "required": ["records"],
            "additionalProperties": false
        }),
        "enums" => json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "type": "object",
            "properties": {
                "mode": {"type": "string", "enum": ["alpha", "beta", "gamma", "delta"], "description": "Synthetic stable execution mode."},
                "level": {"type": "integer", "enum": [1, 2, 3, 4], "description": "Synthetic stable execution level."},
                "state": {"type": "string", "enum": ["ready", "paused", "closed"], "description": "Synthetic stable execution state."}
            },
            "required": ["mode", "level", "state"],
            "additionalProperties": false
        }),
        "combinators" => json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "type": "object",
            "properties": {
                "selector": {
                    "oneOf": [
                        {"type": "object", "properties": {"kind": {"const": "numeric"}, "value": {"type": "integer"}}, "required": ["kind", "value"], "additionalProperties": false},
                        {"type": "object", "properties": {"kind": {"const": "textual"}, "value": {"type": "string"}}, "required": ["kind", "value"], "additionalProperties": false}
                    ]
                },
                "fallback": {"anyOf": [{"type": "integer"}, {"type": "string"}, {"type": "null"}], "description": "Synthetic stable fallback selection."}
            },
            "required": ["selector"],
            "additionalProperties": false
        }),
        "formats" => json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "type": "object",
            "properties": {
                "email": {"type": "string", "format": "email", "description": "Synthetic stable contact address."},
                "endpoint": {"type": "string", "format": "uri", "description": "Synthetic stable public endpoint."},
                "created": {"type": "string", "format": "date-time", "description": "Synthetic stable creation timestamp."},
                "day": {"type": "string", "format": "date", "description": "Synthetic stable calendar date."}
            },
            "required": ["email", "endpoint", "created", "day"],
            "additionalProperties": false
        }),
        "local-ref-pattern" => json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "$defs": {
                "entry": {
                    "type": "object",
                    "properties": {
                        "code": {"type": "string", "pattern": "^[A-Za-z0-9_-]+$", "description": "Synthetic stable ASCII code."},
                        "active": {"type": "boolean", "description": "Synthetic stable entry state."}
                    },
                    "required": ["code", "active"],
                    "additionalProperties": false
                }
            },
            "type": "object",
            "properties": {
                "primary": {"$ref": "#/$defs/entry"},
                "related": {"type": "array", "items": {"$ref": "#/$defs/entry"}, "maxItems": 16}
            },
            "required": ["primary"],
            "additionalProperties": false
        }),
        _ => return None,
    };
    Some(pad(schema))
}

fn pad(mut schema: Value) -> Value {
    schema
        .as_object_mut()
        .expect("every synthetic gate schema is an object")
        .entry("description")
        .or_insert_with(|| {
            Value::String("Synthetic private schema sentinel never report 7f2c".to_owned())
        });
    let mut index = 0_u32;
    while serde_json::to_vec(&schema)
        .expect("the synthetic schema should serialize")
        .len()
        < 1_536
    {
        let object = schema
            .as_object_mut()
            .expect("every synthetic gate schema is an object");
        let properties = object
            .entry("properties")
            .or_insert_with(|| Value::Object(Map::new()))
            .as_object_mut()
            .expect("the synthetic properties collection is an object");
        properties.insert(
            format!("optional_{index:02}"),
            json!({
                "type": "string",
                "description": "Synthetic optional text input with a stable, ordinary description."
            }),
        );
        index = index.saturating_add(1);
    }
    schema
}
