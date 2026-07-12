# Provider Notes

Provider behavior is not uniform. Keep adapter-specific assumptions here.

## Solana JSON-RPC

Adapter: `solana_rpc`

Sends:

```json
{
  "method": "sendTransaction",
  "params": [
    "<base64 transaction>",
    {
      "encoding": "base64",
      "skipPreflight": true,
      "maxRetries": 0
    }
  ]
}
```

The provider ACK only confirms that the RPC accepted or rejected the request.

## RPCEdge Raw HTTP

Adapter: `rpcedge_raw_http`

Sends raw transaction bytes with `content-type: application/octet-stream`.
Optional API key is read from `api_key_env` and sent as `x-api-key`.

## RPCEdge Route-Aware HTTP

Adapter: `rpcedge_route_aware_http`

Sends a JSON body with transaction base64 and route selection. Route selection
supports:

- `server_default`
- `only`
- `default_plus`
- `default_minus`

The route list is adapter-specific. Example:

```yaml
route_mode: only
routes:
  - tpu_quic
```

## RPCEdge QUIC Raw Transaction

Adapter: `rpcedge_quic_raw_tx`

Uses a persistent QUIC connection to the RPCEdge relay and sends raw
transactions with an RPCEdge route set. Static route selection uses the
configured `route_mode` and `routes`.

In `run-leader-paced --route-strategy always_race`, the runner overrides the
route set per transaction to:

- `tpu_quic + jito_bundle + harmonic_bundle`

This strategy is deliberately expensive. Use it as a control to measure whether
extra provider routes improve landing quality enough to justify the added
priority/tip spend.

In `run-leader-paced --route-strategy software_client_aware`, the runner
overrides the route set per transaction using the scheduled leader's
`client.software` and `client.softwareClientId` from the captured
`getLeaderSlots` snapshot:

- `JitoLabs` / ID `1`: `tpu_quic + jito_bundle`
- `AgaveBam` / ID `6`: `tpu_quic + jito_bundle`
- `FireBAM` / ID `12`: `tpu_quic + jito_bundle`
- `HarmonicAgave` / ID `10`: `tpu_quic + harmonic_bundle`
- `HarmonicFiredancer` / ID `9`: `tpu_quic + harmonic_bundle`
- `HarmonicFrankendancer` / ID `11`: `tpu_quic + harmonic_bundle`
- `Rakurai` / ID `8` and `Raiku` by name/family:
  `tpu_quic + jito_bundle + harmonic_bundle`
- anything else or unknown: `tpu_quic`

The selected route policy and selected route list are written to
`samples.ndjson` and `leader-sends.ndjson`.

The `client_aware` strategy uses only normalized `client.family`, so it is too
coarse for BAM/Rakurai/Raiku semantics. Prefer `software_client_aware` when the
leader snapshot includes software-client IDs.

Single-policy `run-leader-paced` invocations are route smoke tests. They are not
valid A/B comparisons against other policies unless the artifacts come from a
paired multi-policy run where all policies are sent inside the same leader
window and comparison group.

For paired comparison runs, use:

```bash
solana-tx-bench run-leader-paced \
  --route-strategy paired_route_policies \
  --slot-trigger grpc_slot \
  --capture-leader-slots \
  --collect-rpcedge
```

This sends `tpu_quic_only`, `always_race`, and `software_client_aware` arms
concurrently for every leader run. The configured QUIC provider remains the
transport; the per-transaction RPCEdge route set is overridden by the selected
arm.

## Harmonic

Harmonic bundles use ordinary Solana compute-unit priority fees as the economic
signal. There is no Jito-style transfer to a tip account for Harmonic, and the
current public docs describe the tip as the normal Solana CU price.

Live RPCEdge provider rejects on 2026-07-02 showed the current Harmonic server
gate:

```text
priority fee >= 10000 lamports OR avg CU price >= 300000 micro-lamports/CU
```

For the default benchmark transfer shape (`compute_unit_limit: 660`), the cheap
qualifying value is:

```text
ceil(660 * 300000 / 1_000_000) = 198 lamports priority fee
```

Benchmark implication: the benchmarker automatically raises only transactions
whose selected route set includes `harmonic_bundle` to at least `300000`
micro-lamports/CU. The resulting cost must be reported with the landing result.
Use `--client-aware-harmonic-cu-price-microlamports` only to test a higher
Harmonic fee floor.

## Jito Bundle Route

RPCEdge's internal `jito_bundle` route adds a separate Jito tip transaction when
the route executes. The amount is controlled by relay-side configuration and
Jito tip-priority logic, not by the caller transaction's compute-unit price.

Private RPCEdge route-attempt telemetry records the added tip signature,
lamports, and account. Public benchmark artifacts should not assume exact Jito
tip cost unless that route-attempt data or route-detailed ACK data is present.
For QUIC benchmarks, set `route_results: true` on the `rpcedge_quic_raw_tx`
provider to request bounded route-result ACKs. When the relay returns
`route_tip_signature`, `route_tip_lamports`, and `route_tip_account`, reports
show the route-added Jito tip as generated spend. They count it as landed spend
only if the tip signature is observed in the benchmark streams. Since the stream
subscription is account-filtered, the benchmarker includes Jito tip accounts in
the observer filter automatically. If a future adapter adds another tip-based
sender such as Naozomi or Astralane, add that provider's tip/settlement
accounts to the built-in provider observer list. If route-tip fields are absent,
arms that include `jito_bundle` can still be compared on landing quality, but
their full economic cost is unknown.

## Rakurai / Raiku

Rakurai and Raiku leaders are not treated as a proven Jito or Harmonic
integration. Current `software_client_aware` behavior routes them through
`tpu_quic + jito_bundle + harmonic_bundle` as an explicit benchmark hypothesis:
we want to measure whether either existing block-engine route helps before
building a dedicated Rakurai adapter.

Rakurai's public docs describe three relevant integration surfaces:

- **Bundle support for block-engine partners**: a provider exposes a
  block-engine endpoint list, Rakurai validators connect to the lowest-latency
  endpoint, and bundles sent through that relationship must include a tip
  transfer to a Rakurai-recognized tip account. This requires partner setup; it
  is not the current RPCEdge `jito_bundle` route.
- **Virtual priority for transactions or bundles**: include a plain SOL
  transfer to one of Rakurai's tip accounts inside the transaction or bundle.
  The tip is an additional scheduling signal on Rakurai leaders and is not a
  replacement for regular priority fees.
- **Post-pack confirmations**: Rakurai can stream near-final scheduled packets
  to a configured gRPC endpoint using the Jito packet gRPC shape; consumers can
  respond with bundles that include the original packet plus additional
  transactions.

Future adapters should be explicit:

- `rakurai_virtual_priority`: add or require the Rakurai tip transfer and record
  `rakurai_tip_lamports`, `rakurai_tip_account`, and the transaction signature.
- `rakurai_bundle_partner`: submit bundles through a partner block-engine
  integration and record bundle IDs, tip configuration, and provider ACKs.

Do not label Rakurai as Jito in benchmark reports. If a Rakurai/Raiku leader is
slow or fast today, the route list in the run manifest is the source of truth.
`software_client_aware_jito_harmonic` means "generic TPU+Jito+Harmonic race for
a Rakurai/Raiku leader", not a Rakurai-native route.

## Adding External Providers

External provider adapters should be added as isolated config variants. Good
candidate examples:

- Astralane;
- Helius Sender;
- Harmonic.
- bloXroute;
- Nozomi / Temporal-style senders;
- TPU sender proxies such as Yellowstone Jet.

Provider-specific fee floors, minimum priority fees, payload limits, preflight
rules, and bundle rules must be documented with the adapter.

## Adapter Design Notes From Related Tools

Provider adapters should keep protocol-specific behavior isolated:

- HTTP JSON-RPC senders should expose preflight, retry, and encoding settings.
- Raw HTTP senders should record request size and status code.
- QUIC senders should document connection reuse, stream model, and whether the
  provider returns an ACK or is fire-and-forget.
- Bundle senders should document bundle size limits and tip requirements.
- Provider-specific tip builders should be opt-in and visible in config.

When comparing providers concurrently, prefer one funded sender keypair per
provider. Shared fee-payer accounts can serialize or otherwise couple traffic
and make route comparisons less clean.

`paired_providers` enforces that isolation: configure exactly two route-aware
providers and two `paired_payer_keypair_paths`. It binds one signed transaction
to each provider, rotates payer assignment between leader runs, and overrides
both route sets to `only: [tpu_quic]`. Provider order defines baseline arm 0 and
candidate arm 1 in paired reports.
