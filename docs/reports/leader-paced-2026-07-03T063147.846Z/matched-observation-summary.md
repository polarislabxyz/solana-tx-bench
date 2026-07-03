# Solana Observation Summary

- Test ID: `leader-paced-2026-07-03T063147.846Z`
- Generated: `2026-07-03T07:32:19.482033650+00:00`
- Observations: `5466`
- Matched signatures: `2733`
- Minimum sources: `2`

## Source Summary

| Source | Kind | Observed | Missing | First seen | First seen share | p50 submit->obs us | p90 | p99 | max |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `rpcedge_deshred` | `YellowstoneDeshred` | 2733 | 0 | 2717 | 0.9941 | 112726 | 227853 | 486767 | 3433319 |
| `rpcedge_processed` | `YellowstoneProcessed` | 2733 | 0 | 16 | 0.0059 | 114060 | 233168 | 487430 | 3434452 |

## Pair Delta Summary

Positive delta means `from_source` observed before `to_source`.

| From | To | Matched | From wins | To wins | Same | p50 delta us | p90 | p99 | max |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `rpcedge_deshred` | `rpcedge_processed` | 2733 | 2716 | 16 | 1 | 1064 | 2362 | 10011 | 29065 |
