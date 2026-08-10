use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct BenchConfig {
    pub test_id: Option<String>,
    pub keypair_path: Option<PathBuf>,
    pub paired_payer_keypair_paths: Vec<PathBuf>,
    pub rpc_url: String,
    pub artifact_dir: PathBuf,
    pub providers: Vec<ProviderSpec>,
    pub count: Option<usize>,
    pub duration_seconds: Option<u64>,
    pub rate_per_second: Option<f64>,
    pub lamports: u64,
    pub compute_unit_limit: u32,
    pub compute_unit_price_microlamports: u64,
    pub memo_prefix: Option<String>,
    pub max_spend_lamports: Option<u64>,
    pub timeout_ms: u64,
}

impl Default for BenchConfig {
    fn default() -> Self {
        Self {
            test_id: None,
            keypair_path: None,
            paired_payer_keypair_paths: Vec::new(),
            rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
            artifact_dir: PathBuf::from("artifacts"),
            providers: Vec::new(),
            count: Some(1),
            duration_seconds: None,
            rate_per_second: None,
            lamports: 500,
            compute_unit_limit: 660,
            compute_unit_price_microlamports: 0,
            memo_prefix: None,
            max_spend_lamports: None,
            timeout_ms: 2_000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frankfurt_doublezero_public_example_has_two_isolated_arms_and_payers() {
        let config: BenchConfig = serde_yaml::from_str(include_str!(
            "../examples/rpcedge-doublezero-public-frankfurt.yaml"
        ))
        .expect("parse DoubleZero/public example");

        assert_eq!(config.providers.len(), 2);
        assert_eq!(config.providers[0].name, "public_internet");
        assert_eq!(config.providers[1].name, "doublezero");
        assert_eq!(config.paired_payer_keypair_paths.len(), 2);
        assert!(config
            .providers
            .iter()
            .all(|provider| provider.config.supports_route_selection()));
    }

    #[test]
    fn frankfurt_http_quic_example_keeps_transport_arms_equivalent() {
        let config: BenchConfig =
            serde_yaml::from_str(include_str!("../examples/rpcedge-http-quic-frankfurt.yaml"))
                .expect("parse HTTP/QUIC example");

        assert_eq!(config.providers.len(), 2);
        assert_eq!(config.paired_payer_keypair_paths.len(), 2);
        assert_eq!(
            config.providers[0].config.configured_routes(),
            config.providers[1].config.configured_routes()
        );
        assert_eq!(
            config.providers[0].config.configured_route_mode(),
            config.providers[1].config.configured_route_mode()
        );
        assert!(config
            .providers
            .iter()
            .all(|provider| provider.config.supports_route_selection()));
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProviderSpec {
    pub name: String,
    #[serde(flatten)]
    pub config: crate::adapters::ProviderConfig,
}
