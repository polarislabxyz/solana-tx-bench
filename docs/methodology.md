# Methodology

This benchmark is designed to compare transaction observation latency across
Yellowstone processed and SubscribeDeshred sources, then enrich those
observations with a saved `getLeaderSlots` snapshot when available.

## Measurement Boundary

Primary public artifact boundary:

```text
signed transaction generated locally
  -> configured provider adapter
  -> observed by RPCEdge Yellowstone processed and SubscribeDeshred
  -> matched by signature
  -> source percentile and win-rate report
```

Portable enrichment boundary:

```text
signature + observed slot from public artifact
  -> getLeaderSlots snapshot captured during the run
  -> leader cohort, region, validator client, datacenter, route hints
```

The public tool should never require private ClickHouse or private validator
metadata. `getLeaderSlots` is the portable enrichment API; the saved snapshot is
part of the benchmark artifact.

## Transaction Shape

Each generated transaction contains:

- compute-unit-limit instruction;
- optional compute-unit-price instruction;
- self-transfer from the funded keypair back to itself;
- iteration-varying transfer amount for deterministic unique signatures.

The default transaction intentionally does not include a Memo instruction. The
benchmark identifies transactions by signature and generated artifacts, so Memo
compute overhead would only make cheap latency probes more expensive. The
self-transfer avoids receiver account management and is net-neutral for
lamports, but the transaction still spends normal Solana fees and any configured
priority fee.

## Recommended Run Ladder

Start with correctness:

```text
count=1, one provider
count=1, each provider individually
count=5, all providers
```

Then run low-rate benchmarks:

```text
duration_seconds=120
rate_per_second=0.2, 0.5, 1.0
```

Only increase rate after:

- provider rejects are understood;
- spend cap is correct;
- artifact persistence is clean;
- observation collectors can join by signature.

## Leader-Paced Runs

For geography and leader-cohort coverage, prefer leader-paced runs over fixed
rate runs:

```bash
cargo run --release -- run-leader-paced \
  --config bench.yaml \
  --duration-seconds 300 \
  --txs-per-leader-run 1 \
  --leader-run-concurrency 1 \
  --slot-trigger grpc_slot \
  --capture-leader-slots \
  --collect-rpcedge
```

The default trigger is `grpc_slot`: the runner subscribes to RPCEdge
Yellowstone slot updates and sends from that stream instead of polling
`getSlot`. It uses captured `getLeaderSlots` snapshots for leader identity,
client, and geography metadata, refreshing those snapshots when the gRPC slot
stream approaches the cached horizon. `rpc_poll` remains available only as a
debug trigger.

The runner groups contiguous slots with the same leader and sends only once per
observed leader run. Each transaction is signed just-in-time with a fresh
blockhash.

`--txs-per-leader-run` controls how many distinct transactions are generated for
that leader run. `--leader-run-concurrency` controls how many of those
transactions are submitted simultaneously. Use concurrency `1` for clean
baselines; set it equal to `txs_per_leader_run` when intentionally measuring a
small burst into the same leader window.

When `--capture-leader-slots` is set, the runner writes
`leader-slots-snapshot-<start_slot>.json` refresh files, and
`leader-slots-snapshot.json` for `rpc_poll` runs. Those JSON-RPC
responses are the portable enrichment source for leader geography, validator
client, route hints, and historical landing-latency priors. Store them with the
benchmark artifacts; a later report should use the saved snapshots rather than
fetching a newer leader schedule.

Suggested ladder:

- 5 minutes, 1 tx per leader run: smoke and artifact validation.
- 30 minutes, 1 tx per leader run: first useful route comparison.
- 2 hours, 1 tx per leader run: enough samples to start inspecting p95/p99
  tails by leader cohorts captured in `getLeaderSlots`.

Run route-isolated configs first. A multi-route race measures the product-level
path, not which route would have independently landed fastest.

## Route Strategies

The first RPCEdge strategy comparison must use paired samples from the same
leader run and same slot trigger. Running one policy for five minutes and a
second policy for the next five minutes is invalid for performance comparison:
the scheduled leaders, slot phase, congestion, block-engine state, and fee
market all changed.

Use `paired_route_policies` for performance comparisons. The runner sends all
policy arms concurrently from the same gRPC slot signal and writes a
`comparison_group_id` into `leader-sends.ndjson` and `samples.ndjson`.
Single-policy invocations of the strategies below are route availability smoke
tests only:

1. `tpu_quic_only`: static route set `only: [tpu_quic]` for every leader.
2. `always_race`: route set `only: [tpu_quic, jito_bundle, harmonic_bundle]`
   for every leader. This is the cost-heavy control that tells us whether the
   extra block-engine paths improve landing latency, landed slot delta, or block
   position enough to justify their fee/tip cost.
3. `software_client_aware`: route set chosen from `client.software` and
   `client.softwareClientId` in the captured `getLeaderSlots` snapshot:
   - `JitoLabs` / software client ID `1`: `only: [tpu_quic, jito_bundle]`
   - `AgaveBam` / software client ID `6`: `only: [tpu_quic, jito_bundle]`
   - `FireBAM` / software client ID `12`: `only: [tpu_quic, jito_bundle]`
   - `HarmonicAgave` / ID `10`, `HarmonicFiredancer` / ID `9`, and
     `HarmonicFrankendancer` / ID `11`: `only: [tpu_quic, harmonic_bundle]`
   - all other or unknown clients: `only: [tpu_quic]`

`client_aware` is the coarse strategy keyed only by normalized
`client.family`. Use `software_client_aware` when the saved leader snapshot has
software-client IDs; use `client_aware` only when that richer metadata is not
available.

Run `software_client_aware` as a smoke test with:

```bash
cargo run --release -- run-leader-paced \
  --config bench.yaml \
  --route-strategy software_client_aware \
  --capture-leader-slots \
  --collect-rpcedge
```

The software-client-aware strategy intentionally requires `--capture-leader-slots`.
If the leader software metadata is missing, the strategy falls back to TPU-only
for that transaction and records `software_client_aware_tpu_only` in the
artifacts.

For a valid policy comparison, the benchmark must write one comparison group per
leader run:

```text
comparison_group_id = same leader run + same slot signal
policy arms = tpu_quic_only, always_race, software_client_aware
```

Each policy arm gets its own signed transaction. The report should compare
policies within `comparison_group_id`, not across adjacent artifact directories.
Use:

```bash
cargo run --release -- run-leader-paced \
  --config bench.yaml \
  --route-strategy paired_route_policies \
  --txs-per-leader-run 1 \
  --leader-run-concurrency 3 \
  --slot-trigger grpc_slot \
  --capture-leader-slots \
  --collect-rpcedge
```

In paired mode, `--txs-per-leader-run 1` means one transaction per policy arm,
not one transaction total. The send loop rotates arm order by leader slot to
avoid a fixed first-arm bias, but all arms in the group are still submitted in
one concurrent batch.

Do not treat validators.app `jito=true` as a route selector. That flag is useful
metadata, but it is broad enough that it can mark most scheduled stake as Jito
related. Route-policy experiments should be based on explicit policies and
observable outcomes, not on assuming `jito=true` means "always use Jito".

Uniform route-policy baselines must use the same transaction shape, compute-unit
limit, and compute-unit price across every policy arm. Cost-aware route-policy
runs may instead pass
`--client-aware-harmonic-cu-price-microlamports 300000`; then any transaction
whose selected route set includes `harmonic_bundle` is signed with at least that
CU price and the extra cost is recorded in the artifacts.

Harmonic public docs describe Harmonic tips as normal Solana compute-unit
priority fees, with no separate tip instruction. Live RPCEdge provider rejects
on 2026-07-02 established the current operational gate:

```text
priority fee >= 10000 lamports OR average CU price >= 300000 micro-lamports/CU
```

For the benchmark's 660-CU transfer shape, `300000` micro-lamports/CU costs
`ceil(660 * 300000 / 1_000_000) = 198` lamports of priority fee, so it is the
minimal Harmonic-eligible setting we use for cost-aware Harmonic arms. This fee
must be present before signing; RPCEdge cannot add it to an already signed
customer transaction without invalidating the signature.

For Jito leaders, `jito_bundle` is an internal RPCEdge route in this benchmark.
The relay adds the Jito bundle tip transaction when the route executes. The
benchmark's public fast ACK does not include the added tip signature. For QUIC
benchmarks, set `route_results: true` on the RPCEdge provider to request bounded
route-result ACKs. When returned, artifacts include `route_tip_signature`,
`route_tip_lamports`, and `route_tip_account`.

Reports separate:

- signed caller transaction fee: base fee plus priority fee;
- route-tip generated spend: relay returned a tip transaction for the Jito
  route; and
- route-tip landed spend: that route-tip signature was observed by the matched
  streams.

If route-tip telemetry is absent, cost ranking is incomplete for any arm that
includes `jito_bundle`. If route-tip telemetry exists but the stream does not
observe the tip signature, the generated cost is visible but landed route-tip
cost remains zero. Include known relay Jito tip accounts with
`run-leader-paced --observation-account <pubkey>` when landed route-tip
attribution matters.

Retries and intentional resends are not part of the route-policy baseline.
The baseline measures one signed transaction per policy arm from the same gRPC
slot signal. Retry-after-X-ms, duplicate fanout, and burst/spam ladders are
valid product experiments for tail mitigation, but they must be labeled as
separate reliability strategies because they change traffic shape, fee spend,
and block-position pressure.

Rakurai and Raiku are intentionally not mapped to Jito or Harmonic by the
software-client-aware strategy as a proved integration. Rakurai publishes its
own transaction-inclusion and virtual-priority path, and Raiku appears as a
separate client ID. For the current benchmark hypothesis, `software_client_aware`
routes Rakurai/Raiku leaders through `tpu_quic + jito_bundle + harmonic_bundle`
to test whether either block-engine route helps those leaders before we build a
dedicated Rakurai adapter.

Rakurai should be modeled as a separate future route, not as Jito-by-another
name. Their public transaction-inclusion docs describe three different
surfaces:

- **Bundle partner mode**: a transaction landing service shares a block-engine
  endpoint list with Rakurai; opted-in Rakurai validators connect to that
  endpoint and then accept bundles that include a Rakurai-recognized tip
  transfer. This is an integration between Rakurai validators and a block
  engine, not the same as calling the Jito bundle route.
- **Virtual priority**: add a normal `SystemProgram.transfer` tip to one of
  Rakurai's tip accounts inside the transaction or bundle. The tip is an extra
  scheduler signal and does not replace regular Solana priority fees.
- **Post-pack confirmations**: Rakurai can stream post-pack transaction packets
  to configured gRPC endpoints; consumers can respond with bundles that include
  the original packet plus additional transactions.

A future `rakurai_virtual_priority` adapter should record the Rakurai tip
account, tip lamports, and transaction signature in artifacts. A future
`rakurai_bundle_partner` adapter should record the returned bundle identifier
and any partner/custom tip account configuration. Until those adapters exist,
benchmark reports should label Rakurai/Raiku `software_client_aware` rows as a
generic TPU+Jito+Harmonic race, not as a Rakurai-native route.

Observation collection is required for landing metrics. Without
`--collect-rpcedge` or a separate collector keyed by the same `test_id`, the
benchmark only knows provider/client ACKs, which are diagnostics rather than
landing evidence.

For RPCEdge QUIC, ACK means the relay accepted and enqueued the request. It is
not proof that a TPU/JET backend later sent the transaction successfully.
Backend callback failures such as leader-side QUIC `disallowed` must be
interpreted from relay route telemetry and the final deshred/processed
observation rate.

## Cohort Analysis

The public artifact includes signatures, observation timestamps, and optionally
a `getLeaderSlots` snapshot. Cohorts such as leader region, stake weight,
validator client, or leader proximity should be computed from:

- observation events by `signature` and `slot`;
- landed slot / slot index observations;
- the saved `leader-slots-snapshot.json` response;

That separation lets external users compare processed-vs-deshred observation
behavior without Polaris-private ClickHouse access. Private RPCEdge reports can
still join deeper internal telemetry such as route-attempt internals or customer
dimensions, but the public report should be reproducible from local artifacts.

Generate the standalone report with:

```bash
solana-tx-bench report --artifact-dir artifacts/<test_id>
```

This command reads only local artifacts from the run directory and writes
`report.json`, `report.md`, `report.html`, `tail-events.json`,
`tail-events.csv`, and `tails.html`.

## Tail Event Methodology

Tail rows are meant to explain bad landings, not only list slow signatures.
They therefore keep both sides of the leader attribution:

- **Send slot / send leader**: the leader visible when the benchmark submitted
  the transaction, from `leader-sends.ndjson` and the saved `getLeaderSlots`
  snapshot.
- **Landed slot / landed leader**: the leader of the slot where the first
  stream observation reported the transaction.

This distinction matters for multi-second tails. A transaction can be sent
during one leader's window and finally land under a later leader; blaming only
the landed leader hides whether the sender route missed the original leader or
whether the transaction was forwarded/held until a later slot.

Latency uses the first stream observation for the signature, normally
SubscribeDeshred, because it is the fastest public signal this benchmark
collects. Transaction block position uses Yellowstone processed gRPC metadata
when available, because processed transaction updates carry the canonical
`slot_index` while deshred observations generally do not. The report can
therefore show `source=rpcedge_deshred` for the timing row and still show a
processed `Tx index` for the same signature.

## Landing-Performance Buckets

Beam-style provider benchmarks use the right broad dimensions for transaction
landing comparisons. This project reports those dimensions as raw measurements
first:

- **Landing latency in milliseconds**: submit timestamp to deshred and
  processed observation by signature.
- **Landing latency in slots**: observed landed slot minus sender-observed slot
  at submission.
- **Transaction position within the block**: processed transaction
  `slot_index`, when the observation source provides it.
- **Same-slot landing rate**: share of transactions with landed-slot delta
  equal to zero.
- **Overall success rate**: observed signatures divided by submitted
  signatures.

For a single-provider or single-route run, a normalized 0-100 score is usually
misleading because there is no comparison set. For route-isolated or
multi-provider runs, the raw buckets above can be normalized against the best
provider in that run. Tail latency should receive more weight than averages
because trading systems care more about bad windows than headline mean latency.

## Cost-Normalized Comparison

For `tpu_quic_only` versus `client_aware`, report both raw landing quality and
cost-normalized deltas:

- deshred submit-to-observed p50/p90/p99;
- processed submit-to-observed p50/p90/p99;
- landed slot delta p50/p90;
- processed `slot_index` p50/p90/p99;
- observed success ratio;
- extra priority fee lamports versus the TPU-only baseline;
- extra route tip generated and route tip landed lamports for Jito bundle
  attempts;
- milliseconds improved per additional lamport;
- block-index positions improved per additional lamport.

The cost floor for the generated transaction is:

```text
base signature fee + ceil(CU limit * CU price microlamports / 1_000_000)
```

Jito route-added bundle tips are not visible in the caller transaction itself.
Use RPCEdge route-result telemetry for generated tip cost, and matched
observations for landed tip cost. Do not collapse those into one estimated
spend number.

## Comparison Scoring

The `compare` command uses the RPCEdge landing score. Provider ACK is retained
in raw artifacts for diagnostics only. Landing truth comes from matched stream
observations by signature. By default, landing latency uses the first observed
stream event for each signature; processed metadata is still used for block
position because deshred observations do not carry a transaction index.

The score has five buckets:

```text
landed_ms = 0.2 * avg_score(submit_to_landed_ms)
          + 0.8 * p90_score(submit_to_landed_ms)

landed_slots = 0.2 * avg_score(landed_slot_delta)
             + 0.8 * p90_score(landed_slot_delta)

landed_idx = 0.5 * avg_score(processed_slot_index)
           + 0.5 * p90_score(processed_slot_index)

same_slot = higher_is_better_score(same_slot_landing_rate)

success_ratio = higher_is_better_score(observed_signatures / submitted_signatures)

performance_rate_pct = (
  landed_ms + landed_slots + landed_idx + same_slot + success_ratio
) / 5
```

Latency, slot delta, and block index are lower-is-better metrics normalized
relative to the best run in the comparison set. Same-slot and success use rates,
not raw counts, so runs with slightly different sample counts remain comparable.

Only use the performance score for runs that share:

- transaction shape;
- sender region;
- slot trigger mode;
- observation sources;
- primary landing source;
- fee and route-tip policy.

If those conditions do not hold, treat the table as descriptive telemetry, not
a provider ranking.

## Matched Observation Reports

The core benchmark uses matched-signature comparisons: send
or observe the same transaction stream from multiple sources, then report which
source saw each signature first.

The first live collector is deliberately narrow:

```text
RPCEdge Yellowstone processed Subscribe
RPCEdge Yellowstone SubscribeDeshred
```

Both are read from the same endpoint and token via `.env`:

```text
RPCEDGE_GRPC_URL=...
YELLOWSTONE_X_TOKEN=...
```

Good matched-observation outputs:

- matched transaction count;
- unmatched count per source;
- win rate by source;
- average delta;
- p75/p90/p95/p99 deltas;
- landed slot and slot index when available.

Observation events include the stable join key: `signature`.

## Interpretation Rules

- Provider ACK latency is not landing latency and is not the benchmark result.
- Accepted by a provider does not mean landed.
- Lowest ACK provider is not always best landed provider.
- The primary benchmark report is matched processed/deshred observation.
- Provider-specific fees, tips, and preflight behavior must be documented per
  adapter.
- Results should state transaction count, rate, priority fee, provider config,
  cluster, and run time.
- Provider comparison claims should state whether they are ACK-only,
  confirmation-based, processed-observation-based, or first-shred/deshred-based.
