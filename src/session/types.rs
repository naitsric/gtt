use chrono::{DateTime, FixedOffset, NaiveDate};
use crate::git::Commit;

#[derive(Debug, Clone)]
pub struct Session {
    pub start: DateTime<FixedOffset>,
    pub end: DateTime<FixedOffset>,
    pub duration_minutes: u32,
    pub commits: Vec<Commit>,
    pub repos: Vec<String>,
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
}

impl ClientReport {
    pub fn total_hours(&self) -> f64 {
        self.total_minutes as f64 / 60.0
    }

    pub fn billable_amount(&self) -> f64 {
        self.total_hours() * self.hourly_rate
    }
}
