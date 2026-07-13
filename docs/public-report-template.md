# Public Report Template

Use this template for website-safe benchmark reports.

## Methodology

- Tool: `solana-tx-bench`
- Receivers: `<configured observation receivers>`
- Matching key: transaction signature
- Landing clock: earliest matched receiver observation per signature
- Window: `<start>` to `<end>`
- Region: `<region>`
- Endpoint: `<redacted endpoint label>`

## Results

For a single run, paste the reviewed first-observed landing and block-placement
summary here.

For route/provider comparisons, collapse receiver-specific artifacts to the
earliest observation per signature before calculating paired deltas. Use this
public shape:

```text
Block placement
| Measure | Pairs | Candidate earlier | Same | Baseline earlier | p10 | p25 | p50 | p75 | p90 | p95 |

First-observed timing
| Population | Pairs | Candidate first | Same | Baseline first | p10 | p25 | p50 | p75 | p90 | p95 |

Landing coverage
| Outcome | Groups |
```

Do not add receiver-specific timing rows, absolute per-route observation-time
tables, or different-slot pairs to the timing percentiles.

For geography, group by validator identity and saved city/coordinates. Do not
use a coarse region classifier or create a fallback geography bucket when
coordinates are available.

## Interpretation

- Processed gRPC is final transaction metadata from the normal Yellowstone path.
- SubscribeDeshred is pre-processed transaction visibility from the deshred path.
- The report selects whichever configured receiver observed each signature
  first. It does not compare receivers with each other.
- Processed metadata supplies canonical transaction index even when another
  receiver supplied the first-observation timestamp.
- ACK latency is not used for the headline result.
- Comparison scores use landing latency, landed slots, block position,
  same-slot rate, and success ratio. Only compare runs with the same transaction
  shape, sender region, trigger mode, observation source, and fee/tip policy.

## Links

- RPCEdge website: https://rpcedge.com
- RPCEdge docs: https://docs.rpcedge.com
- Benchmark repo: https://github.com/rpc-edge/solana-tx-bench
- Published report archive: https://rpc-edge.github.io/solana-tx-bench/reports/

## Publishing

Reviewed static reports live under:

```text
docs/reports/<test-id>/index.html
```

GitHub Pages publishes the `docs/` directory, so the public URL shape is:

```text
https://rpc-edge.github.io/solana-tx-bench/reports/<test-id>/
```
