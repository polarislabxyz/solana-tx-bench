use crate::paired_analysis::{
    CohortSummary, ComparisonMetrics, MetricSummary, PairedAnalysis, PairedRow,
};
use anyhow::{Context, Result};
use chrono::{DateTime, SecondsFormat, Utc};
use serde::Serialize;
use std::{fs, path::Path};

#[derive(Serialize)]
struct PublicSummary<'a> {
    title: &'a str,
    generated_at: DateTime<Utc>,
    #[serde(flatten)]
    analysis: &'a PairedAnalysis,
}

pub(crate) fn write_paired_report(
    output_dir: &Path,
    title: &str,
    generated_at: DateTime<Utc>,
    analysis: &PairedAnalysis,
    rows: &[PairedRow],
) -> Result<()> {
    fs::create_dir_all(output_dir).with_context(|| format!("create {}", output_dir.display()))?;
    let summary = PublicSummary {
        title,
        generated_at,
        analysis,
    };
    write_json(output_dir.join("summary.json"), &summary)?;
    write_json(output_dir.join("paired-rows.json"), rows)?;
    fs::write(
        output_dir.join("index.html"),
        render_html(title, generated_at, analysis),
    )
    .with_context(|| format!("write {}", output_dir.join("index.html").display()))?;
    Ok(())
}

fn write_json(path: impl AsRef<Path>, value: &(impl Serialize + ?Sized)) -> Result<()> {
    let path = path.as_ref();
    fs::write(path, serde_json::to_vec_pretty(value)?)
        .with_context(|| format!("write {}", path.display()))
}

fn render_html(title: &str, generated_at: DateTime<Utc>, analysis: &PairedAnalysis) -> String {
    let baseline = html_escape(&arm_label(&analysis.baseline_arm));
    let candidate = html_escape(&arm_label(&analysis.candidate_arm));
    let metrics = &analysis.same_slot_metrics;
    let poh = &metrics.preceding_poh_ticks;
    let mut out = String::from(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">",
    );
    out.push_str(&format!("<title>{}</title>", html_escape(title)));
    out.push_str(
        "<style>
:root{color-scheme:light;--ink:#17202a;--muted:#586672;--line:#d9e0e5;--soft:#f4f7f8;--candidate:#087f5b;--baseline:#b54708;--link:#0b63ce}
*{box-sizing:border-box}body{margin:0;color:var(--ink);background:#fff;font:15px/1.55 system-ui,-apple-system,Segoe UI,sans-serif;letter-spacing:0}main,footer{width:min(1240px,calc(100% - 40px));margin:0 auto}main{padding:40px 0 56px}footer{padding:22px 0 36px;border-top:1px solid var(--line);color:var(--muted)}h1{margin:6px 0 12px;font-size:2.35rem;line-height:1.08;letter-spacing:0}h2{margin:40px 0 10px;font-size:1.4rem;letter-spacing:0}p{max-width:900px}.eyebrow,.note{color:var(--muted)}.lede{font-size:1.08rem;color:#35424c}.result{margin:24px 0;padding:16px 18px;border-left:4px solid var(--candidate);background:#edf8f3;max-width:980px}.stats{display:grid;grid-template-columns:repeat(4,minmax(0,1fr));border:1px solid var(--line);background:var(--line);gap:1px;margin:24px 0}.stat{background:#fff;padding:15px;min-height:92px}.stat strong{display:block;font-size:1.55rem}.stat span{display:block;color:var(--muted);margin-top:5px}.table-wrap{overflow-x:auto;border:1px solid var(--line)}table{width:100%;border-collapse:collapse;min-width:1050px}th,td{padding:9px 10px;border-bottom:1px solid var(--line);text-align:right;white-space:nowrap;font-size:13px}th{background:var(--soft);font-size:12px}th:first-child,td:first-child{text-align:left}tr:last-child td{border-bottom:0}code{font:0.92em ui-monospace,SFMono-Regular,Consolas,monospace}.method{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:22px;max-width:980px}.method section{border-top:2px solid var(--line);padding-top:10px}.method h3{margin:0 0 5px;font-size:1rem}.method p{margin:0;color:var(--muted)}a{color:var(--link)}@media(max-width:760px){main,footer{width:min(100% - 24px,1240px)}main{padding-top:24px}h1{font-size:1.85rem}.stats{grid-template-columns:repeat(2,minmax(0,1fr))}.method{grid-template-columns:1fr}}
</style></head><body><main>",
    );
    out.push_str(&format!(
        "<div class=\"eyebrow\">Matched leader-paced A/B benchmark</div><h1>{}</h1><p class=\"lede\">For each eligible leader run, the benchmark submitted one transaction through <code>{}</code> and one through <code>{}</code>. Every performance delta below requires both transactions to be observed in the same Solana slot.</p>",
        html_escape(title), candidate, baseline
    ));
    out.push_str(&format!(
        "<div class=\"result\"><strong>PoH placement:</strong> across {} complete same-slot pairs, {} was earlier {}, both were in the same interval {}, and {} was earlier {}. PoH placement describes produced-block order, not packet arrival at the leader TPU.</div>",
        poh.pairs, candidate, poh.candidate_earlier, poh.same, baseline, poh.baseline_earlier
    ));
    let coverage = &analysis.landing_coverage;
    out.push_str(&format!(
        "<div class=\"stats\"><div class=\"stat\"><strong>{}</strong><span>paired leader runs</span></div><div class=\"stat\"><strong>{}</strong><span>same-slot pairs</span></div><div class=\"stat\"><strong>{}</strong><span>eligible leader identities</span></div><div class=\"stat\"><strong>{:.2}%</strong><span>eligible scheduled slots</span></div></div>",
        coverage.comparison_groups,
        coverage.same_slot_pairs,
        coverage.distinct_leader_identities,
        analysis.coverage.eligible_scheduled_slot_pct,
    ));

    out.push_str("<h2>Overall same-slot comparison</h2><p>Each delta is candidate minus baseline. Negative values favor the candidate. Percentiles select observed values without interpolation.</p>");
    out.push_str(&metrics_table(metrics, &candidate, &baseline));
    out.push_str(&format!(
        "<p class=\"note\">Coverage outside the comparison population: {} different-slot pairs, {} baseline-only, {} candidate-only, and {} observed through neither route. These outcomes are reported as coverage and are never converted into timing or placement deltas.</p>",
        coverage.different_slot_pairs,
        coverage.baseline_only,
        coverage.candidate_only,
        coverage.neither_observed,
    ));

    cohort_section(
        &mut out,
        "By leader city",
        "Cities and countries come from the getLeaderSlots snapshots saved during the run.",
        &analysis.by_city,
        &candidate,
        &baseline,
        false,
    );
    cohort_section(
        &mut out,
        "By distance from sender",
        "Distance is calculated from the saved validator coordinates to the sender coordinates supplied to the report command. Missing coordinates remain an explicit unavailable cohort.",
        &analysis.by_distance,
        &candidate,
        &baseline,
        false,
    );
    cohort_section(
        &mut out,
        "By validator identity",
        "Each validator remains separate. Repeated leader runs from one validator are not independent validator samples; small cohorts are descriptive only.",
        &analysis.by_validator,
        &candidate,
        &baseline,
        true,
    );

    out.push_str("<h2>Methodology</h2><div class=\"method\">");
    for (heading, text) in [
        ("Paired design", "Both transactions share the leader run, slot trigger, sender host, transaction shape, and fee configuration. Only the network route differs."),
        ("Same-slot rule", "First-observation time, processed transaction index, entry index, and PoH interval are compared only when both signatures land in the same slot."),
        ("Observation clock", "Each signature uses its earliest event across the configured RPCEdge receivers. Deshred and processed receivers are not compared with each other."),
        ("Block placement", "Processed transaction index is canonical transaction order. Entry index and preceding PoH ticks are derived after collection from the saved contiguous entry history."),
        ("Interpretation", "Observation time includes sender scheduling, network transit, leader admission, block production, and propagation to the Frankfurt observer. It is not network RTT or TPU packet-arrival latency."),
        ("Geography", "City, country, latitude, and longitude are historical values saved in getLeaderSlots snapshots. The coarse region classifier is not used."),
    ] {
        out.push_str(&format!("<section><h3>{}</h3><p>{}</p></section>", heading, text));
    }
    out.push_str("</div><p class=\"note\">The report does not claim that repeated transactions from one validator are independent samples or that geographic distance alone caused an observed difference.</p>");
    out.push_str(&format!(
        "</main><footer>Generated {} · <a href=\"summary.json\">summary.json</a> · <a href=\"paired-rows.json\">paired-rows.json</a></footer></body></html>",
        html_escape(&generated_at.to_rfc3339_opts(SecondsFormat::Millis, true))
    ));
    out
}

fn metrics_table(metrics: &ComparisonMetrics, candidate: &str, baseline: &str) -> String {
    let mut out = format!(
        "<div class=\"table-wrap\"><table><thead><tr><th>Measure</th><th>Pairs</th><th>{} earlier</th><th>Same</th><th>{} earlier</th><th>Delta p50</th><th>Delta p95</th><th>Delta p99</th></tr></thead><tbody>",
        candidate, baseline
    );
    for (label, metric, unit) in metric_rows(metrics) {
        out.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            label,
            metric.pairs,
            metric.candidate_earlier,
            metric.same,
            metric.baseline_earlier,
            format_value(metric.delta_p50, unit),
            format_value(metric.delta_p95, unit),
            format_value(metric.delta_p99, unit),
        ));
    }
    out.push_str("</tbody></table></div>");
    out
}

fn cohort_section(
    out: &mut String,
    heading: &str,
    description: &str,
    cohorts: &[CohortSummary],
    candidate: &str,
    baseline: &str,
    show_identity: bool,
) {
    out.push_str(&format!("<h2>{}</h2><p>{}</p>", heading, description));
    if cohorts.is_empty() {
        out.push_str("<p class=\"note\">No cohort data was available.</p>");
        return;
    }
    out.push_str(&format!(
        "<div class=\"table-wrap\"><table><thead><tr><th>Cohort</th><th>Validators</th><th>Same-slot n</th><th>Observation {}/{}/{}</th><th>Observation p50 / p95 / p99</th><th>Tx index {}/{}/{}</th><th>Tx index p50 / p95 / p99</th><th>Entry {}/{}/{}</th><th>Entry p50 / p95 / p99</th><th>PoH {}/{}/{}</th><th>PoH p50 / p95 / p99</th></tr></thead><tbody>",
        candidate, "same", baseline, candidate, "same", baseline, candidate, "same", baseline, candidate, "same", baseline
    ));
    for cohort in cohorts {
        let label = if show_identity {
            let location = match (&cohort.city, &cohort.country) {
                (Some(city), Some(country)) => format!("{city}, {country}"),
                (Some(city), None) => city.clone(),
                _ => "Location unavailable".to_string(),
            };
            format!(
                "<code title=\"{}\">{}</code><br><span class=\"note\">{}</span>",
                html_escape(cohort.identity.as_deref().unwrap_or(&cohort.label)),
                html_escape(&short_identity(
                    cohort.identity.as_deref().unwrap_or(&cohort.label)
                )),
                html_escape(&location)
            )
        } else {
            html_escape(&cohort.label)
        };
        let metrics = &cohort.metrics;
        out.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td>{}{}{}{}</tr>",
            label,
            cohort.validators,
            metrics.first_observed_ms.pairs,
            cohort_metric_cells(&metrics.first_observed_ms, " ms"),
            cohort_metric_cells(&metrics.processed_transaction_index, ""),
            cohort_metric_cells(&metrics.entry_index, ""),
            cohort_metric_cells(&metrics.preceding_poh_ticks, ""),
        ));
    }
    out.push_str("</tbody></table></div><p class=\"note\">Outcome triplets are candidate earlier / same / baseline earlier. Every percentile uses only complete same-slot values for that measure.</p>");
}

fn cohort_metric_cells(metric: &MetricSummary, unit: &str) -> String {
    format!(
        "<td>{} / {} / {}</td><td>{} / {} / {}</td>",
        metric.candidate_earlier,
        metric.same,
        metric.baseline_earlier,
        format_value(metric.delta_p50, unit),
        format_value(metric.delta_p95, unit),
        format_value(metric.delta_p99, unit),
    )
}

fn metric_rows(metrics: &ComparisonMetrics) -> [(&'static str, &MetricSummary, &'static str); 4] {
    [
        ("First observation time", &metrics.first_observed_ms, " ms"),
        (
            "Processed transaction index",
            &metrics.processed_transaction_index,
            "",
        ),
        ("Entry index", &metrics.entry_index, ""),
        ("Preceding PoH ticks", &metrics.preceding_poh_ticks, ""),
    ]
}

fn format_value(value: Option<f64>, unit: &str) -> String {
    value
        .map(|value| {
            if unit.is_empty() {
                format!("{value:.0}")
            } else {
                format!("{value:.3}{unit}")
            }
        })
        .unwrap_or_else(|| "-".to_string())
}

fn short_identity(identity: &str) -> String {
    if identity.chars().count() <= 16 {
        identity.to_string()
    } else {
        format!("{}...", identity.chars().take(12).collect::<String>())
    }
}

fn arm_label(arm: &str) -> String {
    match arm {
        "doublezero" => "DoubleZero".to_string(),
        "public_internet" => "Public Internet".to_string(),
        _ => arm.replace('_', " "),
    }
}

fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
