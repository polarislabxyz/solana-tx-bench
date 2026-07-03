# Transaction Landing Report: `leader-paced-2026-07-03T063147.846Z`

- Generated: `2026-07-03T08:16:06.374Z`
- Artifact dir: `artifacts/leader-paced-2026-07-03T063147.846Z`
- Sent transactions: `2197`
- Provider samples: `2197`
- Observations: `5466`
- Matched caller signatures: `2196`
- Unobserved caller signatures: `1`
- Matched route-tip signatures: `537`
- Leader-slot snapshots: `108`

This report is generated only from local benchmark artifacts. Leader metadata comes from saved `getLeaderSlots` snapshots captured during the run; no ClickHouse or private database join is required.

## Cost Summary

Base fee and priority fee are deterministic fields from the signed benchmark transaction. Route-tip generated spend comes from route-result ACK metadata. Route-tip landed spend is counted only when the route-tip signature was observed in the matched streams.

| Submitted txs | Avg base fee | Avg priority fee | Avg tx fee | Avg route tip generated | Avg route tip landed | Avg total generated | Avg total landed | Generated tip txs | Landed tip txs |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 2197 | 5000.000 | 198.000 | 5198.000 | 8997.724 | 454.711 | 14195.724 | 5652.711 | 19768 | 999 |

## Landing Observations

Landing latency uses the earliest observation per transaction across configured streams. In normal RPCEdge runs this should be the deshred stream; processed metadata is only counted here when it is the first or only observation for a signature.

| First source | Count | p50 ms | p90 ms | p99 ms | max ms | same-slot | slot delta p50 | slot index p50 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `rpcedge_deshred` | 2181 | 108.397 | 224.971 | 436.051 | 3433.314 | 2153 / 98.7% | 0 | - |
| `rpcedge_processed` | 15 | 208.294 | 260.974 | 574.734 | 574.734 | 14 / 93.3% | 0 | 559 |

## Unobserved Signatures

These signatures were submitted by the benchmark but were not observed by the configured landing streams during the collection window. Treat them as the first investigation bucket for dropped transactions, late-beyond-tail transactions, or observation-path misses.

| Signature | Arm | Routes | Send slot | Send leader | Send client | Provider ACK | Route ACKs | Total generated spend |
| --- | --- | --- | ---: | --- | --- | --- | --- | ---: |
| `2JKZEE9Q...GZBNXjV1` | `-` | `tpu_quic+jito_bundle+harmonic_bundle` | 430475052 | `yJeahQNRHNWtL9Z1SqPX3SBwTYXr5ECMYYVK4uYVwxt (London)` | `JitoLabs` | 1/1 accepted (220.204ms) | `tpu_quic:ok:23us | jito_bundle:ok:813us | jito_bundle:ok:6446us | jito_bundle:ok:19343us | jito_bundle:ok:852us | jito_bundle:ok:11770us | jito_bundle:ok:75840us | jito_bundle:ok:118034us | jito_bundle:ok:149044us | jito_bundle:ok:219257us | harmonic_bundle:ok:6234us | harmonic_bundle:ok:75856us | harmonic_bundle:ok:428us | harmonic_bundle:ok:11585us | harmonic_bundle:ok:149222us | harmonic_bundle:ok:213822us` | 14198 |

## Cohort: `leader_region`

| Value | Source | Count | p50 ms | p90 ms | p99 ms | max ms |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| `Europe` | `rpcedge_deshred` | 1293 | 87.923 | 191.926 | 373.575 | 3433.314 |
| `AsiaPacific` | `rpcedge_deshred` | 246 | 223.772 | 334.266 | 542.508 | 700.712 |
| `Unknown` | `rpcedge_deshred` | 224 | 115.683 | 189.689 | 309.002 | 1570.537 |
| `UsEast` | `rpcedge_deshred` | 193 | 156.388 | 204.858 | 1457.193 | 1532.772 |
| `UsCentral` | `rpcedge_deshred` | 139 | 106.477 | 208.033 | 289.983 | 1533.465 |
| `UsWest` | `rpcedge_deshred` | 85 | 164.312 | 183.531 | 552.094 | 552.094 |
| `Europe` | `rpcedge_processed` | 12 | 208.294 | 213.399 | 215.194 | 215.194 |
| `AsiaPacific` | `rpcedge_processed` | 1 | 167.096 | 167.096 | 167.096 | 167.096 |
| `UsCentral` | `rpcedge_processed` | 1 | 574.734 | 574.734 | 574.734 | 574.734 |
| `UsEast` | `rpcedge_processed` | 1 | 260.974 | 260.974 | 260.974 | 260.974 |
| `unknown` | `rpcedge_deshred` | 1 | 201.153 | 201.153 | 201.153 | 201.153 |

## Cohort: `leader_city`

| Value | Source | Count | p50 ms | p90 ms | p99 ms | max ms |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| `Frankfurt am Main` | `rpcedge_deshred` | 459 | 87.305 | 169.173 | 373.575 | 2093.311 |
| `Amsterdam` | `rpcedge_deshred` | 387 | 88.821 | 141.681 | 428.250 | 542.848 |
| `Vilnius` | `rpcedge_deshred` | 116 | 68.998 | 116.660 | 159.456 | 184.518 |
| `Tokyo` | `rpcedge_deshred` | 111 | 231.980 | 314.454 | 450.732 | 542.508 |
| `Beauharnois` | `rpcedge_deshred` | 108 | 121.810 | 157.309 | 222.048 | 256.255 |
| `America/Chicago` | `rpcedge_deshred` | 104 | 89.784 | 150.322 | 289.983 | 1533.465 |
| `London` | `rpcedge_deshred` | 75 | 214.442 | 225.369 | 3433.314 | 3433.314 |
| `Ashburn` | `rpcedge_deshred` | 72 | 128.200 | 193.143 | 258.424 | 258.424 |
| `Singapore` | `rpcedge_deshred` | 61 | 211.341 | 347.366 | 419.690 | 419.690 |
| `Los Angeles` | `rpcedge_deshred` | 54 | 162.256 | 196.921 | 552.094 | 552.094 |
| `City of London` | `rpcedge_deshred` | 48 | 101.686 | 130.769 | 329.664 | 329.664 |
| `New York` | `rpcedge_deshred` | 36 | 160.113 | 176.886 | 188.738 | 188.738 |
| `Stockholm` | `rpcedge_deshred` | 36 | 42.029 | 241.107 | 1570.537 | 1570.537 |
| `Asia/Singapore` | `rpcedge_deshred` | 33 | 171.519 | 223.702 | 312.163 | 312.163 |
| `Chicago` | `rpcedge_deshred` | 32 | 141.212 | 280.403 | 286.761 | 286.761 |
| `Boardman` | `rpcedge_deshred` | 31 | 170.359 | 179.920 | 189.344 | 189.344 |
| `Luxembourg` | `rpcedge_deshred` | 31 | 107.912 | 158.720 | 1495.249 | 1495.249 |
| `Rödelheim` | `rpcedge_deshred` | 31 | 68.500 | 123.150 | 1498.504 | 1498.504 |
| `Dublin` | `rpcedge_deshred` | 30 | 103.190 | 133.869 | 148.647 | 148.647 |
| `Fechenheim` | `rpcedge_deshred` | 30 | 82.299 | 263.246 | 302.570 | 302.570 |
| `Rüsselsheim am Main` | `rpcedge_deshred` | 28 | 92.059 | 116.357 | 166.714 | 166.714 |
| `Newark` | `rpcedge_deshred` | 27 | 160.095 | 218.270 | 1532.772 | 1532.772 |
| `Offenbach` | `rpcedge_deshred` | 26 | 76.143 | 91.379 | 107.461 | 107.461 |
| `Sterling` | `rpcedge_deshred` | 15 | 166.715 | 188.239 | 221.382 | 221.382 |

## Cohort: `data_center_key`

| Value | Source | Count | p50 ms | p90 ms | p99 ms | max ms |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| `20326-DE-Frankfurt am Main` | `rpcedge_deshred` | 231 | 89.441 | 212.024 | 400.531 | 2093.311 |
| `20326-NL-Amsterdam` | `rpcedge_deshred` | 225 | 83.740 | 111.378 | 173.115 | 479.544 |
| `16276-CA-Beauharnois` | `rpcedge_deshred` | 108 | 121.810 | 157.309 | 222.048 | 256.255 |
| `20326-JP-Tokyo` | `rpcedge_deshred` | 77 | 226.458 | 317.676 | 542.508 | 542.508 |
| `16125-LT-Vilnius` | `rpcedge_deshred` | 74 | 68.755 | 113.036 | 159.456 | 159.456 |
| `14618-US-Ashburn` | `rpcedge_deshred` | 67 | 126.357 | 195.227 | 258.424 | 258.424 |
| `396356-DE-Frankfurt am Main` | `rpcedge_deshred` | 66 | 77.881 | 150.760 | 479.094 | 479.094 |
| `400963-GB-Europe/London` | `rpcedge_deshred` | 63 | 215.398 | 225.369 | 236.585 | 236.585 |
| `213896-DE-Frankfurt am Main` | `rpcedge_deshred` | 62 | 87.999 | 144.697 | 338.187 | 338.187 |
| `1072-US-America/Chicago` | `rpcedge_deshred` | 51 | 106.477 | 150.322 | 289.983 | 289.983 |
| `20326-US-America/Chicago` | `rpcedge_deshred` | 45 | 57.477 | 94.003 | 1533.465 | 1533.465 |
| `7979-NL-Europe/Amsterdam` | `rpcedge_deshred` | 41 | 118.307 | 251.646 | 275.677 | 275.677 |
| `16125-LT-Europe/Vilnius` | `rpcedge_deshred` | 40 | 68.998 | 117.416 | 122.225 | 122.225 |
| `9002-DE-Frankfurt am Main` | `rpcedge_deshred` | 39 | 106.880 | 123.900 | 138.216 | 138.216 |
| `214159-SE-Stockholm` | `rpcedge_deshred` | 35 | 42.029 | 241.107 | 1570.537 | 1570.537 |
| `396356-US-Los Angeles` | `rpcedge_deshred` | 33 | 163.149 | 180.844 | 552.094 | 552.094 |
| `60068-NL-Amsterdam` | `rpcedge_deshred` | 33 | 100.539 | 135.134 | 143.147 | 143.147 |
| `16509-US-Boardman` | `rpcedge_deshred` | 31 | 170.359 | 179.920 | 189.344 | 189.344 |
| `20326-SG-Singapore` | `rpcedge_deshred` | 31 | 206.463 | 223.272 | 384.997 | 384.997 |
| `395201-DE-Rödelheim` | `rpcedge_deshred` | 31 | 68.500 | 123.150 | 1498.504 | 1498.504 |
| `7979-LU-Europe/Luxembourg` | `rpcedge_deshred` | 31 | 107.912 | 158.720 | 1495.249 | 1495.249 |
| `395201-DE-Fechenheim` | `rpcedge_deshred` | 30 | 82.299 | 263.246 | 302.570 | 302.570 |
| `20326-GB-City of London` | `rpcedge_deshred` | 28 | 104.556 | 133.744 | 148.816 | 148.816 |
| `395201-DE-Rüsselsheim am Main` | `rpcedge_deshred` | 28 | 92.059 | 116.357 | 166.714 | 166.714 |

## Cohort: `client_family`

| Value | Source | Count | p50 ms | p90 ms | p99 ms | max ms |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| `bam` | `rpcedge_deshred` | 700 | 100.336 | 223.827 | 286.761 | 3433.314 |
| `harmonic` | `rpcedge_deshred` | 515 | 109.785 | 260.172 | 542.848 | 2093.311 |
| `jito` | `rpcedge_deshred` | 461 | 82.299 | 236.088 | 552.094 | 1570.537 |
| `rakurai` | `rpcedge_deshred` | 222 | 111.525 | 154.301 | 260.103 | 1498.504 |
| `frankendancer` | `rpcedge_deshred` | 181 | 179.251 | 226.834 | 373.575 | 400.531 |
| `agave` | `rpcedge_deshred` | 54 | 162.568 | 181.160 | 270.674 | 270.674 |
| `firedancer` | `rpcedge_deshred` | 45 | 94.355 | 118.307 | 142.589 | 142.589 |
| `bam` | `rpcedge_processed` | 5 | 86.732 | 260.974 | 260.974 | 260.974 |
| `frankendancer` | `rpcedge_processed` | 5 | 213.353 | 215.194 | 215.194 | 215.194 |
| `harmonic` | `rpcedge_processed` | 4 | 208.294 | 574.734 | 574.734 | 574.734 |
| `unknown` | `rpcedge_deshred` | 3 | 201.153 | 260.631 | 260.631 | 260.631 |
| `jito` | `rpcedge_processed` | 1 | 92.149 | 92.149 | 92.149 | 92.149 |

## Cohort: `client_software`

| Value | Source | Count | p50 ms | p90 ms | p99 ms | max ms |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| `AgaveBam` | `rpcedge_deshred` | 697 | 100.444 | 223.827 | 286.761 | 3433.314 |
| `HarmonicAgave` | `rpcedge_deshred` | 470 | 107.348 | 257.360 | 486.767 | 2093.311 |
| `JitoLabs` | `rpcedge_deshred` | 461 | 82.299 | 236.088 | 552.094 | 1570.537 |
| `Rakurai` | `rpcedge_deshred` | 222 | 111.525 | 154.301 | 260.103 | 1498.504 |
| `Frankendancer` | `rpcedge_deshred` | 181 | 179.251 | 226.834 | 373.575 | 400.531 |
| `Agave` | `rpcedge_deshred` | 54 | 162.568 | 181.160 | 270.674 | 270.674 |
| `Firedancer` | `rpcedge_deshred` | 45 | 94.355 | 118.307 | 142.589 | 142.589 |
| `HarmonicFrankendancer` | `rpcedge_deshred` | 44 | 170.993 | 320.657 | 1532.772 | 1532.772 |
| `AgaveBam` | `rpcedge_processed` | 5 | 86.732 | 260.974 | 260.974 | 260.974 |
| `Frankendancer` | `rpcedge_processed` | 5 | 213.353 | 215.194 | 215.194 | 215.194 |
| `FireBam` | `rpcedge_deshred` | 3 | 88.233 | 90.830 | 90.830 | 90.830 |
| `HarmonicAgave` | `rpcedge_processed` | 2 | 207.243 | 207.243 | 207.243 | 207.243 |
| `HarmonicFrankendancer` | `rpcedge_processed` | 2 | 574.734 | 574.734 | 574.734 | 574.734 |
| `HarmonicFiredancer` | `rpcedge_deshred` | 1 | 542.508 | 542.508 | 542.508 | 542.508 |
| `JitoLabs` | `rpcedge_processed` | 1 | 92.149 | 92.149 | 92.149 | 92.149 |
| `Raiku` | `rpcedge_deshred` | 1 | 30.724 | 30.724 | 30.724 | 30.724 |
| `Unknown` | `rpcedge_deshred` | 1 | 260.631 | 260.631 | 260.631 | 260.631 |
| `unknown` | `rpcedge_deshred` | 1 | 201.153 | 201.153 | 201.153 | 201.153 |

## Cohort: `stake_bucket`

| Value | Source | Count | p50 ms | p90 ms | p99 ms | max ms |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| `>=1m_sol` | `rpcedge_deshred` | 1573 | 107.670 | 219.542 | 373.575 | 2093.311 |
| `100k_500k_sol` | `rpcedge_deshred` | 349 | 111.505 | 259.216 | 1457.193 | 3433.314 |
| `500k_1m_sol` | `rpcedge_deshred` | 204 | 110.424 | 239.438 | 384.997 | 662.290 |
| `<100k_sol` | `rpcedge_deshred` | 54 | 161.829 | 267.708 | 432.927 | 432.927 |
| `>=1m_sol` | `rpcedge_processed` | 9 | 211.888 | 215.194 | 215.194 | 215.194 |
| `100k_500k_sol` | `rpcedge_processed` | 3 | 260.974 | 574.734 | 574.734 | 574.734 |
| `500k_1m_sol` | `rpcedge_processed` | 3 | 34.144 | 208.294 | 208.294 | 208.294 |
| `unknown` | `rpcedge_deshred` | 1 | 201.153 | 201.153 | 201.153 | 201.153 |

## Tail Events

Tail latency uses the first observation time, normally deshred. Transaction slot index uses processed Yellowstone/gRPC metadata when available, because deshred is fastest but processed carries canonical transaction metadata.

| Signature | Source | Latency ms | Arm | Routes | Send slot | Send leader | Send client | Landed slot | Landed leader | Landed client | Tx index | Tx fee | Route tip generated | Route tip landed | Total landed |
| --- | --- | ---: | --- | --- | ---: | --- | --- | ---: | --- | --- | ---: | ---: | ---: | ---: | ---: |
| `22NH2N2D...3aaSarRb` | `rpcedge_deshred` | 3433.314 | `-` | `tpu_quic+jito_bundle+harmonic_bundle` | 430473280 | `dcntruDNP5SEcGV4RxnsqXFURdDZGT3DTQv68Q8H7Vu (Tsuen Wan)` | `JitoLabs` | 430473288 | `CVvaeDPR2o7P1eawG5c9TPFLzSXAewwPovPmREaEL4Cm (London)` | `AgaveBam` | 784 | 5198 | 9000 | 1000 | 6198 |
| `3yG8HZWW...nECVMbyG` | `rpcedge_deshred` | 2093.311 | `-` | `tpu_quic+jito_bundle+harmonic_bundle` | 430468756 | `3tm92VTxwyZ5MDhGoYR4tVTkwWYkzfam6hwBjauUACCk (Minamishinagawa)` | `JitoLabs` | 430468760 | `EXckihF3qmguH5znjhfzLvHsbk2E3nEW2DqNh4MMnDMm (Frankfurt am Main)` | `HarmonicAgave` | 757 | 5198 | 9000 | 2000 | 7198 |
| `5M9nHoLe...Y75xUZBX` | `rpcedge_deshred` | 1570.537 | `-` | `tpu_quic+jito_bundle+harmonic_bundle` | 430475660 | `8nbE53mcKhy74HLiGZ1q5HRocwiCvgh49csSaHSdtukr (London)` | `JitoLabs` | 430475664 | `FbYX2uN573G5WsgiPdHU6fS5PNUyjdXfGfpZNkYUuT4k (Stockholm)` | `JitoLabs` | 587 | 5198 | 9000 | 2000 | 7198 |
| `33e6njYs...NwUxc1Ua` | `rpcedge_deshred` | 1533.465 | `-` | `tpu_quic+jito_bundle+harmonic_bundle` | 430468196 | `8nbE53mcKhy74HLiGZ1q5HRocwiCvgh49csSaHSdtukr (London)` | `JitoLabs` | 430468200 | `E1r4Psq84tHfQ6aPTvvDka4U3u8zPVD7gEUrH25RdxHL (America/Chicago)` | `JitoLabs` | 409 | 5198 | 9000 | 1000 | 6198 |
| `2TRMLnFZ...5eQD9RmG` | `rpcedge_deshred` | 1532.772 | `-` | `tpu_quic+jito_bundle+harmonic_bundle` | 430476776 | `FugJZepeGfh1Ruunhep19JC4F3Hr2FL3oKUMezoK8ajp (Newark)` | `HarmonicFrankendancer` | 430476779 | `FugJZepeGfh1Ruunhep19JC4F3Hr2FL3oKUMezoK8ajp (Newark)` | `HarmonicFrankendancer` | 89 | 5198 | 9000 | 1000 | 6198 |
| `5PPXRJSz...RV682n9o` | `rpcedge_deshred` | 1498.504 | `-` | `tpu_quic+jito_bundle+harmonic_bundle` | 430474280 | `chrtyETASKQhsndRM9pr6qC3gAHG5MuRwCgXSNVqnJL (Tsuen Wan)` | `JitoLabs` | 430474284 | `TopjgY7N1fJdnW89S9fX6t7LF61nspgXGL1NpgAKhDG (Rödelheim)` | `Rakurai` | 258 | 5198 | 9000 | 1000 | 6198 |
| `67L67Kni...ZQiDx1s3` | `rpcedge_deshred` | 1495.249 | `-` | `tpu_quic+jito_bundle+harmonic_bundle` | 430475388 | `CoG8d9Fp2TFJRkAmrPMiPsGhQWHzdTTVoegEp9svRgmJ (Singapore)` | `JitoLabs` | 430475392 | `5pPRHniefFjkiaArbGX3Y8NUysJmQ9tMZg3FrFGwHzSm (Luxembourg)` | `JitoLabs` | 538 | 5198 | 9000 | 1000 | 6198 |
| `5T1hW4tN...CQFGXrLt` | `rpcedge_deshred` | 1457.193 | `-` | `tpu_quic+jito_bundle+harmonic_bundle` | 430469456 | `FugJZepeGfh1Ruunhep19JC4F3Hr2FL3oKUMezoK8ajp (Newark)` | `HarmonicFrankendancer` | 430469459 | `FugJZepeGfh1Ruunhep19JC4F3Hr2FL3oKUMezoK8ajp (Newark)` | `HarmonicFrankendancer` | 84 | 5198 | 9000 | 1000 | 6198 |
| `3LzTvmGQ...xgRYE6rA` | `rpcedge_deshred` | 700.712 | `-` | `tpu_quic+jito_bundle+harmonic_bundle` | 430468752 | `8hAYbagNt7CMBooFfqVJhBgLqLffpjXTWJMk8yybjJsN (Asia/Hong_Kong)` | `HarmonicAgave` | 430468753 | `8hAYbagNt7CMBooFfqVJhBgLqLffpjXTWJMk8yybjJsN (Asia/Hong_Kong)` | `HarmonicAgave` | 933 | 5198 | 9000 | 0 | 5198 |
| `3LyZBQWg...3XLfDgTC` | `rpcedge_deshred` | 662.290 | `-` | `tpu_quic+jito_bundle+harmonic_bundle` | 430473396 | `Ha1iade1AH3B12K9SccfWoPdFtQKKQsj2ZyWwxcjqJJU (Asia/Tokyo)` | `HarmonicAgave` | 430473397 | `Ha1iade1AH3B12K9SccfWoPdFtQKKQsj2ZyWwxcjqJJU (Asia/Tokyo)` | `HarmonicAgave` | 203 | 5198 | 9000 | 2000 | 7198 |
| `2xcJsrtS...89j8Kkux` | `rpcedge_deshred` | 588.537 | `-` | `tpu_quic+jito_bundle+harmonic_bundle` | 430475288 | `8ZQg3K1V1Z2BVJkjmnxpi43WKhjPGXphzu5QmBkJibSP (São Paulo)` | `JitoLabs` | 430475289 | `8ZQg3K1V1Z2BVJkjmnxpi43WKhjPGXphzu5QmBkJibSP (São Paulo)` | `JitoLabs` | 90 | 5198 | 9000 | 0 | 5198 |
| `nZDUEuZQ...iuG97h1J` | `rpcedge_processed` | 574.734 | `-` | `tpu_quic+jito_bundle+harmonic_bundle` | 430475292 | `zeroT6PTAEjipvZuACTh1mbGCqTHgA6i1ped9DcuidX (America/Chicago)` | `HarmonicFrankendancer` | 430475293 | `zeroT6PTAEjipvZuACTh1mbGCqTHgA6i1ped9DcuidX (America/Chicago)` | `HarmonicFrankendancer` | 128 | 5198 | 9000 | 1000 | 6198 |

## Appendix: Sender ACK Diagnostics

ACK confirms that the relay accepted or rejected the request. It is useful for route availability and gateway health debugging, but it is not landing proof and is not used as a performance target.

| Provider | Policy | Routes | Count | Accepted | Errors | p50 ms | p90 ms | p99 ms | max ms |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `rpcedge-quic-frankfurt` | `always_race` | `tpu_quic,jito_bundle,harmonic_bundle` | 2197 | 2197 | 0 | 219.759 | 220.080 | 220.787 | 642.485 |
