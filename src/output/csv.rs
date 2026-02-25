use anyhow::Result;
use crate::session::types::ClientReport;

pub fn serialize_csv(report: &ClientReport) -> Result<String> {
    let mut wtr = csv::Writer::from_writer(vec![]);

    wtr.write_record(["date", "sessions", "hours", "minutes", "commits", "repos", "amount", "llm_cost", "total_amount", "currency", "lines_added", "lines_deleted"])?;

    for day in &report.days {
        let repos = day.repos.join("|");
        let hours = day.total_hours();
        let amount = if report.hourly_rate > 0.0 {
            hours * report.hourly_rate
        } else {
            0.0
        };
        let day_lines = (day.total_lines_added + day.total_lines_deleted) as f64;
        let day_llm = (day_lines / 1000.0) * report.llm_cost_per_kloc;

        wtr.write_record([
            &day.date.format("%Y-%m-%d").to_string(),
            &day.sessions.len().to_string(),
            &format!("{:.4}", hours),
            &day.total_minutes.to_string(),
            &day.total_commits.to_string(),
            &repos,
            &format!("{:.2}", amount),
            &format!("{:.2}", day_llm),
            &format!("{:.2}", amount + day_llm),
            &report.currency,
            &day.total_lines_added.to_string(),
            &day.total_lines_deleted.to_string(),
        ])?;
    }

    let data = String::from_utf8(wtr.into_inner()?)?;
    Ok(data)
}
