# RPCEdge resend comparison: no resend vs 3x50ms

- Generated: `2026-07-03T04:21:23.567Z`
- Primary landing source: `first_observed`
- Runs: `6`

## Methodology

Landing proof comes from matched observations by transaction signature, not provider ACK. By default, landing latency uses the first observed stream event for each caller transaction; processed metadata is still used for block position because deshred observations do not carry a transaction index. Cost columns separate deterministic transaction fees from route-added tips: base fee and priority fee are encoded in the signed transaction, generated route tips come from route-result ACK metadata, and landed route tips count only tip signatures observed in the matched streams. Lower-is-better metrics are normalized relative to the best run in the comparison set. Landing latency and landed slots weight p90 at 80% and average at 20%. Block position weights average and p90 equally. Same-slot and success use rates so runs with different counts remain comparable. The final performance rate is the unweighted average of the five bucket scores. Only compare runs with the same transaction shape, trigger mode, sender region, and fee/tip policy; otherwise treat the score as descriptive rather than a claim.

> **Warning:** Compared runs use route-dependent priority fees. Treat this as a cost-aware route-strategy comparison, not a fee-neutral transport isolation run.

## Comparison

| Run | Routes | Avg First-Observed ms | P90 First-Observed ms | Avg Landed slots | P90 Landed slots | Avg Idx-in-Block | P90 Idx-in-Block | Avg Base Fee | Avg Priority Fee | Avg Tx Fee | Avg Route Tip Generated | Avg Route Tip Landed | Avg Total Generated | Avg Total Landed | Max Slots | Min Slots | Same-slot landed | Landed runs | Block seen | Total runs | Success ratio % | Performance rate % |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `RPCEdge QUIC Frankfurt 5m no resend / always_race` | `tpu_quic+jito_bundle+harmonic_bundle (181)` | 147.565 | 259.417 | 0.050 | 0.000 | 600.746 | 1093.000 | 5000.000 | 198.000 | 5198.000 | 1000.000 | 171.271 | 6198.000 | 5369.271 | 4 | 0 | 175 | 181 | 181 | 181 | 100.00 | 99.78 |
| `RPCEdge QUIC Frankfurt 5m no resend / software_client_aware` | `tpu_quic (36), tpu_quic+harmonic_bundle (31), tpu_quic+jito_bundle (101), tpu_quic+jito_bundle+harmonic_bundle (13)` | 159.857 | 278.606 | 0.056 | 0.000 | 627.172 | 1123.000 | 5000.000 | 48.133 | 5048.133 | 629.834 | 71.823 | 5677.967 | 5119.956 | 4 | 0 | 174 | 180 | 180 | 181 | 99.45 | 97.05 |
| `RPCEdge QUIC Frankfurt 5m no resend / tpu_quic_only` | `tpu_quic (181)` | 208.776 | 443.027 | 0.137 | 0.000 | 631.954 | 1120.000 | 5000.000 | 0.000 | 5000.000 | 0.000 | 0.000 | 5000.000 | 5000.000 | 3 | 0 | 160 | 175 | 175 | 181 | 96.69 | 86.33 |
| `RPCEdge QUIC Frankfurt 5m resend 3x50ms / always_race` | `tpu_quic+jito_bundle+harmonic_bundle (186)` | 249.981 | 431.413 | 0.226 | 1.000 | 624.952 | 1191.000 | 5000.000 | 198.000 | 5198.000 | 0.000 | 0.000 | 5198.000 | 5198.000 | 4 | 0 | 156 | 186 | 186 | 186 | 100.00 | 76.79 |
| `RPCEdge QUIC Frankfurt 5m resend 3x50ms / software_client_aware` | `tpu_quic (28), tpu_quic+harmonic_bundle (45), tpu_quic+jito_bundle (100), tpu_quic+jito_bundle+harmonic_bundle (13)` | 288.953 | 439.352 | 0.333 | 1.000 | 587.769 | 1209.000 | 5000.000 | 61.742 | 5061.742 | 2177.419 | 220.430 | 7239.161 | 5282.172 | 4 | 0 | 140 | 186 | 186 | 186 | 100.00 | 74.70 |
| `RPCEdge QUIC Frankfurt 5m resend 3x50ms / tpu_quic_only` | `tpu_quic (186)` | 657.964 | 1706.228 | 1.234 | 4.000 | 630.509 | 1197.000 | 5000.000 | 0.000 | 5000.000 | 0.000 | 0.000 | 5000.000 | 5000.000 | 16 | 0 | 85 | 171 | 171 | 186 | 91.94 | 52.98 |

## Score Components

| Run | Landed ms | Landed slots | Block index | Same-slot | Success | Final |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `RPCEdge QUIC Frankfurt 5m no resend / always_race` | 100.00 | 100.00 | 98.92 | 100.00 | 100.00 | 99.78 |
| `RPCEdge QUIC Frankfurt 5m no resend / software_client_aware` | 92.95 | 97.90 | 95.52 | 99.43 | 99.45 | 97.05 |
| `RPCEdge QUIC Frankfurt 5m no resend / tpu_quic_only` | 60.98 | 87.25 | 95.30 | 91.43 | 96.69 | 86.33 |
| `RPCEdge QUIC Frankfurt 5m resend 3x50ms / always_race` | 59.91 | 44.40 | 92.91 | 86.75 | 100.00 | 76.79 |
| `RPCEdge QUIC Frankfurt 5m resend 3x50ms / software_client_aware` | 57.45 | 42.98 | 95.20 | 77.85 | 100.00 | 74.70 |
| `RPCEdge QUIC Frankfurt 5m resend 3x50ms / tpu_quic_only` | 16.65 | 16.81 | 92.27 | 47.27 | 91.94 | 52.98 |
