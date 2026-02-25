use anyhow::Result;
use chrono::{Datelike, Local};
use colored::Colorize;
use std::path::Path;
use crate::config::load_config;
use crate::git::{get_repo_user_email, parse_git_log, run_git_log};
use crate::output::format_duration;
use crate::session::analyze;

pub fn run() -> Result<()> {
    let config = load_config()?;
    let today = Local::now().date_naive();

    // Week range: Monday to today
    let days_since_monday = today.weekday().num_days_from_monday();
    let week_start = today - chrono::Duration::days(days_since_monday as i64);

    println!();
    println!("{}", "gtt status".bold());
    println!("{}", format!("  Hoy: {}    Esta semana: {} — {}",
        today.format("%d/%m/%Y"),
        week_start.format("%d/%m/%Y"),
        today.format("%d/%m/%Y")
    ).dimmed());
    println!();

    if config.client.is_empty() {
        println!("{}", "No hay clientes configurados. Ejecuta `gtt init`.".yellow());
        return Ok(());
    }

    for (client_name, client_cfg) in &config.client {
        let mut today_minutes = 0u32;
        let mut week_minutes = 0u32;
        let mut today_commits = 0usize;
        let mut week_commits = 0usize;

        for repo_path in &client_cfg.repos {
            let path = Path::new(repo_path);
            if !path.exists() {
                continue;
            }
            let author_email = get_repo_user_email(path);

            // Today's commits
            if let Ok(raw) = run_git_log(path, Some(today), Some(today), author_email.as_deref()) {
                if let Ok(commits) = parse_git_log(&raw, path, &config.settings.bot_authors) {
                    let sessions = analyze(commits, &config.settings);
                    today_minutes += sessions.iter().map(|s| s.duration_minutes).sum::<u32>();
                    today_commits += sessions.iter().map(|s| s.commits.len()).sum::<usize>();
                }
            }

            // This week's commits
            if let Ok(raw) = run_git_log(path, Some(week_start), Some(today), author_email.as_deref()) {
                if let Ok(commits) = parse_git_log(&raw, path, &config.settings.bot_authors) {
                    let sessions = analyze(commits, &config.settings);
                    week_minutes += sessions.iter().map(|s| s.duration_minutes).sum::<u32>();
                    week_commits += sessions.iter().map(|s| s.commits.len()).sum::<usize>();
                }
            }
        }

        println!(
            "  {} — Hoy: {}  ({} commits)   Esta semana: {}  ({} commits)",
            client_name.bold(),
            if today_minutes > 0 {
                format_duration(today_minutes).green().to_string()
            } else {
                "0m".dimmed().to_string()
            },
            today_commits,
            if week_minutes > 0 {
                format_duration(week_minutes).green().bold().to_string()
            } else {
                "0m".dimmed().to_string()
            },
            week_commits
        );
    }

    println!();
    Ok(())
}
