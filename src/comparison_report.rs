use crate::compare::{ComparisonOutput, RunComparisonSummary};
use anyhow::{Context, Result};
use chrono::SecondsFormat;
use std::{fs, path::Path};

pub(crate) fn write_comparison_report(output_dir: &Path, output: &ComparisonOutput) -> Result<()> {
    fs::write(
        output_dir.join("comparison.json"),
        serde_json::to_vec_pretty(output)?,
    )
    .with_context(|| format!("write {}", output_dir.join("comparison.json").display()))?;
    fs::write(output_dir.join("comparison.md"), markdown(output))
        .with_context(|| format!("write {}", output_dir.join("comparison.md").display()))?;
    let html = html(output);
    fs::write(output_dir.join("comparison.html"), &html)
        .with_context(|| format!("write {}", output_dir.join("comparison.html").display()))?;
    fs::write(output_dir.join("index.html"), html)
        .with_context(|| format!("write {}", output_dir.join("index.html").display()))
}

fn markdown(output: &ComparisonOutput) -> String {
    let mut out = format!(
        "# {}\n\n- Generated: `{}`\n- Landing clock: earliest configured receiver\n- Runs: `{}`\n\n## Methodology\n\n{}",
        output.title,
        output
            .generated_at
            .to_rfc3339_opts(SecondsFormat::Millis, true),
        output.run_count,
        methodology(),
    );
    for warning in &output.warnings {
        out.push_str(&format!("\n\n> **Warning:** {warning}"));
    }
    out.push_str("\n\n## Comparison\n\n| Run | Routes | Avg observed ms | P90 observed ms | Avg landed slots | P90 landed slots | Avg block index | P90 block index | Same-slot | Landed | Total | Success % | Score % |\n| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |\n");
    for run in &output.runs {
        out.push_str(&format!(
            "| `{}` | `{}` | {} | {} | {} | {} | {} | {} | {} | {} | {} | {:.2} | {:.2} |\n",
            run.label,
            run.routes,
            number(run.avg_landed_ms),
            number(run.p90_landed_ms),
            number(run.avg_landed_slots),
            number(run.p90_landed_slots),
            number(run.avg_idx_in_block),
            number(run.p90_idx_in_block),
            run.same_slot_landed,
            run.landed_runs,
            run.total_runs,
            run.success_ratio_pct,
            run.performance_rate_pct,
        ));
    }
    out
}

fn html(output: &ComparisonOutput) -> String {
    let mut out = format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>{}</title><style>*{{box-sizing:border-box}}body{{margin:0;color:#17202a;font:15px/1.5 system-ui,sans-serif;letter-spacing:0}}main{{width:min(1180px,calc(100% - 32px));margin:0 auto;padding:36px 0}}h1{{font-size:2rem;letter-spacing:0}}h2{{margin-top:36px;letter-spacing:0}}.note{{max-width:940px;color:#586672}}.warning{{border-left:4px solid #b54708;background:#fff6ed;padding:10px 12px}}.table-wrap{{overflow-x:auto;border:1px solid #d9e0e5}}table{{width:100%;border-collapse:collapse;min-width:960px}}th,td{{padding:9px;border-bottom:1px solid #d9e0e5;text-align:right;white-space:nowrap}}th:first-child,td:first-child,td.left{{text-align:left}}th{{background:#f4f7f8;font-size:12px}}code{{font-family:ui-monospace,monospace}}</style></head><body><main><div class=\"note\">Transaction landing comparison</div><h1>{}</h1><p class=\"note\">Generated {}. Landing clock: earliest configured receiver.</p><h2>Methodology</h2><p class=\"note\">{}</p>",
        escape(&output.title),
        escape(&output.title),
        escape(&output.generated_at.to_rfc3339_opts(SecondsFormat::Millis, true)),
        escape(methodology()),
    );
    for warning in &output.warnings {
        out.push_str(&format!(
            "<p class=\"warning\"><strong>Warning:</strong> {}</p>",
            escape(warning)
        ));
    }
    out.push_str("<h2>Comparison</h2><div class=\"table-wrap\"><table><thead><tr><th>Run</th><th>Routes</th><th>Avg observed ms</th><th>P90 observed ms</th><th>Avg landed slots</th><th>P90 landed slots</th><th>Avg block index</th><th>P90 block index</th><th>Same-slot</th><th>Landed</th><th>Total</th><th>Success %</th><th>Score %</th></tr></thead><tbody>");
    for run in &output.runs {
        out.push_str(&run_row(run));
    }
    out.push_str("</tbody></table></div></main></body></html>");
    out
}

fn run_row(run: &RunComparisonSummary) -> String {
    format!(
        "<tr><td class=\"left\"><code>{}</code></td><td class=\"left\"><code>{}</code></td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{:.2}</td><td>{:.2}</td></tr>",
        escape(&run.label),
        escape(&run.routes),
        number(run.avg_landed_ms),
        number(run.p90_landed_ms),
        number(run.avg_landed_slots),
        number(run.p90_landed_slots),
        number(run.avg_idx_in_block),
        number(run.p90_idx_in_block),
        run.same_slot_landed,
        run.landed_runs,
        run.total_runs,
        run.success_ratio_pct,
        run.performance_rate_pct,
    )
}

fn methodology() -> &'static str {
    "Landing proof comes from matched observations by transaction signature, not provider ACK. The default timing value is the earliest configured receiver event per transaction. Processed metadata supplies block position. Scores are descriptive unless transaction shape, trigger, sender, fees, and observation configuration match."
}

fn number(value: Option<f64>) -> String {
    value
        .map(|value| format!("{value:.3}"))
        .unwrap_or_else(|| "-".to_string())
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
