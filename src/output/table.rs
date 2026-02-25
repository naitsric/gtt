use comfy_table::{Cell, CellAlignment, Color, ContentArrangement, Table};
use colored::Colorize;
use crate::session::types::ClientReport;

pub fn print_client_report(report: &ClientReport) {
    println!();
    println!("{}", format!("Cliente: {}", report.client_name).bold());
    println!(
        "{}",
        format!(
            "Periodo: {} — {}",
            report.period_start.format("%d/%m/%Y"),
            report.period_end.format("%d/%m/%Y")
        )
        .dimmed()
    );
    println!();

    let mut table = Table::new();
    table
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Fecha").fg(Color::Cyan),
            Cell::new("Sesiones").fg(Color::Cyan),
            Cell::new("Horas").fg(Color::Cyan),
            Cell::new("Commits").fg(Color::Cyan),
            Cell::new("Repos").fg(Color::Cyan),
        ]);

    for day in &report.days {
        table.add_row(vec![
            Cell::new(day.date.format("%a %d/%m").to_string()),
            Cell::new(day.sessions.len().to_string()).set_alignment(CellAlignment::Right),
            Cell::new(format_duration(day.total_minutes)).set_alignment(CellAlignment::Right),
            Cell::new(day.total_commits.to_string()).set_alignment(CellAlignment::Right),
            Cell::new(day.repos.join(", ")),
        ]);
    }

    // Totals row
    let total_sessions: usize = report.days.iter().map(|d| d.sessions.len()).sum();
    table.add_row(vec![
        Cell::new("Total").fg(Color::Yellow),
        Cell::new(total_sessions.to_string())
            .set_alignment(CellAlignment::Right)
            .fg(Color::Yellow),
        Cell::new(format_duration(report.total_minutes))
            .set_alignment(CellAlignment::Right)
            .fg(Color::Yellow),
        Cell::new(report.total_commits.to_string())
            .set_alignment(CellAlignment::Right)
            .fg(Color::Yellow),
        Cell::new("").fg(Color::Yellow),
    ]);

    println!("{table}");
    println!();

    if report.hourly_rate > 0.0 {
        println!(
            "{}",
            format!(
                "Monto: {:.2}h × {}/h = {:.2} {}",
                report.total_hours(),
                report.hourly_rate,
                report.billable_amount(),
                report.currency
            )
            .green()
            .bold()
        );
    } else {
        println!(
            "{}",
            format!("Total: {} (sin tasa horaria configurada)", format_duration(report.total_minutes)).yellow()
        );
    }
    println!();
}

pub fn print_verify_report(report: &ClientReport) {
    println!();
    println!("{}", format!("Verificar sesiones: {}", report.client_name).bold());
    println!(
        "{}",
        format!(
            "Periodo: {} — {}",
            report.period_start.format("%d/%m/%Y"),
            report.period_end.format("%d/%m/%Y")
        )
        .dimmed()
    );
    println!();

    for day in &report.days {
        println!(
            "{}",
            format!("── {} ({} sesiones, {}) ──",
                day.date.format("%A %d/%m/%Y"),
                day.sessions.len(),
                format_duration(day.total_minutes)
            ).cyan().bold()
        );

        for (i, session) in day.sessions.iter().enumerate() {
            println!(
                "  Sesión {}:  {} → {}  ({}, {} commits)",
                i + 1,
                session.start.format("%H:%M"),
                session.end.format("%H:%M"),
                format_duration(session.duration_minutes),
                session.commits.len()
            );
            for commit in &session.commits {
                println!(
                    "    {} {} {}",
                    commit.author_date.format("%H:%M").to_string().dimmed(),
                    &commit.hash[..7].yellow(),
                    commit.subject
                );
            }
        }
        println!();
    }
}


pub fn format_duration(minutes: u32) -> String {
    let hours = minutes / 60;
    let mins = minutes % 60;
    if hours == 0 {
        format!("{}m", mins)
    } else if mins == 0 {
        format!("{}h", hours)
    } else {
        format!("{}h {}m", hours, mins)
    }
}
