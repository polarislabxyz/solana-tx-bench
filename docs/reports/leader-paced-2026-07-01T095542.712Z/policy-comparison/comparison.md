# RPCEdge Paired Route Policy Comparison

- Generated: `2026-07-02T03:58:05.036Z`
- Primary landing source: `first_observed`
- Runs: `3`

## Methodology

Landing proof comes from matched observations by transaction signature, not provider ACK. By default, landing latency uses the first observed stream event for each signature; processed metadata is still used for block position because deshred observations do not carry a transaction index. Lower-is-better metrics are normalized relative to the best run in the comparison set. Landing latency and landed slots weight p90 at 80% and average at 20%. Block position weights average and p90 equally. Same-slot and success use rates so runs with different counts remain comparable. The final performance rate is the unweighted average of the five bucket scores. Only compare runs with the same transaction shape, trigger mode, sender region, and fee/tip policy; otherwise treat the score as descriptive rather than a claim.

> **Warning:** Compared runs use different average priority fees. Treat this as exploratory; clean route-policy baselines must use the same compute-unit limit and compute-unit price in every arm.

> **Warning:** At least one compared run includes jito_bundle. Full economic cost is incomplete unless route-tip telemetry is joined; the visible priority-fee column does not include relay-added Jito tips.

## Comparison

| Run | Routes | Avg First-Observed ms | P90 First-Observed ms | Avg Landed slots | P90 Landed slots | Avg Idx-in-Block | P90 Idx-in-Block | Avg Priority Fee | Max Slots | Min Slots | Same-slot landed | Landed runs | Block seen | Total runs | Success ratio % | Performance rate % |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `always_race` | `tpu_quic+jito_bundle+harmonic_bundle (188)` | 147.611 | 240.309 | 0.069 | 0.000 | 524.856 | 1022.000 | 198.000 | 9 | 0 | 183 | 188 | 188 | 188 | 100.00 | 100.00 |
| `software_client_aware` | `tpu_quic (36), tpu_quic+harmonic_bundle (45), tpu_quic+jito_bundle (107)` | 163.007 | 252.712 | 0.075 | 0.000 | 548.128 | 1041.000 | 47.394 | 3 | 0 | 180 | 187 | 187 | 188 | 99.47 | 97.49 |
| `tpu_quic_only` | `tpu_quic (188)` | 281.246 | 252.722 | 0.343 | 0.000 | 567.044 | 1053.000 | 0.000 | 48 | 0 | 172 | 181 | 181 | 188 | 96.28 | 91.14 |

## Score Components

| Run | Landed ms | Landed slots | Block index | Same-slot | Success | Final |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `always_race` | 100.00 | 100.00 | 100.00 | 100.00 | 100.00 | 100.00 |
| `software_client_aware` | 94.18 | 98.47 | 96.96 | 98.36 | 99.47 | 97.49 |
| `tpu_quic_only` | 86.57 | 84.04 | 94.81 | 93.99 | 96.28 | 91.14 |
