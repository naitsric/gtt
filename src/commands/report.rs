use anyhow::{bail, Result};
use chrono::{Datelike, Local, NaiveDate};
use colored::Colorize;
use std::path::Path;
use crate::config::{load_config, ClientConfig};
use crate::errors::GttError;
use crate::git::{get_repo_user_email, merge_numstat, parse_git_log, parse_numstat, run_git_log, run_git_log_numstat};
use crate::output::{print_client_report, print_verify_report};
use crate::output::csv::serialize_csv;
use crate::output::json_fmt::serialize_json;
use crate::session::{analyze, group_by_day};
use crate::session::types::ClientReport;

pub struct ReportArgs {
    pub client: Option<String>,
    pub last_week: bool,
    pub last_month: bool,
    pub since: Option<String>,
    pub until: Option<String>,
    pub format: String,
    pub output: Option<String>,
    pub verify_mode: bool,
}

pub fn run(args: ReportArgs) -> Result<()> {
    let config = load_config()?;

    let (since, until) = resolve_date_range(&args)?;

    let clients: Vec<(String, &ClientConfig)> = if let Some(ref name) = args.client {
        let cfg = config
            .client
            .get(name)
            .ok_or_else(|| GttError::ClientNotFound(name.clone()))?;
        vec![(name.clone(), cfg)]
    } else {
        config.client.iter().map(|(k, v)| (k.clone(), v)).collect()
    };

    if clients.is_empty() {
        bail!("No hay clientes configurados. Ejecuta `gtt init` para comenzar.");
    }

    for (client_name, client_cfg) in clients {
        let report = build_client_report(&client_name, client_cfg, since, until, &config.settings.bot_authors)?;

        if report.days.is_empty() {
            println!(
                "{}",
                format!(
                    "Sin commits para '{}' entre {} y {}.",
                    client_name,
                    since.format("%d/%m/%Y"),
                    until.format("%d/%m/%Y"),
                )
                .yellow()
            );
            continue;
        }

        match args.format.as_str() {
            "table" => {
                if args.verify_mode {
                    print_verify_report(&report);
                } else {
                    print_client_report(&report);
                }
            }
            "csv" => {
                let data = serialize_csv(&report)?;
                output_data(&data, &args.output, &client_name, "csv")?;
            }
            "json" => {
                let data = serialize_json(&report)?;
                output_data(&data, &args.output, &client_name, "json")?;
            }
            other => bail!("Formato no soportado: '{}'. Usa: table, csv, json", other),
        }
    }

    Ok(())
}

fn build_client_report(
    client_name: &str,
    client_cfg: &ClientConfig,
    since: NaiveDate,
    until: NaiveDate,
    bot_authors: &[String],
) -> Result<ClientReport> {
    let mut all_commits = Vec::new();

    for repo_path in &client_cfg.repos {
        let path = Path::new(repo_path);
        if !path.exists() {
            eprintln!(
                "{}",
                format!(
                    "Advertencia: repositorio no encontrado: {}",
                    path.display()
                )
                .yellow()
            );
            continue;
        }

        // Filter by the repo's configured author email if possible
        let author_email = get_repo_user_email(path);

        let raw = match run_git_log(path, Some(since), Some(until), author_email.as_deref()) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("{}", format!("Advertencia: error en {}: {}", path.display(), e).yellow());
                continue;
            }
        };

        let mut commits = parse_git_log(&raw, path, bot_authors)?;

        // Fetch and merge numstat (lines added/deleted per commit)
        if let Ok(numstat_raw) = run_git_log_numstat(path, Some(since), Some(until), author_email.as_deref()) {
            let numstat_map = parse_numstat(&numstat_raw);
            merge_numstat(&mut commits, &numstat_map);
        }

        all_commits.append(&mut commits);
    }

    // Load actual settings from config
    let config = load_config().unwrap_or_default();
    let settings = config.settings;

    let sessions = analyze(all_commits, &settings);
    let days = group_by_day(sessions);

    let total_minutes = days.iter().map(|d| d.total_minutes).sum();
    let total_commits = days.iter().map(|d| d.total_commits).sum();
    let total_lines_added = days.iter().map(|d| d.total_lines_added).sum();
    let total_lines_deleted = days.iter().map(|d| d.total_lines_deleted).sum();

    let total_lines = (total_lines_added + total_lines_deleted) as f64;
    let llm_cost = (total_lines / 1000.0) * client_cfg.llm_cost_per_kloc;

    Ok(ClientReport {
        client_name: client_name.to_string(),
        period_start: since,
        period_end: until,
        days,
        total_minutes,
        total_commits,
        hourly_rate: client_cfg.hourly_rate,
        currency: client_cfg.currency.clone(),
        total_lines_added,
        total_lines_deleted,
        llm_cost_per_kloc: client_cfg.llm_cost_per_kloc,
        llm_cost,
    })
}

fn resolve_date_range(args: &ReportArgs) -> Result<(NaiveDate, NaiveDate)> {
    let today = Local::now().date_naive();

    if args.last_week {
        let days_since_monday = today.weekday().num_days_from_monday();
        let this_monday = today - chrono::Duration::days(days_since_monday as i64);
        let last_monday = this_monday - chrono::Duration::weeks(1);
        let last_sunday = last_monday + chrono::Duration::days(6);
        return Ok((last_monday, last_sunday));
    }

    if args.last_month {
        let year = if today.month() == 1 { today.year() - 1 } else { today.year() };
        let month = if today.month() == 1 { 12 } else { today.month() - 1 };
        let first = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
        let last = NaiveDate::from_ymd_opt(
            if month == 12 { year + 1 } else { year },
            if month == 12 { 1 } else { month + 1 },
            1,
        )
        .unwrap()
        .pred_opt()
        .unwrap();
        return Ok((first, last));
    }

    let since = if let Some(ref s) = args.since {
        NaiveDate::parse_from_str(s, "%Y-%m-%d")
            .map_err(|_| GttError::DateParse(format!("Fecha inválida: '{}'. Usa YYYY-MM-DD.", s)))?
    } else {
        // Default: current month
        NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap()
    };

    let until = if let Some(ref u) = args.until {
        NaiveDate::parse_from_str(u, "%Y-%m-%d")
            .map_err(|_| GttError::DateParse(format!("Fecha inválida: '{}'. Usa YYYY-MM-DD.", u)))?
    } else {
        today
    };

    if since > until {
        return Err(GttError::InvalidDateRange(
            format!("{} es posterior a {}", since, until)
        ).into());
    }

    Ok((since, until))
}

fn output_data(data: &str, output: &Option<String>, _client_name: &str, _ext: &str) -> Result<()> {
    if let Some(path) = output {
        std::fs::write(path, data)?;
        println!("{}", format!("Guardado en: {}", path).green());
    } else {
        print!("{}", data);
    }
    Ok(())
}
