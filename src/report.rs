use crate::{
    artifacts::BenchSample,
    leader_paced::LeaderSendEvent,
    leader_slots::LeaderSlotsSnapshotArtifact,
    observations::{EntryPlacementEvent, ObservationEvent},
};
use anyhow::{Context, Result};
use chrono::{DateTime, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportOutput {
    pub schema_version: u32,
    pub test_id: String,
    pub generated_at: DateTime<Utc>,
    pub artifact_dir: String,
    pub totals: ReportTotals,
    pub provider_summaries: Vec<ProviderReportSummary>,
    pub source_summaries: Vec<SourceReportSummary>,
    pub cost_summary: CostReportSummary,
    pub cohort_summaries: Vec<CohortReportSummary>,
    pub tail_events: Vec<TailEvent>,
    pub unobserved_events: Vec<UnobservedEvent>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportTotals {
    pub samples: usize,
    pub sent_transactions: usize,
    pub observations: usize,
    pub matched_signatures: usize,
    pub unobserved_signatures: usize,
    pub matched_route_tip_signatures: usize,
    pub leader_slot_snapshots: usize,
    pub entry_placements: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CostReportSummary {
    pub submitted_transactions: usize,
    pub total_base_fee_lamports: u64,
    pub total_priority_fee_lamports: u64,
    pub total_transaction_fee_lamports: u64,
    pub total_route_tip_generated_lamports: u64,
    pub total_route_tip_landed_lamports: u64,
    pub total_generated_spend_lamports: u64,
    pub total_landed_spend_lamports: u64,
    pub avg_base_fee_lamports: Option<f64>,
    pub avg_priority_fee_lamports: Option<f64>,
    pub avg_transaction_fee_lamports: Option<f64>,
    pub avg_route_tip_generated_lamports: Option<f64>,
    pub avg_route_tip_landed_lamports: Option<f64>,
    pub avg_total_generated_spend_lamports: Option<f64>,
    pub avg_total_landed_spend_lamports: Option<f64>,
    pub generated_route_tip_count: usize,
    pub landed_route_tip_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderReportSummary {
    pub provider_name: String,
    pub route_policy: String,
    pub selected_routes: Vec<String>,
    pub count: usize,
    pub accepted: usize,
    pub errors: usize,
    pub ack_p50_ms: Option<f64>,
    pub ack_p90_ms: Option<f64>,
    pub ack_p99_ms: Option<f64>,
    pub ack_max_ms: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceReportSummary {
    pub source_name: String,
    pub count: usize,
    pub first_seen: usize,
    pub submit_to_observed_p50_ms: Option<f64>,
    pub submit_to_observed_p90_ms: Option<f64>,
    pub submit_to_observed_p99_ms: Option<f64>,
    pub submit_to_observed_max_ms: Option<f64>,
    pub same_slot_count: usize,
    pub same_slot_rate: f64,
    pub landed_slot_delta_p50: Option<i64>,
    pub landed_slot_delta_p90: Option<i64>,
    pub slot_index_p50: Option<u64>,
    pub slot_index_p90: Option<u64>,
    pub entry_index_p50: Option<u64>,
    pub entry_index_p90: Option<u64>,
    pub tx_index_in_entry_p50: Option<u64>,
    pub tx_index_in_entry_p90: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CohortReportSummary {
    pub cohort_kind: String,
    pub cohort_value: String,
    pub source_name: String,
    pub count: usize,
    pub submit_to_observed_p50_ms: Option<f64>,
    pub submit_to_observed_p90_ms: Option<f64>,
    pub submit_to_observed_p99_ms: Option<f64>,
    pub submit_to_observed_max_ms: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TailEvent {
    pub signature: String,
    pub source_name: String,
    pub latency_ms: f64,
    pub policy_arm: Option<String>,
    pub route_policy: Option<String>,
    pub selected_routes: Vec<String>,
    pub comparison_group_id: Option<String>,
    pub sent_at: Option<DateTime<Utc>>,
    pub send_slot: Option<u64>,
    pub landed_slot: Option<u64>,
    pub slot_index: Option<u64>,
    pub first_observed_slot_index: Option<u64>,
    pub processed_slot_index: Option<u64>,
    pub entry_index: Option<u64>,
    pub tx_index_in_entry: Option<u64>,
    pub entry_executed_transaction_count: Option<u64>,
    pub send_leader_identity: String,
    pub send_leader_region: String,
    pub send_leader_city: String,
    pub send_leader_client: String,
    pub landed_leader_identity: String,
    pub landed_leader_region: String,
    pub landed_leader_city: String,
    pub landed_leader_client: String,
    pub compute_unit_price_microlamports: Option<u64>,
    pub base_fee_lamports: Option<u64>,
    pub priority_fee_lamports: Option<u64>,
    pub transaction_fee_lamports: Option<u64>,
    pub route_tip_generated_lamports: Option<u64>,
    pub route_tip_landed_lamports: Option<u64>,
    pub total_generated_spend_lamports: Option<u64>,
    pub total_landed_spend_lamports: Option<u64>,
    pub leader_identity: String,
    pub leader_region: String,
    pub leader_city: String,
    pub leader_client: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnobservedEvent {
    pub signature: String,
    pub policy_arm: Option<String>,
    pub route_policy: Option<String>,
    pub selected_routes: Vec<String>,
    pub comparison_group_id: Option<String>,
    pub sent_at: DateTime<Utc>,
    pub send_slot: u64,
    pub leader_run_start_slot: u64,
    pub leader_run_end_slot: u64,
    pub trigger_source: String,
    pub slot_signal_status: Option<String>,
    pub send_leader_identity: String,
    pub send_leader_region: String,
    pub send_leader_city: String,
    pub send_leader_client: String,
    pub provider_name: Option<String>,
    pub provider_attempts: usize,
    pub provider_accepted_attempts: usize,
    pub provider_ack_latency_us_min: Option<u128>,
    pub provider_ack_latency_us_max: Option<u128>,
    pub provider_accepted: Option<bool>,
    pub provider_ack_latency_us: Option<u128>,
    pub provider_request_id: Option<String>,
    pub status_code: Option<u16>,
    pub error_class: Option<String>,
    pub error: Option<String>,
    pub route_results_complete: Option<bool>,
    pub route_results: Vec<crate::adapters::RouteResultAck>,
    pub compute_unit_price_microlamports: Option<u64>,
    pub base_fee_lamports: Option<u64>,
    pub priority_fee_lamports: Option<u64>,
    pub transaction_fee_lamports: Option<u64>,
    pub route_tip_generated_lamports: Option<u64>,
    pub total_generated_spend_lamports: Option<u64>,
}

#[derive(Debug, Clone, Default)]
struct LeaderMeta {
    identity: String,
    region: String,
    city: String,
    data_center_key: String,
    client_family: String,
    client_software: String,
    stake_bucket: String,
}

#[derive(Debug, Clone)]
struct ObservationRow {
    event: ObservationEvent,
    send: Option<LeaderSendEvent>,
    latency_us: Option<i128>,
    leader: LeaderMeta,
    placement: Option<EntryPlacementEvent>,
}

pub fn generate_report(artifact_dir: &Path) -> Result<ReportOutput> {
    let samples: Vec<BenchSample> = read_ndjson(&artifact_dir.join("samples.ndjson"))?;
    let sends: Vec<LeaderSendEvent> = read_ndjson(&artifact_dir.join("leader-sends.ndjson"))?;
    let observations: Vec<ObservationEvent> =
        read_ndjson(&artifact_dir.join("matched-observations.ndjson"))?;
    let entry_placements: Vec<EntryPlacementEvent> =
        read_ndjson(&artifact_dir.join("entry-placements.ndjson"))?;
    let manifest = read_manifest(artifact_dir)?;
    let snapshots = read_leader_slot_snapshots(artifact_dir)?;

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

    let leader_by_slot = build_leader_lookup(&snapshots);
    let send_by_signature = sends
        .iter()
        .cloned()
        .map(|send| (send.signature.clone(), send))
        .collect::<BTreeMap<_, _>>();
    let mut samples_by_signature = BTreeMap::<String, Vec<BenchSample>>::new();
    for sample in samples.iter().cloned() {
        samples_by_signature
            .entry(sample.signature.clone())
            .or_default()
            .push(sample);
    }
    let sample_by_signature = samples_by_signature
        .iter()
        .filter_map(|(signature, samples)| {
            samples
                .first()
                .cloned()
                .map(|sample| (signature.clone(), sample))
        })
        .collect::<BTreeMap<_, _>>();
    let caller_observations = observations
        .iter()
        .filter(|event| !is_route_tip_observation(event))
        .cloned()
        .collect::<Vec<_>>();
    let route_tip_observations = observations
        .iter()
        .filter(|event| is_route_tip_observation(event))
        .cloned()
        .collect::<Vec<_>>();
    let caller_observed_signatures = caller_observations
        .iter()
        .map(|event| event.signature.clone())
        .collect::<BTreeSet<_>>();
    let observed_signatures = observations
        .iter()
        .map(|event| event.signature.clone())
        .collect::<BTreeSet<_>>();
    let cost_by_signature = cost_rows_by_signature(&samples, &sends, &observed_signatures);
    let processed_slot_index_by_signature = build_processed_slot_index_lookup(&caller_observations);
    let placement_by_signature = build_entry_placement_lookup(&entry_placements);

    let rows = caller_observations
        .iter()
        .cloned()
        .map(|event| {
            let send = send_by_signature.get(&event.signature).cloned();
            let leader = event
                .slot
                .and_then(|slot| leader_by_slot.get(&slot).cloned())
                .or_else(|| {
                    send.as_ref().and_then(|send| {
                        leader_by_slot
                            .get(&send.send_slot)
                            .cloned()
                            .or_else(|| leader_by_slot.get(&send.leader_run_start_slot).cloned())
                    })
                })
                .unwrap_or_else(|| {
                    let mut meta = LeaderMeta::default();
                    if let Some(send) = &send {
                        meta.identity = send.leader_identity.clone();
                        meta.client_family = send
                            .leader_client_family
                            .clone()
                            .unwrap_or_else(|| "unknown".to_string());
                    }
                    normalize_meta(meta)
                });
            let latency_us = event.submitted_at.and_then(|submitted_at| {
                event
                    .observed_at
                    .signed_duration_since(submitted_at)
                    .num_microseconds()
                    .map(|value| value as i128)
            });
            ObservationRow {
                placement: placement_by_signature.get(&event.signature).cloned(),
                event,
                send,
                latency_us,
                leader,
            }
        })
        .collect::<Vec<_>>();
    let landing_rows = first_observed_rows(&rows);

    let report = ReportOutput {
        schema_version: 1,
        test_id,
        generated_at: Utc::now(),
        artifact_dir: artifact_dir.display().to_string(),
        totals: ReportTotals {
            samples: samples.len(),
            sent_transactions: sends.len(),
            observations: observations.len(),
            matched_signatures: caller_observed_signatures.len(),
            unobserved_signatures: sends
                .iter()
                .filter(|send| !caller_observed_signatures.contains(&send.signature))
                .count(),
            matched_route_tip_signatures: route_tip_observations
                .iter()
                .map(|event| event.signature.as_str())
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            leader_slot_snapshots: snapshots.len(),
            entry_placements: entry_placements.len(),
        },
        provider_summaries: summarize_providers(&samples, manifest.as_ref()),
        source_summaries: summarize_sources(&landing_rows),
        cost_summary: summarize_costs(&cost_by_signature),
        cohort_summaries: summarize_cohorts(&landing_rows),
        tail_events: summarize_tail_events(
            &landing_rows,
            &sample_by_signature,
            &cost_by_signature,
            &processed_slot_index_by_signature,
            &leader_by_slot,
            12,
        ),
        unobserved_events: summarize_unobserved_events(
            &sends,
            &samples_by_signature,
            &caller_observed_signatures,
            &cost_by_signature,
            &leader_by_slot,
        ),
        warnings: report_warnings(&samples, &sends),
    };

    fs::write(
        artifact_dir.join("report.json"),
        serde_json::to_vec_pretty(&report)?,
    )
    .with_context(|| format!("write {}", artifact_dir.join("report.json").display()))?;
    fs::write(artifact_dir.join("report.md"), report_markdown(&report))
        .with_context(|| format!("write {}", artifact_dir.join("report.md").display()))?;
    fs::write(artifact_dir.join("report.html"), report_html(&report))
        .with_context(|| format!("write {}", artifact_dir.join("report.html").display()))?;
    write_tail_artifacts(artifact_dir, &report.tail_events)?;
    write_unobserved_artifacts(artifact_dir, &report.unobserved_events)?;

    Ok(report)
}

fn report_warnings(samples: &[BenchSample], sends: &[LeaderSendEvent]) -> Vec<String> {
    let mut warnings = Vec::new();
    let send_cu_prices = sends
        .iter()
        .map(|send| send.compute_unit_price_microlamports)
        .collect::<BTreeSet<_>>();
    let sample_cu_prices = samples
        .iter()
        .map(|sample| sample.compute_unit_price_microlamports)
        .collect::<BTreeSet<_>>();

    if send_cu_prices.len() > 1 || sample_cu_prices.len() > 1 {
        warnings.push(
            "This artifact used route-dependent priority fees. Treat it as a cost-aware route-strategy comparison, not a fee-neutral transport isolation run.".to_string(),
        );
    }

    let has_jito_route = samples.iter().any(|sample| {
        sample
            .selected_routes
            .iter()
            .any(|route| route == "jito_bundle")
    });
    if has_jito_route {
        let has_route_tip = samples.iter().any(|sample| {
            sample
                .route_results
                .iter()
                .any(|result| result.route_tip_lamports.unwrap_or(0) > 0)
        });
        if !has_route_tip {
            warnings.push(
                "This artifact includes jito_bundle route attempts, but no route-tip telemetry was present. Route-tip generated and landed cost will be zero/unknown for those attempts.".to_string(),
            );
        }
    }

    warnings
}

fn first_observed_rows(rows: &[ObservationRow]) -> Vec<ObservationRow> {
    let mut first_by_signature = BTreeMap::<String, &ObservationRow>::new();
    for row in rows {
        first_by_signature
            .entry(row.event.signature.clone())
            .and_modify(|existing| {
                if row.event.observed_at < existing.event.observed_at {
                    *existing = row;
                }
            })
            .or_insert(row);
    }
    first_by_signature.into_values().cloned().collect()
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

fn read_leader_slot_snapshots(artifact_dir: &Path) -> Result<Vec<LeaderSlotsSnapshotArtifact>> {
    let mut paths = fs::read_dir(artifact_dir)
        .with_context(|| format!("read {}", artifact_dir.display()))?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| {
                    name == "leader-slots-snapshot.json"
                        || name.starts_with("leader-slots-snapshot-")
                })
                .unwrap_or(false)
        })
        .collect::<Vec<PathBuf>>();
    paths.sort();

    let mut snapshots = Vec::new();
    for path in paths {
        let bytes = fs::read(&path).with_context(|| format!("read {}", path.display()))?;
        snapshots.push(
            serde_json::from_slice::<LeaderSlotsSnapshotArtifact>(&bytes)
                .with_context(|| format!("parse {}", path.display()))?,
        );
    }
    Ok(snapshots)
}

fn read_manifest(artifact_dir: &Path) -> Result<Option<crate::artifacts::BenchManifest>> {
    let path = artifact_dir.join("manifest.json");
    if !path.exists() {
        return Ok(None);
    }
    let bytes = fs::read(&path).with_context(|| format!("read {}", path.display()))?;
    Ok(Some(
        serde_json::from_slice::<crate::artifacts::BenchManifest>(&bytes)
            .with_context(|| format!("parse {}", path.display()))?,
    ))
}

fn build_processed_slot_index_lookup(events: &[ObservationEvent]) -> BTreeMap<String, u64> {
    let mut rows = events
        .iter()
        .filter_map(|event| {
            let slot_index = event.slot_index?;
            let source_name = event.source_name.to_ascii_lowercase();
            let is_processed = event.source_kind
                == crate::observations::ObservationSourceKind::YellowstoneProcessed
                || source_name.contains("processed");
            is_processed.then_some((event.signature.clone(), event.observed_at, slot_index))
        })
        .collect::<Vec<_>>();
    rows.sort_by_key(|(_, observed_at, _)| *observed_at);

    let mut out = BTreeMap::new();
    for (signature, _, slot_index) in rows {
        out.entry(signature).or_insert(slot_index);
    }
    out
}

fn build_entry_placement_lookup(
    placements: &[EntryPlacementEvent],
) -> BTreeMap<String, EntryPlacementEvent> {
    let mut rows = placements
        .iter()
        .cloned()
        .map(|placement| {
            (
                placement.signature.clone(),
                placement.observed_at,
                placement,
            )
        })
        .collect::<Vec<_>>();
    rows.sort_by_key(|(_, observed_at, _)| *observed_at);

    let mut out = BTreeMap::new();
    for (signature, _, placement) in rows {
        out.entry(signature).or_insert(placement);
    }
    out
}

fn build_leader_lookup(snapshots: &[LeaderSlotsSnapshotArtifact]) -> BTreeMap<u64, LeaderMeta> {
    let mut out = BTreeMap::new();
    for snapshot in snapshots {
        let rows = snapshot
            .response
            .get("result")
            .and_then(|result| result.get("data"))
            .and_then(Value::as_array)
            .or_else(|| snapshot.response.get("data").and_then(Value::as_array));
        let Some(rows) = rows else {
            continue;
        };
        for row in rows {
            let Some(slot) = row.get("slot").and_then(Value::as_u64) else {
                continue;
            };
            out.entry(slot).or_insert_with(|| meta_from_leader_row(row));
        }
    }
    out
}

fn meta_from_leader_row(row: &Value) -> LeaderMeta {
    let identity = row
        .get("identity")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    let location = row.get("location").unwrap_or(&Value::Null);
    let client = row.get("client").unwrap_or(&Value::Null);
    normalize_meta(LeaderMeta {
        identity,
        region: string_field(location, "region"),
        city: string_field(location, "city"),
        data_center_key: string_field(location, "dataCenterKey"),
        client_family: string_field(client, "family"),
        client_software: string_field(client, "software"),
        stake_bucket: row
            .get("stake")
            .and_then(Value::as_u64)
            .map(stake_bucket)
            .unwrap_or_else(|| "unknown".to_string()),
    })
}

fn string_field(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string()
}

fn normalize_meta(mut meta: LeaderMeta) -> LeaderMeta {
    if meta.identity.is_empty() {
        meta.identity = "unknown".to_string();
    }
    if meta.region.is_empty() {
        meta.region = "unknown".to_string();
    }
    if meta.city.is_empty() {
        meta.city = "unknown".to_string();
    }
    if meta.data_center_key.is_empty() {
        meta.data_center_key = "unknown".to_string();
    }
    if meta.client_family.is_empty() {
        meta.client_family = "unknown".to_string();
    }
    if meta.client_software.is_empty() {
        meta.client_software = meta.client_family.clone();
    }
    if meta.stake_bucket.is_empty() {
        meta.stake_bucket = "unknown".to_string();
    }
    meta
}

fn stake_bucket(stake_lamports: u64) -> String {
    let sol = stake_lamports / 1_000_000_000;
    if sol >= 1_000_000 {
        ">=1m_sol".to_string()
    } else if sol >= 500_000 {
        "500k_1m_sol".to_string()
    } else if sol >= 100_000 {
        "100k_500k_sol".to_string()
    } else if sol > 0 {
        "<100k_sol".to_string()
    } else {
        "unknown".to_string()
    }
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
    generated_route_tip_count: usize,
    landed_route_tip_count: usize,
}

fn cost_rows_by_signature(
    samples: &[BenchSample],
    sends: &[LeaderSendEvent],
    observed_signatures: &BTreeSet<String>,
) -> BTreeMap<String, CostRow> {
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
                    crate::cost::priority_fee_lamports(limit, price)
                });
            let transaction_fee_lamports = send
                .map(|send| send.transaction_fee_lamports)
                .or_else(|| sample.map(|sample| sample.transaction_fee_lamports))
                .unwrap_or_else(|| base_fee_lamports.saturating_add(priority_fee_lamports));

            let mut route_tip_generated_lamports = 0_u64;
            let mut route_tip_landed_lamports = 0_u64;
            let mut generated_route_tip_count = 0_usize;
            let mut landed_route_tip_count = 0_usize;
            if let Some(samples) = sample_by_signature.get(&signature) {
                for sample in samples {
                    for result in &sample.route_results {
                        let lamports = result.route_tip_lamports.unwrap_or(0);
                        if lamports == 0 {
                            continue;
                        }
                        generated_route_tip_count += 1;
                        route_tip_generated_lamports =
                            route_tip_generated_lamports.saturating_add(lamports);
                        if result
                            .route_tip_signature
                            .as_ref()
                            .map(|signature| observed_signatures.contains(signature))
                            .unwrap_or(false)
                        {
                            landed_route_tip_count += 1;
                            route_tip_landed_lamports =
                                route_tip_landed_lamports.saturating_add(lamports);
                        }
                    }
                }
            }

            (
                signature,
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
                    generated_route_tip_count,
                    landed_route_tip_count,
                },
            )
        })
        .collect()
}

fn summarize_costs(cost_by_signature: &BTreeMap<String, CostRow>) -> CostReportSummary {
    let mut out = CostReportSummary {
        submitted_transactions: cost_by_signature.len(),
        ..CostReportSummary::default()
    };
    for cost in cost_by_signature.values() {
        out.total_base_fee_lamports = out
            .total_base_fee_lamports
            .saturating_add(cost.base_fee_lamports);
        out.total_priority_fee_lamports = out
            .total_priority_fee_lamports
            .saturating_add(cost.priority_fee_lamports);
        out.total_transaction_fee_lamports = out
            .total_transaction_fee_lamports
            .saturating_add(cost.transaction_fee_lamports);
        out.total_route_tip_generated_lamports = out
            .total_route_tip_generated_lamports
            .saturating_add(cost.route_tip_generated_lamports);
        out.total_route_tip_landed_lamports = out
            .total_route_tip_landed_lamports
            .saturating_add(cost.route_tip_landed_lamports);
        out.total_generated_spend_lamports = out
            .total_generated_spend_lamports
            .saturating_add(cost.total_generated_spend_lamports);
        out.total_landed_spend_lamports = out
            .total_landed_spend_lamports
            .saturating_add(cost.total_landed_spend_lamports);
        out.generated_route_tip_count += cost.generated_route_tip_count;
        out.landed_route_tip_count += cost.landed_route_tip_count;
    }

    let count = out.submitted_transactions as f64;
    if count > 0.0 {
        out.avg_base_fee_lamports = Some(out.total_base_fee_lamports as f64 / count);
        out.avg_priority_fee_lamports = Some(out.total_priority_fee_lamports as f64 / count);
        out.avg_transaction_fee_lamports = Some(out.total_transaction_fee_lamports as f64 / count);
        out.avg_route_tip_generated_lamports =
            Some(out.total_route_tip_generated_lamports as f64 / count);
        out.avg_route_tip_landed_lamports =
            Some(out.total_route_tip_landed_lamports as f64 / count);
        out.avg_total_generated_spend_lamports =
            Some(out.total_generated_spend_lamports as f64 / count);
        out.avg_total_landed_spend_lamports = Some(out.total_landed_spend_lamports as f64 / count);
    }
    out
}

fn is_route_tip_observation(event: &ObservationEvent) -> bool {
    event.signature_role.as_deref() == Some("route_tip")
}

fn summarize_providers(
    samples: &[BenchSample],
    manifest: Option<&crate::artifacts::BenchManifest>,
) -> Vec<ProviderReportSummary> {
    let provider_manifest = manifest
        .map(|manifest| {
            manifest
                .providers
                .iter()
                .map(|provider| (provider.name.as_str(), provider))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    let mut groups = BTreeMap::<(String, String, Vec<String>), Vec<&BenchSample>>::new();
    for sample in samples {
        let manifest_provider = provider_manifest.get(sample.provider_name.as_str());
        let route_policy = sample
            .route_policy
            .clone()
            .or_else(|| manifest_provider.and_then(|provider| provider.route_mode.clone()))
            .unwrap_or_else(|| "static".to_string());
        let selected_routes = if !sample.selected_routes.is_empty() {
            sample.selected_routes.clone()
        } else {
            manifest_provider
                .map(|provider| provider.routes.clone())
                .unwrap_or_default()
        };
        groups
            .entry((sample.provider_name.clone(), route_policy, selected_routes))
            .or_default()
            .push(sample);
    }
    groups
        .into_iter()
        .map(|((provider_name, route_policy, selected_routes), rows)| {
            let mut latencies = rows
                .iter()
                .filter(|row| row.accepted)
                .map(|row| row.provider_ack_latency_us as i128)
                .collect::<Vec<_>>();
            latencies.sort_unstable();
            ProviderReportSummary {
                provider_name,
                route_policy,
                selected_routes,
                count: rows.len(),
                accepted: latencies.len(),
                errors: rows.len().saturating_sub(latencies.len()),
                ack_p50_ms: percentile_ms(&latencies, 0.50),
                ack_p90_ms: percentile_ms(&latencies, 0.90),
                ack_p99_ms: percentile_ms(&latencies, 0.99),
                ack_max_ms: percentile_ms(&latencies, 1.0),
            }
        })
        .collect()
}

fn summarize_sources(rows: &[ObservationRow]) -> Vec<SourceReportSummary> {
    let mut first_seen_by_signature = BTreeMap::<String, DateTime<Utc>>::new();
    for row in rows {
        first_seen_by_signature
            .entry(row.event.signature.clone())
            .and_modify(|existing| {
                if row.event.observed_at < *existing {
                    *existing = row.event.observed_at;
                }
            })
            .or_insert(row.event.observed_at);
    }

    let mut groups = BTreeMap::<String, Vec<&ObservationRow>>::new();
    for row in rows {
        groups
            .entry(row.event.source_name.clone())
            .or_default()
            .push(row);
    }
    groups
        .into_iter()
        .map(|(source_name, rows)| {
            let mut latencies = rows
                .iter()
                .filter_map(|row| row.latency_us)
                .collect::<Vec<_>>();
            latencies.sort_unstable();
            let mut slot_deltas = rows
                .iter()
                .filter_map(|row| {
                    Some(row.event.slot? as i64 - row.send.as_ref()?.send_slot as i64)
                })
                .collect::<Vec<_>>();
            slot_deltas.sort_unstable();
            let same_slot_count = slot_deltas.iter().filter(|delta| **delta == 0).count();
            let mut slot_indexes = rows
                .iter()
                .filter_map(|row| row.event.slot_index)
                .collect::<Vec<_>>();
            slot_indexes.sort_unstable();
            let mut entry_indexes = rows
                .iter()
                .filter_map(|row| {
                    row.placement
                        .as_ref()
                        .map(|placement| placement.entry_index)
                })
                .collect::<Vec<_>>();
            entry_indexes.sort_unstable();
            let mut tx_indexes_in_entry = rows
                .iter()
                .filter_map(|row| {
                    row.placement
                        .as_ref()
                        .map(|placement| placement.tx_index_in_entry)
                })
                .collect::<Vec<_>>();
            tx_indexes_in_entry.sort_unstable();
            let first_seen = rows
                .iter()
                .filter(|row| {
                    first_seen_by_signature.get(&row.event.signature)
                        == Some(&row.event.observed_at)
                })
                .count();
            SourceReportSummary {
                source_name,
                count: rows.len(),
                first_seen,
                submit_to_observed_p50_ms: percentile_ms(&latencies, 0.50),
                submit_to_observed_p90_ms: percentile_ms(&latencies, 0.90),
                submit_to_observed_p99_ms: percentile_ms(&latencies, 0.99),
                submit_to_observed_max_ms: percentile_ms(&latencies, 1.0),
                same_slot_count,
                same_slot_rate: ratio(same_slot_count, slot_deltas.len()),
                landed_slot_delta_p50: percentile_i64(&slot_deltas, 0.50),
                landed_slot_delta_p90: percentile_i64(&slot_deltas, 0.90),
                slot_index_p50: percentile_u64(&slot_indexes, 0.50),
                slot_index_p90: percentile_u64(&slot_indexes, 0.90),
                entry_index_p50: percentile_u64(&entry_indexes, 0.50),
                entry_index_p90: percentile_u64(&entry_indexes, 0.90),
                tx_index_in_entry_p50: percentile_u64(&tx_indexes_in_entry, 0.50),
                tx_index_in_entry_p90: percentile_u64(&tx_indexes_in_entry, 0.90),
            }
        })
        .collect()
}

fn summarize_cohorts(rows: &[ObservationRow]) -> Vec<CohortReportSummary> {
    let mut groups = BTreeMap::<(String, String, String), Vec<i128>>::new();
    for row in rows {
        let Some(latency) = row.latency_us else {
            continue;
        };
        for (kind, value) in [
            ("leader_region", row.leader.region.as_str()),
            ("leader_city", row.leader.city.as_str()),
            ("data_center_key", row.leader.data_center_key.as_str()),
            ("client_family", row.leader.client_family.as_str()),
            ("client_software", row.leader.client_software.as_str()),
            ("stake_bucket", row.leader.stake_bucket.as_str()),
        ] {
            groups
                .entry((
                    kind.to_string(),
                    value.to_string(),
                    row.event.source_name.clone(),
                ))
                .or_default()
                .push(latency);
        }
    }
    let mut summaries = groups
        .into_iter()
        .map(
            |((cohort_kind, cohort_value, source_name), mut latencies)| {
                latencies.sort_unstable();
                CohortReportSummary {
                    cohort_kind,
                    cohort_value,
                    source_name,
                    count: latencies.len(),
                    submit_to_observed_p50_ms: percentile_ms(&latencies, 0.50),
                    submit_to_observed_p90_ms: percentile_ms(&latencies, 0.90),
                    submit_to_observed_p99_ms: percentile_ms(&latencies, 0.99),
                    submit_to_observed_max_ms: percentile_ms(&latencies, 1.0),
                }
            },
        )
        .collect::<Vec<_>>();
    summaries.sort_by(|a, b| {
        a.cohort_kind
            .cmp(&b.cohort_kind)
            .then_with(|| b.count.cmp(&a.count))
            .then_with(|| a.cohort_value.cmp(&b.cohort_value))
            .then_with(|| a.source_name.cmp(&b.source_name))
    });
    summaries
}

fn summarize_tail_events(
    rows: &[ObservationRow],
    sample_by_signature: &BTreeMap<String, BenchSample>,
    cost_by_signature: &BTreeMap<String, CostRow>,
    processed_slot_index_by_signature: &BTreeMap<String, u64>,
    leader_by_slot: &BTreeMap<u64, LeaderMeta>,
    limit: usize,
) -> Vec<TailEvent> {
    let mut rows = rows
        .iter()
        .filter_map(|row| {
            let latency_us = row.latency_us?;
            Some((latency_us, row))
        })
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| b.0.cmp(&a.0));
    rows.into_iter()
        .take(limit)
        .map(|(latency_us, row)| {
            let sample = sample_by_signature.get(&row.event.signature);
            let send = row.send.as_ref();
            let send_leader = send_leader_meta(send, leader_by_slot);
            let processed_slot_index = processed_slot_index_by_signature
                .get(&row.event.signature)
                .copied();
            let slot_index = processed_slot_index.or(row.event.slot_index);
            let placement = row.placement.as_ref();
            let policy_arm = sample
                .and_then(|sample| sample.policy_arm.clone())
                .or_else(|| send.and_then(|send| send.policy_arm.clone()));
            let route_policy = sample
                .and_then(|sample| sample.route_policy.clone())
                .or_else(|| send.and_then(|send| send.route_policy.clone()));
            let selected_routes = sample
                .map(|sample| sample.selected_routes.clone())
                .filter(|routes| !routes.is_empty())
                .or_else(|| {
                    send.map(|send| send.selected_routes.clone())
                        .filter(|routes| !routes.is_empty())
                })
                .unwrap_or_default();
            let comparison_group_id = sample
                .and_then(|sample| sample.comparison_group_id.clone())
                .or_else(|| send.and_then(|send| send.comparison_group_id.clone()));
            let compute_unit_price_microlamports = sample
                .map(|sample| sample.compute_unit_price_microlamports)
                .or_else(|| send.map(|send| send.compute_unit_price_microlamports));
            let cost = cost_by_signature.get(&row.event.signature);

            TailEvent {
                signature: row.event.signature.clone(),
                source_name: row.event.source_name.clone(),
                latency_ms: latency_us as f64 / 1000.0,
                policy_arm,
                route_policy,
                selected_routes,
                comparison_group_id,
                sent_at: send.map(|send| send.sent_at),
                send_slot: send.map(|send| send.send_slot),
                landed_slot: row.event.slot,
                slot_index,
                first_observed_slot_index: row.event.slot_index,
                processed_slot_index,
                entry_index: placement.map(|placement| placement.entry_index),
                tx_index_in_entry: placement.map(|placement| placement.tx_index_in_entry),
                entry_executed_transaction_count: placement
                    .map(|placement| placement.entry_executed_transaction_count),
                send_leader_identity: send_leader.identity.clone(),
                send_leader_region: send_leader.region.clone(),
                send_leader_city: send_leader.city.clone(),
                send_leader_client: send_leader.client_software.clone(),
                landed_leader_identity: row.leader.identity.clone(),
                landed_leader_region: row.leader.region.clone(),
                landed_leader_city: row.leader.city.clone(),
                landed_leader_client: row.leader.client_software.clone(),
                compute_unit_price_microlamports,
                base_fee_lamports: cost.map(|cost| cost.base_fee_lamports),
                priority_fee_lamports: cost.map(|cost| cost.priority_fee_lamports),
                transaction_fee_lamports: cost.map(|cost| cost.transaction_fee_lamports),
                route_tip_generated_lamports: cost.map(|cost| cost.route_tip_generated_lamports),
                route_tip_landed_lamports: cost.map(|cost| cost.route_tip_landed_lamports),
                total_generated_spend_lamports: cost
                    .map(|cost| cost.total_generated_spend_lamports),
                total_landed_spend_lamports: cost.map(|cost| cost.total_landed_spend_lamports),
                leader_identity: row.leader.identity.clone(),
                leader_region: row.leader.region.clone(),
                leader_city: row.leader.city.clone(),
                leader_client: row.leader.client_software.clone(),
            }
        })
        .collect()
}

fn summarize_unobserved_events(
    sends: &[LeaderSendEvent],
    samples_by_signature: &BTreeMap<String, Vec<BenchSample>>,
    caller_observed_signatures: &BTreeSet<String>,
    cost_by_signature: &BTreeMap<String, CostRow>,
    leader_by_slot: &BTreeMap<u64, LeaderMeta>,
) -> Vec<UnobservedEvent> {
    let mut out = sends
        .iter()
        .filter(|send| !caller_observed_signatures.contains(&send.signature))
        .map(|send| {
            let signature_samples = samples_by_signature
                .get(&send.signature)
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            let sample = signature_samples.first();
            let send_leader = send_leader_meta(Some(send), leader_by_slot);
            let cost = cost_by_signature.get(&send.signature);
            let policy_arm = sample
                .and_then(|sample| sample.policy_arm.clone())
                .or_else(|| send.policy_arm.clone());
            let route_policy = sample
                .and_then(|sample| sample.route_policy.clone())
                .or_else(|| send.route_policy.clone());
            let selected_routes = sample
                .map(|sample| sample.selected_routes.clone())
                .filter(|routes| !routes.is_empty())
                .unwrap_or_else(|| send.selected_routes.clone());
            let comparison_group_id = sample
                .and_then(|sample| sample.comparison_group_id.clone())
                .or_else(|| send.comparison_group_id.clone());
            let compute_unit_price_microlamports = sample
                .map(|sample| sample.compute_unit_price_microlamports)
                .or(Some(send.compute_unit_price_microlamports));
            let provider_ack_latencies = signature_samples
                .iter()
                .map(|sample| sample.provider_ack_latency_us)
                .collect::<Vec<_>>();

            UnobservedEvent {
                signature: send.signature.clone(),
                policy_arm,
                route_policy,
                selected_routes,
                comparison_group_id,
                sent_at: send.sent_at,
                send_slot: send.send_slot,
                leader_run_start_slot: send.leader_run_start_slot,
                leader_run_end_slot: send.leader_run_end_slot,
                trigger_source: send.trigger_source.clone(),
                slot_signal_status: send.slot_signal_status.clone(),
                send_leader_identity: send_leader.identity,
                send_leader_region: send_leader.region,
                send_leader_city: send_leader.city,
                send_leader_client: send_leader.client_software,
                provider_name: sample.map(|sample| sample.provider_name.clone()),
                provider_attempts: signature_samples.len(),
                provider_accepted_attempts: signature_samples
                    .iter()
                    .filter(|sample| sample.accepted)
                    .count(),
                provider_ack_latency_us_min: provider_ack_latencies.iter().min().copied(),
                provider_ack_latency_us_max: provider_ack_latencies.iter().max().copied(),
                provider_accepted: sample.map(|sample| sample.accepted),
                provider_ack_latency_us: sample.map(|sample| sample.provider_ack_latency_us),
                provider_request_id: sample.and_then(|sample| sample.provider_request_id.clone()),
                status_code: sample.and_then(|sample| sample.status_code),
                error_class: sample.and_then(|sample| sample.error_class.clone()),
                error: sample.and_then(|sample| sample.error.clone()),
                route_results_complete: sample.and_then(|sample| sample.route_results_complete),
                route_results: signature_samples
                    .iter()
                    .flat_map(|sample| sample.route_results.iter().cloned())
                    .collect(),
                compute_unit_price_microlamports,
                base_fee_lamports: cost.map(|cost| cost.base_fee_lamports),
                priority_fee_lamports: cost.map(|cost| cost.priority_fee_lamports),
                transaction_fee_lamports: cost.map(|cost| cost.transaction_fee_lamports),
                route_tip_generated_lamports: cost.map(|cost| cost.route_tip_generated_lamports),
                total_generated_spend_lamports: cost
                    .map(|cost| cost.total_generated_spend_lamports),
            }
        })
        .collect::<Vec<_>>();
    out.sort_by_key(|event| (event.send_slot, event.sent_at, event.signature.clone()));
    out
}

fn send_leader_meta(
    send: Option<&LeaderSendEvent>,
    leader_by_slot: &BTreeMap<u64, LeaderMeta>,
) -> LeaderMeta {
    let Some(send) = send else {
        return normalize_meta(LeaderMeta::default());
    };
    leader_by_slot
        .get(&send.send_slot)
        .cloned()
        .or_else(|| leader_by_slot.get(&send.leader_run_start_slot).cloned())
        .unwrap_or_else(|| {
            normalize_meta(LeaderMeta {
                identity: send.leader_identity.clone(),
                region: String::new(),
                city: String::new(),
                data_center_key: String::new(),
                client_family: send
                    .leader_client_family
                    .clone()
                    .unwrap_or_else(|| "unknown".to_string()),
                client_software: send
                    .leader_software_client
                    .clone()
                    .unwrap_or_else(|| "unknown".to_string()),
                stake_bucket: String::new(),
            })
        })
}

fn percentile_ms(sorted_us: &[i128], p: f64) -> Option<f64> {
    percentile_i128(sorted_us, p).map(|value| value as f64 / 1000.0)
}

fn percentile_i128(sorted: &[i128], p: f64) -> Option<i128> {
    if sorted.is_empty() {
        return None;
    }
    let rank = ((sorted.len() - 1) as f64 * p).ceil() as usize;
    sorted.get(rank).copied()
}

fn percentile_i64(sorted: &[i64], p: f64) -> Option<i64> {
    if sorted.is_empty() {
        return None;
    }
    let rank = ((sorted.len() - 1) as f64 * p).ceil() as usize;
    sorted.get(rank).copied()
}

fn percentile_u64(sorted: &[u64], p: f64) -> Option<u64> {
    if sorted.is_empty() {
        return None;
    }
    let rank = ((sorted.len() - 1) as f64 * p).ceil() as usize;
    sorted.get(rank).copied()
}

fn ratio(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}

fn report_markdown(report: &ReportOutput) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "# Transaction Landing Report: `{}`\n\n",
        report.test_id
    ));
    out.push_str(&format!(
        "- Generated: `{}`\n- Artifact dir: `{}`\n- Sent transactions: `{}`\n- Provider samples: `{}`\n- Observations: `{}`\n- Entry placements: `{}`\n- Matched caller signatures: `{}`\n- Unobserved caller signatures: `{}`\n- Matched route-tip signatures: `{}`\n- Leader-slot snapshots: `{}`\n\n",
        report.generated_at.to_rfc3339_opts(SecondsFormat::Millis, true),
        report.artifact_dir,
        report.totals.sent_transactions,
        report.totals.samples,
        report.totals.observations,
        report.totals.entry_placements,
        report.totals.matched_signatures,
        report.totals.unobserved_signatures,
        report.totals.matched_route_tip_signatures,
        report.totals.leader_slot_snapshots,
    ));
    out.push_str("This report is generated only from local benchmark artifacts. Leader metadata comes from saved `getLeaderSlots` snapshots captured during the run; no ClickHouse or private database join is required.\n\n");
    for warning in &report.warnings {
        out.push_str(&format!("> **Warning:** {warning}\n\n"));
    }

    out.push_str("## Cost Summary\n\n");
    out.push_str("Base fee and priority fee are deterministic fields from the signed benchmark transaction. Route-tip generated spend comes from route-result ACK metadata. Route-tip landed spend is counted only when the route-tip signature was observed in the matched streams.\n\n");
    out.push_str("| Submitted txs | Avg base fee | Avg priority fee | Avg tx fee | Avg route tip generated | Avg route tip landed | Avg total generated | Avg total landed | Generated tip txs | Landed tip txs |\n");
    out.push_str("| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |\n");
    let cost = &report.cost_summary;
    out.push_str(&format!(
        "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n\n",
        cost.submitted_transactions,
        fmt_f64(cost.avg_base_fee_lamports),
        fmt_f64(cost.avg_priority_fee_lamports),
        fmt_f64(cost.avg_transaction_fee_lamports),
        fmt_f64(cost.avg_route_tip_generated_lamports),
        fmt_f64(cost.avg_route_tip_landed_lamports),
        fmt_f64(cost.avg_total_generated_spend_lamports),
        fmt_f64(cost.avg_total_landed_spend_lamports),
        cost.generated_route_tip_count,
        cost.landed_route_tip_count,
    ));

    out.push_str("## Landing Observations\n\n");
    out.push_str("Landing latency uses the earliest observation per transaction across configured streams. In normal RPCEdge runs this should be the deshred stream; processed metadata is only counted here when it is the first or only observation for a signature.\n\n");
    out.push_str("| First source | Count | p50 ms | p90 ms | p99 ms | max ms | same-slot | slot delta p50 | slot index p50 | entry index p50 | tx-in-entry p50 |\n");
    out.push_str("| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |\n");
    for row in &report.source_summaries {
        out.push_str(&format!(
            "| `{}` | {} | {} | {} | {} | {} | {} / {:.1}% | {} | {} | {} | {} |\n",
            row.source_name,
            row.count,
            fmt_ms(row.submit_to_observed_p50_ms),
            fmt_ms(row.submit_to_observed_p90_ms),
            fmt_ms(row.submit_to_observed_p99_ms),
            fmt_ms(row.submit_to_observed_max_ms),
            row.same_slot_count,
            row.same_slot_rate * 100.0,
            fmt_i64(row.landed_slot_delta_p50),
            fmt_u64(row.slot_index_p50),
            fmt_u64(row.entry_index_p50),
            fmt_u64(row.tx_index_in_entry_p50),
        ));
    }

    out.push_str("\n## Unobserved Signatures\n\n");
    out.push_str("These signatures were submitted by the benchmark but were not observed by the configured landing streams during the collection window. Treat them as the first investigation bucket for dropped transactions, late-beyond-tail transactions, or observation-path misses.\n\n");
    out.push_str("| Signature | Arm | Routes | Send slot | Send leader | Send client | Provider ACK | Route ACKs | Total generated spend |\n");
    out.push_str("| --- | --- | --- | ---: | --- | --- | --- | --- | ---: |\n");
    for row in report.unobserved_events.iter().take(24) {
        out.push_str(&format!(
            "| `{}` | `{}` | `{}` | {} | `{}` | `{}` | {} | `{}` | {} |\n",
            short_sig(&row.signature),
            row.policy_arm.as_deref().unwrap_or("-"),
            row.selected_routes.join("+"),
            row.send_slot,
            leader_label(&row.send_leader_identity, &row.send_leader_city),
            row.send_leader_client,
            provider_attempt_label(row),
            route_results_summary(&row.route_results),
            fmt_u64(row.total_generated_spend_lamports),
        ));
    }

    for kind in [
        "leader_region",
        "leader_city",
        "data_center_key",
        "client_family",
        "client_software",
        "stake_bucket",
    ] {
        out.push_str(&format!("\n## Cohort: `{kind}`\n\n"));
        out.push_str("| Value | Source | Count | p50 ms | p90 ms | p99 ms | max ms |\n");
        out.push_str("| --- | --- | ---: | ---: | ---: | ---: | ---: |\n");
        for row in report
            .cohort_summaries
            .iter()
            .filter(|row| row.cohort_kind == kind)
            .take(24)
        {
            out.push_str(&format!(
                "| `{}` | `{}` | {} | {} | {} | {} | {} |\n",
                row.cohort_value,
                row.source_name,
                row.count,
                fmt_ms(row.submit_to_observed_p50_ms),
                fmt_ms(row.submit_to_observed_p90_ms),
                fmt_ms(row.submit_to_observed_p99_ms),
                fmt_ms(row.submit_to_observed_max_ms),
            ));
        }
    }

    out.push_str("\n## Tail Events\n\n");
    out.push_str("Tail latency uses the first observation time, normally deshred. Transaction slot index uses processed Yellowstone/gRPC metadata when available. Entry placement is joined from Yellowstone entry updates when `--collect-entries` was enabled; it is block placement context, not proof of packet arrival time at the leader.\n\n");
    out.push_str("| Signature | Source | Latency ms | Arm | Routes | Send slot | Send leader | Send client | Landed slot | Landed leader | Landed client | Tx index | Entry index | Tx-in-entry | Entry txs | Tx fee | Route tip generated | Route tip landed | Total landed |\n");
    out.push_str("| --- | --- | ---: | --- | --- | ---: | --- | --- | ---: | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |\n");
    for row in &report.tail_events {
        out.push_str(&format!(
            "| `{}` | `{}` | {:.3} | `{}` | `{}` | {} | `{}` | `{}` | {} | `{}` | `{}` | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            short_sig(&row.signature),
            row.source_name,
            row.latency_ms,
            row.policy_arm.as_deref().unwrap_or("-"),
            row.selected_routes.join("+"),
            fmt_u64(row.send_slot),
            leader_label(&row.send_leader_identity, &row.send_leader_city),
            row.send_leader_client,
            fmt_u64(row.landed_slot),
            leader_label(&row.landed_leader_identity, &row.landed_leader_city),
            row.landed_leader_client,
            fmt_u64(row.processed_slot_index),
            fmt_u64(row.entry_index),
            fmt_u64(row.tx_index_in_entry),
            fmt_u64(row.entry_executed_transaction_count),
            fmt_u64(row.transaction_fee_lamports),
            fmt_u64(row.route_tip_generated_lamports),
            fmt_u64(row.route_tip_landed_lamports),
            fmt_u64(row.total_landed_spend_lamports),
        ));
    }
    out.push_str("\n## Appendix: Sender ACK Diagnostics\n\n");
    out.push_str("ACK confirms that the relay accepted or rejected the request. It is useful for route availability and gateway health debugging, but it is not landing proof and is not used as a performance target.\n\n");
    out.push_str("| Provider | Policy | Routes | Count | Accepted | Errors | p50 ms | p90 ms | p99 ms | max ms |\n");
    out.push_str("| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |\n");
    for row in &report.provider_summaries {
        out.push_str(&format!(
            "| `{}` | `{}` | `{}` | {} | {} | {} | {} | {} | {} | {} |\n",
            row.provider_name,
            row.route_policy,
            row.selected_routes.join(","),
            row.count,
            row.accepted,
            row.errors,
            fmt_ms(row.ack_p50_ms),
            fmt_ms(row.ack_p90_ms),
            fmt_ms(row.ack_p99_ms),
            fmt_ms(row.ack_max_ms),
        ));
    }
    out
}

fn report_html(report: &ReportOutput) -> String {
    let mut out = String::new();
    let primary = primary_source_summary(report);
    let observed_ratio = ratio(
        report.totals.matched_signatures,
        report.totals.sent_transactions,
    ) * 100.0;
    let avg_landed_spend = fmt_f64(report.cost_summary.avg_total_landed_spend_lamports);

    out.push_str("<!doctype html><html><head><meta charset=\"utf-8\">");
    out.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">");
    out.push_str(&format!(
        "<title>RPCEdge Transaction Landing Benchmark: {}</title>",
        html_escape(&report.test_id)
    ));
    out.push_str(
        "<style>
*{box-sizing:border-box}
body{font-family:Inter,ui-sans-serif,system-ui,-apple-system,Segoe UI,sans-serif;margin:0;background:#f5f7f8;color:#17202a;line-height:1.45}
main{max-width:1280px;margin:0 auto;padding:28px}
a{color:#1769aa}.hero{background:#fff;border:1px solid #dfe5e8;border-radius:8px;padding:24px;margin-bottom:18px}
.eyebrow{font-size:12px;letter-spacing:.08em;text-transform:uppercase;color:#667681;font-weight:700}
h1{font-size:30px;line-height:1.08;margin:6px 0 8px;letter-spacing:0}h2{font-size:18px;margin:28px 0 10px;letter-spacing:0}
.subtitle{max-width:860px;color:#53636e;margin:0}.muted{color:#667681}.mono,code{font-family:ui-monospace,SFMono-Regular,Menlo,monospace}
code{background:#eef2f4;padding:1px 5px;border-radius:4px}.pill{display:inline-flex;align-items:center;gap:6px;border:1px solid #d9e1e5;background:#f7fafb;border-radius:999px;padding:3px 8px;font-size:12px;color:#465760}
.summary-grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(170px,1fr));gap:10px;margin-top:18px}
.metric{background:#fbfcfd;border:1px solid #e1e7ea;border-radius:8px;padding:12px;min-height:82px}.metric span{display:block;color:#667681;font-size:12px}.metric b{display:block;font-size:24px;line-height:1.15;margin-top:4px}.metric small{display:block;color:#72818a;margin-top:4px}
.section{background:#fff;border:1px solid #dfe5e8;border-radius:8px;padding:18px;margin:16px 0}.section h2:first-child{margin-top:0}
.note{background:#f8fbfc;border-left:4px solid #2f7dbd;padding:11px 13px;margin:12px 0;color:#465760}.warning{border-left-color:#d18800;background:#fff8e8}
.method-grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(240px,1fr));gap:10px}.method-card{border:1px solid #e2e8eb;border-radius:8px;padding:12px;background:#fbfcfd}.method-card b{display:block;margin-bottom:4px}
.table-wrap{overflow-x:auto;border:1px solid #e2e8eb;border-radius:8px;background:#fff}table{width:100%;border-collapse:collapse;min-width:900px}
th,td{padding:8px 10px;border-bottom:1px solid #edf1f3;text-align:right;font-size:13px;vertical-align:top;white-space:nowrap}
th:first-child,td:first-child,td.left{text-align:left}th{background:#f0f4f6;color:#41515b;font-weight:700}.sig{font-family:ui-monospace,SFMono-Regular,Menlo,monospace}
details{background:#fff;border:1px solid #dfe5e8;border-radius:8px;padding:14px;margin-top:16px}summary{cursor:pointer}.links{display:flex;flex-wrap:wrap;gap:8px;margin-top:14px}
@media(max-width:720px){main{padding:14px}.hero{padding:18px}h1{font-size:24px}.metric b{font-size:21px}}
</style></head><body><main>",
    );
    out.push_str(&format!(
        "<section class=\"hero\"><div class=\"eyebrow\">RPCEdge transaction landing benchmark</div><h1>{}</h1><p class=\"subtitle\">Leader-paced mainnet transactions submitted through <span class=\"mono\">rpcedge-quic-frankfurt</span>. Landing proof is matched by transaction signature. The primary latency clock is the earliest observed stream event; processed metadata is used for block position.</p><div class=\"links\"><span class=\"pill\">artifact <code>{}</code></span><span class=\"pill\">generated <code>{}</code></span><a class=\"pill\" href=\"./policy-comparison/\">Policy comparison</a><a class=\"pill\" href=\"tails.html\">Tail events</a><a class=\"pill\" href=\"unobserved.html\">Unobserved</a></div>",
        html_escape(&report.test_id),
        html_escape(&report.test_id),
        html_escape(&report.generated_at.to_rfc3339_opts(SecondsFormat::Millis, true)),
    ));
    out.push_str("<div class=\"summary-grid\">");
    for (label, value, help) in [
        (
            "Observed caller signatures",
            format!(
                "{} / {}",
                report.totals.matched_signatures, report.totals.sent_transactions
            ),
            format!("{observed_ratio:.2}% matched"),
        ),
        (
            "First-observed p50",
            fmt_ms(primary.and_then(|row| row.submit_to_observed_p50_ms)),
            primary
                .map(|row| row.source_name.clone())
                .unwrap_or_else(|| "no matched observations".to_string()),
        ),
        (
            "First-observed p90",
            fmt_ms(primary.and_then(|row| row.submit_to_observed_p90_ms)),
            "tail-weighted landing clock".to_string(),
        ),
        (
            "First-observed p99",
            fmt_ms(primary.and_then(|row| row.submit_to_observed_p99_ms)),
            "tail inspection starts here".to_string(),
        ),
        (
            "Unobserved signatures",
            report.totals.unobserved_signatures.to_string(),
            "submitted but not found in landing streams".to_string(),
        ),
        (
            "Entry placements",
            report.totals.entry_placements.to_string(),
            "processed tx index joined to Yellowstone entry range".to_string(),
        ),
        (
            "Same-slot observed",
            primary
                .map(|row| {
                    format!(
                        "{} / {:.1}%",
                        row.same_slot_count,
                        row.same_slot_rate * 100.0
                    )
                })
                .unwrap_or_else(|| "-".to_string()),
            "landed in sender-observed slot".to_string(),
        ),
        (
            "Landed slot delta p90",
            fmt_i64(primary.and_then(|row| row.landed_slot_delta_p90)),
            "landed slot minus send slot".to_string(),
        ),
        (
            "Avg landed spend",
            avg_landed_spend,
            "base + priority + observed route tip".to_string(),
        ),
        (
            "Route-tip sigs observed",
            report.totals.matched_route_tip_signatures.to_string(),
            "only observed tips count as spend".to_string(),
        ),
    ] {
        out.push_str(&format!(
            "<div class=\"metric\"><span>{}</span><b>{}</b><small>{}</small></div>",
            html_escape(label),
            html_escape(&value),
            html_escape(&help),
        ));
    }
    out.push_str("</div></section>");

    out.push_str("<section class=\"section\"><h2>Methodology</h2><div class=\"method-grid\"><div class=\"method-card\"><b>What was sent</b><span class=\"muted\">Memo-free self-transfer with compute budget instructions. Fees and route tips are read from signed transaction metadata and observed route-tip signatures.</span></div><div class=\"method-card\"><b>Landing clock</b><span class=\"muted\">Latency uses the first matched observation across configured streams. Usually this is SubscribeDeshred; processed is used when it is first or only.</span></div><div class=\"method-card\"><b>Block position</b><span class=\"muted\">Transaction index comes from Yellowstone processed metadata. Optional entry placement joins that index to Yellowstone entry ranges when <code>--collect-entries</code> is enabled.</span></div><div class=\"method-card\"><b>What ACK means</b><span class=\"muted\">ACK is retained only as a sender diagnostic. It is not leader admission, not landing proof, and not used as the product latency claim.</span></div></div><div class=\"note\">Generated only from local benchmark artifacts. Leader metadata comes from saved <code>getLeaderSlots</code> snapshots captured during the run; no ClickHouse or private database join is required. Entry placement is block-record context, not packet-arrival time at the leader.</div>");
    for warning in &report.warnings {
        out.push_str(&format!(
            "<div class=\"note warning\"><b>Warning:</b> {}</div>",
            html_escape(warning)
        ));
    }
    out.push_str("</section>");

    out.push_str("<section class=\"section\">");
    let cost = &report.cost_summary;
    out.push_str("<h2>Cost Summary</h2><p class=\"muted\">Base and priority fees come from the signed caller transaction. Route-tip generated spend comes from route-result ACKs. Route-tip landed spend counts only route-tip signatures observed by the matched streams.</p>");
    out.push_str("<div class=\"table-wrap\"><table><thead><tr><th>Submitted txs</th><th>Avg base fee</th><th>Avg priority fee</th><th>Avg tx fee</th><th>Avg route tip generated</th><th>Avg route tip landed</th><th>Avg total generated</th><th>Avg total landed</th><th>Generated tip txs</th><th>Landed tip txs</th></tr></thead><tbody>");
    out.push_str(&format!(
        "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
        cost.submitted_transactions,
        fmt_f64(cost.avg_base_fee_lamports),
        fmt_f64(cost.avg_priority_fee_lamports),
        fmt_f64(cost.avg_transaction_fee_lamports),
        fmt_f64(cost.avg_route_tip_generated_lamports),
        fmt_f64(cost.avg_route_tip_landed_lamports),
        fmt_f64(cost.avg_total_generated_spend_lamports),
        fmt_f64(cost.avg_total_landed_spend_lamports),
        cost.generated_route_tip_count,
        cost.landed_route_tip_count,
    ));
    out.push_str("</tbody></table></div></section>");

    out.push_str("<section class=\"section\"><h2>Landing Observations</h2><p class=\"muted\">Landing latency uses the earliest observation per transaction across configured streams. Processed metadata is not forced to be the timing source; it is used for transaction index when available.</p>");
    out.push_str("<div class=\"table-wrap\"><table><thead><tr><th>First source</th><th>Count</th><th>p50 ms</th><th>p90 ms</th><th>p99 ms</th><th>max ms</th><th>same-slot</th><th>slot delta p50</th><th>slot delta p90</th><th>slot index p50</th><th>entry index p50</th><th>tx-in-entry p50</th></tr></thead><tbody>");
    for row in &report.source_summaries {
        out.push_str(&format!(
            "<tr><td class=\"left\"><code>{}</code></td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{} / {:.1}%</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            html_escape(&row.source_name),
            row.count,
            fmt_ms(row.submit_to_observed_p50_ms),
            fmt_ms(row.submit_to_observed_p90_ms),
            fmt_ms(row.submit_to_observed_p99_ms),
            fmt_ms(row.submit_to_observed_max_ms),
            row.same_slot_count,
            row.same_slot_rate * 100.0,
            fmt_i64(row.landed_slot_delta_p50),
            fmt_i64(row.landed_slot_delta_p90),
            fmt_u64(row.slot_index_p50),
            fmt_u64(row.entry_index_p50),
            fmt_u64(row.tx_index_in_entry_p50),
        ));
    }
    out.push_str("</tbody></table></div></section>");

    out.push_str("<section class=\"section\"><h2>Unobserved Signatures</h2><p class=\"muted\">Submitted caller signatures that were not observed by the configured landing streams during the collection window. This is the investigation bucket for probable drops, very late landings beyond the observation tail, or observation-path misses. Full signatures and route ACKs live in <a href=\"unobserved.html\">the unobserved artifact</a>.</p>");
    out.push_str("<div class=\"table-wrap\"><table><thead><tr><th>Signature</th><th>Arm</th><th>Routes</th><th>Send slot</th><th>Send leader</th><th>Send client</th><th>Provider ACK</th><th>Route ACKs</th><th>Total generated spend</th></tr></thead><tbody>");
    for row in report.unobserved_events.iter().take(24) {
        out.push_str(&format!(
            "<tr><td class=\"left sig\"><code>{}</code></td><td><code>{}</code></td><td class=\"left\"><code>{}</code></td><td>{}</td><td class=\"left\">{}</td><td class=\"left\">{}</td><td>{}</td><td class=\"left\"><code>{}</code></td><td>{}</td></tr>",
            html_escape(&short_sig(&row.signature)),
            html_escape(row.policy_arm.as_deref().unwrap_or("-")),
            html_escape(&row.selected_routes.join("+")),
            row.send_slot,
            html_escape(&leader_label(&row.send_leader_identity, &row.send_leader_city)),
            html_escape(&row.send_leader_client),
            html_escape(&provider_attempt_label(row)),
            html_escape(&route_results_summary(&row.route_results)),
            fmt_u64(row.total_generated_spend_lamports),
        ));
    }
    out.push_str("</tbody></table></div></section>");

    out.push_str("<section class=\"section\"><h2>Leader Cohorts</h2><p class=\"muted\">Cohorts are grouped by the scheduled/landed leader metadata captured from <code>getLeaderSlots</code>. They are useful for tail hunting, not as final claims from a 5-minute smoke.</p>");
    for kind in [
        "leader_region",
        "leader_city",
        "data_center_key",
        "client_family",
        "client_software",
        "stake_bucket",
    ] {
        out.push_str(&format!(
            "<h2><code>{}</code></h2><div class=\"table-wrap\"><table><thead><tr><th>Value</th><th>Source</th><th>Count</th><th>p50 ms</th><th>p90 ms</th><th>p99 ms</th><th>max ms</th></tr></thead><tbody>",
            html_escape(kind)
        ));
        for row in report
            .cohort_summaries
            .iter()
            .filter(|row| row.cohort_kind == kind)
            .take(24)
        {
            out.push_str(&format!(
                "<tr><td class=\"left\"><code>{}</code></td><td><code>{}</code></td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                html_escape(&row.cohort_value),
                html_escape(&row.source_name),
                row.count,
                fmt_ms(row.submit_to_observed_p50_ms),
                fmt_ms(row.submit_to_observed_p90_ms),
                fmt_ms(row.submit_to_observed_p99_ms),
                fmt_ms(row.submit_to_observed_max_ms),
            ));
        }
        out.push_str("</tbody></table></div>");
    }
    out.push_str("</section>");

    out.push_str("<section class=\"section\"><h2>Tail Events</h2><p class=\"muted\">Slowest first-observed caller transactions. Timing uses the first stream observation; transaction index uses processed metadata when available. Entry fields are present only when entry collection was enabled. Full signatures and CSV/JSON live in <a href=\"tails.html\">the tail artifact</a>.</p><div class=\"table-wrap\"><table><thead><tr><th>Signature</th><th>Source</th><th>Latency ms</th><th>Arm</th><th>Routes</th><th>Send slot</th><th>Send leader</th><th>Send client</th><th>Landed slot</th><th>Landed leader</th><th>Landed client</th><th>Tx index</th><th>Entry</th><th>Tx-in-entry</th><th>Entry txs</th><th>Tx fee</th><th>Route tip gen</th><th>Route tip landed</th><th>Total landed</th></tr></thead><tbody>");
    for row in &report.tail_events {
        out.push_str(&format!(
            "<tr><td class=\"left sig\"><code>{}</code></td><td><code>{}</code></td><td>{:.3}</td><td><code>{}</code></td><td class=\"left\"><code>{}</code></td><td>{}</td><td class=\"left\">{}</td><td class=\"left\">{}</td><td>{}</td><td class=\"left\">{}</td><td class=\"left\">{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            html_escape(&short_sig(&row.signature)),
            html_escape(&row.source_name),
            row.latency_ms,
            html_escape(row.policy_arm.as_deref().unwrap_or("-")),
            html_escape(&row.selected_routes.join("+")),
            fmt_u64(row.send_slot),
            html_escape(&leader_label(&row.send_leader_identity, &row.send_leader_city)),
            html_escape(&row.send_leader_client),
            fmt_u64(row.landed_slot),
            html_escape(&leader_label(&row.landed_leader_identity, &row.landed_leader_city)),
            html_escape(&row.landed_leader_client),
            fmt_u64(row.processed_slot_index),
            fmt_u64(row.entry_index),
            fmt_u64(row.tx_index_in_entry),
            fmt_u64(row.entry_executed_transaction_count),
            fmt_u64(row.transaction_fee_lamports),
            fmt_u64(row.route_tip_generated_lamports),
            fmt_u64(row.route_tip_landed_lamports),
            fmt_u64(row.total_landed_spend_lamports),
        ));
    }
    out.push_str("</tbody></table></div></section>");

    out.push_str("<details><summary><b>Sender ACK diagnostics</b></summary><p class=\"muted\">ACK confirms that the relay accepted or rejected the request. It is useful for route availability and gateway health debugging, but it is not landing proof and is not used as a performance target.</p><div class=\"table-wrap\"><table><thead><tr><th>Provider</th><th>Policy</th><th>Routes</th><th>Count</th><th>Accepted</th><th>Errors</th><th>p50 ms</th><th>p90 ms</th><th>p99 ms</th><th>max ms</th></tr></thead><tbody>");
    for row in &report.provider_summaries {
        out.push_str(&format!(
            "<tr><td class=\"left\"><code>{}</code></td><td><code>{}</code></td><td><code>{}</code></td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            html_escape(&row.provider_name),
            html_escape(&row.route_policy),
            html_escape(&row.selected_routes.join(",")),
            row.count,
            row.accepted,
            row.errors,
            fmt_ms(row.ack_p50_ms),
            fmt_ms(row.ack_p90_ms),
            fmt_ms(row.ack_p99_ms),
            fmt_ms(row.ack_max_ms),
        ));
    }
    out.push_str("</tbody></table></div></details></main></body></html>");
    out
}

fn primary_source_summary(report: &ReportOutput) -> Option<&SourceReportSummary> {
    report.source_summaries.iter().max_by(|a, b| {
        a.first_seen
            .cmp(&b.first_seen)
            .then_with(|| a.count.cmp(&b.count))
    })
}

fn write_tail_artifacts(artifact_dir: &Path, tail_events: &[TailEvent]) -> Result<()> {
    fs::write(
        artifact_dir.join("tail-events.json"),
        serde_json::to_vec_pretty(tail_events)?,
    )
    .with_context(|| format!("write {}", artifact_dir.join("tail-events.json").display()))?;
    fs::write(
        artifact_dir.join("tail-events.csv"),
        tail_events_csv(tail_events),
    )
    .with_context(|| format!("write {}", artifact_dir.join("tail-events.csv").display()))?;
    fs::write(
        artifact_dir.join("tails.html"),
        tail_events_html(tail_events),
    )
    .with_context(|| format!("write {}", artifact_dir.join("tails.html").display()))?;
    Ok(())
}

fn write_unobserved_artifacts(artifact_dir: &Path, rows: &[UnobservedEvent]) -> Result<()> {
    fs::write(
        artifact_dir.join("unobserved-events.json"),
        serde_json::to_vec_pretty(rows)?,
    )
    .with_context(|| {
        format!(
            "write {}",
            artifact_dir.join("unobserved-events.json").display()
        )
    })?;
    fs::write(
        artifact_dir.join("unobserved-events.csv"),
        unobserved_events_csv(rows),
    )
    .with_context(|| {
        format!(
            "write {}",
            artifact_dir.join("unobserved-events.csv").display()
        )
    })?;
    fs::write(
        artifact_dir.join("unobserved.html"),
        unobserved_events_html(rows),
    )
    .with_context(|| format!("write {}", artifact_dir.join("unobserved.html").display()))?;
    Ok(())
}

fn tail_events_csv(rows: &[TailEvent]) -> String {
    let mut out = String::new();
    out.push_str("signature,source_name,latency_ms,policy_arm,route_policy,selected_routes,send_slot,send_leader_identity,send_leader_city,send_leader_client,landed_slot,landed_leader_identity,landed_leader_city,landed_leader_client,first_observed_slot_index,processed_slot_index,entry_index,tx_index_in_entry,entry_executed_transaction_count,compute_unit_price_microlamports,base_fee_lamports,priority_fee_lamports,transaction_fee_lamports,route_tip_generated_lamports,route_tip_landed_lamports,total_generated_spend_lamports,total_landed_spend_lamports,comparison_group_id\n");
    for row in rows {
        let fields = [
            row.signature.clone(),
            row.source_name.clone(),
            format!("{:.3}", row.latency_ms),
            row.policy_arm.clone().unwrap_or_default(),
            row.route_policy.clone().unwrap_or_default(),
            row.selected_routes.join("+"),
            fmt_u64(row.send_slot),
            row.send_leader_identity.clone(),
            row.send_leader_city.clone(),
            row.send_leader_client.clone(),
            fmt_u64(row.landed_slot),
            row.landed_leader_identity.clone(),
            row.landed_leader_city.clone(),
            row.landed_leader_client.clone(),
            fmt_u64(row.first_observed_slot_index),
            fmt_u64(row.processed_slot_index),
            fmt_u64(row.entry_index),
            fmt_u64(row.tx_index_in_entry),
            fmt_u64(row.entry_executed_transaction_count),
            row.compute_unit_price_microlamports
                .map(|value| value.to_string())
                .unwrap_or_default(),
            row.base_fee_lamports
                .map(|value| value.to_string())
                .unwrap_or_default(),
            row.priority_fee_lamports
                .map(|value| value.to_string())
                .unwrap_or_default(),
            row.transaction_fee_lamports
                .map(|value| value.to_string())
                .unwrap_or_default(),
            row.route_tip_generated_lamports
                .map(|value| value.to_string())
                .unwrap_or_default(),
            row.route_tip_landed_lamports
                .map(|value| value.to_string())
                .unwrap_or_default(),
            row.total_generated_spend_lamports
                .map(|value| value.to_string())
                .unwrap_or_default(),
            row.total_landed_spend_lamports
                .map(|value| value.to_string())
                .unwrap_or_default(),
            row.comparison_group_id.clone().unwrap_or_default(),
        ];
        out.push_str(
            &fields
                .iter()
                .map(|field| csv_escape(field))
                .collect::<Vec<_>>()
                .join(","),
        );
        out.push('\n');
    }
    out
}

fn tail_events_html(rows: &[TailEvent]) -> String {
    let mut out = String::new();
    out.push_str("<!doctype html><html><head><meta charset=\"utf-8\"><title>Tail Events</title>");
    out.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">");
    out.push_str("<style>*{box-sizing:border-box}body{font-family:Inter,ui-sans-serif,system-ui,-apple-system,Segoe UI,sans-serif;margin:0;background:#f5f7f8;color:#17202a;line-height:1.45}main{max-width:1380px;margin:0 auto;padding:28px}.hero,.section{background:#fff;border:1px solid #dfe5e8;border-radius:8px;padding:20px;margin-bottom:16px}.eyebrow{font-size:12px;letter-spacing:.08em;text-transform:uppercase;color:#667681;font-weight:700}h1{font-size:30px;margin:6px 0 8px}.muted{color:#667681}.links{display:flex;flex-wrap:wrap;gap:8px;margin-top:14px}.pill{display:inline-flex;border:1px solid #d9e1e5;background:#f7fafb;border-radius:999px;padding:3px 8px;font-size:12px;color:#465760;text-decoration:none}.table-wrap{overflow-x:auto;border:1px solid #e2e8eb;border-radius:8px;background:#fff}table{border-collapse:collapse;width:100%;min-width:1280px}th,td{border-bottom:1px solid #edf1f3;padding:8px 10px;text-align:right;vertical-align:top;font-size:13px;white-space:nowrap}th:first-child,td:first-child,td.left{text-align:left}th{background:#f0f4f6;color:#41515b;font-weight:700}.sig{font-family:ui-monospace,SFMono-Regular,Menlo,monospace;font-size:12px;word-break:break-all;white-space:normal}code{background:#eef2f4;padding:1px 5px;border-radius:4px}@media(max-width:720px){main{padding:14px}.hero,.section{padding:16px}h1{font-size:24px}}</style>");
    out.push_str("</head><body><main><section class=\"hero\"><div class=\"eyebrow\">RPCEdge benchmark tail inspection</div><h1>Tail Events</h1>");
    out.push_str("<p class=\"muted\">Full signatures and route arms for the slowest first-observed landing events. Latency uses the first stream observation. Transaction index uses processed Yellowstone/gRPC metadata when available. Entry placement is shown when <code>--collect-entries</code> was enabled. Route-tip landed cost counts only observed tip signatures.</p>");
    out.push_str("<div class=\"links\"><a class=\"pill\" href=\"./\">Full report</a><a class=\"pill\" href=\"./policy-comparison/\">Policy comparison</a><a class=\"pill\" href=\"tail-events.csv\">CSV</a><a class=\"pill\" href=\"tail-events.json\">JSON</a></div></section>");
    out.push_str("<section class=\"section\"><div class=\"table-wrap\"><table><thead><tr><th>Signature</th><th>First source</th><th>Latency ms</th><th>Arm</th><th>Routes</th><th>Send slot</th><th>Send leader</th><th>Send client</th><th>Landed slot</th><th>Landed leader</th><th>Landed client</th><th>Tx index</th><th>Entry</th><th>Tx-in-entry</th><th>Entry txs</th><th>Tx fee</th><th>Route tip gen</th><th>Route tip landed</th><th>Total landed</th></tr></thead><tbody>");
    for row in rows {
        out.push_str(&format!(
            "<tr><td class=\"left sig\">{}</td><td><code>{}</code></td><td>{:.3}</td><td><code>{}</code></td><td class=\"left\"><code>{}</code></td><td>{}</td><td class=\"left\">{}</td><td class=\"left\">{}</td><td>{}</td><td class=\"left\">{}</td><td class=\"left\">{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            html_escape(&row.signature),
            html_escape(&row.source_name),
            row.latency_ms,
            html_escape(row.policy_arm.as_deref().unwrap_or("-")),
            html_escape(&row.selected_routes.join("+")),
            fmt_u64(row.send_slot),
            html_escape(&leader_label(&row.send_leader_identity, &row.send_leader_city)),
            html_escape(&row.send_leader_client),
            fmt_u64(row.landed_slot),
            html_escape(&leader_label(&row.landed_leader_identity, &row.landed_leader_city)),
            html_escape(&row.landed_leader_client),
            fmt_u64(row.processed_slot_index),
            fmt_u64(row.entry_index),
            fmt_u64(row.tx_index_in_entry),
            fmt_u64(row.entry_executed_transaction_count),
            fmt_u64(row.transaction_fee_lamports),
            fmt_u64(row.route_tip_generated_lamports),
            fmt_u64(row.route_tip_landed_lamports),
            fmt_u64(row.total_landed_spend_lamports),
        ));
    }
    out.push_str("</tbody></table></div></section></main></body></html>");
    out
}

fn unobserved_events_csv(rows: &[UnobservedEvent]) -> String {
    let mut out = String::new();
    out.push_str("signature,policy_arm,route_policy,selected_routes,comparison_group_id,sent_at,send_slot,leader_run_start_slot,leader_run_end_slot,trigger_source,slot_signal_status,send_leader_identity,send_leader_region,send_leader_city,send_leader_client,provider_name,provider_attempts,provider_accepted_attempts,provider_ack_latency_us_min,provider_ack_latency_us_max,provider_accepted,provider_ack_latency_us,provider_request_id,status_code,error_class,error,route_results_complete,route_results,compute_unit_price_microlamports,base_fee_lamports,priority_fee_lamports,transaction_fee_lamports,route_tip_generated_lamports,total_generated_spend_lamports\n");
    for row in rows {
        let fields = [
            row.signature.clone(),
            row.policy_arm.clone().unwrap_or_default(),
            row.route_policy.clone().unwrap_or_default(),
            row.selected_routes.join("+"),
            row.comparison_group_id.clone().unwrap_or_default(),
            row.sent_at.to_rfc3339_opts(SecondsFormat::Micros, true),
            row.send_slot.to_string(),
            row.leader_run_start_slot.to_string(),
            row.leader_run_end_slot.to_string(),
            row.trigger_source.clone(),
            row.slot_signal_status.clone().unwrap_or_default(),
            row.send_leader_identity.clone(),
            row.send_leader_region.clone(),
            row.send_leader_city.clone(),
            row.send_leader_client.clone(),
            row.provider_name.clone().unwrap_or_default(),
            row.provider_attempts.to_string(),
            row.provider_accepted_attempts.to_string(),
            row.provider_ack_latency_us_min
                .map(|value| value.to_string())
                .unwrap_or_default(),
            row.provider_ack_latency_us_max
                .map(|value| value.to_string())
                .unwrap_or_default(),
            row.provider_accepted
                .map(|value| value.to_string())
                .unwrap_or_default(),
            row.provider_ack_latency_us
                .map(|value| value.to_string())
                .unwrap_or_default(),
            row.provider_request_id.clone().unwrap_or_default(),
            row.status_code
                .map(|value| value.to_string())
                .unwrap_or_default(),
            row.error_class.clone().unwrap_or_default(),
            row.error.clone().unwrap_or_default(),
            row.route_results_complete
                .map(|value| value.to_string())
                .unwrap_or_default(),
            route_results_summary(&row.route_results),
            row.compute_unit_price_microlamports
                .map(|value| value.to_string())
                .unwrap_or_default(),
            row.base_fee_lamports
                .map(|value| value.to_string())
                .unwrap_or_default(),
            row.priority_fee_lamports
                .map(|value| value.to_string())
                .unwrap_or_default(),
            row.transaction_fee_lamports
                .map(|value| value.to_string())
                .unwrap_or_default(),
            row.route_tip_generated_lamports
                .map(|value| value.to_string())
                .unwrap_or_default(),
            row.total_generated_spend_lamports
                .map(|value| value.to_string())
                .unwrap_or_default(),
        ];
        out.push_str(
            &fields
                .iter()
                .map(|field| csv_escape(field))
                .collect::<Vec<_>>()
                .join(","),
        );
        out.push('\n');
    }
    out
}

fn unobserved_events_html(rows: &[UnobservedEvent]) -> String {
    let mut out = String::new();
    out.push_str(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>Unobserved Signatures</title>",
    );
    out.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">");
    out.push_str("<style>*{box-sizing:border-box}body{font-family:Inter,ui-sans-serif,system-ui,-apple-system,Segoe UI,sans-serif;margin:0;background:#f5f7f8;color:#17202a;line-height:1.45}main{max-width:1380px;margin:0 auto;padding:28px}.hero,.section{background:#fff;border:1px solid #dfe5e8;border-radius:8px;padding:20px;margin-bottom:16px}.eyebrow{font-size:12px;letter-spacing:.08em;text-transform:uppercase;color:#667681;font-weight:700}h1{font-size:30px;margin:6px 0 8px}.muted{color:#667681}.links{display:flex;flex-wrap:wrap;gap:8px;margin-top:14px}.pill{display:inline-flex;border:1px solid #d9e1e5;background:#f7fafb;border-radius:999px;padding:3px 8px;font-size:12px;color:#465760;text-decoration:none}.table-wrap{overflow-x:auto;border:1px solid #e2e8eb;border-radius:8px;background:#fff}table{border-collapse:collapse;width:100%;min-width:1320px}th,td{border-bottom:1px solid #edf1f3;padding:8px 10px;text-align:right;vertical-align:top;font-size:13px;white-space:nowrap}th:first-child,td:first-child,td.left{text-align:left}th{background:#f0f4f6;color:#41515b;font-weight:700}.sig{font-family:ui-monospace,SFMono-Regular,Menlo,monospace;font-size:12px;word-break:break-all;white-space:normal}code{background:#eef2f4;padding:1px 5px;border-radius:4px}@media(max-width:720px){main{padding:14px}.hero,.section{padding:16px}h1{font-size:24px}}</style>");
    out.push_str("</head><body><main><section class=\"hero\"><div class=\"eyebrow\">RPCEdge benchmark failure inspection</div><h1>Unobserved Signatures</h1>");
    out.push_str("<p class=\"muted\">Submitted caller signatures that were not observed by the configured landing streams during the collection window. Use this page to inspect likely drops, landings beyond the observation tail, or observation-path misses by scheduled leader and route ACK status.</p>");
    out.push_str("<div class=\"links\"><a class=\"pill\" href=\"./\">Full report</a><a class=\"pill\" href=\"./policy-comparison/\">Policy comparison</a><a class=\"pill\" href=\"tails.html\">Tail events</a><a class=\"pill\" href=\"unobserved-events.csv\">CSV</a><a class=\"pill\" href=\"unobserved-events.json\">JSON</a></div></section>");
    out.push_str("<section class=\"section\"><div class=\"table-wrap\"><table><thead><tr><th>Signature</th><th>Arm</th><th>Routes</th><th>Send slot</th><th>Scheduled leader</th><th>Client</th><th>Provider ACK</th><th>Route ACKs</th><th>Generated spend</th><th>Provider error</th></tr></thead><tbody>");
    for row in rows {
        out.push_str(&format!(
            "<tr><td class=\"left sig\">{}</td><td><code>{}</code></td><td class=\"left\"><code>{}</code></td><td>{}</td><td class=\"left\">{}</td><td class=\"left\">{}</td><td>{}</td><td class=\"left\"><code>{}</code></td><td>{}</td><td class=\"left\">{}</td></tr>",
            html_escape(&row.signature),
            html_escape(row.policy_arm.as_deref().unwrap_or("-")),
            html_escape(&row.selected_routes.join("+")),
            row.send_slot,
            html_escape(&leader_label(&row.send_leader_identity, &row.send_leader_city)),
            html_escape(&row.send_leader_client),
            html_escape(&provider_attempt_label(row)),
            html_escape(&route_results_summary(&row.route_results)),
            fmt_u64(row.total_generated_spend_lamports),
            html_escape(
                &row.error
                    .clone()
                    .or_else(|| row.error_class.clone())
                    .unwrap_or_else(|| "-".to_string())
            ),
        ));
    }
    out.push_str("</tbody></table></div></section></main></body></html>");
    out
}

fn provider_ack_label(accepted: Option<bool>, latency_us: Option<u128>) -> String {
    match (accepted, latency_us) {
        (Some(true), Some(latency_us)) => format!("accepted {:.3}ms", latency_us as f64 / 1000.0),
        (Some(true), None) => "accepted".to_string(),
        (Some(false), Some(latency_us)) => format!("rejected {:.3}ms", latency_us as f64 / 1000.0),
        (Some(false), None) => "rejected".to_string(),
        (None, _) => "-".to_string(),
    }
}

fn provider_attempt_label(row: &UnobservedEvent) -> String {
    if row.provider_attempts == 0 {
        return provider_ack_label(row.provider_accepted, row.provider_ack_latency_us);
    }
    let latency = match (
        row.provider_ack_latency_us_min,
        row.provider_ack_latency_us_max,
    ) {
        (Some(min), Some(max)) if min == max => format!("{:.3}ms", min as f64 / 1000.0),
        (Some(min), Some(max)) => {
            format!("{:.3}-{:.3}ms", min as f64 / 1000.0, max as f64 / 1000.0)
        }
        _ => "-".to_string(),
    };
    format!(
        "{}/{} accepted ({latency})",
        row.provider_accepted_attempts, row.provider_attempts
    )
}

fn route_results_summary(rows: &[crate::adapters::RouteResultAck]) -> String {
    if rows.is_empty() {
        return "-".to_string();
    }
    rows.iter()
        .map(|row| {
            let status = if row.accepted { "ok" } else { "err" };
            let mut label = format!("{}:{status}:{}us", row.route, row.latency_us);
            if let Some(error_class) = &row.error_class {
                label.push(':');
                label.push_str(error_class);
            }
            label
        })
        .collect::<Vec<_>>()
        .join(" | ")
}

fn csv_escape(value: &str) -> String {
    if value.contains([',', '"', '\n']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn leader_label(identity: &str, city: &str) -> String {
    if identity == "unknown" && city == "unknown" {
        "unknown".to_string()
    } else if city == "unknown" || city.is_empty() {
        identity.to_string()
    } else {
        format!("{identity} ({city})")
    }
}

fn fmt_ms(value: Option<f64>) -> String {
    value
        .map(|value| format!("{value:.3}"))
        .unwrap_or_else(|| "-".to_string())
}

fn fmt_i64(value: Option<i64>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "-".to_string())
}

fn fmt_u64(value: Option<u64>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "-".to_string())
}

fn fmt_f64(value: Option<f64>) -> String {
    value
        .map(|value| format!("{value:.3}"))
        .unwrap_or_else(|| "-".to_string())
}

fn short_sig(signature: &str) -> String {
    if signature.len() <= 16 {
        signature.to_string()
    } else {
        format!(
            "{}...{}",
            &signature[..8],
            &signature[signature.len() - 8..]
        )
    }
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
    use serde_json::json;
    use tempfile::tempdir;

    #[test]
    fn report_uses_saved_leader_slot_snapshot() {
        let dir = tempdir().expect("tempdir");
        let root = dir.path();
        fs::write(
            root.join("samples.ndjson"),
            r#"{"schema_version":2,"test_id":"run-a","iteration":0,"signature":"sig-a","provider_name":"p","provider_kind":"rpcedge_quic_raw_tx","accepted":true,"client_started_at":"2026-01-01T00:00:00Z","client_finished_at":"2026-01-01T00:00:00.001Z","client_ack_latency_us":1000,"provider_send_started_at":"2026-01-01T00:00:00Z","provider_send_finished_at":"2026-01-01T00:00:00.001Z","provider_ack_latency_us":1000,"provider_request_id":"run-a-0","returned_signature":"sig-a","status_code":200,"error_class":null,"error":null,"route_policy":"static","route_mode":"only","selected_routes":["tpu_quic"],"leader_client_family":"jito","compute_unit_limit":660,"compute_unit_price_microlamports":0,"base_fee_lamports":5000,"priority_fee_lamports":0,"transaction_fee_lamports":5000}"#,
        )
        .expect("samples");
        fs::write(
            root.join("leader-sends.ndjson"),
            r#"{"schema_version":2,"test_id":"run-a","iteration":0,"signature":"sig-a","leader_identity":"leader-a","leader_run_start_slot":100,"leader_run_end_slot":103,"send_slot":100,"sent_at":"2026-01-01T00:00:00Z","trigger_source":"grpc_slot","slot_signal_status":"SLOT_FIRST_SHRED_RECEIVED","slot_signal_observed_at":"2026-01-01T00:00:00Z","leader_client_family":"jito","route_policy":"static","selected_routes":["tpu_quic"],"compute_unit_limit":660,"compute_unit_price_microlamports":0,"base_fee_lamports":5000,"priority_fee_lamports":0,"transaction_fee_lamports":5000}"#,
        )
        .expect("sends");
        fs::write(
            root.join("matched-observations.ndjson"),
            r#"{"schema_version":1,"test_id":"run-a","signature":"sig-a","source_name":"deshred","source_kind":"yellowstone_deshred","observed_at":"2026-01-01T00:00:00.100Z","submitted_at":"2026-01-01T00:00:00Z","slot":101,"slot_index":null,"source_sequence":null}
{"schema_version":1,"test_id":"run-a","signature":"sig-a","source_name":"processed","source_kind":"yellowstone_processed","observed_at":"2026-01-01T00:00:00.120Z","submitted_at":"2026-01-01T00:00:00Z","slot":101,"slot_index":42,"source_sequence":null}"#,
        )
        .expect("observations");
        let snapshot = LeaderSlotsSnapshotArtifact {
            schema_version: 1,
            fetched_at: Utc::now(),
            rpc_url_label: "http://rpc.example".to_string(),
            start_slot: 100,
            limit: 4,
            response: json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": {
                    "success": true,
                    "data": [{
                        "slot": 100,
                        "identity": "leader-a",
                        "stake": 1_500_000_000_000_000_u64,
                        "location": {
                            "region": "Europe",
                            "city": "Frankfurt",
                            "dataCenterKey": "20326-DE-Frankfurt"
                        },
                        "client": {
                            "family": "jito",
                            "software": "JitoLabs"
                        }
                    }, {
                        "slot": 101,
                        "identity": "leader-b",
                        "stake": 900_000_000_000_000_u64,
                        "location": {
                            "region": "North America",
                            "city": "Chicago",
                            "dataCenterKey": "1072-US-Chicago"
                        },
                        "client": {
                            "family": "harmonic",
                            "software": "HarmonicAgave"
                        }
                    }]
                }
            }),
        };
        fs::write(
            root.join("leader-slots-snapshot-100.json"),
            serde_json::to_vec(&snapshot).expect("snapshot"),
        )
        .expect("snapshot");

        let report = generate_report(root).expect("report");
        assert_eq!(report.totals.matched_signatures, 1);
        assert_eq!(report.tail_events.len(), 1);
        let tail = &report.tail_events[0];
        assert_eq!(tail.source_name, "deshred");
        assert_eq!(tail.send_leader_identity, "leader-a");
        assert_eq!(tail.landed_leader_identity, "leader-b");
        assert_eq!(tail.processed_slot_index, Some(42));
        assert!(report
            .cohort_summaries
            .iter()
            .any(|row| row.cohort_kind == "leader_region" && row.cohort_value == "North America"));
        assert!(root.join("report.md").exists());
        assert!(root.join("report.html").exists());
        assert!(root.join("tail-events.csv").exists());
        assert!(root.join("tails.html").exists());
    }

    #[test]
    fn report_counts_route_tip_landed_cost_without_polluting_landing_count() {
        let dir = tempdir().expect("tempdir");
        let root = dir.path();
        fs::write(
            root.join("samples.ndjson"),
            r#"{"schema_version":4,"test_id":"run-tip","iteration":0,"signature":"caller-sig","provider_name":"p","provider_kind":"rpcedge_quic_raw_tx","accepted":true,"client_started_at":"2026-01-01T00:00:00Z","client_finished_at":"2026-01-01T00:00:00.001Z","client_ack_latency_us":1000,"provider_send_started_at":"2026-01-01T00:00:00Z","provider_send_finished_at":"2026-01-01T00:00:00.001Z","provider_ack_latency_us":1000,"provider_request_id":"run-tip-0","returned_signature":"caller-sig","status_code":200,"error_class":null,"error":null,"route_policy":"always_race","route_mode":"only","selected_routes":["tpu_quic","jito_bundle"],"leader_client_family":"jito","compute_unit_limit":660,"compute_unit_price_microlamports":300000,"base_fee_lamports":5000,"priority_fee_lamports":198,"transaction_fee_lamports":5198,"route_results":[{"route":"jito_bundle","route_name":"jito_bundle","provider":"jito","accepted":true,"latency_us":100,"bundle_id":"bundle-1","route_tip_signature":"tip-sig","route_tip_lamports":1000,"route_tip_account":"tip-account"}]}"#,
        )
        .expect("samples");
        fs::write(
            root.join("leader-sends.ndjson"),
            r#"{"schema_version":4,"test_id":"run-tip","iteration":0,"signature":"caller-sig","leader_identity":"leader-a","leader_run_start_slot":100,"leader_run_end_slot":103,"send_slot":100,"sent_at":"2026-01-01T00:00:00Z","trigger_source":"grpc_slot","slot_signal_status":"SLOT_FIRST_SHRED_RECEIVED","slot_signal_observed_at":"2026-01-01T00:00:00Z","leader_client_family":"jito","route_policy":"always_race","selected_routes":["tpu_quic","jito_bundle"],"compute_unit_limit":660,"compute_unit_price_microlamports":300000,"base_fee_lamports":5000,"priority_fee_lamports":198,"transaction_fee_lamports":5198}"#,
        )
        .expect("sends");
        fs::write(
            root.join("matched-observations.ndjson"),
            r#"{"schema_version":1,"test_id":"run-tip","signature":"caller-sig","signature_role":"caller","source_name":"rpcedge_deshred","source_kind":"yellowstone_deshred","observed_at":"2026-01-01T00:00:00.100Z","submitted_at":"2026-01-01T00:00:00Z","slot":100,"slot_index":null,"source_sequence":null}
{"schema_version":1,"test_id":"run-tip","signature":"tip-sig","signature_role":"route_tip","caller_signature":"caller-sig","source_name":"rpcedge_deshred","source_kind":"yellowstone_deshred","observed_at":"2026-01-01T00:00:00.110Z","submitted_at":"2026-01-01T00:00:00Z","slot":100,"slot_index":null,"source_sequence":null}"#,
        )
        .expect("observations");

        let report = generate_report(root).expect("report");
        assert_eq!(report.totals.matched_signatures, 1);
        assert_eq!(report.totals.matched_route_tip_signatures, 1);
        assert_eq!(report.cost_summary.total_transaction_fee_lamports, 5_198);
        assert_eq!(
            report.cost_summary.total_route_tip_generated_lamports,
            1_000
        );
        assert_eq!(report.cost_summary.total_route_tip_landed_lamports, 1_000);
        assert_eq!(report.cost_summary.total_landed_spend_lamports, 6_198);
        assert_eq!(report.source_summaries[0].count, 1);
    }

    #[test]
    fn report_aggregates_unobserved_resend_attempts_by_signature() {
        let dir = tempdir().expect("tempdir");
        let root = dir.path();
        fs::write(
            root.join("samples.ndjson"),
            r#"{"schema_version":5,"test_id":"run-resend","iteration":0,"signature":"caller-sig","provider_name":"p","provider_kind":"rpcedge_quic_raw_tx","accepted":true,"client_started_at":"2026-01-01T00:00:00Z","client_finished_at":"2026-01-01T00:00:00.001Z","client_ack_latency_us":1000,"provider_send_started_at":"2026-01-01T00:00:00Z","provider_send_finished_at":"2026-01-01T00:00:00.001Z","provider_ack_latency_us":1000,"provider_request_id":"run-resend-0","returned_signature":"caller-sig","status_code":null,"error_class":null,"error":null,"resend_attempt_index":0,"resend_attempts":2,"resend_interval_ms":50,"route_policy":"tpu_quic_only","route_mode":"only","selected_routes":["tpu_quic"],"leader_client_family":"jito","compute_unit_limit":660,"compute_unit_price_microlamports":0,"base_fee_lamports":5000,"priority_fee_lamports":0,"transaction_fee_lamports":5000,"route_results":[{"route":"tpu_quic","route_name":"yellowstone-tpu-frankfurt","provider":"yellowstone-tpu","accepted":true,"latency_us":2}]}
{"schema_version":5,"test_id":"run-resend","iteration":0,"signature":"caller-sig","provider_name":"p","provider_kind":"rpcedge_quic_raw_tx","accepted":true,"client_started_at":"2026-01-01T00:00:00.050Z","client_finished_at":"2026-01-01T00:00:00.052Z","client_ack_latency_us":2000,"provider_send_started_at":"2026-01-01T00:00:00.050Z","provider_send_finished_at":"2026-01-01T00:00:00.052Z","provider_ack_latency_us":2000,"provider_request_id":"run-resend-1","returned_signature":"caller-sig","status_code":null,"error_class":null,"error":null,"resend_attempt_index":1,"resend_attempts":2,"resend_interval_ms":50,"route_policy":"tpu_quic_only","route_mode":"only","selected_routes":["tpu_quic"],"leader_client_family":"jito","compute_unit_limit":660,"compute_unit_price_microlamports":0,"base_fee_lamports":5000,"priority_fee_lamports":0,"transaction_fee_lamports":5000,"route_results":[{"route":"tpu_quic","route_name":"yellowstone-tpu-frankfurt","provider":"yellowstone-tpu","accepted":true,"latency_us":3}]}"#,
        )
        .expect("samples");
        fs::write(
            root.join("leader-sends.ndjson"),
            r#"{"schema_version":3,"test_id":"run-resend","iteration":0,"signature":"caller-sig","leader_identity":"leader-a","leader_run_start_slot":100,"leader_run_end_slot":103,"send_slot":100,"sent_at":"2026-01-01T00:00:00Z","trigger_source":"grpc_slot","slot_signal_status":"SLOT_FIRST_SHRED_RECEIVED","slot_signal_observed_at":"2026-01-01T00:00:00Z","leader_client_family":"jito","route_policy":"tpu_quic_only","selected_routes":["tpu_quic"],"compute_unit_limit":660,"compute_unit_price_microlamports":0,"base_fee_lamports":5000,"priority_fee_lamports":0,"transaction_fee_lamports":5000}"#,
        )
        .expect("sends");
        fs::write(root.join("matched-observations.ndjson"), "").expect("observations");

        let report = generate_report(root).expect("report");
        assert_eq!(report.totals.sent_transactions, 1);
        assert_eq!(report.totals.matched_signatures, 0);
        assert_eq!(report.totals.unobserved_signatures, 1);
        assert_eq!(report.provider_summaries[0].count, 2);
        assert_eq!(report.unobserved_events.len(), 1);
        let unobserved = &report.unobserved_events[0];
        assert_eq!(unobserved.provider_attempts, 2);
        assert_eq!(unobserved.provider_accepted_attempts, 2);
        assert_eq!(unobserved.provider_ack_latency_us_min, Some(1000));
        assert_eq!(unobserved.provider_ack_latency_us_max, Some(2000));
        assert_eq!(unobserved.route_results.len(), 2);
    }
}
