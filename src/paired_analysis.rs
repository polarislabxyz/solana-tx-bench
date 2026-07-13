use crate::{
    artifact_io::{read_leader_slot_snapshots, read_ndjson, visit_ndjson},
    leader_paced::{LeaderRunDecisionEvent, LeaderSendEvent},
    observations::{EntryEvent, EntryPlacementEvent, ObservationEvent, ObservationSourceKind},
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

#[derive(Debug, Clone, Copy)]
pub(crate) struct SenderLocation {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairedAnalysis {
    pub schema_version: u32,
    pub baseline_arm: String,
    pub candidate_arm: String,
    pub delta_convention: String,
    pub coverage: LeaderCoverage,
    pub landing_coverage: LandingCoverage,
    pub same_slot_metrics: ComparisonMetrics,
    pub by_city: Vec<CohortSummary>,
    pub by_distance: Vec<CohortSummary>,
    pub by_validator: Vec<CohortSummary>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LeaderCoverage {
    pub observed_leader_runs: usize,
    pub eligible_leader_runs: usize,
    pub observed_scheduled_slots: u64,
    pub eligible_scheduled_slots: u64,
    pub eligible_leader_run_pct: f64,
    pub eligible_scheduled_slot_pct: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LandingCoverage {
    pub comparison_groups: usize,
    pub distinct_leader_identities: usize,
    pub same_slot_pairs: usize,
    pub different_slot_pairs: usize,
    pub baseline_only: usize,
    pub candidate_only: usize,
    pub neither_observed: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComparisonMetrics {
    pub first_observed_ms: MetricSummary,
    pub processed_transaction_index: MetricSummary,
    pub entry_index: MetricSummary,
    pub preceding_poh_ticks: MetricSummary,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MetricSummary {
    pub pairs: usize,
    pub candidate_earlier: usize,
    pub same: usize,
    pub baseline_earlier: usize,
    pub delta_p50: Option<f64>,
    pub delta_p95: Option<f64>,
    pub delta_p99: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CohortSummary {
    pub label: String,
    pub validators: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identity: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latitude: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub longitude: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub distance_km: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed_candidate_rtt_us: Option<u64>,
    pub metrics: ComparisonMetrics,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PairStatus {
    SameSlot,
    DifferentSlot,
    BaselineOnly,
    CandidateOnly,
    NeitherObserved,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairedRow {
    pub comparison_group_id: String,
    pub leader_identity: String,
    pub leader_city: Option<String>,
    pub leader_country: Option<String>,
    pub leader_latitude: Option<f64>,
    pub leader_longitude: Option<f64>,
    pub distance_km: Option<f64>,
    pub candidate_route_rtt_us: Option<u64>,
    pub baseline_signature: String,
    pub candidate_signature: String,
    pub status: PairStatus,
    pub baseline_first_source: Option<String>,
    pub candidate_first_source: Option<String>,
    pub baseline_observed_slot: Option<u64>,
    pub candidate_observed_slot: Option<u64>,
    pub first_observed_delta_ms: Option<f64>,
    pub baseline_processed_transaction_index: Option<u64>,
    pub candidate_processed_transaction_index: Option<u64>,
    pub processed_transaction_index_delta: Option<i64>,
    pub baseline_entry_index: Option<u64>,
    pub candidate_entry_index: Option<u64>,
    pub entry_index_delta: Option<i64>,
    pub baseline_preceding_poh_ticks: Option<u64>,
    pub candidate_preceding_poh_ticks: Option<u64>,
    pub preceding_poh_ticks_delta: Option<i64>,
}

#[derive(Debug, Clone, Default)]
struct LeaderLocation {
    city: Option<String>,
    country: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
}

pub(crate) fn analyze_paired_artifact(
    artifact_dir: &Path,
    sender_location: Option<SenderLocation>,
) -> Result<Option<(PairedAnalysis, Vec<PairedRow>)>> {
    let sends: Vec<LeaderSendEvent> = read_ndjson(&artifact_dir.join("leader-sends.ndjson"))?;
    let arms = policy_arms(&sends)?;
    let Some((baseline_arm, candidate_arm)) = arms else {
        return Ok(None);
    };

    let groups = paired_sends(&sends, &baseline_arm, &candidate_arm);
    if groups.is_empty() {
        return Ok(None);
    }

    let observations: Vec<ObservationEvent> =
        read_ndjson(&artifact_dir.join("matched-observations.ndjson"))?;
    let placements: Vec<EntryPlacementEvent> =
        read_ndjson(&artifact_dir.join("entry-placements.ndjson"))?;
    let decisions: Vec<LeaderRunDecisionEvent> =
        read_ndjson(&artifact_dir.join("leader-runs.ndjson"))?;
    let locations = location_by_slot(artifact_dir)?;
    let first_by_signature = earliest_caller_observations(&observations);
    let processed_by_signature = earliest_processed_observations(&observations);
    let placement_by_signature = earliest_placements(&placements);

    let mut rows = groups
        .into_iter()
        .map(|(group_id, sends)| {
            build_row(
                group_id,
                sends[&baseline_arm],
                sends[&candidate_arm],
                &first_by_signature,
                &processed_by_signature,
                &placement_by_signature,
                &locations,
                sender_location,
            )
        })
        .collect::<Vec<_>>();
    attach_poh_ticks(artifact_dir, &mut rows)?;

    let analysis = PairedAnalysis {
        schema_version: 2,
        baseline_arm,
        candidate_arm,
        delta_convention: "candidate_minus_baseline".to_string(),
        coverage: summarize_leader_coverage(&decisions),
        landing_coverage: summarize_landing_coverage(&rows),
        same_slot_metrics: summarize_metrics_owned(&rows),
        by_city: summarize_cohorts(&rows, CohortKind::City),
        by_distance: summarize_cohorts(&rows, CohortKind::Distance),
        by_validator: summarize_cohorts(&rows, CohortKind::Validator),
    };
    Ok(Some((analysis, rows)))
}

fn policy_arms(sends: &[LeaderSendEvent]) -> Result<Option<(String, String)>> {
    let arms = sends
        .iter()
        .filter_map(|send| Some((send.policy_arm_index?, send.policy_arm.clone()?)))
        .collect::<BTreeMap<_, _>>();
    if arms.is_empty() {
        return Ok(None);
    }
    if arms.len() != 2 {
        return Ok(None);
    }
    let mut arms = arms.into_values();
    Ok(Some((
        arms.next().expect("two arms"),
        arms.next().expect("two arms"),
    )))
}

fn paired_sends<'a>(
    sends: &'a [LeaderSendEvent],
    baseline_arm: &str,
    candidate_arm: &str,
) -> BTreeMap<String, BTreeMap<String, &'a LeaderSendEvent>> {
    let mut groups = BTreeMap::<String, BTreeMap<String, &LeaderSendEvent>>::new();
    for send in sends {
        let (Some(group_id), Some(arm)) = (&send.comparison_group_id, &send.policy_arm) else {
            continue;
        };
        groups
            .entry(group_id.clone())
            .or_default()
            .entry(arm.clone())
            .and_modify(|current| {
                if send.sent_at < current.sent_at {
                    *current = send;
                }
            })
            .or_insert(send);
    }
    groups.retain(|_, arms| arms.contains_key(baseline_arm) && arms.contains_key(candidate_arm));
    groups
}

fn earliest_caller_observations(
    observations: &[ObservationEvent],
) -> BTreeMap<String, ObservationEvent> {
    earliest_observations(observations.iter().filter(|event| {
        event.signature_role.as_deref() != Some("route_tip") && event.caller_signature.is_none()
    }))
}

fn earliest_processed_observations(
    observations: &[ObservationEvent],
) -> BTreeMap<String, ObservationEvent> {
    earliest_observations(observations.iter().filter(|event| {
        event.signature_role.as_deref() != Some("route_tip")
            && event.caller_signature.is_none()
            && (event.source_kind == ObservationSourceKind::YellowstoneProcessed
                || event.source_name.to_ascii_lowercase().contains("processed"))
    }))
}

fn earliest_observations<'a>(
    observations: impl Iterator<Item = &'a ObservationEvent>,
) -> BTreeMap<String, ObservationEvent> {
    let mut out = BTreeMap::new();
    for event in observations {
        out.entry(event.signature.clone())
            .and_modify(|current: &mut ObservationEvent| {
                if event.observed_at < current.observed_at {
                    *current = event.clone();
                }
            })
            .or_insert_with(|| event.clone());
    }
    out
}

fn earliest_placements(
    placements: &[EntryPlacementEvent],
) -> BTreeMap<String, EntryPlacementEvent> {
    let mut out = BTreeMap::new();
    for placement in placements {
        out.entry(placement.signature.clone())
            .and_modify(|current: &mut EntryPlacementEvent| {
                if placement.observed_at < current.observed_at {
                    *current = placement.clone();
                }
            })
            .or_insert_with(|| placement.clone());
    }
    out
}

#[allow(clippy::too_many_arguments)]
fn build_row(
    comparison_group_id: String,
    baseline_send: &LeaderSendEvent,
    candidate_send: &LeaderSendEvent,
    first: &BTreeMap<String, ObservationEvent>,
    processed: &BTreeMap<String, ObservationEvent>,
    placements: &BTreeMap<String, EntryPlacementEvent>,
    locations: &BTreeMap<u64, (String, LeaderLocation)>,
    sender_location: Option<SenderLocation>,
) -> PairedRow {
    let baseline_observation = first.get(&baseline_send.signature);
    let candidate_observation = first.get(&candidate_send.signature);
    let baseline_slot = baseline_observation.and_then(|event| event.slot);
    let candidate_slot = candidate_observation.and_then(|event| event.slot);
    let status = match (baseline_observation, candidate_observation) {
        (Some(_), Some(_)) if baseline_slot.is_some() && baseline_slot == candidate_slot => {
            PairStatus::SameSlot
        }
        (Some(_), Some(_)) => PairStatus::DifferentSlot,
        (Some(_), None) => PairStatus::BaselineOnly,
        (None, Some(_)) => PairStatus::CandidateOnly,
        (None, None) => PairStatus::NeitherObserved,
    };
    let same_slot = (status == PairStatus::SameSlot)
        .then_some(baseline_slot)
        .flatten();

    let baseline_processed = processed
        .get(&baseline_send.signature)
        .filter(|event| event.slot == same_slot);
    let candidate_processed = processed
        .get(&candidate_send.signature)
        .filter(|event| event.slot == same_slot);
    let baseline_placement = placements
        .get(&baseline_send.signature)
        .filter(|placement| Some(placement.slot) == same_slot);
    let candidate_placement = placements
        .get(&candidate_send.signature)
        .filter(|placement| Some(placement.slot) == same_slot);

    let location = locations
        .get(&baseline_send.send_slot)
        .filter(|(identity, _)| identity == &baseline_send.leader_identity)
        .map(|(_, location)| location)
        .cloned()
        .unwrap_or_else(|| LeaderLocation {
            city: baseline_send.leader_city.clone(),
            ..LeaderLocation::default()
        });
    let distance_km = sender_location
        .zip(location.latitude.zip(location.longitude))
        .map(|(sender, (latitude, longitude))| {
            haversine_km(sender.latitude, sender.longitude, latitude, longitude)
        });

    let baseline_latency = same_slot.and_then(|_| {
        baseline_observation.and_then(|event| observation_latency_ms(event, baseline_send))
    });
    let candidate_latency = same_slot.and_then(|_| {
        candidate_observation.and_then(|event| observation_latency_ms(event, candidate_send))
    });
    let baseline_index = baseline_processed.and_then(|event| event.slot_index);
    let candidate_index = candidate_processed.and_then(|event| event.slot_index);
    let baseline_entry = baseline_placement.map(|placement| placement.entry_index);
    let candidate_entry = candidate_placement.map(|placement| placement.entry_index);

    PairedRow {
        comparison_group_id,
        leader_identity: baseline_send.leader_identity.clone(),
        leader_city: location.city,
        leader_country: location.country,
        leader_latitude: location.latitude,
        leader_longitude: location.longitude,
        distance_km,
        candidate_route_rtt_us: candidate_send.leader_network_path_rtt_us,
        baseline_signature: baseline_send.signature.clone(),
        candidate_signature: candidate_send.signature.clone(),
        status,
        baseline_first_source: baseline_observation.map(|event| event.source_name.clone()),
        candidate_first_source: candidate_observation.map(|event| event.source_name.clone()),
        baseline_observed_slot: baseline_slot,
        candidate_observed_slot: candidate_slot,
        first_observed_delta_ms: baseline_latency
            .zip(candidate_latency)
            .map(|(baseline, candidate)| candidate - baseline),
        baseline_processed_transaction_index: baseline_index,
        candidate_processed_transaction_index: candidate_index,
        processed_transaction_index_delta: delta_u64(baseline_index, candidate_index),
        baseline_entry_index: baseline_entry,
        candidate_entry_index: candidate_entry,
        entry_index_delta: delta_u64(baseline_entry, candidate_entry),
        baseline_preceding_poh_ticks: None,
        candidate_preceding_poh_ticks: None,
        preceding_poh_ticks_delta: None,
    }
}

fn observation_latency_ms(event: &ObservationEvent, send: &LeaderSendEvent) -> Option<f64> {
    let submitted_at = event.submitted_at.as_ref().unwrap_or(&send.sent_at);
    event
        .observed_at
        .signed_duration_since(submitted_at.to_owned())
        .num_microseconds()
        .map(|value| value as f64 / 1_000.0)
}

fn delta_u64(baseline: Option<u64>, candidate: Option<u64>) -> Option<i64> {
    baseline
        .zip(candidate)
        .map(|(baseline, candidate)| candidate as i64 - baseline as i64)
}

#[derive(Default)]
struct TickState {
    expected_entry: u64,
    preceding_ticks: u64,
    valid: bool,
}

fn attach_poh_ticks(artifact_dir: &Path, rows: &mut [PairedRow]) -> Result<()> {
    let targets = rows
        .iter()
        .filter(|row| row.status == PairStatus::SameSlot)
        .flat_map(|row| {
            let slot = row.baseline_observed_slot;
            [row.baseline_entry_index, row.candidate_entry_index]
                .into_iter()
                .flatten()
                .filter_map(move |entry| slot.map(|slot| (slot, entry)))
        })
        .collect::<BTreeSet<_>>();
    if targets.is_empty() {
        return Ok(());
    }
    let target_slots = targets
        .iter()
        .map(|(slot, _)| *slot)
        .collect::<BTreeSet<_>>();
    let mut states = BTreeMap::<u64, TickState>::new();
    let mut ticks_by_entry = BTreeMap::<(u64, u64), u64>::new();
    visit_ndjson::<EntryEvent>(
        &artifact_dir.join("rpcedge-observations/entries.ndjson"),
        |entry| {
            if !target_slots.contains(&entry.slot) {
                return Ok(());
            }
            let state = states.entry(entry.slot).or_insert_with(|| TickState {
                valid: true,
                ..TickState::default()
            });
            if !state.valid || entry.entry_index < state.expected_entry {
                return Ok(());
            }
            if entry.entry_index != state.expected_entry {
                state.valid = false;
                return Ok(());
            }
            if targets.contains(&(entry.slot, entry.entry_index)) {
                ticks_by_entry.insert((entry.slot, entry.entry_index), state.preceding_ticks);
            }
            if entry.executed_transaction_count == 0 {
                state.preceding_ticks += 1;
            }
            state.expected_entry += 1;
            Ok(())
        },
    )?;

    for row in rows
        .iter_mut()
        .filter(|row| row.status == PairStatus::SameSlot)
    {
        let Some(slot) = row.baseline_observed_slot else {
            continue;
        };
        row.baseline_preceding_poh_ticks = row
            .baseline_entry_index
            .and_then(|entry| ticks_by_entry.get(&(slot, entry)).copied());
        row.candidate_preceding_poh_ticks = row
            .candidate_entry_index
            .and_then(|entry| ticks_by_entry.get(&(slot, entry)).copied());
        row.preceding_poh_ticks_delta = delta_u64(
            row.baseline_preceding_poh_ticks,
            row.candidate_preceding_poh_ticks,
        );
    }
    Ok(())
}

fn location_by_slot(artifact_dir: &Path) -> Result<BTreeMap<u64, (String, LeaderLocation)>> {
    let mut out = BTreeMap::new();
    for snapshot in read_leader_slot_snapshots(artifact_dir)? {
        let rows = snapshot
            .response
            .get("result")
            .and_then(|result| result.get("data"))
            .and_then(Value::as_array)
            .or_else(|| snapshot.response.get("data").and_then(Value::as_array));
        let Some(rows) = rows else { continue };
        for row in rows {
            let (Some(slot), Some(identity)) = (
                row.get("slot").and_then(Value::as_u64),
                row.get("identity").and_then(Value::as_str),
            ) else {
                continue;
            };
            let location = row.get("location").unwrap_or(&Value::Null);
            out.entry(slot).or_insert_with(|| {
                (
                    identity.to_string(),
                    LeaderLocation {
                        city: string_field(location, "city"),
                        country: string_field(location, "country"),
                        latitude: location.get("latitude").and_then(Value::as_f64),
                        longitude: location.get("longitude").and_then(Value::as_f64),
                    },
                )
            });
        }
    }
    Ok(out)
}

fn string_field(value: &Value, field: &str) -> Option<String> {
    let value = value
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(ToOwned::to_owned)?;
    if field == "city" {
        let city = value.rsplit('/').next().unwrap_or(&value).replace('_', " ");
        Some(city)
    } else {
        Some(value)
    }
}

fn summarize_leader_coverage(decisions: &[LeaderRunDecisionEvent]) -> LeaderCoverage {
    let run_slots = |row: &LeaderRunDecisionEvent| {
        row.leader_run_end_slot
            .saturating_sub(row.leader_run_start_slot)
            .saturating_add(1)
    };
    let observed_scheduled_slots = decisions.iter().map(run_slots).sum::<u64>();
    let eligible_scheduled_slots = decisions
        .iter()
        .filter(|row| row.eligible)
        .map(run_slots)
        .sum::<u64>();
    let eligible_leader_runs = decisions.iter().filter(|row| row.eligible).count();
    LeaderCoverage {
        observed_leader_runs: decisions.len(),
        eligible_leader_runs,
        observed_scheduled_slots,
        eligible_scheduled_slots,
        eligible_leader_run_pct: percent(eligible_leader_runs, decisions.len()),
        eligible_scheduled_slot_pct: percent_u64(
            eligible_scheduled_slots,
            observed_scheduled_slots,
        ),
    }
}

fn summarize_landing_coverage(rows: &[PairedRow]) -> LandingCoverage {
    LandingCoverage {
        comparison_groups: rows.len(),
        distinct_leader_identities: rows
            .iter()
            .map(|row| row.leader_identity.as_str())
            .collect::<BTreeSet<_>>()
            .len(),
        same_slot_pairs: count_status(rows, PairStatus::SameSlot),
        different_slot_pairs: count_status(rows, PairStatus::DifferentSlot),
        baseline_only: count_status(rows, PairStatus::BaselineOnly),
        candidate_only: count_status(rows, PairStatus::CandidateOnly),
        neither_observed: count_status(rows, PairStatus::NeitherObserved),
    }
}

fn count_status(rows: &[PairedRow], status: PairStatus) -> usize {
    rows.iter().filter(|row| row.status == status).count()
}

fn summarize_metrics(rows: &[&PairedRow]) -> ComparisonMetrics {
    ComparisonMetrics {
        first_observed_ms: metric(rows.iter().filter_map(|row| row.first_observed_delta_ms)),
        processed_transaction_index: metric(rows.iter().filter_map(|row| {
            row.processed_transaction_index_delta
                .map(|value| value as f64)
        })),
        entry_index: metric(
            rows.iter()
                .filter_map(|row| row.entry_index_delta.map(|value| value as f64)),
        ),
        preceding_poh_ticks: metric(
            rows.iter()
                .filter_map(|row| row.preceding_poh_ticks_delta.map(|value| value as f64)),
        ),
    }
}

fn summarize_metrics_owned(rows: &[PairedRow]) -> ComparisonMetrics {
    summarize_metrics(&rows.iter().collect::<Vec<_>>())
}

fn metric(values: impl Iterator<Item = f64>) -> MetricSummary {
    let mut values = values.collect::<Vec<_>>();
    values.sort_by(f64::total_cmp);
    MetricSummary {
        pairs: values.len(),
        candidate_earlier: values.iter().filter(|value| **value < 0.0).count(),
        same: values.iter().filter(|value| **value == 0.0).count(),
        baseline_earlier: values.iter().filter(|value| **value > 0.0).count(),
        delta_p50: percentile(&values, 0.50),
        delta_p95: percentile(&values, 0.95),
        delta_p99: percentile(&values, 0.99),
    }
}

#[derive(Clone, Copy)]
enum CohortKind {
    City,
    Distance,
    Validator,
}

fn summarize_cohorts(rows: &[PairedRow], kind: CohortKind) -> Vec<CohortSummary> {
    let mut groups = BTreeMap::<String, Vec<&PairedRow>>::new();
    for row in rows.iter().filter(|row| row.status == PairStatus::SameSlot) {
        groups.entry(cohort_key(row, kind)).or_default().push(row);
    }
    let mut cohorts = groups
        .into_iter()
        .map(|(label, rows)| cohort_summary(label, &rows, kind))
        .collect::<Vec<_>>();
    cohorts.sort_by(|left, right| {
        right
            .metrics
            .first_observed_ms
            .pairs
            .cmp(&left.metrics.first_observed_ms.pairs)
            .then_with(|| left.label.cmp(&right.label))
    });
    cohorts
}

fn cohort_key(row: &PairedRow, kind: CohortKind) -> String {
    match kind {
        CohortKind::City => match (&row.leader_city, &row.leader_country) {
            (Some(city), Some(country)) => format!("{city}, {country}"),
            (Some(city), None) => city.clone(),
            _ => "Location unavailable".to_string(),
        },
        CohortKind::Distance => match row.distance_km {
            Some(distance) if distance < 1_000.0 => "Under 1,000 km".to_string(),
            Some(distance) if distance < 5_000.0 => "1,000-5,000 km".to_string(),
            Some(_) => "At least 5,000 km".to_string(),
            None => "Coordinate unavailable".to_string(),
        },
        CohortKind::Validator => row.leader_identity.clone(),
    }
}

fn cohort_summary(label: String, rows: &[&PairedRow], kind: CohortKind) -> CohortSummary {
    let first = rows[0];
    let is_validator = matches!(kind, CohortKind::Validator);
    let has_named_location = matches!(kind, CohortKind::City | CohortKind::Validator);
    CohortSummary {
        label,
        validators: rows
            .iter()
            .map(|row| row.leader_identity.as_str())
            .collect::<BTreeSet<_>>()
            .len(),
        identity: is_validator.then(|| first.leader_identity.clone()),
        city: has_named_location
            .then(|| first.leader_city.clone())
            .flatten(),
        country: has_named_location
            .then(|| first.leader_country.clone())
            .flatten(),
        latitude: is_validator.then_some(first.leader_latitude).flatten(),
        longitude: is_validator.then_some(first.leader_longitude).flatten(),
        distance_km: is_validator.then_some(first.distance_km).flatten(),
        observed_candidate_rtt_us: rows
            .iter()
            .filter_map(|row| row.candidate_route_rtt_us)
            .min(),
        metrics: summarize_metrics(rows),
    }
}

fn percentile(sorted: &[f64], percentile: f64) -> Option<f64> {
    if sorted.is_empty() {
        return None;
    }
    let index = ((sorted.len() - 1) as f64 * percentile).ceil() as usize;
    sorted.get(index).copied()
}

fn percent(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64 * 100.0
    }
}

fn percent_u64(numerator: u64, denominator: u64) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64 * 100.0
    }
}

fn haversine_km(from_lat: f64, from_lon: f64, to_lat: f64, to_lon: f64) -> f64 {
    let earth_radius_km = 6_371.0;
    let latitude_delta = (to_lat - from_lat).to_radians();
    let longitude_delta = (to_lon - from_lon).to_radians();
    let a = (latitude_delta / 2.0).sin().powi(2)
        + from_lat.to_radians().cos()
            * to_lat.to_radians().cos()
            * (longitude_delta / 2.0).sin().powi(2);
    earth_radius_km * 2.0 * a.sqrt().asin()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn metrics_use_only_present_same_slot_deltas() {
        let values = metric([-4.0, 0.0, 2.0].into_iter());
        assert_eq!(values.pairs, 3);
        assert_eq!(values.candidate_earlier, 1);
        assert_eq!(values.same, 1);
        assert_eq!(values.baseline_earlier, 1);
        assert_eq!(values.delta_p50, Some(0.0));
        assert_eq!(values.delta_p95, Some(2.0));
        assert_eq!(values.delta_p99, Some(2.0));
    }

    #[test]
    fn distance_uses_saved_coordinates() {
        let distance = haversine_km(50.1109, 8.6821, 45.3151, -73.8736);
        assert!((distance - 5_876.0).abs() < 25.0);
    }

    #[test]
    fn paired_analysis_uses_one_earliest_clock_and_same_slot_placement() {
        let dir = tempdir().expect("tempdir");
        let root = dir.path();
        fs::create_dir(root.join("rpcedge-observations")).expect("observation dir");
        fs::write(
            root.join("leader-sends.ndjson"),
            concat!(
                "{\"schema_version\":3,\"test_id\":\"paired\",\"iteration\":0,\"signature\":\"baseline\",\"comparison_group_id\":\"group\",\"policy_arm\":\"public\",\"policy_arm_index\":0,\"leader_identity\":\"leader\",\"leader_run_start_slot\":100,\"leader_run_end_slot\":103,\"send_slot\":100,\"sent_at\":\"2026-01-01T00:00:00Z\",\"trigger_source\":\"grpc_slot\",\"slot_signal_status\":null,\"slot_signal_observed_at\":null,\"leader_client_family\":\"agave\",\"route_policy\":\"public\",\"selected_routes\":[\"tpu_quic\"],\"compute_unit_limit\":660,\"compute_unit_price_microlamports\":0,\"base_fee_lamports\":5000,\"priority_fee_lamports\":0,\"transaction_fee_lamports\":5000}\n",
                "{\"schema_version\":3,\"test_id\":\"paired\",\"iteration\":1,\"signature\":\"candidate\",\"comparison_group_id\":\"group\",\"policy_arm\":\"doublezero\",\"policy_arm_index\":1,\"leader_identity\":\"leader\",\"leader_run_start_slot\":100,\"leader_run_end_slot\":103,\"send_slot\":100,\"sent_at\":\"2026-01-01T00:00:00Z\",\"trigger_source\":\"grpc_slot\",\"slot_signal_status\":null,\"slot_signal_observed_at\":null,\"leader_client_family\":\"agave\",\"route_policy\":\"doublezero\",\"selected_routes\":[\"tpu_quic\"],\"compute_unit_limit\":660,\"compute_unit_price_microlamports\":0,\"base_fee_lamports\":5000,\"priority_fee_lamports\":0,\"transaction_fee_lamports\":5000}\n"
            ),
        )
        .expect("sends");
        fs::write(
            root.join("matched-observations.ndjson"),
            concat!(
                "{\"schema_version\":1,\"test_id\":\"paired\",\"signature\":\"baseline\",\"source_name\":\"processed\",\"source_kind\":\"yellowstone_processed\",\"observed_at\":\"2026-01-01T00:00:00.120Z\",\"submitted_at\":\"2026-01-01T00:00:00Z\",\"slot\":100,\"slot_index\":20,\"source_sequence\":null}\n",
                "{\"schema_version\":1,\"test_id\":\"paired\",\"signature\":\"baseline\",\"source_name\":\"deshred\",\"source_kind\":\"yellowstone_deshred\",\"observed_at\":\"2026-01-01T00:00:00.100Z\",\"submitted_at\":\"2026-01-01T00:00:00Z\",\"slot\":100,\"slot_index\":null,\"source_sequence\":null}\n",
                "{\"schema_version\":1,\"test_id\":\"paired\",\"signature\":\"candidate\",\"source_name\":\"processed\",\"source_kind\":\"yellowstone_processed\",\"observed_at\":\"2026-01-01T00:00:00.130Z\",\"submitted_at\":\"2026-01-01T00:00:00Z\",\"slot\":100,\"slot_index\":10,\"source_sequence\":null}\n",
                "{\"schema_version\":1,\"test_id\":\"paired\",\"signature\":\"candidate\",\"source_name\":\"deshred\",\"source_kind\":\"yellowstone_deshred\",\"observed_at\":\"2026-01-01T00:00:00.090Z\",\"submitted_at\":\"2026-01-01T00:00:00Z\",\"slot\":100,\"slot_index\":null,\"source_sequence\":null}\n"
            ),
        )
        .expect("observations");
        fs::write(
            root.join("entry-placements.ndjson"),
            concat!(
                "{\"schema_version\":1,\"test_id\":\"paired\",\"signature\":\"baseline\",\"source_name\":\"entry\",\"source_kind\":\"yellowstone_entry\",\"observed_at\":\"2026-01-01T00:00:00.140Z\",\"slot\":100,\"slot_index\":20,\"entry_index\":3,\"tx_index_in_entry\":0,\"entry_num_hashes\":1,\"entry_hash\":\"b\",\"entry_executed_transaction_count\":1,\"entry_starting_transaction_index\":20}\n",
                "{\"schema_version\":1,\"test_id\":\"paired\",\"signature\":\"candidate\",\"source_name\":\"entry\",\"source_kind\":\"yellowstone_entry\",\"observed_at\":\"2026-01-01T00:00:00.140Z\",\"slot\":100,\"slot_index\":10,\"entry_index\":1,\"tx_index_in_entry\":0,\"entry_num_hashes\":1,\"entry_hash\":\"c\",\"entry_executed_transaction_count\":1,\"entry_starting_transaction_index\":10}\n"
            ),
        )
        .expect("placements");
        let entries = (0..=3)
            .map(|entry_index| {
                let executed = usize::from(entry_index == 1 || entry_index == 3);
                format!("{{\"schema_version\":1,\"test_id\":\"paired\",\"source_name\":\"entry\",\"source_kind\":\"yellowstone_entry\",\"observed_at\":\"2026-01-01T00:00:00.140Z\",\"slot\":100,\"entry_index\":{entry_index},\"num_hashes\":1,\"entry_hash\":\"{entry_index}\",\"executed_transaction_count\":{executed},\"starting_transaction_index\":0}}\n")
            })
            .collect::<String>();
        fs::write(root.join("rpcedge-observations/entries.ndjson"), entries).expect("entries");
        fs::write(
            root.join("leader-runs.ndjson"),
            "{\"schema_version\":1,\"test_id\":\"paired\",\"leader_identity\":\"leader\",\"leader_run_start_slot\":100,\"leader_run_end_slot\":103,\"observed_slot\":100,\"observed_at\":\"2026-01-01T00:00:00Z\",\"eligible\":true,\"network_path_class\":\"doublezero\",\"network_path_verified\":true}\n",
        )
        .expect("decisions");

        let (analysis, rows) = analyze_paired_artifact(root, None)
            .expect("analysis")
            .expect("paired");
        assert_eq!(rows[0].baseline_first_source.as_deref(), Some("deshred"));
        assert_eq!(rows[0].candidate_first_source.as_deref(), Some("deshred"));
        assert_eq!(analysis.landing_coverage.same_slot_pairs, 1);
        assert_eq!(
            analysis.same_slot_metrics.first_observed_ms.delta_p50,
            Some(-10.0)
        );
        assert_eq!(
            analysis
                .same_slot_metrics
                .processed_transaction_index
                .delta_p50,
            Some(-10.0)
        );
        assert_eq!(analysis.same_slot_metrics.entry_index.delta_p50, Some(-2.0));
        assert_eq!(
            analysis.same_slot_metrics.preceding_poh_ticks.delta_p50,
            Some(-1.0)
        );
    }
}
