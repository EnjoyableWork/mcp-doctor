//! Bounded Streamable HTTP transport for explicitly selected MCP revisions.

use std::collections::BTreeSet;
use std::fmt;
use std::future::Future;
use std::io::Read;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::OnceLock;
use std::time::Duration;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use reqwest::header::{
    ACCEPT, ACCEPT_ENCODING, AUTHORIZATION, CONTENT_ENCODING, CONTENT_TYPE, HeaderMap, HeaderName,
    HeaderValue, USER_AGENT,
};
use reqwest::{Certificate, Client, StatusCode, Url};
use serde_json::Value;
use tokio::time::Instant;

use super::{Conversation, ProbeRequest, ProbeResponse};
use crate::bound_file::BoundFile;

const PROTOCOL_REVISION: &str = "2026-07-28";
const JSON_MEDIA_TYPE: &str = "application/json";
const SSE_MEDIA_TYPE: &str = "text/event-stream";
const ACCEPT_VALUE: &str = "application/json, text/event-stream";
const USER_AGENT_VALUE: &str = concat!("mcp-doctor/", env!("CARGO_PKG_VERSION"));
const BASE64_PREFIX: &str = "=?base64?";
const BASE64_SUFFIX: &str = "?=";
const MAX_SAFE_INTEGER: i64 = 9_007_199_254_740_991;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct HttpLimits {
    pub(crate) startup_ms: u64,
    pub(crate) discovery_ms: u64,
    pub(crate) request_ms: u64,
    pub(crate) response_ms: u64,
    pub(crate) shutdown_grace_ms: u64,
    pub(crate) total_ms: u64,
    pub(crate) endpoint_bytes: u64,
    pub(crate) resolution_addresses: u64,
    pub(crate) trust_bytes: u64,
    pub(crate) trust_certificates: u64,
    pub(crate) request_fields: u64,
    pub(crate) request_field_name_bytes: u64,
    pub(crate) request_field_value_bytes: u64,
    pub(crate) request_fields_bytes: u64,
    pub(crate) response_fields: u64,
    pub(crate) response_field_name_bytes: u64,
    pub(crate) response_field_value_bytes: u64,
    pub(crate) response_fields_bytes: u64,
    pub(crate) message_bytes: u64,
    pub(crate) aggregate_output_bytes: u64,
    pub(crate) message_count: u64,
    pub(crate) protocol_revisions: u64,
}

#[derive(Clone, Default, Eq, PartialEq)]
pub(crate) struct RemoteOptions {
    pub(crate) endpoint: String,
    pub(crate) allow_private_network: Option<String>,
    pub(crate) allow_cleartext_http: Option<String>,
    pub(crate) allow_credentials_to: Option<String>,
    pub(crate) bearer_token_env: Option<String>,
    pub(crate) header_env: Vec<String>,
    pub(crate) tls_ca_file: Option<PathBuf>,
}

impl fmt::Debug for RemoteOptions {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RemoteOptions")
            .field("endpoint", &"[REDACTED]")
            .field(
                "private_gate_present",
                &self.allow_private_network.is_some(),
            )
            .field(
                "cleartext_gate_present",
                &self.allow_cleartext_http.is_some(),
            )
            .field(
                "credential_gate_present",
                &self.allow_credentials_to.is_some(),
            )
            .field("bearer_present", &self.bearer_token_env.is_some())
            .field("custom_field_count", &self.header_env.len())
            .field("custom_trust_present", &self.tls_ca_file.is_some())
            .finish()
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum HttpLimit {
    StartupTime,
    DiscoveryTime,
    RequestTime,
    ResponseTime,
    TotalTime,
    EndpointBytes,
    ResolutionAddresses,
    TrustBytes,
    TrustCertificates,
    RequestFields,
    RequestFieldNameBytes,
    RequestFieldValueBytes,
    RequestFieldsBytes,
    ResponseFields,
    ResponseFieldNameBytes,
    ResponseFieldValueBytes,
    ResponseFieldsBytes,
    MessageBytes,
    AggregateOutputBytes,
    MessageCount,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum TargetFailure {
    InvalidEndpoint,
    PrivateNetworkAuthorizationRequired,
    CleartextAuthorizationRequired,
    CredentialAuthorizationRequired,
    CredentialsRequireHttps,
    InvalidCredential,
    InvalidCustomField,
    InvalidTrustFile,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum ResolutionFailure {
    Unavailable,
    ProhibitedAddress,
    MixedAddressClasses,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum ResponseFailure {
    Redirect { status: u16 },
    Authentication { status: u16 },
    Status { status: u16 },
    ContentEncoding,
    MediaType,
    InvalidMessage,
    InvalidSse,
    HeaderMismatch,
    InvalidSession,
    SessionChanged,
    SessionRequired { status: u16 },
    SessionLost { status: u16 },
    InitializedRejected { status: u16 },
    ProtocolVersionRejected,
    UnsupportedProtocolVersion,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum HttpFailure {
    Target(TargetFailure),
    Resolution(ResolutionFailure),
    Tls,
    Request,
    ResponseIo,
    PeerMismatch,
    Response(ResponseFailure),
    Limit {
        kind: HttpLimit,
        observed: u64,
        maximum: u64,
    },
}

impl HttpFailure {
    fn limit(kind: HttpLimit, observed: u64, maximum: u64) -> Self {
        debug_assert!(observed > maximum);
        Self::Limit {
            kind,
            observed,
            maximum,
        }
    }

    fn timeout(kind: HttpLimit, maximum: u64) -> Self {
        Self::limit(kind, maximum.saturating_add(1), maximum)
    }
}

pub(crate) trait Resolver: Send + Sync {
    fn resolve<'a>(
        &'a self,
        host: &'a str,
        port: u16,
        maximum: u64,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<SocketAddr>, ()>> + Send + 'a>>;
}

type BodyFuture<'a> =
    Pin<Box<dyn Future<Output = Result<Option<Vec<u8>>, HttpFailure>> + Send + 'a>>;

struct ConnectorRequest {
    endpoint: Url,
    accepted_peers: Vec<SocketAddr>,
    headers: HeaderMap,
    body: Vec<u8>,
    response_ms: u64,
}

impl fmt::Debug for ConnectorRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ConnectorRequest")
            .field("endpoint", &"[REDACTED]")
            .field("accepted_peer_count", &self.accepted_peers.len())
            .field("accepted_peers", &"[REDACTED]")
            .field("field_count", &self.headers.len())
            .field("fields", &"[REDACTED]")
            .field("body_bytes", &self.body.len())
            .field("body", &"[REDACTED]")
            .field("response_ms", &self.response_ms)
            .finish()
    }
}

trait ResponseBody: Send {
    fn next_chunk<'a>(&'a mut self) -> BodyFuture<'a>;
}

struct ConnectorResponse {
    status: StatusCode,
    headers: HeaderMap,
    peer: Option<SocketAddr>,
    body: Box<dyn ResponseBody>,
}

impl fmt::Debug for ConnectorResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ConnectorResponse")
            .field("status", &self.status.as_u16())
            .field("field_count", &self.headers.len())
            .field("fields", &"[REDACTED]")
            .field("peer_present", &self.peer.is_some())
            .field("peer", &"[REDACTED]")
            .field("body", &"[REDACTED]")
            .finish()
    }
}

trait Connector: Send + Sync {
    fn post<'a>(
        &'a self,
        request: ConnectorRequest,
    ) -> Pin<Box<dyn Future<Output = Result<ConnectorResponse, HttpFailure>> + Send + 'a>>;

    fn delete<'a>(
        &'a self,
        _request: ConnectorRequest,
    ) -> Pin<Box<dyn Future<Output = Result<ConnectorResponse, HttpFailure>> + Send + 'a>> {
        Box::pin(async { Err(HttpFailure::Request) })
    }
}

struct ReqwestConnector {
    client: Client,
}

impl Connector for ReqwestConnector {
    fn post<'a>(
        &'a self,
        request: ConnectorRequest,
    ) -> Pin<Box<dyn Future<Output = Result<ConnectorResponse, HttpFailure>> + Send + 'a>> {
        Box::pin(async move {
            let https = request.endpoint.scheme() == "https";
            let send = self
                .client
                .post(request.endpoint)
                .headers(request.headers)
                .body(request.body)
                .timeout(Duration::from_millis(request.response_ms))
                .send();
            let response = send
                .await
                .map_err(|error| classify_send_error(error, https))?;
            let status = response.status();
            let headers = response.headers().clone();
            let peer = response.remote_addr();
            Ok(ConnectorResponse {
                status,
                headers,
                peer,
                body: Box::new(ReqwestBody { response }),
            })
        })
    }

    fn delete<'a>(
        &'a self,
        request: ConnectorRequest,
    ) -> Pin<Box<dyn Future<Output = Result<ConnectorResponse, HttpFailure>> + Send + 'a>> {
        Box::pin(async move {
            let https = request.endpoint.scheme() == "https";
            let response = self
                .client
                .delete(request.endpoint)
                .headers(request.headers)
                .timeout(Duration::from_millis(request.response_ms))
                .send()
                .await
                .map_err(|error| classify_send_error(error, https))?;
            let status = response.status();
            let headers = response.headers().clone();
            let peer = response.remote_addr();
            Ok(ConnectorResponse {
                status,
                headers,
                peer,
                body: Box::new(ReqwestBody { response }),
            })
        })
    }
}

struct ReqwestBody {
    response: reqwest::Response,
}

impl ResponseBody for ReqwestBody {
    fn next_chunk<'a>(&'a mut self) -> BodyFuture<'a> {
        Box::pin(async move {
            self.response
                .chunk()
                .await
                .map(|chunk| chunk.map(|chunk| chunk.to_vec()))
                .map_err(classify_body_error)
        })
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct SystemResolver;

impl Resolver for SystemResolver {
    fn resolve<'a>(
        &'a self,
        host: &'a str,
        port: u16,
        maximum: u64,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<SocketAddr>, ()>> + Send + 'a>> {
        Box::pin(async move {
            tokio::net::lookup_host((host, port))
                .await
                .map(|addresses| collect_resolver_addresses(addresses, maximum))
                .map_err(|_| ())
        })
    }
}

fn collect_resolver_addresses(
    addresses: impl IntoIterator<Item = SocketAddr>,
    maximum: u64,
) -> Vec<SocketAddr> {
    let retained = usize::try_from(maximum.saturating_add(1)).unwrap_or(usize::MAX);
    addresses.into_iter().take(retained).collect()
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum AddressClass {
    Public,
    EligiblePrivate,
    Loopback,
    Prohibited,
}

struct Credentials {
    fields: Vec<(HeaderName, HeaderValue)>,
}

impl Credentials {
    fn none() -> Self {
        Self { fields: Vec::new() }
    }
}

impl fmt::Debug for Credentials {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Credentials")
            .field("field_count", &self.fields.len())
            .field("values", &"[REDACTED]")
            .finish()
    }
}

struct CanonicalEndpoint {
    url: Url,
    host: String,
    port: u16,
    explicit_port: bool,
    https: bool,
}

impl fmt::Debug for CanonicalEndpoint {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CanonicalEndpoint")
            .field("scheme", &if self.https { "https" } else { "http" })
            .field("host", &"[REDACTED]")
            .field("port", &"[REDACTED]")
            .field("explicit_port", &self.explicit_port)
            .field("path", &"[REDACTED]")
            .finish()
    }
}

pub(crate) struct HttpTarget {
    endpoint: CanonicalEndpoint,
    addresses: Vec<SocketAddr>,
    credentials: Credentials,
    trust: Vec<Certificate>,
    limits: HttpLimits,
    started: Instant,
}

impl fmt::Debug for HttpTarget {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HttpTarget")
            .field("endpoint", &self.endpoint)
            .field("address_count", &self.addresses.len())
            .field("addresses", &"[REDACTED]")
            .field("credentials", &self.credentials)
            .field("trust_certificate_count", &self.trust.len())
            .finish()
    }
}

impl HttpTarget {
    pub(crate) async fn prepare<R: Resolver>(
        options: RemoteOptions,
        limits: HttpLimits,
        resolver: &R,
    ) -> Result<Self, HttpFailure> {
        let started = Instant::now();
        let endpoint = parse_endpoint(&options.endpoint, limits.endpoint_bytes)?;
        let private_gate = parse_optional_gate(
            options.allow_private_network.as_deref(),
            limits.endpoint_bytes,
        )?;
        let cleartext_gate = parse_optional_gate(
            options.allow_cleartext_http.as_deref(),
            limits.endpoint_bytes,
        )?;
        let credential_gate = parse_optional_gate(
            options.allow_credentials_to.as_deref(),
            limits.endpoint_bytes,
        )?;

        if !endpoint.https {
            if !gate_matches(private_gate.as_ref(), &endpoint) {
                return Err(HttpFailure::Target(
                    TargetFailure::PrivateNetworkAuthorizationRequired,
                ));
            }
            if !gate_matches(cleartext_gate.as_ref(), &endpoint) {
                return Err(HttpFailure::Target(
                    TargetFailure::CleartextAuthorizationRequired,
                ));
            }
            if options.bearer_token_env.is_some() || !options.header_env.is_empty() {
                return Err(HttpFailure::Target(TargetFailure::CredentialsRequireHttps));
            }
        }

        let credentials_requested =
            options.bearer_token_env.is_some() || !options.header_env.is_empty();
        if credentials_requested && !gate_matches(credential_gate.as_ref(), &endpoint) {
            return Err(HttpFailure::Target(
                TargetFailure::CredentialAuthorizationRequired,
            ));
        }
        let trust = read_trust_file(options.tls_ca_file.as_deref(), limits)?;
        let credentials = resolve_credentials(&options, &endpoint, limits)?;
        ignore_tls_trust_environment();

        let startup_deadline = started + Duration::from_millis(limits.startup_ms);
        let total_deadline = started + Duration::from_millis(limits.total_ms);
        let (resolution_deadline, timeout_kind, timeout_maximum) =
            if total_deadline <= startup_deadline {
                (total_deadline, HttpLimit::TotalTime, limits.total_ms)
            } else {
                (startup_deadline, HttpLimit::StartupTime, limits.startup_ms)
            };
        let raw_addresses = if let Ok(ip) = endpoint.host.parse::<IpAddr>() {
            vec![SocketAddr::new(ip, endpoint.port)]
        } else {
            match tokio::time::timeout_at(
                resolution_deadline,
                resolver.resolve(&endpoint.host, endpoint.port, limits.resolution_addresses),
            )
            .await
            {
                Ok(Ok(addresses)) => addresses,
                Ok(Err(())) => {
                    return Err(HttpFailure::Resolution(ResolutionFailure::Unavailable));
                }
                Err(_) => {
                    return Err(HttpFailure::timeout(timeout_kind, timeout_maximum));
                }
            }
        };
        let addresses =
            validate_addresses(raw_addresses, &endpoint, private_gate.as_ref(), limits)?;

        Ok(Self {
            endpoint,
            addresses,
            credentials,
            trust,
            limits,
            started,
        })
    }
}

#[cfg(all(
    not(test),
    unix,
    not(target_os = "android"),
    not(target_vendor = "apple")
))]
fn ignore_tls_trust_environment() {
    // SAFETY: the binary uses Tokio's current-thread runtime and calls this
    // before the first await or spawned task in remote preparation. Explicit
    // credential values have already been copied. No other application thread
    // can concurrently read or mutate the process environment at this point.
    unsafe {
        std::env::remove_var("SSL_CERT_FILE");
        std::env::remove_var("SSL_CERT_DIR");
    }
}

#[cfg(any(test, not(unix), target_os = "android", target_vendor = "apple"))]
const fn ignore_tls_trust_environment() {}

fn parse_optional_gate(
    value: Option<&str>,
    maximum: u64,
) -> Result<Option<CanonicalEndpoint>, HttpFailure> {
    value
        .map(|value| parse_endpoint(value, maximum))
        .transpose()
}

fn gate_matches(gate: Option<&CanonicalEndpoint>, endpoint: &CanonicalEndpoint) -> bool {
    gate.is_some_and(|gate| gate.url == endpoint.url)
}

fn parse_endpoint(input: &str, maximum: u64) -> Result<CanonicalEndpoint, HttpFailure> {
    let observed = u64::try_from(input.len()).unwrap_or(u64::MAX);
    if observed > maximum {
        return Err(HttpFailure::limit(
            HttpLimit::EndpointBytes,
            observed,
            maximum,
        ));
    }
    let url = Url::parse(input).map_err(|_| HttpFailure::Target(TargetFailure::InvalidEndpoint))?;
    let https = match url.scheme() {
        "https" => true,
        "http" => false,
        _ => return Err(HttpFailure::Target(TargetFailure::InvalidEndpoint)),
    };
    if !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(HttpFailure::Target(TargetFailure::InvalidEndpoint));
    }

    let authority = input
        .split_once("://")
        .map(|(_, rest)| rest.split(['/', '?', '#']).next().unwrap_or_default())
        .ok_or(HttpFailure::Target(TargetFailure::InvalidEndpoint))?;
    if authority.is_empty() || authority.contains(['%', '@']) {
        return Err(HttpFailure::Target(TargetFailure::InvalidEndpoint));
    }
    let (raw_host, raw_port) = split_authority(authority)?;
    let host = url
        .host_str()
        .filter(|host| !host.is_empty())
        .ok_or(HttpFailure::Target(TargetFailure::InvalidEndpoint))?;
    let host = host
        .strip_prefix('[')
        .and_then(|host| host.strip_suffix(']'))
        .unwrap_or(host)
        .to_owned();
    if host.ends_with('.') {
        return Err(HttpFailure::Target(TargetFailure::InvalidEndpoint));
    }
    if let Ok(ipv4) = host.parse::<Ipv4Addr>()
        && raw_host != ipv4.to_string()
    {
        return Err(HttpFailure::Target(TargetFailure::InvalidEndpoint));
    }
    let explicit_port = raw_port.is_some();
    if let Some(raw_port) = raw_port {
        let parsed = raw_port
            .parse::<u16>()
            .ok()
            .filter(|port| *port != 0)
            .ok_or(HttpFailure::Target(TargetFailure::InvalidEndpoint))?;
        if raw_port != parsed.to_string() {
            return Err(HttpFailure::Target(TargetFailure::InvalidEndpoint));
        }
    }
    let port = url
        .port_or_known_default()
        .filter(|port| *port != 0)
        .ok_or(HttpFailure::Target(TargetFailure::InvalidEndpoint))?;

    Ok(CanonicalEndpoint {
        url,
        host,
        port,
        explicit_port,
        https,
    })
}

fn split_authority(authority: &str) -> Result<(&str, Option<&str>), HttpFailure> {
    if let Some(rest) = authority.strip_prefix('[') {
        let (host, suffix) = rest
            .split_once(']')
            .ok_or(HttpFailure::Target(TargetFailure::InvalidEndpoint))?;
        if host.is_empty() || host.contains('%') {
            return Err(HttpFailure::Target(TargetFailure::InvalidEndpoint));
        }
        let port = match suffix {
            "" => None,
            suffix => Some(
                suffix
                    .strip_prefix(':')
                    .filter(|port| !port.is_empty())
                    .ok_or(HttpFailure::Target(TargetFailure::InvalidEndpoint))?,
            ),
        };
        Ok((host, port))
    } else {
        if authority.matches(':').count() > 1 {
            return Err(HttpFailure::Target(TargetFailure::InvalidEndpoint));
        }
        Ok(match authority.rsplit_once(':') {
            Some((host, port)) if !host.is_empty() && !port.is_empty() => (host, Some(port)),
            Some(_) => return Err(HttpFailure::Target(TargetFailure::InvalidEndpoint)),
            None if !authority.is_empty() => (authority, None),
            None => return Err(HttpFailure::Target(TargetFailure::InvalidEndpoint)),
        })
    }
}

fn resolve_credentials(
    options: &RemoteOptions,
    endpoint: &CanonicalEndpoint,
    limits: HttpLimits,
) -> Result<Credentials, HttpFailure> {
    let credential_count =
        usize::from(options.bearer_token_env.is_some()).saturating_add(options.header_env.len());
    if credential_count > 0 {
        let observed = u64::try_from(credential_count.saturating_add(8)).unwrap_or(u64::MAX);
        if observed > limits.request_fields {
            return Err(HttpFailure::limit(
                HttpLimit::RequestFields,
                observed,
                limits.request_fields,
            ));
        }
    }

    let mut fields = Vec::new();
    let mut names = BTreeSet::new();
    if let Some(source) = options.bearer_token_env.as_deref() {
        validate_environment_name(source)
            .map_err(|()| HttpFailure::Target(TargetFailure::InvalidCredential))?;
        let value = std::env::var_os(source)
            .and_then(|value| value.into_string().ok())
            .filter(|value| valid_bearer_token(value))
            .ok_or(HttpFailure::Target(TargetFailure::InvalidCredential))?;
        let observed =
            u64::try_from(value.len().saturating_add("Bearer ".len())).unwrap_or(u64::MAX);
        if observed > limits.request_field_value_bytes {
            return Err(HttpFailure::limit(
                HttpLimit::RequestFieldValueBytes,
                observed,
                limits.request_field_value_bytes,
            ));
        }
        let value = HeaderValue::from_str(&format!("Bearer {value}"))
            .map_err(|_| HttpFailure::Target(TargetFailure::InvalidCredential))?;
        fields.push((AUTHORIZATION, value));
        names.insert("authorization".to_owned());
    }

    for mapping in &options.header_env {
        let (field, source) = mapping
            .split_once('=')
            .filter(|(field, source)| !field.is_empty() && !source.is_empty())
            .ok_or(HttpFailure::Target(TargetFailure::InvalidCustomField))?;
        if !valid_token(field) || reserved_field(field) {
            return Err(HttpFailure::Target(TargetFailure::InvalidCustomField));
        }
        let observed = u64::try_from(field.len()).unwrap_or(u64::MAX);
        if observed > limits.request_field_name_bytes {
            return Err(HttpFailure::limit(
                HttpLimit::RequestFieldNameBytes,
                observed,
                limits.request_field_name_bytes,
            ));
        }
        validate_environment_name(source)
            .map_err(|()| HttpFailure::Target(TargetFailure::InvalidCustomField))?;
        let normalized = field.to_ascii_lowercase();
        if !names.insert(normalized) {
            return Err(HttpFailure::Target(TargetFailure::InvalidCustomField));
        }
        let value = std::env::var_os(source)
            .and_then(|value| value.into_string().ok())
            .filter(|value| valid_custom_value(value))
            .ok_or(HttpFailure::Target(TargetFailure::InvalidCustomField))?;
        let observed = u64::try_from(value.len()).unwrap_or(u64::MAX);
        if observed > limits.request_field_value_bytes {
            return Err(HttpFailure::limit(
                HttpLimit::RequestFieldValueBytes,
                observed,
                limits.request_field_value_bytes,
            ));
        }
        let name = HeaderName::from_bytes(field.as_bytes())
            .map_err(|_| HttpFailure::Target(TargetFailure::InvalidCustomField))?;
        let value = HeaderValue::from_bytes(value.as_bytes())
            .map_err(|_| HttpFailure::Target(TargetFailure::InvalidCustomField))?;
        fields.push((name, value));
    }

    if fields.is_empty() {
        return Ok(Credentials::none());
    }
    validate_credential_field_budget(&fields, endpoint, limits)?;
    Ok(Credentials { fields })
}

fn validate_credential_field_budget(
    fields: &[(HeaderName, HeaderValue)],
    endpoint: &CanonicalEndpoint,
    limits: HttpLimits,
) -> Result<(), HttpFailure> {
    let mut aggregate = 0_u64;
    for (name, value) in fields {
        observe_field(
            name.as_str().len(),
            value.as_bytes().len(),
            true,
            &mut aggregate,
            limits,
        )?;
    }

    // Reserve the fixed fields used by every request, including the longest
    // generated method and the largest permitted Content-Length spelling. This
    // rejects an over-budget credential set before DNS or a connection begins.
    for (name_bytes, value_bytes) in [
        ("content-type".len(), JSON_MEDIA_TYPE.len()),
        ("accept".len(), ACCEPT_VALUE.len()),
        ("accept-encoding".len(), "identity".len()),
        ("user-agent".len(), USER_AGENT_VALUE.len()),
        ("mcp-protocol-version".len(), PROTOCOL_REVISION.len()),
        ("mcp-method".len(), "resources/templates/list".len()),
    ] {
        observe_field(name_bytes, value_bytes, true, &mut aggregate, limits)?;
    }
    let host_value_bytes = endpoint
        .host
        .len()
        .saturating_add(if endpoint.explicit_port {
            endpoint.port.to_string().len().saturating_add(1)
        } else {
            0
        });
    observe_field(4, host_value_bytes, true, &mut aggregate, limits)?;
    observe_field(
        "content-length".len(),
        limits.message_bytes.to_string().len(),
        true,
        &mut aggregate,
        limits,
    )?;
    if aggregate > limits.request_fields_bytes {
        return Err(HttpFailure::limit(
            HttpLimit::RequestFieldsBytes,
            aggregate,
            limits.request_fields_bytes,
        ));
    }
    Ok(())
}

fn validate_environment_name(name: &str) -> Result<(), ()> {
    let mut bytes = name.bytes();
    let Some(first) = bytes.next() else {
        return Err(());
    };
    if !(first == b'_' || first.is_ascii_alphabetic())
        || bytes.any(|byte| !(byte == b'_' || byte.is_ascii_alphanumeric()))
    {
        return Err(());
    }
    Ok(())
}

fn valid_bearer_token(value: &str) -> bool {
    let mut base = value.as_bytes();
    while base.last() == Some(&b'=') {
        base = &base[..base.len().saturating_sub(1)];
    }
    !base.is_empty()
        && base.iter().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~' | b'+' | b'/')
        })
}

fn valid_custom_value(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| matches!(byte, b'\t' | b' '..=b'~'))
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

fn reserved_field(field: &str) -> bool {
    let field = field.to_ascii_lowercase();
    matches!(
        field.as_str(),
        "host"
            | "authorization"
            | "proxy-authorization"
            | "cookie"
            | "origin"
            | "referer"
            | "forwarded"
            | "connection"
            | "te"
            | "trailer"
            | "transfer-encoding"
            | "upgrade"
            | "user-agent"
            | "expect"
            | "keep-alive"
            | "via"
    ) || field.starts_with("x-forwarded-")
        || field.starts_with("content-")
        || field.starts_with("accept")
        || field.starts_with("mcp-")
        || field.starts_with("proxy-")
        || field.starts_with("sec-")
}

fn read_trust_file(
    path: Option<&Path>,
    limits: HttpLimits,
) -> Result<Vec<Certificate>, HttpFailure> {
    let Some(path) = path else {
        return Ok(Vec::new());
    };
    let bound =
        BoundFile::open(path).map_err(|_| HttpFailure::Target(TargetFailure::InvalidTrustFile))?;
    if bound.metadata().len() > limits.trust_bytes {
        return Err(HttpFailure::limit(
            HttpLimit::TrustBytes,
            bound.metadata().len(),
            limits.trust_bytes,
        ));
    }
    let file = bound.into_file();
    let mut bytes = Vec::new();
    file.take(limits.trust_bytes.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|_| HttpFailure::Target(TargetFailure::InvalidTrustFile))?;
    let observed = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
    if observed > limits.trust_bytes {
        return Err(HttpFailure::limit(
            HttpLimit::TrustBytes,
            observed,
            limits.trust_bytes,
        ));
    }
    if bytes
        .windows(b"PRIVATE KEY".len())
        .any(|part| part == b"PRIVATE KEY")
    {
        return Err(HttpFailure::Target(TargetFailure::InvalidTrustFile));
    }
    let certificates = Certificate::from_pem_bundle(&bytes)
        .map_err(|_| HttpFailure::Target(TargetFailure::InvalidTrustFile))?;
    if certificates.is_empty() {
        return Err(HttpFailure::Target(TargetFailure::InvalidTrustFile));
    }
    let count = u64::try_from(certificates.len()).unwrap_or(u64::MAX);
    if count > limits.trust_certificates {
        return Err(HttpFailure::limit(
            HttpLimit::TrustCertificates,
            count,
            limits.trust_certificates,
        ));
    }
    Ok(certificates)
}

fn validate_addresses(
    addresses: Vec<SocketAddr>,
    endpoint: &CanonicalEndpoint,
    private_gate: Option<&CanonicalEndpoint>,
    limits: HttpLimits,
) -> Result<Vec<SocketAddr>, HttpFailure> {
    let mut unique = BTreeSet::new();
    for address in addresses {
        unique.insert(address.ip());
        let observed = u64::try_from(unique.len()).unwrap_or(u64::MAX);
        if observed > limits.resolution_addresses {
            return Err(HttpFailure::limit(
                HttpLimit::ResolutionAddresses,
                observed,
                limits.resolution_addresses,
            ));
        }
    }
    if unique.is_empty() {
        return Err(HttpFailure::Resolution(ResolutionFailure::Unavailable));
    }

    let mut saw_public = false;
    let mut saw_private = false;
    let mut all_loopback = true;
    for address in &unique {
        match classify_address(*address) {
            AddressClass::Public => {
                saw_public = true;
                all_loopback = false;
            }
            AddressClass::EligiblePrivate => {
                saw_private = true;
                all_loopback = false;
            }
            AddressClass::Loopback => saw_private = true,
            AddressClass::Prohibited => {
                return Err(HttpFailure::Resolution(
                    ResolutionFailure::ProhibitedAddress,
                ));
            }
        }
    }
    if saw_public && saw_private {
        return Err(HttpFailure::Resolution(
            ResolutionFailure::MixedAddressClasses,
        ));
    }
    if saw_private && !gate_matches(private_gate, endpoint) {
        return Err(HttpFailure::Target(
            TargetFailure::PrivateNetworkAuthorizationRequired,
        ));
    }
    if !endpoint.https && !all_loopback {
        return Err(HttpFailure::Target(
            TargetFailure::CleartextAuthorizationRequired,
        ));
    }
    Ok(unique
        .into_iter()
        .map(|address| SocketAddr::new(address, endpoint.port))
        .collect())
}

fn classify_address(address: IpAddr) -> AddressClass {
    match address {
        IpAddr::V4(address) => classify_ipv4(address),
        IpAddr::V6(address) => address
            .to_ipv4_mapped()
            .map_or_else(|| classify_ipv6(address), classify_ipv4),
    }
}

fn classify_ipv4(address: Ipv4Addr) -> AddressClass {
    let value = u32::from(address);
    if in_v4(value, Ipv4Addr::new(127, 0, 0, 0), 8) {
        return AddressClass::Loopback;
    }
    if in_v4(value, Ipv4Addr::new(10, 0, 0, 0), 8)
        || in_v4(value, Ipv4Addr::new(172, 16, 0, 0), 12)
        || in_v4(value, Ipv4Addr::new(192, 168, 0, 0), 16)
        || in_v4(value, Ipv4Addr::new(100, 64, 0, 0), 10)
    {
        return AddressClass::EligiblePrivate;
    }
    if address == Ipv4Addr::new(192, 0, 0, 9)
        || address == Ipv4Addr::new(192, 0, 0, 10)
        || in_v4(value, Ipv4Addr::new(192, 31, 196, 0), 24)
        || in_v4(value, Ipv4Addr::new(192, 52, 193, 0), 24)
        || in_v4(value, Ipv4Addr::new(192, 175, 48, 0), 24)
    {
        return AddressClass::Public;
    }
    if in_v4(value, Ipv4Addr::UNSPECIFIED, 8)
        || in_v4(value, Ipv4Addr::new(169, 254, 0, 0), 16)
        || in_v4(value, Ipv4Addr::new(192, 0, 0, 0), 24)
        || in_v4(value, Ipv4Addr::new(192, 0, 2, 0), 24)
        || in_v4(value, Ipv4Addr::new(192, 88, 99, 0), 24)
        || in_v4(value, Ipv4Addr::new(198, 18, 0, 0), 15)
        || in_v4(value, Ipv4Addr::new(198, 51, 100, 0), 24)
        || in_v4(value, Ipv4Addr::new(203, 0, 113, 0), 24)
        || in_v4(value, Ipv4Addr::new(224, 0, 0, 0), 4)
        || in_v4(value, Ipv4Addr::new(240, 0, 0, 0), 4)
    {
        AddressClass::Prohibited
    } else {
        AddressClass::Public
    }
}

fn in_v4(value: u32, network: Ipv4Addr, prefix: u32) -> bool {
    let mask = if prefix == 0 {
        0
    } else {
        u32::MAX << (32 - prefix)
    };
    value & mask == u32::from(network) & mask
}

fn classify_ipv6(address: Ipv6Addr) -> AddressClass {
    let value = u128::from(address);
    if address == Ipv6Addr::LOCALHOST {
        return AddressClass::Loopback;
    }
    if in_v6(value, Ipv6Addr::new(0xfc00, 0, 0, 0, 0, 0, 0, 0), 7) {
        return AddressClass::EligiblePrivate;
    }
    if in_v6(value, Ipv6Addr::new(0x0064, 0xff9b, 0, 0, 0, 0, 0, 0), 96)
        || address == Ipv6Addr::new(0x2001, 1, 0, 0, 0, 0, 0, 1)
        || address == Ipv6Addr::new(0x2001, 1, 0, 0, 0, 0, 0, 2)
        || address == Ipv6Addr::new(0x2001, 1, 0, 0, 0, 0, 0, 3)
        || in_v6(value, Ipv6Addr::new(0x2001, 3, 0, 0, 0, 0, 0, 0), 32)
        || in_v6(value, Ipv6Addr::new(0x2001, 4, 0x0112, 0, 0, 0, 0, 0), 48)
        || in_v6(value, Ipv6Addr::new(0x2001, 0x20, 0, 0, 0, 0, 0, 0), 28)
        || in_v6(value, Ipv6Addr::new(0x2001, 0x30, 0, 0, 0, 0, 0, 0), 28)
        || in_v6(
            value,
            Ipv6Addr::new(0x2620, 0x004f, 0x8000, 0, 0, 0, 0, 0),
            48,
        )
    {
        return AddressClass::Public;
    }
    if in_v6(value, Ipv6Addr::UNSPECIFIED, 96)
        || in_v6(value, Ipv6Addr::new(0x0064, 0xff9b, 1, 0, 0, 0, 0, 0), 48)
        || in_v6(value, Ipv6Addr::new(0x0100, 0, 0, 0, 0, 0, 0, 0), 64)
        || in_v6(value, Ipv6Addr::new(0x0100, 0, 0, 1, 0, 0, 0, 0), 64)
        || in_v6(value, Ipv6Addr::new(0x2001, 0, 0, 0, 0, 0, 0, 0), 23)
        || in_v6(value, Ipv6Addr::new(0x2001, 0x0db8, 0, 0, 0, 0, 0, 0), 32)
        || in_v6(value, Ipv6Addr::new(0x2002, 0, 0, 0, 0, 0, 0, 0), 16)
        || in_v6(value, Ipv6Addr::new(0x3fff, 0, 0, 0, 0, 0, 0, 0), 20)
        || in_v6(value, Ipv6Addr::new(0x5f00, 0, 0, 0, 0, 0, 0, 0), 16)
        || in_v6(value, Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 0), 10)
        || in_v6(value, Ipv6Addr::new(0xff00, 0, 0, 0, 0, 0, 0, 0), 8)
    {
        AddressClass::Prohibited
    } else {
        AddressClass::Public
    }
}

fn in_v6(value: u128, network: Ipv6Addr, prefix: u32) -> bool {
    let mask = if prefix == 0 {
        0
    } else {
        u128::MAX << (128 - prefix)
    };
    value & mask == u128::from(network) & mask
}

#[derive(Debug)]
pub(crate) struct HttpRun {
    responses: Vec<ProbeResponse>,
    failure: Option<HttpFailure>,
    tls_applicable: bool,
    session_cleanup_failed: bool,
}

#[derive(Debug, Default)]
struct ResponseBudget {
    output_bytes: u64,
    message_count: u64,
}

impl ResponseBudget {
    fn observe_output(&mut self, bytes: usize, limits: HttpLimits) -> Result<(), HttpFailure> {
        let observed = self
            .output_bytes
            .saturating_add(u64::try_from(bytes).unwrap_or(u64::MAX));
        if observed > limits.aggregate_output_bytes {
            return Err(HttpFailure::limit(
                HttpLimit::AggregateOutputBytes,
                observed,
                limits.aggregate_output_bytes,
            ));
        }
        self.output_bytes = observed;
        Ok(())
    }

    fn observe_message(&mut self, limits: HttpLimits) -> Result<(), HttpFailure> {
        let observed = self.message_count.saturating_add(1);
        if observed > limits.message_count {
            return Err(HttpFailure::limit(
                HttpLimit::MessageCount,
                observed,
                limits.message_count,
            ));
        }
        self.message_count = observed;
        Ok(())
    }
}

impl HttpRun {
    pub(crate) fn responses(&self) -> &[ProbeResponse] {
        &self.responses
    }

    pub(crate) const fn failure(&self) -> Option<HttpFailure> {
        self.failure
    }

    pub(crate) const fn tls_applicable(&self) -> bool {
        self.tls_applicable
    }

    pub(crate) const fn session_cleanup_failed(&self) -> bool {
        self.session_cleanup_failed
    }
}

#[derive(Clone, Eq, PartialEq)]
struct SessionId(HeaderValue);

impl fmt::Debug for SessionId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SessionId([REDACTED])")
    }
}

pub(crate) struct HttpTransport {
    target: HttpTarget,
    connector: Box<dyn Connector>,
    protocol_revision: &'static str,
    initialize_handshake: bool,
    accept_server_requests: bool,
    session: Option<SessionId>,
}

impl fmt::Debug for HttpTransport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HttpTransport")
            .field("target", &self.target)
            .field("connector", &"[REDACTED]")
            .field("protocol_revision", &self.protocol_revision)
            .field("initialize_handshake", &self.initialize_handshake)
            .field("accept_server_requests", &self.accept_server_requests)
            .field("session_present", &self.session.is_some())
            .finish()
    }
}

impl HttpTransport {
    pub(crate) fn new_for_protocol(
        target: HttpTarget,
        protocol_revision: &'static str,
        initialize_handshake: bool,
    ) -> Result<Self, HttpFailure> {
        Self::build(target, protocol_revision, initialize_handshake, false)
    }

    pub(crate) fn new_for_active_protocol(
        target: HttpTarget,
        protocol_revision: &'static str,
        initialize_handshake: bool,
    ) -> Result<Self, HttpFailure> {
        Self::build(
            target,
            protocol_revision,
            initialize_handshake,
            initialize_handshake,
        )
    }

    fn build(
        target: HttpTarget,
        protocol_revision: &'static str,
        initialize_handshake: bool,
        accept_server_requests: bool,
    ) -> Result<Self, HttpFailure> {
        install_ring_provider();
        let selected = connection_candidates(&target.addresses);
        let connect_timeout = remaining_connection_time(target.started, target.limits)?;
        let mut builder = Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .retry(reqwest::retry::never())
            .no_proxy()
            .referer(false)
            .no_gzip()
            .no_brotli()
            .no_deflate()
            .no_zstd()
            .http1_only()
            .connect_timeout(connect_timeout)
            .read_timeout(Duration::from_millis(target.limits.response_ms))
            .pool_max_idle_per_host(1)
            .tls_version_min(reqwest::tls::Version::TLS_1_2)
            .tls_version_max(reqwest::tls::Version::TLS_1_3)
            .resolve_to_addrs(&target.endpoint.host, &selected);
        if !target.trust.is_empty() {
            builder = builder.tls_certs_merge(target.trust.iter().cloned());
        }
        let client = builder.build().map_err(|_| HttpFailure::Tls)?;
        Ok(Self {
            target,
            connector: Box::new(ReqwestConnector { client }),
            protocol_revision,
            initialize_handshake,
            accept_server_requests,
            session: None,
        })
    }

    #[cfg(test)]
    fn with_connector(target: HttpTarget, connector: Box<dyn Connector>) -> Self {
        Self {
            target,
            connector,
            protocol_revision: PROTOCOL_REVISION,
            initialize_handshake: false,
            accept_server_requests: false,
            session: None,
        }
    }

    pub(crate) async fn probe<C: Conversation>(mut self, conversation: &mut C) -> HttpRun {
        let total_deadline =
            self.target.started + Duration::from_millis(self.target.limits.total_ms);
        let mut responses = Vec::new();
        let mut response_budget = ResponseBudget::default();
        let failure = loop {
            if Instant::now() > total_deadline {
                break Some(HttpFailure::timeout(
                    HttpLimit::TotalTime,
                    self.target.limits.total_ms,
                ));
            }
            let request = conversation.next_request(responses.last());
            let Some(request) = request else {
                break None;
            };
            let stage_kind = if responses.is_empty() {
                HttpLimit::DiscoveryTime
            } else {
                HttpLimit::ResponseTime
            };
            let stage_ms = if responses.is_empty() {
                self.target.limits.discovery_ms
            } else {
                self.target.limits.response_ms
            };
            let stage_deadline = Instant::now() + Duration::from_millis(stage_ms);
            let (deadline, timeout_kind, timeout_maximum) = if total_deadline <= stage_deadline {
                (
                    total_deadline,
                    HttpLimit::TotalTime,
                    self.target.limits.total_ms,
                )
            } else {
                (stage_deadline, stage_kind, stage_ms)
            };
            match tokio::time::timeout_at(deadline, self.exchange(&request, &mut response_budget))
                .await
            {
                Ok(Ok(Some(response))) => responses.push(response),
                Ok(Ok(None)) => {}
                Ok(Err(failure)) => break Some(failure),
                Err(_) => break Some(HttpFailure::timeout(timeout_kind, timeout_maximum)),
            }
        };
        let session_cleanup_failed = self
            .teardown_session(total_deadline, &mut response_budget)
            .await
            .is_err();
        HttpRun {
            responses: if failure.is_some() {
                Vec::new()
            } else {
                responses
            },
            failure,
            tls_applicable: self.target.endpoint.https,
            session_cleanup_failed,
        }
    }

    async fn exchange(
        &mut self,
        request: &ProbeRequest,
        response_budget: &mut ResponseBudget,
    ) -> Result<Option<ProbeResponse>, HttpFailure> {
        let request_bytes = u64::try_from(request.as_bytes().len()).unwrap_or(u64::MAX);
        if request_bytes > self.target.limits.message_bytes {
            return Err(HttpFailure::limit(
                HttpLimit::MessageBytes,
                request_bytes,
                self.target.limits.message_bytes,
            ));
        }
        let headers = self.request_headers(request)?;
        let exchange = self.connector.post(ConnectorRequest {
            endpoint: self.target.endpoint.url.clone(),
            accepted_peers: self.target.addresses.clone(),
            headers,
            body: request.as_bytes().to_vec(),
            response_ms: self.target.limits.response_ms,
        });
        let response = tokio::time::timeout(
            Duration::from_millis(self.target.limits.request_ms),
            exchange,
        )
        .await
        .map_err(|_| {
            HttpFailure::timeout(HttpLimit::RequestTime, self.target.limits.request_ms)
        })??;
        let peer = response.peer.ok_or(HttpFailure::PeerMismatch)?;
        if peer.port() != self.target.endpoint.port
            || !self
                .target
                .addresses
                .iter()
                .any(|address| address.ip() == peer.ip())
        {
            return Err(HttpFailure::PeerMismatch);
        }
        self.response(request, response, response_budget).await
    }

    fn request_headers(&self, request: &ProbeRequest) -> Result<HeaderMap, HttpFailure> {
        let mut fields = vec![
            (CONTENT_TYPE, HeaderValue::from_static(JSON_MEDIA_TYPE)),
            (ACCEPT, HeaderValue::from_static(ACCEPT_VALUE)),
            (ACCEPT_ENCODING, HeaderValue::from_static("identity")),
            (USER_AGENT, HeaderValue::from_static(USER_AGENT_VALUE)),
        ];
        let initialize = self.initialize_handshake && request.method() == "initialize";
        if !initialize {
            fields.push((
                HeaderName::from_static("mcp-protocol-version"),
                HeaderValue::from_static(self.protocol_revision),
            ));
            if let Some(session) = &self.session {
                fields.push((HeaderName::from_static("mcp-session-id"), session.0.clone()));
            }
        }
        if !self.initialize_handshake {
            let method = HeaderValue::from_str(request.method())
                .map_err(|_| HttpFailure::Response(ResponseFailure::InvalidMessage))?;
            fields.push((HeaderName::from_static("mcp-method"), method));
            if let Some(name) = request.principal_name() {
                fields.push((
                    HeaderName::from_static("mcp-name"),
                    HeaderValue::from_str(&encode_mcp_value(name))
                        .map_err(|_| HttpFailure::Response(ResponseFailure::InvalidMessage))?,
                ));
            }
            for field in request.mirrored_fields() {
                let name = format!("mcp-param-{}", field.suffix());
                let name = HeaderName::from_bytes(name.as_bytes())
                    .map_err(|_| HttpFailure::Response(ResponseFailure::InvalidMessage))?;
                let value = HeaderValue::from_str(&encode_mcp_value(field.value()))
                    .map_err(|_| HttpFailure::Response(ResponseFailure::InvalidMessage))?;
                fields.push((name, value));
            }
        }
        fields.extend(self.target.credentials.fields.iter().cloned());
        validate_request_field_budget(
            &fields,
            &self.target.endpoint,
            request.as_bytes().len(),
            self.target.limits,
        )?;
        let mut headers = HeaderMap::with_capacity(fields.len());
        for (name, value) in fields {
            headers.insert(name, value);
        }
        Ok(headers)
    }

    async fn response(
        &mut self,
        request: &ProbeRequest,
        mut response: ConnectorResponse,
        response_budget: &mut ResponseBudget,
    ) -> Result<Option<ProbeResponse>, HttpFailure> {
        validate_response_field_budget(&response.headers, self.target.limits)?;
        self.observe_session(&response.headers, request.method() == "initialize")?;
        let status = response.status;
        if status.is_redirection() {
            return Err(HttpFailure::Response(ResponseFailure::Redirect {
                status: status.as_u16(),
            }));
        }
        if matches!(status, StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN) {
            return Err(HttpFailure::Response(ResponseFailure::Authentication {
                status: status.as_u16(),
            }));
        }
        if !request.expects_response() {
            if status != StatusCode::ACCEPTED {
                return Err(HttpFailure::Response(if self.initialize_handshake {
                    ResponseFailure::InitializedRejected {
                        status: status.as_u16(),
                    }
                } else {
                    ResponseFailure::Status {
                        status: status.as_u16(),
                    }
                }));
            }
            if has_non_identity_encoding(&response.headers) {
                return Err(HttpFailure::Response(ResponseFailure::ContentEncoding));
            }
            let body = collect_body(
                &mut response,
                self.target.limits.message_bytes,
                self.target.limits,
                response_budget,
            )
            .await?;
            if !body.is_empty() {
                return Err(HttpFailure::Response(ResponseFailure::InvalidMessage));
            }
            return Ok(None);
        }
        if self.initialize_handshake && status == StatusCode::NOT_FOUND {
            return Err(HttpFailure::Response(if self.session.is_some() {
                ResponseFailure::SessionLost {
                    status: status.as_u16(),
                }
            } else {
                ResponseFailure::SessionRequired {
                    status: status.as_u16(),
                }
            }));
        }
        if status != StatusCode::OK {
            if status != StatusCode::BAD_REQUEST || has_non_identity_encoding(&response.headers) {
                return Err(HttpFailure::Response(ResponseFailure::Status {
                    status: status.as_u16(),
                }));
            }
            let json_response = matches!(
                response_media_type(&response.headers),
                Ok(ResponseMediaType::Json)
            );
            let body = collect_body(
                &mut response,
                self.target.limits.message_bytes,
                self.target.limits,
                response_budget,
            )
            .await?;
            response_budget.observe_message(self.target.limits)?;
            if !self.initialize_handshake
                && status == StatusCode::BAD_REQUEST
                && json_response
                && is_unsupported_protocol_version(
                    &body,
                    request.id(),
                    self.protocol_revision,
                    self.target.limits.protocol_revisions,
                )
            {
                return Err(HttpFailure::Response(
                    ResponseFailure::UnsupportedProtocolVersion,
                ));
            }
            if status == StatusCode::BAD_REQUEST && is_header_mismatch(&body, request.id()) {
                return Err(HttpFailure::Response(if self.initialize_handshake {
                    ResponseFailure::ProtocolVersionRejected
                } else {
                    ResponseFailure::HeaderMismatch
                }));
            }
            if self.initialize_handshake && status == StatusCode::BAD_REQUEST {
                return Err(HttpFailure::Response(if self.session.is_some() {
                    ResponseFailure::SessionLost {
                        status: status.as_u16(),
                    }
                } else {
                    ResponseFailure::SessionRequired {
                        status: status.as_u16(),
                    }
                }));
            }
            return Err(HttpFailure::Response(ResponseFailure::Status {
                status: status.as_u16(),
            }));
        }
        if has_non_identity_encoding(&response.headers) {
            return Err(HttpFailure::Response(ResponseFailure::ContentEncoding));
        }

        let media_type = response_media_type(&response.headers)?;
        match media_type {
            ResponseMediaType::Json => {
                let bytes = collect_body(
                    &mut response,
                    self.target.limits.message_bytes,
                    self.target.limits,
                    response_budget,
                )
                .await?;
                response_budget.observe_message(self.target.limits)?;
                validate_json_response(
                    &bytes,
                    request.id(),
                    self.accept_server_requests && request.method() == "tools/call",
                )?;
                Ok(Some(ProbeResponse::new(request.id(), bytes)))
            }
            ResponseMediaType::Sse => {
                let response = collect_sse_response(
                    &mut response,
                    request.id(),
                    self.protocol_revision == "2025-11-25",
                    self.accept_server_requests && request.method() == "tools/call",
                    self.target.limits,
                    response_budget,
                )
                .await?;
                Ok(Some(ProbeResponse::new(request.id(), response)))
            }
        }
    }

    fn observe_session(
        &mut self,
        headers: &HeaderMap,
        initialize: bool,
    ) -> Result<(), HttpFailure> {
        if !self.initialize_handshake {
            return Ok(());
        }
        let observed = session_header(headers)?;
        if initialize {
            self.session = observed;
            return Ok(());
        }
        if let Some(observed) = observed
            && self.session.as_ref() != Some(&observed)
        {
            return Err(HttpFailure::Response(ResponseFailure::SessionChanged));
        }
        Ok(())
    }

    async fn teardown_session(
        &self,
        total_deadline: Instant,
        response_budget: &mut ResponseBudget,
    ) -> Result<(), HttpFailure> {
        let Some(session) = &self.session else {
            return Ok(());
        };
        let mut fields = vec![
            (ACCEPT, HeaderValue::from_static(ACCEPT_VALUE)),
            (ACCEPT_ENCODING, HeaderValue::from_static("identity")),
            (USER_AGENT, HeaderValue::from_static(USER_AGENT_VALUE)),
            (
                HeaderName::from_static("mcp-protocol-version"),
                HeaderValue::from_static(self.protocol_revision),
            ),
            (HeaderName::from_static("mcp-session-id"), session.0.clone()),
        ];
        fields.extend(self.target.credentials.fields.iter().cloned());
        validate_request_field_budget(&fields, &self.target.endpoint, 0, self.target.limits)?;
        let mut headers = HeaderMap::with_capacity(fields.len());
        for (name, value) in fields {
            headers.insert(name, value);
        }
        let cleanup_deadline = std::cmp::min(
            total_deadline,
            Instant::now() + Duration::from_millis(self.target.limits.shutdown_grace_ms),
        );
        let operation = async {
            let mut response = self
                .connector
                .delete(ConnectorRequest {
                    endpoint: self.target.endpoint.url.clone(),
                    accepted_peers: self.target.addresses.clone(),
                    headers,
                    body: Vec::new(),
                    response_ms: self.target.limits.shutdown_grace_ms,
                })
                .await?;
            validate_peer(&self.target, response.peer)?;
            validate_response_field_budget(&response.headers, self.target.limits)?;
            if let Some(observed) = session_header(&response.headers)?
                && observed != *session
            {
                return Err(HttpFailure::Response(ResponseFailure::SessionChanged));
            }
            if !response.status.is_success()
                && !matches!(
                    response.status,
                    StatusCode::METHOD_NOT_ALLOWED | StatusCode::NOT_FOUND
                )
            {
                return Err(HttpFailure::Response(ResponseFailure::Status {
                    status: response.status.as_u16(),
                }));
            }
            if has_non_identity_encoding(&response.headers) {
                return Err(HttpFailure::Response(ResponseFailure::ContentEncoding));
            }
            let _ = collect_body(
                &mut response,
                self.target.limits.message_bytes,
                self.target.limits,
                response_budget,
            )
            .await?;
            Ok(())
        };
        tokio::time::timeout_at(cleanup_deadline, operation)
            .await
            .map_err(|_| {
                HttpFailure::timeout(
                    HttpLimit::ResponseTime,
                    self.target.limits.shutdown_grace_ms,
                )
            })?
    }
}

fn session_header(headers: &HeaderMap) -> Result<Option<SessionId>, HttpFailure> {
    let mut values = headers
        .get_all(HeaderName::from_static("mcp-session-id"))
        .iter();
    let Some(value) = values.next() else {
        return Ok(None);
    };
    if values.next().is_some()
        || value.as_bytes().is_empty()
        || !value
            .as_bytes()
            .iter()
            .all(|byte| (0x21..=0x7e).contains(byte))
    {
        return Err(HttpFailure::Response(ResponseFailure::InvalidSession));
    }
    Ok(Some(SessionId(value.clone())))
}

fn validate_peer(target: &HttpTarget, peer: Option<SocketAddr>) -> Result<(), HttpFailure> {
    let peer = peer.ok_or(HttpFailure::PeerMismatch)?;
    if peer.port() != target.endpoint.port
        || !target
            .addresses
            .iter()
            .any(|address| address.ip() == peer.ip())
    {
        return Err(HttpFailure::PeerMismatch);
    }
    Ok(())
}

fn has_non_identity_encoding(headers: &HeaderMap) -> bool {
    headers
        .get_all(CONTENT_ENCODING)
        .iter()
        .any(|value| !value.as_bytes().eq_ignore_ascii_case(b"identity"))
}

fn remaining_connection_time(
    started: Instant,
    limits: HttpLimits,
) -> Result<Duration, HttpFailure> {
    let startup_deadline = started + Duration::from_millis(limits.startup_ms);
    let total_deadline = started + Duration::from_millis(limits.total_ms);
    let (deadline, kind, maximum) = if total_deadline <= startup_deadline {
        (total_deadline, HttpLimit::TotalTime, limits.total_ms)
    } else {
        (startup_deadline, HttpLimit::StartupTime, limits.startup_ms)
    };
    deadline
        .checked_duration_since(Instant::now())
        .filter(|remaining| !remaining.is_zero())
        .ok_or_else(|| HttpFailure::timeout(kind, maximum))
}

fn connection_candidates(addresses: &[SocketAddr]) -> Vec<SocketAddr> {
    let selected_family_is_ipv4 = addresses
        .first()
        .expect("validated resolution retains an address")
        .is_ipv4();
    addresses
        .iter()
        .copied()
        .filter(|address| address.is_ipv4() == selected_family_is_ipv4)
        .collect()
}

fn install_ring_provider() {
    static PROVIDER: OnceLock<()> = OnceLock::new();
    PROVIDER.get_or_init(|| {
        if rustls::crypto::CryptoProvider::get_default().is_none() {
            let _ = rustls::crypto::ring::default_provider().install_default();
        }
    });
}

fn classify_reqwest_error(error: reqwest::Error) -> HttpFailure {
    let mut source = std::error::Error::source(&error);
    while let Some(error) = source {
        if error.downcast_ref::<rustls::Error>().is_some() {
            return HttpFailure::Tls;
        }
        source = error.source();
    }
    HttpFailure::Request
}

fn classify_body_error(error: reqwest::Error) -> HttpFailure {
    match classify_reqwest_error(error) {
        HttpFailure::Tls => HttpFailure::Tls,
        _ => HttpFailure::ResponseIo,
    }
}

fn classify_send_error(error: reqwest::Error, https: bool) -> HttpFailure {
    if https {
        let mut source = std::error::Error::source(&error);
        while let Some(error) = source {
            if error.downcast_ref::<rustls::Error>().is_some()
                || error
                    .downcast_ref::<std::io::Error>()
                    .is_some_and(io_error_contains_tls_failure)
            {
                return HttpFailure::Tls;
            }
            source = error.source();
        }
    }
    classify_reqwest_error(error)
}

fn io_error_contains_tls_failure(error: &std::io::Error) -> bool {
    if error.kind() == std::io::ErrorKind::InvalidData {
        return true;
    }
    let Some(inner) = error.get_ref() else {
        return false;
    };
    inner.downcast_ref::<rustls::Error>().is_some()
        || inner
            .downcast_ref::<std::io::Error>()
            .is_some_and(io_error_contains_tls_failure)
}

fn encode_mcp_value(value: &str) -> String {
    let safe = !value.is_empty()
        && value
            .bytes()
            .all(|byte| matches!(byte, b'\t' | b' '..=b'~'))
        && value.trim_matches([' ', '\t']) == value
        && !(value.starts_with(BASE64_PREFIX) && value.ends_with(BASE64_SUFFIX));
    if safe {
        value.to_owned()
    } else {
        format!(
            "{BASE64_PREFIX}{}{BASE64_SUFFIX}",
            BASE64_STANDARD.encode(value.as_bytes())
        )
    }
}

fn validate_request_field_budget(
    fields: &[(HeaderName, HeaderValue)],
    endpoint: &CanonicalEndpoint,
    body_bytes: usize,
    limits: HttpLimits,
) -> Result<(), HttpFailure> {
    let count = u64::try_from(fields.len().saturating_add(2)).unwrap_or(u64::MAX);
    if count > limits.request_fields {
        return Err(HttpFailure::limit(
            HttpLimit::RequestFields,
            count,
            limits.request_fields,
        ));
    }
    let mut aggregate = 0_u64;
    for (name, value) in fields {
        observe_field(
            name.as_str().len(),
            value.as_bytes().len(),
            true,
            &mut aggregate,
            limits,
        )?;
    }
    let host_value_bytes = endpoint
        .host
        .len()
        .saturating_add(if endpoint.explicit_port {
            endpoint.port.to_string().len().saturating_add(1)
        } else {
            0
        });
    observe_field(4, host_value_bytes, true, &mut aggregate, limits)?;
    observe_field(
        "content-length".len(),
        body_bytes.to_string().len(),
        true,
        &mut aggregate,
        limits,
    )?;
    if aggregate > limits.request_fields_bytes {
        return Err(HttpFailure::limit(
            HttpLimit::RequestFieldsBytes,
            aggregate,
            limits.request_fields_bytes,
        ));
    }
    Ok(())
}

fn validate_response_field_budget(
    headers: &HeaderMap,
    limits: HttpLimits,
) -> Result<(), HttpFailure> {
    let count = u64::try_from(headers.len()).unwrap_or(u64::MAX);
    if count > limits.response_fields {
        return Err(HttpFailure::limit(
            HttpLimit::ResponseFields,
            count,
            limits.response_fields,
        ));
    }
    let mut aggregate = 0_u64;
    for (name, value) in headers {
        observe_field(
            name.as_str().len(),
            value.as_bytes().len(),
            false,
            &mut aggregate,
            limits,
        )?;
    }
    if aggregate > limits.response_fields_bytes {
        return Err(HttpFailure::limit(
            HttpLimit::ResponseFieldsBytes,
            aggregate,
            limits.response_fields_bytes,
        ));
    }
    Ok(())
}

fn observe_field(
    name_bytes: usize,
    value_bytes: usize,
    request: bool,
    aggregate: &mut u64,
    limits: HttpLimits,
) -> Result<(), HttpFailure> {
    let name_bytes = u64::try_from(name_bytes).unwrap_or(u64::MAX);
    let value_bytes = u64::try_from(value_bytes).unwrap_or(u64::MAX);
    let (name_maximum, value_maximum, name_kind, value_kind) = if request {
        (
            limits.request_field_name_bytes,
            limits.request_field_value_bytes,
            HttpLimit::RequestFieldNameBytes,
            HttpLimit::RequestFieldValueBytes,
        )
    } else {
        (
            limits.response_field_name_bytes,
            limits.response_field_value_bytes,
            HttpLimit::ResponseFieldNameBytes,
            HttpLimit::ResponseFieldValueBytes,
        )
    };
    if name_bytes > name_maximum {
        return Err(HttpFailure::limit(name_kind, name_bytes, name_maximum));
    }
    if value_bytes > value_maximum {
        return Err(HttpFailure::limit(value_kind, value_bytes, value_maximum));
    }
    *aggregate = aggregate
        .saturating_add(name_bytes)
        .saturating_add(value_bytes)
        .saturating_add(4);
    Ok(())
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum ResponseMediaType {
    Json,
    Sse,
}

fn response_media_type(headers: &HeaderMap) -> Result<ResponseMediaType, HttpFailure> {
    let mut values = headers.get_all(CONTENT_TYPE).iter();
    let value = values
        .next()
        .filter(|_| values.next().is_none())
        .and_then(|value| value.to_str().ok())
        .ok_or(HttpFailure::Response(ResponseFailure::MediaType))?;
    let media = value.split(';').next().unwrap_or_default().trim();
    if media.eq_ignore_ascii_case(JSON_MEDIA_TYPE) {
        Ok(ResponseMediaType::Json)
    } else if media.eq_ignore_ascii_case(SSE_MEDIA_TYPE) {
        Ok(ResponseMediaType::Sse)
    } else {
        Err(HttpFailure::Response(ResponseFailure::MediaType))
    }
}

async fn collect_body(
    response: &mut ConnectorResponse,
    maximum: u64,
    limits: HttpLimits,
    response_budget: &mut ResponseBudget,
) -> Result<Vec<u8>, HttpFailure> {
    let mut body = Vec::new();
    while let Some(chunk) = response.body.next_chunk().await? {
        let observed = u64::try_from(body.len().saturating_add(chunk.len())).unwrap_or(u64::MAX);
        if observed > maximum {
            return Err(HttpFailure::limit(
                HttpLimit::MessageBytes,
                observed,
                maximum,
            ));
        }
        response_budget.observe_output(chunk.len(), limits)?;
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

async fn collect_sse_response(
    response: &mut ConnectorResponse,
    request_id: i64,
    allow_empty_priming_event: bool,
    accept_server_requests: bool,
    limits: HttpLimits,
    response_budget: &mut ResponseBudget,
) -> Result<Vec<u8>, HttpFailure> {
    let mut body = Vec::new();
    loop {
        match response.body.next_chunk().await? {
            Some(chunk) => {
                response_budget.observe_output(chunk.len(), limits)?;
                body.extend_from_slice(&chunk);
                if let Some((response, message_count)) = parse_sse_events(
                    &body,
                    request_id,
                    allow_empty_priming_event,
                    accept_server_requests,
                    limits,
                    response_budget.message_count,
                    false,
                )? {
                    for _ in 0..message_count {
                        response_budget.observe_message(limits)?;
                    }
                    return Ok(response);
                }
            }
            None => {
                let Some((response, message_count)) = parse_sse_events(
                    &body,
                    request_id,
                    allow_empty_priming_event,
                    accept_server_requests,
                    limits,
                    response_budget.message_count,
                    true,
                )?
                else {
                    return Err(HttpFailure::Response(ResponseFailure::InvalidSse));
                };
                for _ in 0..message_count {
                    response_budget.observe_message(limits)?;
                }
                return Ok(response);
            }
        }
    }
}

fn validate_json_response(
    bytes: &[u8],
    request_id: i64,
    accept_server_requests: bool,
) -> Result<(), HttpFailure> {
    let value: Value = serde_json::from_slice(bytes)
        .map_err(|_| HttpFailure::Response(ResponseFailure::InvalidMessage))?;
    let object = value
        .as_object()
        .ok_or(HttpFailure::Response(ResponseFailure::InvalidMessage))?;
    let valid_response = object.get("jsonrpc").and_then(Value::as_str) == Some("2.0")
        && object.get("id").and_then(Value::as_i64) == Some(request_id)
        && (object.contains_key("result") ^ object.contains_key("error"))
        && !object.contains_key("method");
    if valid_response || (accept_server_requests && valid_server_request(object)) {
        Ok(())
    } else {
        Err(HttpFailure::Response(ResponseFailure::InvalidMessage))
    }
}

fn valid_server_request(object: &serde_json::Map<String, Value>) -> bool {
    object.get("jsonrpc").and_then(Value::as_str) == Some("2.0")
        && object
            .get("id")
            .is_some_and(|id| id.is_string() || id.as_i64().is_some())
        && object.get("method").is_some_and(Value::is_string)
        && object
            .get("params")
            .is_none_or(|params| params.is_object() || params.is_array())
        && !object.contains_key("result")
        && !object.contains_key("error")
}

fn is_header_mismatch(bytes: &[u8], request_id: i64) -> bool {
    serde_json::from_slice::<Value>(bytes)
        .ok()
        .is_some_and(|value| {
            value.get("jsonrpc").and_then(Value::as_str) == Some("2.0")
                && value.get("id").and_then(Value::as_i64) == Some(request_id)
                && value
                    .get("error")
                    .and_then(|error| error.get("code"))
                    .and_then(Value::as_i64)
                    == Some(-32020)
        })
}

fn is_unsupported_protocol_version(
    bytes: &[u8],
    request_id: i64,
    requested_revision: &str,
    maximum_revisions: u64,
) -> bool {
    serde_json::from_slice::<Value>(bytes)
        .ok()
        .and_then(|value| {
            let object = value.as_object()?;
            let error = object.get("error")?.as_object()?;
            let data = error.get("data")?.as_object()?;
            let supported = data.get("supported")?.as_array()?;
            let requested = data.get("requested")?.as_str()?;
            Some(
                object.get("jsonrpc").and_then(Value::as_str) == Some("2.0")
                    && object.get("id").and_then(Value::as_i64) == Some(request_id)
                    && !object.contains_key("result")
                    && !object.contains_key("method")
                    && error.get("code").and_then(Value::as_i64) == Some(-32022)
                    && error.get("message").is_some_and(Value::is_string)
                    && requested == requested_revision
                    && u64::try_from(supported.len()).unwrap_or(u64::MAX) <= maximum_revisions
                    && supported.iter().all(Value::is_string)
                    && !supported
                        .iter()
                        .any(|revision| revision.as_str() == Some(requested_revision)),
            )
        })
        .unwrap_or(false)
}

fn parse_sse_events(
    bytes: &[u8],
    request_id: i64,
    allow_empty_priming_event: bool,
    accept_server_requests: bool,
    limits: HttpLimits,
    prior_messages: u64,
    eof: bool,
) -> Result<Option<(Vec<u8>, u64)>, HttpFailure> {
    let text = match std::str::from_utf8(bytes) {
        Ok(text) => text,
        Err(error) if !eof && error.error_len().is_none() => return Ok(None),
        Err(_) => return Err(HttpFailure::Response(ResponseFailure::InvalidSse)),
    };
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    let mut start = 0;
    let mut message_count = 0_u64;
    loop {
        let remaining = &normalized[start..];
        let boundary = remaining.find("\n\n");
        let event = match boundary {
            Some(index) => &remaining[..index],
            None if eof && !remaining.is_empty() => remaining,
            None => {
                let observed = u64::try_from(remaining.len()).unwrap_or(u64::MAX);
                if observed > limits.message_bytes {
                    return Err(HttpFailure::limit(
                        HttpLimit::MessageBytes,
                        observed,
                        limits.message_bytes,
                    ));
                }
                return Ok(None);
            }
        };
        let event_bytes = u64::try_from(event.len()).unwrap_or(u64::MAX);
        if event_bytes > limits.message_bytes {
            return Err(HttpFailure::limit(
                HttpLimit::MessageBytes,
                event_bytes,
                limits.message_bytes,
            ));
        }
        let data = event
            .split('\n')
            .filter(|line| !line.starts_with(':'))
            .filter_map(|line| {
                let (field, value) = line.split_once(':').unwrap_or((line, ""));
                (field == "data").then(|| value.strip_prefix(' ').unwrap_or(value))
            })
            .collect::<Vec<_>>();
        if !data.is_empty() {
            message_count = message_count.saturating_add(1);
            let observed_messages = prior_messages.saturating_add(message_count);
            if observed_messages > limits.message_count {
                return Err(HttpFailure::limit(
                    HttpLimit::MessageCount,
                    observed_messages,
                    limits.message_count,
                ));
            }
            let priming_event = allow_empty_priming_event
                && data.iter().all(|value| value.is_empty())
                && event.lines().any(|line| {
                    line.split_once(':').is_some_and(|(field, value)| {
                        field == "id" && !value.strip_prefix(' ').unwrap_or(value).is_empty()
                    })
                });
            if priming_event {
                match boundary {
                    Some(index) => start = start.saturating_add(index).saturating_add(2),
                    None => return Ok(None),
                }
                continue;
            }
            let payload = data.join("\n");
            let value: Value = serde_json::from_str(&payload)
                .map_err(|_| HttpFailure::Response(ResponseFailure::InvalidSse))?;
            let object = value
                .as_object()
                .ok_or(HttpFailure::Response(ResponseFailure::InvalidSse))?;
            if object.get("method").is_some() {
                if object.contains_key("result")
                    || object.contains_key("error")
                    || object.get("jsonrpc").and_then(Value::as_str) != Some("2.0")
                {
                    return Err(HttpFailure::Response(ResponseFailure::InvalidSse));
                }
                if object.contains_key("id") {
                    if !accept_server_requests || !valid_server_request(object) {
                        return Err(HttpFailure::Response(ResponseFailure::InvalidSse));
                    }
                    return Ok(Some((payload.into_bytes(), message_count)));
                }
            } else {
                validate_json_response(payload.as_bytes(), request_id, false)?;
                return Ok(Some((payload.into_bytes(), message_count)));
            }
        }
        match boundary {
            Some(index) => start = start.saturating_add(index).saturating_add(2),
            None => return Ok(None),
        }
    }
}

pub(crate) fn mirrored_primitive(value: &Value) -> Result<Option<String>, ()> {
    match value {
        Value::Null => Ok(None),
        Value::String(value) => Ok(Some(value.clone())),
        Value::Bool(value) => Ok(Some(value.to_string())),
        Value::Number(value) => {
            let integer = value.as_i64().ok_or(())?;
            if !(-MAX_SAFE_INTEGER..=MAX_SAFE_INTEGER).contains(&integer) {
                return Err(());
            }
            Ok(Some(integer.to_string()))
        }
        Value::Array(_) | Value::Object(_) => Err(()),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::fs;
    use std::future::{Future, pending};
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
    use std::pin::Pin;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    use reqwest::StatusCode;
    use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue, LOCATION};

    use crate::transport::{Conversation, MirroredField, ProbeRequest, ProbeResponse};

    use super::{
        AddressClass, BodyFuture, Connector, ConnectorRequest, ConnectorResponse, HttpFailure,
        HttpLimit, HttpLimits, HttpTarget, HttpTransport, RemoteOptions, ResolutionFailure,
        Resolver, ResponseBody, ResponseFailure, TargetFailure, classify_address,
        collect_resolver_addresses, connection_candidates, encode_mcp_value,
        is_unsupported_protocol_version, parse_endpoint, read_trust_file,
        remaining_connection_time, reserved_field, valid_bearer_token, valid_custom_value,
        valid_token, validate_environment_name, validate_request_field_budget,
        validate_response_field_budget,
    };

    #[derive(Clone)]
    struct FixedResolver {
        answers: Result<Vec<SocketAddr>, ()>,
        calls: Arc<AtomicUsize>,
    }

    impl FixedResolver {
        fn new(answers: Vec<SocketAddr>) -> Self {
            Self {
                answers: Ok(answers),
                calls: Arc::new(AtomicUsize::new(0)),
            }
        }

        fn failed() -> Self {
            Self {
                answers: Err(()),
                calls: Arc::new(AtomicUsize::new(0)),
            }
        }

        fn call_count(&self) -> usize {
            self.calls.load(Ordering::SeqCst)
        }
    }

    impl Resolver for FixedResolver {
        fn resolve<'a>(
            &'a self,
            _host: &'a str,
            _port: u16,
            _maximum: u64,
        ) -> Pin<Box<dyn Future<Output = Result<Vec<SocketAddr>, ()>> + Send + 'a>> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            let answers = self.answers.clone();
            Box::pin(async move { answers })
        }
    }

    struct PendingResolver;

    impl Resolver for PendingResolver {
        fn resolve<'a>(
            &'a self,
            _host: &'a str,
            _port: u16,
            _maximum: u64,
        ) -> Pin<Box<dyn Future<Output = Result<Vec<SocketAddr>, ()>> + Send + 'a>> {
            Box::pin(pending())
        }
    }

    struct FixtureBody {
        chunks: VecDeque<Vec<u8>>,
        pending_after_chunks: bool,
    }

    impl ResponseBody for FixtureBody {
        fn next_chunk<'a>(&'a mut self) -> BodyFuture<'a> {
            let chunk = self.chunks.pop_front();
            let pending_after_chunks = self.pending_after_chunks;
            Box::pin(async move {
                if chunk.is_none() && pending_after_chunks {
                    pending().await
                } else {
                    Ok(chunk)
                }
            })
        }
    }

    enum ConnectorPlan {
        Reply {
            status: StatusCode,
            headers: HeaderMap,
            peer: Option<SocketAddr>,
            chunks: VecDeque<Vec<u8>>,
            pending_body: bool,
        },
        Pending,
    }

    struct FixtureConnector {
        plans: Mutex<VecDeque<ConnectorPlan>>,
        calls: Arc<AtomicUsize>,
        requests: Arc<Mutex<Vec<ConnectorRequest>>>,
    }

    impl FixtureConnector {
        fn new(plans: impl IntoIterator<Item = ConnectorPlan>) -> Self {
            Self {
                plans: Mutex::new(plans.into_iter().collect()),
                calls: Arc::new(AtomicUsize::new(0)),
                requests: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn call_count(&self) -> Arc<AtomicUsize> {
            Arc::clone(&self.calls)
        }

        fn requests(&self) -> Arc<Mutex<Vec<ConnectorRequest>>> {
            Arc::clone(&self.requests)
        }
    }

    impl Connector for FixtureConnector {
        fn post<'a>(
            &'a self,
            request: ConnectorRequest,
        ) -> Pin<Box<dyn Future<Output = Result<ConnectorResponse, HttpFailure>> + Send + 'a>>
        {
            Box::pin(async move {
                self.calls.fetch_add(1, Ordering::SeqCst);
                self.requests.lock().unwrap().push(request);
                let plan = self
                    .plans
                    .lock()
                    .unwrap()
                    .pop_front()
                    .expect("each fixture request has one planned connector outcome");
                match plan {
                    ConnectorPlan::Reply {
                        status,
                        headers,
                        peer,
                        chunks,
                        pending_body,
                    } => Ok(ConnectorResponse {
                        status,
                        headers,
                        peer,
                        body: Box::new(FixtureBody {
                            chunks,
                            pending_after_chunks: pending_body,
                        }),
                    }),
                    ConnectorPlan::Pending => pending().await,
                }
            })
        }
    }

    struct OneRequest {
        sent: bool,
        mirrored: Vec<MirroredField>,
    }

    impl OneRequest {
        fn new() -> Self {
            Self {
                sent: false,
                mirrored: Vec::new(),
            }
        }
    }

    impl Conversation for OneRequest {
        fn next_request(&mut self, previous: Option<&ProbeResponse>) -> Option<ProbeRequest> {
            if self.sent || previous.is_some() {
                return None;
            }
            self.sent = true;
            Some(
                ProbeRequest::new(
                    1,
                    br#"{"jsonrpc":"2.0","id":1,"method":"mcp/discover","params":{}}"#.to_vec(),
                )
                .with_mirrored_fields(std::mem::take(&mut self.mirrored)),
            )
        }
    }

    struct TwoRequests {
        next_id: i64,
    }

    impl Conversation for TwoRequests {
        fn next_request(&mut self, previous: Option<&ProbeResponse>) -> Option<ProbeRequest> {
            if self.next_id > 2 {
                return None;
            }
            if self.next_id == 1 {
                assert!(previous.is_none());
            } else {
                assert_eq!(previous.map(ProbeResponse::request_id), Some(1));
            }
            let id = self.next_id;
            self.next_id += 1;
            Some(ProbeRequest::new(
                id,
                format!(
                    "{{\"jsonrpc\":\"2.0\",\"id\":{id},\"method\":\"mcp/discover\",\"params\":{{}}}}"
                )
                .into_bytes(),
            ))
        }
    }

    fn limits() -> HttpLimits {
        HttpLimits {
            startup_ms: 1_000,
            discovery_ms: 1_000,
            request_ms: 1_000,
            response_ms: 1_000,
            shutdown_grace_ms: 100,
            total_ms: 5_000,
            endpoint_bytes: 8_192,
            resolution_addresses: 16,
            trust_bytes: 1_048_576,
            trust_certificates: 32,
            request_fields: 64,
            request_field_name_bytes: 256,
            request_field_value_bytes: 8_192,
            request_fields_bytes: 32_768,
            response_fields: 96,
            response_field_name_bytes: 256,
            response_field_value_bytes: 16_384,
            response_fields_bytes: 65_536,
            message_bytes: 1_048_576,
            aggregate_output_bytes: 8_388_608,
            message_count: 1_024,
            protocol_revisions: 32,
        }
    }

    fn options(endpoint: &str) -> RemoteOptions {
        RemoteOptions {
            endpoint: endpoint.to_owned(),
            ..RemoteOptions::default()
        }
    }

    fn address(ip: &str, port: u16) -> SocketAddr {
        SocketAddr::new(ip.parse().unwrap(), port)
    }

    fn response_headers(media_type: &'static str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static(media_type));
        headers
    }

    fn json_reply(peer: SocketAddr) -> ConnectorPlan {
        json_reply_for(peer, 1)
    }

    fn json_reply_for(peer: SocketAddr, id: i64) -> ConnectorPlan {
        ConnectorPlan::Reply {
            status: StatusCode::OK,
            headers: response_headers("application/json"),
            peer: Some(peer),
            chunks: VecDeque::from([format!(
                "{{\"jsonrpc\":\"2.0\",\"id\":{id},\"result\":{{}}}}"
            )
            .into_bytes()]),
            pending_body: false,
        }
    }

    async fn probe_fixture(
        plan: ConnectorPlan,
        limits: HttpLimits,
        mirrored: Vec<MirroredField>,
    ) -> (
        super::HttpRun,
        Arc<AtomicUsize>,
        Arc<Mutex<Vec<ConnectorRequest>>>,
    ) {
        let target = HttpTarget::prepare(
            options("https://8.8.8.8/mcp"),
            limits,
            &FixedResolver::failed(),
        )
        .await
        .unwrap();
        let connector = FixtureConnector::new([plan]);
        let calls = connector.call_count();
        let requests = connector.requests();
        let transport = HttpTransport::with_connector(target, Box::new(connector));
        let mut conversation = OneRequest {
            sent: false,
            mirrored,
        };
        let run = transport.probe(&mut conversation).await;
        (run, calls, requests)
    }

    #[test]
    fn endpoint_parser_rejects_ambiguous_or_value_bearing_authorities() {
        let maximum = 8_192;
        for endpoint in [
            "https://user@example.test/mcp",
            "https://example.test/mcp?secret=value",
            "https://example.test/mcp#fragment",
            "https://127.1/mcp",
            "https://127.000.000.001/mcp",
            "https://[fe80::1%25en0]/mcp",
            "https://example.test:0/mcp",
            "ftp://example.test/mcp",
        ] {
            assert_eq!(
                parse_endpoint(endpoint, maximum).unwrap_err(),
                super::HttpFailure::Target(TargetFailure::InvalidEndpoint)
            );
        }

        let canonical = parse_endpoint("HTTPS://EXAMPLE.TEST:443/mcp", maximum).unwrap();
        assert_eq!(canonical.url.as_str(), "https://example.test/mcp");
        assert_eq!(canonical.host, "example.test");
        assert_eq!(canonical.port, 443);
        assert!(canonical.explicit_port);
        let ipv6 = parse_endpoint("https://[::1]/mcp", maximum).unwrap();
        assert_eq!(ipv6.host, "::1");
        assert_eq!(ipv6.port, 443);

        assert_eq!(
            parse_endpoint("https://example.test/mcp", 8).unwrap_err(),
            HttpFailure::Limit {
                kind: HttpLimit::EndpointBytes,
                observed: 24,
                maximum: 8,
            }
        );
    }

    #[test]
    fn trust_files_are_regular_pem_only_and_enforce_byte_and_certificate_bounds() {
        const CA: &[u8] = include_bytes!("../../tests/fixtures/http/ca.pem");
        const KEY: &[u8] =
            b"-----BEGIN PRIVATE KEY-----\nsynthetic-fixture-only\n-----END PRIVATE KEY-----\n";
        let root = tempfile::tempdir().expect("a disposable trust root should be created");
        let path = root.path().join("ca.pem");
        fs::write(&path, CA).expect("the synthetic CA should be writable");
        assert_eq!(read_trust_file(Some(&path), limits()).unwrap().len(), 1);

        let mut byte_limited = limits();
        byte_limited.trust_bytes = u64::try_from(CA.len() - 1).unwrap();
        assert_eq!(
            read_trust_file(Some(&path), byte_limited).unwrap_err(),
            HttpFailure::Limit {
                kind: HttpLimit::TrustBytes,
                observed: u64::try_from(CA.len()).unwrap(),
                maximum: u64::try_from(CA.len() - 1).unwrap(),
            }
        );

        let key_path = root.path().join("key.pem");
        fs::write(&key_path, KEY).expect("the synthetic key fixture should be writable");
        assert_eq!(
            read_trust_file(Some(&key_path), limits()).unwrap_err(),
            HttpFailure::Target(TargetFailure::InvalidTrustFile)
        );
        assert_eq!(
            read_trust_file(Some(root.path()), limits()).unwrap_err(),
            HttpFailure::Target(TargetFailure::InvalidTrustFile)
        );

        let many_path = root.path().join("many.pem");
        let many = CA.repeat(33);
        fs::write(&many_path, &many).expect("the synthetic CA bundle should be writable");
        let mut certificate_limited = limits();
        certificate_limited.trust_bytes = u64::try_from(many.len()).unwrap();
        certificate_limited.trust_certificates = 32;
        assert_eq!(
            read_trust_file(Some(&many_path), certificate_limited).unwrap_err(),
            HttpFailure::Limit {
                kind: HttpLimit::TrustCertificates,
                observed: 33,
                maximum: 32,
            }
        );
    }

    #[test]
    fn dated_iana_address_table_distinguishes_public_private_and_prohibited() {
        assert_eq!(
            classify_address(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8))),
            AddressClass::Public
        );
        assert_eq!(
            classify_address(IpAddr::V4(Ipv4Addr::new(192, 0, 0, 9))),
            AddressClass::Public
        );
        assert_eq!(
            classify_address(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))),
            AddressClass::EligiblePrivate
        );
        assert_eq!(
            classify_address(IpAddr::V6(Ipv6Addr::LOCALHOST)),
            AddressClass::Loopback
        );
        assert_eq!(
            classify_address(IpAddr::V4(Ipv4Addr::new(169, 254, 169, 254))),
            AddressClass::Prohibited
        );
        assert_eq!(
            classify_address(IpAddr::V6("2001:db8::1".parse().unwrap())),
            AddressClass::Prohibited
        );
        assert_eq!(
            classify_address(IpAddr::V6("::ffff:10.0.0.1".parse().unwrap())),
            AddressClass::EligiblePrivate
        );
    }

    #[test]
    fn mcp_values_use_the_exact_base64_sentinel_when_plain_ascii_is_unsafe() {
        assert_eq!(encode_mcp_value("us-west1"), "us-west1");
        assert_eq!(
            encode_mcp_value("Hello, 世界"),
            "=?base64?SGVsbG8sIOS4lueVjA==?="
        );
        assert_eq!(encode_mcp_value(" padded "), "=?base64?IHBhZGRlZCA=?=");
        assert_ne!(encode_mcp_value("=?base64?literal?="), "=?base64?literal?=");
    }

    #[test]
    fn credential_grammars_are_deliberately_narrow() {
        assert!(valid_bearer_token("abc.DEF_123-~/+=="));
        assert!(!valid_bearer_token(""));
        assert!(!valid_bearer_token("contains space"));
        assert!(valid_token("X-Synthetic_1"));
        assert!(!valid_token("bad field"));
        assert!(valid_custom_value("synthetic value\twith tab"));
        assert!(!valid_custom_value("line\nbreak"));
        assert!(validate_environment_name("SYNTHETIC_1").is_ok());
        assert!(validate_environment_name("1_SYNTHETIC").is_err());
        for field in [
            "Host",
            "Authorization",
            "Content-Type",
            "Accept-Language",
            "Mcp-Method",
            "Proxy-Synthetic",
            "Sec-Synthetic",
            "X-Forwarded-For",
        ] {
            assert!(reserved_field(field));
        }
    }

    #[test]
    fn request_and_response_field_name_value_and_aggregate_limits_are_independent() {
        let endpoint = parse_endpoint("https://example.test/mcp", 8_192).unwrap();

        let long_name = HeaderName::from_bytes(format!("x{}", "a".repeat(256)).as_bytes())
            .expect("the HTTP type accepts a syntactically valid long field name");
        assert!(matches!(
            validate_request_field_budget(
                &[(long_name.clone(), HeaderValue::from_static("v"))],
                &endpoint,
                1,
                limits(),
            ),
            Err(HttpFailure::Limit {
                kind: HttpLimit::RequestFieldNameBytes,
                observed: 257,
                maximum: 256,
            })
        ));

        let long_value = HeaderValue::from_bytes(&vec![b'v'; 8_193])
            .expect("the HTTP type accepts a bounded synthetic field value");
        assert!(matches!(
            validate_request_field_budget(
                &[(HeaderName::from_static("x-synthetic"), long_value)],
                &endpoint,
                1,
                limits(),
            ),
            Err(HttpFailure::Limit {
                kind: HttpLimit::RequestFieldValueBytes,
                observed: 8_193,
                maximum: 8_192,
            })
        ));

        let mut aggregate_limits = limits();
        aggregate_limits.request_fields_bytes = 1;
        assert!(matches!(
            validate_request_field_budget(
                &[(HeaderName::from_static("x"), HeaderValue::from_static("v"))],
                &endpoint,
                1,
                aggregate_limits,
            ),
            Err(HttpFailure::Limit {
                kind: HttpLimit::RequestFieldsBytes,
                maximum: 1,
                ..
            })
        ));

        let mut response = HeaderMap::new();
        response.insert(long_name, HeaderValue::from_static("v"));
        assert!(matches!(
            validate_response_field_budget(&response, limits()),
            Err(HttpFailure::Limit {
                kind: HttpLimit::ResponseFieldNameBytes,
                observed: 257,
                maximum: 256,
            })
        ));

        let mut response = HeaderMap::new();
        response.insert(
            "x-synthetic",
            HeaderValue::from_bytes(&vec![b'v'; 16_385])
                .expect("the HTTP type accepts a bounded synthetic field value"),
        );
        assert!(matches!(
            validate_response_field_budget(&response, limits()),
            Err(HttpFailure::Limit {
                kind: HttpLimit::ResponseFieldValueBytes,
                observed: 16_385,
                maximum: 16_384,
            })
        ));

        let mut aggregate_limits = limits();
        aggregate_limits.response_fields_bytes = 1;
        let mut response = HeaderMap::new();
        response.insert("x", HeaderValue::from_static("v"));
        assert!(matches!(
            validate_response_field_budget(&response, aggregate_limits),
            Err(HttpFailure::Limit {
                kind: HttpLimit::ResponseFieldsBytes,
                maximum: 1,
                ..
            })
        ));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn resolver_answers_are_bounded_sorted_pinned_and_used_once() {
        assert_eq!(
            collect_resolver_addresses(
                [
                    address("8.8.8.8", 443),
                    address("9.9.9.9", 443),
                    address("1.1.1.1", 443),
                    address("4.2.2.2", 443),
                ],
                2,
            ),
            [
                address("8.8.8.8", 443),
                address("9.9.9.9", 443),
                address("1.1.1.1", 443),
            ]
        );
        let resolver = FixedResolver::new(vec![
            address("9.9.9.9", 9),
            address("8.8.8.8", 8),
            address("9.9.9.9", 99),
        ]);
        let target =
            HttpTarget::prepare(options("https://public.invalid/mcp"), limits(), &resolver)
                .await
                .unwrap();
        assert_eq!(resolver.call_count(), 1);
        assert_eq!(
            target.addresses,
            [address("8.8.8.8", 443), address("9.9.9.9", 443)]
        );
        assert_eq!(
            connection_candidates(&[
                address("8.8.8.8", 443),
                address("9.9.9.9", 443),
                address("2606:4700:4700::1111", 443),
            ]),
            [address("8.8.8.8", 443), address("9.9.9.9", 443)]
        );
        assert_eq!(
            connection_candidates(&[
                address("2606:4700:4700::1111", 443),
                address("8.8.8.8", 443),
            ]),
            [address("2606:4700:4700::1111", 443)]
        );

        let connector = FixtureConnector::new([json_reply(address("9.9.9.9", 443))]);
        let requests = connector.requests();
        let transport = HttpTransport::with_connector(target, Box::new(connector));
        let run = transport.probe(&mut OneRequest::new()).await;
        assert_eq!(run.failure(), None);

        let requests = requests.lock().unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(
            requests[0].accepted_peers,
            [address("8.8.8.8", 443), address("9.9.9.9", 443)]
        );
        let debug = format!("{:?}", requests[0]);
        assert!(!debug.contains("public.invalid"));
        assert!(!debug.contains("8.8.8.8"));
        assert!(!debug.contains("mcp/discover"));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn target_gates_and_address_classes_fail_closed_before_connecting() {
        let mut credentialed = options("https://8.8.8.8/mcp");
        credentialed.bearer_token_env = Some("1_INVALID_SYNTHETIC_SOURCE".to_owned());
        assert_eq!(
            HttpTarget::prepare(credentialed.clone(), limits(), &FixedResolver::failed())
                .await
                .unwrap_err(),
            HttpFailure::Target(TargetFailure::CredentialAuthorizationRequired)
        );
        credentialed.allow_credentials_to = Some("https://8.8.8.8/mcp".to_owned());
        assert_eq!(
            HttpTarget::prepare(credentialed, limits(), &FixedResolver::failed())
                .await
                .unwrap_err(),
            HttpFailure::Target(TargetFailure::InvalidCredential)
        );

        let private = FixedResolver::new(vec![address("10.0.0.8", 443)]);
        let failure =
            HttpTarget::prepare(options("https://private.invalid/mcp"), limits(), &private)
                .await
                .unwrap_err();
        assert_eq!(
            failure,
            HttpFailure::Target(TargetFailure::PrivateNetworkAuthorizationRequired)
        );

        let mut allowed = options("https://private.invalid/mcp");
        allowed.allow_private_network = Some("https://private.invalid/mcp".to_owned());
        assert!(
            HttpTarget::prepare(allowed, limits(), &private)
                .await
                .is_ok()
        );

        let mixed = FixedResolver::new(vec![address("10.0.0.8", 443), address("8.8.8.8", 443)]);
        let mut mixed_options = options("https://mixed.invalid/mcp");
        mixed_options.allow_private_network = Some("https://mixed.invalid/mcp".to_owned());
        assert_eq!(
            HttpTarget::prepare(mixed_options, limits(), &mixed)
                .await
                .unwrap_err(),
            HttpFailure::Resolution(ResolutionFailure::MixedAddressClasses)
        );

        let prohibited = FixedResolver::new(vec![address("169.254.169.254", 443)]);
        let mut prohibited_options = options("https://special.invalid/mcp");
        prohibited_options.allow_private_network = Some("https://special.invalid/mcp".to_owned());
        assert_eq!(
            HttpTarget::prepare(prohibited_options, limits(), &prohibited)
                .await
                .unwrap_err(),
            HttpFailure::Resolution(ResolutionFailure::ProhibitedAddress)
        );

        let overflow = FixedResolver::new(vec![
            address("8.8.8.8", 443),
            address("9.9.9.9", 443),
            address("1.1.1.1", 443),
        ]);
        let mut bounded = limits();
        bounded.resolution_addresses = 2;
        assert_eq!(
            HttpTarget::prepare(options("https://many.invalid/mcp"), bounded, &overflow)
                .await
                .unwrap_err(),
            HttpFailure::Limit {
                kind: HttpLimit::ResolutionAddresses,
                observed: 3,
                maximum: 2,
            }
        );

        assert_eq!(
            HttpTarget::prepare(
                options("https://missing.invalid/mcp"),
                limits(),
                &FixedResolver::failed(),
            )
            .await
            .unwrap_err(),
            HttpFailure::Resolution(ResolutionFailure::Unavailable)
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn resolution_obeys_startup_and_total_deadlines() {
        let mut startup_limited = limits();
        startup_limited.startup_ms = 1;
        assert_eq!(
            HttpTarget::prepare(
                options("https://pending.invalid/mcp"),
                startup_limited,
                &PendingResolver,
            )
            .await
            .unwrap_err(),
            HttpFailure::Limit {
                kind: HttpLimit::StartupTime,
                observed: 2,
                maximum: 1,
            }
        );

        let mut total_limited = limits();
        total_limited.startup_ms = 100;
        total_limited.total_ms = 1;
        assert_eq!(
            HttpTarget::prepare(
                options("https://pending.invalid/mcp"),
                total_limited,
                &PendingResolver,
            )
            .await
            .unwrap_err(),
            HttpFailure::Limit {
                kind: HttpLimit::TotalTime,
                observed: 2,
                maximum: 1,
            }
        );

        let mut connection_limited = limits();
        connection_limited.startup_ms = 1_000;
        let remaining = remaining_connection_time(
            tokio::time::Instant::now() - std::time::Duration::from_millis(100),
            connection_limited,
        )
        .unwrap();
        assert!(remaining <= std::time::Duration::from_millis(900));
        assert_eq!(
            remaining_connection_time(
                tokio::time::Instant::now() - std::time::Duration::from_millis(1_001),
                connection_limited,
            ),
            Err(HttpFailure::Limit {
                kind: HttpLimit::StartupTime,
                observed: 1_001,
                maximum: 1_000,
            })
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn cleartext_requires_both_exact_gates_and_all_loopback_answers() {
        let endpoint = "http://127.0.0.1:8123/mcp";
        let mut missing_cleartext = options(endpoint);
        missing_cleartext.allow_private_network = Some(endpoint.to_owned());
        assert_eq!(
            HttpTarget::prepare(missing_cleartext, limits(), &FixedResolver::failed())
                .await
                .unwrap_err(),
            HttpFailure::Target(TargetFailure::CleartextAuthorizationRequired)
        );

        let mut allowed = options(endpoint);
        allowed.allow_private_network = Some(endpoint.to_owned());
        allowed.allow_cleartext_http = Some(endpoint.to_owned());
        assert!(
            HttpTarget::prepare(allowed, limits(), &FixedResolver::failed())
                .await
                .is_ok()
        );

        let endpoint = "http://private.invalid:8123/mcp";
        let mut non_loopback = options(endpoint);
        non_loopback.allow_private_network = Some(endpoint.to_owned());
        non_loopback.allow_cleartext_http = Some(endpoint.to_owned());
        assert_eq!(
            HttpTarget::prepare(
                non_loopback,
                limits(),
                &FixedResolver::new(vec![address("10.0.0.8", 8123)]),
            )
            .await
            .unwrap_err(),
            HttpFailure::Target(TargetFailure::CleartextAuthorizationRequired)
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn connector_peer_mismatch_and_redirect_never_trigger_a_replay() {
        let (run, calls, _) =
            probe_fixture(json_reply(address("8.8.4.4", 443)), limits(), Vec::new()).await;
        assert_eq!(run.failure(), Some(HttpFailure::PeerMismatch));
        assert_eq!(calls.load(Ordering::SeqCst), 1);

        let mut headers = response_headers("application/json");
        headers.insert(
            LOCATION,
            HeaderValue::from_static("https://redirect.invalid/value-must-not-escape"),
        );
        let (run, calls, _) = probe_fixture(
            ConnectorPlan::Reply {
                status: StatusCode::TEMPORARY_REDIRECT,
                headers,
                peer: Some(address("8.8.8.8", 443)),
                chunks: VecDeque::new(),
                pending_body: false,
            },
            limits(),
            Vec::new(),
        )
        .await;
        assert_eq!(
            run.failure(),
            Some(HttpFailure::Response(ResponseFailure::Redirect {
                status: 307,
            }))
        );
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn current_mcp_fields_and_mirrored_values_are_bounded_and_redacted() {
        let mirrored = vec![MirroredField::new(
            "region".to_owned(),
            "Hello, 世界".to_owned(),
        )];
        let (run, calls, requests) =
            probe_fixture(json_reply(address("8.8.8.8", 443)), limits(), mirrored).await;
        assert_eq!(run.failure(), None);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        {
            let requests = requests.lock().unwrap();
            let headers = &requests[0].headers;
            assert_eq!(headers["mcp-protocol-version"], "2026-07-28");
            assert_eq!(headers["mcp-method"], "mcp/discover");
            assert_eq!(
                headers["mcp-param-region"],
                "=?base64?SGVsbG8sIOS4lueVjA==?="
            );
            assert_eq!(headers["accept-encoding"], "identity");
            let debug = format!("{:?}", requests[0]);
            assert!(!debug.contains("Hello"));
            assert!(!debug.contains("mcp-param-region"));
        }

        let mut too_few_fields = limits();
        too_few_fields.request_fields = 7;
        let connector = FixtureConnector::new([]);
        let calls = connector.call_count();
        let target = HttpTarget::prepare(
            options("https://8.8.8.8/mcp"),
            too_few_fields,
            &FixedResolver::failed(),
        )
        .await
        .unwrap();
        let run = HttpTransport::with_connector(target, Box::new(connector))
            .probe(&mut OneRequest::new())
            .await;
        assert_eq!(
            run.failure(),
            Some(HttpFailure::Limit {
                kind: HttpLimit::RequestFields,
                observed: 8,
                maximum: 7,
            })
        );
        assert_eq!(calls.load(Ordering::SeqCst), 0);

        let (run, calls, _) = probe_fixture(
            json_reply(address("8.8.8.8", 443)),
            limits(),
            vec![MirroredField::new("large".to_owned(), "v".repeat(8_193))],
        )
        .await;
        assert_eq!(
            run.failure(),
            Some(HttpFailure::Limit {
                kind: HttpLimit::RequestFieldValueBytes,
                observed: 8_193,
                maximum: 8_192,
            })
        );
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn unsupported_protocol_version_requires_the_exact_bounded_structured_signal() {
        let valid = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 7,
            "error": {
                "code": -32022,
                "message": "synthetic",
                "data": {
                    "supported": ["2025-11-25", "2025-06-18"],
                    "requested": "2026-07-28"
                }
            }
        });
        assert!(is_unsupported_protocol_version(
            &serde_json::to_vec(&valid).unwrap(),
            7,
            "2026-07-28",
            32,
        ));

        for invalid in [
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 8,
                "error": valid["error"].clone()
            }),
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 7,
                "error": {
                    "code": -32022,
                    "message": "synthetic",
                    "data": {
                        "supported": ["2026-07-28"],
                        "requested": "2026-07-28"
                    }
                }
            }),
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 7,
                "error": {
                    "code": -32022,
                    "message": "synthetic",
                    "data": {
                        "supported": [7],
                        "requested": "2026-07-28"
                    }
                }
            }),
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 7,
                "error": {
                    "code": -32022,
                    "message": "synthetic",
                    "data": {
                        "supported": ["2025-11-25"],
                        "requested": "2025-11-25"
                    }
                }
            }),
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 7,
                "error": {
                    "code": -32022,
                    "data": {
                        "supported": ["2025-11-25"],
                        "requested": "2026-07-28"
                    }
                }
            }),
        ] {
            assert!(!is_unsupported_protocol_version(
                &serde_json::to_vec(&invalid).unwrap(),
                7,
                "2026-07-28",
                32,
            ));
        }

        let too_many = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 7,
            "error": {
                "code": -32022,
                "message": "synthetic",
                "data": {
                    "supported": vec!["2025-11-25"; 33],
                    "requested": "2026-07-28"
                }
            }
        });
        assert!(!is_unsupported_protocol_version(
            &serde_json::to_vec(&too_many).unwrap(),
            7,
            "2026-07-28",
            32,
        ));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn response_status_framing_and_resource_bounds_are_typed() {
        for (status, expected) in [
            (
                StatusCode::UNAUTHORIZED,
                ResponseFailure::Authentication { status: 401 },
            ),
            (
                StatusCode::FORBIDDEN,
                ResponseFailure::Authentication { status: 403 },
            ),
            (
                StatusCode::TOO_MANY_REQUESTS,
                ResponseFailure::Status { status: 429 },
            ),
        ] {
            let (run, calls, _) = probe_fixture(
                ConnectorPlan::Reply {
                    status,
                    headers: response_headers("application/json"),
                    peer: Some(address("8.8.8.8", 443)),
                    chunks: VecDeque::new(),
                    pending_body: false,
                },
                limits(),
                Vec::new(),
            )
            .await;
            assert_eq!(run.failure(), Some(HttpFailure::Response(expected)));
            assert_eq!(calls.load(Ordering::SeqCst), 1);
        }

        let (run, calls, _) = probe_fixture(
            ConnectorPlan::Reply {
                status: StatusCode::BAD_REQUEST,
                headers: response_headers("application/json"),
                peer: Some(address("8.8.8.8", 443)),
                chunks: VecDeque::from([
                    br#"{"jsonrpc":"2.0","id":1,"error":{"code":-32020,"message":"synthetic"}}"#
                        .to_vec(),
                ]),
                pending_body: false,
            },
            limits(),
            Vec::new(),
        )
        .await;
        assert_eq!(
            run.failure(),
            Some(HttpFailure::Response(ResponseFailure::HeaderMismatch))
        );
        assert_eq!(calls.load(Ordering::SeqCst), 1);

        let unsupported = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "error": {
                "code": -32022,
                "message": "synthetic",
                "data": {
                    "supported": ["2025-11-25"],
                    "requested": "2026-07-28"
                }
            }
        });
        let (run, calls, _) = probe_fixture(
            ConnectorPlan::Reply {
                status: StatusCode::BAD_REQUEST,
                headers: response_headers("application/json"),
                peer: Some(address("8.8.8.8", 443)),
                chunks: VecDeque::from([serde_json::to_vec(&unsupported).unwrap()]),
                pending_body: false,
            },
            limits(),
            Vec::new(),
        )
        .await;
        assert_eq!(
            run.failure(),
            Some(HttpFailure::Response(
                ResponseFailure::UnsupportedProtocolVersion
            ))
        );
        assert_eq!(calls.load(Ordering::SeqCst), 1);

        let (run, calls, _) = probe_fixture(
            ConnectorPlan::Reply {
                status: StatusCode::BAD_REQUEST,
                headers: response_headers("text/plain"),
                peer: Some(address("8.8.8.8", 443)),
                chunks: VecDeque::from([serde_json::to_vec(&unsupported).unwrap()]),
                pending_body: false,
            },
            limits(),
            Vec::new(),
        )
        .await;
        assert_eq!(
            run.failure(),
            Some(HttpFailure::Response(ResponseFailure::Status {
                status: 400
            }))
        );
        assert_eq!(calls.load(Ordering::SeqCst), 1);

        let mut encoded_headers = response_headers("application/json");
        encoded_headers.insert("content-encoding", HeaderValue::from_static("gzip"));
        let (run, _, _) = probe_fixture(
            ConnectorPlan::Reply {
                status: StatusCode::OK,
                headers: encoded_headers,
                peer: Some(address("8.8.8.8", 443)),
                chunks: VecDeque::new(),
                pending_body: false,
            },
            limits(),
            Vec::new(),
        )
        .await;
        assert_eq!(
            run.failure(),
            Some(HttpFailure::Response(ResponseFailure::ContentEncoding))
        );

        let (run, _, _) = probe_fixture(
            ConnectorPlan::Reply {
                status: StatusCode::OK,
                headers: response_headers("text/plain"),
                peer: Some(address("8.8.8.8", 443)),
                chunks: VecDeque::new(),
                pending_body: false,
            },
            limits(),
            Vec::new(),
        )
        .await;
        assert_eq!(
            run.failure(),
            Some(HttpFailure::Response(ResponseFailure::MediaType))
        );

        let mut body_limits = limits();
        body_limits.message_bytes = 61;
        let (run, _, _) = probe_fixture(
            ConnectorPlan::Reply {
                status: StatusCode::OK,
                headers: response_headers("application/json"),
                peer: Some(address("8.8.8.8", 443)),
                chunks: VecDeque::from([vec![b'x'; 62]]),
                pending_body: false,
            },
            body_limits,
            Vec::new(),
        )
        .await;
        assert_eq!(
            run.failure(),
            Some(HttpFailure::Limit {
                kind: HttpLimit::MessageBytes,
                observed: 62,
                maximum: 61,
            })
        );

        let mut aggregate_limits = limits();
        aggregate_limits.message_bytes = 100;
        aggregate_limits.aggregate_output_bytes = 61;
        let (run, _, _) = probe_fixture(
            ConnectorPlan::Reply {
                status: StatusCode::OK,
                headers: response_headers("application/json"),
                peer: Some(address("8.8.8.8", 443)),
                chunks: VecDeque::from([vec![b'x'; 62]]),
                pending_body: false,
            },
            aggregate_limits,
            Vec::new(),
        )
        .await;
        assert_eq!(
            run.failure(),
            Some(HttpFailure::Limit {
                kind: HttpLimit::AggregateOutputBytes,
                observed: 62,
                maximum: 61,
            })
        );

        let first_bytes = br#"{"jsonrpc":"2.0","id":1,"result":{}}"#.len();
        let second_bytes = br#"{"jsonrpc":"2.0","id":2,"result":{}}"#.len();
        let aggregate_maximum =
            u64::try_from(first_bytes.saturating_add(second_bytes).saturating_sub(1)).unwrap();
        let mut conversation_limits = limits();
        conversation_limits.aggregate_output_bytes = aggregate_maximum;
        let target = HttpTarget::prepare(
            options("https://8.8.8.8/mcp"),
            conversation_limits,
            &FixedResolver::failed(),
        )
        .await
        .unwrap();
        let connector = FixtureConnector::new([
            json_reply_for(address("8.8.8.8", 443), 1),
            json_reply_for(address("8.8.8.8", 443), 2),
        ]);
        let run = HttpTransport::with_connector(target, Box::new(connector))
            .probe(&mut TwoRequests { next_id: 1 })
            .await;
        assert_eq!(
            run.failure(),
            Some(HttpFailure::Limit {
                kind: HttpLimit::AggregateOutputBytes,
                observed: aggregate_maximum + 1,
                maximum: aggregate_maximum,
            })
        );

        let mut conversation_limits = limits();
        conversation_limits.message_count = 1;
        let target = HttpTarget::prepare(
            options("https://8.8.8.8/mcp"),
            conversation_limits,
            &FixedResolver::failed(),
        )
        .await
        .unwrap();
        let connector = FixtureConnector::new([
            json_reply_for(address("8.8.8.8", 443), 1),
            json_reply_for(address("8.8.8.8", 443), 2),
        ]);
        let run = HttpTransport::with_connector(target, Box::new(connector))
            .probe(&mut TwoRequests { next_id: 1 })
            .await;
        assert_eq!(
            run.failure(),
            Some(HttpFailure::Limit {
                kind: HttpLimit::MessageCount,
                observed: 2,
                maximum: 1,
            })
        );

        let mut field_headers = response_headers("application/json");
        field_headers.insert("x-synthetic-one", HeaderValue::from_static("one"));
        let mut field_limits = limits();
        field_limits.response_fields = 1;
        let (run, _, _) = probe_fixture(
            ConnectorPlan::Reply {
                status: StatusCode::OK,
                headers: field_headers,
                peer: Some(address("8.8.8.8", 443)),
                chunks: VecDeque::new(),
                pending_body: false,
            },
            field_limits,
            Vec::new(),
        )
        .await;
        assert_eq!(
            run.failure(),
            Some(HttpFailure::Limit {
                kind: HttpLimit::ResponseFields,
                observed: 2,
                maximum: 1,
            })
        );

        let events_after_response = concat!(
            "data: {\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{}}\n\n",
            "data: {\"jsonrpc\":\"2.0\",\"method\":\"notifications/progress\"}\n\n"
        );
        let (run, _, _) = probe_fixture(
            ConnectorPlan::Reply {
                status: StatusCode::OK,
                headers: response_headers("text/event-stream"),
                peer: Some(address("8.8.8.8", 443)),
                chunks: VecDeque::from([events_after_response.as_bytes().to_vec()]),
                pending_body: false,
            },
            limits(),
            Vec::new(),
        )
        .await;
        assert_eq!(run.failure(), None);
        assert_eq!(run.responses().len(), 1);

        let mut event_limits = limits();
        event_limits.message_count = 1;
        let events = concat!(
            "data: {\"jsonrpc\":\"2.0\",\"method\":\"notifications/progress\"}\n\n",
            "data: {\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{}}\n\n"
        );
        let (run, _, _) = probe_fixture(
            ConnectorPlan::Reply {
                status: StatusCode::OK,
                headers: response_headers("text/event-stream"),
                peer: Some(address("8.8.8.8", 443)),
                chunks: VecDeque::from([events.as_bytes().to_vec()]),
                pending_body: false,
            },
            event_limits,
            Vec::new(),
        )
        .await;
        assert_eq!(
            run.failure(),
            Some(HttpFailure::Limit {
                kind: HttpLimit::MessageCount,
                observed: 2,
                maximum: 1,
            })
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn request_stage_and_total_deadlines_remain_distinct() {
        let mut request_limits = limits();
        request_limits.request_ms = 1;
        let (run, calls, _) =
            probe_fixture(ConnectorPlan::Pending, request_limits, Vec::new()).await;
        assert_eq!(
            run.failure(),
            Some(HttpFailure::Limit {
                kind: HttpLimit::RequestTime,
                observed: 2,
                maximum: 1,
            })
        );
        assert_eq!(calls.load(Ordering::SeqCst), 1);

        let mut response_limits = limits();
        response_limits.discovery_ms = 1;
        let (run, _, _) = probe_fixture(
            ConnectorPlan::Reply {
                status: StatusCode::OK,
                headers: response_headers("application/json"),
                peer: Some(address("8.8.8.8", 443)),
                chunks: VecDeque::new(),
                pending_body: true,
            },
            response_limits,
            Vec::new(),
        )
        .await;
        assert_eq!(
            run.failure(),
            Some(HttpFailure::Limit {
                kind: HttpLimit::DiscoveryTime,
                observed: 2,
                maximum: 1,
            })
        );

        let mut response_limits = limits();
        response_limits.response_ms = 1;
        let target = HttpTarget::prepare(
            options("https://8.8.8.8/mcp"),
            response_limits,
            &FixedResolver::failed(),
        )
        .await
        .unwrap();
        let connector = FixtureConnector::new([
            json_reply_for(address("8.8.8.8", 443), 1),
            ConnectorPlan::Reply {
                status: StatusCode::OK,
                headers: response_headers("application/json"),
                peer: Some(address("8.8.8.8", 443)),
                chunks: VecDeque::new(),
                pending_body: true,
            },
        ]);
        let run = HttpTransport::with_connector(target, Box::new(connector))
            .probe(&mut TwoRequests { next_id: 1 })
            .await;
        assert_eq!(
            run.failure(),
            Some(HttpFailure::Limit {
                kind: HttpLimit::ResponseTime,
                observed: 2,
                maximum: 1,
            })
        );

        let mut total_limits = limits();
        total_limits.total_ms = 1;
        let (run, _, _) = probe_fixture(ConnectorPlan::Pending, total_limits, Vec::new()).await;
        assert_eq!(
            run.failure(),
            Some(HttpFailure::Limit {
                kind: HttpLimit::TotalTime,
                observed: 2,
                maximum: 1,
            })
        );
    }
}
