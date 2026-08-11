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
use support::{TestEnvironment, parse_and_validate_junit, parse_and_validate_report};

const TOOL: &str = "synthetic.remote-reviewed";
const CASE_ID: &str = "author-only-remote-case-never-report-4a91";
const MIRRORED_VALUE: &str = "synthetic-north-4a91";
const BEARER_SOURCE: &str = "MCP_DOCTOR_SYNTHETIC_BEARER_4A91";
const BEARER_VALUE: &str = "synthetic.bearer-token-4a91";
const CUSTOM_SOURCE: &str = "MCP_DOCTOR_SYNTHETIC_ROUTE_4A91";
const CUSTOM_FIELD: &str = "X-Synthetic-Route-4A91";
const CUSTOM_VALUE: &str = "synthetic-route-value-4a91";

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
    expected: Option<ExpectedRequest>,
    response: Option<FixtureResponse>,
}

impl PlannedExchange {
    fn reply(expected: ExpectedRequest, response: FixtureResponse) -> Self {
        Self {
            expected: Some(expected),
            response: Some(response),
        }
    }

    fn handshake_only() -> Self {
        Self {
            expected: None,
            response: None,
        }
    }
}

struct FixtureResponse {
    status: u16,
    reason: &'static str,
    fields: Vec<(String, String)>,
    body: Vec<u8>,
}

impl FixtureResponse {
    fn json(value: Value) -> Self {
        Self {
            status: 200,
            reason: "OK",
            fields: vec![("Content-Type".to_owned(), "application/json".to_owned())],
            body: serde_json::to_vec(&value).expect("synthetic response should serialize"),
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
        }
    }

    fn status(status: u16, reason: &'static str) -> Self {
        Self {
            status,
            reason,
            fields: vec![("Content-Type".to_owned(), "application/json".to_owned())],
            body: Vec::new(),
        }
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
    if let (Some(expected), Ok(request)) = (exchange.expected, request.as_ref())
        && request_matches(request, expected)
    {
        outcome.valid_requests += 1;
    }
    if let Some(response) = exchange.response {
        if request.is_err() {
            return;
        }
        let _ = write_response(stream, response);
    }
}

struct FixtureRequest {
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
    if request_line.next() != Some("POST") || request_line.next_back() != Some("HTTP/1.1") {
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
    request.request_target == "/mcp"
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

fn write_response(stream: &mut impl Write, response: FixtureResponse) -> io::Result<()> {
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
    stream.flush()
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
fn passive_remote_inspection_never_calls_an_advertised_tool() {
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
    let mut command = remote_command(&environment, "inspect", &endpoint);
    let output = run(&mut command);
    let outcome = server.finish();

    assert_successful_inspection(&output);
    assert_eq!(outcome.accepted_connections, 2);
    assert_eq!(outcome.valid_requests, 2);
    assert_eq!(outcome.unexpected_connections, 0);
    let (stdout, _) = text(&output);
    assert!(stdout.contains("SKIP  runtime.tools"));
    assert_redacted(&output, &endpoint, &[TOOL]);
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
    let mut command = remote_command(&environment, "check", &endpoint);
    command
        .arg("--scenario")
        .arg(&scenario)
        .arg("--allow-tool")
        .arg(TOOL)
        .arg("--tls-ca-file")
        .arg(&ca);
    let output = run(&mut command);
    let outcome = server.finish();

    assert!(output.status.success());
    assert_eq!(outcome.accepted_connections, 3);
    assert_eq!(outcome.valid_requests, 3);
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
        .arg("--tls-ca-file")
        .arg(&ca);
    let output = run(&mut command);
    let outcome = server.finish();

    assert!(output.status.success());
    assert_eq!(outcome.accepted_connections, 3);
    assert_eq!(outcome.valid_requests, 3);
    assert_eq!(outcome.unexpected_connections, 0);
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
