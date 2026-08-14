mod support;

use std::collections::BTreeMap;
use std::fs;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::{Arc, OnceLock};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use rcgen::{
    BasicConstraints, CertificateParams, ExtendedKeyUsagePurpose, IsCa, Issuer, KeyPair,
    KeyUsagePurpose, date_time_ymd,
};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use rustls::{ServerConfig, ServerConnection, StreamOwned};
use serde_json::{Value, json};
use support::{
    TestEnvironment, parse_and_validate_contract_snapshot, parse_and_validate_junit,
    parse_and_validate_report,
};

const TOOL: &str = "synthetic.remote-reviewed";
const CASE_ID: &str = "author-only-remote-case-never-report-4a91";
const MIRRORED_VALUE: &str = "synthetic-north-4a91";
const BEARER_SOURCE: &str = "MCP_DOCTOR_SYNTHETIC_BEARER_4A91";
const BEARER_VALUE: &str = "synthetic.bearer-token-4a91";
const CUSTOM_SOURCE: &str = "MCP_DOCTOR_SYNTHETIC_ROUTE_4A91";
const CUSTOM_FIELD: &str = "X-Synthetic-Route-4A91";
const CUSTOM_VALUE: &str = "synthetic-route-value-4a91";
const LEGACY_SESSION: &str = "synthetic-legacy-session-4a91";
const LEGACY_CURSOR: &str = "synthetic-legacy-cursor-never-report-4a91";

#[derive(Clone, Copy)]
enum WireMode {
    Http,
    Https,
}

#[derive(Clone, Copy)]
struct ExpectedRequest {
    method: &'static str,
    name: Option<&'static str>,
    bearer: Option<&'static str>,
    custom: Option<(&'static str, &'static str)>,
    mirrored: Option<(&'static str, &'static str)>,
}

impl ExpectedRequest {
    const fn method(method: &'static str) -> Self {
        Self {
            method,
            name: None,
            bearer: None,
            custom: None,
            mirrored: None,
        }
    }
}

struct PlannedExchange {
    expected: Option<RequestExpectation>,
    response: Option<FixtureResponse>,
    stall_for: Option<Duration>,
}

enum RequestExpectation {
    Current(ExpectedRequest),
    Legacy(LegacyExpectedRequest),
}

#[derive(Clone, Copy)]
struct LegacyExpectedRequest {
    verb: &'static str,
    method: Option<&'static str>,
    revision: &'static str,
    protocol_header: bool,
    session: Option<&'static str>,
    cursor: Option<&'static str>,
    bearer: Option<&'static str>,
    custom: Option<(&'static str, &'static str)>,
    name: Option<&'static str>,
    argument: Option<(&'static str, &'static str)>,
}

impl LegacyExpectedRequest {
    const fn initialize(revision: &'static str) -> Self {
        Self {
            verb: "POST",
            method: Some("initialize"),
            revision,
            protocol_header: false,
            session: None,
            cursor: None,
            bearer: None,
            custom: None,
            name: None,
            argument: None,
        }
    }

    const fn initialized(revision: &'static str, session: Option<&'static str>) -> Self {
        Self {
            verb: "POST",
            method: Some("notifications/initialized"),
            revision,
            protocol_header: true,
            session,
            cursor: None,
            bearer: None,
            custom: None,
            name: None,
            argument: None,
        }
    }

    const fn list(
        revision: &'static str,
        session: Option<&'static str>,
        cursor: Option<&'static str>,
    ) -> Self {
        Self {
            verb: "POST",
            method: Some("tools/list"),
            revision,
            protocol_header: true,
            session,
            cursor,
            bearer: None,
            custom: None,
            name: None,
            argument: None,
        }
    }

    const fn call(
        revision: &'static str,
        session: Option<&'static str>,
        name: &'static str,
        argument: Option<(&'static str, &'static str)>,
    ) -> Self {
        Self {
            verb: "POST",
            method: Some("tools/call"),
            revision,
            protocol_header: true,
            session,
            cursor: None,
            bearer: None,
            custom: None,
            name: Some(name),
            argument,
        }
    }

    const fn delete(revision: &'static str, session: &'static str) -> Self {
        Self {
            verb: "DELETE",
            method: None,
            revision,
            protocol_header: true,
            session: Some(session),
            cursor: None,
            bearer: None,
            custom: None,
            name: None,
            argument: None,
        }
    }

    const fn with_credentials(
        mut self,
        bearer: &'static str,
        custom: (&'static str, &'static str),
    ) -> Self {
        self.bearer = Some(bearer);
        self.custom = Some(custom);
        self
    }
}

impl PlannedExchange {
    fn reply(expected: ExpectedRequest, response: FixtureResponse) -> Self {
        Self {
            expected: Some(RequestExpectation::Current(expected)),
            response: Some(response),
            stall_for: None,
        }
    }

    fn legacy_reply(expected: LegacyExpectedRequest, response: FixtureResponse) -> Self {
        Self {
            expected: Some(RequestExpectation::Legacy(expected)),
            response: Some(response),
            stall_for: None,
        }
    }

    fn legacy_stall(expected: LegacyExpectedRequest) -> Self {
        Self {
            expected: Some(RequestExpectation::Legacy(expected)),
            response: None,
            stall_for: Some(Duration::from_secs(3)),
        }
    }

    fn legacy_timeout(expected: LegacyExpectedRequest) -> Self {
        Self {
            expected: Some(RequestExpectation::Legacy(expected)),
            response: None,
            stall_for: Some(Duration::from_secs(11)),
        }
    }

    fn disconnect(expected: ExpectedRequest) -> Self {
        Self {
            expected: Some(RequestExpectation::Current(expected)),
            response: None,
            stall_for: None,
        }
    }

    fn handshake_only() -> Self {
        Self {
            expected: None,
            response: None,
            stall_for: None,
        }
    }
}

struct FixtureResponse {
    status: u16,
    reason: &'static str,
    fields: Vec<(String, String)>,
    body: Vec<u8>,
    hold_open: bool,
}

impl FixtureResponse {
    fn json(value: Value) -> Self {
        Self {
            status: 200,
            reason: "OK",
            fields: vec![("Content-Type".to_owned(), "application/json".to_owned())],
            body: serde_json::to_vec(&value).expect("synthetic response should serialize"),
            hold_open: false,
        }
    }

    fn sse(body: Vec<u8>) -> Self {
        Self {
            status: 200,
            reason: "OK",
            fields: vec![(
                "Content-Type".to_owned(),
                "text/event-stream; charset=utf-8".to_owned(),
            )],
            body,
            hold_open: false,
        }
    }

    fn status(status: u16, reason: &'static str) -> Self {
        Self {
            status,
            reason,
            fields: vec![("Content-Type".to_owned(), "application/json".to_owned())],
            body: Vec::new(),
            hold_open: false,
        }
    }

    fn accepted() -> Self {
        Self {
            status: 202,
            reason: "Accepted",
            fields: Vec::new(),
            body: Vec::new(),
            hold_open: false,
        }
    }

    fn with_session(mut self, session: &str) -> Self {
        self.fields
            .push(("Mcp-Session-Id".to_owned(), session.to_owned()));
        self
    }

    fn holding_open(mut self) -> Self {
        self.hold_open = true;
        self
    }
}

#[derive(Debug)]
struct FixtureOutcome {
    accepted_connections: usize,
    valid_requests: usize,
    unexpected_connections: usize,
    request_failures: usize,
}

struct FixtureServer {
    port: u16,
    mode: WireMode,
    join: thread::JoinHandle<FixtureOutcome>,
}

struct FixtureIdentity {
    ca_pem: String,
    server_certificate: Vec<u8>,
    server_key: Vec<u8>,
}

impl FixtureServer {
    fn spawn(mode: WireMode, exchanges: Vec<PlannedExchange>, watch_for_extra: bool) -> Self {
        let listener = TcpListener::bind(("127.0.0.1", 0))
            .expect("a disposable loopback listener should bind");
        let port = listener
            .local_addr()
            .expect("the loopback listener should have an address")
            .port();
        listener
            .set_nonblocking(true)
            .expect("the disposable listener should become nonblocking");
        let tls = matches!(mode, WireMode::Https).then(tls_server_config);
        let join = thread::spawn(move || serve_fixture(listener, tls, exchanges, watch_for_extra));
        Self { port, mode, join }
    }

    fn endpoint(&self) -> String {
        let scheme = match self.mode {
            WireMode::Http => "http",
            WireMode::Https => "https",
        };
        format!("{scheme}://127.0.0.1:{}/mcp", self.port)
    }

    fn endpoint_for_host(&self, host: &str) -> String {
        let scheme = match self.mode {
            WireMode::Http => "http",
            WireMode::Https => "https",
        };
        format!("{scheme}://{host}:{}/mcp", self.port)
    }

    fn finish(self) -> FixtureOutcome {
        self.join
            .join()
            .expect("the disposable HTTP fixture should not panic")
    }
}

fn tls_server_config() -> Arc<ServerConfig> {
    static PROVIDER: OnceLock<()> = OnceLock::new();
    PROVIDER.get_or_init(|| {
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
    let identity = fixture_identity();
    let certificates = vec![CertificateDer::from(identity.server_certificate.clone())];
    let key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(identity.server_key.clone()));
    Arc::new(
        ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certificates, key)
            .expect("the synthetic TLS identity should be coherent"),
    )
}

fn fixture_identity() -> &'static FixtureIdentity {
    static IDENTITY: OnceLock<FixtureIdentity> = OnceLock::new();
    IDENTITY.get_or_init(|| {
        let year = current_utc_year();
        let not_before = date_time_ymd(year, 1, 1);
        // At most 397 days, satisfying the platform verifier's leaf validity
        // ceiling while allowing a test process to cross the year boundary.
        let not_after = date_time_ymd(year + 1, 2, 1);

        let mut ca_params = CertificateParams::new(Vec::<String>::new())
            .expect("an empty CA subject-alt-name list is valid");
        ca_params.not_before = not_before;
        ca_params.not_after = not_after;
        ca_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        ca_params.key_usages = vec![
            KeyUsagePurpose::DigitalSignature,
            KeyUsagePurpose::KeyCertSign,
            KeyUsagePurpose::CrlSign,
        ];
        let ca_key = KeyPair::generate().expect("the synthetic CA key should generate");
        let ca_certificate = ca_params
            .self_signed(&ca_key)
            .expect("the synthetic CA should self-sign");
        let issuer = Issuer::new(ca_params, ca_key);

        let mut server_params = CertificateParams::new(vec!["127.0.0.1".to_owned()])
            .expect("the loopback IP subject-alt-name is valid");
        server_params.not_before = not_before;
        server_params.not_after = not_after;
        server_params.key_usages = vec![KeyUsagePurpose::DigitalSignature];
        server_params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
        server_params.use_authority_key_identifier_extension = true;
        let server_key = KeyPair::generate().expect("the synthetic server key should generate");
        let server_certificate = server_params
            .signed_by(&server_key, &issuer)
            .expect("the synthetic server identity should be signed by the disposable CA");

        FixtureIdentity {
            ca_pem: ca_certificate.pem(),
            server_certificate: server_certificate.der().to_vec(),
            server_key: server_key.serialize_der(),
        }
    })
}

fn current_utc_year() -> i32 {
    let mut days = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        / 86_400;
    let mut year = 1970_i32;
    loop {
        let days_this_year = if is_leap_year(year) { 366 } else { 365 };
        if days < days_this_year {
            return year;
        }
        days -= days_this_year;
        year += 1;
    }
}

fn is_leap_year(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

fn serve_fixture(
    listener: TcpListener,
    tls: Option<Arc<ServerConfig>>,
    exchanges: Vec<PlannedExchange>,
    watch_for_extra: bool,
) -> FixtureOutcome {
    let mut outcome = FixtureOutcome {
        accepted_connections: 0,
        valid_requests: 0,
        unexpected_connections: 0,
        request_failures: 0,
    };
    for exchange in exchanges {
        let Some(stream) = accept_before(&listener, Instant::now() + Duration::from_secs(5)) else {
            break;
        };
        outcome.accepted_connections += 1;
        stream
            .set_nonblocking(false)
            .expect("the accepted fixture stream should become blocking");
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .expect("the fixture read deadline should be set");
        stream
            .set_write_timeout(Some(Duration::from_secs(5)))
            .expect("the fixture write deadline should be set");

        if let Some(config) = tls.as_ref() {
            let connection =
                ServerConnection::new(Arc::clone(config)).expect("TLS state should initialize");
            let mut stream = StreamOwned::new(connection, stream);
            serve_exchange(&mut stream, exchange, &mut outcome);
        } else {
            let mut stream = stream;
            serve_exchange(&mut stream, exchange, &mut outcome);
        }
    }

    if watch_for_extra {
        let deadline = Instant::now() + Duration::from_millis(300);
        while Instant::now() < deadline {
            match listener.accept() {
                Ok((_stream, _)) => outcome.unexpected_connections += 1,
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(5));
                }
                Err(_) => break,
            }
        }
    }
    outcome
}

fn accept_before(listener: &TcpListener, deadline: Instant) -> Option<TcpStream> {
    loop {
        match listener.accept() {
            Ok((stream, _)) => return Some(stream),
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                if Instant::now() >= deadline {
                    return None;
                }
                thread::sleep(Duration::from_millis(5));
            }
            Err(_) => return None,
        }
    }
}

fn serve_exchange(
    stream: &mut (impl Read + Write),
    exchange: PlannedExchange,
    outcome: &mut FixtureOutcome,
) {
    let request = read_request(stream);
    if request.is_err() {
        outcome.request_failures += 1;
    }
    if let (Some(expected), Ok(request)) = (exchange.expected.as_ref(), request.as_ref()) {
        let matches = match expected {
            RequestExpectation::Current(expected) => request_matches(request, *expected),
            RequestExpectation::Legacy(expected) => legacy_request_matches(request, *expected),
        };
        if matches {
            outcome.valid_requests += 1;
        }
    }
    if let Some(duration) = exchange.stall_for
        && request.is_ok()
    {
        thread::sleep(duration);
        return;
    }
    if let Some(response) = exchange.response {
        if request.is_err() {
            return;
        }
        let _ = write_response(stream, response);
    }
}

struct FixtureRequest {
    verb: String,
    request_target: String,
    fields: BTreeMap<String, Vec<String>>,
    body: Vec<u8>,
}

fn read_request(stream: &mut impl Read) -> io::Result<FixtureRequest> {
    const MAX_REQUEST_BYTES: usize = 1_200_000;
    let mut bytes = Vec::new();
    let mut buffer = [0_u8; 4_096];
    let header_end = loop {
        let read = stream.read(&mut buffer)?;
        if read == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "request ended before its fields",
            ));
        }
        bytes.extend_from_slice(&buffer[..read]);
        if bytes.len() > MAX_REQUEST_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "request exceeded fixture bound",
            ));
        }
        if let Some(index) = bytes.windows(4).position(|window| window == b"\r\n\r\n") {
            break index + 4;
        }
    };
    let head = std::str::from_utf8(&bytes[..header_end])
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "fields were not ASCII"))?;
    let mut lines = head.split("\r\n");
    let request_line = lines
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing request line"))?;
    let mut request_line = request_line.split_whitespace();
    let verb = request_line.next().unwrap_or_default().to_owned();
    if !matches!(verb.as_str(), "POST" | "DELETE") || request_line.next_back() != Some("HTTP/1.1") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unexpected request line",
        ));
    }
    let request_target = request_line
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing request target"))?
        .to_owned();
    let mut fields = BTreeMap::<String, Vec<String>>::new();
    for line in lines.filter(|line| !line.is_empty()) {
        let (name, value) = line
            .split_once(':')
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid field"))?;
        fields
            .entry(name.to_ascii_lowercase())
            .or_default()
            .push(value.trim().to_owned());
    }
    let content_length = fields
        .get("content-length")
        .and_then(|values| (values.len() == 1).then_some(&values[0]))
        .and_then(|value| value.parse::<usize>().ok())
        .or_else(|| (verb == "DELETE").then_some(0))
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing body length"))?;
    if header_end.saturating_add(content_length) > MAX_REQUEST_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "body exceeded fixture bound",
        ));
    }
    while bytes.len() < header_end + content_length {
        let read = stream.read(&mut buffer)?;
        if read == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "request ended before its body",
            ));
        }
        bytes.extend_from_slice(&buffer[..read]);
    }
    Ok(FixtureRequest {
        verb,
        request_target,
        fields,
        body: bytes[header_end..header_end + content_length].to_vec(),
    })
}

fn request_matches(request: &FixtureRequest, expected: ExpectedRequest) -> bool {
    let body: Value = match serde_json::from_slice(&request.body) {
        Ok(body) => body,
        Err(_) => return false,
    };
    let field = |name: &str| {
        request
            .fields
            .get(name)
            .and_then(|values| (values.len() == 1).then_some(values[0].as_str()))
    };
    let expected_bearer = expected.bearer.map(|token| format!("Bearer {token}"));
    request.verb == "POST"
        && request.request_target == "/mcp"
        && field("host").is_some()
        && field("content-type") == Some("application/json")
        && field("accept") == Some("application/json, text/event-stream")
        && field("accept-encoding") == Some("identity")
        && field("mcp-protocol-version") == Some("2026-07-28")
        && field("mcp-method") == Some(expected.method)
        && field("user-agent").is_some_and(|value| value.starts_with("mcp-doctor/"))
        && body.get("jsonrpc").and_then(Value::as_str) == Some("2.0")
        && body.get("method").and_then(Value::as_str) == Some(expected.method)
        && !request
            .body
            .windows("initialize".len())
            .any(|window| window == b"initialize")
        && match expected.name {
            Some(name) => field("mcp-name") == Some(name),
            None => field("mcp-name").is_none(),
        }
        && match expected_bearer.as_deref() {
            Some(value) => field("authorization") == Some(value),
            None => field("authorization").is_none(),
        }
        && match expected.custom {
            Some((name, value)) => field(&name.to_ascii_lowercase()) == Some(value),
            None => true,
        }
        && match expected.mirrored {
            Some((suffix, value)) => {
                field(&format!("mcp-param-{}", suffix.to_ascii_lowercase())) == Some(value)
            }
            None => true,
        }
}

fn legacy_request_matches(request: &FixtureRequest, expected: LegacyExpectedRequest) -> bool {
    let field = |name: &str| {
        request
            .fields
            .get(name)
            .and_then(|values| (values.len() == 1).then_some(values[0].as_str()))
    };
    let expected_bearer = expected.bearer.map(|token| format!("Bearer {token}"));
    if request.verb != expected.verb
        || request.request_target != "/mcp"
        || field("host").is_none()
        || field("accept") != Some("application/json, text/event-stream")
        || field("accept-encoding") != Some("identity")
        || field("user-agent").is_none_or(|value| !value.starts_with("mcp-doctor/"))
        || field("mcp-method").is_some()
        || field("mcp-name").is_some()
        || request
            .fields
            .keys()
            .any(|name| name.starts_with("mcp-param-"))
        || match expected.protocol_header {
            true => field("mcp-protocol-version") != Some(expected.revision),
            false => field("mcp-protocol-version").is_some(),
        }
        || field("mcp-session-id") != expected.session
        || match expected_bearer.as_deref() {
            Some(value) => field("authorization") != Some(value),
            None => false,
        }
        || match expected.custom {
            Some((name, value)) => field(&name.to_ascii_lowercase()) != Some(value),
            None => false,
        }
    {
        return false;
    }
    let Some(method) = expected.method else {
        return request.body.is_empty() && field("content-type").is_none();
    };
    if field("content-type") != Some("application/json") {
        return false;
    }
    let Ok(body) = serde_json::from_slice::<Value>(&request.body) else {
        return false;
    };
    if body.get("jsonrpc").and_then(Value::as_str) != Some("2.0")
        || body.get("method").and_then(Value::as_str) != Some(method)
    {
        return false;
    }
    match method {
        "initialize" => {
            body.get("id").and_then(Value::as_i64) == Some(1)
                && body["params"]["protocolVersion"] == expected.revision
                && body["params"]["capabilities"] == json!({})
                && body["params"]["clientInfo"]["name"] == "mcp-doctor"
        }
        "notifications/initialized" => body.get("id").is_none() && body.get("params").is_none(),
        "tools/list" => {
            body.get("id").and_then(Value::as_i64).is_some()
                && body["params"].get("_meta").is_none()
                && body["params"].get("cursor").and_then(Value::as_str) == expected.cursor
        }
        "tools/call" => {
            body.get("id").and_then(Value::as_i64).is_some()
                && body["params"].get("_meta").is_none()
                && body["params"].get("name").and_then(Value::as_str) == expected.name
                && body["params"]
                    .get("arguments")
                    .is_some_and(Value::is_object)
                && expected.argument.is_none_or(|(name, value)| {
                    body["params"]["arguments"]
                        .get(name)
                        .and_then(Value::as_str)
                        == Some(value)
                })
        }
        _ => false,
    }
}

fn write_response(stream: &mut impl Write, response: FixtureResponse) -> io::Result<()> {
    let hold_open = response.hold_open;
    write!(
        stream,
        "HTTP/1.1 {} {}\r\nContent-Length: {}\r\nConnection: close\r\n",
        response.status,
        response.reason,
        response.body.len()
    )?;
    for (name, value) in response.fields {
        write!(stream, "{name}: {value}\r\n")?;
    }
    stream.write_all(b"\r\n")?;
    stream.write_all(&response.body)?;
    stream.flush()?;
    if hold_open {
        thread::sleep(Duration::from_secs(2));
    }
    Ok(())
}

fn discovery_response(capabilities: Value) -> FixtureResponse {
    FixtureResponse::json(json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
            "resultType": "complete",
            "supportedVersions": ["2026-07-28"],
            "capabilities": capabilities,
            "ttlMs": 0,
            "cacheScope": "private"
        }
    }))
}

fn tools_response(with_mirrored_field: bool) -> FixtureResponse {
    let mut region = json!({"type": "string"});
    if with_mirrored_field {
        region["x-mcp-header"] = Value::String("Region".to_owned());
    }
    FixtureResponse::json(json!({
        "jsonrpc": "2.0",
        "id": 2,
        "result": {
            "resultType": "complete",
            "ttlMs": 0,
            "cacheScope": "private",
            "tools": [{
                "name": TOOL,
                "annotations": {"readOnlyHint": true, "destructiveHint": false},
                "inputSchema": {
                    "type": "object",
                    "properties": {"region": region},
                    "required": ["region"],
                    "additionalProperties": false
                },
                "outputSchema": {
                    "type": "object",
                    "properties": {"ok": {"type": "boolean"}},
                    "required": ["ok"],
                    "additionalProperties": false
                }
            }]
        }
    }))
}

fn reject_tools_response() -> FixtureResponse {
    FixtureResponse::json(json!({
        "jsonrpc": "2.0",
        "id": 2,
        "result": {
            "resultType": "complete",
            "ttlMs": 0,
            "cacheScope": "private",
            "tools": [{
                "name": TOOL,
                "annotations": {"readOnlyHint": true, "destructiveHint": false},
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "region": {"type": "string"},
                        "synthetic_private_mode_never_report_4a91": {
                            "type": "string",
                            "enum": ["safe", "strict"]
                        }
                    },
                    "required": ["region"],
                    "additionalProperties": false
                },
                "outputSchema": {
                    "type": "object",
                    "properties": {"ok": {"type": "boolean"}},
                    "required": ["ok"],
                    "additionalProperties": false
                }
            }]
        }
    }))
}

fn invalid_params_response(id: i64, message: &str) -> FixtureResponse {
    FixtureResponse::json(json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": -32602,
            "message": message
        }
    }))
}

fn legacy_initialize_response(
    revision: &str,
    capabilities: Value,
    session: Option<&str>,
) -> FixtureResponse {
    let response = FixtureResponse::json(json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
            "protocolVersion": revision,
            "capabilities": capabilities,
            "serverInfo": {"name": "synthetic-legacy", "version": "1.0.0"}
        }
    }));
    match session {
        Some(session) => response.with_session(session),
        None => response,
    }
}

fn legacy_tools_response(id: i64, next_cursor: Option<&str>) -> FixtureResponse {
    let tools = if id == 2 {
        vec![json!({
            "name": "synthetic.legacy-passive",
            "inputSchema": {
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "type": "object",
                "properties": {"query": {"type": "string"}}
            }
        })]
    } else {
        Vec::new()
    };
    let mut result = json!({"tools": tools});
    if let Some(cursor) = next_cursor {
        result["nextCursor"] = Value::String(cursor.to_owned());
    }
    FixtureResponse::json(json!({"jsonrpc": "2.0", "id": id, "result": result}))
}

fn legacy_snapshot_tools_response(id: i64, next_cursor: Option<&str>) -> FixtureResponse {
    let tools = if id == 2 {
        vec![json!({
            "name": "synthetic.legacy-snapshot",
            "description": "synthetic legacy description never persisted 4a91",
            "inputSchema": {
                "type": "object",
                "properties": {"query": {"type": "string"}},
                "additionalProperties": false
            },
            "outputSchema": {
                "type": "object",
                "properties": {"ok": {"type": "boolean"}}
            }
        })]
    } else {
        Vec::new()
    };
    let mut result = json!({"tools": tools});
    if let Some(cursor) = next_cursor {
        result["nextCursor"] = Value::String(cursor.to_owned());
    }
    FixtureResponse::json(json!({"jsonrpc": "2.0", "id": id, "result": result}))
}

fn legacy_active_tools_response(name: &str, task_support: Option<&str>) -> FixtureResponse {
    legacy_active_tools_response_for_revision("2025-11-25", name, task_support)
}

fn legacy_active_tools_response_for_revision(
    revision: &str,
    name: &str,
    task_support: Option<&str>,
) -> FixtureResponse {
    let mut tool = json!({
        "name": name,
        "inputSchema": {
            "type": "object",
            "properties": {
                "region": {
                    "type": "string",
                    "x-mcp-header": "Region"
                }
            },
            "required": ["region"],
            "additionalProperties": false
        },
        "outputSchema": {
            "type": "object",
            "properties": {"ok": {"type": "boolean"}},
            "required": ["ok"],
            "additionalProperties": false
        }
    });
    if let Some(task_support) = task_support {
        tool["execution"] = json!({"taskSupport": task_support});
    }
    if revision == "2025-06-18" {
        tool["inputSchema"]["$schema"] = json!("https://json-schema.org/draft/2020-12/schema");
        tool["outputSchema"]["$schema"] = json!("https://json-schema.org/draft/2020-12/schema");
    }
    FixtureResponse::json(json!({
        "jsonrpc": "2.0",
        "id": 2,
        "result": {"tools": [tool]}
    }))
}

fn legacy_active_call_response() -> FixtureResponse {
    FixtureResponse::json(json!({
        "jsonrpc": "2.0",
        "id": 3,
        "result": {
            "content": [],
            "structuredContent": {"ok": true},
            "isError": false
        }
    }))
}

fn ca_file(environment: &TestEnvironment) -> PathBuf {
    let path = environment.artifact_path("synthetic-ca.pem");
    fs::write(&path, fixture_identity().ca_pem.as_bytes())
        .expect("the synthetic CA should be copied to the disposable root");
    path
}

fn remote_command(environment: &TestEnvironment, command: &str, endpoint: &str) -> Command {
    let mut process = environment.command();
    process.arg(command).arg(endpoint);
    process.arg("--allow-private-network").arg(endpoint);
    if endpoint.starts_with("http://") {
        process.arg("--allow-cleartext-http").arg(endpoint);
    }
    process
}

fn legacy_remote_command(
    environment: &TestEnvironment,
    endpoint: &str,
    revision: &str,
    format: Option<&str>,
) -> Command {
    let mut process = environment.command();
    process
        .arg("inspect")
        .arg("--protocol-version")
        .arg(revision);
    if let Some(format) = format {
        process.arg("--format").arg(format);
    }
    process.arg(endpoint);
    process.arg("--allow-private-network").arg(endpoint);
    if endpoint.starts_with("http://") {
        process.arg("--allow-cleartext-http").arg(endpoint);
    }
    process
}

fn legacy_active_remote_command(
    environment: &TestEnvironment,
    command: &str,
    endpoint: &str,
) -> Command {
    legacy_active_remote_command_for_revision(environment, command, endpoint, "2025-11-25")
}

fn legacy_active_remote_command_for_revision(
    environment: &TestEnvironment,
    command: &str,
    endpoint: &str,
    revision: &str,
) -> Command {
    let mut process = remote_command(environment, command, endpoint);
    process.arg("--protocol-version").arg(revision);
    process
}

fn run(process: &mut Command) -> Output {
    process.output().expect("mcp-doctor should start")
}

fn text(output: &Output) -> (&str, &str) {
    (
        std::str::from_utf8(&output.stdout).expect("STDOUT should be UTF-8"),
        std::str::from_utf8(&output.stderr).expect("STDERR should be UTF-8"),
    )
}

fn assert_redacted(output: &Output, endpoint: &str, extra: &[&str]) {
    let (stdout, stderr) = text(output);
    for forbidden in [endpoint].into_iter().chain(extra.iter().copied()) {
        assert!(!stdout.contains(forbidden));
        assert!(!stderr.contains(forbidden));
    }
}

fn assert_successful_inspection(output: &Output) {
    let (stdout, stderr) = text(output);
    assert!(
        output.status.success(),
        "status={:?}\n{stdout}\n{stderr}",
        output.status.code()
    );
    assert!(stderr.is_empty());
    assert!(stdout.contains("PASS  network.target"));
    assert!(stdout.contains("PASS  network.resolution"));
    assert!(stdout.contains("PASS  transport.http"));
    assert!(stdout.contains("PASS  protocol.envelope"));
}

fn assert_report_artifacts(json_path: &std::path::Path, junit_path: &std::path::Path) {
    let json = fs::read(json_path).expect("the JSON report artifact should exist");
    let junit = fs::read(junit_path).expect("the JUnit report artifact should exist");
    let report = parse_and_validate_report(&json);
    let (document, summary) = parse_and_validate_junit(&junit);
    assert_eq!(report["outcome"], "passed");
    assert_eq!(report["exit_code"], 0);
    assert_eq!(
        summary.tests,
        report["checks"]
            .as_array()
            .map(Vec::len)
            .unwrap_or_default()
    );
    assert_eq!(summary.failures, 0);
    assert!(document.contains("report_outcome=passed\nexit_code=0"));
}

fn legacy_success_exchanges(revision: &'static str) -> Vec<PlannedExchange> {
    vec![
        PlannedExchange::legacy_reply(
            LegacyExpectedRequest::initialize(revision),
            legacy_initialize_response(
                revision,
                json!({"tools": {"listChanged": false}}),
                Some(LEGACY_SESSION),
            ),
        ),
        PlannedExchange::legacy_reply(
            LegacyExpectedRequest::initialized(revision, Some(LEGACY_SESSION)),
            FixtureResponse::accepted(),
        ),
        PlannedExchange::legacy_reply(
            LegacyExpectedRequest::list(revision, Some(LEGACY_SESSION), None),
            legacy_tools_response(2, Some(LEGACY_CURSOR)),
        ),
        PlannedExchange::legacy_reply(
            LegacyExpectedRequest::list(revision, Some(LEGACY_SESSION), Some(LEGACY_CURSOR)),
            legacy_tools_response(3, None),
        ),
        PlannedExchange::legacy_reply(
            LegacyExpectedRequest::delete(revision, LEGACY_SESSION),
            FixtureResponse::status(200, "OK"),
        ),
    ]
}

#[test]
fn explicit_legacy_http_revisions_preserve_session_pagination_and_reporter_parity() {
    for revision in ["2025-11-25", "2025-06-18"] {
        for format in [None, Some("json"), Some("junit")] {
            let label = format.unwrap_or("human");
            let server =
                FixtureServer::spawn(WireMode::Http, legacy_success_exchanges(revision), true);
            let endpoint = server.endpoint();
            let environment = TestEnvironment::new();
            let json_path = environment.artifact_path("legacy-report.json");
            let junit_path = environment.artifact_path("legacy-report.xml");
            let mut command = legacy_remote_command(&environment, &endpoint, revision, format);
            command
                .arg("--json-report")
                .arg(&json_path)
                .arg("--junit-report")
                .arg(&junit_path);
            let output = run(&mut command);
            let outcome = server.finish();
            let (stdout, stderr) = text(&output);
            assert!(
                output.status.success(),
                "{revision}/{label}: {stdout}\n{stderr}"
            );
            assert!(stderr.is_empty());
            assert_eq!(outcome.accepted_connections, 5);
            assert_eq!(outcome.valid_requests, 5);
            assert_eq!(outcome.unexpected_connections, 0);
            assert_report_artifacts(&json_path, &junit_path);
            match format {
                None => assert!(stdout.contains(&format!(
                    "protocol selection · selected {revision} · negotiated {revision}"
                ))),
                Some("json") => {
                    let report = parse_and_validate_report(&output.stdout);
                    assert_eq!(report["protocol_revision"], revision);
                    assert_eq!(report["negotiated_protocol_revision"], revision);
                    assert_eq!(report["outcome"], "passed");
                }
                Some("junit") => {
                    let (document, summary) = parse_and_validate_junit(&output.stdout);
                    assert_eq!(summary.failures, 0);
                    assert!(document.contains(&format!("protocol_revision={revision}")));
                    assert!(document.contains(&format!("negotiated_protocol_revision={revision}")));
                }
                Some(unexpected) => panic!("unexpected report format: {unexpected}"),
            }
            assert_redacted(&output, &endpoint, &[LEGACY_SESSION, LEGACY_CURSOR]);
        }
    }
}

#[test]
fn legacy_http_sse_completes_at_the_matching_response_without_waiting_for_eof() {
    let revision = "2025-11-25";
    let sse = FixtureResponse::sse(
        concat!(
            "id: synthetic-priming-event-never-report-4a91\n",
            "data:\n\n",
            "data: {\"jsonrpc\":\"2.0\",\"id\":2,\"result\":{\"tools\":[]}}\n\n"
        )
        .as_bytes()
        .to_vec(),
    )
    .holding_open();
    let server = FixtureServer::spawn(
        WireMode::Http,
        vec![
            PlannedExchange::legacy_reply(
                LegacyExpectedRequest::initialize(revision),
                legacy_initialize_response(revision, json!({"tools": {}}), None),
            ),
            PlannedExchange::legacy_reply(
                LegacyExpectedRequest::initialized(revision, None),
                FixtureResponse::accepted(),
            ),
            PlannedExchange::legacy_reply(LegacyExpectedRequest::list(revision, None, None), sse),
        ],
        false,
    );
    let endpoint = server.endpoint();
    let environment = TestEnvironment::new();
    let started = Instant::now();
    let output = run(&mut legacy_remote_command(
        &environment,
        &endpoint,
        revision,
        None,
    ));
    let elapsed = started.elapsed();
    let outcome = server.finish();
    assert_successful_inspection(&output);
    assert!(
        elapsed < Duration::from_millis(1_500),
        "elapsed: {elapsed:?}"
    );
    assert_eq!(outcome.valid_requests, 3);
    assert_redacted(
        &output,
        &endpoint,
        &["synthetic-priming-event-never-report-4a91"],
    );
}

#[test]
fn legacy_http_revision_and_session_failures_are_causal_without_reinitialization() {
    for revision in ["2025-11-25", "2025-06-18"] {
        let mismatched = if revision == "2025-11-25" {
            "2025-06-18"
        } else {
            "2025-11-25"
        };
        let server = FixtureServer::spawn(
            WireMode::Http,
            vec![PlannedExchange::legacy_reply(
                LegacyExpectedRequest::initialize(revision),
                legacy_initialize_response(mismatched, json!({"tools": {}}), None),
            )],
            true,
        );
        let endpoint = server.endpoint();
        let environment = TestEnvironment::new();
        let output = run(&mut legacy_remote_command(
            &environment,
            &endpoint,
            revision,
            Some("json"),
        ));
        let outcome = server.finish();
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(outcome.accepted_connections, 1);
        assert_eq!(outcome.valid_requests, 1);
        assert_eq!(outcome.unexpected_connections, 0);
        let report = parse_and_validate_report(&output.stdout);
        assert_eq!(report["protocol_revision"], revision);
        assert_eq!(report["negotiated_protocol_revision"], mismatched);
        assert_eq!(report["primary_diagnosis"]["check_id"], "protocol.revision");
        assert_eq!(
            report["primary_diagnosis"]["findings"][0]["code"],
            "MCP-PROTOCOL-005"
        );
    }

    let revision = "2025-11-25";
    const HEADER_REJECTION_SENTINEL: &str = "synthetic-header-error-never-report-4a91";
    let invalid_session =
        legacy_initialize_response(revision, json!({"tools": {}}), Some(LEGACY_SESSION))
            .with_session("synthetic-duplicate-session-never-report-4a91");
    let mut header_rejection = FixtureResponse::json(json!({
        "jsonrpc": "2.0",
        "id": 2,
        "error": {"code": -32020, "message": HEADER_REJECTION_SENTINEL}
    }));
    header_rejection.status = 400;
    header_rejection.reason = "Bad Request";
    let cases = [
        (
            "invalid-session",
            vec![PlannedExchange::legacy_reply(
                LegacyExpectedRequest::initialize(revision),
                invalid_session,
            )],
            "invalid_session_id",
            1,
        ),
        (
            "missing",
            vec![
                PlannedExchange::legacy_reply(
                    LegacyExpectedRequest::initialize(revision),
                    legacy_initialize_response(revision, json!({"tools": {}}), None),
                ),
                PlannedExchange::legacy_reply(
                    LegacyExpectedRequest::initialized(revision, None),
                    FixtureResponse::accepted(),
                ),
                PlannedExchange::legacy_reply(
                    LegacyExpectedRequest::list(revision, None, None),
                    FixtureResponse::status(400, "Bad Request"),
                ),
            ],
            "session_id_required",
            3,
        ),
        (
            "initialized",
            vec![
                PlannedExchange::legacy_reply(
                    LegacyExpectedRequest::initialize(revision),
                    legacy_initialize_response(
                        revision,
                        json!({"tools": {}}),
                        Some(LEGACY_SESSION),
                    ),
                ),
                PlannedExchange::legacy_reply(
                    LegacyExpectedRequest::initialized(revision, Some(LEGACY_SESSION)),
                    FixtureResponse::status(400, "Bad Request"),
                ),
                PlannedExchange::legacy_reply(
                    LegacyExpectedRequest::delete(revision, LEGACY_SESSION),
                    FixtureResponse::status(200, "OK"),
                ),
            ],
            "initialized_notification_rejected",
            3,
        ),
        (
            "protocol-header",
            vec![
                PlannedExchange::legacy_reply(
                    LegacyExpectedRequest::initialize(revision),
                    legacy_initialize_response(revision, json!({"tools": {}}), None),
                ),
                PlannedExchange::legacy_reply(
                    LegacyExpectedRequest::initialized(revision, None),
                    FixtureResponse::accepted(),
                ),
                PlannedExchange::legacy_reply(
                    LegacyExpectedRequest::list(revision, None, None),
                    header_rejection,
                ),
            ],
            "protocol_version_header_rejected",
            3,
        ),
    ];
    for (name, exchanges, evidence, expected_requests) in cases {
        let server = FixtureServer::spawn(WireMode::Http, exchanges, true);
        let endpoint = server.endpoint();
        let environment = TestEnvironment::new();
        let output = run(&mut legacy_remote_command(
            &environment,
            &endpoint,
            revision,
            None,
        ));
        let outcome = server.finish();
        let (stdout, stderr) = text(&output);
        assert_eq!(output.status.code(), Some(1), "{name}: {stdout}\n{stderr}");
        assert!(stderr.is_empty());
        assert!(stdout.contains(evidence), "{name}: {stdout}");
        assert!(
            stdout.contains("SKIP  protocol.envelope"),
            "{name}: {stdout}"
        );
        assert_eq!(outcome.accepted_connections, expected_requests);
        assert_eq!(outcome.valid_requests, expected_requests);
        assert_eq!(outcome.unexpected_connections, 0);
        assert_redacted(&output, &endpoint, &[LEGACY_SESSION]);
        assert!(!stdout.contains(HEADER_REJECTION_SENTINEL));
    }
}

#[test]
fn legacy_http_changed_and_lost_sessions_stop_pagination_but_still_teardown() {
    let revision = "2025-06-18";
    for (name, page_two, cleanup, evidence, cleanup_fails) in [
        (
            "changed",
            legacy_tools_response(3, None).with_session("synthetic-changed-session-4a91"),
            FixtureResponse::status(200, "OK"),
            "session_id_changed",
            false,
        ),
        (
            "lost",
            FixtureResponse::status(404, "Not Found"),
            FixtureResponse::status(500, "Internal Server Error"),
            "session_lost",
            true,
        ),
    ] {
        let server = FixtureServer::spawn(
            WireMode::Http,
            vec![
                PlannedExchange::legacy_reply(
                    LegacyExpectedRequest::initialize(revision),
                    legacy_initialize_response(
                        revision,
                        json!({"tools": {}}),
                        Some(LEGACY_SESSION),
                    ),
                ),
                PlannedExchange::legacy_reply(
                    LegacyExpectedRequest::initialized(revision, Some(LEGACY_SESSION)),
                    FixtureResponse::accepted(),
                ),
                PlannedExchange::legacy_reply(
                    LegacyExpectedRequest::list(revision, Some(LEGACY_SESSION), None),
                    legacy_tools_response(2, Some(LEGACY_CURSOR)),
                ),
                PlannedExchange::legacy_reply(
                    LegacyExpectedRequest::list(
                        revision,
                        Some(LEGACY_SESSION),
                        Some(LEGACY_CURSOR),
                    ),
                    page_two,
                ),
                PlannedExchange::legacy_reply(
                    LegacyExpectedRequest::delete(revision, LEGACY_SESSION),
                    cleanup,
                ),
            ],
            true,
        );
        let endpoint = server.endpoint();
        let environment = TestEnvironment::new();
        let output = run(&mut legacy_remote_command(
            &environment,
            &endpoint,
            revision,
            None,
        ));
        let outcome = server.finish();
        let (stdout, stderr) = text(&output);
        assert_eq!(output.status.code(), Some(1), "{name}: {stdout}\n{stderr}");
        assert!(stderr.is_empty());
        assert!(stdout.contains(evidence), "{name}: {stdout}");
        assert_eq!(stdout.contains("MCP-SAFETY-002"), cleanup_fails, "{stdout}");
        assert_eq!(outcome.accepted_connections, 5);
        assert_eq!(outcome.valid_requests, 5);
        assert_eq!(outcome.unexpected_connections, 0);
        assert_redacted(
            &output,
            &endpoint,
            &[
                LEGACY_SESSION,
                LEGACY_CURSOR,
                "synthetic-changed-session-4a91",
            ],
        );
    }
}

#[test]
fn legacy_http_teardown_failure_remains_an_independent_safety_finding() {
    let revision = "2025-11-25";
    for stalled in [false, true] {
        let cleanup = if stalled {
            PlannedExchange::legacy_stall(LegacyExpectedRequest::delete(revision, LEGACY_SESSION))
        } else {
            PlannedExchange::legacy_reply(
                LegacyExpectedRequest::delete(revision, LEGACY_SESSION),
                FixtureResponse::status(500, "Internal Server Error"),
            )
        };
        let server = FixtureServer::spawn(
            WireMode::Http,
            vec![
                PlannedExchange::legacy_reply(
                    LegacyExpectedRequest::initialize(revision),
                    legacy_initialize_response(revision, json!({}), Some(LEGACY_SESSION)),
                ),
                PlannedExchange::legacy_reply(
                    LegacyExpectedRequest::initialized(revision, Some(LEGACY_SESSION)),
                    FixtureResponse::accepted(),
                ),
                cleanup,
            ],
            true,
        );
        let endpoint = server.endpoint();
        let environment = TestEnvironment::new();
        let snapshot_path = environment.artifact_path(if stalled {
            "stalled-cleanup-contract.json"
        } else {
            "failed-cleanup-contract.json"
        });
        let mut command = legacy_remote_command(&environment, &endpoint, revision, Some("json"));
        command
            .arg("--snapshot")
            .arg(&snapshot_path)
            .arg("--allow-sensitive-snapshot")
            .arg(&snapshot_path);
        let started = Instant::now();
        let output = run(&mut command);
        let elapsed = started.elapsed();
        let outcome = server.finish();
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(outcome.accepted_connections, 3);
        assert_eq!(outcome.valid_requests, 3);
        assert_eq!(outcome.unexpected_connections, 0);
        let report = parse_and_validate_report(&output.stdout);
        assert_eq!(report["protocol_revision"], revision);
        assert_eq!(report["negotiated_protocol_revision"], revision);
        assert!(
            report["independent_findings"]
                .as_array()
                .is_some_and(|findings| {
                    findings
                        .iter()
                        .any(|finding| finding["code"] == "MCP-SAFETY-002")
                })
        );
        if stalled {
            assert!(
                elapsed < Duration::from_millis(2_750),
                "teardown exceeded its shutdown grace: {elapsed:?}"
            );
        }
        assert!(!snapshot_path.exists());
        assert_redacted(&output, &endpoint, &[LEGACY_SESSION]);
    }
}

fn write_scenario(environment: &TestEnvironment) -> PathBuf {
    let path = environment.artifact_path("remote-scenario.json");
    fs::write(
        &path,
        serde_json::to_vec_pretty(&json!({
            "schema_version": "mcp-doctor.scenario/v1alpha1",
            "tool": TOOL,
            "safety": {"effects": "read_only"},
            "cases": [{
                "id": CASE_ID,
                "arguments": {"region": MIRRORED_VALUE},
                "expect": {
                    "result": "success",
                    "structured_output_schema": {
                        "type": "object",
                        "properties": {"ok": {"type": "boolean"}},
                        "required": ["ok"],
                        "additionalProperties": false
                    }
                }
            }]
        }))
        .expect("the synthetic scenario should serialize"),
    )
    .expect("the synthetic scenario should be writable");
    path
}

#[test]
fn explicit_legacy_http_check_uses_initialize_session_and_legacy_wire_for_json_and_sse() {
    const REVISION: &str = "2025-11-25";
    let responses = [
        ("json", legacy_active_call_response()),
        (
            "sse",
            FixtureResponse::sse(
                concat!(
                    "id: synthetic-active-priming-never-report-4a91\n",
                    "data:\n\n",
                    "data: {\"jsonrpc\":\"2.0\",\"id\":3,\"result\":{",
                    "\"content\":[],\"structuredContent\":{\"ok\":true},",
                    "\"isError\":false}}\n\n"
                )
                .as_bytes()
                .to_vec(),
            ),
        ),
    ];

    for (label, call_response) in responses {
        let server = FixtureServer::spawn(
            WireMode::Http,
            vec![
                PlannedExchange::legacy_reply(
                    LegacyExpectedRequest::initialize(REVISION),
                    legacy_initialize_response(
                        REVISION,
                        json!({"tools": {}}),
                        Some(LEGACY_SESSION),
                    ),
                ),
                PlannedExchange::legacy_reply(
                    LegacyExpectedRequest::initialized(REVISION, Some(LEGACY_SESSION)),
                    FixtureResponse::accepted(),
                ),
                PlannedExchange::legacy_reply(
                    LegacyExpectedRequest::list(REVISION, Some(LEGACY_SESSION), None),
                    legacy_active_tools_response(TOOL, None),
                ),
                PlannedExchange::legacy_reply(
                    LegacyExpectedRequest::call(
                        REVISION,
                        Some(LEGACY_SESSION),
                        TOOL,
                        Some(("region", MIRRORED_VALUE)),
                    ),
                    call_response,
                ),
                PlannedExchange::legacy_reply(
                    LegacyExpectedRequest::delete(REVISION, LEGACY_SESSION),
                    FixtureResponse::status(200, "OK"),
                ),
            ],
            true,
        );
        let endpoint = server.endpoint();
        let environment = TestEnvironment::new();
        let scenario = write_scenario(&environment);
        let json_path = environment.artifact_path("legacy-active-http.json");
        let junit_path = environment.artifact_path("legacy-active-http.xml");
        let mut command = legacy_active_remote_command(&environment, "check", &endpoint);
        command
            .arg("--scenario")
            .arg(&scenario)
            .arg("--allow-tool")
            .arg(TOOL)
            .arg("--format")
            .arg("json")
            .arg("--json-report")
            .arg(&json_path)
            .arg("--junit-report")
            .arg(&junit_path);
        let output = run(&mut command);
        let outcome = server.finish();
        let (_, stderr) = text(&output);
        assert!(
            output.status.success(),
            "{label}: {}\n{stderr}",
            String::from_utf8_lossy(&output.stdout)
        );
        assert!(stderr.is_empty(), "{label}: {stderr}");
        assert_eq!(outcome.accepted_connections, 5, "{label}");
        assert_eq!(outcome.valid_requests, 5, "{label}");
        assert_eq!(outcome.unexpected_connections, 0, "{label}");
        let report = parse_and_validate_report(&output.stdout);
        assert_eq!(report["protocol_revision"], REVISION);
        assert_eq!(report["negotiated_protocol_revision"], REVISION);
        assert_eq!(report["outcome"], "passed");
        assert_report_artifacts(&json_path, &junit_path);
        assert_redacted(
            &output,
            &endpoint,
            &[
                LEGACY_SESSION,
                TOOL,
                CASE_ID,
                MIRRORED_VALUE,
                "mcp-param-region",
                "synthetic-active-priming-never-report-4a91",
                scenario.to_str().unwrap(),
            ],
        );
    }
}

#[test]
fn v2025_06_active_http_check_uses_exact_headers_session_and_json_or_sse() {
    const REVISION: &str = "2025-06-18";
    let responses = [
        ("json", legacy_active_call_response()),
        (
            "sse",
            FixtureResponse::sse(
                concat!(
                    "data: {\"jsonrpc\":\"2.0\",\"id\":3,\"result\":{",
                    "\"content\":[],\"structuredContent\":{\"ok\":true},",
                    "\"isError\":false}}\n\n"
                )
                .as_bytes()
                .to_vec(),
            ),
        ),
    ];

    for (label, call_response) in responses {
        let server = FixtureServer::spawn(
            WireMode::Http,
            vec![
                PlannedExchange::legacy_reply(
                    LegacyExpectedRequest::initialize(REVISION),
                    legacy_initialize_response(
                        REVISION,
                        json!({"tools": {}}),
                        Some(LEGACY_SESSION),
                    ),
                ),
                PlannedExchange::legacy_reply(
                    LegacyExpectedRequest::initialized(REVISION, Some(LEGACY_SESSION)),
                    FixtureResponse::accepted(),
                ),
                PlannedExchange::legacy_reply(
                    LegacyExpectedRequest::list(REVISION, Some(LEGACY_SESSION), None),
                    legacy_active_tools_response_for_revision(REVISION, TOOL, None),
                ),
                PlannedExchange::legacy_reply(
                    LegacyExpectedRequest::call(
                        REVISION,
                        Some(LEGACY_SESSION),
                        TOOL,
                        Some(("region", MIRRORED_VALUE)),
                    ),
                    call_response,
                ),
                PlannedExchange::legacy_reply(
                    LegacyExpectedRequest::delete(REVISION, LEGACY_SESSION),
                    FixtureResponse::status(200, "OK"),
                ),
            ],
            true,
        );
        let endpoint = server.endpoint();
        let environment = TestEnvironment::new();
        let scenario = write_scenario(&environment);
        let json_path = environment.artifact_path("v2025-06-active-http.json");
        let junit_path = environment.artifact_path("v2025-06-active-http.xml");
        let mut command =
            legacy_active_remote_command_for_revision(&environment, "check", &endpoint, REVISION);
        command
            .arg("--scenario")
            .arg(&scenario)
            .arg("--allow-tool")
            .arg(TOOL)
            .arg("--format")
            .arg("json")
            .arg("--json-report")
            .arg(&json_path)
            .arg("--junit-report")
            .arg(&junit_path);
        let output = run(&mut command);
        let outcome = server.finish();
        let (_, stderr) = text(&output);
        assert!(
            output.status.success(),
            "{label}: {}\n{stderr}",
            String::from_utf8_lossy(&output.stdout)
        );
        assert!(stderr.is_empty(), "{label}: {stderr}");
        assert_eq!(outcome.accepted_connections, 5, "{label}");
        assert_eq!(outcome.valid_requests, 5, "{label}");
        assert_eq!(outcome.unexpected_connections, 0, "{label}");
        let report = parse_and_validate_report(&output.stdout);
        assert_eq!(report["protocol_revision"], REVISION);
        assert_eq!(report["negotiated_protocol_revision"], REVISION);
        assert_eq!(report["outcome"], "passed");
        assert_report_artifacts(&json_path, &junit_path);
        assert_redacted(
            &output,
            &endpoint,
            &[
                LEGACY_SESSION,
                TOOL,
                CASE_ID,
                MIRRORED_VALUE,
                "mcp-method",
                "mcp-name",
                "mcp-param-region",
                "x-mcp-header",
                scenario.to_str().unwrap(),
            ],
        );
    }
}

#[test]
fn v2025_06_active_http_schema_ambiguity_stops_before_call_and_still_tears_down() {
    const REVISION: &str = "2025-06-18";
    let server = FixtureServer::spawn(
        WireMode::Http,
        vec![
            PlannedExchange::legacy_reply(
                LegacyExpectedRequest::initialize(REVISION),
                legacy_initialize_response(REVISION, json!({"tools": {}}), Some(LEGACY_SESSION)),
            ),
            PlannedExchange::legacy_reply(
                LegacyExpectedRequest::initialized(REVISION, Some(LEGACY_SESSION)),
                FixtureResponse::accepted(),
            ),
            PlannedExchange::legacy_reply(
                LegacyExpectedRequest::list(REVISION, Some(LEGACY_SESSION), None),
                legacy_active_tools_response(TOOL, None),
            ),
            PlannedExchange::legacy_reply(
                LegacyExpectedRequest::delete(REVISION, LEGACY_SESSION),
                FixtureResponse::status(200, "OK"),
            ),
        ],
        true,
    );
    let endpoint = server.endpoint();
    let environment = TestEnvironment::new();
    let scenario = write_scenario(&environment);
    let mut command =
        legacy_active_remote_command_for_revision(&environment, "check", &endpoint, REVISION);
    command
        .arg("--scenario")
        .arg(&scenario)
        .arg("--allow-tool")
        .arg(TOOL)
        .arg("--format")
        .arg("json");
    let output = run(&mut command);
    let outcome = server.finish();
    let (_, stderr) = text(&output);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(stderr.is_empty());
    assert_eq!(outcome.accepted_connections, 4);
    assert_eq!(outcome.valid_requests, 4);
    assert_eq!(outcome.unexpected_connections, 0);
    let report = parse_and_validate_report(&output.stdout);
    assert_eq!(report["protocol_revision"], REVISION);
    assert_eq!(report["negotiated_protocol_revision"], REVISION);
    assert_eq!(report["primary_diagnosis"]["check_id"], "schema.contracts");
    assert_eq!(
        report["primary_diagnosis"]["findings"][0]["code"],
        "MCP-SCHEMA-002"
    );
    let runtime = report["checks"]
        .as_array()
        .and_then(|checks| {
            checks
                .iter()
                .find(|check| check["id"] == "runtime.tools.case[0]")
        })
        .expect("the blocked runtime case should remain explicit");
    assert_eq!(runtime["state"], "skipped");
    assert_eq!(runtime["blocked_by"]["check_id"], "schema.contracts");
    assert_redacted(
        &output,
        &endpoint,
        &[
            LEGACY_SESSION,
            TOOL,
            MIRRORED_VALUE,
            scenario.to_str().unwrap(),
        ],
    );
}

#[test]
fn explicit_v2025_06_http_break_generates_one_exact_dialect_call() {
    const REVISION: &str = "2025-06-18";
    const GENERATED_TOOL: &str = "synthetic.generated";
    let server = FixtureServer::spawn(
        WireMode::Http,
        vec![
            PlannedExchange::legacy_reply(
                LegacyExpectedRequest::initialize(REVISION),
                legacy_initialize_response(REVISION, json!({"tools": {}}), Some(LEGACY_SESSION)),
            ),
            PlannedExchange::legacy_reply(
                LegacyExpectedRequest::initialized(REVISION, Some(LEGACY_SESSION)),
                FixtureResponse::accepted(),
            ),
            PlannedExchange::legacy_reply(
                LegacyExpectedRequest::list(REVISION, Some(LEGACY_SESSION), None),
                legacy_active_tools_response_for_revision(REVISION, GENERATED_TOOL, None),
            ),
            PlannedExchange::legacy_reply(
                LegacyExpectedRequest::call(REVISION, Some(LEGACY_SESSION), GENERATED_TOOL, None),
                legacy_active_call_response(),
            ),
            PlannedExchange::legacy_reply(
                LegacyExpectedRequest::delete(REVISION, LEGACY_SESSION),
                FixtureResponse::status(200, "OK"),
            ),
        ],
        true,
    );
    let endpoint = server.endpoint();
    let environment = TestEnvironment::new();
    let mut command =
        legacy_active_remote_command_for_revision(&environment, "break", &endpoint, REVISION);
    command
        .arg("--tool")
        .arg(GENERATED_TOOL)
        .arg("--allow-tool")
        .arg(GENERATED_TOOL)
        .arg("--effects")
        .arg("read_only")
        .arg("--cases")
        .arg("1")
        .arg("--seed")
        .arg("8080")
        .arg("--format")
        .arg("json");
    let output = run(&mut command);
    let outcome = server.finish();
    let (_, stderr) = text(&output);
    assert!(
        output.status.success(),
        "{}\n{stderr}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(stderr.is_empty());
    assert_eq!(outcome.accepted_connections, 5);
    assert_eq!(outcome.valid_requests, 5);
    assert_eq!(outcome.unexpected_connections, 0);
    let report = parse_and_validate_report(&output.stdout);
    assert_eq!(report["protocol_revision"], REVISION);
    assert_eq!(report["negotiated_protocol_revision"], REVISION);
    assert_eq!(report["outcome"], "passed");
    assert_redacted(&output, &endpoint, &[LEGACY_SESSION, GENERATED_TOOL]);
}

#[test]
fn legacy_active_http_handshake_negatives_stop_without_fallback_or_replay() {
    const SECRET: &str = "synthetic-handshake-secret-never-report-4a91";
    for revision in ["2025-11-25", "2025-06-18"] {
        let mismatched = if revision == "2025-11-25" {
            "2025-06-18"
        } else {
            "2025-11-25"
        };
        let cases = [
            (
                "mismatch",
                legacy_initialize_response(mismatched, json!({"tools": {}}), None),
                "MCP-PROTOCOL-005",
                Some(mismatched),
            ),
            (
                "malformed",
                FixtureResponse::json(json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "result": {
                        "protocolVersion": {"private": SECRET},
                        "capabilities": {"tools": {}},
                        "serverInfo": {"name": SECRET, "version": "1.0.0"}
                    }
                })),
                "MCP-PROTOCOL-003",
                None,
            ),
            (
                "oversized",
                FixtureResponse {
                    status: 200,
                    reason: "OK",
                    fields: vec![("Content-Type".to_owned(), "application/json".to_owned())],
                    body: vec![b'x'; 1_048_577],
                    hold_open: false,
                },
                "MCP-LIMIT-001",
                None,
            ),
        ];

        for (label, response, expected_code, negotiated) in cases {
            let server = FixtureServer::spawn(
                WireMode::Http,
                vec![PlannedExchange::legacy_reply(
                    LegacyExpectedRequest::initialize(revision),
                    response,
                )],
                true,
            );
            let endpoint = server.endpoint();
            let environment = TestEnvironment::new();
            let scenario = write_scenario(&environment);
            let mut command = legacy_active_remote_command_for_revision(
                &environment,
                "check",
                &endpoint,
                revision,
            );
            command
                .arg("--scenario")
                .arg(&scenario)
                .arg("--allow-tool")
                .arg(TOOL)
                .arg("--format")
                .arg("json");
            let output = run(&mut command);
            let outcome = server.finish();
            let (_, stderr) = text(&output);
            assert_eq!(
                output.status.code(),
                Some(1),
                "{revision} {label}: {stderr}"
            );
            assert!(stderr.is_empty(), "{revision} {label}: {stderr}");
            assert_eq!(outcome.accepted_connections, 1, "{revision} {label}");
            assert_eq!(outcome.valid_requests, 1, "{revision} {label}");
            assert_eq!(outcome.unexpected_connections, 0, "{revision} {label}");
            let report = parse_and_validate_report(&output.stdout);
            assert_eq!(report["protocol_revision"], revision, "{revision} {label}");
            assert_eq!(
                report["primary_diagnosis"]["findings"][0]["code"], expected_code,
                "{revision} {label}: {report}"
            );
            match negotiated {
                Some(negotiated) => {
                    assert_eq!(
                        report["negotiated_protocol_revision"], negotiated,
                        "{revision} {label}"
                    )
                }
                None => assert!(
                    report.get("negotiated_protocol_revision").is_none(),
                    "{revision} {label}: {report}"
                ),
            }
            assert_redacted(
                &output,
                &endpoint,
                &[SECRET, TOOL, MIRRORED_VALUE, scenario.to_str().unwrap()],
            );
        }
    }
}

#[test]
fn legacy_active_http_session_loss_stops_without_reinitialize_or_call_and_tears_down() {
    for revision in ["2025-11-25", "2025-06-18"] {
        let server = FixtureServer::spawn(
            WireMode::Http,
            vec![
                PlannedExchange::legacy_reply(
                    LegacyExpectedRequest::initialize(revision),
                    legacy_initialize_response(
                        revision,
                        json!({"tools": {}}),
                        Some(LEGACY_SESSION),
                    ),
                ),
                PlannedExchange::legacy_reply(
                    LegacyExpectedRequest::initialized(revision, Some(LEGACY_SESSION)),
                    FixtureResponse::accepted(),
                ),
                PlannedExchange::legacy_reply(
                    LegacyExpectedRequest::list(revision, Some(LEGACY_SESSION), None),
                    FixtureResponse::status(404, "Not Found"),
                ),
                PlannedExchange::legacy_reply(
                    LegacyExpectedRequest::delete(revision, LEGACY_SESSION),
                    FixtureResponse::status(200, "OK"),
                ),
            ],
            true,
        );
        let endpoint = server.endpoint();
        let environment = TestEnvironment::new();
        let scenario = write_scenario(&environment);
        let mut command =
            legacy_active_remote_command_for_revision(&environment, "check", &endpoint, revision);
        command
            .arg("--scenario")
            .arg(&scenario)
            .arg("--allow-tool")
            .arg(TOOL)
            .arg("--format")
            .arg("json");
        let output = run(&mut command);
        let outcome = server.finish();
        let (_, stderr) = text(&output);
        assert_eq!(output.status.code(), Some(1), "{revision}: {stderr}");
        assert!(stderr.is_empty());
        assert_eq!(outcome.accepted_connections, 4);
        assert_eq!(outcome.valid_requests, 4);
        assert_eq!(outcome.unexpected_connections, 0);

        let report = parse_and_validate_report(&output.stdout);
        assert_eq!(report["protocol_revision"], revision);
        assert_eq!(report["negotiated_protocol_revision"], revision);
        assert_eq!(report["primary_diagnosis"]["check_id"], "transport.http");
        assert_eq!(
            report["primary_diagnosis"]["findings"][0]["code"],
            "MCP-HTTP-002"
        );
        let transport = report["checks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|check| check["id"] == "transport.http")
            .unwrap();
        assert_eq!(transport["findings"][0]["evidence"]["rule"], "session_lost");
        for id in [
            "discovery.catalogs",
            "schema.contracts",
            "runtime.tools.case[0]",
        ] {
            let check = report["checks"]
                .as_array()
                .unwrap()
                .iter()
                .find(|check| check["id"] == id)
                .unwrap();
            assert_eq!(check["state"], "skipped", "{id}: {check}");
            assert_eq!(check["blocked_by"]["check_id"], "transport.http");
            assert_eq!(check["blocked_by"]["findings"][0]["code"], "MCP-HTTP-002");
        }
        assert_redacted(
            &output,
            &endpoint,
            &[
                LEGACY_SESSION,
                TOOL,
                CASE_ID,
                MIRRORED_VALUE,
                scenario.to_str().unwrap(),
            ],
        );
    }
}

#[test]
fn legacy_http_required_task_support_stops_before_call() {
    const REVISION: &str = "2025-11-25";
    let server = FixtureServer::spawn(
        WireMode::Http,
        vec![
            PlannedExchange::legacy_reply(
                LegacyExpectedRequest::initialize(REVISION),
                legacy_initialize_response(REVISION, json!({"tools": {}}), Some(LEGACY_SESSION)),
            ),
            PlannedExchange::legacy_reply(
                LegacyExpectedRequest::initialized(REVISION, Some(LEGACY_SESSION)),
                FixtureResponse::accepted(),
            ),
            PlannedExchange::legacy_reply(
                LegacyExpectedRequest::list(REVISION, Some(LEGACY_SESSION), None),
                legacy_active_tools_response(TOOL, Some("required")),
            ),
            PlannedExchange::legacy_reply(
                LegacyExpectedRequest::delete(REVISION, LEGACY_SESSION),
                FixtureResponse::status(200, "OK"),
            ),
        ],
        true,
    );
    let endpoint = server.endpoint();
    let environment = TestEnvironment::new();
    let scenario = write_scenario(&environment);
    let mut command = legacy_active_remote_command(&environment, "check", &endpoint);
    command
        .arg("--scenario")
        .arg(&scenario)
        .arg("--allow-tool")
        .arg(TOOL)
        .arg("--format")
        .arg("json");
    let output = run(&mut command);
    let outcome = server.finish();
    let (_, stderr) = text(&output);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(stderr.is_empty());
    assert_eq!(outcome.accepted_connections, 4);
    assert_eq!(outcome.valid_requests, 4);
    assert_eq!(outcome.unexpected_connections, 0);
    let report = parse_and_validate_report(&output.stdout);
    assert_eq!(report["protocol_revision"], REVISION);
    assert_eq!(report["negotiated_protocol_revision"], REVISION);
    assert_eq!(
        report["primary_diagnosis"]["findings"][0]["code"],
        "MCP-ACTIVE-007"
    );
    assert_redacted(&output, &endpoint, &[LEGACY_SESSION, TOOL, MIRRORED_VALUE]);
}

#[test]
fn legacy_http_input_requests_are_incomplete_without_answer_or_retry() {
    const REVISION: &str = "2025-11-25";
    const REQUEST_ID: &str = "synthetic-server-request-never-report-4a91";
    const SECRET: &str = "synthetic-input-request-secret-never-report-4a91";
    let responses = [
        (
            "url-error",
            FixtureResponse::json(json!({
                "jsonrpc": "2.0",
                "id": 3,
                "error": {
                    "code": -32042,
                    "message": SECRET,
                    "data": {
                        "elicitations": [{
                            "mode": "url",
                            "elicitationId": SECRET,
                            "url": "https://synthetic.invalid/private-action?secret=4a91",
                            "message": SECRET
                        }]
                    }
                }
            })),
        ),
        (
            "server-request",
            FixtureResponse::json(json!({
                "jsonrpc": "2.0",
                "id": REQUEST_ID,
                "method": "sampling/createMessage",
                "params": {
                    "maxTokens": 1,
                    "messages": [{"role": "user", "content": SECRET}]
                }
            })),
        ),
        (
            "sse-server-request",
            FixtureResponse::sse(
                format!(
                    "data: {}\n\n",
                    json!({
                        "jsonrpc": "2.0",
                        "id": REQUEST_ID,
                        "method": "sampling/createMessage",
                        "params": {
                            "maxTokens": 1,
                            "messages": [{"role": "user", "content": SECRET}]
                        }
                    })
                )
                .into_bytes(),
            ),
        ),
    ];

    for (label, call_response) in responses {
        let server = FixtureServer::spawn(
            WireMode::Http,
            vec![
                PlannedExchange::legacy_reply(
                    LegacyExpectedRequest::initialize(REVISION),
                    legacy_initialize_response(
                        REVISION,
                        json!({"tools": {}}),
                        Some(LEGACY_SESSION),
                    ),
                ),
                PlannedExchange::legacy_reply(
                    LegacyExpectedRequest::initialized(REVISION, Some(LEGACY_SESSION)),
                    FixtureResponse::accepted(),
                ),
                PlannedExchange::legacy_reply(
                    LegacyExpectedRequest::list(REVISION, Some(LEGACY_SESSION), None),
                    legacy_active_tools_response(TOOL, None),
                ),
                PlannedExchange::legacy_reply(
                    LegacyExpectedRequest::call(
                        REVISION,
                        Some(LEGACY_SESSION),
                        TOOL,
                        Some(("region", MIRRORED_VALUE)),
                    ),
                    call_response,
                ),
                PlannedExchange::legacy_reply(
                    LegacyExpectedRequest::delete(REVISION, LEGACY_SESSION),
                    FixtureResponse::status(200, "OK"),
                ),
            ],
            true,
        );
        let endpoint = server.endpoint();
        let environment = TestEnvironment::new();
        let scenario = write_scenario(&environment);
        let mut command = legacy_active_remote_command(&environment, "check", &endpoint);
        command
            .arg("--scenario")
            .arg(&scenario)
            .arg("--allow-tool")
            .arg(TOOL)
            .arg("--format")
            .arg("json");
        let output = run(&mut command);
        let outcome = server.finish();
        let (_, stderr) = text(&output);
        assert_eq!(output.status.code(), Some(3), "{label}: {stderr}");
        assert!(stderr.is_empty(), "{label}: {stderr}");
        assert_eq!(outcome.accepted_connections, 5, "{label}");
        assert_eq!(outcome.valid_requests, 5, "{label}");
        assert_eq!(outcome.unexpected_connections, 0, "{label}");
        let report = parse_and_validate_report(&output.stdout);
        assert_eq!(report["protocol_revision"], REVISION);
        assert_eq!(report["negotiated_protocol_revision"], REVISION);
        assert_eq!(report["outcome"], "incomplete");
        assert_redacted(
            &output,
            &endpoint,
            &[
                LEGACY_SESSION,
                TOOL,
                MIRRORED_VALUE,
                REQUEST_ID,
                SECRET,
                "synthetic.invalid",
            ],
        );
    }
}

#[test]
fn v2025_06_http_server_request_is_unanswered_incomplete_and_not_retried() {
    const REVISION: &str = "2025-06-18";
    const REQUEST_ID: &str = "synthetic-v2025-06-request-never-report-4a91";
    const SECRET: &str = "synthetic-v2025-06-input-never-report-4a91";
    let response = FixtureResponse::sse(
        format!(
            "data: {}\n\n",
            json!({
                "jsonrpc": "2.0",
                "id": REQUEST_ID,
                "method": "sampling/createMessage",
                "params": {
                    "maxTokens": 1,
                    "messages": [{"role": "user", "content": SECRET}]
                }
            })
        )
        .into_bytes(),
    );
    let server = FixtureServer::spawn(
        WireMode::Http,
        vec![
            PlannedExchange::legacy_reply(
                LegacyExpectedRequest::initialize(REVISION),
                legacy_initialize_response(REVISION, json!({"tools": {}}), Some(LEGACY_SESSION)),
            ),
            PlannedExchange::legacy_reply(
                LegacyExpectedRequest::initialized(REVISION, Some(LEGACY_SESSION)),
                FixtureResponse::accepted(),
            ),
            PlannedExchange::legacy_reply(
                LegacyExpectedRequest::list(REVISION, Some(LEGACY_SESSION), None),
                legacy_active_tools_response_for_revision(REVISION, TOOL, None),
            ),
            PlannedExchange::legacy_reply(
                LegacyExpectedRequest::call(
                    REVISION,
                    Some(LEGACY_SESSION),
                    TOOL,
                    Some(("region", MIRRORED_VALUE)),
                ),
                response,
            ),
            PlannedExchange::legacy_reply(
                LegacyExpectedRequest::delete(REVISION, LEGACY_SESSION),
                FixtureResponse::status(200, "OK"),
            ),
        ],
        true,
    );
    let endpoint = server.endpoint();
    let environment = TestEnvironment::new();
    let scenario = write_scenario(&environment);
    let mut command =
        legacy_active_remote_command_for_revision(&environment, "check", &endpoint, REVISION);
    command
        .arg("--scenario")
        .arg(&scenario)
        .arg("--allow-tool")
        .arg(TOOL)
        .arg("--format")
        .arg("json");
    let output = run(&mut command);
    let outcome = server.finish();
    let (_, stderr) = text(&output);
    assert_eq!(output.status.code(), Some(3), "{stderr}");
    assert!(stderr.is_empty());
    assert_eq!(outcome.accepted_connections, 5);
    assert_eq!(outcome.valid_requests, 5);
    assert_eq!(outcome.unexpected_connections, 0);
    let report = parse_and_validate_report(&output.stdout);
    assert_eq!(report["protocol_revision"], REVISION);
    assert_eq!(report["negotiated_protocol_revision"], REVISION);
    assert_eq!(report["outcome"], "incomplete");
    assert_eq!(report["exit_code"], 3);
    assert_redacted(
        &output,
        &endpoint,
        &[
            LEGACY_SESSION,
            TOOL,
            MIRRORED_VALUE,
            REQUEST_ID,
            SECRET,
            scenario.to_str().unwrap(),
        ],
    );
}

#[test]
fn v2025_06_active_http_initialize_timeout_is_bounded_without_retry() {
    const REVISION: &str = "2025-06-18";
    let server = FixtureServer::spawn(
        WireMode::Http,
        vec![PlannedExchange::legacy_timeout(
            LegacyExpectedRequest::initialize(REVISION),
        )],
        true,
    );
    let endpoint = server.endpoint();
    let environment = TestEnvironment::new();
    let scenario = write_scenario(&environment);
    let mut command =
        legacy_active_remote_command_for_revision(&environment, "check", &endpoint, REVISION);
    command
        .arg("--scenario")
        .arg(&scenario)
        .arg("--allow-tool")
        .arg(TOOL)
        .arg("--format")
        .arg("json");
    let started = Instant::now();
    let output = run(&mut command);
    let elapsed = started.elapsed();
    let outcome = server.finish();
    let (_, stderr) = text(&output);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(stderr.is_empty());
    assert_eq!(outcome.accepted_connections, 1);
    assert_eq!(outcome.valid_requests, 1);
    assert_eq!(outcome.unexpected_connections, 0);
    let report = parse_and_validate_report(&output.stdout);
    assert_eq!(report["protocol_revision"], REVISION);
    assert_eq!(
        report["primary_diagnosis"]["check_id"], "network.resolution",
        "{report:#}"
    );
    assert_eq!(
        report["primary_diagnosis"]["findings"][0]["code"],
        "MCP-LIMIT-001"
    );
    let resolution = report["checks"]
        .as_array()
        .and_then(|checks| {
            checks
                .iter()
                .find(|check| check["id"] == "network.resolution")
        })
        .expect("the bounded discovery timeout should remain explicit");
    assert_eq!(
        resolution["findings"][0]["evidence"]["limit"],
        "discovery_time"
    );
    assert!(elapsed >= Duration::from_secs(9), "elapsed: {elapsed:?}");
    assert!(elapsed < Duration::from_secs(20), "elapsed: {elapsed:?}");
    assert_redacted(
        &output,
        &endpoint,
        &[TOOL, MIRRORED_VALUE, scenario.to_str().unwrap()],
    );
}

#[test]
fn explicit_legacy_http_break_generates_one_immediate_call() {
    const REVISION: &str = "2025-11-25";
    const GENERATED_TOOL: &str = "synthetic.generated";
    let server = FixtureServer::spawn(
        WireMode::Http,
        vec![
            PlannedExchange::legacy_reply(
                LegacyExpectedRequest::initialize(REVISION),
                legacy_initialize_response(REVISION, json!({"tools": {}}), Some(LEGACY_SESSION)),
            ),
            PlannedExchange::legacy_reply(
                LegacyExpectedRequest::initialized(REVISION, Some(LEGACY_SESSION)),
                FixtureResponse::accepted(),
            ),
            PlannedExchange::legacy_reply(
                LegacyExpectedRequest::list(REVISION, Some(LEGACY_SESSION), None),
                legacy_active_tools_response(GENERATED_TOOL, None),
            ),
            PlannedExchange::legacy_reply(
                LegacyExpectedRequest::call(REVISION, Some(LEGACY_SESSION), GENERATED_TOOL, None),
                legacy_active_call_response(),
            ),
            PlannedExchange::legacy_reply(
                LegacyExpectedRequest::delete(REVISION, LEGACY_SESSION),
                FixtureResponse::status(200, "OK"),
            ),
        ],
        true,
    );
    let endpoint = server.endpoint();
    let environment = TestEnvironment::new();
    let mut command = legacy_active_remote_command(&environment, "break", &endpoint);
    command
        .arg("--tool")
        .arg(GENERATED_TOOL)
        .arg("--allow-tool")
        .arg(GENERATED_TOOL)
        .arg("--effects")
        .arg("read_only")
        .arg("--cases")
        .arg("1")
        .arg("--seed")
        .arg("8080")
        .arg("--format")
        .arg("json");
    let output = run(&mut command);
    let outcome = server.finish();
    let (_, stderr) = text(&output);
    assert!(
        output.status.success(),
        "{}\n{stderr}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(stderr.is_empty());
    assert_eq!(outcome.accepted_connections, 5);
    assert_eq!(outcome.valid_requests, 5);
    assert_eq!(outcome.unexpected_connections, 0);
    let report = parse_and_validate_report(&output.stdout);
    assert_eq!(report["protocol_revision"], REVISION);
    assert_eq!(report["negotiated_protocol_revision"], REVISION);
    assert_eq!(report["outcome"], "passed");
    assert_redacted(&output, &endpoint, &[LEGACY_SESSION, GENERATED_TOOL]);
}

#[test]
fn legacy_active_http_cleanup_failure_is_an_independent_safety_finding() {
    const REVISION: &str = "2025-11-25";
    let server = FixtureServer::spawn(
        WireMode::Http,
        vec![
            PlannedExchange::legacy_reply(
                LegacyExpectedRequest::initialize(REVISION),
                legacy_initialize_response(REVISION, json!({"tools": {}}), Some(LEGACY_SESSION)),
            ),
            PlannedExchange::legacy_reply(
                LegacyExpectedRequest::initialized(REVISION, Some(LEGACY_SESSION)),
                FixtureResponse::accepted(),
            ),
            PlannedExchange::legacy_reply(
                LegacyExpectedRequest::list(REVISION, Some(LEGACY_SESSION), None),
                legacy_active_tools_response(TOOL, None),
            ),
            PlannedExchange::legacy_reply(
                LegacyExpectedRequest::call(
                    REVISION,
                    Some(LEGACY_SESSION),
                    TOOL,
                    Some(("region", MIRRORED_VALUE)),
                ),
                legacy_active_call_response(),
            ),
            PlannedExchange::legacy_reply(
                LegacyExpectedRequest::delete(REVISION, LEGACY_SESSION),
                FixtureResponse::status(500, "Internal Server Error"),
            ),
        ],
        true,
    );
    let endpoint = server.endpoint();
    let environment = TestEnvironment::new();
    let scenario = write_scenario(&environment);
    let mut command = legacy_active_remote_command(&environment, "check", &endpoint);
    command
        .arg("--scenario")
        .arg(&scenario)
        .arg("--allow-tool")
        .arg(TOOL)
        .arg("--format")
        .arg("json");
    let output = run(&mut command);
    let outcome = server.finish();
    let (_, stderr) = text(&output);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(stderr.is_empty());
    assert_eq!(outcome.accepted_connections, 5);
    assert_eq!(outcome.valid_requests, 5);
    assert_eq!(outcome.unexpected_connections, 0);
    let report = parse_and_validate_report(&output.stdout);
    assert!(
        report["independent_findings"]
            .as_array()
            .is_some_and(|findings| findings
                .iter()
                .any(|finding| finding["code"] == "MCP-SAFETY-002"))
    );
    assert_redacted(&output, &endpoint, &[LEGACY_SESSION, TOOL, MIRRORED_VALUE]);
}

#[test]
fn v2025_06_active_http_cleanup_failure_remains_independent() {
    const REVISION: &str = "2025-06-18";
    let server = FixtureServer::spawn(
        WireMode::Http,
        vec![
            PlannedExchange::legacy_reply(
                LegacyExpectedRequest::initialize(REVISION),
                legacy_initialize_response(REVISION, json!({"tools": {}}), Some(LEGACY_SESSION)),
            ),
            PlannedExchange::legacy_reply(
                LegacyExpectedRequest::initialized(REVISION, Some(LEGACY_SESSION)),
                FixtureResponse::accepted(),
            ),
            PlannedExchange::legacy_reply(
                LegacyExpectedRequest::list(REVISION, Some(LEGACY_SESSION), None),
                legacy_active_tools_response_for_revision(REVISION, TOOL, None),
            ),
            PlannedExchange::legacy_reply(
                LegacyExpectedRequest::call(
                    REVISION,
                    Some(LEGACY_SESSION),
                    TOOL,
                    Some(("region", MIRRORED_VALUE)),
                ),
                legacy_active_call_response(),
            ),
            PlannedExchange::legacy_reply(
                LegacyExpectedRequest::delete(REVISION, LEGACY_SESSION),
                FixtureResponse::status(500, "Internal Server Error"),
            ),
        ],
        true,
    );
    let endpoint = server.endpoint();
    let environment = TestEnvironment::new();
    let scenario = write_scenario(&environment);
    let mut command =
        legacy_active_remote_command_for_revision(&environment, "check", &endpoint, REVISION);
    command
        .arg("--scenario")
        .arg(&scenario)
        .arg("--allow-tool")
        .arg(TOOL)
        .arg("--format")
        .arg("json");
    let output = run(&mut command);
    let outcome = server.finish();
    let (_, stderr) = text(&output);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(stderr.is_empty());
    assert_eq!(outcome.accepted_connections, 5);
    assert_eq!(outcome.valid_requests, 5);
    assert_eq!(outcome.unexpected_connections, 0);
    let report = parse_and_validate_report(&output.stdout);
    assert!(
        report["independent_findings"]
            .as_array()
            .is_some_and(|findings| findings
                .iter()
                .any(|finding| finding["code"] == "MCP-SAFETY-002"))
    );
    let runtime = report["checks"]
        .as_array()
        .and_then(|checks| {
            checks
                .iter()
                .find(|check| check["id"] == "runtime.tools.case[0]")
        })
        .expect("the completed runtime case should remain explicit");
    assert_eq!(runtime["state"], "performed");
    assert_eq!(runtime["outcome"], "passed");
    assert_redacted(
        &output,
        &endpoint,
        &[
            LEGACY_SESSION,
            TOOL,
            MIRRORED_VALUE,
            scenario.to_str().unwrap(),
        ],
    );
}

#[test]
fn loopback_http_requires_exact_gates_and_accepts_json_and_sse() {
    for response in [
        discovery_response(json!({})),
        FixtureResponse::sse(
            concat!(
                "data: {\"jsonrpc\":\"2.0\",\"method\":\"notifications/progress\"}\n\n",
                "data: {\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{",
                "\"resultType\":\"complete\",\"supportedVersions\":[\"2026-07-28\"],",
                "\"capabilities\":{},\"ttlMs\":0,\"cacheScope\":\"private\"}}\n\n"
            )
            .as_bytes()
            .to_vec(),
        ),
    ] {
        let server = FixtureServer::spawn(
            WireMode::Http,
            vec![PlannedExchange::reply(
                ExpectedRequest::method("server/discover"),
                response,
            )],
            false,
        );
        let endpoint = server.endpoint();
        let environment = TestEnvironment::new();
        let mut command = remote_command(&environment, "inspect", &endpoint);
        let output = run(&mut command);
        let outcome = server.finish();

        assert_successful_inspection(&output);
        assert_eq!(outcome.accepted_connections, 1);
        assert_eq!(outcome.valid_requests, 1);
        assert_eq!(outcome.unexpected_connections, 0);
        assert_redacted(&output, &endpoint, &[]);
    }
}

#[test]
fn unsupported_current_revision_is_a_protocol_diagnosis_without_replay_or_fallback() {
    const MESSAGE_SENTINEL: &str = "synthetic-version-message-never-report-4a91";
    const VERSION_SENTINEL: &str = "synthetic-version-value-never-report-4a91";

    for format in [None, Some("json"), Some("junit")] {
        let mut response = FixtureResponse::json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "error": {
                "code": -32022,
                "message": MESSAGE_SENTINEL,
                "data": {
                    "supported": ["2025-11-25", VERSION_SENTINEL],
                    "requested": "2026-07-28"
                }
            }
        }));
        response.status = 400;
        response.reason = "Bad Request";
        let server = FixtureServer::spawn(
            WireMode::Http,
            vec![PlannedExchange::reply(
                ExpectedRequest::method("server/discover"),
                response,
            )],
            true,
        );
        let endpoint = server.endpoint();
        let environment = TestEnvironment::new();
        let mut command = remote_command(&environment, "inspect", &endpoint);
        if let Some(format) = format {
            command.arg("--format").arg(format);
        }
        let output = run(&mut command);
        let outcome = server.finish();

        assert_eq!(output.status.code(), Some(1));
        assert_eq!(outcome.accepted_connections, 1);
        assert_eq!(outcome.valid_requests, 1);
        assert_eq!(outcome.unexpected_connections, 0);
        let (stdout, stderr) = text(&output);
        assert!(stderr.is_empty());
        assert!(!stdout.contains("MCP-HTTP-002"));
        match format {
            None => {
                assert!(stdout.contains("PRIMARY DIAGNOSIS · protocol.revision"));
                assert!(stdout.contains("PASS  transport.http"));
                assert!(stdout.contains("PASS  protocol.envelope"));
                assert!(stdout.contains("FAIL  protocol.revision"));
                assert!(stdout.contains("MCP-PROTOCOL-002 · http.body"));
                assert!(stdout.contains("rule unsupported_protocol_version"));
                assert!(
                    stdout.contains("blocked by protocol.revision (MCP-PROTOCOL-002 at http.body)")
                );
            }
            Some("json") => {
                let report = parse_and_validate_report(&output.stdout);
                assert_eq!(report["primary_diagnosis"]["check_id"], "protocol.revision");
                assert_eq!(
                    report["primary_diagnosis"]["findings"][0]["code"],
                    "MCP-PROTOCOL-002"
                );
                assert_eq!(
                    report["primary_diagnosis"]["findings"][0]["location"],
                    "http.body"
                );
                let revision_check = report["checks"]
                    .as_array()
                    .expect("the stable report has checks")
                    .iter()
                    .find(|check| check["id"] == "protocol.revision")
                    .expect("the stable report has a protocol revision check");
                assert_eq!(
                    revision_check["findings"][0]["evidence"]["rule"],
                    "unsupported_protocol_version"
                );
            }
            Some("junit") => {
                let (document, _) = parse_and_validate_junit(&output.stdout);
                assert!(document.contains("MCP-PROTOCOL-002"));
                assert!(document.contains("primary=true"));
                assert!(document.contains("unsupported_protocol_version"));
            }
            Some(_) => unreachable!("the test selects only stable report formats"),
        }
        assert_redacted(&output, &endpoint, &[MESSAGE_SENTINEL, VERSION_SENTINEL]);
    }

    let mut response = FixtureResponse::json(json!({
        "jsonrpc": "2.0",
        "id": 1,
        "error": {
            "code": -32022,
            "message": MESSAGE_SENTINEL,
            "data": {
                "supported": ["2025-11-25"],
                "requested": "2026-07-28"
            }
        }
    }));
    response.status = 400;
    response.reason = "Bad Request";
    let server = FixtureServer::spawn(
        WireMode::Http,
        vec![PlannedExchange::reply(
            ExpectedRequest::method("server/discover"),
            response,
        )],
        true,
    );
    let endpoint = server.endpoint();
    let environment = TestEnvironment::new();
    let scenario = write_scenario(&environment);
    let mut command = remote_command(&environment, "check", &endpoint);
    command
        .arg("--scenario")
        .arg(&scenario)
        .arg("--allow-tool")
        .arg(TOOL);
    let output = run(&mut command);
    let outcome = server.finish();
    let (stdout, stderr) = text(&output);

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(outcome.accepted_connections, 1);
    assert_eq!(outcome.valid_requests, 1);
    assert_eq!(outcome.unexpected_connections, 0);
    assert!(stderr.is_empty());
    assert!(stdout.contains("PRIMARY DIAGNOSIS · protocol.revision"));
    assert!(stdout.contains("MCP-PROTOCOL-002 · http.body"));
    assert!(stdout.contains("the protocol revision is unsupported"));
    assert!(!stdout.contains("MCP-HTTP-002"));
    assert_redacted(
        &output,
        &endpoint,
        &[MESSAGE_SENTINEL, TOOL, CASE_ID, scenario.to_str().unwrap()],
    );
}

#[test]
fn private_and_cleartext_authority_fail_before_any_connection() {
    let listener =
        TcpListener::bind(("127.0.0.1", 0)).expect("a disposable loopback listener should bind");
    listener
        .set_nonblocking(true)
        .expect("the disposable listener should become nonblocking");
    let endpoint = format!(
        "http://127.0.0.1:{}/mcp",
        listener.local_addr().unwrap().port()
    );
    let environment = TestEnvironment::new();
    let output = run(environment.command().arg("inspect").arg(&endpoint));

    assert_eq!(output.status.code(), Some(1));
    assert!(listener.accept().is_err());
    let (stdout, stderr) = text(&output);
    assert!(stderr.is_empty());
    assert!(stdout.contains("MCP-TARGET-002"));
    assert!(stdout.contains("SKIP  network.resolution"));
    assert_redacted(&output, &endpoint, &[]);
}

#[test]
fn invalid_report_destination_fails_before_remote_resolution_or_connection() {
    let listener =
        TcpListener::bind(("127.0.0.1", 0)).expect("a disposable loopback listener should bind");
    listener
        .set_nonblocking(true)
        .expect("the disposable listener should become nonblocking");
    let endpoint = format!(
        "http://127.0.0.1:{}/mcp",
        listener.local_addr().unwrap().port()
    );
    let environment = TestEnvironment::new();
    let existing = environment.artifact_path("existing-report.json");
    fs::write(&existing, "unchanged").expect("the existing destination should be writable");
    let mut command = remote_command(&environment, "inspect", &endpoint);
    command.arg("--json-report").arg(&existing);
    let output = run(&mut command);
    let (stdout, stderr) = text(&output);

    assert_eq!(output.status.code(), Some(2), "{stdout}\n{stderr}");
    assert!(stdout.is_empty());
    assert!(stderr.contains("already exists"), "{stderr}");
    assert!(listener.accept().is_err(), "artifact preflight connected");
    assert_eq!(fs::read_to_string(&existing).unwrap(), "unchanged");
    assert!(!stderr.contains(&endpoint));
    assert!(!stderr.contains(existing.to_string_lossy().as_ref()));
}

#[test]
fn redirects_status_replays_and_proxy_environment_are_fail_closed() {
    let trap = TcpListener::bind(("127.0.0.1", 0))
        .expect("a disposable redirect and proxy trap should bind");
    trap.set_nonblocking(true)
        .expect("the trap should become nonblocking");
    let trap_url = format!(
        "http://127.0.0.1:{}/trap",
        trap.local_addr().unwrap().port()
    );
    let mut redirect = FixtureResponse::status(307, "Temporary Redirect");
    redirect
        .fields
        .push(("Location".to_owned(), trap_url.clone()));
    let server = FixtureServer::spawn(
        WireMode::Http,
        vec![PlannedExchange::reply(
            ExpectedRequest::method("server/discover"),
            redirect,
        )],
        true,
    );
    let endpoint = server.endpoint();
    let environment = TestEnvironment::new();
    let mut command = remote_command(&environment, "inspect", &endpoint);
    command
        .env("HTTP_PROXY", &trap_url)
        .env("HTTPS_PROXY", &trap_url)
        .env("ALL_PROXY", &trap_url)
        .env("NO_PROXY", "");
    let output = run(&mut command);
    let outcome = server.finish();

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(outcome.accepted_connections, 1);
    assert_eq!(outcome.valid_requests, 1);
    assert_eq!(outcome.unexpected_connections, 0);
    assert!(trap.accept().is_err());
    let (stdout, stderr) = text(&output);
    assert!(stderr.is_empty());
    assert!(stdout.contains("MCP-HTTP-002"));
    assert!(stdout.contains("HTTP status 307"));
    assert_redacted(&output, &endpoint, &[&trap_url]);
}

#[test]
fn authentication_and_retryable_statuses_are_structural_and_never_replayed() {
    const CHALLENGE_SENTINEL: &str =
        "synthetic-challenge-never-report-4a91 https://127.0.0.1/private-metadata";
    const BODY_SENTINEL: &str = "synthetic-status-body-never-report-4a91";
    for (status, reason, code) in [
        (401, "Unauthorized", "MCP-HTTP-AUTH-001"),
        (403, "Forbidden", "MCP-HTTP-AUTH-001"),
        (429, "Too Many Requests", "MCP-HTTP-002"),
    ] {
        let mut response = FixtureResponse::status(status, reason);
        response.fields.push((
            "WWW-Authenticate".to_owned(),
            format!("Bearer realm=\"{CHALLENGE_SENTINEL}\""),
        ));
        response
            .fields
            .push(("Content-Encoding".to_owned(), "gzip".to_owned()));
        response.body = BODY_SENTINEL.as_bytes().to_vec();
        let server = FixtureServer::spawn(
            WireMode::Http,
            vec![PlannedExchange::reply(
                ExpectedRequest::method("server/discover"),
                response,
            )],
            true,
        );
        let endpoint = server.endpoint();
        let environment = TestEnvironment::new();
        let mut command = remote_command(&environment, "inspect", &endpoint);
        let output = run(&mut command);
        let outcome = server.finish();

        assert_eq!(output.status.code(), Some(1));
        assert_eq!(outcome.accepted_connections, 1);
        assert_eq!(outcome.valid_requests, 1);
        assert_eq!(outcome.unexpected_connections, 0);
        let (stdout, stderr) = text(&output);
        assert!(stderr.is_empty());
        assert!(stdout.contains(code));
        assert!(stdout.contains(&format!("HTTP status {status}")));
        assert_redacted(&output, &endpoint, &[CHALLENGE_SENTINEL, BODY_SENTINEL]);
    }
}

#[test]
fn passive_remote_inspection_fans_out_without_calling_or_replaying_an_advertised_tool() {
    let server = FixtureServer::spawn(
        WireMode::Http,
        vec![
            PlannedExchange::reply(
                ExpectedRequest::method("server/discover"),
                discovery_response(json!({"tools": {}})),
            ),
            PlannedExchange::reply(ExpectedRequest::method("tools/list"), tools_response(false)),
        ],
        true,
    );
    let endpoint = server.endpoint();
    let environment = TestEnvironment::new();
    let json_path = environment.artifact_path("remote-report.json");
    let junit_path = environment.artifact_path("remote-report.xml");
    let mut command = remote_command(&environment, "inspect", &endpoint);
    command
        .arg("--json-report")
        .arg(&json_path)
        .arg("--junit-report")
        .arg(&junit_path);
    let output = run(&mut command);
    let outcome = server.finish();

    assert_successful_inspection(&output);
    assert_eq!(outcome.accepted_connections, 2);
    assert_eq!(outcome.valid_requests, 2);
    assert_eq!(outcome.unexpected_connections, 0);
    assert_report_artifacts(&json_path, &junit_path);
    let (stdout, _) = text(&output);
    assert!(stdout.contains("SKIP  runtime.tools"));
    assert_redacted(&output, &endpoint, &[TOOL]);
}

#[test]
fn acknowledged_legacy_http_snapshots_reuse_one_credentialed_passive_session() {
    for (revision, dialect) in [("2025-11-25", "draft_2020_12"), ("2025-06-18", "ambiguous")] {
        let capabilities = if revision == "2025-11-25" {
            json!({
                "tools": {"listChanged": false},
                "logging": {"synthetic": "synthetic-private-http-log-never-report-4a91"},
                "completions": {"synthetic": "synthetic-private-http-completion-never-report-4a91"},
                "experimental": {"synthetic": {"private": "synthetic-private-http-experimental-never-report-4a91"}},
                "tasks": {
                    "list": {"synthetic": "synthetic-private-http-task-never-report-4a91"},
                    "cancel": {},
                    "requests": {"tools": {"call": {"synthetic": true}}}
                }
            })
        } else {
            json!({
                "tools": {"listChanged": false},
                "logging": {"synthetic": "synthetic-private-http-log-never-report-4a91"},
                "completions": {"synthetic": "synthetic-private-http-completion-never-report-4a91"},
                "experimental": {"synthetic": {"private": "synthetic-private-http-experimental-never-report-4a91"}}
            })
        };
        let credentialed = |expected: LegacyExpectedRequest| {
            expected.with_credentials(BEARER_VALUE, (CUSTOM_FIELD, CUSTOM_VALUE))
        };
        let server = FixtureServer::spawn(
            WireMode::Https,
            vec![
                PlannedExchange::legacy_reply(
                    credentialed(LegacyExpectedRequest::initialize(revision)),
                    legacy_initialize_response(revision, capabilities, Some(LEGACY_SESSION)),
                ),
                PlannedExchange::legacy_reply(
                    credentialed(LegacyExpectedRequest::initialized(
                        revision,
                        Some(LEGACY_SESSION),
                    )),
                    FixtureResponse::accepted(),
                ),
                PlannedExchange::legacy_reply(
                    credentialed(LegacyExpectedRequest::list(
                        revision,
                        Some(LEGACY_SESSION),
                        None,
                    )),
                    legacy_snapshot_tools_response(2, Some(LEGACY_CURSOR)),
                ),
                PlannedExchange::legacy_reply(
                    credentialed(LegacyExpectedRequest::list(
                        revision,
                        Some(LEGACY_SESSION),
                        Some(LEGACY_CURSOR),
                    )),
                    legacy_snapshot_tools_response(3, None),
                ),
                PlannedExchange::legacy_reply(
                    credentialed(LegacyExpectedRequest::delete(revision, LEGACY_SESSION)),
                    FixtureResponse::status(200, "OK"),
                ),
            ],
            true,
        );
        let endpoint = server.endpoint();
        let environment = TestEnvironment::new();
        let ca = ca_file(&environment);
        let snapshot_path = environment.artifact_path("legacy-remote-contract.json");
        let mut command = legacy_remote_command(&environment, &endpoint, revision, Some("json"));
        command
            .arg("--snapshot")
            .arg(&snapshot_path)
            .arg("--allow-sensitive-snapshot")
            .arg(&snapshot_path)
            .arg("--allow-credentials-to")
            .arg(&endpoint)
            .arg("--bearer-token-env")
            .arg(BEARER_SOURCE)
            .arg("--header-env")
            .arg(format!("{CUSTOM_FIELD}={CUSTOM_SOURCE}"))
            .arg("--tls-ca-file")
            .arg(&ca)
            .env(BEARER_SOURCE, BEARER_VALUE)
            .env(CUSTOM_SOURCE, CUSTOM_VALUE);
        let output = run(&mut command);
        let outcome = server.finish();
        let (stdout, stderr) = text(&output);

        assert!(output.status.success(), "{revision}: {stdout}\n{stderr}");
        assert!(stderr.is_empty());
        assert_eq!(outcome.accepted_connections, 5);
        assert_eq!(outcome.valid_requests, 5);
        assert_eq!(outcome.unexpected_connections, 0);
        let report = parse_and_validate_report(&output.stdout);
        assert_eq!(report["protocol_revision"], revision);
        assert_eq!(report["negotiated_protocol_revision"], revision);
        assert_eq!(report["outcome"], "passed");

        let bytes = fs::read(&snapshot_path).expect("the legacy HTTP snapshot should exist");
        let snapshot = parse_and_validate_contract_snapshot(&bytes);
        assert_eq!(snapshot["protocol_revision"], revision);
        assert_eq!(snapshot["negotiated_protocol_revision"], revision);
        assert_eq!(snapshot["capabilities"]["logging"]["advertised"], true);
        assert_eq!(snapshot["capabilities"]["completions"]["advertised"], true);
        assert_eq!(
            snapshot["catalogs"]["tools"]["contracts"][0]["input_schema_dialect"],
            dialect
        );
        assert_eq!(
            snapshot["catalogs"]["tools"]["contracts"][0]["output_schema_dialect"],
            dialect
        );
        if revision == "2025-11-25" {
            assert_eq!(snapshot["capabilities"]["tasks"]["list"], true);
            assert_eq!(snapshot["capabilities"]["tasks"]["cancel"], true);
            assert_eq!(
                snapshot["capabilities"]["tasks"]["requests_tools_call"],
                true
            );
        } else {
            assert!(snapshot["capabilities"].get("tasks").is_none());
        }

        let artifact = std::str::from_utf8(&bytes).expect("snapshot should be UTF-8");
        for excluded in [
            endpoint.as_str(),
            BEARER_SOURCE,
            BEARER_VALUE,
            CUSTOM_SOURCE,
            CUSTOM_FIELD,
            CUSTOM_VALUE,
            ca.to_str().expect("CA path should be UTF-8"),
            LEGACY_SESSION,
            LEGACY_CURSOR,
            "synthetic-legacy",
            "synthetic legacy description never persisted 4a91",
            "experimental",
            "synthetic-private-http-log-never-report-4a91",
            "synthetic-private-http-completion-never-report-4a91",
            "synthetic-private-http-experimental-never-report-4a91",
            "synthetic-private-http-task-never-report-4a91",
        ] {
            assert!(!artifact.contains(excluded), "snapshot exposed {excluded}");
            assert!(!stdout.contains(excluded), "report exposed {excluded}");
        }
        assert_redacted(
            &output,
            &endpoint,
            &[
                BEARER_SOURCE,
                BEARER_VALUE,
                CUSTOM_SOURCE,
                CUSTOM_FIELD,
                CUSTOM_VALUE,
                ca.to_str().expect("CA path should be UTF-8"),
                LEGACY_SESSION,
                LEGACY_CURSOR,
            ],
        );
    }
}

#[test]
fn acknowledged_http_snapshot_reuses_one_credentialed_passive_conversation() {
    let expected_discovery = ExpectedRequest {
        method: "server/discover",
        name: None,
        bearer: Some(BEARER_VALUE),
        custom: Some((CUSTOM_FIELD, CUSTOM_VALUE)),
        mirrored: None,
    };
    let expected_tools = ExpectedRequest {
        method: "tools/list",
        ..expected_discovery
    };
    let server = FixtureServer::spawn(
        WireMode::Https,
        vec![
            PlannedExchange::reply(expected_discovery, discovery_response(json!({"tools": {}}))),
            PlannedExchange::reply(expected_tools, tools_response(false)),
        ],
        true,
    );
    let endpoint = server.endpoint();
    let environment = TestEnvironment::new();
    let ca = ca_file(&environment);
    let snapshot_path = environment.artifact_path("remote-contract.json");
    let mut command = remote_command(&environment, "inspect", &endpoint);
    command
        .arg("--format")
        .arg("json")
        .arg("--snapshot")
        .arg(&snapshot_path)
        .arg("--allow-sensitive-snapshot")
        .arg(&snapshot_path)
        .arg("--allow-credentials-to")
        .arg(&endpoint)
        .arg("--bearer-token-env")
        .arg(BEARER_SOURCE)
        .arg("--header-env")
        .arg(format!("{CUSTOM_FIELD}={CUSTOM_SOURCE}"))
        .arg("--tls-ca-file")
        .arg(&ca)
        .env(BEARER_SOURCE, BEARER_VALUE)
        .env(CUSTOM_SOURCE, CUSTOM_VALUE);
    let output = run(&mut command);
    let outcome = server.finish();
    let (stdout, stderr) = text(&output);

    assert!(output.status.success(), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    assert_eq!(outcome.accepted_connections, 2);
    assert_eq!(outcome.valid_requests, 2);
    assert_eq!(outcome.unexpected_connections, 0);
    let report = parse_and_validate_report(&output.stdout);
    assert_eq!(report["outcome"], "passed");
    let bytes = fs::read(&snapshot_path).expect("the HTTP snapshot should exist");
    let snapshot = parse_and_validate_contract_snapshot(&bytes);
    assert_eq!(snapshot["catalogs"]["tools"]["contracts"][0]["name"], TOOL);
    let artifact = std::str::from_utf8(&bytes).expect("snapshot should be UTF-8");
    for excluded in [
        endpoint.as_str(),
        BEARER_SOURCE,
        BEARER_VALUE,
        CUSTOM_SOURCE,
        CUSTOM_FIELD,
        CUSTOM_VALUE,
        ca.to_str().expect("CA path should be UTF-8"),
    ] {
        assert!(!artifact.contains(excluded), "snapshot exposed HTTP input");
    }
    assert_redacted(
        &output,
        &endpoint,
        &[
            TOOL,
            BEARER_SOURCE,
            BEARER_VALUE,
            CUSTOM_SOURCE,
            CUSTOM_FIELD,
            CUSTOM_VALUE,
            ca.to_str().expect("CA path should be UTF-8"),
        ],
    );

    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("an HTTP preflight trap should bind");
    listener
        .set_nonblocking(true)
        .expect("the HTTP preflight trap should be nonblocking");
    let trap_endpoint = format!(
        "http://127.0.0.1:{}/mcp",
        listener.local_addr().expect("trap address").port()
    );
    let rejected = run(environment
        .command()
        .arg("inspect")
        .arg("--snapshot")
        .arg(&snapshot_path)
        .arg("--allow-sensitive-snapshot")
        .arg(&snapshot_path)
        .arg(&trap_endpoint)
        .arg("--allow-private-network")
        .arg(&trap_endpoint)
        .arg("--allow-cleartext-http")
        .arg(&trap_endpoint));
    assert_eq!(rejected.status.code(), Some(2));
    assert!(rejected.stdout.is_empty());
    assert!(
        listener.accept().is_err(),
        "snapshot preflight contacted HTTP"
    );
}

#[test]
fn request_and_response_resource_bounds_stop_at_the_first_excess() {
    let listener =
        TcpListener::bind(("127.0.0.1", 0)).expect("a disposable request-bound trap should bind");
    listener
        .set_nonblocking(true)
        .expect("the request-bound trap should become nonblocking");
    let endpoint = format!(
        "https://127.0.0.1:{}/mcp",
        listener.local_addr().unwrap().port()
    );
    let environment = TestEnvironment::new();
    let ca = ca_file(&environment);
    let mut command = remote_command(&environment, "inspect", &endpoint);
    command
        .arg("--allow-credentials-to")
        .arg(&endpoint)
        .arg("--tls-ca-file")
        .arg(&ca);
    for index in 0..57 {
        let field = format!("X-Synthetic-Bound-{index}");
        let source = format!("MCP_DOCTOR_SYNTHETIC_BOUND_{index}");
        command
            .arg("--header-env")
            .arg(format!("{field}={source}"))
            .env(source, "bounded");
    }
    let output = run(&mut command);
    assert_eq!(output.status.code(), Some(1));
    assert!(listener.accept().is_err());
    let (stdout, stderr) = text(&output);
    assert!(stderr.is_empty());
    assert!(stdout.contains("MCP-LIMIT-001"));
    assert!(stdout.contains("request_fields observed 65 count; maximum 64 count"));
    assert!(stdout.contains("FAIL  network.target"));
    assert!(stdout.contains("SKIP  network.resolution"));
    assert!(!stdout.contains("PASS  transport.tls"));
    assert_redacted(
        &output,
        &endpoint,
        &[
            "X-Synthetic-Bound-0",
            "MCP_DOCTOR_SYNTHETIC_BOUND_0",
            "X-Synthetic-Bound-56",
            "MCP_DOCTOR_SYNTHETIC_BOUND_56",
            ca.to_str().unwrap(),
        ],
    );

    const LARGE_CREDENTIAL_SENTINEL: &str = "synthetic-credential-never-report-4a91";
    let large_credential = format!(
        "{LARGE_CREDENTIAL_SENTINEL}{}",
        "a".repeat(8_193 - LARGE_CREDENTIAL_SENTINEL.len())
    );
    let mut command = remote_command(&environment, "inspect", &endpoint);
    command
        .arg("--allow-credentials-to")
        .arg(&endpoint)
        .arg("--bearer-token-env")
        .arg("MCP_DOCTOR_SYNTHETIC_LARGE_BEARER")
        .arg("--tls-ca-file")
        .arg(&ca)
        .env("MCP_DOCTOR_SYNTHETIC_LARGE_BEARER", &large_credential);
    let output = run(&mut command);
    assert_eq!(output.status.code(), Some(1));
    assert!(listener.accept().is_err());
    let (stdout, stderr) = text(&output);
    assert!(stderr.is_empty());
    assert!(stdout.contains("MCP-LIMIT-001"));
    assert!(stdout.contains("request_field_value_bytes observed 8200 bytes; maximum 8192 bytes"));
    assert!(stdout.contains("FAIL  network.target"));
    assert!(stdout.contains("SKIP  network.resolution"));
    assert!(!stdout.contains("PASS  transport.tls"));
    assert_redacted(
        &output,
        &endpoint,
        &[
            LARGE_CREDENTIAL_SENTINEL,
            "MCP_DOCTOR_SYNTHETIC_LARGE_BEARER",
            ca.to_str().unwrap(),
        ],
    );

    let mut too_many_fields = discovery_response(json!({}));
    for index in 0..94 {
        too_many_fields.fields.push((
            format!("X-Synthetic-Response-{index}"),
            "bounded".to_owned(),
        ));
    }
    let oversized_body = FixtureResponse {
        status: 200,
        reason: "OK",
        fields: vec![("Content-Type".to_owned(), "application/json".to_owned())],
        body: vec![b'x'; 1_048_577],
        hold_open: false,
    };
    let mut events = Vec::new();
    for _ in 0..1_024 {
        events.extend_from_slice(
            b"data: {\"jsonrpc\":\"2.0\",\"method\":\"notifications/progress\"}\n\n",
        );
    }
    events.extend_from_slice(b"data: {\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{}}\n\n");
    let event_overflow = FixtureResponse::sse(events);

    for (response, limit) in [
        (too_many_fields, "response_fields"),
        (oversized_body, "message_bytes"),
        (event_overflow, "message_count"),
    ] {
        let server = FixtureServer::spawn(
            WireMode::Http,
            vec![PlannedExchange::reply(
                ExpectedRequest::method("server/discover"),
                response,
            )],
            false,
        );
        let endpoint = server.endpoint();
        let environment = TestEnvironment::new();
        let mut command = remote_command(&environment, "inspect", &endpoint);
        let output = run(&mut command);
        let outcome = server.finish();
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(outcome.accepted_connections, 1);
        assert_eq!(outcome.valid_requests, 1);
        let (stdout, stderr) = text(&output);
        assert!(stderr.is_empty());
        assert!(stdout.contains("MCP-LIMIT-001"));
        assert!(stdout.contains(limit));
        assert_redacted(&output, &endpoint, &[]);
    }
}

#[test]
fn human_json_and_junit_reports_share_the_same_primary_cause_and_causal_skips() {
    let endpoint = "http://127.0.0.1:9/synthetic-private-path-never-report-4a91";
    let human_environment = TestEnvironment::new();
    let human = run(human_environment.command().arg("inspect").arg(endpoint));
    let json_environment = TestEnvironment::new();
    let json_output = run(json_environment
        .command()
        .arg("inspect")
        .arg("--format")
        .arg("json")
        .arg(endpoint));
    let junit_environment = TestEnvironment::new();
    let junit_output = run(junit_environment
        .command()
        .arg("inspect")
        .arg("--format")
        .arg("junit")
        .arg(endpoint));
    assert_eq!(human.status.code(), Some(1));
    assert_eq!(json_output.status.code(), Some(1));
    assert_eq!(junit_output.status.code(), Some(1));
    let (human_stdout, human_stderr) = text(&human);
    let (json_stdout, json_stderr) = text(&json_output);
    let (_, junit_stderr) = text(&junit_output);
    assert!(human_stderr.is_empty());
    assert!(json_stderr.is_empty());
    assert!(junit_stderr.is_empty());
    assert!(human_stdout.contains("PRIMARY DIAGNOSIS · network.target"));
    assert!(human_stdout.contains("MCP-TARGET-002"));
    assert!(human_stdout.contains("blocked by network.target"));

    let report = parse_and_validate_report(json_stdout.as_bytes());
    assert_eq!(report["primary_diagnosis"]["check_id"], "network.target");
    assert_eq!(
        report["primary_diagnosis"]["findings"][0]["code"],
        "MCP-TARGET-002"
    );
    for id in [
        "network.resolution",
        "transport.tls",
        "transport.http",
        "protocol.envelope",
    ] {
        let check = report["checks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|check| check["id"] == id)
            .unwrap();
        assert_eq!(check["state"], "skipped");
        assert_eq!(check["blocked_by"]["check_id"], "network.target");
        assert_eq!(check["blocked_by"]["findings"][0]["code"], "MCP-TARGET-002");
    }
    let (junit, junit_summary) = parse_and_validate_junit(&junit_output.stdout);
    assert_eq!(junit_summary.failures, 1);
    assert!(junit_summary.skipped > 0);
    assert!(junit.contains("type=\"MCP-TARGET-002\""));
    assert!(junit.contains("blocked_by.check_id=network.target"));
    assert!(junit.contains("report_outcome=failed\nexit_code=1"));
    assert_redacted(&human, endpoint, &[]);
    assert_redacted(&json_output, endpoint, &[]);
    assert_redacted(&junit_output, endpoint, &[]);
}

#[test]
fn verified_tls_delivers_explicit_credentials_without_reporting_them() {
    let expected = ExpectedRequest {
        method: "server/discover",
        name: None,
        bearer: Some(BEARER_VALUE),
        custom: Some((CUSTOM_FIELD, CUSTOM_VALUE)),
        mirrored: None,
    };
    let server = FixtureServer::spawn(
        WireMode::Https,
        vec![PlannedExchange::reply(
            expected,
            discovery_response(json!({})),
        )],
        false,
    );
    let endpoint = server.endpoint();
    let environment = TestEnvironment::new();
    let ca = ca_file(&environment);
    let mut command = remote_command(&environment, "inspect", &endpoint);
    command
        .arg("--allow-credentials-to")
        .arg(&endpoint)
        .arg("--bearer-token-env")
        .arg(BEARER_SOURCE)
        .arg("--header-env")
        .arg(format!("{CUSTOM_FIELD}={CUSTOM_SOURCE}"))
        .arg("--tls-ca-file")
        .arg(&ca)
        .env(BEARER_SOURCE, BEARER_VALUE)
        .env(CUSTOM_SOURCE, CUSTOM_VALUE);
    let output = run(&mut command);
    let outcome = server.finish();

    assert_successful_inspection(&output);
    assert_eq!(outcome.accepted_connections, 1);
    assert_eq!(outcome.valid_requests, 1);
    assert_redacted(
        &output,
        &endpoint,
        &[
            BEARER_SOURCE,
            BEARER_VALUE,
            CUSTOM_SOURCE,
            CUSTOM_FIELD,
            CUSTOM_VALUE,
            ca.to_str().unwrap(),
        ],
    );
}

#[test]
fn tls_identity_and_ambient_trust_fail_without_disclosing_verifier_values() {
    let mismatch = FixtureServer::spawn(
        WireMode::Https,
        vec![PlannedExchange::handshake_only()],
        false,
    );
    let endpoint = mismatch.endpoint_for_host("localhost");
    let environment = TestEnvironment::new();
    let ca = ca_file(&environment);
    let mut command = remote_command(&environment, "inspect", &endpoint);
    command.arg("--tls-ca-file").arg(&ca);
    let output = run(&mut command);
    let outcome = mismatch.finish();
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(outcome.accepted_connections, 1);
    let (stdout, stderr) = text(&output);
    assert!(stderr.is_empty());
    assert!(stdout.contains("MCP-TLS-001"), "{stdout}");
    assert_redacted(&output, &endpoint, &[ca.to_str().unwrap()]);

    let ambient = FixtureServer::spawn(
        WireMode::Https,
        vec![PlannedExchange::handshake_only()],
        false,
    );
    let endpoint = ambient.endpoint();
    let environment = TestEnvironment::new();
    let ca = ca_file(&environment);
    let mut command = remote_command(&environment, "inspect", &endpoint);
    command.env("SSL_CERT_FILE", &ca).env("SSL_CERT_DIR", &ca);
    let output = run(&mut command);
    let outcome = ambient.finish();
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(outcome.accepted_connections, 1);
    let (stdout, stderr) = text(&output);
    assert!(stderr.is_empty());
    assert!(stdout.contains("MCP-TLS-001"), "{stdout}");
    assert_redacted(&output, &endpoint, &[ca.to_str().unwrap()]);
}

#[test]
fn authorized_remote_check_maps_validated_arguments_to_mcp_fields() {
    let discovery = discovery_response(json!({"tools": {}}));
    let tools = tools_response(true);
    let call = FixtureResponse::json(json!({
        "jsonrpc": "2.0",
        "id": 3,
        "result": {
            "resultType": "complete",
            "content": [],
            "structuredContent": {"ok": true},
            "isError": false
        }
    }));
    let server = FixtureServer::spawn(
        WireMode::Https,
        vec![
            PlannedExchange::reply(ExpectedRequest::method("server/discover"), discovery),
            PlannedExchange::reply(ExpectedRequest::method("tools/list"), tools),
            PlannedExchange::reply(
                ExpectedRequest {
                    method: "tools/call",
                    name: Some(TOOL),
                    bearer: None,
                    custom: None,
                    mirrored: Some(("Region", MIRRORED_VALUE)),
                },
                call,
            ),
        ],
        false,
    );
    let endpoint = server.endpoint();
    let environment = TestEnvironment::new();
    let ca = ca_file(&environment);
    let scenario = write_scenario(&environment);
    let json_path = environment.artifact_path("remote-check.json");
    let junit_path = environment.artifact_path("remote-check.xml");
    let mut command = remote_command(&environment, "check", &endpoint);
    command
        .arg("--scenario")
        .arg(&scenario)
        .arg("--allow-tool")
        .arg(TOOL)
        .arg("--json-report")
        .arg(&json_path)
        .arg("--junit-report")
        .arg(&junit_path)
        .arg("--tls-ca-file")
        .arg(&ca);
    let output = run(&mut command);
    let outcome = server.finish();

    assert!(output.status.success());
    assert_eq!(outcome.accepted_connections, 3);
    assert_eq!(outcome.valid_requests, 3);
    assert_report_artifacts(&json_path, &junit_path);
    let (stdout, stderr) = text(&output);
    assert!(stderr.is_empty());
    assert!(stdout.contains("PASS  runtime.tools.case[0]"));
    assert_redacted(
        &output,
        &endpoint,
        &[
            TOOL,
            CASE_ID,
            MIRRORED_VALUE,
            "mcp-param-region",
            ca.to_str().unwrap(),
            scenario.to_str().unwrap(),
        ],
    );
}

#[test]
fn authorized_remote_break_generates_for_only_the_same_exact_endpoint_and_tool() {
    let discovery = discovery_response(json!({"tools": {}}));
    let tools = tools_response(false);
    let call = FixtureResponse::json(json!({
        "jsonrpc": "2.0",
        "id": 3,
        "result": {
            "resultType": "complete",
            "content": [],
            "structuredContent": {"ok": true},
            "isError": false
        }
    }));
    let server = FixtureServer::spawn(
        WireMode::Https,
        vec![
            PlannedExchange::reply(ExpectedRequest::method("server/discover"), discovery),
            PlannedExchange::reply(ExpectedRequest::method("tools/list"), tools),
            PlannedExchange::reply(
                ExpectedRequest {
                    method: "tools/call",
                    name: Some(TOOL),
                    bearer: None,
                    custom: None,
                    mirrored: None,
                },
                call,
            ),
        ],
        false,
    );
    let endpoint = server.endpoint();
    let environment = TestEnvironment::new();
    let ca = ca_file(&environment);
    let json_path = environment.artifact_path("remote-break.json");
    let junit_path = environment.artifact_path("remote-break.xml");
    let mut command = remote_command(&environment, "break", &endpoint);
    command
        .arg("--tool")
        .arg(TOOL)
        .arg("--allow-tool")
        .arg(TOOL)
        .arg("--effects")
        .arg("read_only")
        .arg("--cases")
        .arg("1")
        .arg("--seed")
        .arg("8080")
        .arg("--json-report")
        .arg(&json_path)
        .arg("--junit-report")
        .arg(&junit_path)
        .arg("--tls-ca-file")
        .arg(&ca);
    let output = run(&mut command);
    let outcome = server.finish();

    assert!(output.status.success());
    assert_eq!(outcome.accepted_connections, 3);
    assert_eq!(outcome.valid_requests, 3);
    assert_eq!(outcome.unexpected_connections, 0);
    assert_report_artifacts(&json_path, &junit_path);
    let (stdout, stderr) = text(&output);
    assert!(stderr.is_empty());
    assert!(stdout.contains("PASS  generation.cases"));
    assert!(stdout.contains("PASS  runtime.tools.case[0]"));
    assert!(stdout.contains("mcp-doctor.generator/v1 · seed=8080 · input=object"));
    assert_redacted(
        &output,
        &endpoint,
        &[TOOL, MIRRORED_VALUE, ca.to_str().unwrap()],
    );

    let listener = TcpListener::bind(("127.0.0.1", 0))
        .expect("an unauthorized generated endpoint trap should bind");
    listener
        .set_nonblocking(true)
        .expect("the generated endpoint trap should become nonblocking");
    let endpoint = format!(
        "http://127.0.0.1:{}/mcp",
        listener.local_addr().unwrap().port()
    );
    let environment = TestEnvironment::new();
    let mut rejected = remote_command(&environment, "break", &endpoint);
    rejected
        .arg("--tool")
        .arg(TOOL)
        .arg("--allow-tool")
        .arg("synthetic.remote-other")
        .arg("--effects")
        .arg("read_only")
        .arg("--cases")
        .arg("1")
        .arg("--seed")
        .arg("8080");
    let output = run(&mut rejected);
    let (stdout, stderr) = text(&output);
    assert_eq!(output.status.code(), Some(2), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    assert!(stdout.contains("MCP-AUTH-001"));
    assert!(
        listener.accept().is_err(),
        "authorization rejection connected"
    );
    assert_redacted(&output, &endpoint, &[TOOL, "synthetic.remote-other"]);
}

#[test]
fn authorized_remote_reject_accepts_only_exact_invalid_params_without_retaining_prose() {
    const REJECTION_PROSE: &str = "synthetic private invalid detail never report 4a91";
    let mut exchanges = vec![
        PlannedExchange::reply(
            ExpectedRequest::method("server/discover"),
            discovery_response(json!({"tools": {}})),
        ),
        PlannedExchange::reply(
            ExpectedRequest::method("tools/list"),
            reject_tools_response(),
        ),
    ];
    for id in 3..=9 {
        exchanges.push(PlannedExchange::reply(
            ExpectedRequest {
                method: "tools/call",
                name: Some(TOOL),
                bearer: None,
                custom: None,
                mirrored: None,
            },
            invalid_params_response(id, REJECTION_PROSE),
        ));
    }
    let server = FixtureServer::spawn(WireMode::Http, exchanges, true);
    let endpoint = server.endpoint();
    let environment = TestEnvironment::new();
    let mut command = remote_command(&environment, "reject", &endpoint);
    command
        .arg("--tool")
        .arg(TOOL)
        .arg("--allow-tool")
        .arg(TOOL)
        .arg("--effects")
        .arg("read_only")
        .arg("--seed")
        .arg(u64::MAX.to_string())
        .arg("--format")
        .arg("json");
    let output = run(&mut command);
    let outcome = server.finish();

    let (stdout, stderr) = text(&output);
    assert!(output.status.success(), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    assert_eq!(outcome.accepted_connections, 9);
    assert_eq!(outcome.valid_requests, 9);
    assert_eq!(outcome.unexpected_connections, 0);
    let report = parse_and_validate_report(&output.stdout);
    assert_eq!(report["outcome"], "passed");
    let cases = report["checks"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|check| {
            check["id"]
                .as_str()
                .is_some_and(|id| id.starts_with("runtime.tools.case["))
        })
        .collect::<Vec<_>>();
    assert_eq!(cases.len(), 7);
    assert_eq!(
        cases
            .iter()
            .filter(|check| check["state"] == "performed" && check["outcome"] == "passed")
            .count(),
        7
    );
    assert_eq!(
        cases
            .iter()
            .filter(|check| check["state"] == "skipped")
            .count(),
        0
    );
    assert!(cases.iter().all(|check| match check["state"].as_str() {
        Some("performed") => {
            check["reproduction"]["mutation_kind"].is_string()
                && check["reproduction"].get("arguments").is_none()
        }
        Some("skipped") => check.get("reproduction").is_none(),
        _ => false,
    }));
    assert_redacted(
        &output,
        &endpoint,
        &[
            TOOL,
            REJECTION_PROSE,
            "synthetic_private_mode_never_report_4a91",
            "mcp-doctor-invalid-enum",
        ],
    );
}

#[test]
fn remote_reject_disconnect_is_distinct_and_never_retried() {
    let server = FixtureServer::spawn(
        WireMode::Http,
        vec![
            PlannedExchange::reply(
                ExpectedRequest::method("server/discover"),
                discovery_response(json!({"tools": {}})),
            ),
            PlannedExchange::reply(
                ExpectedRequest::method("tools/list"),
                reject_tools_response(),
            ),
            PlannedExchange::disconnect(ExpectedRequest {
                method: "tools/call",
                name: Some(TOOL),
                bearer: None,
                custom: None,
                mirrored: None,
            }),
        ],
        true,
    );
    let endpoint = server.endpoint();
    let environment = TestEnvironment::new();
    let mut command = remote_command(&environment, "reject", &endpoint);
    command
        .arg("--tool")
        .arg(TOOL)
        .arg("--allow-tool")
        .arg(TOOL)
        .arg("--effects")
        .arg("read_only")
        .arg("--seed")
        .arg("8082")
        .arg("--format")
        .arg("json");
    let output = run(&mut command);
    let outcome = server.finish();

    let report = parse_and_validate_report(&output.stdout);
    assert_eq!(output.status.code(), Some(1), "{report:#}");
    assert_eq!(report["outcome"], "failed");
    assert!(report["checks"].as_array().unwrap().iter().any(|check| {
        check["findings"].as_array().is_some_and(|findings| {
            findings
                .iter()
                .any(|finding| finding["code"] == "MCP-HTTP-001")
        })
    }));
    assert_eq!(outcome.accepted_connections, 3);
    assert_eq!(outcome.valid_requests, 3);
    assert_eq!(outcome.unexpected_connections, 0);
    assert_redacted(&output, &endpoint, &[TOOL]);
}

#[test]
fn remote_reject_skips_a_locally_invalid_case_that_cannot_be_encoded_as_a_mapped_field() {
    let mut exchanges = vec![
        PlannedExchange::reply(
            ExpectedRequest::method("server/discover"),
            discovery_response(json!({"tools": {}})),
        ),
        PlannedExchange::reply(ExpectedRequest::method("tools/list"), tools_response(true)),
    ];
    for id in 3..=7 {
        exchanges.push(PlannedExchange::reply(
            ExpectedRequest {
                method: "tools/call",
                name: Some(TOOL),
                bearer: None,
                custom: None,
                mirrored: None,
            },
            invalid_params_response(id, "synthetic mapped rejection never report"),
        ));
    }
    let server = FixtureServer::spawn(WireMode::Http, exchanges, true);
    let endpoint = server.endpoint();
    let environment = TestEnvironment::new();
    let mut command = remote_command(&environment, "reject", &endpoint);
    command
        .arg("--tool")
        .arg(TOOL)
        .arg("--allow-tool")
        .arg(TOOL)
        .arg("--effects")
        .arg("read_only")
        .arg("--seed")
        .arg("8081")
        .arg("--format")
        .arg("json");
    let output = run(&mut command);
    let outcome = server.finish();

    let report = parse_and_validate_report(&output.stdout);
    assert!(output.status.success(), "{report:#}");
    assert_eq!(outcome.accepted_connections, 7);
    assert_eq!(outcome.valid_requests, 7);
    assert_eq!(outcome.unexpected_connections, 0);
    for (index, state) in [
        "performed",
        "performed",
        "performed",
        "skipped",
        "performed",
        "skipped",
        "performed",
    ]
    .into_iter()
    .enumerate()
    {
        let id = format!("runtime.tools.case[{index}]");
        let case = report["checks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|check| check["id"] == id)
            .unwrap();
        assert_eq!(case["state"], state, "{id}: {case:#}");
        if state == "skipped" {
            assert_eq!(case["skip_reason"], "not_applicable");
        }
    }
    assert_redacted(
        &output,
        &endpoint,
        &[
            TOOL,
            "synthetic mapped rejection never report",
            "mcp-param-region",
        ],
    );
}

#[test]
fn remote_reject_treats_any_result_as_critical_and_stops_later_calls() {
    let server = FixtureServer::spawn(
        WireMode::Http,
        vec![
            PlannedExchange::reply(
                ExpectedRequest::method("server/discover"),
                discovery_response(json!({"tools": {}})),
            ),
            PlannedExchange::reply(ExpectedRequest::method("tools/list"), tools_response(false)),
            PlannedExchange::reply(
                ExpectedRequest {
                    method: "tools/call",
                    name: Some(TOOL),
                    bearer: None,
                    custom: None,
                    mirrored: None,
                },
                FixtureResponse::json(json!({
                    "jsonrpc": "2.0",
                    "id": 3,
                    "result": {
                        "resultType": "input_required",
                        "content": [{"type": "text", "text": "private accepted value 4a91"}],
                        "isError": true
                    }
                })),
            ),
        ],
        true,
    );
    let endpoint = server.endpoint();
    let environment = TestEnvironment::new();
    let mut command = remote_command(&environment, "reject", &endpoint);
    command
        .arg("--tool")
        .arg(TOOL)
        .arg("--allow-tool")
        .arg(TOOL)
        .arg("--effects")
        .arg("read_only")
        .arg("--seed")
        .arg("8080")
        .arg("--format")
        .arg("json");
    let output = run(&mut command);
    let outcome = server.finish();

    let report = parse_and_validate_report(&output.stdout);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(report["outcome"], "failed");
    let finding = report["checks"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|check| check["findings"].as_array().into_iter().flatten())
        .find(|finding| finding["code"] == "MCP-ACTIVE-008")
        .expect("unsafe acceptance should have a dedicated finding");
    assert_eq!(finding["severity"], "critical");
    assert_eq!(outcome.accepted_connections, 3);
    assert_eq!(outcome.valid_requests, 3);
    assert_eq!(outcome.unexpected_connections, 0);
    assert_redacted(&output, &endpoint, &[TOOL, "private accepted value 4a91"]);
}
