# Transaction Landing Report: `leader-paced-2026-07-03T041112.334Z`

- Generated: `2026-07-03T04:21:23.529Z`
- Artifact dir: `artifacts/leader-paced-2026-07-03T041112.334Z`
- Sent transactions: `558`
- Provider samples: `1674`
- Observations: `1130`
- Matched caller signatures: `543`
- Unobserved caller signatures: `15`
- Matched route-tip signatures: `22`
- Leader-slot snapshots: `10`

This report is generated only from local benchmark artifacts. Leader metadata comes from saved `getLeaderSlots` snapshots captured during the run; no ClickHouse or private database join is required.

> **Warning:** This artifact used route-dependent priority fees. Treat it as a cost-aware route-strategy comparison, not a fee-neutral transport isolation run.

## Cost Summary

Base fee and priority fee are deterministic fields from the signed benchmark transaction. Route-tip generated spend comes from route-result ACK metadata. Route-tip landed spend is counted only when the route-tip signature was observed in the matched streams.

| Submitted txs | Avg base fee | Avg priority fee | Avg tx fee | Avg route tip generated | Avg route tip landed | Avg total generated | Avg total landed | Generated tip txs | Landed tip txs |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 558 | 5000.000 | 86.581 | 5086.581 | 725.806 | 73.477 | 5812.387 | 5160.057 | 405 | 41 |

## Landing Observations

Landing latency uses the earliest observation per transaction across configured streams. In normal RPCEdge runs this should be the deshred stream; processed metadata is only counted here when it is the first or only observation for a signature.

| First source | Count | p50 ms | p90 ms | p99 ms | max ms | same-slot | slot delta p50 | slot index p50 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `rpcedge_deshred` | 538 | 262.739 | 585.660 | 2066.279 | 6367.252 | 377 / 70.1% | 0 | - |
| `rpcedge_processed` | 5 | 216.102 | 453.157 | 453.157 | 453.157 | 4 / 80.0% | 0 | 661 |

## Unobserved Signatures

These signatures were submitted by the benchmark but were not observed by the configured landing streams during the collection window. Treat them as the first investigation bucket for dropped transactions, late-beyond-tail transactions, or observation-path misses.

| Signature | Arm | Routes | Send slot | Send leader | Send client | Provider ACK | Route ACKs | Total generated spend |
| --- | --- | --- | ---: | --- | --- | --- | --- | ---: |
| `3EErVxvo...8Budqsk5` | `tpu_quic_only` | `tpu_quic` | 430446892 | `A1vqhA2fS6K7CvHsJKX1ACcHJFEmyRg4KuR5pctHANy4 (Amsterdam)` | `HarmonicAgave` | 3/3 accepted (0.368-251.173ms) | `tpu_quic:ok:9us` | 5000 |
| `3TYauNxe...bfVt7CiM` | `tpu_quic_only` | `tpu_quic` | 430446992 | `C1ocKDYMCm2ooWptMMnpd5VEB2Nx4UMJgRuYofysyzcA (New York)` | `HarmonicAgave` | 3/3 accepted (250.569-251.899ms) | `-` | 5000 |
| `4JaxgEpP...WVUt6SYk` | `tpu_quic_only` | `tpu_quic` | 430447028 | `BtsmiEEvnSuUnKxqXj2PZRYpPJAc7C34mGz8gtJ1DAaH (Frankfurt am Main)` | `HarmonicAgave` | 3/3 accepted (250.749-251.144ms) | `-` | 5000 |
| `35GgAPCM...QAjgG6W2` | `tpu_quic_only` | `tpu_quic` | 430447032 | `GoeW4aFK4dGoekJySgUynWDxBZiQJqm8GDAF4H53tDK9 (Frankfurt am Main)` | `HarmonicAgave` | 3/3 accepted (131.082-251.552ms) | `tpu_quic:ok:11us | tpu_quic:ok:9us` | 5000 |
| `51mXskhq...SsqKS7d8` | `tpu_quic_only` | `tpu_quic` | 430447076 | `9UM8wQ8F5oMiRcP5YdqD6Lr4krpBWCD8LtgQYoisJd9i (City of London)` | `HarmonicAgave` | 3/3 accepted (251.024-251.472ms) | `-` | 5000 |
| `2iCUwHp2...WzvYFwo1` | `tpu_quic_only` | `tpu_quic` | 430447148 | `q9XWcZ7T1wP4bW9SB4XgNNwjnFEJ982nE8aVbbNuwot (Frankfurt am Main)` | `Rakurai` | 3/3 accepted (136.964-251.686ms) | `tpu_quic:ok:1us` | 5000 |
| `2J9ymMQB...nSENCNrE` | `tpu_quic_only` | `tpu_quic` | 430447160 | `Hz5aLvpKScNWoe9YZWxBLrQA3qzHJivBGtfciMekk8m5 (City of London)` | `HarmonicAgave` | 3/3 accepted (0.430-251.597ms) | `tpu_quic:ok:11us` | 5000 |
| `3XKfAk8g...uA7XGa69` | `tpu_quic_only` | `tpu_quic` | 430447200 | `CW9C7HBwAMgqNdXkNgFg9Ujr3edR2Ab9ymEuQnVacd1A (City of London)` | `HarmonicAgave` | 3/3 accepted (219.593-251.347ms) | `tpu_quic:ok:7us` | 5000 |
| `2eU7ox48...wM9zwNPJ` | `tpu_quic_only` | `tpu_quic` | 430447212 | `JUPiTERrZqgf1jUyR7dSkhMx4Kn2qJyekWsg3LT1h4b (Frankfurt am Main)` | `HarmonicAgave` | 3/3 accepted (250.602-251.460ms) | `-` | 5000 |
| `3YP6tzMv...qdgbxYt6` | `tpu_quic_only` | `tpu_quic` | 430447216 | `ANC1u9sY36q3mi2MyVhtz71un8yLgTsFBUuyLcSPzKsk (Tsuen Wan)` | `JitoLabs` | 3/3 accepted (0.294-251.717ms) | `tpu_quic:ok:8us | tpu_quic:ok:7us` | 5000 |
| `2YEbVJX7...LDhp6Hsu` | `tpu_quic_only` | `tpu_quic` | 430447220 | `RNXnAJV1DeBt6Lytjz4wYzvS3d6bhsfidS5Np4ovwZz (Prague)` | `Frankendancer` | 3/3 accepted (0.575-219.884ms) | `tpu_quic:ok:12us | tpu_quic:ok:10us | tpu_quic:ok:13us` | 5000 |
| `5hJtzCAa...5oGy57dG` | `tpu_quic_only` | `tpu_quic` | 430447264 | `G1bLKfyNm7zsmmYEL9dyxBvMtxpFcwy2s84bHDj2ZFUY (São Paulo)` | `Rakurai` | 3/3 accepted (0.396-251.610ms) | `tpu_quic:ok:9us` | 5000 |
| `4NdKqUPv...GBiTDH3q` | `tpu_quic_only` | `tpu_quic` | 430447268 | `q9XWcZ7T1wP4bW9SB4XgNNwjnFEJ982nE8aVbbNuwot (Frankfurt am Main)` | `Rakurai` | 3/3 accepted (250.947-251.757ms) | `-` | 5000 |
| `41F1h4Xf...9SUtu9oH` | `tpu_quic_only` | `tpu_quic` | 430447332 | `BtsmiEEvnSuUnKxqXj2PZRYpPJAc7C34mGz8gtJ1DAaH (Frankfurt am Main)` | `HarmonicAgave` | 3/3 accepted (0.378-250.750ms) | `tpu_quic:ok:8us | tpu_quic:ok:6us` | 5000 |
| `24J7fzHi...7hoeznD3` | `tpu_quic_only` | `tpu_quic` | 430447620 | `CW9C7HBwAMgqNdXkNgFg9Ujr3edR2Ab9ymEuQnVacd1A (City of London)` | `HarmonicAgave` | 3/3 accepted (130.179-252.268ms) | `tpu_quic:ok:8us | tpu_quic:ok:9us` | 5000 |

## Cohort: `leader_region`

| Value | Source | Count | p50 ms | p90 ms | p99 ms | max ms |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| `Europe` | `rpcedge_deshred` | 328 | 255.580 | 492.758 | 2066.279 | 3249.993 |
| `UsCentral` | `rpcedge_deshred` | 52 | 284.927 | 1523.956 | 6367.252 | 6367.252 |
| `UsEast` | `rpcedge_deshred` | 49 | 311.708 | 566.226 | 608.669 | 608.669 |
| `Unknown` | `rpcedge_deshred` | 47 | 295.137 | 1545.408 | 1916.359 | 1916.359 |
| `AsiaPacific` | `rpcedge_deshred` | 42 | 423.070 | 1591.615 | 1795.339 | 1795.339 |
| `UsWest` | `rpcedge_deshred` | 20 | 375.908 | 1741.771 | 1814.347 | 1814.347 |
| `Europe` | `rpcedge_processed` | 4 | 216.102 | 453.157 | 453.157 | 453.157 |
| `AsiaPacific` | `rpcedge_processed` | 1 | 232.407 | 232.407 | 232.407 | 232.407 |

## Cohort: `leader_city`

| Value | Source | Count | p50 ms | p90 ms | p99 ms | max ms |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| `Amsterdam` | `rpcedge_deshred` | 130 | 246.693 | 971.535 | 3241.785 | 3249.993 |
| `Frankfurt am Main` | `rpcedge_deshred` | 106 | 243.118 | 466.315 | 1729.424 | 1858.702 |
| `America/Chicago` | `rpcedge_deshred` | 30 | 274.047 | 479.566 | 1577.985 | 1577.985 |
| `Ashburn` | `rpcedge_deshred` | 24 | 319.704 | 574.814 | 608.669 | 608.669 |
| `London` | `rpcedge_deshred` | 24 | 420.767 | 505.277 | 2066.279 | 2066.279 |
| `Tokyo` | `rpcedge_deshred` | 23 | 438.627 | 1610.737 | 1795.339 | 1795.339 |
| `Chicago` | `rpcedge_deshred` | 18 | 394.651 | 4739.359 | 6367.252 | 6367.252 |
| `Dublin` | `rpcedge_deshred` | 15 | 276.282 | 1539.890 | 1541.377 | 1541.377 |
| `City of London` | `rpcedge_deshred` | 14 | 419.827 | 455.085 | 516.482 | 516.482 |
| `Los Angeles` | `rpcedge_deshred` | 14 | 400.014 | 1741.771 | 1814.347 | 1814.347 |
| `New York` | `rpcedge_deshred` | 13 | 311.708 | 425.031 | 566.226 | 566.226 |
| `Singapore` | `rpcedge_deshred` | 13 | 409.789 | 436.067 | 1688.653 | 1688.653 |
| `Vilnius` | `rpcedge_deshred` | 11 | 265.897 | 310.080 | 511.652 | 511.652 |
| `Beauharnois` | `rpcedge_deshred` | 10 | 333.222 | 1705.900 | 1705.900 | 1705.900 |
| `Luxembourg` | `rpcedge_deshred` | 10 | 260.398 | 1642.913 | 1642.913 | 1642.913 |
| `Rödelheim` | `rpcedge_deshred` | 8 | 277.792 | 475.356 | 475.356 | 475.356 |
| `Vancouver` | `rpcedge_deshred` | 7 | 390.137 | 1916.359 | 1916.359 | 1916.359 |
| `Asia/Singapore` | `rpcedge_deshred` | 6 | 422.899 | 425.675 | 425.675 | 425.675 |
| `Boardman` | `rpcedge_deshred` | 6 | 166.631 | 375.908 | 375.908 | 375.908 |
| `Fechenheim` | `rpcedge_deshred` | 6 | 335.295 | 335.541 | 335.541 | 335.541 |
| `Stockholm` | `rpcedge_deshred` | 5 | 154.106 | 341.838 | 341.838 | 341.838 |
| `São Paulo` | `rpcedge_deshred` | 5 | 230.223 | 495.734 | 495.734 | 495.734 |
| `Dallas` | `rpcedge_deshred` | 4 | 570.904 | 1738.793 | 1738.793 | 1738.793 |
| `Paris` | `rpcedge_deshred` | 4 | 479.130 | 1715.662 | 1715.662 | 1715.662 |

## Cohort: `data_center_key`

| Value | Source | Count | p50 ms | p90 ms | p99 ms | max ms |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| `20326-NL-Amsterdam` | `rpcedge_deshred` | 71 | 249.007 | 971.535 | 3249.993 | 3249.993 |
| `20326-DE-Frankfurt am Main` | `rpcedge_deshred` | 47 | 243.118 | 325.587 | 1858.702 | 1858.702 |
| `20326-JP-Tokyo` | `rpcedge_deshred` | 17 | 441.819 | 1650.226 | 1795.339 | 1795.339 |
| `7979-NL-Europe/Amsterdam` | `rpcedge_deshred` | 17 | 240.101 | 1647.573 | 1692.931 | 1692.931 |
| `9002-DE-Frankfurt am Main` | `rpcedge_deshred` | 16 | 105.704 | 280.241 | 1725.749 | 1725.749 |
| `14618-US-Ashburn` | `rpcedge_deshred` | 15 | 318.367 | 574.814 | 599.466 | 599.466 |
| `20326-IE-Dublin` | `rpcedge_deshred` | 15 | 276.282 | 1539.890 | 1541.377 | 1541.377 |
| `204770-US-Chicago` | `rpcedge_deshred` | 15 | 436.589 | 4739.359 | 6367.252 | 6367.252 |
| `396356-DE-Frankfurt am Main` | `rpcedge_deshred` | 15 | 270.936 | 474.845 | 1729.424 | 1729.424 |
| `20326-US-America/Chicago` | `rpcedge_deshred` | 14 | 279.736 | 1523.956 | 1577.985 | 1577.985 |
| `400963-GB-Europe/London` | `rpcedge_deshred` | 14 | 433.428 | 446.972 | 2066.279 | 2066.279 |
| `59642-NL-Amsterdam` | `rpcedge_deshred` | 13 | 242.092 | 1657.972 | 1706.228 | 1706.228 |
| `1072-US-America/Chicago` | `rpcedge_deshred` | 11 | 129.460 | 437.184 | 457.710 | 457.710 |
| `16276-CA-Beauharnois` | `rpcedge_deshred` | 10 | 333.222 | 1705.900 | 1705.900 | 1705.900 |
| `20326-GB-City of London` | `rpcedge_deshred` | 10 | 427.148 | 516.482 | 516.482 | 516.482 |
| `396356-US-Los Angeles` | `rpcedge_deshred` | 10 | 392.346 | 1741.771 | 1741.771 | 1741.771 |
| `7979-LU-Europe/Luxembourg` | `rpcedge_deshred` | 10 | 260.398 | 1642.913 | 1642.913 | 1642.913 |
| `213896-DE-Frankfurt am Main` | `rpcedge_deshred` | 9 | 116.173 | 438.269 | 438.269 | 438.269 |
| `396356-US-Ashburn` | `rpcedge_deshred` | 9 | 330.608 | 608.669 | 608.669 | 608.669 |
| `16125-LT-Vilnius` | `rpcedge_deshred` | 8 | 301.826 | 511.652 | 511.652 | 511.652 |
| `262287-NL-Amsterdam` | `rpcedge_deshred` | 8 | 114.008 | 427.817 | 427.817 | 427.817 |
| `395201-DE-Rödelheim` | `rpcedge_deshred` | 8 | 277.792 | 475.356 | 475.356 | 475.356 |
| `6939-CA-Vancouver` | `rpcedge_deshred` | 7 | 390.137 | 1916.359 | 1916.359 | 1916.359 |
| `16509-US-Boardman` | `rpcedge_deshred` | 6 | 166.631 | 375.908 | 375.908 | 375.908 |

## Cohort: `client_family`

| Value | Source | Count | p50 ms | p90 ms | p99 ms | max ms |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| `bam` | `rpcedge_deshred` | 182 | 263.922 | 566.226 | 4739.359 | 6367.252 |
| `jito` | `rpcedge_deshred` | 120 | 250.451 | 590.248 | 1741.771 | 1916.359 |
| `harmonic` | `rpcedge_deshred` | 110 | 353.879 | 1539.890 | 3241.785 | 3249.993 |
| `frankendancer` | `rpcedge_deshred` | 65 | 255.571 | 446.972 | 2066.279 | 2066.279 |
| `rakurai` | `rpcedge_deshred` | 39 | 239.143 | 1545.416 | 1705.900 | 1705.900 |
| `agave` | `rpcedge_deshred` | 12 | 166.631 | 375.900 | 375.908 | 375.908 |
| `firedancer` | `rpcedge_deshred` | 10 | 117.628 | 1692.931 | 1692.931 | 1692.931 |
| `bam` | `rpcedge_processed` | 2 | 232.407 | 232.407 | 232.407 | 232.407 |
| `frankendancer` | `rpcedge_processed` | 2 | 216.102 | 216.102 | 216.102 | 216.102 |
| `jito` | `rpcedge_processed` | 1 | 453.157 | 453.157 | 453.157 | 453.157 |

## Cohort: `client_software`

| Value | Source | Count | p50 ms | p90 ms | p99 ms | max ms |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| `AgaveBam` | `rpcedge_deshred` | 182 | 263.922 | 566.226 | 4739.359 | 6367.252 |
| `JitoLabs` | `rpcedge_deshred` | 120 | 250.451 | 590.248 | 1741.771 | 1916.359 |
| `HarmonicAgave` | `rpcedge_deshred` | 94 | 310.339 | 509.094 | 1541.377 | 1541.377 |
| `Frankendancer` | `rpcedge_deshred` | 65 | 255.571 | 446.972 | 2066.279 | 2066.279 |
| `Rakurai` | `rpcedge_deshred` | 39 | 239.143 | 1545.416 | 1705.900 | 1705.900 |
| `HarmonicFrankendancer` | `rpcedge_deshred` | 13 | 431.413 | 3241.785 | 3249.993 | 3249.993 |
| `Agave` | `rpcedge_deshred` | 12 | 166.631 | 375.900 | 375.908 | 375.908 |
| `Firedancer` | `rpcedge_deshred` | 10 | 117.628 | 1692.931 | 1692.931 | 1692.931 |
| `HarmonicFiredancer` | `rpcedge_deshred` | 3 | 1610.737 | 1650.226 | 1650.226 | 1650.226 |
| `AgaveBam` | `rpcedge_processed` | 2 | 232.407 | 232.407 | 232.407 | 232.407 |
| `Frankendancer` | `rpcedge_processed` | 2 | 216.102 | 216.102 | 216.102 | 216.102 |
| `JitoLabs` | `rpcedge_processed` | 1 | 453.157 | 453.157 | 453.157 | 453.157 |

## Cohort: `stake_bucket`

| Value | Source | Count | p50 ms | p90 ms | p99 ms | max ms |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| `>=1m_sol` | `rpcedge_deshred` | 350 | 264.324 | 576.020 | 1916.359 | 3249.993 |
| `100k_500k_sol` | `rpcedge_deshred` | 112 | 257.639 | 536.343 | 4739.359 | 6367.252 |
| `500k_1m_sol` | `rpcedge_deshred` | 56 | 277.487 | 1591.615 | 3241.785 | 3241.785 |
| `<100k_sol` | `rpcedge_deshred` | 20 | 249.226 | 425.129 | 429.910 | 429.910 |
| `>=1m_sol` | `rpcedge_processed` | 4 | 216.102 | 453.157 | 453.157 | 453.157 |
| `500k_1m_sol` | `rpcedge_processed` | 1 | 232.407 | 232.407 | 232.407 | 232.407 |

## Tail Events

Tail latency uses the first observation time, normally deshred. Transaction slot index uses processed Yellowstone/gRPC metadata when available, because deshred is fastest but processed carries canonical transaction metadata.

| Signature | Source | Latency ms | Arm | Routes | Send slot | Send leader | Send client | Landed slot | Landed leader | Landed client | Tx index | Tx fee | Route tip generated | Route tip landed | Total landed |
| --- | --- | ---: | --- | --- | ---: | --- | --- | ---: | --- | --- | ---: | ---: | ---: | ---: | ---: |
| `3Szn39fw...C9ojQo5t` | `rpcedge_deshred` | 6367.252 | `tpu_quic_only` | `tpu_quic` | 430446884 | `XkCriyrNwS3G4rzAXtG5B1nnvb5Ka1JtCku93VqeKAr (Amsterdam)` | `HarmonicAgave` | 430446900 | `F6kVwubXEfZZo6e4Kozrtg7WoWk5wTRmrC8pwoxSLa7S (Chicago)` | `AgaveBam` | 494 | 5000 | 0 | 0 | 5000 |
| `5SUUoofN...cUtiUTa5` | `rpcedge_deshred` | 4739.359 | `tpu_quic_only` | `tpu_quic` | 430446888 | `qZMH9GWnnBkx7aM1h98iKSv2Lz5N78nwNSocAxDQrbP (Bratislava)` | `AgaveBam` | 430446900 | `F6kVwubXEfZZo6e4Kozrtg7WoWk5wTRmrC8pwoxSLa7S (Chicago)` | `AgaveBam` | 493 | 5000 | 0 | 0 | 5000 |
| `pVFrJK85...H7FQG4er` | `rpcedge_deshred` | 3249.993 | `tpu_quic_only` | `tpu_quic` | 430447280 | `CW9C7HBwAMgqNdXkNgFg9Ujr3edR2Ab9ymEuQnVacd1A (City of London)` | `HarmonicAgave` | 430447287 | `pSoLoZx55zZz61gjxSTwHtwTg4yTwdm7ruBmyjbYgT2 (Amsterdam)` | `HarmonicFrankendancer` | 1361 | 5000 | 0 | 0 | 5000 |
| `58GNyz2C...wHy4ygRt` | `rpcedge_deshred` | 3241.785 | `tpu_quic_only` | `tpu_quic` | 430447596 | `84gC25fbFKYueR9WEfreUysk1n3ZFxLFDDjbyqeqGpoW (Amsterdam)` | `HarmonicFrankendancer` | 430447599 | `84gC25fbFKYueR9WEfreUysk1n3ZFxLFDDjbyqeqGpoW (Amsterdam)` | `HarmonicFrankendancer` | 1786 | 5000 | 0 | 0 | 5000 |
| `4SPsUUxk...HLZmBDQR` | `rpcedge_deshred` | 3098.579 | `tpu_quic_only` | `tpu_quic` | 430447096 | `7EzbSahSfSjeRexHcNDLDpzHBAGBLjLKtjbmuoQnEtjE (America/Chicago)` | `AgaveBam` | 430447103 | `pSoLoZx55zZz61gjxSTwHtwTg4yTwdm7ruBmyjbYgT2 (Amsterdam)` | `HarmonicFrankendancer` | 1336 | 5000 | 0 | 0 | 5000 |
| `5TFoHFMB...ZC8rgdMg` | `rpcedge_deshred` | 2066.279 | `tpu_quic_only` | `tpu_quic` | 430447152 | `CAo1dCGYrB6NhHh5xb1cGjUiu86iyCfMTENxgHumSve4 (America/Chicago)` | `HarmonicAgave` | 430447156 | `9eGrDohdNTAo61DRHyfMuqKWXqYnA3i254Wiszxe8FoY (London)` | `Frankendancer` | 1116 | 5000 | 0 | 0 | 5000 |
| `4A7hGfkL...Bio6bYqm` | `rpcedge_deshred` | 1916.359 | `tpu_quic_only` | `tpu_quic` | 430447192 | `FNKgX9dYUhYQFRTM9bkeKoRpsyEtZGNMxbdQLDzfqB8a (Amsterdam)` | `HarmonicAgave` | 430447196 | `Ninja1spj6n9t5hVYgF3PdnYz2PLnkt7rvaw3firmjs (Vancouver)` | `JitoLabs` | 621 | 5000 | 0 | 0 | 5000 |
| `3AFQVaGn...HnNegLxd` | `rpcedge_deshred` | 1858.702 | `tpu_quic_only` | `tpu_quic` | 430447476 | `JUPiTERrZqgf1jUyR7dSkhMx4Kn2qJyekWsg3LT1h4b (Frankfurt am Main)` | `HarmonicAgave` | 430447480 | `EvnRmnMrd69kFdbLMxWkTn1icZ7DCceRhvmb2SJXqDo4 (Frankfurt am Main)` | `Frankendancer` | 938 | 5000 | 0 | 0 | 5000 |
| `3bJEd5sB...q7mtz5ZU` | `rpcedge_deshred` | 1827.424 | `tpu_quic_only` | `tpu_quic` | 430446936 | `CAo1dCGYrB6NhHh5xb1cGjUiu86iyCfMTENxgHumSve4 (America/Chicago)` | `HarmonicAgave` | 430446940 | `9UbU7oaVXX6t7bMthxzzGPnWumFNxoWqUwX3qsrxb4pp (London)` | `Frankendancer` | 744 | 5000 | 0 | 0 | 5000 |
| `2TyDYSi4...ajTGrnDc` | `rpcedge_deshred` | 1814.347 | `tpu_quic_only` | `tpu_quic` | 430447528 | `q9XWcZ7T1wP4bW9SB4XgNNwjnFEJ982nE8aVbbNuwot (Frankfurt am Main)` | `Rakurai` | 430447532 | `7PpXQgDb9eCHN1Uudgi77Wm89cRz4T85YgDw83qvaJXd (Los Angeles)` | `AgaveBam` | 449 | 5000 | 0 | 0 | 5000 |
| `2dEWKdEZ...rgCqTk4V` | `rpcedge_deshred` | 1795.339 | `tpu_quic_only` | `tpu_quic` | 430447380 | `FNKgX9dYUhYQFRTM9bkeKoRpsyEtZGNMxbdQLDzfqB8a (Amsterdam)` | `HarmonicAgave` | 430447384 | `5EhGYUyQNrxgUbuYF4vbL2SZDT6RMfhq3yjeyevvULeC (Tokyo)` | `AgaveBam` | 158 | 5000 | 0 | 0 | 5000 |
| `5hwMgxdv...MP3qrTzL` | `rpcedge_deshred` | 1741.771 | `tpu_quic_only` | `tpu_quic` | 430447272 | `5ZqveVffQPiUbkjBg4KD9kib1MKHLqiFno4ke9jSq9qk (Frankfurt am Main)` | `HarmonicAgave` | 430447276 | `9rkJMARqK6VBkcxGfKBAwnA44gPAfGxPbPsfsggFNDSQ (Los Angeles)` | `JitoLabs` | 146 | 5000 | 0 | 0 | 5000 |

## Appendix: Sender ACK Diagnostics

ACK confirms that the relay accepted or rejected the request. It is useful for route availability and gateway health debugging, but it is not landing proof and is not used as a performance target.

| Provider | Policy | Routes | Count | Accepted | Errors | p50 ms | p90 ms | p99 ms | max ms |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `rpcedge-quic-frankfurt` | `always_race` | `tpu_quic,jito_bundle,harmonic_bundle` | 558 | 392 | 166 | 251.422 | 251.967 | 252.284 | 252.468 |
| `rpcedge-quic-frankfurt` | `software_client_aware_harmonic` | `tpu_quic,harmonic_bundle` | 135 | 135 | 0 | 251.237 | 252.115 | 252.362 | 252.413 |
| `rpcedge-quic-frankfurt` | `software_client_aware_jito` | `tpu_quic,jito_bundle` | 300 | 298 | 2 | 251.342 | 251.908 | 252.248 | 252.270 |
| `rpcedge-quic-frankfurt` | `software_client_aware_jito_harmonic` | `tpu_quic,jito_bundle,harmonic_bundle` | 39 | 34 | 5 | 251.417 | 251.952 | 252.283 | 252.283 |
| `rpcedge-quic-frankfurt` | `software_client_aware_tpu_only` | `tpu_quic` | 84 | 84 | 0 | 0.941 | 220.063 | 220.957 | 220.957 |
| `rpcedge-quic-frankfurt` | `tpu_quic_only` | `tpu_quic` | 558 | 558 | 0 | 250.825 | 251.829 | 252.248 | 252.478 |
