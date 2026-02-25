use chrono::{DateTime, FixedOffset, NaiveDate};
use crate::git::Commit;

#[derive(Debug, Clone)]
pub struct Session {
    pub start: DateTime<FixedOffset>,
    pub end: DateTime<FixedOffset>,
    pub duration_minutes: u32,
    pub commits: Vec<Commit>,
    pub repos: Vec<String>,
    pub lines_added: u32,
    pub lines_deleted: u32,
}

impl Session {
    #[allow(dead_code)]
    pub fn duration_hours(&self) -> f64 {
        self.duration_minutes as f64 / 60.0
    }

    pub fn date(&self) -> NaiveDate {
        self.start.date_naive()
    }
}

#[derive(Debug, Clone)]
pub struct DayReport {
    pub date: NaiveDate,
    pub sessions: Vec<Session>,
    pub total_minutes: u32,
    pub total_commits: usize,
    pub repos: Vec<String>,
    pub total_lines_added: u32,
    pub total_lines_deleted: u32,
}

impl DayReport {
    pub fn total_hours(&self) -> f64 {
        self.total_minutes as f64 / 60.0
    }
}

#[derive(Debug, Clone)]
pub struct ClientReport {
    pub client_name: String,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub days: Vec<DayReport>,
    pub total_minutes: u32,
    pub total_commits: usize,
    pub hourly_rate: f64,
    pub currency: String,
    pub total_lines_added: u32,
    pub total_lines_deleted: u32,
    /// Cost per 1000 lines changed to offset LLM usage (0.0 = disabled)
    pub llm_cost_per_kloc: f64,
    /// Total LLM cost for the period
    pub llm_cost: f64,
}

impl ClientReport {
    pub fn total_hours(&self) -> f64 {
        self.total_minutes as f64 / 60.0
    }

    pub fn billable_amount(&self) -> f64 {
        self.total_hours() * self.hourly_rate
    }

    pub fn total_amount(&self) -> f64 {
        self.billable_amount() + self.llm_cost
    }
}
