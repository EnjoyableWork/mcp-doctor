use serde_json::{Map, Value, json};

use super::catalog::{LocalSchemaDialectPolicy, request_meta};
use super::protocol::{ActiveProtocolRevision, SupportedRevision};
use crate::transport::{MirroredField, ProbeRequest};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(super) enum ActiveStartKind {
    Discover,
    Initialize,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(super) enum ActiveToolResultKind {
    Modern,
    Legacy,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(super) enum ActiveTaskSupport {
    Immediate,
    Required,
    InvalidExecution,
    InvalidTaskSupport,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(super) struct ActiveProtocolAdapter {
    revision: ActiveProtocolRevision,
}

impl ActiveProtocolAdapter {
    pub(super) const fn new(revision: ActiveProtocolRevision) -> Self {
        Self { revision }
    }

    pub(super) const fn revision(self) -> SupportedRevision {
        self.revision.as_supported()
    }

    pub(super) const fn revision_name(self) -> &'static str {
        self.revision.as_str()
    }

    pub(super) const fn uses_initialize(self) -> bool {
        self.revision.uses_initialize()
    }

    pub(super) const fn start_kind(self) -> ActiveStartKind {
        if self.uses_initialize() {
            ActiveStartKind::Initialize
        } else {
            ActiveStartKind::Discover
        }
    }

    pub(super) const fn tool_result_kind(self) -> ActiveToolResultKind {
        if self.uses_initialize() {
            ActiveToolResultKind::Legacy
        } else {
            ActiveToolResultKind::Modern
        }
    }

    pub(super) const fn permits_http_mappings(self) -> bool {
        !self.uses_initialize()
    }

    pub(super) const fn schema_dialect_policy(self) -> LocalSchemaDialectPolicy {
        match self.revision {
            ActiveProtocolRevision::V2025_06_18 => {
                LocalSchemaDialectPolicy::RequireExactDraft202012
            }
            ActiveProtocolRevision::V2025_11_25 | ActiveProtocolRevision::V2026_07_28 => {
                LocalSchemaDialectPolicy::RevisionDefaultDraft202012
            }
        }
    }

    pub(super) fn task_support(self, tool: &Map<String, Value>) -> ActiveTaskSupport {
        if !self.uses_initialize() {
            return ActiveTaskSupport::Immediate;
        }
        let Some(execution) = tool.get("execution") else {
            return ActiveTaskSupport::Immediate;
        };
        let Some(execution) = execution.as_object() else {
            return ActiveTaskSupport::InvalidExecution;
        };
        match execution.get("taskSupport").and_then(Value::as_str) {
            None if !execution.contains_key("taskSupport") => ActiveTaskSupport::Immediate,
            Some("forbidden" | "optional") => ActiveTaskSupport::Immediate,
            Some("required") => ActiveTaskSupport::Required,
            Some(_) | None => ActiveTaskSupport::InvalidTaskSupport,
        }
    }

    pub(super) fn start_request(self, id: i64) -> ProbeRequest {
        let value = match self.start_kind() {
            ActiveStartKind::Discover => json!({
                "jsonrpc": "2.0",
                "id": id,
                "method": "server/discover",
                "params": {"_meta": request_meta()},
            }),
            ActiveStartKind::Initialize => json!({
                "jsonrpc": "2.0",
                "id": id,
                "method": "initialize",
                "params": {
                    "protocolVersion": self.revision_name(),
                    "capabilities": {},
                    "clientInfo": {
                        "name": "mcp-doctor",
                        "version": env!("CARGO_PKG_VERSION"),
                    },
                },
            }),
        };
        ProbeRequest::new(id, encode(value))
    }

    pub(super) fn initialized_notification(self) -> Option<ProbeRequest> {
        self.uses_initialize().then(|| {
            ProbeRequest::notification(encode(json!({
                "jsonrpc": "2.0",
                "method": "notifications/initialized",
            })))
        })
    }

    pub(super) fn tools_request(self, id: i64, cursor: Option<&str>) -> ProbeRequest {
        let mut params = Map::new();
        if !self.uses_initialize() {
            params.insert("_meta".to_owned(), request_meta());
        }
        if let Some(cursor) = cursor {
            params.insert("cursor".to_owned(), Value::String(cursor.to_owned()));
        }
        ProbeRequest::new(
            id,
            encode(json!({
                "jsonrpc": "2.0",
                "id": id,
                "method": "tools/list",
                "params": params,
            })),
        )
    }

    pub(super) fn tool_call_request(
        self,
        id: i64,
        tool: String,
        arguments: Value,
        mirrored_fields: Vec<MirroredField>,
    ) -> ProbeRequest {
        let mut params = Map::new();
        params.insert("name".to_owned(), Value::String(tool));
        params.insert("arguments".to_owned(), arguments);
        if !self.uses_initialize() {
            params.insert("_meta".to_owned(), request_meta());
        }
        let request = ProbeRequest::new(
            id,
            encode(json!({
                "jsonrpc": "2.0",
                "id": id,
                "method": "tools/call",
                "params": params,
            })),
        );
        if self.permits_http_mappings() {
            request.with_mirrored_fields(mirrored_fields)
        } else {
            request
        }
    }
}

fn encode(value: Value) -> Vec<u8> {
    serde_json::to_vec(&value).expect("typed active protocol messages must serialize")
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::{ActiveProtocolAdapter, ActiveStartKind, ActiveTaskSupport, ActiveToolResultKind};
    use crate::contract::catalog::LocalSchemaDialectPolicy;
    use crate::contract::protocol::ActiveProtocolRevision;
    use crate::transport::MirroredField;

    fn value(bytes: &[u8]) -> Value {
        serde_json::from_slice(bytes).expect("adapter output should be JSON")
    }

    #[test]
    fn current_adapter_preserves_modern_discovery_and_call_contract() {
        let adapter = ActiveProtocolAdapter::new(ActiveProtocolRevision::CURRENT);
        assert_eq!(adapter.start_kind(), ActiveStartKind::Discover);
        assert_eq!(adapter.tool_result_kind(), ActiveToolResultKind::Modern);
        assert_eq!(
            adapter.schema_dialect_policy(),
            LocalSchemaDialectPolicy::RevisionDefaultDraft202012
        );
        assert!(adapter.initialized_notification().is_none());
        assert!(adapter.permits_http_mappings());

        let discovery = adapter.start_request(1);
        assert_eq!(discovery.method(), "server/discover");
        assert_eq!(
            value(discovery.as_bytes()),
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "server/discover",
                "params": {
                    "_meta": {
                        "io.modelcontextprotocol/clientCapabilities": {},
                        "io.modelcontextprotocol/clientInfo": {
                            "name": "mcp-doctor",
                            "version": env!("CARGO_PKG_VERSION"),
                        },
                        "io.modelcontextprotocol/protocolVersion": "2026-07-28",
                    },
                },
            })
        );

        let call = adapter.tool_call_request(
            3,
            "synthetic-tool".to_owned(),
            json!({"safe": true}),
            vec![MirroredField::new("safe".to_owned(), "true".to_owned())],
        );
        assert_eq!(
            value(adapter.tools_request(2, None).as_bytes()),
            json!({
                "jsonrpc": "2.0",
                "id": 2,
                "method": "tools/list",
                "params": {
                    "_meta": {
                        "io.modelcontextprotocol/clientCapabilities": {},
                        "io.modelcontextprotocol/clientInfo": {
                            "name": "mcp-doctor",
                            "version": env!("CARGO_PKG_VERSION"),
                        },
                        "io.modelcontextprotocol/protocolVersion": "2026-07-28",
                    },
                },
            })
        );
        assert_eq!(call.method(), "tools/call");
        assert_eq!(call.mirrored_fields().len(), 1);
        assert_eq!(
            value(call.as_bytes()),
            json!({
                "jsonrpc": "2.0",
                "id": 3,
                "method": "tools/call",
                "params": {
                    "name": "synthetic-tool",
                    "arguments": {"safe": true},
                    "_meta": {
                        "io.modelcontextprotocol/clientCapabilities": {},
                        "io.modelcontextprotocol/clientInfo": {
                            "name": "mcp-doctor",
                            "version": env!("CARGO_PKG_VERSION"),
                        },
                        "io.modelcontextprotocol/protocolVersion": "2026-07-28",
                    },
                },
            })
        );
    }

    #[test]
    fn legacy_adapter_owns_initialize_and_omits_modern_request_metadata() {
        let adapter = ActiveProtocolAdapter::new(ActiveProtocolRevision::V2025_11_25);
        assert_eq!(adapter.start_kind(), ActiveStartKind::Initialize);
        assert_eq!(adapter.tool_result_kind(), ActiveToolResultKind::Legacy);
        assert_eq!(
            adapter.schema_dialect_policy(),
            LocalSchemaDialectPolicy::RevisionDefaultDraft202012
        );
        assert!(!adapter.permits_http_mappings());

        let initialize = adapter.start_request(1);
        assert_eq!(initialize.method(), "initialize");
        assert_eq!(
            value(initialize.as_bytes()),
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {
                    "protocolVersion": "2025-11-25",
                    "capabilities": {},
                    "clientInfo": {
                        "name": "mcp-doctor",
                        "version": env!("CARGO_PKG_VERSION"),
                    },
                },
            })
        );
        let initialized = adapter
            .initialized_notification()
            .expect("legacy runs require initialized");
        assert!(!initialized.expects_response());
        assert_eq!(initialized.method(), "notifications/initialized");

        let list = adapter.tools_request(2, None);
        assert_eq!(value(list.as_bytes())["params"], json!({}));
        let call = adapter.tool_call_request(
            3,
            "synthetic-tool".to_owned(),
            json!({}),
            vec![MirroredField::new("unsafe".to_owned(), "value".to_owned())],
        );
        assert!(value(call.as_bytes())["params"].get("_meta").is_none());
        assert!(call.mirrored_fields().is_empty());

        for tool in [
            json!({"name": "synthetic"}),
            json!({"name": "synthetic", "execution": {"taskSupport": "forbidden"}}),
            json!({"name": "synthetic", "execution": {"taskSupport": "optional"}}),
        ] {
            assert_eq!(
                adapter.task_support(tool.as_object().expect("fixture object")),
                ActiveTaskSupport::Immediate
            );
        }
        assert_eq!(
            adapter.task_support(
                json!({"execution": {"taskSupport": "required"}})
                    .as_object()
                    .expect("fixture object")
            ),
            ActiveTaskSupport::Required
        );
        assert_eq!(
            adapter.task_support(
                json!({"execution": []})
                    .as_object()
                    .expect("fixture object")
            ),
            ActiveTaskSupport::InvalidExecution
        );
        assert_eq!(
            adapter.task_support(
                json!({"execution": {"taskSupport": "future"}})
                    .as_object()
                    .expect("fixture object")
            ),
            ActiveTaskSupport::InvalidTaskSupport
        );
    }

    #[test]
    fn v2025_06_adapter_reuses_legacy_wire_contract_with_an_exact_schema_gate() {
        let adapter = ActiveProtocolAdapter::new(ActiveProtocolRevision::V2025_06_18);
        assert_eq!(adapter.start_kind(), ActiveStartKind::Initialize);
        assert_eq!(adapter.tool_result_kind(), ActiveToolResultKind::Legacy);
        assert_eq!(
            adapter.schema_dialect_policy(),
            LocalSchemaDialectPolicy::RequireExactDraft202012
        );
        assert!(!adapter.permits_http_mappings());

        let initialize = adapter.start_request(1);
        assert_eq!(initialize.method(), "initialize");
        assert_eq!(
            value(initialize.as_bytes()),
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {
                    "protocolVersion": "2025-06-18",
                    "capabilities": {},
                    "clientInfo": {
                        "name": "mcp-doctor",
                        "version": env!("CARGO_PKG_VERSION"),
                    },
                },
            })
        );
        assert_eq!(
            adapter
                .initialized_notification()
                .expect("the selected legacy revision requires initialized")
                .method(),
            "notifications/initialized"
        );
        assert_eq!(
            value(adapter.tools_request(2, None).as_bytes())["params"],
            json!({})
        );
        let call = adapter.tool_call_request(
            3,
            "synthetic-tool".to_owned(),
            json!({}),
            vec![MirroredField::new("unsafe".to_owned(), "value".to_owned())],
        );
        assert!(value(call.as_bytes())["params"].get("_meta").is_none());
        assert!(call.mirrored_fields().is_empty());
    }
}
