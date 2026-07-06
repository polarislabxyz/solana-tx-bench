use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use chrono::{DateTime, Utc};
use reqwest::StatusCode;
use rpcedge_relay_client::{
    QuicRelayClient, QuicRelayClientConfig, RelayRoute, ResponseMode,
    RouteResultResponse as RelayRouteResultResponse, RouteSet as RelayRouteSet,
    RouteSetMode as RelayRouteSetMode,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{collections::HashMap, net::SocketAddr, time::Duration};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ProviderKind {
    SolanaRpc,
    RpcedgeRawHttp,
    RpcedgeRouteAwareHttp,
    RpcedgeQuicRawTx,
    HeliusSender,
    AstralaneSender,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProviderConfig {
    SolanaRpc {
        endpoint: String,
        #[serde(default)]
        headers: HashMap<String, String>,
    },
    RpcedgeRawHttp {
        endpoint: String,
        #[serde(default)]
        api_key_env: Option<String>,
        #[serde(default)]
        headers: HashMap<String, String>,
    },
    RpcedgeRouteAwareHttp {
        endpoint: String,
        #[serde(default)]
        api_key_env: Option<String>,
        #[serde(default)]
        route_mode: RouteMode,
        #[serde(default)]
        routes: Vec<String>,
        #[serde(default)]
        headers: HashMap<String, String>,
    },
    RpcedgeQuicRawTx {
        endpoint: SocketAddr,
        api_key_env: String,
        #[serde(default)]
        route_mode: RouteMode,
        #[serde(default)]
        routes: Vec<String>,
        #[serde(default)]
        server_name: Option<String>,
        #[serde(default)]
        route_results: bool,
    },
    HeliusSender {
        #[serde(default = "helius_sender_default_endpoint")]
        endpoint: String,
        #[serde(default)]
        api_key_env: Option<String>,
        #[serde(default)]
        swqos_only: bool,
        #[serde(default)]
        headers: HashMap<String, String>,
    },
    AstralaneSender {
        endpoint: String,
        #[serde(default)]
        api_key_env: Option<String>,
        #[serde(default)]
        mev_protect: bool,
        #[serde(default)]
        headers: HashMap<String, String>,
    },
}

impl ProviderConfig {
    pub fn kind(&self) -> ProviderKind {
        match self {
            Self::SolanaRpc { .. } => ProviderKind::SolanaRpc,
            Self::RpcedgeRawHttp { .. } => ProviderKind::RpcedgeRawHttp,
            Self::RpcedgeRouteAwareHttp { .. } => ProviderKind::RpcedgeRouteAwareHttp,
            Self::RpcedgeQuicRawTx { .. } => ProviderKind::RpcedgeQuicRawTx,
            Self::HeliusSender { .. } => ProviderKind::HeliusSender,
            Self::AstralaneSender { .. } => ProviderKind::AstralaneSender,
        }
    }

    pub fn configured_route_mode(&self) -> Option<RouteMode> {
        match self {
            Self::RpcedgeRouteAwareHttp { route_mode, .. }
            | Self::RpcedgeQuicRawTx { route_mode, .. } => Some(*route_mode),
            _ => None,
        }
    }

    pub fn configured_routes(&self) -> Vec<String> {
        match self {
            Self::RpcedgeRouteAwareHttp { routes, .. } | Self::RpcedgeQuicRawTx { routes, .. } => {
                routes.clone()
            }
            _ => Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RouteMode {
    #[default]
    ServerDefault,
    Only,
    DefaultPlus,
    DefaultMinus,
}

impl RouteMode {
    pub fn as_wire(self) -> &'static str {
        match self {
            Self::ServerDefault => "server_default",
            Self::Only => "only",
            Self::DefaultPlus => "default_plus",
            Self::DefaultMinus => "default_minus",
        }
    }

    pub fn as_relay_mode(self) -> RelayRouteSetMode {
        match self {
            Self::ServerDefault => RelayRouteSetMode::ServerDefault,
            Self::Only => RelayRouteSetMode::Only,
            Self::DefaultPlus => RelayRouteSetMode::DefaultPlus,
            Self::DefaultMinus => RelayRouteSetMode::DefaultMinus,
        }
    }
}

pub fn parse_relay_route(route: &str) -> Result<RelayRoute> {
    match route.trim() {
        "tpu_quic" | "tpu-quic" => Ok(RelayRoute::TpuQuic),
        "tpu_udp" | "tpu-udp" => Ok(RelayRoute::TpuUdp),
        "harmonic_bundle" | "harmonic-bundle" => Ok(RelayRoute::HarmonicBundle),
        "helius_sender_swqos" | "helius-sender-swqos" => Ok(RelayRoute::HeliusSenderSwqos),
        "rpc_fallback" | "rpc-fallback" | "rpc" => Ok(RelayRoute::RpcFallback),
        "jito_transaction" | "jito-transaction" => Ok(RelayRoute::JitoTransaction),
        "jito_bundle" | "jito-bundle" => Ok(RelayRoute::JitoBundle),
        "" => Err(anyhow!("empty relay route")),
        other => Err(anyhow!("unknown relay route {other}")),
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct RouteSelection {
    pub policy: String,
    pub mode: RouteMode,
    pub routes: Vec<String>,
}

impl RouteSelection {
    pub fn only(
        policy: impl Into<String>,
        routes: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            policy: policy.into(),
            mode: RouteMode::Only,
            routes: routes.into_iter().map(Into::into).collect(),
        }
    }

    fn as_relay_route_set(&self) -> Result<RelayRouteSet> {
        Ok(RelayRouteSet {
            mode: self.mode.as_relay_mode(),
            routes: self
                .routes
                .iter()
                .map(|route| parse_relay_route(route))
                .collect::<Result<Vec<_>>>()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct SendContext {
    pub test_id: String,
    pub iteration: usize,
    pub signature: String,
    pub tx_base64: String,
    pub timeout: Duration,
    pub route_selection: Option<RouteSelection>,
    pub leader_client_family: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProviderAck {
    pub provider_name: String,
    pub provider_kind: ProviderKind,
    pub accepted: bool,
    pub provider_request_id: Option<String>,
    pub returned_signature: Option<String>,
    pub status_code: Option<u16>,
    pub error_class: Option<String>,
    pub error: Option<String>,
    pub send_started_at: DateTime<Utc>,
    pub send_finished_at: DateTime<Utc>,
    pub ack_latency_us: u128,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub route_results: Vec<RouteResultAck>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub route_results_complete: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RouteResultAck {
    pub route: String,
    pub route_name: String,
    pub provider: String,
    pub accepted: bool,
    pub latency_us: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_class: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bundle_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub route_tip_signature: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub route_tip_lamports: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub route_tip_account: Option<String>,
}

impl From<RelayRouteResultResponse> for RouteResultAck {
    fn from(value: RelayRouteResultResponse) -> Self {
        Self {
            route: value.route,
            route_name: value.route_name,
            provider: value.provider,
            accepted: value.accepted,
            latency_us: value.latency_us,
            error_class: value.error_class,
            bundle_id: value.bundle_id,
            route_tip_signature: value.route_tip_signature,
            route_tip_lamports: value.route_tip_lamports,
            route_tip_account: value.route_tip_account,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ProviderError {
    pub class: String,
    pub message: String,
    pub status_code: Option<u16>,
}

#[async_trait]
pub trait ProviderAdapter: Send + Sync {
    fn name(&self) -> &str;
    fn kind(&self) -> ProviderKind;
    async fn warmup(&mut self) -> Result<()> {
        Ok(())
    }
    async fn send_transaction(&self, tx: &[u8], ctx: &SendContext) -> ProviderAck;
}

pub fn build_adapter(
    name: String,
    config: ProviderConfig,
    timeout: Duration,
) -> Result<Box<dyn ProviderAdapter>> {
    let client = reqwest::Client::builder()
        .timeout(timeout)
        .pool_idle_timeout(Duration::from_secs(30))
        .build()
        .context("build HTTP client")?;
    match config {
        ProviderConfig::SolanaRpc { endpoint, headers } => Ok(Box::new(JsonRpcAdapter {
            name,
            endpoint,
            headers,
            client,
        })),
        ProviderConfig::RpcedgeRawHttp {
            endpoint,
            api_key_env,
            headers,
        } => Ok(Box::new(RpcedgeRawHttpAdapter {
            name,
            endpoint,
            api_key: read_optional_secret(api_key_env)?,
            headers,
            client,
        })),
        ProviderConfig::RpcedgeRouteAwareHttp {
            endpoint,
            api_key_env,
            route_mode,
            routes,
            headers,
        } => Ok(Box::new(RpcedgeRouteAwareAdapter {
            name,
            endpoint,
            api_key: read_optional_secret(api_key_env)?,
            route_mode,
            routes,
            headers,
            client,
        })),
        ProviderConfig::RpcedgeQuicRawTx {
            endpoint,
            api_key_env,
            route_mode,
            routes,
            server_name,
            route_results,
        } => Ok(Box::new(RpcedgeQuicRawTxAdapter {
            name,
            endpoint,
            api_key: std::env::var(&api_key_env)
                .with_context(|| format!("read API key env var {api_key_env}"))?,
            route_mode,
            routes: routes
                .iter()
                .map(|route| parse_relay_route(route))
                .collect::<Result<Vec<_>>>()?,
            server_name,
            route_results,
            timeout,
            client: None,
        })),
        ProviderConfig::HeliusSender {
            endpoint,
            api_key_env,
            swqos_only,
            headers,
        } => {
            let api_key = read_optional_secret(api_key_env)?;
            let url = helius_sender_url(&endpoint, swqos_only, api_key.as_deref());
            Ok(Box::new(HeliusSenderAdapter {
                name,
                url,
                headers,
                client,
            }))
        }
        ProviderConfig::AstralaneSender {
            endpoint,
            api_key_env,
            mev_protect,
            headers,
        } => {
            let api_key = read_optional_secret(api_key_env)?;
            let url = astralane_sender_url(&endpoint, api_key.as_deref());
            Ok(Box::new(AstralaneSenderAdapter {
                name,
                url,
                mev_protect,
                headers,
                client,
            }))
        }
    }
}

fn read_optional_secret(env_var: Option<String>) -> Result<Option<String>> {
    env_var
        .map(|name| std::env::var(&name).with_context(|| format!("read API key env var {name}")))
        .transpose()
}

struct JsonRpcAdapter {
    name: String,
    endpoint: String,
    headers: HashMap<String, String>,
    client: reqwest::Client,
}

struct RpcedgeRawHttpAdapter {
    name: String,
    endpoint: String,
    api_key: Option<String>,
    headers: HashMap<String, String>,
    client: reqwest::Client,
}

struct RpcedgeRouteAwareAdapter {
    name: String,
    endpoint: String,
    api_key: Option<String>,
    route_mode: RouteMode,
    routes: Vec<String>,
    headers: HashMap<String, String>,
    client: reqwest::Client,
}

struct RpcedgeQuicRawTxAdapter {
    name: String,
    endpoint: SocketAddr,
    api_key: String,
    route_mode: RouteMode,
    routes: Vec<RelayRoute>,
    server_name: Option<String>,
    route_results: bool,
    timeout: Duration,
    client: Option<QuicRelayClient>,
}

struct HeliusSenderAdapter {
    name: String,
    url: String,
    headers: HashMap<String, String>,
    client: reqwest::Client,
}

struct AstralaneSenderAdapter {
    name: String,
    url: String,
    mev_protect: bool,
    headers: HashMap<String, String>,
    client: reqwest::Client,
}

#[async_trait]
impl ProviderAdapter for JsonRpcAdapter {
    fn name(&self) -> &str {
        &self.name
    }

    fn kind(&self) -> ProviderKind {
        ProviderKind::SolanaRpc
    }

    async fn send_transaction(&self, _tx: &[u8], ctx: &SendContext) -> ProviderAck {
        let body = json!({
            "jsonrpc": "2.0",
            "id": ctx.iteration,
            "method": "sendTransaction",
            "params": [
                ctx.tx_base64,
                {
                    "encoding": "base64",
                    "skipPreflight": true,
                    "maxRetries": 0
                }
            ]
        });
        send_json(
            &self.client,
            &self.endpoint,
            &self.headers,
            None,
            self.name(),
            self.kind(),
            ctx,
            body,
            parse_json_rpc_response,
        )
        .await
    }
}

#[async_trait]
impl ProviderAdapter for RpcedgeRawHttpAdapter {
    fn name(&self) -> &str {
        &self.name
    }

    fn kind(&self) -> ProviderKind {
        ProviderKind::RpcedgeRawHttp
    }

    async fn send_transaction(&self, tx: &[u8], ctx: &SendContext) -> ProviderAck {
        let started = Utc::now();
        let timer = std::time::Instant::now();
        let mut request = self
            .client
            .post(&self.endpoint)
            .header("content-type", "application/octet-stream")
            .body(tx.to_vec());
        request = apply_headers(request, &self.headers, self.api_key.as_deref());
        finish_response(
            self.name(),
            self.kind(),
            started,
            timer,
            ctx,
            request.send().await,
        )
        .await
    }
}

#[async_trait]
impl ProviderAdapter for RpcedgeRouteAwareAdapter {
    fn name(&self) -> &str {
        &self.name
    }

    fn kind(&self) -> ProviderKind {
        ProviderKind::RpcedgeRouteAwareHttp
    }

    async fn send_transaction(&self, _tx: &[u8], ctx: &SendContext) -> ProviderAck {
        let mode = ctx
            .route_selection
            .as_ref()
            .map(|selection| selection.mode)
            .unwrap_or(self.route_mode);
        let routes = ctx
            .route_selection
            .as_ref()
            .map(|selection| selection.routes.clone())
            .unwrap_or_else(|| self.routes.clone());
        let route_set = json!({
            "mode": mode.as_wire(),
            "routes": routes,
        });
        let body = json!({
            "jsonrpc": "2.0",
            "id": ctx.iteration,
            "method": "sendTransaction",
            "request_id": format!("{}-{}-{}", ctx.test_id, self.name, ctx.iteration),
            "params": {
                "transaction": ctx.tx_base64,
                "encoding": "base64",
            },
            "route_set": route_set,
        });
        send_json(
            &self.client,
            &self.endpoint,
            &self.headers,
            self.api_key.as_deref(),
            self.name(),
            self.kind(),
            ctx,
            body,
            parse_rpcedge_submit_response,
        )
        .await
    }
}

#[async_trait]
impl ProviderAdapter for RpcedgeQuicRawTxAdapter {
    fn name(&self) -> &str {
        &self.name
    }

    fn kind(&self) -> ProviderKind {
        ProviderKind::RpcedgeQuicRawTx
    }

    async fn warmup(&mut self) -> Result<()> {
        let mut config = QuicRelayClientConfig::new(self.endpoint, self.api_key.clone())
            .with_timeout(self.timeout)
            .with_max_response_bytes(64 * 1024);
        if let Some(server_name) = self.server_name.as_ref() {
            config = config.with_server_name(server_name.clone());
        }
        let client = QuicRelayClient::connect(config)
            .await
            .map_err(|err| anyhow!("connect QUIC relay client: {err}"))?;
        self.client = Some(client);
        Ok(())
    }

    async fn send_transaction(&self, tx: &[u8], ctx: &SendContext) -> ProviderAck {
        let started = Utc::now();
        let timer = std::time::Instant::now();
        let Some(client) = self.client.as_ref() else {
            return ack_error(
                self.name(),
                self.kind(),
                started,
                timer,
                None,
                "invalid_config",
                "QUIC client was not warmed up",
            );
        };

        let route_set = match ctx.route_selection.as_ref() {
            Some(selection) => match selection.as_relay_route_set() {
                Ok(route_set) => route_set,
                Err(err) => {
                    return ack_error(
                        self.name(),
                        self.kind(),
                        started,
                        timer,
                        None,
                        "invalid_route_selection",
                        err,
                    )
                }
            },
            None => RelayRouteSet {
                mode: self.route_mode.as_relay_mode(),
                routes: self.routes.clone(),
            },
        };
        let request_id = Some(format!("{}-{}-{}", ctx.test_id, self.name, ctx.iteration));
        let response_mode = self.route_results.then_some(ResponseMode::RouteResults);
        match client
            .send_transaction_raw_bytes_with_request_id_and_response_mode(
                tx,
                route_set,
                request_id,
                response_mode,
            )
            .await
        {
            Ok(response) => {
                let route_results = response.route_results.into_iter().map(Into::into).collect();
                ProviderAck {
                    provider_name: self.name().to_string(),
                    provider_kind: self.kind(),
                    accepted: response.accepted,
                    provider_request_id: Some(response.request_id),
                    returned_signature: Some(response.signature),
                    status_code: None,
                    error_class: None,
                    error: None,
                    send_started_at: started,
                    send_finished_at: Utc::now(),
                    ack_latency_us: timer.elapsed().as_micros(),
                    route_results,
                    route_results_complete: response.route_results_complete,
                }
            }
            Err(err) => ack_error(
                self.name(),
                self.kind(),
                started,
                timer,
                None,
                "transport_error",
                err,
            ),
        }
    }
}

#[async_trait]
impl ProviderAdapter for HeliusSenderAdapter {
    fn name(&self) -> &str {
        &self.name
    }

    fn kind(&self) -> ProviderKind {
        ProviderKind::HeliusSender
    }

    async fn send_transaction(&self, _tx: &[u8], ctx: &SendContext) -> ProviderAck {
        let body = json!({
            "jsonrpc": "2.0",
            "id": ctx.iteration,
            "method": "sendTransaction",
            "params": [
                ctx.tx_base64,
                {
                    "encoding": "base64",
                    "skipPreflight": true,
                    "maxRetries": 0
                }
            ]
        });
        send_json(
            &self.client,
            &self.url,
            &self.headers,
            None,
            self.name(),
            self.kind(),
            ctx,
            body,
            parse_json_rpc_response,
        )
        .await
    }
}

#[async_trait]
impl ProviderAdapter for AstralaneSenderAdapter {
    fn name(&self) -> &str {
        &self.name
    }

    fn kind(&self) -> ProviderKind {
        ProviderKind::AstralaneSender
    }

    async fn send_transaction(&self, _tx: &[u8], ctx: &SendContext) -> ProviderAck {
        let mut params = vec![
            json!(ctx.tx_base64),
            json!({"encoding": "base64", "skipPreflight": true}),
        ];
        if self.mev_protect {
            params.push(json!({"mevProtect": true}));
        }
        let body = json!({
            "jsonrpc": "2.0",
            "id": ctx.iteration,
            "method": "sendTransaction",
            "params": params,
        });
        send_json(
            &self.client,
            &self.url,
            &self.headers,
            None,
            self.name(),
            self.kind(),
            ctx,
            body,
            parse_json_rpc_response,
        )
        .await
    }
}

async fn send_json<F>(
    client: &reqwest::Client,
    endpoint: &str,
    headers: &HashMap<String, String>,
    api_key: Option<&str>,
    name: &str,
    kind: ProviderKind,
    ctx: &SendContext,
    body: serde_json::Value,
    parser: F,
) -> ProviderAck
where
    F: Fn(
        StatusCode,
        &serde_json::Value,
        &SendContext,
    ) -> std::result::Result<(bool, Option<String>, Option<String>), ProviderError>,
{
    let started = Utc::now();
    let timer = std::time::Instant::now();
    let mut request = client.post(endpoint).json(&body);
    request = apply_headers(request, headers, api_key);
    match request.send().await {
        Ok(response) => {
            let status = response.status();
            match response.json::<serde_json::Value>().await {
                Ok(value) => {
                    let finished = Utc::now();
                    let latency = timer.elapsed().as_micros();
                    match parser(status, &value, ctx) {
                        Ok((accepted, provider_request_id, returned_signature)) => ProviderAck {
                            provider_name: name.to_string(),
                            provider_kind: kind,
                            accepted,
                            provider_request_id,
                            returned_signature,
                            status_code: Some(status.as_u16()),
                            error_class: None,
                            error: None,
                            send_started_at: started,
                            send_finished_at: finished,
                            ack_latency_us: latency,
                            route_results: Vec::new(),
                            route_results_complete: None,
                        },
                        Err(err) => ProviderAck {
                            provider_name: name.to_string(),
                            provider_kind: kind,
                            accepted: false,
                            provider_request_id: None,
                            returned_signature: None,
                            status_code: err.status_code.or(Some(status.as_u16())),
                            error_class: Some(err.class),
                            error: Some(err.message),
                            send_started_at: started,
                            send_finished_at: finished,
                            ack_latency_us: latency,
                            route_results: Vec::new(),
                            route_results_complete: None,
                        },
                    }
                }
                Err(err) => ack_error(
                    name,
                    kind,
                    started,
                    timer,
                    Some(status.as_u16()),
                    "decode_error",
                    err,
                ),
            }
        }
        Err(err) => ack_error(name, kind, started, timer, None, "transport_error", err),
    }
}

async fn finish_response(
    name: &str,
    kind: ProviderKind,
    started: DateTime<Utc>,
    timer: std::time::Instant,
    ctx: &SendContext,
    response: Result<reqwest::Response, reqwest::Error>,
) -> ProviderAck {
    match response {
        Ok(response) => {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            let finished = Utc::now();
            let latency = timer.elapsed().as_micros();
            if status.is_success() {
                let parsed = serde_json::from_str::<serde_json::Value>(&body).ok();
                let returned_signature = parsed
                    .as_ref()
                    .and_then(|v| v.get("signature"))
                    .and_then(|v| v.as_str())
                    .or_else(|| {
                        parsed
                            .as_ref()
                            .and_then(|v| v.get("result"))
                            .and_then(|v| v.as_str())
                    })
                    .map(ToOwned::to_owned)
                    .or_else(|| Some(ctx.signature.clone()));
                let provider_request_id = parsed
                    .as_ref()
                    .and_then(|v| v.get("request_id"))
                    .and_then(|v| v.as_str())
                    .map(ToOwned::to_owned);
                ProviderAck {
                    provider_name: name.to_string(),
                    provider_kind: kind,
                    accepted: true,
                    provider_request_id,
                    returned_signature,
                    status_code: Some(status.as_u16()),
                    error_class: None,
                    error: None,
                    send_started_at: started,
                    send_finished_at: finished,
                    ack_latency_us: latency,
                    route_results: Vec::new(),
                    route_results_complete: None,
                }
            } else {
                ProviderAck {
                    provider_name: name.to_string(),
                    provider_kind: kind,
                    accepted: false,
                    provider_request_id: None,
                    returned_signature: None,
                    status_code: Some(status.as_u16()),
                    error_class: Some("provider_error".to_string()),
                    error: Some(body),
                    send_started_at: started,
                    send_finished_at: finished,
                    ack_latency_us: latency,
                    route_results: Vec::new(),
                    route_results_complete: None,
                }
            }
        }
        Err(err) => ack_error(name, kind, started, timer, None, "transport_error", err),
    }
}

fn ack_error<E: std::fmt::Display>(
    name: &str,
    kind: ProviderKind,
    started: DateTime<Utc>,
    timer: std::time::Instant,
    status_code: Option<u16>,
    class: &str,
    err: E,
) -> ProviderAck {
    ProviderAck {
        provider_name: name.to_string(),
        provider_kind: kind,
        accepted: false,
        provider_request_id: None,
        returned_signature: None,
        status_code,
        error_class: Some(class.to_string()),
        error: Some(err.to_string()),
        send_started_at: started,
        send_finished_at: Utc::now(),
        ack_latency_us: timer.elapsed().as_micros(),
        route_results: Vec::new(),
        route_results_complete: None,
    }
}

fn apply_headers(
    mut request: reqwest::RequestBuilder,
    headers: &HashMap<String, String>,
    api_key: Option<&str>,
) -> reqwest::RequestBuilder {
    for (key, value) in headers {
        request = request.header(key, value);
    }
    if let Some(api_key) = api_key {
        request = request.header("x-api-key", api_key);
    }
    request
}

fn parse_json_rpc_response(
    status: StatusCode,
    value: &serde_json::Value,
    _ctx: &SendContext,
) -> std::result::Result<(bool, Option<String>, Option<String>), ProviderError> {
    if !status.is_success() {
        return Err(provider_error(status, "provider_error", value));
    }
    if let Some(error) = value.get("error") {
        return Err(ProviderError {
            class: "provider_error".to_string(),
            message: error.to_string(),
            status_code: Some(status.as_u16()),
        });
    }
    let signature = value
        .get("result")
        .and_then(|v| v.as_str())
        .map(ToOwned::to_owned)
        .ok_or_else(|| ProviderError {
            class: "missing_signature".to_string(),
            message: format!("JSON-RPC response did not include result signature: {value}"),
            status_code: Some(status.as_u16()),
        })?;
    Ok((true, None, Some(signature)))
}

fn parse_rpcedge_submit_response(
    status: StatusCode,
    value: &serde_json::Value,
    ctx: &SendContext,
) -> std::result::Result<(bool, Option<String>, Option<String>), ProviderError> {
    if !status.is_success() {
        return Err(provider_error(status, "provider_error", value));
    }
    let accepted = value
        .get("accepted")
        .and_then(|v| v.as_bool())
        .unwrap_or(status.is_success());
    let provider_request_id = value
        .get("request_id")
        .and_then(|v| v.as_str())
        .map(ToOwned::to_owned);
    let signature = value
        .get("signature")
        .and_then(|v| v.as_str())
        .map(ToOwned::to_owned)
        .or_else(|| Some(ctx.signature.clone()));
    Ok((accepted, provider_request_id, signature))
}

fn provider_error(status: StatusCode, class: &str, value: &serde_json::Value) -> ProviderError {
    ProviderError {
        class: class.to_string(),
        message: value.to_string(),
        status_code: Some(status.as_u16()),
    }
}

/// Join query parameters onto a base URL, using `?` or `&` as appropriate and
/// skipping parameters whose value is empty. Existing query strings are honored.
fn append_query(base: &str, params: &[(&str, &str)]) -> String {
    let mut url = base.to_string();
    let mut has_query = base.contains('?');
    for (key, value) in params {
        if value.is_empty() {
            continue;
        }
        url.push(if has_query { '&' } else { '?' });
        url.push_str(key);
        url.push('=');
        url.push_str(value);
        has_query = true;
    }
    url
}

fn helius_sender_default_endpoint() -> String {
    "https://sender.helius-rpc.com/fast".to_string()
}

/// Build the effective Helius Sender URL. `swqos_only` selects the cheaper
/// SWQoS-only path; `api_key` is only needed for approved higher-TPS accounts.
fn helius_sender_url(base: &str, swqos_only: bool, api_key: Option<&str>) -> String {
    let mut params: Vec<(&str, &str)> = Vec::new();
    if swqos_only {
        params.push(("swqos_only", "true"));
    }
    if let Some(key) = api_key {
        params.push(("api-key", key));
    }
    append_query(base, &params)
}

/// Build the effective Astralane Iris URL. Astralane passes the API key as the
/// `api-key` query parameter (its primary auth mechanism).
fn astralane_sender_url(base: &str, api_key: Option<&str>) -> String {
    match api_key {
        Some(key) => append_query(base, &[("api-key", key)]),
        None => base.to_string(),
    }
}

pub fn base64_tx(tx: &[u8]) -> String {
    STANDARD.encode(tx)
}

pub fn decode_tx_base64(encoded: &str) -> Result<Vec<u8>> {
    STANDARD
        .decode(encoded)
        .map_err(|err| anyhow!("decode base64 transaction: {err}"))
}

#[cfg(test)]
mod external_sender_tests {
    use super::*;
    use wiremock::{
        matchers::{body_json, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    #[test]
    fn append_query_uses_question_mark_then_ampersand() {
        assert_eq!(
            append_query("http://h/fast", &[("a", "1"), ("b", "2")]),
            "http://h/fast?a=1&b=2"
        );
    }

    #[test]
    fn append_query_appends_with_ampersand_when_base_has_query() {
        assert_eq!(
            append_query("http://h/fast?x=9", &[("a", "1")]),
            "http://h/fast?x=9&a=1"
        );
    }

    #[test]
    fn append_query_skips_empty_values() {
        assert_eq!(
            append_query("http://h/fast", &[("a", ""), ("b", "2")]),
            "http://h/fast?b=2"
        );
    }

    #[test]
    fn helius_sender_url_adds_swqos_and_key() {
        assert_eq!(
            helius_sender_url("http://h/fast", true, Some("K")),
            "http://h/fast?swqos_only=true&api-key=K"
        );
    }

    #[test]
    fn helius_sender_url_is_bare_when_no_options() {
        assert_eq!(
            helius_sender_url("http://h/fast", false, None),
            "http://h/fast"
        );
    }

    #[test]
    fn astralane_sender_url_adds_key() {
        assert_eq!(
            astralane_sender_url("http://g/iris", Some("K")),
            "http://g/iris?api-key=K"
        );
    }

    #[test]
    fn astralane_sender_url_is_bare_without_key() {
        assert_eq!(astralane_sender_url("http://g/iris", None), "http://g/iris");
    }

    fn ctx_for(sig: &str, tx_base64: &str) -> SendContext {
        SendContext {
            test_id: "test".to_string(),
            iteration: 0,
            signature: sig.to_string(),
            tx_base64: tx_base64.to_string(),
            timeout: Duration::from_secs(2),
            route_selection: None,
            leader_client_family: None,
        }
    }

    #[tokio::test]
    async fn helius_sender_reports_accepted_signature() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/fast"))
            .and(body_json(json!({
                "jsonrpc": "2.0",
                "id": 0,
                "method": "sendTransaction",
                "params": ["QUJD", {"encoding": "base64", "skipPreflight": true, "maxRetries": 0}]
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "jsonrpc": "2.0", "id": 0, "result": "SIG123"
            })))
            .mount(&server)
            .await;

        let adapter = build_adapter(
            "helius".to_string(),
            ProviderConfig::HeliusSender {
                endpoint: format!("{}/fast", server.uri()),
                api_key_env: None,
                swqos_only: false,
                headers: HashMap::new(),
            },
            Duration::from_secs(2),
        )
        .unwrap();

        let ack = adapter
            .send_transaction(b"ABC", &ctx_for("LOCAL", "QUJD"))
            .await;
        assert!(ack.accepted);
        assert_eq!(ack.returned_signature.as_deref(), Some("SIG123"));
        assert_eq!(ack.status_code, Some(200));
        assert!(ack.error.is_none());
        assert_eq!(ack.provider_kind, ProviderKind::HeliusSender);
    }

    #[tokio::test]
    async fn helius_sender_surfaces_provider_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "jsonrpc": "2.0", "id": 0, "error": {"code": -32005, "message": "tip too low"}
            })))
            .mount(&server)
            .await;

        let adapter = build_adapter(
            "helius".to_string(),
            ProviderConfig::HeliusSender {
                endpoint: server.uri(),
                api_key_env: None,
                swqos_only: false,
                headers: HashMap::new(),
            },
            Duration::from_secs(2),
        )
        .unwrap();

        let ack = adapter
            .send_transaction(b"ABC", &ctx_for("LOCAL", "QUJD"))
            .await;
        assert!(!ack.accepted);
        assert_eq!(ack.error_class.as_deref(), Some("provider_error"));
        assert!(ack.error.unwrap().contains("tip too low"));
    }

    #[tokio::test]
    async fn helius_sender_flags_malformed_response() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
            .mount(&server)
            .await;

        let adapter = build_adapter(
            "helius".to_string(),
            ProviderConfig::HeliusSender {
                endpoint: server.uri(),
                api_key_env: None,
                swqos_only: false,
                headers: HashMap::new(),
            },
            Duration::from_secs(2),
        )
        .unwrap();

        let ack = adapter
            .send_transaction(b"ABC", &ctx_for("LOCAL", "QUJD"))
            .await;
        assert!(!ack.accepted);
        assert_eq!(ack.error_class.as_deref(), Some("decode_error"));
    }

    #[tokio::test]
    async fn helius_sender_reports_transport_error() {
        let adapter = build_adapter(
            "helius".to_string(),
            ProviderConfig::HeliusSender {
                endpoint: "http://127.0.0.1:1/fast".to_string(),
                api_key_env: None,
                swqos_only: false,
                headers: HashMap::new(),
            },
            Duration::from_millis(500),
        )
        .unwrap();

        let ack = adapter
            .send_transaction(b"ABC", &ctx_for("LOCAL", "QUJD"))
            .await;
        assert!(!ack.accepted);
        assert_eq!(ack.error_class.as_deref(), Some("transport_error"));
    }

    #[test]
    fn helius_sender_config_uses_default_endpoint() {
        let cfg: ProviderConfig = serde_yaml::from_str("kind: helius_sender\n").unwrap();
        assert_eq!(cfg.kind(), ProviderKind::HeliusSender);
        match cfg {
            ProviderConfig::HeliusSender {
                endpoint,
                swqos_only,
                api_key_env,
                ..
            } => {
                assert_eq!(endpoint, "https://sender.helius-rpc.com/fast");
                assert!(!swqos_only);
                assert!(api_key_env.is_none());
            }
            _ => panic!("wrong variant"),
        }
    }

    #[tokio::test]
    async fn astralane_sender_sends_two_element_params_by_default() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/iris"))
            .and(body_json(json!({
                "jsonrpc": "2.0",
                "id": 0,
                "method": "sendTransaction",
                "params": ["QUJD", {"encoding": "base64", "skipPreflight": true}]
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "jsonrpc": "2.0", "id": 0, "result": "SIGA"
            })))
            .mount(&server)
            .await;

        let adapter = build_adapter(
            "astralane".to_string(),
            ProviderConfig::AstralaneSender {
                endpoint: format!("{}/iris", server.uri()),
                api_key_env: None,
                mev_protect: false,
                headers: HashMap::new(),
            },
            Duration::from_secs(2),
        )
        .unwrap();

        let ack = adapter
            .send_transaction(b"ABC", &ctx_for("LOCAL", "QUJD"))
            .await;
        assert!(ack.accepted);
        assert_eq!(ack.returned_signature.as_deref(), Some("SIGA"));
        assert_eq!(ack.provider_kind, ProviderKind::AstralaneSender);
    }

    #[tokio::test]
    async fn astralane_sender_appends_mev_protect_object() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(body_json(json!({
                "jsonrpc": "2.0",
                "id": 0,
                "method": "sendTransaction",
                "params": ["QUJD", {"encoding": "base64", "skipPreflight": true}, {"mevProtect": true}]
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "jsonrpc": "2.0", "id": 0, "result": "SIGA"
            })))
            .mount(&server)
            .await;

        let adapter = build_adapter(
            "astralane".to_string(),
            ProviderConfig::AstralaneSender {
                endpoint: server.uri(),
                api_key_env: None,
                mev_protect: true,
                headers: HashMap::new(),
            },
            Duration::from_secs(2),
        )
        .unwrap();

        let ack = adapter
            .send_transaction(b"ABC", &ctx_for("LOCAL", "QUJD"))
            .await;
        assert!(ack.accepted);
        assert_eq!(ack.returned_signature.as_deref(), Some("SIGA"));
    }

    #[test]
    fn astralane_sender_config_round_trips() {
        let cfg: ProviderConfig = serde_yaml::from_str(
            "kind: astralane_sender\nendpoint: https://fra.gateway.astralane.io/iris\nmev_protect: true\n",
        )
        .unwrap();
        assert_eq!(cfg.kind(), ProviderKind::AstralaneSender);
        match cfg {
            ProviderConfig::AstralaneSender {
                endpoint,
                mev_protect,
                ..
            } => {
                assert_eq!(endpoint, "https://fra.gateway.astralane.io/iris");
                assert!(mev_protect);
            }
            _ => panic!("wrong variant"),
        }
    }
}
