#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "usage: $0 <artifact-dir> [output-dir]" >&2
  exit 2
}

[[ $# -ge 1 && $# -le 2 ]] || usage

run_dir="$(realpath "$1")"
output_dir="${2:-${run_dir}/doublezero-public-comparison}"
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
bin="${SOLANA_TX_BENCH_BIN:-${repo_root}/target/release/solana-tx-bench}"

for command in jq awk sort realpath; do
  command -v "$command" >/dev/null || {
    echo "error: required command not found: $command" >&2
    exit 1
  }
done
[[ -x "$bin" ]] || { echo "error: benchmark binary not executable: $bin" >&2; exit 1; }
for file in manifest.json leader-sends.ndjson matched-observations.ndjson \
  entry-placements.ndjson rpcedge-observations/entries.ndjson; do
  [[ -r "${run_dir}/${file}" ]] || { echo "error: missing artifact: ${file}" >&2; exit 1; }
done

jq -e '
  .route_strategy == "paired_providers"
  and (.providers | map(.name) == ["public_internet", "doublezero"])
' "${run_dir}/manifest.json" >/dev/null || {
  echo "error: artifact is not a public_internet baseline / doublezero candidate run" >&2
  exit 1
}

mkdir -p "$output_dir"
"$bin" report --artifact-dir "$run_dir" >/dev/null
"$bin" compare \
  --artifact-dir "$run_dir" \
  --output-dir "$output_dir" \
  --primary-source rpcedge_processed \
  --title "Frankfurt DoubleZero TPU vs Public Internet" >/dev/null

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

jq -rn \
  --slurpfile rows "${output_dir}/paired-rows.json" \
  --slurpfile placements "${run_dir}/entry-placements.ndjson" '
  ($placements | map({key: .signature, value: .}) | from_entries) as $placement
  | $rows[0][]
  | select(.source_name == "rpcedge_processed" and .status == "both_observed")
  | select(.baseline_landed_slot != null and .baseline_landed_slot == .candidate_landed_slot)
  | ($placement[.baseline_signature]) as $baseline
  | ($placement[.candidate_signature]) as $candidate
  | select($baseline != null and $candidate != null)
  | [
      .comparison_group_id,
      .baseline_signature,
      .candidate_signature,
      .baseline_landed_slot,
      $baseline.entry_index,
      $candidate.entry_index
    ]
  | @tsv
' >"${tmp}/pairs.tsv"
[[ -s "${tmp}/pairs.tsv" ]] || {
  echo "error: no complete same-slot processed pairs with entry placements" >&2
  exit 1
}

awk -F '\t' '{print $4 "\t" $5; print $4 "\t" $6}' "${tmp}/pairs.tsv" \
  | sort -u >"${tmp}/targets.tsv"

jq -r '[.slot, .entry_index, .executed_transaction_count] | @tsv' \
  "${run_dir}/rpcedge-observations/entries.ndjson" \
  | awk -F '\t' -v OFS='\t' '
      NR == FNR { target[$1 FS $2] = 1; wanted_slot[$1] = 1; next }
      !($1 in wanted_slot) { next }
      !($1 in seen) { seen[$1] = 1; expected[$1] = 0; valid[$1] = 1 }
      $2 < expected[$1] { next }
      $2 != expected[$1] { valid[$1] = 0; next }
      valid[$1] && (($1 FS $2) in target) { print $1, $2, ticks[$1] }
      $3 == 0 { ticks[$1]++ }
      { expected[$1]++ }
    ' "${tmp}/targets.tsv" - >"${tmp}/ticks.tsv"

awk -F '\t' -v OFS='\t' '
  NR == FNR { tick[$1 FS $2] = $3; next }
  {
    baseline_tick = tick[$4 FS $5]
    candidate_tick = tick[$4 FS $6]
    tick_delta = (baseline_tick != "" && candidate_tick != "") \
      ? candidate_tick - baseline_tick : ""
    print $1, $2, $3, $4, $5, $6, $6 - $5, baseline_tick, candidate_tick, tick_delta
  }
' "${tmp}/ticks.tsv" "${tmp}/pairs.tsv" >"${tmp}/placement-rows.tsv"

jq -Rn '
  def optional_number: if . == "" then null else tonumber end;
  def median:
    if length == 0 then null else sort | .[((length - 1) * 0.5 | ceil)] end;
  [inputs | split("\t") | {
    comparison_group_id: .[0],
    baseline_signature: .[1],
    candidate_signature: .[2],
    slot: (.[3] | tonumber),
    baseline_entry_index: (.[4] | tonumber),
    candidate_entry_index: (.[5] | tonumber),
    entry_index_delta: (.[6] | tonumber),
    baseline_preceding_poh_ticks: (.[7] | optional_number),
    candidate_preceding_poh_ticks: (.[8] | optional_number),
    preceding_poh_ticks_delta: (.[9] | optional_number)
  }] as $rows
  | ($rows | map(.entry_index_delta)) as $entry
  | ($rows | map(.preceding_poh_ticks_delta) | map(select(. != null))) as $tick
  | {
      schema_version: 1,
      baseline_arm: "public_internet",
      candidate_arm: "doublezero",
      delta_convention: "candidate_minus_baseline",
      same_slot_pairs: ($rows | length),
      tick_complete_pairs: ($tick | length),
      entry: {
        candidate_earlier: ($entry | map(select(. < 0)) | length),
        ties: ($entry | map(select(. == 0)) | length),
        baseline_earlier: ($entry | map(select(. > 0)) | length),
        delta_p50: ($entry | median)
      },
      preceding_poh_ticks: {
        candidate_earlier: ($tick | map(select(. < 0)) | length),
        ties: ($tick | map(select(. == 0)) | length),
        baseline_earlier: ($tick | map(select(. > 0)) | length),
        delta_p50: ($tick | median)
      },
      rows: $rows
    }
' "${tmp}/placement-rows.tsv" >"${output_dir}/poh-placement.json"

jq -r '
  "# Same-Slot Entry and PoH Placement\n\n"
  + "Baseline is `public_internet`; candidate is `doublezero`. Deltas are candidate minus baseline. Negative values favor DoubleZero.\n\n"
  + "This is derived after the run from processed transaction indexes and the complete Yellowstone entry history. It describes where the leader recorded each transaction, not when the packet reached the TPU.\n\n"
  + "| Measure | Pairs | DoubleZero earlier | Tie | Public earlier | Delta p50 |\n"
  + "| --- | ---: | ---: | ---: | ---: | ---: |\n"
  + "| Entry index | \(.same_slot_pairs) | \(.entry.candidate_earlier) | \(.entry.ties) | \(.entry.baseline_earlier) | \(.entry.delta_p50) |\n"
  + "| Preceding PoH ticks | \(.tick_complete_pairs) | \(.preceding_poh_ticks.candidate_earlier) | \(.preceding_poh_ticks.ties) | \(.preceding_poh_ticks.baseline_earlier) | \(.preceding_poh_ticks.delta_p50) |\n"
' "${output_dir}/poh-placement.json" >"${output_dir}/poh-placement.md"

cp "${output_dir}/comparison.md" "${output_dir}/doublezero-public-report.md"
{
  printf '\n\n'
  sed '1d' "${output_dir}/poh-placement.md"
} >>"${output_dir}/doublezero-public-report.md"

printf 'comparison=%s\n' "${output_dir}/comparison.json"
printf 'poh_placement=%s\n' "${output_dir}/poh-placement.json"
printf 'report=%s\n' "${output_dir}/doublezero-public-report.md"
