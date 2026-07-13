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

command -v jq >/dev/null || { echo "error: jq is required" >&2; exit 1; }
[[ -x "$bin" ]] || { echo "error: benchmark binary not executable: $bin" >&2; exit 1; }
for file in manifest.json leader-sends.ndjson matched-observations.ndjson \
  entry-placements.ndjson rpcedge-observations/entries.ndjson; do
  [[ -r "${run_dir}/${file}" ]] || { echo "error: missing artifact: $file" >&2; exit 1; }
done
compgen -G "${run_dir}/leader-slots-snapshot*.json" >/dev/null || {
  echo "error: no saved getLeaderSlots snapshot" >&2
  exit 1
}

jq -e '
  .route_strategy == "paired_providers"
  and (.providers | map(.name) == ["public_internet", "doublezero"])
' "${run_dir}/manifest.json" >/dev/null || {
  echo "error: artifact is not a public_internet / doublezero paired run" >&2
  exit 1
}

if [[ -d "$output_dir" ]] && find "$output_dir" -mindepth 1 -print -quit | grep -q .; then
  echo "error: output directory is not empty: $output_dir" >&2
  exit 1
fi

"$bin" compare \
  --artifact-dir "$run_dir" \
  --output-dir "$output_dir" \
  --sender-latitude 50.1109 \
  --sender-longitude 8.6821 \
  --title "Frankfurt DoubleZero TPU vs Public Internet" >/dev/null

printf 'report=%s\n' "${output_dir}/index.html"
printf 'summary=%s\n' "${output_dir}/summary.json"
