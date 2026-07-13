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
Overall and every cohort breakdown
| Measure | Pairs | Candidate earlier | Same | Baseline earlier | p50 | p95 | p99 |

Landing coverage
| Outcome | Groups |
```

Use the same table contract for first-observation time, processed transaction
index, entry index, and preceding PoH ticks. Do not add receiver-specific timing
rows, absolute per-route observation-time tables, or different-slot pairs to
any performance percentile.

For geography, group by validator identity and saved city/coordinates. Do not
use a coarse region classifier or create a fallback geography bucket when
coordinates are available.

## Interpretation

- Processed gRPC supplies canonical processed transaction metadata from the
  normal Yellowstone path; it is not a finality claim.
- SubscribeDeshred is pre-processed transaction visibility from the deshred path.
- The report selects whichever configured receiver observed each signature
  first. It does not compare receivers with each other.
- Processed metadata supplies canonical transaction index even when another
  receiver supplied the first-observation timestamp.
- ACK latency is not used for the headline result.
- Separate artifact directories are descriptive because they were collected at
  different times. Use one paired artifact for causal route/provider claims.

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
