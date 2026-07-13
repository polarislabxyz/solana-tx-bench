use crate::{
    artifacts::{BenchManifest, BenchSample},
    cost::priority_fee_lamports,
    leader_paced::{LeaderRunDecisionEvent, LeaderSendEvent},
    observations::ObservationEvent,
};
use anyhow::{bail, Context, Result};
use chrono::{DateTime, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct CompareOptions {
    pub artifact_dirs: Vec<PathBuf>,
    pub labels: Vec<String>,
    pub output_dir: PathBuf,
    pub primary_source: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonOutput {
    pub schema_version: u32,
    pub title: String,
    pub generated_at: DateTime<Utc>,
    pub primary_source: String,
    pub run_count: usize,
    pub runs: Vec<RunComparisonSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paired_comparison: Option<PairedComparisonSummary>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LeaderCoverageSummary {
    pub observed_leader_runs: usize,
    pub eligible_leader_runs: usize,
    pub observed_scheduled_slots: u64,
    pub eligible_scheduled_slots: u64,
    pub eligible_leader_run_pct: f64,
    pub eligible_scheduled_slot_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairedComparisonSummary {
    pub baseline_arm: String,
    pub candidate_arm: String,
    pub delta_convention: String,
    pub coverage: LeaderCoverageSummary,
    pub sources: Vec<PairedSourceSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairedSourceSummary {
    pub source_name: String,
    pub comparison_groups: usize,
    pub both_observed: usize,
    pub baseline_only: usize,
    pub candidate_only: usize,
    pub neither_observed: usize,
    pub candidate_wins: usize,
    pub ties: usize,
    pub baseline_wins: usize,
    pub delta_p10_ms: Option<f64>,
    pub delta_p50_ms: Option<f64>,
    pub delta_p90_ms: Option<f64>,
    pub landed_slot_delta_p50: Option<i64>,
    pub slot_index_delta_p50: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairedComparisonRow {
    pub source_name: String,
    pub comparison_group_id: String,
    pub leader_identity: String,
    pub leader_region: Option<String>,
    pub leader_city: Option<String>,
    pub leader_network_path_class: Option<String>,
    pub leader_network_path_rtt_us: Option<u64>,
    pub baseline_arm: String,
    pub candidate_arm: String,
    pub baseline_signature: String,
    pub candidate_signature: String,
    pub status: String,
    pub baseline_latency_ms: Option<f64>,
    pub candidate_latency_ms: Option<f64>,
    pub delta_ms: Option<f64>,
    pub baseline_landed_slot: Option<u64>,
    pub candidate_landed_slot: Option<u64>,
    pub landed_slot_delta: Option<i64>,
    pub baseline_slot_index: Option<u64>,
    pub candidate_slot_index: Option<u64>,
    pub slot_index_delta: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunComparisonSummary {
    pub label: String,
    pub test_id: String,
    pub artifact_dir: String,
    pub route_policy: String,
    pub routes: String,
    pub total_runs: usize,
    pub provider_samples: usize,
    pub accepted: usize,
    pub errors: usize,
    pub avg_ack_ms: Option<f64>,
    pub p90_ack_ms: Option<f64>,
    pub avg_deshred_ms: Option<f64>,
    pub p90_deshred_ms: Option<f64>,
    pub avg_landed_ms: Option<f64>,
    pub p90_landed_ms: Option<f64>,
    pub avg_block_ms: Option<f64>,
    pub p90_block_ms: Option<f64>,
    pub avg_landed_slots: Option<f64>,
    pub p90_landed_slots: Option<f64>,
    pub avg_idx_in_block: Option<f64>,
    pub p90_idx_in_block: Option<f64>,
    pub avg_base_fee_lamports: Option<f64>,
    pub avg_priority_fee_lamports: Option<f64>,
    pub avg_transaction_fee_lamports: Option<f64>,
    pub avg_route_tip_generated_lamports: Option<f64>,
    pub avg_route_tip_landed_lamports: Option<f64>,
    pub avg_total_generated_spend_lamports: Option<f64>,
    pub avg_total_landed_spend_lamports: Option<f64>,
    pub max_slots: Option<i64>,
    pub min_slots: Option<i64>,
    pub same_slot_landed: usize,
    pub landed_runs: usize,
    pub block_seen: usize,
    pub success_ratio_pct: f64,
    pub performance_rate_pct: f64,
    pub score: ScoreBreakdown,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScoreBreakdown {
    pub landed_ms: f64,
    pub landed_slots: f64,
    pub landed_idx: f64,
    pub same_slot: f64,
    pub success_ratio: f64,
}

#[derive(Debug, Clone, Default)]
struct SourceStats {
    count: usize,
    latencies_ms: Vec<f64>,
    slot_deltas: Vec<i64>,
    slot_indexes: Vec<u64>,
}

pub fn generate_comparison(options: CompareOptions) -> Result<ComparisonOutput> {
    if options.artifact_dirs.is_empty() {
        bail!("compare requires at least one --artifact-dir value");
    }
    if !options.labels.is_empty() && options.labels.len() != options.artifact_dirs.len() {
        bail!("--label count must match --artifact-dir count");
    }

    fs::create_dir_all(&options.output_dir)
        .with_context(|| format!("create {}", options.output_dir.display()))?;

    let mut runs = Vec::with_capacity(options.artifact_dirs.len());
    for (index, artifact_dir) in options.artifact_dirs.iter().enumerate() {
        let label = options.labels.get(index).cloned();
        runs.extend(load_runs(artifact_dir, label, &options.primary_source)?);
    }
    if runs.len() < 2 {
        bail!("compare needs at least two runs or one paired artifact with multiple policy arms");
    }

    apply_scores(&mut runs);

    let (paired_comparison, paired_rows) = if options.artifact_dirs.len() == 1 {
        build_paired_comparison(&options.artifact_dirs[0])?
            .map(|(summary, rows)| (Some(summary), rows))
            .unwrap_or_else(|| (None, Vec::new()))
    } else {
        (None, Vec::new())
    };
    if paired_comparison.is_some() {
        write_paired_rows(&options.output_dir, &paired_rows)?;
    }

    let output = ComparisonOutput {
        schema_version: 1,
        title: options.title,
        generated_at: Utc::now(),
        primary_source: options.primary_source,
        run_count: runs.len(),
        paired_comparison,
        warnings: comparison_warnings(&runs),
        runs,
    };

    fs::write(
        options.output_dir.join("comparison.json"),
        serde_json::to_vec_pretty(&output)?,
    )
    .with_context(|| {
        format!(
            "write {}",
            options.output_dir.join("comparison.json").display()
        )
    })?;
    fs::write(
        options.output_dir.join("comparison.md"),
        comparison_markdown(&output),
    )
    .with_context(|| {
        format!(
            "write {}",
            options.output_dir.join("comparison.md").display()
        )
    })?;
    let html = comparison_html(&output);
    fs::write(options.output_dir.join("comparison.html"), &html).with_context(|| {
        format!(
            "write {}",
            options.output_dir.join("comparison.html").display()
        )
    })?;
    fs::write(options.output_dir.join("index.html"), html)
        .with_context(|| format!("write {}", options.output_dir.join("index.html").display()))?;

    Ok(output)
}

fn build_paired_comparison(
    artifact_dir: &Path,
) -> Result<Option<(PairedComparisonSummary, Vec<PairedComparisonRow>)>> {
    let sends: Vec<LeaderSendEvent> = read_ndjson(&artifact_dir.join("leader-sends.ndjson"))?;
    let observations: Vec<ObservationEvent> =
        read_ndjson(&artifact_dir.join("matched-observations.ndjson"))?;
    let decisions: Vec<LeaderRunDecisionEvent> =
        read_ndjson(&artifact_dir.join("leader-runs.ndjson"))?;

    let arms_by_index = sends
        .iter()
        .filter_map(|send| Some((send.policy_arm_index?, send.policy_arm.clone()?)))
        .collect::<BTreeMap<_, _>>();
    if arms_by_index.len() != 2 {
        return Ok(None);
    }
    let mut arms = arms_by_index.into_iter();
    let (_, baseline_arm) = arms.next().expect("checked two arms");
    let (_, candidate_arm) = arms.next().expect("checked two arms");

    let mut groups = BTreeMap::<String, BTreeMap<String, &LeaderSendEvent>>::new();
    for send in &sends {
        let (Some(group_id), Some(arm)) = (&send.comparison_group_id, &send.policy_arm) else {
            continue;
        };
        groups
            .entry(group_id.clone())
            .or_default()
            .insert(arm.clone(), send);
    }
    groups
        .retain(|_, sends| sends.contains_key(&baseline_arm) && sends.contains_key(&candidate_arm));
    if groups.is_empty() {
        return Ok(None);
    }

    let mut observations_by_source_signature =
        BTreeMap::<(String, String), &ObservationEvent>::new();
    for event in observations
        .iter()
        .filter(|event| !is_route_tip_observation(event))
    {
        let key = (event.source_name.clone(), event.signature.clone());
        observations_by_source_signature
            .entry(key)
            .and_modify(|current| {
                if event.observed_at < current.observed_at {
                    *current = event;
                }
            })
            .or_insert(event);
    }

    let mut rows = Vec::new();
    let mut source_summaries = Vec::new();
    for source_name in ["rpcedge_deshred", "rpcedge_processed"] {
        let mut source_rows = Vec::new();
        for (group_id, arm_sends) in &groups {
            let baseline_send = arm_sends
                .get(&baseline_arm)
                .expect("group retained with baseline");
            let candidate_send = arm_sends
                .get(&candidate_arm)
                .expect("group retained with candidate");
            let baseline_observation = observations_by_source_signature
                .get(&(source_name.to_string(), baseline_send.signature.clone()))
                .copied();
            let candidate_observation = observations_by_source_signature
                .get(&(source_name.to_string(), candidate_send.signature.clone()))
                .copied();
            let baseline_latency_ms =
                baseline_observation.and_then(|event| observation_latency_ms(event, baseline_send));
            let candidate_latency_ms = candidate_observation
                .and_then(|event| observation_latency_ms(event, candidate_send));
            let delta_ms = baseline_latency_ms
                .zip(candidate_latency_ms)
                .map(|(baseline, candidate)| candidate - baseline);
            let baseline_landed_slot = baseline_observation.and_then(|event| event.slot);
            let candidate_landed_slot = candidate_observation.and_then(|event| event.slot);
            let baseline_slot_index = baseline_observation.and_then(|event| event.slot_index);
            let candidate_slot_index = candidate_observation.and_then(|event| event.slot_index);
            let status = match (
                baseline_observation.is_some(),
                candidate_observation.is_some(),
            ) {
                (true, true) => "both_observed",
                (true, false) => "baseline_only",
                (false, true) => "candidate_only",
                (false, false) => "neither_observed",
            };
            source_rows.push(PairedComparisonRow {
                source_name: source_name.to_string(),
                comparison_group_id: group_id.clone(),
                leader_identity: baseline_send.leader_identity.clone(),
                leader_region: baseline_send.leader_region.clone(),
                leader_city: baseline_send.leader_city.clone(),
                leader_network_path_class: baseline_send
                    .leader_network_path_class
                    .map(|class| class.as_str().to_string()),
                leader_network_path_rtt_us: baseline_send.leader_network_path_rtt_us,
                baseline_arm: baseline_arm.clone(),
                candidate_arm: candidate_arm.clone(),
                baseline_signature: baseline_send.signature.clone(),
                candidate_signature: candidate_send.signature.clone(),
                status: status.to_string(),
                baseline_latency_ms,
                candidate_latency_ms,
                delta_ms,
                baseline_landed_slot,
                candidate_landed_slot,
                landed_slot_delta: baseline_landed_slot
                    .zip(candidate_landed_slot)
                    .map(|(baseline, candidate)| candidate as i64 - baseline as i64),
                baseline_slot_index,
                candidate_slot_index,
                slot_index_delta: baseline_slot_index
                    .zip(candidate_slot_index)
                    .map(|(baseline, candidate)| candidate as i64 - baseline as i64),
            });
        }
        source_summaries.push(summarize_paired_source(source_name, &source_rows));
        rows.extend(source_rows);
    }

    Ok(Some((
        PairedComparisonSummary {
            baseline_arm,
            candidate_arm,
            delta_convention: "candidate_minus_baseline".to_string(),
            coverage: summarize_leader_coverage(&decisions),
            sources: source_summaries,
        },
        rows,
    )))
}

fn observation_latency_ms(event: &ObservationEvent, send: &LeaderSendEvent) -> Option<f64> {
    let submitted_at = event.submitted_at.as_ref().unwrap_or(&send.sent_at);
    event
        .observed_at
        .signed_duration_since(submitted_at.to_owned())
        .num_microseconds()
        .map(|value| value as f64 / 1_000.0)
}

fn summarize_paired_source(source_name: &str, rows: &[PairedComparisonRow]) -> PairedSourceSummary {
    let mut deltas = rows
        .iter()
        .filter_map(|row| row.delta_ms)
        .collect::<Vec<_>>();
    let mut landed_slot_deltas = rows
        .iter()
        .filter_map(|row| row.landed_slot_delta)
        .collect::<Vec<_>>();
    let mut slot_index_deltas = rows
        .iter()
        .filter_map(|row| row.slot_index_delta)
        .collect::<Vec<_>>();
    deltas.sort_by(f64::total_cmp);
    landed_slot_deltas.sort_unstable();
    slot_index_deltas.sort_unstable();
    PairedSourceSummary {
        source_name: source_name.to_string(),
        comparison_groups: rows.len(),
        both_observed: rows
            .iter()
            .filter(|row| row.status == "both_observed")
            .count(),
        baseline_only: rows
            .iter()
            .filter(|row| row.status == "baseline_only")
            .count(),
        candidate_only: rows
            .iter()
            .filter(|row| row.status == "candidate_only")
            .count(),
        neither_observed: rows
            .iter()
            .filter(|row| row.status == "neither_observed")
            .count(),
        candidate_wins: deltas.iter().filter(|delta| **delta < 0.0).count(),
        ties: deltas.iter().filter(|delta| **delta == 0.0).count(),
        baseline_wins: deltas.iter().filter(|delta| **delta > 0.0).count(),
        delta_p10_ms: percentile_f64(&deltas, 0.10),
        delta_p50_ms: percentile_f64(&deltas, 0.50),
        delta_p90_ms: percentile_f64(&deltas, 0.90),
        landed_slot_delta_p50: percentile_i64_value(&landed_slot_deltas, 0.50),
        slot_index_delta_p50: percentile_i64_value(&slot_index_deltas, 0.50),
    }
}

fn summarize_leader_coverage(decisions: &[LeaderRunDecisionEvent]) -> LeaderCoverageSummary {
    let observed_scheduled_slots = decisions
        .iter()
        .map(|row| {
            row.leader_run_end_slot
                .saturating_sub(row.leader_run_start_slot)
                .saturating_add(1)
        })
        .sum::<u64>();
    let eligible_scheduled_slots = decisions
        .iter()
        .filter(|row| row.eligible)
        .map(|row| {
            row.leader_run_end_slot
                .saturating_sub(row.leader_run_start_slot)
                .saturating_add(1)
        })
        .sum::<u64>();
    let eligible_leader_runs = decisions.iter().filter(|row| row.eligible).count();
    LeaderCoverageSummary {
        observed_leader_runs: decisions.len(),
        eligible_leader_runs,
        observed_scheduled_slots,
        eligible_scheduled_slots,
        eligible_leader_run_pct: ratio(eligible_leader_runs, decisions.len()) * 100.0,
        eligible_scheduled_slot_pct: if observed_scheduled_slots == 0 {
            0.0
        } else {
            eligible_scheduled_slots as f64 / observed_scheduled_slots as f64 * 100.0
        },
    }
}

fn write_paired_rows(output_dir: &Path, rows: &[PairedComparisonRow]) -> Result<()> {
    fs::write(
        output_dir.join("paired-rows.json"),
        serde_json::to_vec_pretty(rows)?,
    )
    .with_context(|| format!("write {}", output_dir.join("paired-rows.json").display()))?;
    let mut csv = String::from("source_name,comparison_group_id,leader_identity,leader_region,leader_city,leader_network_path_class,leader_network_path_rtt_us,baseline_arm,candidate_arm,baseline_signature,candidate_signature,status,baseline_latency_ms,candidate_latency_ms,delta_ms,baseline_landed_slot,candidate_landed_slot,landed_slot_delta,baseline_slot_index,candidate_slot_index,slot_index_delta\n");
    for row in rows {
        let fields = [
            row.source_name.clone(),
            row.comparison_group_id.clone(),
            row.leader_identity.clone(),
            row.leader_region.clone().unwrap_or_default(),
            row.leader_city.clone().unwrap_or_default(),
            row.leader_network_path_class.clone().unwrap_or_default(),
            row.leader_network_path_rtt_us
                .map(|value| value.to_string())
                .unwrap_or_default(),
            row.baseline_arm.clone(),
            row.candidate_arm.clone(),
            row.baseline_signature.clone(),
            row.candidate_signature.clone(),
            row.status.clone(),
            row.baseline_latency_ms
                .map(|value| value.to_string())
                .unwrap_or_default(),
            row.candidate_latency_ms
                .map(|value| value.to_string())
                .unwrap_or_default(),
            row.delta_ms
                .map(|value| value.to_string())
                .unwrap_or_default(),
            row.baseline_landed_slot
                .map(|value| value.to_string())
                .unwrap_or_default(),
            row.candidate_landed_slot
                .map(|value| value.to_string())
                .unwrap_or_default(),
            row.landed_slot_delta
                .map(|value| value.to_string())
                .unwrap_or_default(),
            row.baseline_slot_index
                .map(|value| value.to_string())
                .unwrap_or_default(),
            row.candidate_slot_index
                .map(|value| value.to_string())
                .unwrap_or_default(),
            row.slot_index_delta
                .map(|value| value.to_string())
                .unwrap_or_default(),
        ];
        csv.push_str(
            &fields
                .iter()
                .map(|value| csv_escape(value))
                .collect::<Vec<_>>()
                .join(","),
        );
        csv.push('\n');
    }
    fs::write(output_dir.join("paired-rows.csv"), csv)
        .with_context(|| format!("write {}", output_dir.join("paired-rows.csv").display()))?;
    Ok(())
}

fn csv_escape(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn comparison_warnings(runs: &[RunComparisonSummary]) -> Vec<String> {
    let mut warnings = Vec::new();
    let priority_fee_values = runs
        .iter()
        .filter_map(|run| run.avg_priority_fee_lamports)
        .map(|value| value.round() as u64)
        .collect::<BTreeSet<_>>();

    if priority_fee_values.len() > 1 {
        warnings.push(
            "Compared runs use route-dependent priority fees. Treat this as a cost-aware route-strategy comparison, not a fee-neutral transport isolation run.".to_string(),
        );
    }

    let has_jito_route = runs.iter().any(|run| run.routes.contains("jito_bundle"));
    if has_jito_route {
        let has_route_tip = runs
            .iter()
            .any(|run| run.avg_route_tip_generated_lamports.unwrap_or(0.0) > 0.0);
        if !has_route_tip {
            warnings.push(
                "At least one compared run includes jito_bundle. Full economic cost is incomplete because no route-tip telemetry was present in the artifacts.".to_string(),
            );
        }
        let generated_tip = runs
            .iter()
            .map(|run| run.avg_route_tip_generated_lamports.unwrap_or(0.0))
            .sum::<f64>();
        let landed_tip = runs
            .iter()
            .map(|run| run.avg_route_tip_landed_lamports.unwrap_or(0.0))
            .sum::<f64>();
        if generated_tip > 0.0 && landed_tip <= 0.0 {
            warnings.push(
                "Jito route-tip signatures were returned by the relay, but none were observed in matched streams. The benchmarker includes Jito tip accounts automatically; verify the observation collector was enabled and route-tip signatures were present in the stream window.".to_string(),
            );
        }
    }

    warnings
}

#[derive(Debug)]
struct RunInputs {
    samples: Vec<BenchSample>,
    sends: Vec<LeaderSendEvent>,
    observations: Vec<ObservationEvent>,
    manifest: Option<BenchManifest>,
}

fn load_runs(
    artifact_dir: &Path,
    label: Option<String>,
    requested_primary_source: &str,
) -> Result<Vec<RunComparisonSummary>> {
    let samples: Vec<BenchSample> = read_ndjson(&artifact_dir.join("samples.ndjson"))?;
    let sends: Vec<LeaderSendEvent> = read_ndjson(&artifact_dir.join("leader-sends.ndjson"))?;
    let observations: Vec<ObservationEvent> =
        read_ndjson(&artifact_dir.join("matched-observations.ndjson"))?;
    let manifest = read_manifest(artifact_dir)?;

    let paired_arms = sends
        .iter()
        .filter_map(|send| send.policy_arm.clone())
        .collect::<BTreeSet<_>>();
    if paired_arms.len() > 1 {
        return paired_arms
            .into_iter()
            .map(|arm| {
                let signatures = sends
                    .iter()
                    .filter(|send| send.policy_arm.as_deref() == Some(arm.as_str()))
                    .map(|send| send.signature.clone())
                    .collect::<BTreeSet<_>>();
                let inputs = RunInputs {
                    samples: samples
                        .iter()
                        .filter(|sample| signatures.contains(&sample.signature))
                        .cloned()
                        .collect(),
                    sends: sends
                        .iter()
                        .filter(|send| signatures.contains(&send.signature))
                        .cloned()
                        .collect(),
                    observations: observations
                        .iter()
                        .filter(|event| {
                            signatures.contains(
                                event
                                    .caller_signature
                                    .as_deref()
                                    .unwrap_or(&event.signature),
                            )
                        })
                        .cloned()
                        .collect(),
                    manifest: manifest.clone(),
                };
                let arm_label = label
                    .as_deref()
                    .map(|base| format!("{base} / {arm}"))
                    .unwrap_or_else(|| arm.clone());
                load_run_from_inputs(
                    artifact_dir,
                    Some(arm_label),
                    requested_primary_source,
                    inputs,
                )
            })
            .collect();
    }

    load_run_from_inputs(
        artifact_dir,
        label,
        requested_primary_source,
        RunInputs {
            samples,
            sends,
            observations,
            manifest,
        },
    )
    .map(|run| vec![run])
}

fn load_run_from_inputs(
    artifact_dir: &Path,
    label: Option<String>,
    requested_primary_source: &str,
    inputs: RunInputs,
) -> Result<RunComparisonSummary> {
    let RunInputs {
        samples,
        sends,
        observations,
        manifest,
    } = inputs;

    let test_id = samples
        .first()
        .map(|sample| sample.test_id.clone())
        .or_else(|| sends.first().map(|send| send.test_id.clone()))
        .or_else(|| observations.first().map(|event| event.test_id.clone()))
        .unwrap_or_else(|| {
            artifact_dir
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("unknown")
                .to_string()
        });
    let label = label.unwrap_or_else(|| test_id.clone());

    let send_by_signature = sends
        .iter()
        .map(|send| (send.signature.clone(), send))
        .collect::<BTreeMap<_, _>>();

    let caller_observations = observations
        .iter()
        .filter(|event| !is_route_tip_observation(event))
        .cloned()
        .collect::<Vec<_>>();
    let observed_signatures = observations
        .iter()
        .map(|event| event.signature.clone())
        .collect::<BTreeSet<_>>();

    let primary_source = select_source(&caller_observations, requested_primary_source);
    let primary = source_stats(&caller_observations, &send_by_signature, &primary_source);
    let deshred = source_stats(&caller_observations, &send_by_signature, "rpcedge_deshred");
    let processed = source_stats(
        &caller_observations,
        &send_by_signature,
        "rpcedge_processed",
    );

    let samples_by_signature = samples_by_signature(&samples);
    let mut ack_latencies_ms = samples_by_signature
        .values()
        .filter_map(|rows| {
            rows.iter()
                .filter(|sample| sample.accepted)
                .map(|sample| sample.provider_ack_latency_us)
                .min()
        })
        .map(|ack_us| ack_us as f64 / 1000.0)
        .collect::<Vec<_>>();
    ack_latencies_ms.sort_by(f64::total_cmp);

    let total_runs = submitted_signature_count(&samples, &sends, &caller_observations);
    let accepted = samples_by_signature
        .values()
        .filter(|rows| rows.iter().any(|sample| sample.accepted))
        .count();
    let errors = total_runs.saturating_sub(accepted);

    let cost_rows = cost_rows(&samples, &sends, &observed_signatures);
    let base_fees = cost_rows
        .iter()
        .map(|cost| cost.base_fee_lamports as f64)
        .collect::<Vec<_>>();
    let priority_fees = cost_rows
        .iter()
        .map(|cost| cost.priority_fee_lamports as f64)
        .collect::<Vec<_>>();
    let transaction_fees = cost_rows
        .iter()
        .map(|cost| cost.transaction_fee_lamports as f64)
        .collect::<Vec<_>>();
    let route_tips_generated = cost_rows
        .iter()
        .map(|cost| cost.route_tip_generated_lamports as f64)
        .collect::<Vec<_>>();
    let route_tips_landed = cost_rows
        .iter()
        .map(|cost| cost.route_tip_landed_lamports as f64)
        .collect::<Vec<_>>();
    let total_generated_spends = cost_rows
        .iter()
        .map(|cost| cost.total_generated_spend_lamports as f64)
        .collect::<Vec<_>>();
    let total_landed_spends = cost_rows
        .iter()
        .map(|cost| cost.total_landed_spend_lamports as f64)
        .collect::<Vec<_>>();
    let block_seen = processed.count;
    let landed_runs = primary.count;
    let same_slot_landed = primary
        .slot_deltas
        .iter()
        .filter(|delta| **delta == 0)
        .count();

    Ok(RunComparisonSummary {
        label,
        test_id,
        artifact_dir: artifact_dir.display().to_string(),
        route_policy: route_policy_summary(&samples, manifest.as_ref()),
        routes: routes_summary(&samples, manifest.as_ref()),
        total_runs,
        provider_samples: samples.len(),
        accepted,
        errors,
        avg_ack_ms: avg(&ack_latencies_ms),
        p90_ack_ms: percentile_f64(&ack_latencies_ms, 0.90),
        avg_deshred_ms: avg(&deshred.latencies_ms),
        p90_deshred_ms: percentile_f64(&deshred.latencies_ms, 0.90),
        avg_landed_ms: avg(&primary.latencies_ms),
        p90_landed_ms: percentile_f64(&primary.latencies_ms, 0.90),
        avg_block_ms: avg(&processed.latencies_ms),
        p90_block_ms: percentile_f64(&processed.latencies_ms, 0.90),
        avg_landed_slots: avg_i64(&primary.slot_deltas),
        p90_landed_slots: percentile_i64_as_f64(&primary.slot_deltas, 0.90),
        avg_idx_in_block: avg_u64(&processed.slot_indexes),
        p90_idx_in_block: percentile_u64_as_f64(&processed.slot_indexes, 0.90),
        avg_base_fee_lamports: avg(&base_fees),
        avg_priority_fee_lamports: avg(&priority_fees),
        avg_transaction_fee_lamports: avg(&transaction_fees),
        avg_route_tip_generated_lamports: avg(&route_tips_generated),
        avg_route_tip_landed_lamports: avg(&route_tips_landed),
        avg_total_generated_spend_lamports: avg(&total_generated_spends),
        avg_total_landed_spend_lamports: avg(&total_landed_spends),
        max_slots: primary.slot_deltas.iter().max().copied(),
        min_slots: primary.slot_deltas.iter().min().copied(),
        same_slot_landed,
        landed_runs,
        block_seen,
        success_ratio_pct: ratio(landed_runs, total_runs) * 100.0,
        performance_rate_pct: 0.0,
        score: ScoreBreakdown::default(),
    })
}

fn submitted_signature_count(
    samples: &[BenchSample],
    sends: &[LeaderSendEvent],
    observations: &[ObservationEvent],
) -> usize {
    let mut signatures = sends
        .iter()
        .map(|send| send.signature.as_str())
        .collect::<BTreeSet<_>>();
    if signatures.is_empty() {
        signatures.extend(
            samples
                .iter()
                .filter(|sample| sample.accepted)
                .map(|sample| sample.signature.as_str()),
        );
    }
    if signatures.is_empty() {
        signatures.extend(observations.iter().map(|event| event.signature.as_str()));
    }
    signatures.len()
}

fn samples_by_signature(samples: &[BenchSample]) -> BTreeMap<String, Vec<&BenchSample>> {
    let mut samples_by_signature = BTreeMap::<String, Vec<&BenchSample>>::new();
    for sample in samples {
        samples_by_signature
            .entry(sample.signature.clone())
            .or_default()
            .push(sample);
    }
    samples_by_signature
}

fn representative_samples_by_signature(samples: &[BenchSample]) -> Vec<&BenchSample> {
    let mut by_signature = BTreeMap::<String, &BenchSample>::new();
    for sample in samples {
        by_signature
            .entry(sample.signature.clone())
            .and_modify(|existing| {
                let existing_attempt = existing.resend_attempt_index.unwrap_or(u16::MAX);
                let sample_attempt = sample.resend_attempt_index.unwrap_or(u16::MAX);
                if sample_attempt < existing_attempt {
                    *existing = sample;
                }
            })
            .or_insert(sample);
    }
    by_signature.into_values().collect()
}

#[derive(Debug, Clone, Default)]
struct CostRow {
    base_fee_lamports: u64,
    priority_fee_lamports: u64,
    transaction_fee_lamports: u64,
    route_tip_generated_lamports: u64,
    route_tip_landed_lamports: u64,
    total_generated_spend_lamports: u64,
    total_landed_spend_lamports: u64,
}

fn cost_rows(
    samples: &[BenchSample],
    sends: &[LeaderSendEvent],
    observed_signatures: &BTreeSet<String>,
) -> Vec<CostRow> {
    let mut sample_by_signature = BTreeMap::<String, Vec<&BenchSample>>::new();
    for sample in samples {
        sample_by_signature
            .entry(sample.signature.clone())
            .or_default()
            .push(sample);
    }

    let mut signatures = sends
        .iter()
        .map(|send| send.signature.clone())
        .collect::<BTreeSet<_>>();
    signatures.extend(sample_by_signature.keys().cloned());

    signatures
        .into_iter()
        .map(|signature| {
            let send = sends.iter().find(|send| send.signature == signature);
            let sample = sample_by_signature
                .get(&signature)
                .and_then(|rows| rows.first().copied());

            let base_fee_lamports = send
                .map(|send| send.base_fee_lamports)
                .or_else(|| sample.map(|sample| sample.base_fee_lamports))
                .unwrap_or(5_000);
            let priority_fee_lamports = send
                .map(|send| send.priority_fee_lamports)
                .or_else(|| sample.map(|sample| sample.priority_fee_lamports))
                .unwrap_or_else(|| {
                    let (limit, price) = send
                        .map(|send| {
                            (
                                send.compute_unit_limit,
                                send.compute_unit_price_microlamports,
                            )
                        })
                        .or_else(|| {
                            sample.map(|sample| {
                                (
                                    sample.compute_unit_limit,
                                    sample.compute_unit_price_microlamports,
                                )
                            })
                        })
                        .unwrap_or((0, 0));
                    priority_fee_lamports(limit, price)
                });
            let transaction_fee_lamports = send
                .map(|send| send.transaction_fee_lamports)
                .or_else(|| sample.map(|sample| sample.transaction_fee_lamports))
                .unwrap_or_else(|| base_fee_lamports.saturating_add(priority_fee_lamports));

            let mut route_tip_generated_lamports = 0_u64;
            let mut route_tip_landed_lamports = 0_u64;
            if let Some(samples) = sample_by_signature.get(&signature) {
                for sample in samples {
                    for result in &sample.route_results {
                        let lamports = result.route_tip_lamports.unwrap_or(0);
                        if lamports == 0 {
                            continue;
                        }
                        route_tip_generated_lamports =
                            route_tip_generated_lamports.saturating_add(lamports);
                        if result
                            .route_tip_signature
                            .as_ref()
                            .map(|signature| observed_signatures.contains(signature))
                            .unwrap_or(false)
                        {
                            route_tip_landed_lamports =
                                route_tip_landed_lamports.saturating_add(lamports);
                        }
                    }
                }
            }

            CostRow {
                base_fee_lamports,
                priority_fee_lamports,
                transaction_fee_lamports,
                route_tip_generated_lamports,
                route_tip_landed_lamports,
                total_generated_spend_lamports: transaction_fee_lamports
                    .saturating_add(route_tip_generated_lamports),
                total_landed_spend_lamports: transaction_fee_lamports
                    .saturating_add(route_tip_landed_lamports),
            }
        })
        .collect()
}

fn is_route_tip_observation(event: &ObservationEvent) -> bool {
    event.signature_role.as_deref() == Some("route_tip")
}

fn read_ndjson<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<Vec<T>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let file = File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut rows = Vec::new();
    for (line_no, line) in BufReader::new(file).lines().enumerate() {
        let line = line.with_context(|| format!("read {}:{}", path.display(), line_no + 1))?;
        if line.trim().is_empty() {
            continue;
        }
        rows.push(
            serde_json::from_str::<T>(&line)
                .with_context(|| format!("parse {}:{}", path.display(), line_no + 1))?,
        );
    }
    Ok(rows)
}

fn read_manifest(artifact_dir: &Path) -> Result<Option<BenchManifest>> {
    let path = artifact_dir.join("manifest.json");
    if !path.exists() {
        return Ok(None);
    }
    let bytes = fs::read(&path).with_context(|| format!("read {}", path.display()))?;
    Ok(Some(
        serde_json::from_slice::<BenchManifest>(&bytes)
            .with_context(|| format!("parse {}", path.display()))?,
    ))
}

fn select_source(observations: &[ObservationEvent], requested: &str) -> String {
    if matches!(requested, "first_observed" | "first-observed") {
        return "first_observed".to_string();
    }
    if observations
        .iter()
        .any(|event| event.source_name == requested)
    {
        return requested.to_string();
    }
    observations
        .iter()
        .find(|event| event.source_name.contains("processed"))
        .map(|event| event.source_name.clone())
        .or_else(|| observations.first().map(|event| event.source_name.clone()))
        .unwrap_or_else(|| requested.to_string())
}

fn source_stats(
    observations: &[ObservationEvent],
    send_by_signature: &BTreeMap<String, &LeaderSendEvent>,
    source: &str,
) -> SourceStats {
    let mut first_by_signature = BTreeMap::<String, &ObservationEvent>::new();
    for event in observations
        .iter()
        .filter(|event| source == "first_observed" || event.source_name == source)
    {
        first_by_signature
            .entry(event.signature.clone())
            .and_modify(|existing| {
                if event.observed_at < existing.observed_at {
                    *existing = event;
                }
            })
            .or_insert(event);
    }

    let mut stats = SourceStats {
        count: first_by_signature.len(),
        ..SourceStats::default()
    };
    for event in first_by_signature.values() {
        let send = send_by_signature.get(&event.signature).copied();
        let submitted_at = event.submitted_at.or_else(|| send.map(|send| send.sent_at));
        if let Some(submitted_at) = submitted_at {
            if let Some(us) = event
                .observed_at
                .signed_duration_since(submitted_at)
                .num_microseconds()
            {
                stats.latencies_ms.push(us as f64 / 1000.0);
            }
        }
        if let (Some(slot), Some(send)) = (event.slot, send) {
            stats.slot_deltas.push(slot as i64 - send.send_slot as i64);
        }
        if let Some(slot_index) = event.slot_index {
            stats.slot_indexes.push(slot_index);
        }
    }
    stats.latencies_ms.sort_by(f64::total_cmp);
    stats.slot_deltas.sort_unstable();
    stats.slot_indexes.sort_unstable();
    stats
}

fn route_policy_summary(samples: &[BenchSample], manifest: Option<&BenchManifest>) -> String {
    let mut counts = BTreeMap::<String, usize>::new();
    for sample in representative_samples_by_signature(samples) {
        let policy = sample
            .route_policy
            .clone()
            .or_else(|| sample.route_mode.clone())
            .unwrap_or_else(|| "static".to_string());
        *counts.entry(policy).or_default() += 1;
    }
    if counts.is_empty() {
        if let Some(manifest) = manifest {
            let policies = manifest
                .providers
                .iter()
                .filter_map(|provider| provider.route_mode.clone())
                .collect::<BTreeSet<_>>();
            if !policies.is_empty() {
                return policies.into_iter().collect::<Vec<_>>().join(", ");
            }
        }
        return "static".to_string();
    }
    counts
        .into_iter()
        .map(|(policy, count)| format!("{policy} ({count})"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn routes_summary(samples: &[BenchSample], manifest: Option<&BenchManifest>) -> String {
    let mut counts = BTreeMap::<String, usize>::new();
    for sample in representative_samples_by_signature(samples) {
        let routes = if sample.selected_routes.is_empty() {
            String::new()
        } else {
            sample.selected_routes.join("+")
        };
        if !routes.is_empty() {
            *counts.entry(routes).or_default() += 1;
        }
    }
    if counts.is_empty() {
        if let Some(manifest) = manifest {
            let routes = manifest
                .providers
                .iter()
                .flat_map(|provider| provider.routes.iter().cloned())
                .collect::<BTreeSet<_>>();
            if !routes.is_empty() {
                return routes.into_iter().collect::<Vec<_>>().join("+");
            }
        }
        return "unknown".to_string();
    }
    counts
        .into_iter()
        .map(|(routes, count)| format!("{routes} ({count})"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn apply_scores(runs: &mut [RunComparisonSummary]) {
    let best_avg_landed_ms = min_present(runs.iter().filter_map(|run| run.avg_landed_ms));
    let best_p90_landed_ms = min_present(runs.iter().filter_map(|run| run.p90_landed_ms));
    let best_avg_slots = min_present(runs.iter().filter_map(|run| run.avg_landed_slots));
    let best_p90_slots = min_present(runs.iter().filter_map(|run| run.p90_landed_slots));
    let best_avg_idx = min_present(runs.iter().filter_map(|run| run.avg_idx_in_block));
    let best_p90_idx = min_present(runs.iter().filter_map(|run| run.p90_idx_in_block));
    let best_same_slot = max_present(
        runs.iter()
            .map(|run| ratio(run.same_slot_landed, run.total_runs) * 100.0),
    );
    let best_success = max_present(runs.iter().map(|run| run.success_ratio_pct));

    for run in runs {
        let avg_landed_ms =
            lower_is_better_score(run.avg_landed_ms, best_avg_landed_ms).unwrap_or(0.0);
        let p90_landed_ms =
            lower_is_better_score(run.p90_landed_ms, best_p90_landed_ms).unwrap_or(0.0);
        let avg_slots = lower_is_better_score(run.avg_landed_slots, best_avg_slots).unwrap_or(0.0);
        let p90_slots = lower_is_better_score(run.p90_landed_slots, best_p90_slots).unwrap_or(0.0);
        let avg_idx = lower_is_better_score(run.avg_idx_in_block, best_avg_idx).unwrap_or(0.0);
        let p90_idx = lower_is_better_score(run.p90_idx_in_block, best_p90_idx).unwrap_or(0.0);
        let same_slot_rate = ratio(run.same_slot_landed, run.total_runs) * 100.0;

        run.score = ScoreBreakdown {
            landed_ms: 0.2 * avg_landed_ms + 0.8 * p90_landed_ms,
            landed_slots: 0.2 * avg_slots + 0.8 * p90_slots,
            landed_idx: 0.5 * avg_idx + 0.5 * p90_idx,
            same_slot: higher_is_better_score(same_slot_rate, best_same_slot).unwrap_or(0.0),
            success_ratio: higher_is_better_score(run.success_ratio_pct, best_success)
                .unwrap_or(0.0),
        };
        run.performance_rate_pct = (run.score.landed_ms
            + run.score.landed_slots
            + run.score.landed_idx
            + run.score.same_slot
            + run.score.success_ratio)
            / 5.0;
    }
}

fn min_present(values: impl Iterator<Item = f64>) -> Option<f64> {
    values
        .filter(|value| value.is_finite())
        .min_by(f64::total_cmp)
}

fn max_present(values: impl Iterator<Item = f64>) -> Option<f64> {
    values
        .filter(|value| value.is_finite())
        .max_by(f64::total_cmp)
}

fn lower_is_better_score(value: Option<f64>, best: Option<f64>) -> Option<f64> {
    let value = value?;
    let best = best?;
    if !value.is_finite() || !best.is_finite() {
        return None;
    }
    if value <= best {
        return Some(100.0);
    }
    let score = if best <= 0.0 {
        100.0 / (1.0 + value - best)
    } else {
        100.0 * best / value
    };
    Some(score.clamp(0.0, 100.0))
}

fn higher_is_better_score(value: f64, best: Option<f64>) -> Option<f64> {
    let best = best?;
    if !value.is_finite() || !best.is_finite() {
        return None;
    }
    if best <= 0.0 {
        return Some(if value <= 0.0 { 100.0 } else { 0.0 });
    }
    Some((100.0 * value / best).clamp(0.0, 100.0))
}

fn avg(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        None
    } else {
        Some(values.iter().sum::<f64>() / values.len() as f64)
    }
}

fn avg_i64(values: &[i64]) -> Option<f64> {
    if values.is_empty() {
        None
    } else {
        Some(values.iter().map(|value| *value as f64).sum::<f64>() / values.len() as f64)
    }
}

fn avg_u64(values: &[u64]) -> Option<f64> {
    if values.is_empty() {
        None
    } else {
        Some(values.iter().map(|value| *value as f64).sum::<f64>() / values.len() as f64)
    }
}

fn percentile_f64(sorted: &[f64], p: f64) -> Option<f64> {
    if sorted.is_empty() {
        return None;
    }
    let rank = ((sorted.len() - 1) as f64 * p).ceil() as usize;
    sorted.get(rank).copied()
}

fn percentile_i64_as_f64(sorted: &[i64], p: f64) -> Option<f64> {
    if sorted.is_empty() {
        return None;
    }
    let rank = ((sorted.len() - 1) as f64 * p).ceil() as usize;
    sorted.get(rank).map(|value| *value as f64)
}

fn percentile_i64_value(sorted: &[i64], p: f64) -> Option<i64> {
    if sorted.is_empty() {
        return None;
    }
    let index = ((sorted.len() - 1) as f64 * p).round() as usize;
    sorted.get(index).copied()
}

fn percentile_u64_as_f64(sorted: &[u64], p: f64) -> Option<f64> {
    if sorted.is_empty() {
        return None;
    }
    let rank = ((sorted.len() - 1) as f64 * p).ceil() as usize;
    sorted.get(rank).map(|value| *value as f64)
}

fn ratio(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}

fn comparison_markdown(output: &ComparisonOutput) -> String {
    let mut out = String::new();
    out.push_str(&format!("# {}\n\n", output.title));
    out.push_str(&format!(
        "- Generated: `{}`\n- Primary landing source: `{}`\n- Runs: `{}`\n\n",
        output
            .generated_at
            .to_rfc3339_opts(SecondsFormat::Millis, true),
        output.primary_source,
        output.run_count,
    ));
    out.push_str("## Methodology\n\n");
    out.push_str(methodology_text());
    for warning in &output.warnings {
        out.push_str(&format!("\n\n> **Warning:** {warning}"));
    }
    if let Some(paired) = &output.paired_comparison {
        out.push_str("\n\n## Paired Matched Deltas\n\n");
        out.push_str(&format!(
            "Baseline is `{}` and candidate is `{}`. Every delta is `candidate - baseline`; a negative latency delta means the candidate was observed sooner. Provider ACK is not a TPU-arrival measurement.\n\n",
            paired.baseline_arm, paired.candidate_arm
        ));
        out.push_str(&format!(
            "Observed leader runs: `{}`; eligible: `{}` ({:.2}%). Observed scheduled slots: `{}`; eligible: `{}` ({:.2}%).\n\n",
            paired.coverage.observed_leader_runs,
            paired.coverage.eligible_leader_runs,
            paired.coverage.eligible_leader_run_pct,
            paired.coverage.observed_scheduled_slots,
            paired.coverage.eligible_scheduled_slots,
            paired.coverage.eligible_scheduled_slot_pct,
        ));
        out.push_str("| Source | Groups | Both | Baseline only | Candidate only | Neither | Candidate wins | Ties | Baseline wins | Delta p10 ms | Delta p50 ms | Delta p90 ms | Landed-slot delta p50 | Block-index delta p50 |\n");
        out.push_str("| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |\n");
        for source in &paired.sources {
            out.push_str(&format!(
                "| `{}` | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
                source.source_name,
                source.comparison_groups,
                source.both_observed,
                source.baseline_only,
                source.candidate_only,
                source.neither_observed,
                source.candidate_wins,
                source.ties,
                source.baseline_wins,
                fmt_f64(source.delta_p10_ms),
                fmt_f64(source.delta_p50_ms),
                fmt_f64(source.delta_p90_ms),
                fmt_i64(source.landed_slot_delta_p50),
                fmt_i64(source.slot_index_delta_p50),
            ));
        }
        out.push_str("\nPer-leader matched rows: `paired-rows.json` and `paired-rows.csv`.\n");
    }
    out.push_str("\n\n## Comparison\n\n");
    out.push_str("| Run | Routes | Avg First-Observed ms | P90 First-Observed ms | Avg Landed slots | P90 Landed slots | Avg Idx-in-Block | P90 Idx-in-Block | Avg Base Fee | Avg Priority Fee | Avg Tx Fee | Avg Route Tip Generated | Avg Route Tip Landed | Avg Total Generated | Avg Total Landed | Max Slots | Min Slots | Same-slot landed | Landed runs | Block seen | Total runs | Success ratio % | Performance rate % |\n");
    out.push_str("| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |\n");
    for run in &output.runs {
        out.push_str(&format!(
            "| `{}` | `{}` | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {:.2} | {:.2} |\n",
            run.label,
            run.routes,
            fmt_f64(run.avg_landed_ms),
            fmt_f64(run.p90_landed_ms),
            fmt_f64(run.avg_landed_slots),
            fmt_f64(run.p90_landed_slots),
            fmt_f64(run.avg_idx_in_block),
            fmt_f64(run.p90_idx_in_block),
            fmt_f64(run.avg_base_fee_lamports),
            fmt_f64(run.avg_priority_fee_lamports),
            fmt_f64(run.avg_transaction_fee_lamports),
            fmt_f64(run.avg_route_tip_generated_lamports),
            fmt_f64(run.avg_route_tip_landed_lamports),
            fmt_f64(run.avg_total_generated_spend_lamports),
            fmt_f64(run.avg_total_landed_spend_lamports),
            fmt_i64(run.max_slots),
            fmt_i64(run.min_slots),
            run.same_slot_landed,
            run.landed_runs,
            run.block_seen,
            run.total_runs,
            run.success_ratio_pct,
            run.performance_rate_pct,
        ));
    }
    out.push_str("\n## Score Components\n\n");
    out.push_str(
        "| Run | Landed ms | Landed slots | Block index | Same-slot | Success | Final |\n",
    );
    out.push_str("| --- | ---: | ---: | ---: | ---: | ---: | ---: |\n");
    for run in &output.runs {
        out.push_str(&format!(
            "| `{}` | {:.2} | {:.2} | {:.2} | {:.2} | {:.2} | {:.2} |\n",
            run.label,
            run.score.landed_ms,
            run.score.landed_slots,
            run.score.landed_idx,
            run.score.same_slot,
            run.score.success_ratio,
            run.performance_rate_pct,
        ));
    }
    out
}

fn methodology_text() -> &'static str {
    "Landing proof comes from matched observations by transaction signature, not provider ACK. By default, landing latency uses the first observed stream event for each caller transaction; processed metadata is still used for block position because deshred observations do not carry a transaction index. Cost columns separate deterministic transaction fees from route-added tips: base fee and priority fee are encoded in the signed transaction, generated route tips come from route-result ACK metadata, and landed route tips count only tip signatures observed in the matched streams. Lower-is-better metrics are normalized relative to the best run in the comparison set. Landing latency and landed slots weight p90 at 80% and average at 20%. Block position weights average and p90 equally. Same-slot and success use rates so runs with different counts remain comparable. The final performance rate is the unweighted average of the five bucket scores. Only compare runs with the same transaction shape, trigger mode, sender region, and fee/tip policy; otherwise treat the score as descriptive rather than a claim."
}

fn comparison_html(output: &ComparisonOutput) -> String {
    let mut out = String::new();
    out.push_str("<!doctype html><html><head><meta charset=\"utf-8\">");
    out.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">");
    out.push_str(&format!("<title>{}</title>", html_escape(&output.title)));
    out.push_str(
        "<style>
*{box-sizing:border-box}
body{font-family:Inter,ui-sans-serif,system-ui,-apple-system,Segoe UI,sans-serif;margin:0;background:#f5f7f8;color:#17202a;line-height:1.45}
main{max-width:1380px;margin:0 auto;padding:28px}
a{color:#1769aa}.hero,.section{background:#fff;border:1px solid #dfe5e8;border-radius:8px;padding:20px;margin-bottom:16px}
.eyebrow{font-size:12px;letter-spacing:.08em;text-transform:uppercase;color:#667681;font-weight:700}
h1{font-size:30px;line-height:1.08;margin:6px 0 8px;letter-spacing:0} h2{font-size:18px;margin:26px 0 10px;letter-spacing:0}
.muted{color:#667681}.note{background:#f8fbfc;border-left:4px solid #2f7dbd;padding:11px 13px;margin:12px 0;max-width:1180px;color:#465760}.warning{border-left-color:#d18800;background:#fff8e8}
.grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:10px;margin:18px 0 0}
.card{background:#fbfcfd;border:1px solid #e1e7ea;border-radius:8px;padding:12px;min-height:82px}.card span{display:block;color:#667681;font-size:12px}.card b{display:block;font-size:24px;line-height:1.15;margin-top:4px}.card small{display:block;color:#72818a;margin-top:4px}
.table-wrap{overflow-x:auto;border:1px solid #e2e8eb;border-radius:8px;background:#fff}table{width:100%;border-collapse:collapse;min-width:1120px}
th,td{padding:8px 10px;border-bottom:1px solid #edf1f3;text-align:right;font-size:13px;vertical-align:top;white-space:nowrap}
th:first-child,td:first-child,td.left{text-align:left}th{background:#f0f4f6;color:#41515b;font-weight:700}
code{background:#eef2f4;padding:1px 5px;border-radius:4px}.pill{display:inline-flex;align-items:center;border:1px solid #d9e1e5;background:#f7fafb;border-radius:999px;padding:3px 8px;font-size:12px;color:#465760}.links{display:flex;flex-wrap:wrap;gap:8px;margin-top:14px}
@media(max-width:720px){main{padding:14px}.hero,.section{padding:16px}h1{font-size:24px}}
</style></head><body><main>",
    );
    out.push_str(&format!(
        "<section class=\"hero\"><div class=\"eyebrow\">RPCEdge route policy comparison</div><h1>{}</h1><p class=\"muted\">Paired leader-paced policy arms scored by matched landing observations. First-observed latency is the primary timing metric; processed metadata supplies block position.</p><div class=\"links\"><span class=\"pill\">generated <code>{}</code></span><span class=\"pill\">primary source <code>{}</code></span><a class=\"pill\" href=\"../\">Full landing report</a></div>",
        html_escape(&output.title),
        html_escape(&output.generated_at.to_rfc3339_opts(SecondsFormat::Millis, true)),
        html_escape(&output.primary_source),
    ));
    out.push_str("<div class=\"grid\">");
    if let Some(best) = output
        .runs
        .iter()
        .max_by(|a, b| a.performance_rate_pct.total_cmp(&b.performance_rate_pct))
    {
        out.push_str(&format!(
            "<div class=\"card\"><span>Best score</span><b>{:.2}%</b><small>{}</small></div>",
            best.performance_rate_pct,
            html_escape(&best.label)
        ));
    }
    for (label, value) in [
        ("Runs", output.run_count.to_string()),
        (
            "Total txs",
            output
                .runs
                .iter()
                .map(|run| run.total_runs)
                .sum::<usize>()
                .to_string(),
        ),
        (
            "Landed txs",
            output
                .runs
                .iter()
                .map(|run| run.landed_runs)
                .sum::<usize>()
                .to_string(),
        ),
    ] {
        out.push_str(&format!(
            "<div class=\"card\"><span>{}</span><b>{}</b></div>",
            html_escape(label),
            html_escape(&value)
        ));
    }
    out.push_str("</div></section>");
    out.push_str("<section class=\"section\"><h2>Methodology</h2>");
    out.push_str(&format!(
        "<div class=\"note\">{}</div>",
        html_escape(methodology_text())
    ));
    for warning in &output.warnings {
        out.push_str(&format!(
            "<div class=\"note warning\"><b>Warning:</b> {}</div>",
            html_escape(warning)
        ));
    }
    out.push_str("</section>");

    if let Some(paired) = &output.paired_comparison {
        out.push_str("<section class=\"section\"><h2>Paired Matched Deltas</h2>");
        out.push_str(&format!(
            "<div class=\"note\">Baseline <code>{}</code>; candidate <code>{}</code>. Deltas are candidate minus baseline, so negative latency means the candidate was observed sooner. This is landing/observation evidence, not direct TPU packet-arrival time.</div>",
            html_escape(&paired.baseline_arm),
            html_escape(&paired.candidate_arm),
        ));
        out.push_str(&format!(
            "<div class=\"grid\"><div class=\"card\"><span>Eligible leader runs</span><b>{}/{}</b><small>{:.2}%</small></div><div class=\"card\"><span>Eligible scheduled slots</span><b>{}/{}</b><small>{:.2}%</small></div></div>",
            paired.coverage.eligible_leader_runs,
            paired.coverage.observed_leader_runs,
            paired.coverage.eligible_leader_run_pct,
            paired.coverage.eligible_scheduled_slots,
            paired.coverage.observed_scheduled_slots,
            paired.coverage.eligible_scheduled_slot_pct,
        ));
        out.push_str("<div class=\"table-wrap\"><table><thead><tr><th>Source</th><th>Groups</th><th>Both</th><th>Baseline only</th><th>Candidate only</th><th>Neither</th><th>Candidate wins</th><th>Ties</th><th>Baseline wins</th><th>Delta p10 ms</th><th>Delta p50 ms</th><th>Delta p90 ms</th><th>Slot delta p50</th><th>Index delta p50</th></tr></thead><tbody>");
        for source in &paired.sources {
            out.push_str(&format!(
                "<tr><td class=\"left\"><code>{}</code></td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                html_escape(&source.source_name),
                source.comparison_groups,
                source.both_observed,
                source.baseline_only,
                source.candidate_only,
                source.neither_observed,
                source.candidate_wins,
                source.ties,
                source.baseline_wins,
                fmt_f64(source.delta_p10_ms),
                fmt_f64(source.delta_p50_ms),
                fmt_f64(source.delta_p90_ms),
                fmt_i64(source.landed_slot_delta_p50),
                fmt_i64(source.slot_index_delta_p50),
            ));
        }
        out.push_str("</tbody></table></div><div class=\"links\"><a class=\"pill\" href=\"paired-rows.json\">Per-leader JSON</a><a class=\"pill\" href=\"paired-rows.csv\">Per-leader CSV</a></div></section>");
    }

    out.push_str("<section class=\"section\"><h2>Comparison</h2><div class=\"table-wrap\"><table><thead><tr><th>Run</th><th>Routes</th><th>Avg First-Observed ms</th><th>P90 First-Observed ms</th><th>Avg Landed slots</th><th>P90 Landed slots</th><th>Avg Idx</th><th>P90 Idx</th><th>Avg Base Fee</th><th>Avg Priority Fee</th><th>Avg Tx Fee</th><th>Avg Route Tip Generated</th><th>Avg Route Tip Landed</th><th>Avg Total Generated</th><th>Avg Total Landed</th><th>Max Slots</th><th>Min Slots</th><th>Same-slot</th><th>Landed</th><th>Block seen</th><th>Total</th><th>Success %</th><th>Score %</th></tr></thead><tbody>");
    for run in &output.runs {
        out.push_str(&format!(
            "<tr><td class=\"left\"><code>{}</code></td><td class=\"left\"><code>{}</code></td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{:.2}</td><td><b>{:.2}</b></td></tr>",
            html_escape(&run.label),
            html_escape(&run.routes),
            fmt_f64(run.avg_landed_ms),
            fmt_f64(run.p90_landed_ms),
            fmt_f64(run.avg_landed_slots),
            fmt_f64(run.p90_landed_slots),
            fmt_f64(run.avg_idx_in_block),
            fmt_f64(run.p90_idx_in_block),
            fmt_f64(run.avg_base_fee_lamports),
            fmt_f64(run.avg_priority_fee_lamports),
            fmt_f64(run.avg_transaction_fee_lamports),
            fmt_f64(run.avg_route_tip_generated_lamports),
            fmt_f64(run.avg_route_tip_landed_lamports),
            fmt_f64(run.avg_total_generated_spend_lamports),
            fmt_f64(run.avg_total_landed_spend_lamports),
            fmt_i64(run.max_slots),
            fmt_i64(run.min_slots),
            run.same_slot_landed,
            run.landed_runs,
            run.block_seen,
            run.total_runs,
            run.success_ratio_pct,
            run.performance_rate_pct,
        ));
    }
    out.push_str("</tbody></table></div></section>");

    out.push_str("<section class=\"section\"><h2>Score Components</h2><div class=\"table-wrap\"><table><thead><tr><th>Run</th><th>Landed ms</th><th>Landed slots</th><th>Block index</th><th>Same-slot</th><th>Success</th><th>Final</th></tr></thead><tbody>");
    for run in &output.runs {
        out.push_str(&format!(
            "<tr><td class=\"left\"><code>{}</code></td><td>{:.2}</td><td>{:.2}</td><td>{:.2}</td><td>{:.2}</td><td>{:.2}</td><td><b>{:.2}</b></td></tr>",
            html_escape(&run.label),
            run.score.landed_ms,
            run.score.landed_slots,
            run.score.landed_idx,
            run.score.same_slot,
            run.score.success_ratio,
            run.performance_rate_pct,
        ));
    }
    out.push_str("</tbody></table></div></section>");
    out.push_str("</main></body></html>");
    out
}

fn fmt_f64(value: Option<f64>) -> String {
    value
        .map(|value| format!("{value:.3}"))
        .unwrap_or_else(|| "-".to_string())
}

fn fmt_i64(value: Option<i64>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "-".to_string())
}

fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn paired_row(status: &str, delta_ms: Option<f64>) -> PairedComparisonRow {
        PairedComparisonRow {
            source_name: "rpcedge_processed".to_string(),
            comparison_group_id: format!("group-{status}-{:?}", delta_ms),
            leader_identity: "leader".to_string(),
            leader_region: Some("us-east".to_string()),
            leader_city: Some("New York".to_string()),
            leader_network_path_class: Some("doublezero".to_string()),
            leader_network_path_rtt_us: Some(10_000),
            baseline_arm: "public_internet".to_string(),
            candidate_arm: "doublezero".to_string(),
            baseline_signature: "baseline".to_string(),
            candidate_signature: "candidate".to_string(),
            status: status.to_string(),
            baseline_latency_ms: delta_ms.map(|_| 100.0),
            candidate_latency_ms: delta_ms.map(|delta| 100.0 + delta),
            delta_ms,
            baseline_landed_slot: None,
            candidate_landed_slot: None,
            landed_slot_delta: None,
            baseline_slot_index: None,
            candidate_slot_index: None,
            slot_index_delta: None,
        }
    }

    #[test]
    fn paired_provider_summary_keeps_complete_and_one_sided_outcomes_separate() {
        let rows = vec![
            paired_row("both_observed", Some(-5.0)),
            paired_row("both_observed", Some(0.0)),
            paired_row("both_observed", Some(2.0)),
            paired_row("baseline_only", None),
            paired_row("candidate_only", None),
            paired_row("neither_observed", None),
        ];

        let summary = summarize_paired_source("rpcedge_processed", &rows);

        assert_eq!(summary.comparison_groups, 6);
        assert_eq!(summary.both_observed, 3);
        assert_eq!(summary.baseline_only, 1);
        assert_eq!(summary.candidate_only, 1);
        assert_eq!(summary.neither_observed, 1);
        assert_eq!(summary.candidate_wins, 1);
        assert_eq!(summary.ties, 1);
        assert_eq!(summary.baseline_wins, 1);
        assert_eq!(summary.delta_p50_ms, Some(0.0));
    }

    #[test]
    fn comparison_scores_two_local_artifact_dirs() {
        let dir = tempdir().expect("tempdir");
        let a = dir.path().join("a");
        let b = dir.path().join("b");
        fs::create_dir_all(&a).expect("a");
        fs::create_dir_all(&b).expect("b");
        write_run(&a, "a", 100, 200, 1);
        write_run(&b, "b", 100, 400, 2);
        let output_dir = dir.path().join("out");

        let comparison = generate_comparison(CompareOptions {
            artifact_dirs: vec![a, b],
            labels: vec!["fast".to_string(), "slow".to_string()],
            output_dir: output_dir.clone(),
            primary_source: "rpcedge_processed".to_string(),
            title: "Test Compare".to_string(),
        })
        .expect("comparison");

        assert_eq!(comparison.run_count, 2);
        assert!(comparison.runs[0].performance_rate_pct > comparison.runs[1].performance_rate_pct);
        assert!(output_dir.join("comparison.json").exists());
        assert!(output_dir.join("index.html").exists());
    }

    #[test]
    fn comparison_splits_single_paired_artifact_by_policy_arm() {
        let dir = tempdir().expect("tempdir");
        let run = dir.path().join("paired");
        fs::create_dir_all(&run).expect("run");
        write_paired_run(&run);
        let output_dir = dir.path().join("out");

        let comparison = generate_comparison(CompareOptions {
            artifact_dirs: vec![run],
            labels: vec![],
            output_dir: output_dir.clone(),
            primary_source: "rpcedge_processed".to_string(),
            title: "Paired Compare".to_string(),
        })
        .expect("comparison");

        assert_eq!(comparison.run_count, 2);
        assert_eq!(comparison.runs[0].label, "always_race");
        assert_eq!(comparison.runs[1].label, "tpu_quic_only");
        assert_eq!(comparison.runs[0].total_runs, 1);
        assert_eq!(comparison.runs[1].total_runs, 1);
        assert!(comparison.runs[0].p90_landed_ms < comparison.runs[1].p90_landed_ms);

        let paired = comparison
            .paired_comparison
            .as_ref()
            .expect("paired comparison");
        assert_eq!(paired.baseline_arm, "tpu_quic_only");
        assert_eq!(paired.candidate_arm, "always_race");
        assert_eq!(paired.delta_convention, "candidate_minus_baseline");
        assert_eq!(paired.coverage.observed_leader_runs, 1);
        assert_eq!(paired.coverage.eligible_leader_runs, 1);
        assert_eq!(paired.coverage.observed_scheduled_slots, 4);
        assert_eq!(paired.coverage.eligible_scheduled_slots, 4);
        let processed = paired
            .sources
            .iter()
            .find(|source| source.source_name == "rpcedge_processed")
            .expect("processed summary");
        assert_eq!(processed.comparison_groups, 1);
        assert_eq!(processed.both_observed, 1);
        assert_eq!(processed.candidate_wins, 1);
        assert_eq!(processed.baseline_wins, 0);
        assert_eq!(processed.delta_p50_ms, Some(-200.0));
        let deshred = paired
            .sources
            .iter()
            .find(|source| source.source_name == "rpcedge_deshred")
            .expect("deshred summary");
        assert_eq!(deshred.comparison_groups, 1);
        assert_eq!(deshred.neither_observed, 1);
        assert!(output_dir.join("paired-rows.json").exists());
        assert!(output_dir.join("paired-rows.csv").exists());
    }

    #[test]
    fn comparison_deduplicates_resend_attempts_by_signature() {
        let dir = tempdir().expect("tempdir");
        let run = dir.path().join("resend");
        fs::create_dir_all(&run).expect("run");
        write_resend_run(&run);
        let output_dir = dir.path().join("out");

        let comparison = generate_comparison(CompareOptions {
            artifact_dirs: vec![run],
            labels: vec![],
            output_dir,
            primary_source: "rpcedge_processed".to_string(),
            title: "Resend Compare".to_string(),
        })
        .expect("comparison");

        assert_eq!(comparison.run_count, 2);
        let race = comparison
            .runs
            .iter()
            .find(|run| run.label == "always_race")
            .expect("always race run");
        assert_eq!(race.total_runs, 1);
        assert_eq!(race.provider_samples, 3);
        assert_eq!(race.accepted, 1);
        assert_eq!(race.errors, 0);
        assert_eq!(race.routes, "tpu_quic+jito_bundle+harmonic_bundle (1)");
        assert_eq!(race.route_policy, "always_race (1)");
        assert_eq!(race.avg_route_tip_generated_lamports, Some(3000.0));
        assert_eq!(race.avg_route_tip_landed_lamports, Some(1000.0));
        assert_eq!(race.avg_total_generated_spend_lamports, Some(8198.0));
        assert_eq!(race.avg_total_landed_spend_lamports, Some(6198.0));
    }

    fn write_run(path: &Path, test_id: &str, send_slot: u64, observed_ms: i64, slot_delta: u64) {
        fs::write(
            path.join("samples.ndjson"),
            format!(
                "{{\"schema_version\":2,\"test_id\":\"{test_id}\",\"iteration\":0,\"signature\":\"sig-{test_id}\",\"provider_name\":\"p\",\"provider_kind\":\"rpcedge_quic_raw_tx\",\"accepted\":true,\"client_started_at\":\"2026-01-01T00:00:00Z\",\"client_finished_at\":\"2026-01-01T00:00:00.001Z\",\"client_ack_latency_us\":1000,\"provider_send_started_at\":\"2026-01-01T00:00:00Z\",\"provider_send_finished_at\":\"2026-01-01T00:00:00.001Z\",\"provider_ack_latency_us\":1000,\"provider_request_id\":null,\"returned_signature\":\"sig-{test_id}\",\"status_code\":200,\"error_class\":null,\"error\":null,\"route_policy\":\"static\",\"route_mode\":\"only\",\"selected_routes\":[\"tpu_quic\"],\"leader_client_family\":\"jito\",\"compute_unit_limit\":660,\"compute_unit_price_microlamports\":0,\"base_fee_lamports\":5000,\"priority_fee_lamports\":0,\"transaction_fee_lamports\":5000}}\n"
            ),
        )
        .expect("samples");
        fs::write(
            path.join("leader-sends.ndjson"),
            format!(
                "{{\"schema_version\":2,\"test_id\":\"{test_id}\",\"iteration\":0,\"signature\":\"sig-{test_id}\",\"leader_identity\":\"leader\",\"leader_run_start_slot\":{send_slot},\"leader_run_end_slot\":{},\"send_slot\":{send_slot},\"sent_at\":\"2026-01-01T00:00:00Z\",\"trigger_source\":\"grpc_slot\",\"slot_signal_status\":null,\"slot_signal_observed_at\":null,\"leader_client_family\":\"jito\",\"route_policy\":\"static\",\"selected_routes\":[\"tpu_quic\"],\"compute_unit_limit\":660,\"compute_unit_price_microlamports\":0,\"base_fee_lamports\":5000,\"priority_fee_lamports\":0,\"transaction_fee_lamports\":5000}}\n",
                send_slot + 3
            ),
        )
        .expect("sends");
        fs::write(
            path.join("matched-observations.ndjson"),
            format!(
                "{{\"schema_version\":1,\"test_id\":\"{test_id}\",\"signature\":\"sig-{test_id}\",\"source_name\":\"rpcedge_deshred\",\"source_kind\":\"yellowstone_deshred\",\"observed_at\":\"2026-01-01T00:00:00.{observed_ms:03}Z\",\"submitted_at\":\"2026-01-01T00:00:00Z\",\"slot\":{},\"slot_index\":null,\"source_sequence\":null}}\n{{\"schema_version\":1,\"test_id\":\"{test_id}\",\"signature\":\"sig-{test_id}\",\"source_name\":\"rpcedge_processed\",\"source_kind\":\"yellowstone_processed\",\"observed_at\":\"2026-01-01T00:00:00.{observed_ms:03}Z\",\"submitted_at\":\"2026-01-01T00:00:00Z\",\"slot\":{},\"slot_index\":10,\"source_sequence\":null}}\n",
                send_slot + slot_delta,
                send_slot + slot_delta,
            ),
        )
        .expect("observations");
    }

    fn write_paired_run(path: &Path) {
        fs::write(
            path.join("samples.ndjson"),
            concat!(
                "{\"schema_version\":3,\"test_id\":\"paired\",\"iteration\":0,\"signature\":\"sig-tpu\",\"comparison_group_id\":\"group-1\",\"policy_arm\":\"tpu_quic_only\",\"policy_arm_index\":0,\"provider_name\":\"p\",\"provider_kind\":\"rpcedge_quic_raw_tx\",\"accepted\":true,\"client_started_at\":\"2026-01-01T00:00:00Z\",\"client_finished_at\":\"2026-01-01T00:00:00.001Z\",\"client_ack_latency_us\":1000,\"provider_send_started_at\":\"2026-01-01T00:00:00Z\",\"provider_send_finished_at\":\"2026-01-01T00:00:00.001Z\",\"provider_ack_latency_us\":1000,\"provider_request_id\":null,\"returned_signature\":\"sig-tpu\",\"status_code\":200,\"error_class\":null,\"error\":null,\"route_policy\":\"tpu_quic_only\",\"route_mode\":\"only\",\"selected_routes\":[\"tpu_quic\"],\"leader_client_family\":\"jito\",\"compute_unit_limit\":660,\"compute_unit_price_microlamports\":0,\"base_fee_lamports\":5000,\"priority_fee_lamports\":0,\"transaction_fee_lamports\":5000}\n",
                "{\"schema_version\":3,\"test_id\":\"paired\",\"iteration\":1,\"signature\":\"sig-race\",\"comparison_group_id\":\"group-1\",\"policy_arm\":\"always_race\",\"policy_arm_index\":1,\"provider_name\":\"p\",\"provider_kind\":\"rpcedge_quic_raw_tx\",\"accepted\":true,\"client_started_at\":\"2026-01-01T00:00:00Z\",\"client_finished_at\":\"2026-01-01T00:00:00.001Z\",\"client_ack_latency_us\":1000,\"provider_send_started_at\":\"2026-01-01T00:00:00Z\",\"provider_send_finished_at\":\"2026-01-01T00:00:00.001Z\",\"provider_ack_latency_us\":1000,\"provider_request_id\":null,\"returned_signature\":\"sig-race\",\"status_code\":200,\"error_class\":null,\"error\":null,\"route_policy\":\"always_race\",\"route_mode\":\"only\",\"selected_routes\":[\"tpu_quic\",\"jito_bundle\",\"harmonic_bundle\"],\"leader_client_family\":\"jito\",\"compute_unit_limit\":660,\"compute_unit_price_microlamports\":300000,\"base_fee_lamports\":5000,\"priority_fee_lamports\":198,\"transaction_fee_lamports\":5198}\n"
            ),
        )
        .expect("samples");
        fs::write(
            path.join("leader-sends.ndjson"),
            concat!(
                "{\"schema_version\":3,\"test_id\":\"paired\",\"iteration\":0,\"signature\":\"sig-tpu\",\"comparison_group_id\":\"group-1\",\"policy_arm\":\"tpu_quic_only\",\"policy_arm_index\":0,\"leader_identity\":\"leader\",\"leader_run_start_slot\":100,\"leader_run_end_slot\":103,\"send_slot\":100,\"sent_at\":\"2026-01-01T00:00:00Z\",\"trigger_source\":\"grpc_slot\",\"slot_signal_status\":null,\"slot_signal_observed_at\":null,\"leader_client_family\":\"jito\",\"route_policy\":\"tpu_quic_only\",\"selected_routes\":[\"tpu_quic\"],\"compute_unit_limit\":660,\"compute_unit_price_microlamports\":0,\"base_fee_lamports\":5000,\"priority_fee_lamports\":0,\"transaction_fee_lamports\":5000}\n",
                "{\"schema_version\":3,\"test_id\":\"paired\",\"iteration\":1,\"signature\":\"sig-race\",\"comparison_group_id\":\"group-1\",\"policy_arm\":\"always_race\",\"policy_arm_index\":1,\"leader_identity\":\"leader\",\"leader_run_start_slot\":100,\"leader_run_end_slot\":103,\"send_slot\":100,\"sent_at\":\"2026-01-01T00:00:00Z\",\"trigger_source\":\"grpc_slot\",\"slot_signal_status\":null,\"slot_signal_observed_at\":null,\"leader_client_family\":\"jito\",\"route_policy\":\"always_race\",\"selected_routes\":[\"tpu_quic\",\"jito_bundle\",\"harmonic_bundle\"],\"compute_unit_limit\":660,\"compute_unit_price_microlamports\":300000,\"base_fee_lamports\":5000,\"priority_fee_lamports\":198,\"transaction_fee_lamports\":5198}\n"
            ),
        )
        .expect("sends");
        fs::write(
            path.join("matched-observations.ndjson"),
            concat!(
                "{\"schema_version\":1,\"test_id\":\"paired\",\"signature\":\"sig-tpu\",\"source_name\":\"rpcedge_processed\",\"source_kind\":\"yellowstone_processed\",\"observed_at\":\"2026-01-01T00:00:00.300Z\",\"submitted_at\":\"2026-01-01T00:00:00Z\",\"slot\":101,\"slot_index\":90,\"source_sequence\":null}\n",
                "{\"schema_version\":1,\"test_id\":\"paired\",\"signature\":\"sig-race\",\"source_name\":\"rpcedge_processed\",\"source_kind\":\"yellowstone_processed\",\"observed_at\":\"2026-01-01T00:00:00.100Z\",\"submitted_at\":\"2026-01-01T00:00:00Z\",\"slot\":100,\"slot_index\":10,\"source_sequence\":null}\n"
            ),
        )
        .expect("observations");
        fs::write(
            path.join("leader-runs.ndjson"),
            "{\"schema_version\":1,\"test_id\":\"paired\",\"leader_identity\":\"leader\",\"leader_run_start_slot\":100,\"leader_run_end_slot\":103,\"observed_slot\":100,\"observed_at\":\"2026-01-01T00:00:00Z\",\"eligible\":true,\"network_path_class\":\"doublezero\",\"network_path_verified\":true}\n",
        )
        .expect("leader runs");
    }

    fn write_resend_run(path: &Path) {
        fs::write(
            path.join("samples.ndjson"),
            concat!(
                "{\"schema_version\":5,\"test_id\":\"resend\",\"iteration\":0,\"signature\":\"sig-race\",\"comparison_group_id\":\"group-1\",\"policy_arm\":\"always_race\",\"policy_arm_index\":1,\"provider_name\":\"p\",\"provider_kind\":\"rpcedge_quic_raw_tx\",\"accepted\":true,\"client_started_at\":\"2026-01-01T00:00:00Z\",\"client_finished_at\":\"2026-01-01T00:00:00.001Z\",\"client_ack_latency_us\":1000,\"provider_send_started_at\":\"2026-01-01T00:00:00Z\",\"provider_send_finished_at\":\"2026-01-01T00:00:00.001Z\",\"provider_ack_latency_us\":1000,\"provider_request_id\":null,\"returned_signature\":\"sig-race\",\"status_code\":200,\"error_class\":null,\"error\":null,\"route_policy\":\"always_race\",\"route_mode\":\"only\",\"selected_routes\":[\"tpu_quic\",\"jito_bundle\",\"harmonic_bundle\"],\"leader_client_family\":\"jito\",\"compute_unit_limit\":660,\"compute_unit_price_microlamports\":300000,\"base_fee_lamports\":5000,\"priority_fee_lamports\":198,\"transaction_fee_lamports\":5198,\"resend_attempt_index\":0,\"resend_attempts\":3,\"resend_interval_ms\":50,\"route_results\":[{\"route\":\"jito_bundle\",\"route_name\":\"jito-frankfurt\",\"provider\":\"jito\",\"accepted\":true,\"latency_us\":10,\"error\":null,\"route_tip_signature\":\"tip-landed\",\"route_tip_lamports\":1000,\"route_tip_account\":\"tip\"}]}\n",
                "{\"schema_version\":5,\"test_id\":\"resend\",\"iteration\":0,\"signature\":\"sig-race\",\"comparison_group_id\":\"group-1\",\"policy_arm\":\"always_race\",\"policy_arm_index\":1,\"provider_name\":\"p\",\"provider_kind\":\"rpcedge_quic_raw_tx\",\"accepted\":true,\"client_started_at\":\"2026-01-01T00:00:00Z\",\"client_finished_at\":\"2026-01-01T00:00:00.002Z\",\"client_ack_latency_us\":2000,\"provider_send_started_at\":\"2026-01-01T00:00:00Z\",\"provider_send_finished_at\":\"2026-01-01T00:00:00.002Z\",\"provider_ack_latency_us\":2000,\"provider_request_id\":null,\"returned_signature\":\"sig-race\",\"status_code\":200,\"error_class\":null,\"error\":null,\"route_policy\":\"always_race\",\"route_mode\":\"only\",\"selected_routes\":[\"tpu_quic\",\"jito_bundle\",\"harmonic_bundle\"],\"leader_client_family\":\"jito\",\"compute_unit_limit\":660,\"compute_unit_price_microlamports\":300000,\"base_fee_lamports\":5000,\"priority_fee_lamports\":198,\"transaction_fee_lamports\":5198,\"resend_attempt_index\":1,\"resend_attempts\":3,\"resend_interval_ms\":50,\"route_results\":[{\"route\":\"jito_bundle\",\"route_name\":\"jito-frankfurt\",\"provider\":\"jito\",\"accepted\":true,\"latency_us\":10,\"error\":null,\"route_tip_signature\":\"tip-missed-1\",\"route_tip_lamports\":1000,\"route_tip_account\":\"tip\"}]}\n",
                "{\"schema_version\":5,\"test_id\":\"resend\",\"iteration\":0,\"signature\":\"sig-race\",\"comparison_group_id\":\"group-1\",\"policy_arm\":\"always_race\",\"policy_arm_index\":1,\"provider_name\":\"p\",\"provider_kind\":\"rpcedge_quic_raw_tx\",\"accepted\":true,\"client_started_at\":\"2026-01-01T00:00:00Z\",\"client_finished_at\":\"2026-01-01T00:00:00.003Z\",\"client_ack_latency_us\":3000,\"provider_send_started_at\":\"2026-01-01T00:00:00Z\",\"provider_send_finished_at\":\"2026-01-01T00:00:00.003Z\",\"provider_ack_latency_us\":3000,\"provider_request_id\":null,\"returned_signature\":\"sig-race\",\"status_code\":200,\"error_class\":null,\"error\":null,\"route_policy\":\"always_race\",\"route_mode\":\"only\",\"selected_routes\":[\"tpu_quic\",\"jito_bundle\",\"harmonic_bundle\"],\"leader_client_family\":\"jito\",\"compute_unit_limit\":660,\"compute_unit_price_microlamports\":300000,\"base_fee_lamports\":5000,\"priority_fee_lamports\":198,\"transaction_fee_lamports\":5198,\"resend_attempt_index\":2,\"resend_attempts\":3,\"resend_interval_ms\":50,\"route_results\":[{\"route\":\"jito_bundle\",\"route_name\":\"jito-frankfurt\",\"provider\":\"jito\",\"accepted\":true,\"latency_us\":10,\"error\":null,\"route_tip_signature\":\"tip-missed-2\",\"route_tip_lamports\":1000,\"route_tip_account\":\"tip\"}]}\n",
                "{\"schema_version\":5,\"test_id\":\"resend\",\"iteration\":1,\"signature\":\"sig-tpu\",\"comparison_group_id\":\"group-1\",\"policy_arm\":\"tpu_quic_only\",\"policy_arm_index\":0,\"provider_name\":\"p\",\"provider_kind\":\"rpcedge_quic_raw_tx\",\"accepted\":true,\"client_started_at\":\"2026-01-01T00:00:00Z\",\"client_finished_at\":\"2026-01-01T00:00:00.001Z\",\"client_ack_latency_us\":1000,\"provider_send_started_at\":\"2026-01-01T00:00:00Z\",\"provider_send_finished_at\":\"2026-01-01T00:00:00.001Z\",\"provider_ack_latency_us\":1000,\"provider_request_id\":null,\"returned_signature\":\"sig-tpu\",\"status_code\":200,\"error_class\":null,\"error\":null,\"route_policy\":\"tpu_quic_only\",\"route_mode\":\"only\",\"selected_routes\":[\"tpu_quic\"],\"leader_client_family\":\"jito\",\"compute_unit_limit\":660,\"compute_unit_price_microlamports\":0,\"base_fee_lamports\":5000,\"priority_fee_lamports\":0,\"transaction_fee_lamports\":5000,\"resend_attempt_index\":0,\"resend_attempts\":1,\"resend_interval_ms\":0}\n"
            ),
        )
        .expect("samples");
        fs::write(
            path.join("leader-sends.ndjson"),
            concat!(
                "{\"schema_version\":3,\"test_id\":\"resend\",\"iteration\":0,\"signature\":\"sig-race\",\"comparison_group_id\":\"group-1\",\"policy_arm\":\"always_race\",\"policy_arm_index\":1,\"leader_identity\":\"leader\",\"leader_run_start_slot\":100,\"leader_run_end_slot\":103,\"send_slot\":100,\"sent_at\":\"2026-01-01T00:00:00Z\",\"trigger_source\":\"grpc_slot\",\"slot_signal_status\":null,\"slot_signal_observed_at\":null,\"leader_client_family\":\"jito\",\"route_policy\":\"always_race\",\"selected_routes\":[\"tpu_quic\",\"jito_bundle\",\"harmonic_bundle\"],\"compute_unit_limit\":660,\"compute_unit_price_microlamports\":300000,\"base_fee_lamports\":5000,\"priority_fee_lamports\":198,\"transaction_fee_lamports\":5198}\n",
                "{\"schema_version\":3,\"test_id\":\"resend\",\"iteration\":1,\"signature\":\"sig-tpu\",\"comparison_group_id\":\"group-1\",\"policy_arm\":\"tpu_quic_only\",\"policy_arm_index\":0,\"leader_identity\":\"leader\",\"leader_run_start_slot\":100,\"leader_run_end_slot\":103,\"send_slot\":100,\"sent_at\":\"2026-01-01T00:00:00Z\",\"trigger_source\":\"grpc_slot\",\"slot_signal_status\":null,\"slot_signal_observed_at\":null,\"leader_client_family\":\"jito\",\"route_policy\":\"tpu_quic_only\",\"selected_routes\":[\"tpu_quic\"],\"compute_unit_limit\":660,\"compute_unit_price_microlamports\":0,\"base_fee_lamports\":5000,\"priority_fee_lamports\":0,\"transaction_fee_lamports\":5000}\n"
            ),
        )
        .expect("sends");
        fs::write(
            path.join("matched-observations.ndjson"),
            concat!(
                "{\"schema_version\":1,\"test_id\":\"resend\",\"signature\":\"sig-race\",\"source_name\":\"rpcedge_processed\",\"source_kind\":\"yellowstone_processed\",\"observed_at\":\"2026-01-01T00:00:00.100Z\",\"submitted_at\":\"2026-01-01T00:00:00Z\",\"slot\":100,\"slot_index\":10,\"source_sequence\":null}\n",
                "{\"schema_version\":1,\"test_id\":\"resend\",\"signature\":\"sig-tpu\",\"source_name\":\"rpcedge_processed\",\"source_kind\":\"yellowstone_processed\",\"observed_at\":\"2026-01-01T00:00:00.300Z\",\"submitted_at\":\"2026-01-01T00:00:00Z\",\"slot\":101,\"slot_index\":90,\"source_sequence\":null}\n",
                "{\"schema_version\":1,\"test_id\":\"resend\",\"signature\":\"tip-landed\",\"caller_signature\":\"sig-race\",\"signature_role\":\"route_tip\",\"source_name\":\"rpcedge_processed\",\"source_kind\":\"yellowstone_processed\",\"observed_at\":\"2026-01-01T00:00:00.101Z\",\"submitted_at\":\"2026-01-01T00:00:00Z\",\"slot\":100,\"slot_index\":11,\"source_sequence\":null}\n"
            ),
        )
        .expect("observations");
    }
}
