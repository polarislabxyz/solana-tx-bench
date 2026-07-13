# solana-tx-bench

Reusable Solana transaction observation benchmark.

The tool generates small signed mainnet transactions, submits each transaction
to one or more configured providers, and writes reproducible landing and block-
placement artifacts from configured Yellowstone receivers.

It is intentionally provider-neutral. Add a provider by implementing a sender
adapter, not by hard-coding a benchmark path.

RPCEdge links:

- Website: https://rpcedge.com
- Docs: https://docs.rpcedge.com

## What It Measures

Primary measurement:

```text
signed transaction submitted
  -> observed on one or more configured receivers
  -> matched by signature
  -> earliest receiver observation selected for that transaction
  -> landing, missing-rate, and block-position comparison
```

Provider ACK latency is retained only as a diagnostic side channel. It is not
the benchmark result. Receiver-specific timing is also diagnostic: transaction-
sender reports use one landing clock, the earliest matched observation for each
signature across all configured receivers.

Leader geography, validator client, datacenter, route hints, and bad-leader
context should come from the saved `getLeaderSlots` snapshot when building the
public report. Private ClickHouse joins are optional internal diagnostics, not
required for the benchmarker.

## Supported Adapters

- `solana_rpc`: standard JSON-RPC `sendTransaction`.
- `rpcedge_raw_http`: raw transaction bytes over HTTP.
- `rpcedge_route_aware_http`: RPCEdge route-aware JSON submit endpoint.
- `rpcedge_quic_raw_tx`: raw transaction bytes over persistent RPCEdge QUIC.

Planned adapter examples:

- Astralane
- Helius Sender
- Harmonic
- Other provider-specific low-latency senders

## Quick Start

```bash
git clone <repo-url>
cd solana-tx-bench
cargo run -- init-config --output bench.yaml
```

Edit `bench.yaml`:

- set `keypair_path` to a funded low-value keypair;
- set `max_spend_lamports`;
- configure providers and API key environment variable names.

Run one transaction:

```bash
export RPCEDGE_API_KEY=...
cargo run -- run --config bench.yaml
```

Submission diagnostics are written under `artifact_dir/test_id/`:

- `manifest.json`
- `samples.ndjson`
- `summary.json`
- `summary.md`

Run a leader-paced smoke test:

```bash
cargo run --release -- run-leader-paced \
  --config bench.yaml \
  --duration-seconds 300 \
  --txs-per-leader-run 1 \
  --leader-run-concurrency 1 \
  --slot-trigger grpc_slot \
  --capture-leader-slots \
  --collect-rpcedge \
  --collect-entries
```

This sends at most one transaction for each observed contiguous leader run,
signing each transaction with a fresh blockhash. It is the preferred first
benchmark shape because it avoids spammy fixed-rate traffic and naturally
samples different leaders over the run window.

Set `--txs-per-leader-run N --leader-run-concurrency N` to send multiple
distinct transactions at the same time for each observed leader run. Keep the
default concurrency of `1` for baseline reports unless you are intentionally
testing burst behavior.

By default, `run-leader-paced` is driven by the RPCEdge Yellowstone gRPC slot
stream (`--slot-trigger grpc_slot`). This avoids `getSlot` polling for send
timing. `--slot-trigger rpc_poll` exists only as a debug fallback.

`--collect-entries` adds a normal Yellowstone entry subscription beside
SubscribeDeshred and processed transaction subscriptions. The runner stores
slot-wide `entries.ndjson` and derives `entry-placements.ndjson` by joining the
processed transaction `slot_index` into each entry's transaction range. This is
block-placement context for "did we land early in the produced block"; it is
not packet-arrival time at the leader and it is not an exact PoH tick label.

`--capture-leader-slots` calls JSON-RPC `getLeaderSlots` before the run and
writes `leader-slots-snapshot.json` beside the samples. When pointed at
RPCEdge's RPC gateway, that snapshot can include leader geography, validator
client, route hints, and historical landing-latency profiles. This makes cohort
reports reproducible from local artifacts instead of requiring a private
database join. If `--leader-slots-rpc-url` is omitted, the runner uses the
`rpc_url` from `bench.yaml`.

In `grpc_slot` mode the runner refreshes the leader-slot snapshot when the
gRPC-observed slot approaches or leaves the cached horizon. Refresh files are
written as `leader-slots-snapshot-<start_slot>.json`.

For a QUIC-only RPCEdge sender benchmark, configure a single provider:

```yaml
providers:
  - name: rpcedge-quic-frankfurt
    kind: rpcedge_quic_raw_tx
    endpoint: "185.191.118.181:4433"
    api_key_env: "RPCEDGE_API_KEY"
    route_mode: only
    routes:
      - tpu_quic
    server_name: "relay.rpcedge.com"
```

Use `run` for a tiny fixed-count canary, then `run-leader-paced` for the
leader/cohort benchmark.

Route-policy comparisons must use paired samples. Do not run TPU-only for one
time window and `always_race` for a later time window and call that an equal
comparison; the leader set, slot phase, network state, and fee market changed.

Use `paired_route_policies` for the fair comparison. For each leader run it
builds one transaction per policy arm and sends all arms from the same gRPC slot
signal:

```bash
cargo run --release -- run-leader-paced \
  --config examples/rpcedge-quic-frankfurt.yaml \
  --duration-seconds 1800 \
  --txs-per-leader-run 1 \
  --leader-run-concurrency 3 \
  --slot-trigger grpc_slot \
  --route-strategy paired_route_policies \
  --capture-leader-slots \
  --collect-rpcedge \
  --collect-entries
```

The paired arms are:

- `tpu_quic_only`: `only: [tpu_quic]`
- `always_race`: `only: [tpu_quic, jito_bundle, harmonic_bundle]`
- `software_client_aware`: JitoLabs/BAM/FireBAM get TPU+Jito, Harmonic*
  gets TPU+Harmonic, everything else gets TPU-only.

All paired baseline arms use the same compute-unit limit and the same
compute-unit price from the config. Route policy must not silently mutate
priority fee; fee ladders are separate experiments.

Arms that include `jito_bundle` may pay relay-side Jito tips. Those tips are
tracked separately from the signed transaction fee. For RPCEdge QUIC
benchmarks, set `route_results: true` on the provider to request bounded
route-result ACKs. When returned, reports show route-tip generated spend from
`route_tip_lamports`, and route-tip landed spend only when the corresponding
`route_tip_signature` is observed by the benchmark streams. Jito tip accounts
are included automatically in RPCEdge observation filters. Use
`--observation-account <pubkey>` only for custom/private provider accounts.

Individual strategies are route availability smoke tests only:

```bash
# Baseline: static TPU QUIC only from bench.yaml.
cargo run --release -- run-leader-paced \
  --config examples/rpcedge-quic-frankfurt.yaml \
  --duration-seconds 1800 \
  --slot-trigger grpc_slot \
  --capture-leader-slots \
  --collect-rpcedge

# Control: always race TPU, Jito bundle, and Harmonic bundle.
cargo run --release -- run-leader-paced \
  --config examples/rpcedge-quic-frankfurt.yaml \
  --duration-seconds 1800 \
  --slot-trigger grpc_slot \
  --route-strategy always_race \
  --capture-leader-slots \
  --collect-rpcedge

# Strategy: software-client-aware routing. JitoLabs/BAM/FireBAM get
# TPU+Jito bundle, Harmonic* gets TPU+Harmonic bundle, and everything
# else stays TPU-only until a dedicated provider route exists.
cargo run --release -- run-leader-paced \
  --config examples/rpcedge-quic-frankfurt.yaml \
  --duration-seconds 1800 \
  --slot-trigger grpc_slot \
  --route-strategy software_client_aware \
  --capture-leader-slots \
  --collect-rpcedge
```

Use the same transaction shape and observation endpoints for smoke tests, but
only paired/matched-policy artifacts are valid for comparing first-observed
landing time, landed slot delta, processed block `slot_index`, success ratio,
and extra priority/tip cost. Provider ACK and receiver-specific latency are
diagnostic only.
Retries, delayed resends, and duplicate spam are intentionally excluded from
baseline route-policy comparisons; benchmark those as separate tail-mitigation
strategies. To test same-signature resend behavior, keep
`--txs-per-leader-run 1` and add resend attempts:

```bash
cargo run --release -- run-leader-paced \
  --config examples/rpcedge-quic-frankfurt-route-results-5m.yaml \
  --duration-seconds 300 \
  --txs-per-leader-run 1 \
  --leader-run-concurrency 3 \
  --route-strategy paired_route_policies \
  --resend-attempts 3 \
  --resend-interval-ms 50 \
  --slot-trigger grpc_slot \
  --capture-leader-slots \
  --collect-rpcedge
```

This signs one transaction per policy arm, then submits that same signature
three times at `0ms`, `+50ms`, and `+100ms`. Landing remains measured per
unique signature; provider ACK diagnostics are recorded per attempt.

### Paired provider paths

Use `paired_providers` when two equivalent relay processes select different
network paths. It creates one arm per configured provider and sends each signed
transaction only to its assigned provider. The mode requires exactly two
route-aware providers and two `paired_payer_keypair_paths`.

Payer assignment swaps across arms by the four-slot leader-group index. This
prevents concurrent transactions from sharing a writable fee payer and
prevents either route from remaining coupled to one payer identity.

The Frankfurt DoubleZero/public config is
`examples/rpcedge-doublezero-public-frankfurt.yaml`. Run it inside the
Frankfurt transient public-route wrapper:

```bash
solana-tx-bench run-leader-paced \
  --config examples/rpcedge-doublezero-public-frankfurt.yaml \
  --duration-seconds 900 \
  --txs-per-leader-run 1 \
  --leader-run-concurrency 2 \
  --route-strategy paired_providers \
  --leader-network-path-class doublezero \
  --slot-trigger grpc_slot \
  --capture-leader-slots \
  --collect-rpcedge \
  --collect-entries
```

`networkPath.class=doublezero` is eligible only when `verified=true`; Edge
publication fields are not TPU route evidence. Provider config order defines
the comparison: arm 0 is baseline, arm 1 is candidate, and every reported delta
is `candidate - baseline`. Put public Internet first and DoubleZero second so a
negative latency delta means DoubleZero was observed sooner.

The output selects the earliest matched receiver observation for each signature
and measures first-observed, landed-slot, and block-position outcomes. It does
not measure packet-arrival time inside the leader TPU. Provider ACK and
receiver-specific timing remain diagnostics.

Generate the complete DoubleZero-versus-public output after the run:

```bash
scripts/report-doublezero-public.sh artifacts/<test-id>
```

The script runs the standard report and paired comparison, then derives one
additional same-slot placement report from the saved entry history. It writes
`doublezero-public-report.md`, `poh-placement.json`, and `poh-placement.md`
under the comparison directory. This reporting step does not change live
collection or transaction submission.

Leader-paced outputs add:

- `leader-sends.ndjson`
- `leader-runs.ndjson`
- `leader-slots-snapshot.json`, when `--capture-leader-slots` is enabled
- `leader-slots-snapshot-<start_slot>.json`, for rolling gRPC-slot refreshes
- `matched-observations.ndjson`
- `matched-observation-summary.json`
- `matched-observation-summary.md`

Build the self-contained report:

```bash
cargo run --release -- report \
  --artifact-dir artifacts/<test_id>
```

The report command writes `report.json`, `report.md`, and `report.html` using
only local artifacts from the run directory. Leader/client/location cohorts are
computed from saved `getLeaderSlots` snapshots.

Compare two or more runs:

```bash
cargo run --release -- compare \
  --artifact-dir artifacts/<tpu_only_test_id> \
  --label "TPU QUIC only" \
  --artifact-dir artifacts/<client_aware_test_id> \
  --label "Client aware" \
  --output-dir docs/reports/<comparison_slug> \
  --title "RPCEdge TPU QUIC vs Client-Aware Routing"
```

The compare command writes `comparison.json`, `comparison.md`, `comparison.html`,
and `index.html` for GitHub Pages. It uses an RPCEdge landing score across
first-observed landing milliseconds, landed slots, processed block position,
same-slot rate, and success ratio. Provider ACK is retained in raw artifacts for
diagnostics, but it is not landing proof and is not part of the score.

Collect matched observations from RPCEdge Yellowstone processed + SubscribeDeshred:

```bash
cargo run -- collect-rpcedge \
  --test-id my-run \
  --duration-seconds 120 \
  --min-sources 2
```

For end-to-end landing attribution, run either `--collect-rpcedge` during the
benchmark or run a separate RPCEdge collector for the same `test_id`. Sender ACK
artifacts alone do not prove shred or processed observation.

Observation summaries:

- `observation-summary.json`
- `observation-summary.md`

## Safety Defaults

The generated transaction is a memo-free self-transfer. The configured
`lamports` move from the keypair back to the same keypair, and the tool adds the
iteration number to the transfer amount so every transaction has a unique
signature without Memo program compute overhead. It still spends Solana fees and
any priority fee you configure.

Always set `max_spend_lamports`. This is only a local transaction-fee cap; it
is not a replacement for using a throwaway keypair with tiny funds.

## Example Config

See [examples/bench.example.yaml](examples/bench.example.yaml).

## Methodology

See [docs/methodology.md](docs/methodology.md).

## Related Work

See [docs/related-work.md](docs/related-work.md) for how this repo compares to
existing Solana sender and stream benchmark tools.

## Artifact Schema

See [docs/artifacts.md](docs/artifacts.md).

## RPCEdge Enrichment Boundary

See [docs/polaris-enrichment.md](docs/polaris-enrichment.md) for the intended
boundary between portable `getLeaderSlots` enrichment and Polaris-private
gateway/customer diagnostics. Private data should not be required by this public
benchmark repo.

## Report Visualization

The artifacts are plain NDJSON and JSON. The built-in report command writes
portable `report.json`, `report.md`, and `report.html` files directly from the
artifact directory:

```bash
cargo run --release -- report --artifact-dir artifacts/<test_id>
```

Jupyter or Python notebooks can still be useful for custom charts, but they are
not required for the standard public report. Keep provider ACK and receiver-
specific diagnostics out of the transaction-sender result tables.

Published report archive:

- https://rpc-edge.github.io/solana-tx-bench/reports/
