use anyhow::Result;
use serde::Serialize;
use crate::session::types::ClientReport;

#[derive(Serialize)]
struct JsonReport<'a> {
    client: &'a str,
    period_start: String,
    period_end: String,
    total_minutes: u32,
    total_hours: f64,
    total_commits: usize,
    hourly_rate: f64,
    currency: &'a str,
    billable_amount: f64,
    days: Vec<JsonDay<'a>>,
}

#[derive(Serialize)]
struct JsonDay<'a> {
    date: String,
    sessions: usize,
    total_minutes: u32,
    total_hours: f64,
    total_commits: usize,
    repos: &'a [String],
    amount: f64,
}

pub fn serialize_json(report: &ClientReport) -> Result<String> {
    let days = report
        .days
        .iter()
        .map(|day| {
            let hours = day.total_hours();
            let amount = if report.hourly_rate > 0.0 {
                hours * report.hourly_rate
            } else {
                0.0
            };
            JsonDay {
                date: day.date.format("%Y-%m-%d").to_string(),
                sessions: day.sessions.len(),
                total_minutes: day.total_minutes,
                total_hours: (hours * 100.0).round() / 100.0,
                total_commits: day.total_commits,
                repos: &day.repos,
                amount: (amount * 100.0).round() / 100.0,
            }
        })
        .collect();

    let json_report = JsonReport {
        client: &report.client_name,
        period_start: report.period_start.format("%Y-%m-%d").to_string(),
        period_end: report.period_end.format("%Y-%m-%d").to_string(),
        total_minutes: report.total_minutes,
        total_hours: (report.total_hours() * 100.0).round() / 100.0,
        total_commits: report.total_commits,
        hourly_rate: report.hourly_rate,
        currency: &report.currency,
        billable_amount: (report.billable_amount() * 100.0).round() / 100.0,
        days,
    };

    Ok(serde_json::to_string_pretty(&json_report)?)
}
