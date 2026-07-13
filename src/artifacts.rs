use crate::adapters::{ProviderAck, ProviderKind, RouteResultAck, RouteSelection};
use anyhow::{bail, Context};
use chrono::{DateTime, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchManifest {
    pub schema_version: u32,
    pub test_id: String,
    pub generated_at: DateTime<Utc>,
    pub rpc_url_label: String,
    pub keypair_pubkey: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keypair_pubkeys: Vec<String>,
    pub count: usize,
    pub duration_seconds: Option<u64>,
    pub rate_per_second: Option<f64>,
    pub lamports: u64,
    pub compute_unit_limit: u32,
    pub compute_unit_price_microlamports: u64,
    pub memo_prefix: String,
    pub max_spend_lamports: Option<u64>,
    pub route_strategy: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub leader_network_path_class: Option<String>,
    pub client_aware_harmonic_cu_price_microlamports: Option<u64>,
    #[serde(default = "default_resend_attempts")]
    pub resend_attempts: usize,
    #[serde(default)]
    pub resend_interval_ms: u64,
    pub providers: Vec<ManifestProvider>,
}

fn default_resend_attempts() -> usize {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestProvider {
    pub name: String,
    pub kind: ProviderKind,
    #[serde(default)]
    pub route_mode: Option<String>,
    #[serde(default)]
    pub routes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchSample {
    pub schema_version: u32,
    pub test_id: String,
    pub iteration: usize,
    pub signature: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comparison_group_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy_arm: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy_arm_index: Option<u16>,
    pub provider_name: String,
    pub provider_kind: ProviderKind,
    pub accepted: bool,
    pub client_started_at: DateTime<Utc>,
    pub client_finished_at: DateTime<Utc>,
    pub client_ack_latency_us: u128,
    pub provider_send_started_at: DateTime<Utc>,
    pub provider_send_finished_at: DateTime<Utc>,
    pub provider_ack_latency_us: u128,
    pub provider_request_id: Option<String>,
    pub returned_signature: Option<String>,
    pub status_code: Option<u16>,
    pub error_class: Option<String>,
    pub error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resend_attempt_index: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resend_attempts: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resend_interval_ms: Option<u64>,
    pub route_policy: Option<String>,
    pub route_mode: Option<String>,
    pub selected_routes: Vec<String>,
    pub leader_client_family: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub leader_software_client: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub leader_software_client_id: Option<u16>,
    pub compute_unit_limit: u32,
    pub compute_unit_price_microlamports: u64,
    pub base_fee_lamports: u64,
    pub priority_fee_lamports: u64,
    pub transaction_fee_lamports: u64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub route_results: Vec<RouteResultAck>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub route_results_complete: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct BenchSampleContext {
    pub test_id: String,
    pub iteration: usize,
    pub signature: String,
    pub comparison_group_id: Option<String>,
    pub policy_arm: Option<String>,
    pub policy_arm_index: Option<u16>,
    pub client_started_at: DateTime<Utc>,
    pub client_finished_at: DateTime<Utc>,
    pub client_ack_latency_us: u128,
    pub route_selection: Option<RouteSelection>,
    pub leader_client_family: Option<String>,
    pub leader_software_client: Option<String>,
    pub leader_software_client_id: Option<u16>,
    pub compute_unit_limit: u32,
    pub compute_unit_price_microlamports: u64,
    pub base_fee_lamports: u64,
    pub priority_fee_lamports: u64,
    pub transaction_fee_lamports: u64,
    pub resend_attempt_index: Option<u16>,
    pub resend_attempts: Option<u16>,
    pub resend_interval_ms: Option<u64>,
}

impl BenchSample {
    pub fn from_ack(context: BenchSampleContext, ack: ProviderAck) -> Self {
        let route_selection = context.route_selection.as_ref();
        Self {
            schema_version: 5,
            test_id: context.test_id,
            iteration: context.iteration,
            signature: context.signature,
            comparison_group_id: context.comparison_group_id,
            policy_arm: context.policy_arm,
            policy_arm_index: context.policy_arm_index,
            provider_name: ack.provider_name,
            provider_kind: ack.provider_kind,
            accepted: ack.accepted,
            client_started_at: context.client_started_at,
            client_finished_at: context.client_finished_at,
            client_ack_latency_us: context.client_ack_latency_us,
            provider_send_started_at: ack.send_started_at,
            provider_send_finished_at: ack.send_finished_at,
            provider_ack_latency_us: ack.ack_latency_us,
            provider_request_id: ack.provider_request_id,
            returned_signature: ack.returned_signature,
            status_code: ack.status_code,
            error_class: ack.error_class,
            error: ack.error,
            resend_attempt_index: context.resend_attempt_index,
            resend_attempts: context.resend_attempts,
            resend_interval_ms: context.resend_interval_ms,
            route_policy: route_selection.map(|selection| selection.policy.clone()),
            route_mode: route_selection.map(|selection| selection.mode.as_wire().to_string()),
            selected_routes: route_selection
                .map(|selection| selection.routes.clone())
                .unwrap_or_default(),
            leader_client_family: context.leader_client_family,
            leader_software_client: context.leader_software_client,
            leader_software_client_id: context.leader_software_client_id,
            compute_unit_limit: context.compute_unit_limit,
            compute_unit_price_microlamports: context.compute_unit_price_microlamports,
            base_fee_lamports: context.base_fee_lamports,
            priority_fee_lamports: context.priority_fee_lamports,
            transaction_fee_lamports: context.transaction_fee_lamports,
            route_results: ack.route_results,
            route_results_complete: ack.route_results_complete,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchSummary {
    pub schema_version: u32,
    pub test_id: String,
    pub generated_at: DateTime<Utc>,
    pub total_samples: usize,
    pub provider_summaries: Vec<ProviderSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderSummary {
    pub provider_name: String,
    pub provider_kind: ProviderKind,
    pub count: usize,
    pub accepted: usize,
    pub errors: usize,
    pub min_us: Option<u128>,
    pub p50_us: Option<u128>,
    pub p75_us: Option<u128>,
    pub p90_us: Option<u128>,
    pub p95_us: Option<u128>,
    pub p99_us: Option<u128>,
    pub max_us: Option<u128>,
}

pub struct ArtifactWriter {
    pub run_dir: PathBuf,
    samples: BufWriter<File>,
}

impl ArtifactWriter {
    pub fn create(root: &Path, test_id: &str) -> anyhow::Result<Self> {
        validate_test_id(test_id)?;
        let run_dir = root.join(test_id);
        if run_dir.exists()
            && fs::read_dir(&run_dir)
                .with_context(|| format!("read {}", run_dir.display()))?
                .next()
                .is_some()
        {
            bail!(
                "artifact directory already exists and is not empty: {}",
                run_dir.display()
            );
        }
        fs::create_dir_all(&run_dir).with_context(|| format!("create {}", run_dir.display()))?;
        let samples = BufWriter::new(
            File::create(run_dir.join("samples.ndjson"))
                .with_context(|| format!("create {}/samples.ndjson", run_dir.display()))?,
        );
        Ok(Self { run_dir, samples })
    }

    pub fn write_manifest(&self, manifest: &BenchManifest) -> anyhow::Result<()> {
        let bytes = serde_json::to_vec_pretty(manifest)?;
        fs::write(self.run_dir.join("manifest.json"), bytes)?;
        Ok(())
    }

    pub fn write_sample(&mut self, sample: &BenchSample) -> anyhow::Result<()> {
        serde_json::to_writer(&mut self.samples, sample)?;
        self.samples.write_all(b"\n")?;
        Ok(())
    }

    pub fn finish(mut self, summary: &BenchSummary) -> anyhow::Result<PathBuf> {
        self.samples.flush()?;
        fs::write(
            self.run_dir.join("summary.json"),
            serde_json::to_vec_pretty(summary)?,
        )?;
        fs::write(self.run_dir.join("summary.md"), summary_markdown(summary))?;
        Ok(self.run_dir)
    }
}

fn validate_test_id(test_id: &str) -> anyhow::Result<()> {
    let path = Path::new(test_id);
    let is_single_normal_component = matches!(
        path.components().collect::<Vec<_>>().as_slice(),
        [std::path::Component::Normal(_)]
    );
    if test_id.trim().is_empty() || !is_single_normal_component {
        bail!("test_id must be one non-empty path component");
    }
    Ok(())
}

pub fn summarize(test_id: &str, samples: &[BenchSample]) -> BenchSummary {
    let mut keys = samples
        .iter()
        .map(|sample| (sample.provider_name.clone(), sample.provider_kind))
        .collect::<Vec<_>>();
    keys.sort();
    keys.dedup();

    let provider_summaries = keys
        .into_iter()
        .map(|(name, kind)| {
            let mut latencies = samples
                .iter()
                .filter(|sample| sample.provider_name == name && sample.accepted)
                .map(|sample| sample.provider_ack_latency_us)
                .collect::<Vec<_>>();
            latencies.sort_unstable();
            let count = samples
                .iter()
                .filter(|sample| sample.provider_name == name)
                .count();
            let accepted = latencies.len();
            ProviderSummary {
                provider_name: name,
                provider_kind: kind,
                count,
                accepted,
                errors: count.saturating_sub(accepted),
                min_us: latencies.first().copied(),
                p50_us: percentile(&latencies, 0.50),
                p75_us: percentile(&latencies, 0.75),
                p90_us: percentile(&latencies, 0.90),
                p95_us: percentile(&latencies, 0.95),
                p99_us: percentile(&latencies, 0.99),
                max_us: latencies.last().copied(),
            }
        })
        .collect::<Vec<_>>();

    BenchSummary {
        schema_version: 1,
        test_id: test_id.to_string(),
        generated_at: Utc::now(),
        total_samples: samples.len(),
        provider_summaries,
    }
}

pub fn summary_markdown(summary: &BenchSummary) -> String {
    let mut out = String::new();
    out.push_str("# Solana Tx Bench Summary\n\n");
    out.push_str(&format!(
        "- Test ID: `{}`\n- Generated: `{}`\n- Samples: `{}`\n\n",
        summary.test_id,
        summary
            .generated_at
            .to_rfc3339_opts(SecondsFormat::Millis, true),
        summary.total_samples
    ));
    out.push_str(
        "| Provider | Kind | Count | Accepted | Errors | p50 us | p90 us | p99 us | Max us |\n",
    );
    out.push_str("| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |\n");
    for item in &summary.provider_summaries {
        out.push_str(&format!(
            "| `{}` | `{:?}` | {} | {} | {} | {} | {} | {} | {} |\n",
            item.provider_name,
            item.provider_kind,
            item.count,
            item.accepted,
            item.errors,
            fmt(item.p50_us),
            fmt(item.p90_us),
            fmt(item.p99_us),
            fmt(item.max_us)
        ));
    }
    out.push_str("\nProvider ACK is not landing or first-shred latency.\n");
    out
}

fn percentile(sorted: &[u128], p: f64) -> Option<u128> {
    if sorted.is_empty() {
        return None;
    }
    let rank = ((sorted.len() - 1) as f64 * p).ceil() as usize;
    sorted.get(rank).copied()
}

fn fmt(value: Option<u128>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "-".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn artifact_writer_rejects_path_like_test_ids() {
        let root = tempdir().expect("tempdir");
        for test_id in ["", ".", "..", "nested/run", "../outside"] {
            assert!(ArtifactWriter::create(root.path(), test_id).is_err());
        }
    }

    #[test]
    fn artifact_writer_refuses_to_reuse_nonempty_run_directory() {
        let root = tempdir().expect("tempdir");
        let run = root.path().join("existing-run");
        fs::create_dir(&run).expect("run dir");
        fs::write(run.join("leader-sends.ndjson"), "existing evidence").expect("existing evidence");

        assert!(ArtifactWriter::create(root.path(), "existing-run").is_err());
        assert_eq!(
            fs::read_to_string(run.join("leader-sends.ndjson")).expect("evidence"),
            "existing evidence"
        );
    }
}
