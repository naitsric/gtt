use anyhow::Result;
use crate::commands::report::{run as run_report, ReportArgs};

pub struct VerifyArgs {
    pub client: Option<String>,
    pub last_week: bool,
    pub last_month: bool,
    pub since: Option<String>,
    pub until: Option<String>,
}

pub fn run(args: VerifyArgs) -> Result<()> {
    run_report(ReportArgs {
        client: args.client,
        last_week: args.last_week,
        last_month: args.last_month,
        since: args.since,
        until: args.until,
        format: "table".to_string(),
        output: None,
        verify_mode: true,
    })
}
