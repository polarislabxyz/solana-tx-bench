use crate::{
    artifact_io::read_ndjson,
    artifacts::{BenchManifest, BenchSample},
    comparison_report::write_comparison_report,
    cost::priority_fee_lamports,
    leader_paced::LeaderSendEvent,
    observations::ObservationEvent,
    paired_analysis::{analyze_paired_artifact, PairedAnalysis, SenderLocation},
    paired_report::write_paired_report,
};
use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct CompareOptions {
    pub artifact_dirs: Vec<PathBuf>,
    pub labels: Vec<String>,
    pub output_dir: PathBuf,
    pub title: String,
    pub sender_latitude: Option<f64>,
    pub sender_longitude: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonOutput {
    pub schema_version: u32,
    pub title: String,
    pub generated_at: DateTime<Utc>,
    pub run_count: usize,
    pub runs: Vec<RunComparisonSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paired_comparison: Option<PairedAnalysis>,
    pub warnings: Vec<String>,
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
    pub avg_landed_ms: Option<f64>,
    pub p90_landed_ms: Option<f64>,
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
        runs.extend(load_runs(artifact_dir, label)?);
    }
    if runs.len() < 2 {
        bail!("compare needs at least two runs or one paired artifact with multiple policy arms");
    }

    if options.artifact_dirs.len() == 1 {
        apply_scores(&mut runs);
    }

    let sender_location = match (options.sender_latitude, options.sender_longitude) {
        (Some(latitude), Some(longitude)) => Some(SenderLocation {
            latitude,
            longitude,
        }),
        (None, None) => None,
        _ => bail!("--sender-latitude and --sender-longitude must be provided together"),
    };
    let (paired_comparison, paired_rows) = if options.artifact_dirs.len() == 1 {
        analyze_paired_artifact(&options.artifact_dirs[0], sender_location)?
            .map(|(analysis, rows)| (Some(analysis), rows))
            .unwrap_or_else(|| (None, Vec::new()))
    } else {
        (None, Vec::new())
    };

    let generated_at = Utc::now();
    let mut warnings = comparison_warnings(&runs);
    if options.artifact_dirs.len() > 1 {
        warnings.push("Separate artifact directories were collected at different times. Their results are descriptive; no causal A/B score is calculated. Use one paired artifact for route or provider performance claims.".to_string());
    }
    let output = ComparisonOutput {
        schema_version: 2,
        title: options.title.clone(),
        generated_at,
        run_count: runs.len(),
        paired_comparison,
        warnings,
        runs,
    };

    if let Some(paired) = &output.paired_comparison {
        write_paired_report(
            &options.output_dir,
            &options.title,
            generated_at,
            paired,
            &paired_rows,
        )?;
        return Ok(output);
    }

    write_comparison_report(&options.output_dir, &output)?;

    Ok(output)
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

fn load_runs(artifact_dir: &Path, label: Option<String>) -> Result<Vec<RunComparisonSummary>> {
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
                load_run_from_inputs(artifact_dir, Some(arm_label), inputs)
            })
            .collect();
    }

    load_run_from_inputs(
        artifact_dir,
        label,
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

    let primary = source_stats(&caller_observations, &send_by_signature, "first_observed");
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
        avg_landed_ms: avg(&primary.latencies_ms),
        p90_landed_ms: percentile_f64(&primary.latencies_ms, 0.90),
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

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
            title: "Test Compare".to_string(),
            sender_latitude: None,
            sender_longitude: None,
        })
        .expect("comparison");

        assert_eq!(comparison.run_count, 2);
        assert_eq!(comparison.runs[0].performance_rate_pct, 0.0);
        assert_eq!(comparison.runs[1].performance_rate_pct, 0.0);
        assert!(comparison
            .warnings
            .iter()
            .any(|warning| warning.contains("no causal A/B score")));
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
            title: "Paired Compare".to_string(),
            sender_latitude: None,
            sender_longitude: None,
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
        assert_eq!(paired.landing_coverage.comparison_groups, 1);
        assert_eq!(paired.landing_coverage.different_slot_pairs, 1);
        assert_eq!(paired.same_slot_metrics.first_observed_ms.pairs, 0);
        assert_eq!(
            paired.same_slot_metrics.processed_transaction_index.pairs,
            0
        );
        assert_eq!(paired.same_slot_metrics.entry_index.pairs, 0);
        assert_eq!(paired.same_slot_metrics.preceding_poh_ticks.pairs, 0);
        assert!(output_dir.join("paired-rows.json").exists());
        assert!(output_dir.join("summary.json").exists());
        assert!(output_dir.join("index.html").exists());
        assert!(!output_dir.join("comparison.md").exists());
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
            title: "Resend Compare".to_string(),
            sender_latitude: None,
            sender_longitude: None,
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
