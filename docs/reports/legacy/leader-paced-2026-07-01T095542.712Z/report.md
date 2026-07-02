# Transaction Landing Report: `leader-paced-2026-07-01T095542.712Z`

- Generated: `2026-07-02T02:43:33.365Z`
- Artifact dir: `/tmp/rpcedge-paired-artifact`
- Sent transactions: `564`
- Provider samples: `564`
- Observations: `1112`
- Matched signatures: `556`
- Leader-slot snapshots: `10`

This report is generated only from local benchmark artifacts. Leader metadata comes from saved `getLeaderSlots` snapshots captured during the run; no ClickHouse or private database join is required.

## Landing Observations

Landing latency uses the earliest observation per transaction across configured streams. In normal RPCEdge runs this should be the deshred stream; processed metadata is only counted here when it is the first or only observation for a signature.

| First source | Count | p50 ms | p90 ms | p99 ms | max ms | same-slot | slot delta p50 | slot index p50 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `rpcedge_deshred` | 556 | 103.957 | 248.073 | 1732.464 | 20843.274 | 535 / 96.2% | 0 | - |

## Cohort: `leader_region`

| Value | Source | Count | p50 ms | p90 ms | p99 ms | max ms |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| `Europe` | `rpcedge_deshred` | 373 | 91.343 | 248.073 | 3060.467 | 20843.274 |
| `AsiaPacific` | `rpcedge_deshred` | 62 | 216.581 | 252.218 | 692.045 | 692.045 |
| `Unknown` | `rpcedge_deshred` | 57 | 95.259 | 143.022 | 149.070 | 149.070 |
| `UsEast` | `rpcedge_deshred` | 33 | 159.340 | 593.374 | 1640.582 | 1640.582 |
| `UsCentral` | `rpcedge_deshred` | 19 | 129.251 | 960.849 | 1027.783 | 1027.783 |
| `UsWest` | `rpcedge_deshred` | 12 | 172.568 | 172.650 | 172.654 | 172.654 |

## Cohort: `leader_city`

| Value | Source | Count | p50 ms | p90 ms | p99 ms | max ms |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| `Amsterdam` | `rpcedge_deshred` | 106 | 93.997 | 147.660 | 591.192 | 20843.274 |
| `Frankfurt am Main` | `rpcedge_deshred` | 102 | 92.440 | 257.873 | 3060.467 | 3060.475 |
| `City of London` | `rpcedge_deshred` | 33 | 96.064 | 110.116 | 145.724 | 145.724 |
| `Vilnius` | `rpcedge_deshred` | 33 | 64.319 | 112.696 | 148.468 | 148.468 |
| `London` | `rpcedge_deshred` | 30 | 122.837 | 417.882 | 431.414 | 431.414 |
| `Tokyo` | `rpcedge_deshred` | 24 | 224.091 | 287.809 | 692.045 | 692.045 |
| `Beauharnois` | `rpcedge_deshred` | 21 | 120.333 | 149.054 | 149.070 | 149.070 |
| `Asia/Singapore` | `rpcedge_deshred` | 20 | 186.327 | 223.355 | 223.366 | 223.366 |
| `Rödelheim` | `rpcedge_deshred` | 15 | 89.633 | 407.455 | 407.461 | 407.461 |
| `Singapore` | `rpcedge_deshred` | 12 | 173.536 | 232.882 | 232.887 | 232.887 |
| `Stockholm` | `rpcedge_deshred` | 12 | 46.746 | 65.099 | 65.104 | 65.104 |
| `America/Chicago` | `rpcedge_deshred` | 10 | 56.901 | 1027.783 | 1027.783 | 1027.783 |
| `Ashburn` | `rpcedge_deshred` | 9 | 121.093 | 159.348 | 159.348 | 159.348 |
| `Chicago` | `rpcedge_deshred` | 9 | 129.259 | 323.894 | 323.894 | 323.894 |
| `Dublin` | `rpcedge_deshred` | 9 | 80.692 | 140.112 | 140.112 | 140.112 |
| `Luxembourg` | `rpcedge_deshred` | 9 | 220.977 | 248.073 | 248.073 | 248.073 |
| `Rüsselsheim am Main` | `rpcedge_deshred` | 9 | 91.708 | 174.251 | 174.251 | 174.251 |
| `Aubervilliers` | `rpcedge_deshred` | 6 | 30.124 | 30.140 | 30.140 | 30.140 |
| `Berlin` | `rpcedge_deshred` | 6 | 68.875 | 68.890 | 68.890 | 68.890 |
| `Boardman` | `rpcedge_deshred` | 6 | 172.642 | 172.654 | 172.654 | 172.654 |
| `Frankfurt (Oder)` | `rpcedge_deshred` | 6 | 28.774 | 28.787 | 28.787 | 28.787 |
| `Gurugram` | `rpcedge_deshred` | 6 | 92.928 | 92.945 | 92.945 | 92.945 |
| `Indian/Mahe` | `rpcedge_deshred` | 6 | 95.245 | 95.265 | 95.265 | 95.265 |
| `Los Angeles` | `rpcedge_deshred` | 6 | 172.568 | 172.589 | 172.589 | 172.589 |

## Cohort: `data_center_key`

| Value | Source | Count | p50 ms | p90 ms | p99 ms | max ms |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| `20326-NL-Amsterdam` | `rpcedge_deshred` | 51 | 86.540 | 112.298 | 168.085 | 168.085 |
| `20326-DE-Frankfurt am Main` | `rpcedge_deshred` | 30 | 87.326 | 257.873 | 257.887 | 257.887 |
| `16125-LT-Vilnius` | `rpcedge_deshred` | 24 | 64.305 | 112.682 | 112.696 | 112.696 |
| `16276-CA-Beauharnois` | `rpcedge_deshred` | 21 | 120.333 | 149.054 | 149.070 | 149.070 |
| `396356-DE-Frankfurt am Main` | `rpcedge_deshred` | 21 | 105.440 | 468.584 | 468.599 | 468.599 |
| `20326-JP-Tokyo` | `rpcedge_deshred` | 18 | 217.135 | 236.562 | 236.567 | 236.567 |
| `29066-DE-Frankfurt am Main` | `rpcedge_deshred` | 18 | 34.760 | 92.141 | 92.152 | 92.152 |
| `399460-GB-City of London` | `rpcedge_deshred` | 18 | 69.502 | 96.536 | 99.067 | 99.067 |
| `60068-NL-Amsterdam` | `rpcedge_deshred` | 18 | 94.627 | 138.905 | 154.037 | 154.037 |
| `395201-DE-Rödelheim` | `rpcedge_deshred` | 15 | 89.633 | 407.455 | 407.461 | 407.461 |
| `45102-SG-Asia/Singapore` | `rpcedge_deshred` | 15 | 177.136 | 216.576 | 216.581 | 216.581 |
| `20326-NL-Europe/Amsterdam` | `rpcedge_deshred` | 13 | 107.205 | 318.671 | 20843.274 | 20843.274 |
| `214159-SE-Stockholm` | `rpcedge_deshred` | 12 | 46.746 | 65.099 | 65.104 | 65.104 |
| `9002-DE-Frankfurt am Main` | `rpcedge_deshred` | 12 | 113.160 | 113.629 | 113.634 | 113.634 |
| `14618-US-Ashburn` | `rpcedge_deshred` | 9 | 121.093 | 159.348 | 159.348 | 159.348 |
| `16125-LT-Europe/Vilnius` | `rpcedge_deshred` | 9 | 65.513 | 148.468 | 148.468 | 148.468 |
| `16276-EN-London` | `rpcedge_deshred` | 9 | 252.712 | 263.910 | 263.910 | 263.910 |
| `20326-GB-City of London` | `rpcedge_deshred` | 9 | 103.877 | 145.724 | 145.724 | 145.724 |
| `204770-US-Chicago` | `rpcedge_deshred` | 9 | 129.259 | 323.894 | 323.894 | 323.894 |
| `262287-NL-Amsterdam` | `rpcedge_deshred` | 9 | 114.099 | 147.660 | 147.660 | 147.660 |
| `395201-DE-Rüsselsheim am Main` | `rpcedge_deshred` | 9 | 91.708 | 174.251 | 174.251 | 174.251 |
| `396356-GB-London` | `rpcedge_deshred` | 9 | 70.538 | 100.190 | 100.190 | 100.190 |
| `400963-GB-Europe/London` | `rpcedge_deshred` | 9 | 415.595 | 431.414 | 431.414 | 431.414 |
| `44051-DE-Frankfurt am Main` | `rpcedge_deshred` | 9 | 226.380 | 240.321 | 240.321 | 240.321 |

## Cohort: `client_family`

| Value | Source | Count | p50 ms | p90 ms | p99 ms | max ms |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| `bam` | `rpcedge_deshred` | 221 | 95.245 | 219.013 | 323.889 | 692.045 |
| `harmonic` | `rpcedge_deshred` | 128 | 107.213 | 556.428 | 3060.475 | 20843.274 |
| `jito` | `rpcedge_deshred` | 99 | 78.985 | 222.547 | 3891.270 | 3891.270 |
| `rakurai` | `rpcedge_deshred` | 60 | 114.227 | 263.896 | 468.599 | 468.599 |
| `frankendancer` | `rpcedge_deshred` | 27 | 113.629 | 417.882 | 431.414 | 431.414 |
| `agave` | `rpcedge_deshred` | 18 | 64.305 | 172.650 | 172.654 | 172.654 |
| `firedancer` | `rpcedge_deshred` | 3 | 108.102 | 108.108 | 108.108 | 108.108 |

## Cohort: `client_software`

| Value | Source | Count | p50 ms | p90 ms | p99 ms | max ms |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| `AgaveBam` | `rpcedge_deshred` | 221 | 95.245 | 219.013 | 323.889 | 692.045 |
| `HarmonicAgave` | `rpcedge_deshred` | 104 | 104.684 | 272.131 | 1027.783 | 20843.274 |
| `JitoLabs` | `rpcedge_deshred` | 99 | 78.985 | 222.547 | 3891.270 | 3891.270 |
| `Rakurai` | `rpcedge_deshred` | 60 | 114.227 | 263.896 | 468.599 | 468.599 |
| `Frankendancer` | `rpcedge_deshred` | 27 | 113.629 | 417.882 | 431.414 | 431.414 |
| `HarmonicFrankendancer` | `rpcedge_deshred` | 24 | 219.116 | 1732.479 | 3060.475 | 3060.475 |
| `Agave` | `rpcedge_deshred` | 18 | 64.305 | 172.650 | 172.654 | 172.654 |
| `Firedancer` | `rpcedge_deshred` | 3 | 108.102 | 108.108 | 108.108 | 108.108 |

## Cohort: `stake_bucket`

| Value | Source | Count | p50 ms | p90 ms | p99 ms | max ms |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| `>=1m_sol` | `rpcedge_deshred` | 389 | 103.143 | 236.552 | 1126.618 | 20843.274 |
| `100k_500k_sol` | `rpcedge_deshred` | 84 | 113.132 | 248.046 | 1732.479 | 1732.479 |
| `500k_1m_sol` | `rpcedge_deshred` | 66 | 107.205 | 272.131 | 3060.475 | 3060.475 |
| `<100k_sol` | `rpcedge_deshred` | 17 | 153.662 | 264.660 | 264.669 | 264.669 |

## Tail Events

| Signature | Source | Latency ms | Send slot | Landed slot | Slot index | Region | City | Client |
| --- | --- | ---: | ---: | ---: | ---: | --- | --- | --- |
| `4ucVyy7c...9ihuu45F` | `rpcedge_deshred` | 20843.274 | 430070084 | 430070132 | - | `Europe` | `Amsterdam` | `HarmonicAgave` |
| `5fM5SzgS...SLkFt5Lv` | `rpcedge_deshred` | 3891.270 | 430070136 | 430070145 | - | `Europe` | `Offenbach` | `JitoLabs` |
| `5LbB7t11...ENHR5SMP` | `rpcedge_deshred` | 3060.475 | 430070128 | 430070131 | - | `Europe` | `Frankfurt am Main` | `HarmonicFrankendancer` |
| `3QR4Dq7E...eTutVeWx` | `rpcedge_deshred` | 3060.467 | 430070128 | 430070131 | - | `Europe` | `Frankfurt am Main` | `HarmonicFrankendancer` |
| `4mvyy7zy...CQzf34PJ` | `rpcedge_deshred` | 1732.479 | 430069792 | 430069795 | - | `Europe` | `Frankfurt am Main` | `HarmonicFrankendancer` |
| `2ap5KW4m...Ceus495k` | `rpcedge_deshred` | 1732.464 | 430069792 | 430069795 | - | `Europe` | `Frankfurt am Main` | `HarmonicFrankendancer` |
| `5fy1L5wW...HrFWSzQG` | `rpcedge_deshred` | 1640.582 | 430069744 | 430069747 | - | `UsEast` | `Newark` | `HarmonicFrankendancer` |
| `5qCdxBjM...Lpm7oq5Q` | `rpcedge_deshred` | 1126.618 | 430069744 | 430069746 | - | `UsEast` | `Newark` | `HarmonicFrankendancer` |
| `5xzJiGXy...iD3xgwcv` | `rpcedge_deshred` | 1027.783 | 430069976 | 430069978 | - | `UsCentral` | `America/Chicago` | `HarmonicAgave` |
| `rYVAXkgq...iCvE5wxd` | `rpcedge_deshred` | 960.849 | 430069668 | 430069670 | - | `UsCentral` | `America/Chicago` | `HarmonicAgave` |
| `4VvJqpCG...jswGVGsC` | `rpcedge_deshred` | 692.045 | 430069828 | 430069829 | - | `AsiaPacific` | `Tokyo` | `AgaveBam` |
| `5bDRKykF...BGh1y5E3` | `rpcedge_deshred` | 652.241 | 430069614 | 430069615 | - | `UsEast` | `Ogden` | `JitoLabs` |

## Appendix: Sender ACK Diagnostics

ACK confirms that the relay accepted or rejected the request. It is useful for route availability and gateway health debugging, but it is not landing proof and is not used as a performance target.

| Provider | Policy | Routes | Count | Accepted | Errors | p50 ms | p90 ms | p99 ms | max ms |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `rpcedge-quic-frankfurt` | `always_race` | `tpu_quic,jito_bundle,harmonic_bundle` | 188 | 188 | 0 | 0.358 | 0.461 | 0.630 | 0.666 |
| `rpcedge-quic-frankfurt` | `software_client_aware_harmonic` | `tpu_quic,harmonic_bundle` | 45 | 45 | 0 | 0.351 | 0.463 | 0.628 | 0.628 |
| `rpcedge-quic-frankfurt` | `software_client_aware_jito` | `tpu_quic,jito_bundle` | 107 | 107 | 0 | 0.349 | 0.469 | 0.556 | 0.636 |
| `rpcedge-quic-frankfurt` | `software_client_aware_tpu_only` | `tpu_quic` | 36 | 36 | 0 | 0.366 | 0.483 | 0.548 | 0.548 |
| `rpcedge-quic-frankfurt` | `tpu_quic_only` | `tpu_quic` | 188 | 188 | 0 | 0.354 | 0.469 | 0.652 | 0.663 |
