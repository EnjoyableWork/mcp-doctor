use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Number, Value};

use super::catalog::{DRAFT_2020_12, resolve_local_reference};
use super::limits::DiagnosticLimits;
use super::protocol::{KnownRevision, SupportedRevision};
use crate::transport::ProbeResponse;

pub(crate) const SNAPSHOT_SCHEMA_VERSION: &str = "mcp-doctor.contract-snapshot/v1alpha1";
pub(crate) const DIFF_SCHEMA_VERSION: &str = "mcp-doctor.contract-diff/v1alpha1";

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ContractSnapshot {
    schema_version: String,
    protocol_revision: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    negotiated_protocol_revision: Option<String>,
    capabilities: SnapshotCapabilities,
    catalogs: SnapshotCatalogs,
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SnapshotCapabilities {
    #[serde(default)]
    tools: ListCapability,
    #[serde(default)]
    prompts: ListCapability,
    #[serde(default)]
    resources: ResourceCapability,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    logging: Option<PresenceCapability>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    completions: Option<PresenceCapability>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    tasks: Option<TaskCapability>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PresenceCapability {
    advertised: bool,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct TaskCapability {
    advertised: bool,
    list: bool,
    cancel: bool,
    requests_tools_call: bool,
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ListCapability {
    #[serde(default)]
    advertised: bool,
    #[serde(default)]
    list_changed: bool,
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ResourceCapability {
    #[serde(default)]
    advertised: bool,
    #[serde(default)]
    list_changed: bool,
    #[serde(default)]
    subscribe: bool,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SnapshotCatalogs {
    tools: SnapshotCatalog<ToolContract>,
    prompts: SnapshotCatalog<PromptContract>,
    resources: SnapshotCatalog<ResourceContract>,
    resource_templates: SnapshotCatalog<ResourceTemplateContract>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SnapshotCatalog<T> {
    contracts: Vec<T>,
    correlation: Vec<OrdinalCorrelation>,
}

impl<T> Default for SnapshotCatalog<T> {
    fn default() -> Self {
        Self {
            contracts: Vec::new(),
            correlation: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct OrdinalCorrelation {
    discovery_ordinal: usize,
    contract_index: usize,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ToolContract {
    name: String,
    #[serde(default)]
    behavior_hints: ToolBehaviorHints,
    input_schema: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    input_schema_dialect: Option<SnapshotSchemaDialect>,
    #[serde(skip_serializing_if = "Option::is_none")]
    output_schema: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    output_schema_dialect: Option<SnapshotSchemaDialect>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum SnapshotSchemaDialect {
    #[serde(rename = "draft_2020_12")]
    Draft2020_12,
    Ambiguous,
    Unsupported,
}

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ToolBehaviorHints {
    #[serde(default)]
    read_only: bool,
    #[serde(default = "default_true")]
    destructive: bool,
    #[serde(default)]
    idempotent: bool,
    #[serde(default = "default_true")]
    open_world: bool,
}

impl Default for ToolBehaviorHints {
    fn default() -> Self {
        Self {
            read_only: false,
            destructive: true,
            idempotent: false,
            open_world: true,
        }
    }
}

const fn default_true() -> bool {
    true
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PromptContract {
    name: String,
    #[serde(default)]
    arguments: Vec<PromptArgument>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PromptArgument {
    name: String,
    #[serde(default)]
    required: bool,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ResourceContract {
    name: String,
    uri: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ResourceTemplateContract {
    name: String,
    uri_template: String,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum SnapshotInputKind {
    Malformed,
    UnsupportedVersion,
    UnsupportedRevision,
    Limit,
    ExternalReference,
    Correlation,
    RevisionMismatch,
    RevisionContract,
}

impl SnapshotInputKind {
    const fn code(self) -> &'static str {
        match self {
            Self::Malformed => "MCP-SNAPSHOT-001",
            Self::UnsupportedVersion => "MCP-SNAPSHOT-002",
            Self::UnsupportedRevision => "MCP-SNAPSHOT-003",
            Self::Limit => "MCP-SNAPSHOT-004",
            Self::ExternalReference => "MCP-SNAPSHOT-005",
            Self::Correlation => "MCP-SNAPSHOT-006",
            Self::RevisionMismatch => "MCP-SNAPSHOT-007",
            Self::RevisionContract => "MCP-SNAPSHOT-008",
        }
    }

    const fn message(self) -> &'static str {
        match self {
            Self::Malformed => "The snapshot is not a valid bounded contract artifact.",
            Self::UnsupportedVersion => "The snapshot artifact version is unsupported.",
            Self::UnsupportedRevision => "The snapshot protocol revision is unsupported.",
            Self::Limit => "The snapshot exceeds a finite artifact or schema bound.",
            Self::ExternalReference => {
                "The snapshot contains a prohibited external JSON Schema reference."
            }
            Self::Correlation => "The snapshot ordinal correlation is invalid.",
            Self::RevisionMismatch => {
                "The snapshot revision identities do not match the required scope."
            }
            Self::RevisionContract => {
                "The snapshot revision-specific artifact contract is incompatible."
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct SnapshotInputError {
    kind: SnapshotInputKind,
}

impl SnapshotInputError {
    const fn new(kind: SnapshotInputKind) -> Self {
        Self { kind }
    }
}

impl fmt::Display for SnapshotInputError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.kind.message())
    }
}

impl Error for SnapshotInputError {}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum SnapshotDestinationError {
    Authority,
    InvalidPath,
    ExistingOutput,
    ParentUnavailable,
    Content,
    Create,
    Write,
}

impl fmt::Display for SnapshotDestinationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Authority => {
                "snapshot creation requires identical --snapshot and --allow-sensitive-snapshot paths"
            }
            Self::InvalidPath => "the snapshot output path is not a valid new-file target",
            Self::ExistingOutput => "the snapshot output already exists; overwrite is not supported",
            Self::ParentUnavailable => {
                "the snapshot output parent must already exist as a directory"
            }
            Self::Content => {
                "the advertised contract cannot produce a bounded revision-correct snapshot"
            }
            Self::Create => "the new snapshot output could not be created safely",
            Self::Write => "the new snapshot output could not be written completely",
        })
    }
}

impl Error for SnapshotDestinationError {}

pub(crate) struct SnapshotDestination {
    path: PathBuf,
}

impl SnapshotDestination {
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn persist(&self, bytes: &[u8]) -> Result<(), SnapshotDestinationError> {
        let values = DiagnosticLimits::M1_DEFAULTS.values();
        if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > values.aggregate_output_bytes {
            return Err(SnapshotDestinationError::Write);
        }

        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options
            .open(&self.path)
            .map_err(|_| SnapshotDestinationError::Create)?;

        let result = file.write_all(bytes).and_then(|()| file.sync_all());
        if result.is_err() {
            drop(file);
            let _ = fs::remove_file(&self.path);
            return Err(SnapshotDestinationError::Write);
        }
        Ok(())
    }
}

pub(crate) fn prepare_snapshot_destination(
    snapshot: Option<PathBuf>,
    acknowledgement: Option<PathBuf>,
) -> Result<Option<SnapshotDestination>, SnapshotDestinationError> {
    let path = match (snapshot, acknowledgement) {
        (None, None) => return Ok(None),
        (Some(path), Some(acknowledged)) if path == acknowledged => path,
        _ => return Err(SnapshotDestinationError::Authority),
    };
    if path.file_name().is_none() {
        return Err(SnapshotDestinationError::InvalidPath);
    }
    match fs::symlink_metadata(&path) {
        Ok(_) => return Err(SnapshotDestinationError::ExistingOutput),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(_) => return Err(SnapshotDestinationError::InvalidPath),
    }
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    if !fs::metadata(parent).is_ok_and(|metadata| metadata.is_dir()) {
        return Err(SnapshotDestinationError::ParentUnavailable);
    }
    Ok(Some(SnapshotDestination { path }))
}

pub(crate) fn capture_contract_snapshot(
    revision: SupportedRevision,
    negotiated_revision: Option<KnownRevision>,
    responses: &[ProbeResponse],
) -> Result<Vec<u8>, SnapshotDestinationError> {
    let mut snapshot = snapshot_from_responses(revision, negotiated_revision, responses)
        .map_err(|_| SnapshotDestinationError::Content)?;
    normalize_and_validate(&mut snapshot).map_err(|_| SnapshotDestinationError::Content)?;
    encode_snapshot(&snapshot).map_err(|_| SnapshotDestinationError::Content)
}

fn snapshot_from_responses(
    revision: SupportedRevision,
    negotiated_revision: Option<KnownRevision>,
    responses: &[ProbeResponse],
) -> Result<ContractSnapshot, SnapshotInputError> {
    let discovery = responses
        .first()
        .ok_or_else(|| SnapshotInputError::new(SnapshotInputKind::Malformed))?;
    let discovery = response_result(discovery)?;
    let negotiated_protocol_revision =
        snapshot_revision_identity(revision, negotiated_revision, &discovery)?;
    let capabilities = discovery
        .get("capabilities")
        .and_then(Value::as_object)
        .ok_or_else(|| SnapshotInputError::new(SnapshotInputKind::Malformed))?;

    let snapshot_capabilities = SnapshotCapabilities {
        tools: list_capability(capabilities.get("tools"))?,
        prompts: list_capability(capabilities.get("prompts"))?,
        resources: resource_capability(capabilities.get("resources"))?,
        logging: legacy_presence_capability(revision, capabilities.get("logging"))?,
        completions: legacy_presence_capability(revision, capabilities.get("completions"))?,
        tasks: legacy_task_capability(revision, capabilities.get("tasks"))?,
    };

    let mut catalogs = SnapshotCatalogs {
        tools: SnapshotCatalog::default(),
        prompts: SnapshotCatalog::default(),
        resources: SnapshotCatalog::default(),
        resource_templates: SnapshotCatalog::default(),
    };
    let mut response_index = 1;
    if snapshot_capabilities.tools.advertised {
        let items = consume_catalog_pages(responses, &mut response_index, "tools")?;
        catalogs.tools = build_tool_catalog(items, revision)?;
    }
    if snapshot_capabilities.prompts.advertised {
        let items = consume_catalog_pages(responses, &mut response_index, "prompts")?;
        catalogs.prompts = build_prompt_catalog(items)?;
    }
    if snapshot_capabilities.resources.advertised {
        let resources = consume_catalog_pages(responses, &mut response_index, "resources")?;
        catalogs.resources = build_resource_catalog(resources)?;
        let templates = consume_catalog_pages(responses, &mut response_index, "resourceTemplates")?;
        catalogs.resource_templates = build_resource_template_catalog(templates)?;
    }
    if response_index != responses.len() {
        return Err(SnapshotInputError::new(SnapshotInputKind::Malformed));
    }

    Ok(ContractSnapshot {
        schema_version: SNAPSHOT_SCHEMA_VERSION.to_owned(),
        protocol_revision: revision.as_str().to_owned(),
        negotiated_protocol_revision,
        capabilities: snapshot_capabilities,
        catalogs,
    })
}

fn snapshot_revision_identity(
    revision: SupportedRevision,
    negotiated_revision: Option<KnownRevision>,
    discovery: &Map<String, Value>,
) -> Result<Option<String>, SnapshotInputError> {
    if revision.uses_initialize() {
        if negotiated_revision.map(KnownRevision::as_str) != Some(revision.as_str())
            || discovery.get("protocolVersion").and_then(Value::as_str) != Some(revision.as_str())
        {
            return Err(SnapshotInputError::new(SnapshotInputKind::RevisionMismatch));
        }
        return Ok(Some(revision.as_str().to_owned()));
    }

    if negotiated_revision.is_some() {
        return Err(SnapshotInputError::new(SnapshotInputKind::RevisionMismatch));
    }
    if discovery.get("resultType").and_then(Value::as_str) != Some("complete")
        || !discovery
            .get("supportedVersions")
            .and_then(Value::as_array)
            .is_some_and(|versions| {
                versions
                    .iter()
                    .any(|offered| offered.as_str() == Some(revision.as_str()))
            })
    {
        return Err(SnapshotInputError::new(
            SnapshotInputKind::UnsupportedRevision,
        ));
    }
    Ok(None)
}

fn legacy_presence_capability(
    revision: SupportedRevision,
    value: Option<&Value>,
) -> Result<Option<PresenceCapability>, SnapshotInputError> {
    if !revision.uses_initialize() {
        return Ok(None);
    }
    let Some(value) = value else {
        return Ok(None);
    };
    value
        .as_object()
        .ok_or_else(|| SnapshotInputError::new(SnapshotInputKind::Malformed))?;
    Ok(Some(PresenceCapability { advertised: true }))
}

fn legacy_task_capability(
    revision: SupportedRevision,
    value: Option<&Value>,
) -> Result<Option<TaskCapability>, SnapshotInputError> {
    if revision != SupportedRevision::V2025_11_25 {
        return Ok(None);
    }
    let Some(value) = value else {
        return Ok(None);
    };
    let tasks = value
        .as_object()
        .ok_or_else(|| SnapshotInputError::new(SnapshotInputKind::Malformed))?;
    let list = optional_object_present(tasks, "list")?;
    let cancel = optional_object_present(tasks, "cancel")?;
    let requests_tools_call = if let Some(requests) = optional_object(tasks, "requests")? {
        if let Some(tools) = optional_object(requests, "tools")? {
            optional_object_present(tools, "call")?
        } else {
            false
        }
    } else {
        false
    };
    Ok(Some(TaskCapability {
        advertised: true,
        list,
        cancel,
        requests_tools_call,
    }))
}

fn optional_object<'a>(
    object: &'a Map<String, Value>,
    name: &str,
) -> Result<Option<&'a Map<String, Value>>, SnapshotInputError> {
    object.get(name).map_or(Ok(None), |value| {
        value
            .as_object()
            .map(Some)
            .ok_or_else(|| SnapshotInputError::new(SnapshotInputKind::Malformed))
    })
}

fn optional_object_present(
    object: &Map<String, Value>,
    name: &str,
) -> Result<bool, SnapshotInputError> {
    optional_object(object, name).map(|value| value.is_some())
}

fn response_result(response: &ProbeResponse) -> Result<Map<String, Value>, SnapshotInputError> {
    let value: Value = serde_json::from_slice(response.as_bytes())
        .map_err(|_| SnapshotInputError::new(SnapshotInputKind::Malformed))?;
    value
        .get("result")
        .and_then(Value::as_object)
        .cloned()
        .ok_or_else(|| SnapshotInputError::new(SnapshotInputKind::Malformed))
}

fn list_capability(value: Option<&Value>) -> Result<ListCapability, SnapshotInputError> {
    let Some(value) = value else {
        return Ok(ListCapability::default());
    };
    let object = value
        .as_object()
        .ok_or_else(|| SnapshotInputError::new(SnapshotInputKind::Malformed))?;
    Ok(ListCapability {
        advertised: true,
        list_changed: optional_boolean(object, "listChanged")?,
    })
}

fn resource_capability(value: Option<&Value>) -> Result<ResourceCapability, SnapshotInputError> {
    let Some(value) = value else {
        return Ok(ResourceCapability::default());
    };
    let object = value
        .as_object()
        .ok_or_else(|| SnapshotInputError::new(SnapshotInputKind::Malformed))?;
    Ok(ResourceCapability {
        advertised: true,
        list_changed: optional_boolean(object, "listChanged")?,
        subscribe: optional_boolean(object, "subscribe")?,
    })
}

fn optional_boolean(object: &Map<String, Value>, name: &str) -> Result<bool, SnapshotInputError> {
    object.get(name).map_or(Ok(false), |value| {
        value
            .as_bool()
            .ok_or_else(|| SnapshotInputError::new(SnapshotInputKind::Malformed))
    })
}

fn consume_catalog_pages(
    responses: &[ProbeResponse],
    response_index: &mut usize,
    field: &str,
) -> Result<Vec<Value>, SnapshotInputError> {
    let mut items = Vec::new();
    loop {
        let response = responses
            .get(*response_index)
            .ok_or_else(|| SnapshotInputError::new(SnapshotInputKind::Malformed))?;
        *response_index = response_index.saturating_add(1);
        let result = response_result(response)?;
        let page = result
            .get(field)
            .and_then(Value::as_array)
            .ok_or_else(|| SnapshotInputError::new(SnapshotInputKind::Malformed))?;
        items.extend(page.iter().cloned());
        match result.get("nextCursor") {
            None => break,
            Some(cursor) if cursor.is_string() => {}
            Some(_) => return Err(SnapshotInputError::new(SnapshotInputKind::Malformed)),
        }
    }
    Ok(items)
}

fn build_tool_catalog(
    items: Vec<Value>,
    revision: SupportedRevision,
) -> Result<SnapshotCatalog<ToolContract>, SnapshotInputError> {
    let mut contracts = Vec::with_capacity(items.len());
    for item in items {
        let object = item
            .as_object()
            .ok_or_else(|| SnapshotInputError::new(SnapshotInputKind::Malformed))?;
        let name = required_string(object, "name")?;
        let input_schema = object
            .get("inputSchema")
            .cloned()
            .ok_or_else(|| SnapshotInputError::new(SnapshotInputKind::Malformed))?;
        let output_schema = object.get("outputSchema").cloned();
        let behavior_hints = tool_behavior_hints(object.get("annotations"))?;
        let input_schema_dialect = artifact_schema_dialect(revision, &input_schema)?;
        let output_schema_dialect = output_schema
            .as_ref()
            .map(|schema| artifact_schema_dialect(revision, schema))
            .transpose()?
            .flatten();
        contracts.push(ToolContract {
            name,
            behavior_hints,
            input_schema,
            input_schema_dialect,
            output_schema,
            output_schema_dialect,
        });
    }
    Ok(catalog_with_identity_correlation(contracts))
}

fn artifact_schema_dialect(
    revision: SupportedRevision,
    schema: &Value,
) -> Result<Option<SnapshotSchemaDialect>, SnapshotInputError> {
    if revision == SupportedRevision::CURRENT {
        return Ok(None);
    }
    let declared = schema.as_object().and_then(|object| object.get("$schema"));
    let dialect = match declared {
        None if revision == SupportedRevision::V2025_06_18 => SnapshotSchemaDialect::Ambiguous,
        None => SnapshotSchemaDialect::Draft2020_12,
        Some(value) if value.as_str() == Some(DRAFT_2020_12) => SnapshotSchemaDialect::Draft2020_12,
        Some(value) if value.is_string() => SnapshotSchemaDialect::Unsupported,
        Some(_) => return Err(SnapshotInputError::new(SnapshotInputKind::Malformed)),
    };
    Ok(Some(dialect))
}

fn tool_behavior_hints(value: Option<&Value>) -> Result<ToolBehaviorHints, SnapshotInputError> {
    let Some(value) = value else {
        return Ok(ToolBehaviorHints::default());
    };
    let object = value
        .as_object()
        .ok_or_else(|| SnapshotInputError::new(SnapshotInputKind::Malformed))?;
    Ok(ToolBehaviorHints {
        read_only: optional_boolean(object, "readOnlyHint")?,
        destructive: optional_boolean_with_default(object, "destructiveHint", true)?,
        idempotent: optional_boolean(object, "idempotentHint")?,
        open_world: optional_boolean_with_default(object, "openWorldHint", true)?,
    })
}

fn optional_boolean_with_default(
    object: &Map<String, Value>,
    name: &str,
    default: bool,
) -> Result<bool, SnapshotInputError> {
    object.get(name).map_or(Ok(default), |value| {
        value
            .as_bool()
            .ok_or_else(|| SnapshotInputError::new(SnapshotInputKind::Malformed))
    })
}

fn build_prompt_catalog(
    items: Vec<Value>,
) -> Result<SnapshotCatalog<PromptContract>, SnapshotInputError> {
    let mut contracts = Vec::with_capacity(items.len());
    for item in items {
        let object = item
            .as_object()
            .ok_or_else(|| SnapshotInputError::new(SnapshotInputKind::Malformed))?;
        let mut arguments = Vec::new();
        if let Some(value) = object.get("arguments") {
            let values = value
                .as_array()
                .ok_or_else(|| SnapshotInputError::new(SnapshotInputKind::Malformed))?;
            for value in values {
                let argument = value
                    .as_object()
                    .ok_or_else(|| SnapshotInputError::new(SnapshotInputKind::Malformed))?;
                arguments.push(PromptArgument {
                    name: required_string(argument, "name")?,
                    required: optional_boolean(argument, "required")?,
                });
            }
        }
        contracts.push(PromptContract {
            name: required_string(object, "name")?,
            arguments,
        });
    }
    Ok(catalog_with_identity_correlation(contracts))
}

fn build_resource_catalog(
    items: Vec<Value>,
) -> Result<SnapshotCatalog<ResourceContract>, SnapshotInputError> {
    let mut contracts = Vec::with_capacity(items.len());
    for item in items {
        let object = item
            .as_object()
            .ok_or_else(|| SnapshotInputError::new(SnapshotInputKind::Malformed))?;
        contracts.push(ResourceContract {
            name: required_string(object, "name")?,
            uri: required_string(object, "uri")?,
        });
    }
    Ok(catalog_with_identity_correlation(contracts))
}

fn build_resource_template_catalog(
    items: Vec<Value>,
) -> Result<SnapshotCatalog<ResourceTemplateContract>, SnapshotInputError> {
    let mut contracts = Vec::with_capacity(items.len());
    for item in items {
        let object = item
            .as_object()
            .ok_or_else(|| SnapshotInputError::new(SnapshotInputKind::Malformed))?;
        contracts.push(ResourceTemplateContract {
            name: required_string(object, "name")?,
            uri_template: required_string(object, "uriTemplate")?,
        });
    }
    Ok(catalog_with_identity_correlation(contracts))
}

fn required_string(object: &Map<String, Value>, name: &str) -> Result<String, SnapshotInputError> {
    object
        .get(name)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| SnapshotInputError::new(SnapshotInputKind::Malformed))
}

fn catalog_with_identity_correlation<T>(contracts: Vec<T>) -> SnapshotCatalog<T> {
    let correlation = (0..contracts.len())
        .map(|index| OrdinalCorrelation {
            discovery_ordinal: index,
            contract_index: index,
        })
        .collect();
    SnapshotCatalog {
        contracts,
        correlation,
    }
}

fn encode_snapshot(snapshot: &ContractSnapshot) -> Result<Vec<u8>, SnapshotInputError> {
    let mut bytes = serde_json::to_vec_pretty(snapshot)
        .map_err(|_| SnapshotInputError::new(SnapshotInputKind::Malformed))?;
    bytes.push(b'\n');
    let maximum = DiagnosticLimits::M1_DEFAULTS
        .values()
        .aggregate_output_bytes;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > maximum {
        return Err(SnapshotInputError::new(SnapshotInputKind::Limit));
    }
    Ok(bytes)
}

fn normalize_and_validate(snapshot: &mut ContractSnapshot) -> Result<(), SnapshotInputError> {
    if snapshot.schema_version != SNAPSHOT_SCHEMA_VERSION {
        return Err(SnapshotInputError::new(
            SnapshotInputKind::UnsupportedVersion,
        ));
    }
    let revision = supported_snapshot_revision(&snapshot.protocol_revision)
        .ok_or_else(|| SnapshotInputError::new(SnapshotInputKind::UnsupportedRevision))?;
    validate_artifact_revision_contract(snapshot, revision)?;

    let total_items = snapshot
        .catalogs
        .tools
        .contracts
        .len()
        .saturating_add(snapshot.catalogs.prompts.contracts.len())
        .saturating_add(snapshot.catalogs.resources.contracts.len())
        .saturating_add(snapshot.catalogs.resource_templates.contracts.len());
    let maximum = DiagnosticLimits::M1_DEFAULTS.values().catalog_items;
    if u64::try_from(total_items).unwrap_or(u64::MAX) > maximum {
        return Err(SnapshotInputError::new(SnapshotInputKind::Limit));
    }
    if (!snapshot.capabilities.tools.advertised
        && (!snapshot.catalogs.tools.contracts.is_empty()
            || snapshot.capabilities.tools.list_changed))
        || (!snapshot.capabilities.prompts.advertised
            && (!snapshot.catalogs.prompts.contracts.is_empty()
                || snapshot.capabilities.prompts.list_changed))
        || (!snapshot.capabilities.resources.advertised
            && (!snapshot.catalogs.resources.contracts.is_empty()
                || !snapshot.catalogs.resource_templates.contracts.is_empty()
                || snapshot.capabilities.resources.list_changed
                || snapshot.capabilities.resources.subscribe))
    {
        return Err(SnapshotInputError::new(SnapshotInputKind::Malformed));
    }

    normalize_tool_catalog(&mut snapshot.catalogs.tools, revision)?;
    normalize_prompt_catalog(&mut snapshot.catalogs.prompts)?;
    normalize_resource_catalog(&mut snapshot.catalogs.resources)?;
    normalize_resource_template_catalog(&mut snapshot.catalogs.resource_templates)?;
    Ok(())
}

fn supported_snapshot_revision(value: &str) -> Option<SupportedRevision> {
    match value {
        "2026-07-28" => Some(SupportedRevision::V2026_07_28),
        "2025-11-25" => Some(SupportedRevision::V2025_11_25),
        "2025-06-18" => Some(SupportedRevision::V2025_06_18),
        _ => None,
    }
}

fn validate_artifact_revision_contract(
    snapshot: &ContractSnapshot,
    revision: SupportedRevision,
) -> Result<(), SnapshotInputError> {
    let negotiated = snapshot.negotiated_protocol_revision.as_deref();
    if revision.uses_initialize() {
        if negotiated != Some(revision.as_str()) {
            return Err(SnapshotInputError::new(SnapshotInputKind::RevisionMismatch));
        }
    } else if negotiated.is_some() {
        return Err(SnapshotInputError::new(SnapshotInputKind::RevisionMismatch));
    }

    for capability in [
        snapshot.capabilities.logging.as_ref(),
        snapshot.capabilities.completions.as_ref(),
    ] {
        if capability.is_some_and(|capability| !capability.advertised) {
            return Err(SnapshotInputError::new(SnapshotInputKind::RevisionContract));
        }
    }
    if !revision.uses_initialize()
        && (snapshot.capabilities.logging.is_some() || snapshot.capabilities.completions.is_some())
    {
        return Err(SnapshotInputError::new(SnapshotInputKind::RevisionContract));
    }
    if let Some(tasks) = &snapshot.capabilities.tasks
        && (revision != SupportedRevision::V2025_11_25 || !tasks.advertised)
    {
        return Err(SnapshotInputError::new(SnapshotInputKind::RevisionContract));
    }
    Ok(())
}

fn normalize_tool_catalog(
    catalog: &mut SnapshotCatalog<ToolContract>,
    revision: SupportedRevision,
) -> Result<(), SnapshotInputError> {
    validate_correlation(catalog.contracts.len(), &catalog.correlation)?;
    let mut names = BTreeSet::new();
    for contract in &mut catalog.contracts {
        if !names.insert(contract.name.clone()) {
            return Err(SnapshotInputError::new(SnapshotInputKind::Malformed));
        }
        validate_artifact_schema_dialect(
            revision,
            &contract.input_schema,
            contract.input_schema_dialect,
        )?;
        contract.input_schema = normalize_schema(&contract.input_schema)?;
        if let Some(schema) = &contract.output_schema {
            validate_artifact_schema_dialect(revision, schema, contract.output_schema_dialect)?;
            contract.output_schema = Some(normalize_schema(schema)?);
        } else if contract.output_schema_dialect.is_some() {
            return Err(SnapshotInputError::new(SnapshotInputKind::RevisionContract));
        }
    }
    sort_catalog(catalog, |left, right| left.name.cmp(&right.name));
    Ok(())
}

fn validate_artifact_schema_dialect(
    revision: SupportedRevision,
    schema: &Value,
    stored: Option<SnapshotSchemaDialect>,
) -> Result<(), SnapshotInputError> {
    let expected = artifact_schema_dialect(revision, schema)?;
    if stored != expected {
        return Err(SnapshotInputError::new(SnapshotInputKind::RevisionContract));
    }
    Ok(())
}

fn normalize_prompt_catalog(
    catalog: &mut SnapshotCatalog<PromptContract>,
) -> Result<(), SnapshotInputError> {
    validate_correlation(catalog.contracts.len(), &catalog.correlation)?;
    let mut names = BTreeSet::new();
    for contract in &mut catalog.contracts {
        if !names.insert(contract.name.clone()) {
            return Err(SnapshotInputError::new(SnapshotInputKind::Malformed));
        }
        contract
            .arguments
            .sort_by(|left, right| left.name.cmp(&right.name));
        if u64::try_from(contract.arguments.len()).unwrap_or(u64::MAX)
            > DiagnosticLimits::M1_DEFAULTS.values().catalog_items
        {
            return Err(SnapshotInputError::new(SnapshotInputKind::Limit));
        }
        if contract
            .arguments
            .windows(2)
            .any(|pair| pair[0].name == pair[1].name)
        {
            return Err(SnapshotInputError::new(SnapshotInputKind::Malformed));
        }
    }
    sort_catalog(catalog, |left, right| left.name.cmp(&right.name));
    Ok(())
}

fn normalize_resource_catalog(
    catalog: &mut SnapshotCatalog<ResourceContract>,
) -> Result<(), SnapshotInputError> {
    validate_correlation(catalog.contracts.len(), &catalog.correlation)?;
    let mut names = BTreeSet::new();
    let mut uris = BTreeSet::new();
    for contract in &catalog.contracts {
        if !names.insert(contract.name.clone()) || !uris.insert(contract.uri.clone()) {
            return Err(SnapshotInputError::new(SnapshotInputKind::Malformed));
        }
    }
    sort_catalog(catalog, |left, right| left.uri.cmp(&right.uri));
    Ok(())
}

fn normalize_resource_template_catalog(
    catalog: &mut SnapshotCatalog<ResourceTemplateContract>,
) -> Result<(), SnapshotInputError> {
    validate_correlation(catalog.contracts.len(), &catalog.correlation)?;
    let mut names = BTreeSet::new();
    let mut templates = BTreeSet::new();
    for contract in &catalog.contracts {
        if !names.insert(contract.name.clone()) || !templates.insert(contract.uri_template.clone())
        {
            return Err(SnapshotInputError::new(SnapshotInputKind::Malformed));
        }
    }
    sort_catalog(catalog, |left, right| {
        left.uri_template.cmp(&right.uri_template)
    });
    Ok(())
}

fn validate_correlation(
    contract_count: usize,
    correlation: &[OrdinalCorrelation],
) -> Result<(), SnapshotInputError> {
    if correlation.len() != contract_count {
        return Err(SnapshotInputError::new(SnapshotInputKind::Correlation));
    }
    let mut ordinals = vec![false; contract_count];
    let mut indexes = vec![false; contract_count];
    for entry in correlation {
        let Some(ordinal) = ordinals.get_mut(entry.discovery_ordinal) else {
            return Err(SnapshotInputError::new(SnapshotInputKind::Correlation));
        };
        let Some(index) = indexes.get_mut(entry.contract_index) else {
            return Err(SnapshotInputError::new(SnapshotInputKind::Correlation));
        };
        if *ordinal || *index {
            return Err(SnapshotInputError::new(SnapshotInputKind::Correlation));
        }
        *ordinal = true;
        *index = true;
    }
    Ok(())
}

fn sort_catalog<T>(catalog: &mut SnapshotCatalog<T>, compare: impl Fn(&T, &T) -> Ordering) {
    let mut ordinal_by_old_index = vec![0; catalog.contracts.len()];
    for entry in &catalog.correlation {
        ordinal_by_old_index[entry.contract_index] = entry.discovery_ordinal;
    }
    let mut indexed = catalog.contracts.drain(..).enumerate().collect::<Vec<_>>();
    indexed.sort_by(|(_, left), (_, right)| compare(left, right));
    catalog.correlation.clear();
    for (contract_index, (old_index, contract)) in indexed.into_iter().enumerate() {
        catalog.contracts.push(contract);
        catalog.correlation.push(OrdinalCorrelation {
            discovery_ordinal: ordinal_by_old_index[old_index],
            contract_index,
        });
    }
    catalog
        .correlation
        .sort_by_key(|entry| entry.discovery_ordinal);
}

fn normalize_schema(schema: &Value) -> Result<Value, SnapshotInputError> {
    validate_schema(schema)?;
    normalize_schema_value(schema)
}

fn normalize_schema_value(schema: &Value) -> Result<Value, SnapshotInputError> {
    match schema {
        Value::Bool(value) => Ok(Value::Bool(*value)),
        Value::Object(object) => {
            let mut normalized = Map::new();
            for (key, value) in object {
                if matches!(
                    key.as_str(),
                    "title" | "description" | "default" | "examples" | "$comment"
                ) {
                    continue;
                }
                let value = match key.as_str() {
                    "$defs" | "definitions" | "properties" | "patternProperties"
                    | "dependentSchemas" => normalize_schema_map(value)?,
                    "items"
                    | "contains"
                    | "additionalProperties"
                    | "propertyNames"
                    | "not"
                    | "if"
                    | "then"
                    | "else"
                    | "unevaluatedProperties"
                    | "unevaluatedItems"
                    | "contentSchema" => normalize_schema_or_literal(value)?,
                    "prefixItems" => normalize_schema_array(value, false)?,
                    "allOf" | "anyOf" | "oneOf" => normalize_schema_array(value, true)?,
                    "required" | "type" | "enum" => normalize_set_array(value),
                    "dependentRequired" => normalize_string_array_map(value),
                    _ => canonical_literal(value),
                };
                normalized.insert(key.clone(), value);
            }
            Ok(Value::Object(normalized))
        }
        _ => Ok(canonical_literal(schema)),
    }
}

fn normalize_schema_map(value: &Value) -> Result<Value, SnapshotInputError> {
    let Some(object) = value.as_object() else {
        return Ok(canonical_literal(value));
    };
    let mut normalized = Map::new();
    for (key, schema) in object {
        normalized.insert(key.clone(), normalize_schema_or_literal(schema)?);
    }
    Ok(Value::Object(normalized))
}

fn normalize_schema_or_literal(value: &Value) -> Result<Value, SnapshotInputError> {
    if value.is_boolean() || value.is_object() {
        normalize_schema_value(value)
    } else {
        Ok(canonical_literal(value))
    }
}

fn normalize_schema_array(value: &Value, sort: bool) -> Result<Value, SnapshotInputError> {
    let Some(values) = value.as_array() else {
        return Ok(canonical_literal(value));
    };
    let mut normalized = values
        .iter()
        .map(normalize_schema_or_literal)
        .collect::<Result<Vec<_>, _>>()?;
    if sort {
        normalized.sort_by_key(canonical_key);
    }
    Ok(Value::Array(normalized))
}

fn normalize_set_array(value: &Value) -> Value {
    let Some(values) = value.as_array() else {
        return canonical_literal(value);
    };
    let mut normalized = values.iter().map(canonical_literal).collect::<Vec<_>>();
    normalized.sort_by_key(canonical_key);
    normalized.dedup();
    Value::Array(normalized)
}

fn normalize_string_array_map(value: &Value) -> Value {
    let Some(object) = value.as_object() else {
        return canonical_literal(value);
    };
    let mut normalized = Map::new();
    for (key, value) in object {
        normalized.insert(key.clone(), normalize_set_array(value));
    }
    Value::Object(normalized)
}

fn canonical_literal(value: &Value) -> Value {
    match value {
        Value::Array(values) => Value::Array(values.iter().map(canonical_literal).collect()),
        Value::Object(object) => Value::Object(
            object
                .iter()
                .map(|(key, value)| (key.clone(), canonical_literal(value)))
                .collect(),
        ),
        _ => value.clone(),
    }
}

fn canonical_key(value: &Value) -> String {
    serde_json::to_string(value).expect("a JSON value must serialize")
}

fn validate_schema(schema: &Value) -> Result<(), SnapshotInputError> {
    let values = DiagnosticLimits::M1_DEFAULTS.values();
    let bytes = serde_json::to_vec(schema)
        .map_err(|_| SnapshotInputError::new(SnapshotInputKind::Malformed))?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > values.schema_bytes {
        return Err(SnapshotInputError::new(SnapshotInputKind::Limit));
    }
    if schema
        .as_object()
        .and_then(|object| object.get("$schema"))
        .is_some_and(|dialect| !dialect.is_string())
    {
        return Err(SnapshotInputError::new(SnapshotInputKind::Malformed));
    }

    let mut stack = vec![(schema, 0_u64)];
    let mut nodes = 0_u64;
    let mut references = Vec::new();
    while let Some((value, depth)) = stack.pop() {
        nodes = nodes.saturating_add(1);
        if nodes > values.schema_nodes || depth > values.schema_depth {
            return Err(SnapshotInputError::new(SnapshotInputKind::Limit));
        }
        match value {
            Value::Array(items) => {
                stack.extend(items.iter().map(|item| (item, depth.saturating_add(1))))
            }
            Value::Object(fields) => {
                for (key, value) in fields {
                    if matches!(key.as_str(), "$ref" | "$dynamicRef") {
                        let reference = value
                            .as_str()
                            .ok_or_else(|| SnapshotInputError::new(SnapshotInputKind::Malformed))?;
                        if !(reference.is_empty() || reference.starts_with('#')) {
                            return Err(SnapshotInputError::new(
                                SnapshotInputKind::ExternalReference,
                            ));
                        }
                        references.push(reference.to_owned());
                    }
                    stack.push((value, depth.saturating_add(1)));
                }
            }
            _ => {}
        }
    }
    validate_reference_depth(schema, &references, values.schema_ref_depth)?;
    Ok(())
}

fn validate_reference_depth(
    schema: &Value,
    references: &[String],
    maximum: u64,
) -> Result<(), SnapshotInputError> {
    let mut work = 0_u64;
    let maximum_work = DiagnosticLimits::M1_DEFAULTS
        .values()
        .schema_evaluation_steps;
    for reference in references {
        let mut active = BTreeSet::new();
        follow_reference(
            schema,
            reference,
            1,
            maximum,
            &mut active,
            &mut work,
            maximum_work,
        )?;
    }
    Ok(())
}

fn follow_reference(
    schema: &Value,
    reference: &str,
    depth: u64,
    maximum_depth: u64,
    active: &mut BTreeSet<String>,
    work: &mut u64,
    maximum_work: u64,
) -> Result<(), SnapshotInputError> {
    *work = work.saturating_add(1);
    if *work > maximum_work || depth > maximum_depth {
        return Err(SnapshotInputError::new(SnapshotInputKind::Limit));
    }
    if !active.insert(reference.to_owned()) {
        return Ok(());
    }
    let Some(target) = resolve_local_reference(schema, reference) else {
        active.remove(reference);
        return Ok(());
    };
    let mut nested = Vec::new();
    collect_references(target, &mut nested, work, maximum_work)?;
    for reference in nested {
        follow_reference(
            schema,
            &reference,
            depth.saturating_add(1),
            maximum_depth,
            active,
            work,
            maximum_work,
        )?;
    }
    active.remove(reference);
    Ok(())
}

fn collect_references(
    value: &Value,
    references: &mut Vec<String>,
    work: &mut u64,
    maximum_work: u64,
) -> Result<(), SnapshotInputError> {
    let mut stack = vec![value];
    while let Some(value) = stack.pop() {
        *work = work.saturating_add(1);
        if *work > maximum_work {
            return Err(SnapshotInputError::new(SnapshotInputKind::Limit));
        }
        match value {
            Value::Array(values) => stack.extend(values),
            Value::Object(object) => {
                for (key, value) in object {
                    if matches!(key.as_str(), "$ref" | "$dynamicRef")
                        && let Some(reference) = value.as_str()
                    {
                        references.push(reference.to_owned());
                    }
                    if !matches!(key.as_str(), "$defs" | "definitions") {
                        stack.push(value);
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn read_snapshot(path: &Path) -> Result<ContractSnapshot, SnapshotInputError> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|_| SnapshotInputError::new(SnapshotInputKind::Malformed))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(SnapshotInputError::new(SnapshotInputKind::Malformed));
    }
    let maximum = DiagnosticLimits::M1_DEFAULTS
        .values()
        .aggregate_output_bytes;
    if metadata.len() > maximum {
        return Err(SnapshotInputError::new(SnapshotInputKind::Limit));
    }
    let file =
        File::open(path).map_err(|_| SnapshotInputError::new(SnapshotInputKind::Malformed))?;
    let mut bytes = Vec::new();
    file.take(maximum.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|_| SnapshotInputError::new(SnapshotInputKind::Malformed))?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > maximum {
        return Err(SnapshotInputError::new(SnapshotInputKind::Limit));
    }
    let value: Value = serde_json::from_slice(&bytes)
        .map_err(|_| SnapshotInputError::new(SnapshotInputKind::Malformed))?;
    let version = value
        .get("schema_version")
        .and_then(Value::as_str)
        .ok_or_else(|| SnapshotInputError::new(SnapshotInputKind::Malformed))?;
    if version != SNAPSHOT_SCHEMA_VERSION {
        return Err(SnapshotInputError::new(
            SnapshotInputKind::UnsupportedVersion,
        ));
    }
    let revision = value
        .get("protocol_revision")
        .and_then(Value::as_str)
        .ok_or_else(|| SnapshotInputError::new(SnapshotInputKind::Malformed))?;
    let revision = supported_snapshot_revision(revision)
        .ok_or_else(|| SnapshotInputError::new(SnapshotInputKind::UnsupportedRevision))?;
    validate_raw_revision_contract(&value, revision)?;
    let mut snapshot: ContractSnapshot = serde_json::from_value(value)
        .map_err(|_| SnapshotInputError::new(SnapshotInputKind::Malformed))?;
    normalize_and_validate(&mut snapshot)?;
    Ok(snapshot)
}

fn validate_raw_revision_contract(
    value: &Value,
    revision: SupportedRevision,
) -> Result<(), SnapshotInputError> {
    let root = value
        .as_object()
        .ok_or_else(|| SnapshotInputError::new(SnapshotInputKind::Malformed))?;
    if !revision.uses_initialize() && root.contains_key("negotiated_protocol_revision") {
        return Err(SnapshotInputError::new(SnapshotInputKind::RevisionMismatch));
    }

    let capabilities = root
        .get("capabilities")
        .and_then(Value::as_object)
        .ok_or_else(|| SnapshotInputError::new(SnapshotInputKind::Malformed))?;
    for name in ["logging", "completions", "tasks"] {
        if capabilities.get(name).is_some_and(Value::is_null) {
            return Err(SnapshotInputError::new(SnapshotInputKind::RevisionContract));
        }
    }
    if !revision.uses_initialize()
        && ["logging", "completions", "tasks"]
            .iter()
            .any(|name| capabilities.contains_key(*name))
    {
        return Err(SnapshotInputError::new(SnapshotInputKind::RevisionContract));
    }
    if revision != SupportedRevision::V2025_11_25 && capabilities.contains_key("tasks") {
        return Err(SnapshotInputError::new(SnapshotInputKind::RevisionContract));
    }

    let tools = root
        .get("catalogs")
        .and_then(Value::as_object)
        .and_then(|catalogs| catalogs.get("tools"))
        .and_then(Value::as_object)
        .and_then(|tools| tools.get("contracts"))
        .and_then(Value::as_array)
        .ok_or_else(|| SnapshotInputError::new(SnapshotInputKind::Malformed))?;
    for tool in tools {
        let tool = tool
            .as_object()
            .ok_or_else(|| SnapshotInputError::new(SnapshotInputKind::Malformed))?;
        let input_dialect = tool.get("input_schema_dialect");
        let output_dialect = tool.get("output_schema_dialect");
        if input_dialect.is_some_and(Value::is_null) || output_dialect.is_some_and(Value::is_null) {
            return Err(SnapshotInputError::new(SnapshotInputKind::RevisionContract));
        }
        if !revision.uses_initialize() && (input_dialect.is_some() || output_dialect.is_some()) {
            return Err(SnapshotInputError::new(SnapshotInputKind::RevisionContract));
        }
        if revision.uses_initialize()
            && (input_dialect.is_none()
                || tool.contains_key("output_schema") != output_dialect.is_some())
        {
            return Err(SnapshotInputError::new(SnapshotInputKind::RevisionContract));
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum DiffFormat {
    Human,
    Json,
}

#[derive(Serialize)]
struct ContractDiffReport {
    schema_version: &'static str,
    protocol_revision: Option<&'static str>,
    outcome: DiffOutcome,
    exit_code: u8,
    summary: DiffSummary,
    checks: Vec<DiffCheck>,
    findings: Vec<DiffFinding>,
}

#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
enum DiffOutcome {
    Unchanged,
    Compatible,
    ReviewRequired,
    PotentiallyBreaking,
    Invalid,
}

#[derive(Serialize)]
struct DiffSummary {
    compatible: usize,
    potentially_breaking: usize,
    review_required: usize,
    invalid: usize,
    total: usize,
}

#[derive(Serialize)]
struct DiffCheck {
    id: &'static str,
    state: DiffCheckState,
    #[serde(skip_serializing_if = "Option::is_none")]
    blocked_by: Option<&'static str>,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
enum DiffCheckState {
    Performed,
    Skipped,
}

#[derive(Clone, Serialize)]
struct DiffFinding {
    code: &'static str,
    classification: DiffClassification,
    change: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    catalog: Option<CatalogLabel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    before_ordinal: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    after_ordinal: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    input: Option<DiffInput>,
}

#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
enum DiffClassification {
    Compatible,
    PotentiallyBreaking,
    ReviewRequired,
    Invalid,
}

#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
enum CatalogLabel {
    Tools,
    Prompts,
    Resources,
    ResourceTemplates,
    Capabilities,
}

#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
enum DiffInput {
    Before,
    After,
}

pub(crate) struct RenderedContractDiff {
    pub(crate) output: String,
    pub(crate) exit: ExitCode,
    pub(crate) error: Option<String>,
}

pub(crate) fn render_contract_diff(
    before: &Path,
    after: &Path,
    format: DiffFormat,
) -> RenderedContractDiff {
    let before = match read_snapshot(before) {
        Ok(snapshot) => snapshot,
        Err(error) => return render_invalid_diff(error, Some(DiffInput::Before), format),
    };
    let after = match read_snapshot(after) {
        Ok(snapshot) => snapshot,
        Err(error) => return render_invalid_diff(error, Some(DiffInput::After), format),
    };
    if before.protocol_revision != after.protocol_revision {
        return render_invalid_diff(
            SnapshotInputError::new(SnapshotInputKind::RevisionMismatch),
            None,
            format,
        );
    }
    let report = compare_snapshots(&before, &after);
    render_diff_report(report, format)
}

fn render_invalid_diff(
    error: SnapshotInputError,
    input: Option<DiffInput>,
    format: DiffFormat,
) -> RenderedContractDiff {
    let finding = DiffFinding {
        code: error.kind.code(),
        classification: DiffClassification::Invalid,
        change: "artifact_invalid",
        catalog: None,
        before_ordinal: None,
        after_ordinal: None,
        input,
    };
    let report = ContractDiffReport {
        schema_version: DIFF_SCHEMA_VERSION,
        protocol_revision: None,
        outcome: DiffOutcome::Invalid,
        exit_code: 2,
        summary: summarize_findings(std::slice::from_ref(&finding)),
        checks: vec![
            performed_check("artifact_validation"),
            skipped_check("normalization", "artifact_validation"),
            skipped_check("catalog_comparison", "artifact_validation"),
            skipped_check("schema_comparison", "artifact_validation"),
        ],
        findings: vec![finding],
    };
    render_diff_report(report, format)
}

fn performed_check(id: &'static str) -> DiffCheck {
    DiffCheck {
        id,
        state: DiffCheckState::Performed,
        blocked_by: None,
    }
}

fn skipped_check(id: &'static str, blocked_by: &'static str) -> DiffCheck {
    DiffCheck {
        id,
        state: DiffCheckState::Skipped,
        blocked_by: Some(blocked_by),
    }
}

fn compare_snapshots(before: &ContractSnapshot, after: &ContractSnapshot) -> ContractDiffReport {
    let revision = supported_snapshot_revision(&before.protocol_revision)
        .expect("validated snapshots have a supported revision");
    let mut findings = Vec::new();
    compare_capabilities(&before.capabilities, &after.capabilities, &mut findings);
    compare_tools(&before.catalogs.tools, &after.catalogs.tools, &mut findings);
    compare_prompts(
        &before.catalogs.prompts,
        &after.catalogs.prompts,
        &mut findings,
    );
    compare_resources(
        &before.catalogs.resources,
        &after.catalogs.resources,
        &mut findings,
    );
    compare_resource_templates(
        &before.catalogs.resource_templates,
        &after.catalogs.resource_templates,
        &mut findings,
    );
    if diff_finding_limit_reached(&findings) {
        return comparison_limit_report(revision);
    }
    findings.sort_by(finding_order);
    let summary = summarize_findings(&findings);
    let outcome = if summary.potentially_breaking > 0 {
        DiffOutcome::PotentiallyBreaking
    } else if summary.review_required > 0 {
        DiffOutcome::ReviewRequired
    } else if summary.compatible > 0 {
        DiffOutcome::Compatible
    } else {
        DiffOutcome::Unchanged
    };
    let exit_code = u8::from(!matches!(
        outcome,
        DiffOutcome::Unchanged | DiffOutcome::Compatible
    ));
    ContractDiffReport {
        schema_version: DIFF_SCHEMA_VERSION,
        protocol_revision: Some(revision.as_str()),
        outcome,
        exit_code,
        summary,
        checks: vec![
            performed_check("artifact_validation"),
            performed_check("normalization"),
            performed_check("catalog_comparison"),
            performed_check("schema_comparison"),
        ],
        findings,
    }
}

fn maximum_diff_findings() -> usize {
    usize::try_from(DiagnosticLimits::M1_DEFAULTS.values().report_findings).unwrap_or(usize::MAX)
}

fn push_diff_finding(findings: &mut Vec<DiffFinding>, finding: DiffFinding) {
    if findings.len() <= maximum_diff_findings() {
        findings.push(finding);
    }
}

fn diff_finding_limit_reached(findings: &[DiffFinding]) -> bool {
    findings.len() > maximum_diff_findings()
}

fn comparison_limit_report(revision: SupportedRevision) -> ContractDiffReport {
    let finding = DiffFinding {
        code: "MCP-SNAPSHOT-004",
        classification: DiffClassification::Invalid,
        change: "comparison_limit",
        catalog: None,
        before_ordinal: None,
        after_ordinal: None,
        input: None,
    };
    ContractDiffReport {
        schema_version: DIFF_SCHEMA_VERSION,
        protocol_revision: Some(revision.as_str()),
        outcome: DiffOutcome::Invalid,
        exit_code: 2,
        summary: summarize_findings(std::slice::from_ref(&finding)),
        checks: vec![
            performed_check("artifact_validation"),
            performed_check("normalization"),
            performed_check("catalog_comparison"),
            performed_check("schema_comparison"),
        ],
        findings: vec![finding],
    }
}

fn summarize_findings(findings: &[DiffFinding]) -> DiffSummary {
    let compatible = findings
        .iter()
        .filter(|finding| finding.classification == DiffClassification::Compatible)
        .count();
    let potentially_breaking = findings
        .iter()
        .filter(|finding| finding.classification == DiffClassification::PotentiallyBreaking)
        .count();
    let review_required = findings
        .iter()
        .filter(|finding| finding.classification == DiffClassification::ReviewRequired)
        .count();
    let invalid = findings
        .iter()
        .filter(|finding| finding.classification == DiffClassification::Invalid)
        .count();
    DiffSummary {
        compatible,
        potentially_breaking,
        review_required,
        invalid,
        total: findings.len(),
    }
}

fn finding_order(left: &DiffFinding, right: &DiffFinding) -> Ordering {
    left.catalog
        .cmp(&right.catalog)
        .then_with(|| left.before_ordinal.cmp(&right.before_ordinal))
        .then_with(|| left.after_ordinal.cmp(&right.after_ordinal))
        .then_with(|| left.code.cmp(right.code))
}

fn compare_capabilities(
    before: &SnapshotCapabilities,
    after: &SnapshotCapabilities,
    findings: &mut Vec<DiffFinding>,
) {
    for (old, new) in [
        (before.tools.advertised, after.tools.advertised),
        (before.tools.list_changed, after.tools.list_changed),
        (before.prompts.advertised, after.prompts.advertised),
        (before.prompts.list_changed, after.prompts.list_changed),
        (before.resources.advertised, after.resources.advertised),
        (before.resources.list_changed, after.resources.list_changed),
        (before.resources.subscribe, after.resources.subscribe),
        (
            presence_advertised(before.logging.as_ref()),
            presence_advertised(after.logging.as_ref()),
        ),
        (
            presence_advertised(before.completions.as_ref()),
            presence_advertised(after.completions.as_ref()),
        ),
        (
            task_setting(before.tasks.as_ref(), |tasks| tasks.advertised),
            task_setting(after.tasks.as_ref(), |tasks| tasks.advertised),
        ),
        (
            task_setting(before.tasks.as_ref(), |tasks| tasks.list),
            task_setting(after.tasks.as_ref(), |tasks| tasks.list),
        ),
        (
            task_setting(before.tasks.as_ref(), |tasks| tasks.cancel),
            task_setting(after.tasks.as_ref(), |tasks| tasks.cancel),
        ),
        (
            task_setting(before.tasks.as_ref(), |tasks| tasks.requests_tools_call),
            task_setting(after.tasks.as_ref(), |tasks| tasks.requests_tools_call),
        ),
    ] {
        if old != new {
            push_diff_finding(
                findings,
                if new {
                    compatible_finding(
                        "MCP-DIFF-003",
                        "capability_enabled",
                        CatalogLabel::Capabilities,
                    )
                } else {
                    breaking_finding(
                        "MCP-DIFF-004",
                        "capability_disabled",
                        CatalogLabel::Capabilities,
                    )
                },
            );
        }
    }
}

fn presence_advertised(capability: Option<&PresenceCapability>) -> bool {
    capability.is_some_and(|capability| capability.advertised)
}

fn task_setting(
    capability: Option<&TaskCapability>,
    select: impl FnOnce(&TaskCapability) -> bool,
) -> bool {
    capability.is_some_and(select)
}

fn compatible_finding(
    code: &'static str,
    change: &'static str,
    catalog: CatalogLabel,
) -> DiffFinding {
    DiffFinding {
        code,
        classification: DiffClassification::Compatible,
        change,
        catalog: Some(catalog),
        before_ordinal: None,
        after_ordinal: None,
        input: None,
    }
}

fn breaking_finding(
    code: &'static str,
    change: &'static str,
    catalog: CatalogLabel,
) -> DiffFinding {
    DiffFinding {
        code,
        classification: DiffClassification::PotentiallyBreaking,
        change,
        catalog: Some(catalog),
        before_ordinal: None,
        after_ordinal: None,
        input: None,
    }
}

fn review_finding(change: &'static str, catalog: CatalogLabel) -> DiffFinding {
    DiffFinding {
        code: "MCP-DIFF-009",
        classification: DiffClassification::ReviewRequired,
        change,
        catalog: Some(catalog),
        before_ordinal: None,
        after_ordinal: None,
        input: None,
    }
}

fn ordinal<T>(catalog: &SnapshotCatalog<T>, index: usize) -> Option<usize> {
    catalog
        .correlation
        .iter()
        .find(|entry| entry.contract_index == index)
        .map(|entry| entry.discovery_ordinal)
}

fn mark_before_after<T, U>(
    mut finding: DiffFinding,
    before: Option<(&SnapshotCatalog<T>, usize)>,
    after: Option<(&SnapshotCatalog<U>, usize)>,
) -> DiffFinding {
    finding.before_ordinal = before.and_then(|(catalog, index)| ordinal(catalog, index));
    finding.after_ordinal = after.and_then(|(catalog, index)| ordinal(catalog, index));
    finding
}

fn compare_tools(
    before: &SnapshotCatalog<ToolContract>,
    after: &SnapshotCatalog<ToolContract>,
    findings: &mut Vec<DiffFinding>,
) {
    let before_by_name = before
        .contracts
        .iter()
        .enumerate()
        .map(|(index, contract)| (contract.name.as_str(), (index, contract)))
        .collect::<BTreeMap<_, _>>();
    let after_by_name = after
        .contracts
        .iter()
        .enumerate()
        .map(|(index, contract)| (contract.name.as_str(), (index, contract)))
        .collect::<BTreeMap<_, _>>();
    compare_catalog_membership(
        &before_by_name,
        &after_by_name,
        before,
        after,
        CatalogLabel::Tools,
        findings,
    );
    for name in before_by_name
        .keys()
        .filter(|name| after_by_name.contains_key(*name))
    {
        let (before_index, old) = before_by_name[name];
        let (after_index, new) = after_by_name[name];
        if old.behavior_hints != new.behavior_hints {
            let finding = DiffFinding {
                code: "MCP-DIFF-010",
                classification: DiffClassification::ReviewRequired,
                change: "behavior_hint_changed",
                catalog: Some(CatalogLabel::Tools),
                before_ordinal: None,
                after_ordinal: None,
                input: None,
            };
            push_diff_finding(
                findings,
                mark_before_after(
                    finding,
                    Some((before, before_index)),
                    Some((after, after_index)),
                ),
            );
        }
        let changes = classify_input_schema_with_dialects(
            &old.input_schema,
            &new.input_schema,
            old.input_schema_dialect,
            new.input_schema_dialect,
        );
        for change in changes {
            let finding = schema_finding(change, CatalogLabel::Tools);
            push_diff_finding(
                findings,
                mark_before_after(
                    finding,
                    Some((before, before_index)),
                    Some((after, after_index)),
                ),
            );
        }
        if old.output_schema != new.output_schema {
            let finding = review_finding("output_schema_changed", CatalogLabel::Tools);
            push_diff_finding(
                findings,
                mark_before_after(
                    finding,
                    Some((before, before_index)),
                    Some((after, after_index)),
                ),
            );
        }
    }
}

fn compare_catalog_membership<'a, T, U>(
    before_by_identity: &BTreeMap<&'a str, (usize, &'a T)>,
    after_by_identity: &BTreeMap<&'a str, (usize, &'a U)>,
    before: &SnapshotCatalog<T>,
    after: &SnapshotCatalog<U>,
    label: CatalogLabel,
    findings: &mut Vec<DiffFinding>,
) {
    for identity in before_by_identity
        .keys()
        .filter(|identity| !after_by_identity.contains_key(*identity))
    {
        let (index, _) = before_by_identity[identity];
        push_diff_finding(
            findings,
            mark_before_after::<T, U>(
                breaking_finding("MCP-DIFF-002", "contract_removed", label),
                Some((before, index)),
                None,
            ),
        );
    }
    for identity in after_by_identity
        .keys()
        .filter(|identity| !before_by_identity.contains_key(*identity))
    {
        let (index, _) = after_by_identity[identity];
        push_diff_finding(
            findings,
            mark_before_after::<T, U>(
                compatible_finding("MCP-DIFF-001", "contract_added", label),
                None,
                Some((after, index)),
            ),
        );
    }
}

fn compare_prompts(
    before: &SnapshotCatalog<PromptContract>,
    after: &SnapshotCatalog<PromptContract>,
    findings: &mut Vec<DiffFinding>,
) {
    let before_by_name = before
        .contracts
        .iter()
        .enumerate()
        .map(|(index, contract)| (contract.name.as_str(), (index, contract)))
        .collect::<BTreeMap<_, _>>();
    let after_by_name = after
        .contracts
        .iter()
        .enumerate()
        .map(|(index, contract)| (contract.name.as_str(), (index, contract)))
        .collect::<BTreeMap<_, _>>();
    compare_catalog_membership(
        &before_by_name,
        &after_by_name,
        before,
        after,
        CatalogLabel::Prompts,
        findings,
    );
    for name in before_by_name
        .keys()
        .filter(|name| after_by_name.contains_key(*name))
    {
        let (before_index, old) = before_by_name[name];
        let (after_index, new) = after_by_name[name];
        let old_required = old
            .arguments
            .iter()
            .filter(|argument| argument.required)
            .map(|argument| argument.name.as_str())
            .collect::<BTreeSet<_>>();
        let new_required = new
            .arguments
            .iter()
            .filter(|argument| argument.required)
            .map(|argument| argument.name.as_str())
            .collect::<BTreeSet<_>>();
        if new_required.difference(&old_required).next().is_some() {
            push_diff_finding(
                findings,
                mark_before_after(
                    breaking_finding(
                        "MCP-DIFF-005",
                        "required_input_added",
                        CatalogLabel::Prompts,
                    ),
                    Some((before, before_index)),
                    Some((after, after_index)),
                ),
            );
        }
        if old_required.difference(&new_required).next().is_some() {
            push_diff_finding(
                findings,
                mark_before_after(
                    compatible_finding(
                        "MCP-DIFF-006",
                        "required_input_removed",
                        CatalogLabel::Prompts,
                    ),
                    Some((before, before_index)),
                    Some((after, after_index)),
                ),
            );
        }
        let old_arguments = old
            .arguments
            .iter()
            .map(|argument| argument.name.as_str())
            .collect::<BTreeSet<_>>();
        let new_arguments = new
            .arguments
            .iter()
            .map(|argument| argument.name.as_str())
            .collect::<BTreeSet<_>>();
        if old_arguments != new_arguments && old_required == new_required {
            push_diff_finding(
                findings,
                mark_before_after(
                    review_finding("prompt_arguments_changed", CatalogLabel::Prompts),
                    Some((before, before_index)),
                    Some((after, after_index)),
                ),
            );
        }
    }
}

fn compare_resources(
    before: &SnapshotCatalog<ResourceContract>,
    after: &SnapshotCatalog<ResourceContract>,
    findings: &mut Vec<DiffFinding>,
) {
    let before_by_uri = before
        .contracts
        .iter()
        .enumerate()
        .map(|(index, contract)| (contract.uri.as_str(), (index, contract)))
        .collect::<BTreeMap<_, _>>();
    let after_by_uri = after
        .contracts
        .iter()
        .enumerate()
        .map(|(index, contract)| (contract.uri.as_str(), (index, contract)))
        .collect::<BTreeMap<_, _>>();
    compare_catalog_membership(
        &before_by_uri,
        &after_by_uri,
        before,
        after,
        CatalogLabel::Resources,
        findings,
    );
    for uri in before_by_uri
        .keys()
        .filter(|uri| after_by_uri.contains_key(*uri))
    {
        let (before_index, old) = before_by_uri[uri];
        let (after_index, new) = after_by_uri[uri];
        if old.name != new.name {
            push_diff_finding(
                findings,
                mark_before_after(
                    review_finding("resource_metadata_changed", CatalogLabel::Resources),
                    Some((before, before_index)),
                    Some((after, after_index)),
                ),
            );
        }
    }
}

fn compare_resource_templates(
    before: &SnapshotCatalog<ResourceTemplateContract>,
    after: &SnapshotCatalog<ResourceTemplateContract>,
    findings: &mut Vec<DiffFinding>,
) {
    let before_by_uri = before
        .contracts
        .iter()
        .enumerate()
        .map(|(index, contract)| (contract.uri_template.as_str(), (index, contract)))
        .collect::<BTreeMap<_, _>>();
    let after_by_uri = after
        .contracts
        .iter()
        .enumerate()
        .map(|(index, contract)| (contract.uri_template.as_str(), (index, contract)))
        .collect::<BTreeMap<_, _>>();
    compare_catalog_membership(
        &before_by_uri,
        &after_by_uri,
        before,
        after,
        CatalogLabel::ResourceTemplates,
        findings,
    );
    for uri in before_by_uri
        .keys()
        .filter(|uri| after_by_uri.contains_key(*uri))
    {
        let (before_index, old) = before_by_uri[uri];
        let (after_index, new) = after_by_uri[uri];
        if old.name != new.name {
            push_diff_finding(
                findings,
                mark_before_after(
                    review_finding(
                        "resource_template_metadata_changed",
                        CatalogLabel::ResourceTemplates,
                    ),
                    Some((before, before_index)),
                    Some((after, after_index)),
                ),
            );
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
enum SchemaChange {
    RequiredAdded,
    RequiredRemoved,
    Narrowed,
    Widened,
    Review,
}

fn schema_finding(change: SchemaChange, catalog: CatalogLabel) -> DiffFinding {
    match change {
        SchemaChange::RequiredAdded => {
            breaking_finding("MCP-DIFF-005", "required_input_added", catalog)
        }
        SchemaChange::RequiredRemoved => {
            compatible_finding("MCP-DIFF-006", "required_input_removed", catalog)
        }
        SchemaChange::Narrowed => {
            breaking_finding("MCP-DIFF-007", "schema_constraint_narrowed", catalog)
        }
        SchemaChange::Widened => {
            compatible_finding("MCP-DIFF-008", "schema_constraint_widened", catalog)
        }
        SchemaChange::Review => review_finding("schema_structure_changed", catalog),
    }
}

fn classify_input_schema(before: &Value, after: &Value) -> BTreeSet<SchemaChange> {
    classify_input_schema_with_dialects(before, after, None, None)
}

fn classify_input_schema_with_dialects(
    before: &Value,
    after: &Value,
    before_dialect: Option<SnapshotSchemaDialect>,
    after_dialect: Option<SnapshotSchemaDialect>,
) -> BTreeSet<SchemaChange> {
    let mut changes = BTreeSet::new();
    if before == after && before_dialect == after_dialect {
        return changes;
    }
    if !matches!(
        before_dialect,
        None | Some(SnapshotSchemaDialect::Draft2020_12)
    ) || !matches!(
        after_dialect,
        None | Some(SnapshotSchemaDialect::Draft2020_12)
    ) {
        changes.insert(SchemaChange::Review);
        return changes;
    }
    compare_schema_value(before, after, &mut changes);
    changes
}

fn compare_schema_value(before: &Value, after: &Value, changes: &mut BTreeSet<SchemaChange>) {
    if before == after {
        return;
    }
    let (Some(old), Some(new)) = (before.as_object(), after.as_object()) else {
        changes.insert(SchemaChange::Review);
        return;
    };

    let old_required = string_set(old.get("required"));
    let new_required = string_set(new.get("required"));
    let required_added = set_difference(&new_required, &old_required);
    let required_removed = set_difference(&old_required, &new_required);
    if !required_added.is_empty() {
        changes.insert(SchemaChange::RequiredAdded);
    }
    if !required_removed.is_empty() {
        changes.insert(SchemaChange::RequiredRemoved);
    }

    for key in old.keys().chain(new.keys()).collect::<BTreeSet<_>>() {
        let old_value = old.get(key);
        let new_value = new.get(key);
        if old_value == new_value {
            continue;
        }
        match key.as_str() {
            "required" => {}
            "type" | "enum" => classify_set_change(old_value, new_value, changes),
            "const" => match (old_value, new_value) {
                (None, Some(_)) => {
                    changes.insert(SchemaChange::Narrowed);
                }
                (Some(_), None) => {
                    changes.insert(SchemaChange::Widened);
                }
                _ => {
                    changes.insert(SchemaChange::Review);
                }
            },
            "minimum" | "exclusiveMinimum" | "minLength" | "minItems" | "minProperties"
            | "minContains" => classify_numeric_change(old_value, new_value, true, changes),
            "maximum" | "exclusiveMaximum" | "maxLength" | "maxItems" | "maxProperties"
            | "maxContains" => classify_numeric_change(old_value, new_value, false, changes),
            "additionalProperties" => classify_additional_properties(old_value, new_value, changes),
            "properties" => compare_properties(
                old_value,
                new_value,
                &required_added,
                &required_removed,
                changes,
            ),
            "$defs" | "definitions" | "patternProperties" | "dependentSchemas" => {
                compare_schema_maps(old_value, new_value, changes)
            }
            "items"
            | "contains"
            | "propertyNames"
            | "not"
            | "if"
            | "then"
            | "else"
            | "unevaluatedProperties"
            | "unevaluatedItems"
            | "contentSchema" => match (old_value, new_value) {
                (Some(old), Some(new)) => compare_schema_value(old, new, changes),
                _ => {
                    changes.insert(SchemaChange::Review);
                }
            },
            _ => {
                changes.insert(SchemaChange::Review);
            }
        }
    }
}

fn string_set(value: Option<&Value>) -> BTreeSet<String> {
    value
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect()
}

fn set_difference(left: &BTreeSet<String>, right: &BTreeSet<String>) -> BTreeSet<String> {
    left.difference(right).cloned().collect()
}

fn classify_set_change(
    old: Option<&Value>,
    new: Option<&Value>,
    changes: &mut BTreeSet<SchemaChange>,
) {
    let old = value_set(old);
    let new = value_set(new);
    match (old, new) {
        (None, Some(_)) => {
            changes.insert(SchemaChange::Narrowed);
        }
        (Some(_), None) => {
            changes.insert(SchemaChange::Widened);
        }
        (Some(old), Some(new)) if new.is_subset(&old) => {
            changes.insert(SchemaChange::Narrowed);
        }
        (Some(old), Some(new)) if old.is_subset(&new) => {
            changes.insert(SchemaChange::Widened);
        }
        _ => {
            changes.insert(SchemaChange::Review);
        }
    }
}

fn value_set(value: Option<&Value>) -> Option<BTreeSet<String>> {
    let value = value?;
    let values = value
        .as_array()
        .map_or_else(|| vec![value], |values| values.iter().collect::<Vec<_>>());
    Some(values.into_iter().map(canonical_key).collect())
}

fn classify_numeric_change(
    old: Option<&Value>,
    new: Option<&Value>,
    higher_is_narrower: bool,
    changes: &mut BTreeSet<SchemaChange>,
) {
    match (
        old.and_then(Value::as_number),
        new.and_then(Value::as_number),
    ) {
        (None, Some(_)) => {
            changes.insert(SchemaChange::Narrowed);
        }
        (Some(_), None) => {
            changes.insert(SchemaChange::Widened);
        }
        (Some(old), Some(new)) => match compare_numbers(old, new) {
            Some(Ordering::Less) if higher_is_narrower => {
                changes.insert(SchemaChange::Narrowed);
            }
            Some(Ordering::Greater) if !higher_is_narrower => {
                changes.insert(SchemaChange::Narrowed);
            }
            Some(Ordering::Greater) if higher_is_narrower => {
                changes.insert(SchemaChange::Widened);
            }
            Some(Ordering::Less) if !higher_is_narrower => {
                changes.insert(SchemaChange::Widened);
            }
            Some(Ordering::Equal) => {}
            _ => {
                changes.insert(SchemaChange::Review);
            }
        },
        _ => {
            changes.insert(SchemaChange::Review);
        }
    }
}

fn compare_numbers(left: &Number, right: &Number) -> Option<Ordering> {
    match (left.as_i64(), right.as_i64()) {
        (Some(left), Some(right)) => return Some(left.cmp(&right)),
        (Some(left), None) if right.as_u64().is_some() => {
            return Some(if left.is_negative() {
                Ordering::Less
            } else {
                u64::try_from(left).ok()?.cmp(&right.as_u64()?)
            });
        }
        (None, Some(right)) if left.as_u64().is_some() => {
            return Some(if right.is_negative() {
                Ordering::Greater
            } else {
                left.as_u64()?.cmp(&u64::try_from(right).ok()?)
            });
        }
        _ => {}
    }
    if let (Some(left), Some(right)) = (left.as_u64(), right.as_u64()) {
        return Some(left.cmp(&right));
    }
    let left_is_integer = left.as_i64().is_some() || left.as_u64().is_some();
    let right_is_integer = right.as_i64().is_some() || right.as_u64().is_some();
    if left_is_integer != right_is_integer {
        return None;
    }
    left.as_f64()?.partial_cmp(&right.as_f64()?)
}

fn classify_additional_properties(
    old: Option<&Value>,
    new: Option<&Value>,
    changes: &mut BTreeSet<SchemaChange>,
) {
    let old = match old {
        None => true,
        Some(Value::Bool(value)) => *value,
        Some(_) => {
            changes.insert(SchemaChange::Review);
            return;
        }
    };
    let new = match new {
        None => true,
        Some(Value::Bool(value)) => *value,
        Some(_) => {
            changes.insert(SchemaChange::Review);
            return;
        }
    };
    match (old, new) {
        (true, false) => {
            changes.insert(SchemaChange::Narrowed);
        }
        (false, true) => {
            changes.insert(SchemaChange::Widened);
        }
        _ => {
            changes.insert(SchemaChange::Review);
        }
    }
}

fn compare_properties(
    old: Option<&Value>,
    new: Option<&Value>,
    required_added: &BTreeSet<String>,
    required_removed: &BTreeSet<String>,
    changes: &mut BTreeSet<SchemaChange>,
) {
    let old = old.and_then(Value::as_object);
    let new = new.and_then(Value::as_object);
    let names = old
        .into_iter()
        .flat_map(Map::keys)
        .chain(new.into_iter().flat_map(Map::keys))
        .collect::<BTreeSet<_>>();
    for name in names {
        match (
            old.and_then(|map| map.get(name)),
            new.and_then(|map| map.get(name)),
        ) {
            (Some(old), Some(new)) => compare_schema_value(old, new, changes),
            (None, Some(_)) if required_added.contains(name) => {}
            (Some(_), None) if required_removed.contains(name) => {}
            _ => {
                changes.insert(SchemaChange::Review);
            }
        }
    }
}

fn compare_schema_maps(
    old: Option<&Value>,
    new: Option<&Value>,
    changes: &mut BTreeSet<SchemaChange>,
) {
    let (Some(old), Some(new)) = (
        old.and_then(Value::as_object),
        new.and_then(Value::as_object),
    ) else {
        changes.insert(SchemaChange::Review);
        return;
    };
    let names = old.keys().chain(new.keys()).collect::<BTreeSet<_>>();
    for name in names {
        match (old.get(name), new.get(name)) {
            (Some(old), Some(new)) => compare_schema_value(old, new, changes),
            _ => {
                changes.insert(SchemaChange::Review);
            }
        }
    }
}

fn render_diff_report(report: ContractDiffReport, format: DiffFormat) -> RenderedContractDiff {
    let exit = ExitCode::from(report.exit_code);
    let rendered = match format {
        DiffFormat::Json => serde_json::to_string_pretty(&report).map(|mut output| {
            output.push('\n');
            output
        }),
        DiffFormat::Human => Ok(render_human_diff(&report)),
    };
    match rendered {
        Ok(output)
            if u64::try_from(output.len()).unwrap_or(u64::MAX)
                <= DiagnosticLimits::M1_DEFAULTS.values().report_bytes =>
        {
            RenderedContractDiff {
                output,
                exit,
                error: None,
            }
        }
        _ => RenderedContractDiff {
            output: String::new(),
            exit: ExitCode::from(2),
            error: Some(
                "the contract diff could not be rendered within its finite bound".to_owned(),
            ),
        },
    }
}

fn render_human_diff(report: &ContractDiffReport) -> String {
    use std::fmt::Write as _;

    let mut output = String::new();
    let _ = writeln!(output, "mcp-doctor contract diff");
    if let Some(revision) = report.protocol_revision {
        let _ = writeln!(output, "protocol revision: {revision}");
    } else {
        let _ = writeln!(output, "protocol revision: unavailable");
    }
    let _ = writeln!(output, "outcome: {}", outcome_name(report.outcome));
    let _ = writeln!(output, "checks:");
    for check in &report.checks {
        match check.state {
            DiffCheckState::Performed => {
                let _ = writeln!(output, "  {}: performed", check.id);
            }
            DiffCheckState::Skipped => {
                let _ = writeln!(
                    output,
                    "  {}: skipped (blocked by {})",
                    check.id,
                    check.blocked_by.unwrap_or("artifact_validation")
                );
            }
        }
    }
    let _ = writeln!(output, "findings:");
    if report.findings.is_empty() {
        let _ = writeln!(output, "  none");
    }
    for finding in &report.findings {
        let _ = write!(
            output,
            "  {} [{}] {}",
            finding.code,
            classification_name(finding.classification),
            finding.change
        );
        if let Some(catalog) = finding.catalog {
            let _ = write!(output, " · {}", catalog_name(catalog));
        }
        if let Some(ordinal) = finding.before_ordinal {
            let _ = write!(output, " · before[{ordinal}]");
        }
        if let Some(ordinal) = finding.after_ordinal {
            let _ = write!(output, " · after[{ordinal}]");
        }
        if let Some(input) = finding.input {
            let _ = write!(output, " · {} artifact", input_name(input));
        }
        let _ = writeln!(output);
    }
    let _ = writeln!(
        output,
        "summary: compatible={} potentially_breaking={} review_required={} invalid={} total={}",
        report.summary.compatible,
        report.summary.potentially_breaking,
        report.summary.review_required,
        report.summary.invalid,
        report.summary.total
    );
    output
}

const fn outcome_name(outcome: DiffOutcome) -> &'static str {
    match outcome {
        DiffOutcome::Unchanged => "unchanged",
        DiffOutcome::Compatible => "compatible",
        DiffOutcome::ReviewRequired => "review_required",
        DiffOutcome::PotentiallyBreaking => "potentially_breaking",
        DiffOutcome::Invalid => "invalid",
    }
}

const fn classification_name(classification: DiffClassification) -> &'static str {
    match classification {
        DiffClassification::Compatible => "compatible",
        DiffClassification::PotentiallyBreaking => "potentially_breaking",
        DiffClassification::ReviewRequired => "review_required",
        DiffClassification::Invalid => "invalid",
    }
}

const fn catalog_name(catalog: CatalogLabel) -> &'static str {
    match catalog {
        CatalogLabel::Tools => "tools",
        CatalogLabel::Prompts => "prompts",
        CatalogLabel::Resources => "resources",
        CatalogLabel::ResourceTemplates => "resource_templates",
        CatalogLabel::Capabilities => "capabilities",
    }
}

const fn input_name(input: DiffInput) -> &'static str {
    match input {
        DiffInput::Before => "before",
        DiffInput::After => "after",
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use serde_json::{Value, json};

    use super::{
        DRAFT_2020_12, DiffClassification, SchemaChange, SnapshotInputKind, SnapshotSchemaDialect,
        SupportedRevision, artifact_schema_dialect, classify_input_schema,
        classify_input_schema_with_dialects, normalize_schema, normalize_schema_value,
    };

    #[test]
    fn normalization_removes_only_selected_schema_annotations_and_sorts_sets() {
        let schema = json!({
            "description": "sensitive",
            "default": {"description": "literal-kept-only-if-default-were-kept"},
            "properties": {
                "description": {"type": ["null", "string"], "title": "sensitive"},
                "mode": {"enum": ["z", "a"]}
            },
            "required": ["mode", "description"]
        });
        let normalized = normalize_schema_value(&schema).expect("schema normalization should pass");
        assert!(normalized.get("description").is_none());
        assert!(normalized.get("default").is_none());
        assert_eq!(
            normalized["properties"]["description"]["type"],
            json!(["null", "string"])
        );
        assert_eq!(normalized["properties"]["mode"]["enum"], json!(["a", "z"]));
        assert_eq!(normalized["required"], json!(["description", "mode"]));
    }

    #[test]
    fn required_and_monotonic_constraints_receive_documented_classes() {
        let before = json!({
            "type": "object",
            "properties": {"count": {"type": "integer", "minimum": 0}},
            "required": []
        });
        let after = json!({
            "type": "object",
            "properties": {"count": {"type": "integer", "minimum": 1}},
            "required": ["count"]
        });
        let changes = classify_input_schema(&before, &after);
        assert!(changes.contains(&SchemaChange::RequiredAdded));
        assert!(changes.contains(&SchemaChange::Narrowed));
        assert!(!changes.contains(&SchemaChange::Review));
    }

    #[test]
    fn malformed_local_schema_shapes_remain_available_for_sensitive_correlation() {
        let malformed = json!({
            "description": "excluded",
            "properties": {"field": 7},
            "allOf": [false, 42]
        });
        let normalized = normalize_schema(&malformed)
            .expect("bounded malformed local structure should remain capturable");
        assert!(normalized.get("description").is_none());
        assert_eq!(normalized["properties"]["field"], 7);
        assert_eq!(normalized["allOf"], json!([42, false]));
        assert_eq!(
            normalize_schema(&json!(7)).expect("a malformed root should remain capturable"),
            7
        );
    }

    #[test]
    fn integer_constraints_are_compared_without_floating_point_loss() {
        let before = json!({"minimum": 9_007_199_254_740_992_u64});
        let after = json!({"minimum": 9_007_199_254_740_993_u64});
        let narrowed = classify_input_schema(&before, &after);
        assert_eq!(narrowed, BTreeSet::from([SchemaChange::Narrowed]));
        let widened = classify_input_schema(&after, &before);
        assert_eq!(widened, BTreeSet::from([SchemaChange::Widened]));
    }

    #[test]
    fn unsupported_schema_implication_falls_back_to_review() {
        let before = json!({"type": "string", "pattern": "^a"});
        let after = json!({"type": "string", "pattern": "^b"});
        let changes = classify_input_schema(&before, &after);
        assert_eq!(changes.len(), 1);
        assert!(changes.contains(&SchemaChange::Review));
    }

    #[test]
    fn legacy_dialect_state_preserves_revision_defaults_and_conservative_comparison() {
        assert_eq!(
            artifact_schema_dialect(SupportedRevision::V2025_11_25, &json!({})),
            Ok(Some(SnapshotSchemaDialect::Draft2020_12))
        );
        assert_eq!(
            artifact_schema_dialect(SupportedRevision::V2025_06_18, &json!({})),
            Ok(Some(SnapshotSchemaDialect::Ambiguous))
        );
        assert_eq!(
            artifact_schema_dialect(
                SupportedRevision::V2025_06_18,
                &json!({"$schema": DRAFT_2020_12})
            ),
            Ok(Some(SnapshotSchemaDialect::Draft2020_12))
        );
        assert_eq!(
            artifact_schema_dialect(
                SupportedRevision::V2025_11_25,
                &json!({"$schema": "synthetic-unsupported-dialect"})
            ),
            Ok(Some(SnapshotSchemaDialect::Unsupported))
        );
        assert_eq!(
            artifact_schema_dialect(
                SupportedRevision::CURRENT,
                &json!({"$schema": DRAFT_2020_12})
            ),
            Ok(None)
        );

        let before = json!({"type": "string", "minLength": 1});
        let after = json!({"type": "string", "minLength": 2});
        assert_eq!(
            classify_input_schema_with_dialects(
                &before,
                &after,
                Some(SnapshotSchemaDialect::Draft2020_12),
                Some(SnapshotSchemaDialect::Draft2020_12),
            ),
            BTreeSet::from([SchemaChange::Narrowed])
        );
        for dialect in [
            SnapshotSchemaDialect::Ambiguous,
            SnapshotSchemaDialect::Unsupported,
        ] {
            assert_eq!(
                classify_input_schema_with_dialects(&before, &after, Some(dialect), Some(dialect),),
                BTreeSet::from([SchemaChange::Review])
            );
            assert!(
                classify_input_schema_with_dialects(
                    &before,
                    &before,
                    Some(dialect),
                    Some(dialect),
                )
                .is_empty()
            );
        }
    }

    #[test]
    fn stable_classification_names_remain_machine_safe() {
        let value: Value = serde_json::to_value(DiffClassification::PotentiallyBreaking)
            .expect("classification should serialize");
        assert_eq!(value, "potentially_breaking");
        assert_eq!(
            SnapshotInputKind::ExternalReference.code(),
            "MCP-SNAPSHOT-005"
        );
    }
}
