use anyhow::Result;
use chrono::Local;
use crate::commands::report::{run as run_report, ReportArgs};

pub struct ExportArgs {
    pub client: Option<String>,
    pub last_week: bool,
    pub last_month: bool,
    pub since: Option<String>,
    pub until: Option<String>,
    pub format: String,
    pub output: Option<String>,
}

pub fn run(args: ExportArgs) -> Result<()> {
    // Auto-generate output filename if not provided
    let output = if args.output.is_none() {
        let safe_client = args
            .client
            .as_deref()
            .unwrap_or("all")
            .to_lowercase()
            .replace(' ', "-")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .collect::<String>();
        Some(format!(
            "gtt-{}-{}.{}",
            safe_client,
            Local::now().format("%Y-%m"),
            args.format
        ))
    } else {
        args.output
    };

    run_report(ReportArgs {
        client: args.client,
        last_week: args.last_week,
        last_month: args.last_month,
        since: args.since,
        until: args.until,
        format: args.format,
        output,
        verify_mode: false,
    })
}
