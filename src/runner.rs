use crate::{
    adapters::{build_adapter, SendContext},
    artifacts::{
        summarize, ArtifactWriter, BenchManifest, BenchSample, BenchSampleContext, BenchSummary,
        ManifestProvider,
    },
    config::BenchConfig,
    tx::{build_transactions, load_keypair, TxBuildConfig},
    url::redact_endpoint,
};
use anyhow::{bail, Context, Result};
use chrono::{SecondsFormat, Utc};
use futures::future::join_all;
use solana_sdk::signer::Signer;
use std::time::Duration;

#[derive(Debug)]
pub struct BenchRunOutput {
    pub test_id: String,
    pub run_dir: std::path::PathBuf,
    pub summary: BenchSummary,
}

pub async fn run_benchmark(config: BenchConfig) -> Result<BenchRunOutput> {
    let test_id = config.test_id.clone().unwrap_or_else(default_test_id);
    let max_spend_lamports = config
        .max_spend_lamports
        .context("max_spend_lamports is required for every transaction-producing run")?;
    if max_spend_lamports == 0 {
        bail!("max_spend_lamports must be greater than zero");
    }
    if config.providers.is_empty() {
        bail!("at least one provider is required");
    }
    let tx_count = resolve_tx_count(&config)?;
    if tx_count == 0 {
        bail!("benchmark count resolved to zero");
    }
    let keypair_path = config
        .keypair_path
        .as_ref()
        .context("keypair_path is required for transaction generation")?;
    let payer = load_keypair(keypair_path)?;
    let memo_prefix = config
        .memo_prefix
        .clone()
        .unwrap_or_else(|| test_id.clone());
    let tx_config = TxBuildConfig {
        rpc_url: config.rpc_url.clone(),
        lamports: config.lamports,
        compute_unit_limit: config.compute_unit_limit,
        compute_unit_price_microlamports: config.compute_unit_price_microlamports,
    };
    let txs = build_transactions(&tx_config, &payer, tx_count, Some(max_spend_lamports))?;
    let timeout = Duration::from_millis(config.timeout_ms);
    let mut providers = config
        .providers
        .iter()
        .map(|spec| build_adapter(spec.name.clone(), spec.config.clone(), timeout))
        .collect::<Result<Vec<_>>>()?;
    for provider in providers.iter_mut() {
        provider.warmup().await?;
    }

    let manifest = BenchManifest {
        schema_version: 2,
        test_id: test_id.clone(),
        generated_at: Utc::now(),
        rpc_url_label: redact_endpoint(&config.rpc_url),
        keypair_pubkey: payer.pubkey().to_string(),
        keypair_pubkeys: vec![payer.pubkey().to_string()],
        count: tx_count,
        duration_seconds: config.duration_seconds,
        rate_per_second: config.rate_per_second,
        lamports: config.lamports,
        compute_unit_limit: config.compute_unit_limit,
        compute_unit_price_microlamports: config.compute_unit_price_microlamports,
        memo_prefix,
        max_spend_lamports: config.max_spend_lamports,
        route_strategy: None,
        leader_network_path_class: None,
        client_aware_harmonic_cu_price_microlamports: None,
        resend_attempts: 1,
        resend_interval_ms: 0,
        providers: config
            .providers
            .iter()
            .map(|spec| ManifestProvider {
                name: spec.name.clone(),
                kind: spec.config.kind(),
                route_mode: spec
                    .config
                    .configured_route_mode()
                    .map(|mode| mode.as_wire().to_string()),
                routes: spec.config.configured_routes(),
            })
            .collect(),
    };

    let mut writer = ArtifactWriter::create(&config.artifact_dir, &test_id)?;
    writer.write_manifest(&manifest)?;
    let mut samples = Vec::with_capacity(txs.len() * providers.len());
    let mut interval = config
        .rate_per_second
        .filter(|rate| *rate > 0.0)
        .map(|rate| tokio::time::interval(Duration::from_secs_f64(1.0 / rate)));

    for tx in txs {
        if let Some(interval) = interval.as_mut() {
            interval.tick().await;
        }
        let client_started_at = Utc::now();
        let started = std::time::Instant::now();
        let ctx = SendContext {
            test_id: test_id.clone(),
            iteration: tx.iteration,
            signature: tx.signature.to_string(),
            tx_base64: tx.base64.clone(),
            timeout,
            route_selection: None,
            leader_client_family: None,
        };
        let futures = providers
            .iter()
            .map(|provider| provider.send_transaction(&tx.raw, &ctx));
        let acks = join_all(futures).await;
        let client_finished_at = Utc::now();
        let client_ack_latency_us = started.elapsed().as_micros();
        for ack in acks {
            let sample = BenchSample::from_ack(
                BenchSampleContext {
                    test_id: test_id.clone(),
                    iteration: tx.iteration,
                    signature: tx.signature.to_string(),
                    comparison_group_id: None,
                    policy_arm: None,
                    policy_arm_index: None,
                    client_started_at,
                    client_finished_at,
                    client_ack_latency_us,
                    route_selection: None,
                    leader_client_family: None,
                    leader_software_client: None,
                    leader_software_client_id: None,
                    compute_unit_limit: config.compute_unit_limit,
                    compute_unit_price_microlamports: config.compute_unit_price_microlamports,
                    base_fee_lamports: tx.base_fee_lamports,
                    priority_fee_lamports: tx.priority_fee_lamports,
                    transaction_fee_lamports: tx.transaction_fee_lamports,
                    resend_attempt_index: None,
                    resend_attempts: None,
                    resend_interval_ms: None,
                },
                ack,
            );
            writer.write_sample(&sample)?;
            samples.push(sample);
        }
    }

    let summary = summarize(&test_id, &samples);
    let run_dir = writer.finish(&summary)?;
    Ok(BenchRunOutput {
        test_id,
        run_dir,
        summary,
    })
}

fn resolve_tx_count(config: &BenchConfig) -> Result<usize> {
    match (
        config.count,
        config.duration_seconds,
        config.rate_per_second,
    ) {
        (Some(count), None, _) => Ok(count),
        (None, Some(duration), Some(rate)) if rate > 0.0 => {
            Ok((duration as f64 * rate).ceil() as usize)
        }
        (Some(_), Some(_), Some(_)) => {
            bail!("count is mutually exclusive with duration_seconds + rate_per_second")
        }
        (None, Some(_), Some(_)) => bail!("rate_per_second must be greater than zero"),
        (Some(_), Some(_), None) => bail!("duration_seconds is mutually exclusive with count"),
        (None, Some(_), None) => bail!("duration_seconds requires rate_per_second"),
        (None, None, _) => Ok(1),
    }
}

fn default_test_id() -> String {
    format!(
        "solana-tx-bench-{}",
        Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
    )
    .replace(':', "")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{adapters::ProviderConfig, config::ProviderSpec};

    #[test]
    fn duration_and_rate_resolve_count() {
        let config = BenchConfig {
            count: None,
            duration_seconds: Some(10),
            rate_per_second: Some(2.5),
            providers: vec![ProviderSpec {
                name: "noop".to_string(),
                config: ProviderConfig::SolanaRpc {
                    endpoint: "http://127.0.0.1:8899".to_string(),
                    headers: Default::default(),
                },
            }],
            ..Default::default()
        };
        assert_eq!(resolve_tx_count(&config).expect("count"), 25);
    }

    #[test]
    fn redacts_api_key_query() {
        let redacted = redact_endpoint("https://example.com/path?api-key=secret");
        assert_eq!(redacted, "https://example.com/");
    }
}
