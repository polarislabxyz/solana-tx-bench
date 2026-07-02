# Transaction Landing Report: `leader-paced-2026-07-02T081336.539Z`

- Generated: `2026-07-02T08:19:28.458Z`
- Artifact dir: `/home/sol/solana-tx-bench/artifacts/leader-paced-2026-07-02T081336.539Z`
- Sent transactions: `543`
- Provider samples: `543`
- Observations: `1160`
- Matched caller signatures: `536`
- Matched route-tip signatures: `44`
- Leader-slot snapshots: `9`

This report is generated only from local benchmark artifacts. Leader metadata comes from saved `getLeaderSlots` snapshots captured during the run; no ClickHouse or private database join is required.

> **Warning:** This artifact used mixed compute-unit prices. Treat it as exploratory; it is not a clean route-policy baseline. Current baselines must use the same compute-unit limit and compute-unit price for every policy arm.

## Cost Summary

Base fee and priority fee are deterministic fields from the signed benchmark transaction. Route-tip generated spend comes from route-result ACK metadata. Route-tip landed spend is counted only when the route-tip signature was observed in the matched streams.

| Submitted txs | Avg base fee | Avg priority fee | Avg tx fee | Avg route tip generated | Avg route tip landed | Avg total generated | Avg total landed | Generated tip txs | Landed tip txs |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 543 | 5000.000 | 82.044 | 5082.044 | 543.278 | 84.715 | 5625.322 | 5166.759 | 295 | 46 |

## Landing Observations

Landing latency uses the earliest observation per transaction across configured streams. In normal RPCEdge runs this should be the deshred stream; processed metadata is only counted here when it is the first or only observation for a signature.

| First source | Count | p50 ms | p90 ms | p99 ms | max ms | same-slot | slot delta p50 | slot index p50 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `rpcedge_deshred` | 536 | 119.805 | 278.606 | 1649.936 | 3246.342 | 509 / 95.0% | 0 | - |

## Cohort: `leader_region`

| Value | Source | Count | p50 ms | p90 ms | p99 ms | max ms |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| `Europe` | `rpcedge_deshred` | 332 | 90.516 | 258.238 | 1698.471 | 3246.342 |
| `AsiaPacific` | `rpcedge_deshred` | 57 | 238.975 | 467.943 | 756.022 | 756.022 |
| `UsEast` | `rpcedge_deshred` | 54 | 146.722 | 261.187 | 1657.687 | 1657.687 |
| `Unknown` | `rpcedge_deshred` | 51 | 112.409 | 496.976 | 603.986 | 603.986 |
| `UsCentral` | `rpcedge_deshred` | 27 | 145.828 | 256.806 | 422.027 | 422.027 |
| `UsWest` | `rpcedge_deshred` | 15 | 166.892 | 192.517 | 192.523 | 192.523 |

## Cohort: `leader_city`

| Value | Source | Count | p50 ms | p90 ms | p99 ms | max ms |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| `Frankfurt am Main` | `rpcedge_deshred` | 95 | 102.822 | 258.238 | 3053.767 | 3053.767 |
| `Amsterdam` | `rpcedge_deshred` | 93 | 99.726 | 259.433 | 3246.342 | 3246.342 |
| `City of London` | `rpcedge_deshred` | 54 | 76.925 | 117.876 | 124.353 | 124.353 |
| `Tokyo` | `rpcedge_deshred` | 28 | 278.606 | 503.606 | 756.022 | 756.022 |
| `Vilnius` | `rpcedge_deshred` | 27 | 65.878 | 113.835 | 113.882 | 113.882 |
| `London` | `rpcedge_deshred` | 24 | 436.345 | 455.778 | 461.378 | 461.378 |
| `Asia/Singapore` | `rpcedge_deshred` | 23 | 199.249 | 467.943 | 469.248 | 469.248 |
| `Chicago` | `rpcedge_deshred` | 21 | 145.828 | 254.493 | 254.511 | 254.511 |
| `Ashburn` | `rpcedge_deshred` | 18 | 135.745 | 186.006 | 186.015 | 186.015 |
| `Beauharnois` | `rpcedge_deshred` | 18 | 134.360 | 538.451 | 538.457 | 538.457 |
| `Stockholm` | `rpcedge_deshred` | 18 | 38.860 | 228.021 | 228.026 | 228.026 |
| `Boardman` | `rpcedge_deshred` | 12 | 166.879 | 169.178 | 169.185 | 169.185 |
| `Rödelheim` | `rpcedge_deshred` | 9 | 32.262 | 66.834 | 66.834 | 66.834 |
| `America/Chicago` | `rpcedge_deshred` | 6 | 256.806 | 422.027 | 422.027 | 422.027 |
| `Dublin` | `rpcedge_deshred` | 6 | 106.021 | 106.033 | 106.033 | 106.033 |
| `Frankfurt` | `rpcedge_deshred` | 6 | 25.254 | 30.203 | 30.203 | 30.203 |
| `Luxembourg` | `rpcedge_deshred` | 6 | 184.105 | 221.196 | 221.196 | 221.196 |
| `New York` | `rpcedge_deshred` | 6 | 136.403 | 139.998 | 139.998 | 139.998 |
| `Newark` | `rpcedge_deshred` | 6 | 175.880 | 1657.687 | 1657.687 | 1657.687 |
| `Ogden` | `rpcedge_deshred` | 6 | 205.898 | 205.909 | 205.909 | 205.909 |
| `Sterling` | `rpcedge_deshred` | 6 | 224.947 | 224.959 | 224.959 | 224.959 |
| `America/Toronto` | `rpcedge_deshred` | 3 | 364.256 | 579.565 | 579.565 | 579.565 |
| `Asia/Dubai` | `rpcedge_deshred` | 3 | 162.131 | 162.137 | 162.137 | 162.137 |
| `Aubervilliers` | `rpcedge_deshred` | 3 | 36.348 | 36.354 | 36.354 | 36.354 |

## Cohort: `data_center_key`

| Value | Source | Count | p50 ms | p90 ms | p99 ms | max ms |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| `20326-DE-Frankfurt am Main` | `rpcedge_deshred` | 53 | 108.627 | 261.547 | 1732.301 | 1732.301 |
| `20326-NL-Amsterdam` | `rpcedge_deshred` | 45 | 99.721 | 479.542 | 3246.342 | 3246.342 |
| `399460-GB-City of London` | `rpcedge_deshred` | 27 | 45.279 | 62.962 | 62.978 | 62.978 |
| `400963-GB-Europe/London` | `rpcedge_deshred` | 21 | 436.353 | 455.778 | 461.378 | 461.378 |
| `20326-JP-Tokyo` | `rpcedge_deshred` | 19 | 249.847 | 503.635 | 756.022 | 756.022 |
| `16125-LT-Vilnius` | `rpcedge_deshred` | 18 | 63.937 | 113.871 | 113.882 | 113.882 |
| `16276-CA-Beauharnois` | `rpcedge_deshred` | 18 | 134.360 | 538.451 | 538.457 | 538.457 |
| `20326-GB-City of London` | `rpcedge_deshred` | 18 | 107.871 | 124.344 | 124.353 | 124.353 |
| `214159-SE-Stockholm` | `rpcedge_deshred` | 18 | 38.860 | 228.021 | 228.026 | 228.026 |
| `14618-US-Ashburn` | `rpcedge_deshred` | 15 | 132.201 | 146.727 | 146.732 | 146.732 |
| `204770-US-Chicago` | `rpcedge_deshred` | 15 | 170.695 | 254.506 | 254.511 | 254.511 |
| `396356-DE-Frankfurt am Main` | `rpcedge_deshred` | 15 | 91.731 | 124.923 | 124.933 | 124.933 |
| `396356-SG-Asia/Singapore` | `rpcedge_deshred` | 15 | 165.010 | 225.090 | 469.248 | 469.248 |
| `7979-NL-Europe/Amsterdam` | `rpcedge_deshred` | 15 | 234.701 | 264.400 | 264.406 | 264.406 |
| `9002-DE-Frankfurt am Main` | `rpcedge_deshred` | 15 | 108.729 | 151.507 | 151.513 | 151.513 |
| `16509-US-Boardman` | `rpcedge_deshred` | 12 | 166.879 | 169.178 | 169.185 | 169.185 |
| `16125-LT-Europe/Vilnius` | `rpcedge_deshred` | 9 | 66.159 | 72.852 | 72.852 | 72.852 |
| `20326-NL-Europe/Amsterdam` | `rpcedge_deshred` | 9 | 67.699 | 140.507 | 140.507 | 140.507 |
| `262287-NL-Amsterdam` | `rpcedge_deshred` | 9 | 105.443 | 123.064 | 123.064 | 123.064 |
| `395201-DE-Rödelheim` | `rpcedge_deshred` | 9 | 32.262 | 66.834 | 66.834 | 66.834 |
| `396356-GB-City of London` | `rpcedge_deshred` | 9 | 88.652 | 94.715 | 94.715 | 94.715 |
| `45102-SG-Asia/Singapore` | `rpcedge_deshred` | 8 | 319.476 | 467.958 | 467.958 | 467.958 |
| `16276-DE-Frankfurt` | `rpcedge_deshred` | 6 | 25.254 | 30.203 | 30.203 | 30.203 |
| `18450-US-Ogden` | `rpcedge_deshred` | 6 | 205.898 | 205.909 | 205.909 | 205.909 |

## Cohort: `client_family`

| Value | Source | Count | p50 ms | p90 ms | p99 ms | max ms |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| `bam` | `rpcedge_deshred` | 185 | 113.882 | 265.795 | 503.635 | 756.022 |
| `jito` | `rpcedge_deshred` | 111 | 76.096 | 361.270 | 538.457 | 603.986 |
| `harmonic` | `rpcedge_deshred` | 91 | 114.698 | 457.535 | 3246.342 | 3246.342 |
| `frankendancer` | `rpcedge_deshred` | 68 | 174.034 | 451.226 | 1732.301 | 1732.301 |
| `rakurai` | `rpcedge_deshred` | 39 | 91.731 | 134.375 | 579.565 | 579.565 |
| `agave` | `rpcedge_deshred` | 27 | 162.543 | 228.005 | 228.026 | 228.026 |
| `firedancer` | `rpcedge_deshred` | 15 | 117.961 | 123.382 | 123.388 | 123.388 |

## Cohort: `client_software`

| Value | Source | Count | p50 ms | p90 ms | p99 ms | max ms |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| `AgaveBam` | `rpcedge_deshred` | 185 | 113.882 | 265.795 | 503.635 | 756.022 |
| `JitoLabs` | `rpcedge_deshred` | 111 | 76.096 | 361.270 | 538.457 | 603.986 |
| `HarmonicAgave` | `rpcedge_deshred` | 79 | 108.639 | 266.214 | 601.372 | 601.372 |
| `Frankendancer` | `rpcedge_deshred` | 68 | 174.034 | 451.226 | 1732.301 | 1732.301 |
| `Rakurai` | `rpcedge_deshred` | 39 | 91.731 | 134.375 | 579.565 | 579.565 |
| `Agave` | `rpcedge_deshred` | 27 | 162.543 | 228.005 | 228.026 | 228.026 |
| `Firedancer` | `rpcedge_deshred` | 15 | 117.961 | 123.382 | 123.388 | 123.388 |
| `HarmonicFrankendancer` | `rpcedge_deshred` | 12 | 243.355 | 3053.767 | 3246.342 | 3246.342 |

## Cohort: `stake_bucket`

| Value | Source | Count | p50 ms | p90 ms | p99 ms | max ms |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| `>=1m_sol` | `rpcedge_deshred` | 362 | 113.871 | 266.225 | 1649.936 | 1732.301 |
| `100k_500k_sol` | `rpcedge_deshred` | 105 | 165.000 | 361.288 | 538.457 | 3246.342 |
| `500k_1m_sol` | `rpcedge_deshred` | 48 | 109.782 | 180.279 | 3053.767 | 3053.767 |
| `<100k_sol` | `rpcedge_deshred` | 21 | 137.103 | 225.063 | 469.248 | 469.248 |

## Tail Events

Tail latency uses the first observation time, normally deshred. Transaction slot index uses processed Yellowstone/gRPC metadata when available, because deshred is fastest but processed carries canonical transaction metadata.

| Signature | Source | Latency ms | Arm | Routes | Send slot | Send leader | Send client | Landed slot | Landed leader | Landed client | Tx index | Tx fee | Route tip generated | Route tip landed | Total landed |
| --- | --- | ---: | --- | --- | ---: | --- | --- | ---: | --- | --- | ---: | ---: | ---: | ---: | ---: |
| `5FNHSdwA...syDCQY4y` | `rpcedge_deshred` | 3246.342 | `tpu_quic_only` | `tpu_quic` | 430269324 | `BiGcsiuFCLuiTzXoQgfLdge9sfpwr55YzdT8Kp7bCXmS (Amsterdam)` | `HarmonicFrankendancer` | 430269327 | `BiGcsiuFCLuiTzXoQgfLdge9sfpwr55YzdT8Kp7bCXmS (Amsterdam)` | `HarmonicFrankendancer` | 1408 | 5000 | 0 | 0 | 5000 |
| `3JawZMcJ...jyjxczkS` | `rpcedge_deshred` | 3053.767 | `tpu_quic_only` | `tpu_quic` | 430269300 | `BUokhb8pPF9MZuzW3rHLr6jzakgcz3NDq2PZkpiVv3jb (Frankfurt am Main)` | `HarmonicFrankendancer` | 430269303 | `BUokhb8pPF9MZuzW3rHLr6jzakgcz3NDq2PZkpiVv3jb (Frankfurt am Main)` | `HarmonicFrankendancer` | 2156 | 5000 | 0 | 0 | 5000 |
| `Arbrrbx9...UqCrFq8N` | `rpcedge_deshred` | 1732.301 | `software_client_aware` | `tpu_quic+jito_bundle` | 430269304 | `8T8AJfUCXwPFwEMmjca8gCRSktPrqbUBVa6ggNyhLhFJ (Asia/Hong_Kong)` | `AgaveBam` | 430269308 | `EvnRmnMrd69kFdbLMxWkTn1icZ7DCceRhvmb2SJXqDo4 (Frankfurt am Main)` | `Frankendancer` | 475 | 5000 | 1000 | 1000 | 6000 |
| `5hPYY38s...AYGVN26Y` | `rpcedge_deshred` | 1698.471 | `always_race` | `tpu_quic+jito_bundle+harmonic_bundle` | 430269304 | `8T8AJfUCXwPFwEMmjca8gCRSktPrqbUBVa6ggNyhLhFJ (Asia/Hong_Kong)` | `AgaveBam` | 430269308 | `EvnRmnMrd69kFdbLMxWkTn1icZ7DCceRhvmb2SJXqDo4 (Frankfurt am Main)` | `Frankendancer` | 424 | 5198 | 1000 | 1000 | 6198 |
| `5sLXeyiu...ruJrLdbP` | `rpcedge_deshred` | 1657.687 | `tpu_quic_only` | `tpu_quic` | 430269420 | `J6etcxDdYjPHrtyvDXrbCkx3q9W1UjMj1vy1jBFPJEbK (Newark)` | `HarmonicFrankendancer` | 430269423 | `J6etcxDdYjPHrtyvDXrbCkx3q9W1UjMj1vy1jBFPJEbK (Newark)` | `HarmonicFrankendancer` | 1477 | 5000 | 0 | 0 | 5000 |
| `4LZNJgNA...SCjJGHbd` | `rpcedge_deshred` | 1649.936 | `tpu_quic_only` | `tpu_quic` | 430269036 | `pSoLoZx55zZz61gjxSTwHtwTg4yTwdm7ruBmyjbYgT2 (Amsterdam)` | `HarmonicFrankendancer` | 430269039 | `pSoLoZx55zZz61gjxSTwHtwTg4yTwdm7ruBmyjbYgT2 (Amsterdam)` | `HarmonicFrankendancer` | 1499 | 5000 | 0 | 0 | 5000 |
| `4okRmedH...3xFZ61D9` | `rpcedge_deshred` | 927.217 | `software_client_aware` | `tpu_quic+harmonic_bundle` | 430269036 | `pSoLoZx55zZz61gjxSTwHtwTg4yTwdm7ruBmyjbYgT2 (Amsterdam)` | `HarmonicFrankendancer` | 430269038 | `pSoLoZx55zZz61gjxSTwHtwTg4yTwdm7ruBmyjbYgT2 (Amsterdam)` | `HarmonicFrankendancer` | 148 | 5198 | 0 | 0 | 5198 |
| `4rGaQtxh...dKhEJKoY` | `rpcedge_deshred` | 756.022 | `tpu_quic_only` | `tpu_quic` | 430268988 | `5EhGYUyQNrxgUbuYF4vbL2SZDT6RMfhq3yjeyevvULeC (Tokyo)` | `AgaveBam` | 430268990 | `5EhGYUyQNrxgUbuYF4vbL2SZDT6RMfhq3yjeyevvULeC (Tokyo)` | `AgaveBam` | 23 | 5000 | 0 | 0 | 5000 |
| `4A41qaub...PcQ33ECe` | `rpcedge_deshred` | 603.986 | `tpu_quic_only` | `tpu_quic` | 430269004 | `Ninja1spj6n9t5hVYgF3PdnYz2PLnkt7rvaw3firmjs (Vancouver)` | `JitoLabs` | 430269005 | `Ninja1spj6n9t5hVYgF3PdnYz2PLnkt7rvaw3firmjs (Vancouver)` | `JitoLabs` | 748 | 5000 | 0 | 0 | 5000 |
| `s1EMnoZ1...wkfccjgc` | `rpcedge_deshred` | 601.372 | `tpu_quic_only` | `tpu_quic` | 430269224 | `GoeW4aFK4dGoekJySgUynWDxBZiQJqm8GDAF4H53tDK9 (Frankfurt am Main)` | `HarmonicAgave` | 430269225 | `GoeW4aFK4dGoekJySgUynWDxBZiQJqm8GDAF4H53tDK9 (Frankfurt am Main)` | `HarmonicAgave` | 98 | 5000 | 0 | 0 | 5000 |
| `3XtpTpkx...NcXxaN37` | `rpcedge_deshred` | 584.280 | `tpu_quic_only` | `tpu_quic` | 430268960 | `GoeW4aFK4dGoekJySgUynWDxBZiQJqm8GDAF4H53tDK9 (Frankfurt am Main)` | `HarmonicAgave` | 430268961 | `GoeW4aFK4dGoekJySgUynWDxBZiQJqm8GDAF4H53tDK9 (Frankfurt am Main)` | `HarmonicAgave` | 77 | 5000 | 0 | 0 | 5000 |
| `61x8WQL2...fi7qY9EP` | `rpcedge_deshred` | 579.565 | `tpu_quic_only` | `tpu_quic` | 430269480 | `8tjFeSApQ85ThoQXT28acfF2KUfQr3TvTdirSkzNnYC7 (America/Toronto)` | `Rakurai` | 430269481 | `8tjFeSApQ85ThoQXT28acfF2KUfQr3TvTdirSkzNnYC7 (America/Toronto)` | `Rakurai` | 686 | 5000 | 0 | 0 | 5000 |

## Appendix: Sender ACK Diagnostics

ACK confirms that the relay accepted or rejected the request. It is useful for route availability and gateway health debugging, but it is not landing proof and is not used as a performance target.

| Provider | Policy | Routes | Count | Accepted | Errors | p50 ms | p90 ms | p99 ms | max ms |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `rpcedge-quic-frankfurt` | `always_race` | `tpu_quic,jito_bundle,harmonic_bundle` | 181 | 181 | 0 | 1.629 | 2.322 | 5.486 | 10.254 |
| `rpcedge-quic-frankfurt` | `software_client_aware_harmonic` | `tpu_quic,harmonic_bundle` | 31 | 31 | 0 | 1.191 | 1.762 | 2.088 | 2.088 |
| `rpcedge-quic-frankfurt` | `software_client_aware_jito` | `tpu_quic,jito_bundle` | 101 | 101 | 0 | 1.842 | 2.557 | 6.405 | 16.857 |
| `rpcedge-quic-frankfurt` | `software_client_aware_jito_harmonic` | `tpu_quic,jito_bundle,harmonic_bundle` | 13 | 13 | 0 | 1.658 | 2.346 | 2.352 | 2.352 |
| `rpcedge-quic-frankfurt` | `software_client_aware_tpu_only` | `tpu_quic` | 36 | 36 | 0 | 0.671 | 1.431 | 2.108 | 2.108 |
| `rpcedge-quic-frankfurt` | `tpu_quic_only` | `tpu_quic` | 181 | 181 | 0 | 1.294 | 2.215 | 2.699 | 2.929 |
