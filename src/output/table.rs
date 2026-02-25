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

    let show_llm = report.llm_cost_per_kloc > 0.0;

    let mut table = Table::new();
    let mut header = vec![
        Cell::new("Fecha").fg(Color::Cyan),
        Cell::new("Sesiones").fg(Color::Cyan),
        Cell::new("Horas").fg(Color::Cyan),
        Cell::new("Commits").fg(Color::Cyan),
        Cell::new("+/-").fg(Color::Cyan),
    ];
    if show_llm {
        header.push(Cell::new("LLM").fg(Color::Cyan));
    }
    header.push(Cell::new("Repos").fg(Color::Cyan));
    table
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(header);

    for day in &report.days {
        let day_lines = (day.total_lines_added + day.total_lines_deleted) as f64;
        let day_llm = (day_lines / 1000.0) * report.llm_cost_per_kloc;

        let mut row = vec![
            Cell::new(day.date.format("%a %d/%m").to_string()),
            Cell::new(day.sessions.len().to_string()).set_alignment(CellAlignment::Right),
            Cell::new(format_duration(day.total_minutes)).set_alignment(CellAlignment::Right),
            Cell::new(day.total_commits.to_string()).set_alignment(CellAlignment::Right),
            Cell::new(format!("+{} -{}", day.total_lines_added, day.total_lines_deleted))
                .set_alignment(CellAlignment::Right),
        ];
        if show_llm {
            row.push(Cell::new(format!("{:.2}", day_llm)).set_alignment(CellAlignment::Right));
        }
        row.push(Cell::new(day.repos.join(", ")));
        table.add_row(row);
    }

    // Totals row
    let total_sessions: usize = report.days.iter().map(|d| d.sessions.len()).sum();
    let mut total_row = vec![
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
        Cell::new(format!("+{} -{}", report.total_lines_added, report.total_lines_deleted))
            .set_alignment(CellAlignment::Right)
            .fg(Color::Yellow),
    ];
    if show_llm {
        total_row.push(
            Cell::new(format!("{:.2}", report.llm_cost))
                .set_alignment(CellAlignment::Right)
                .fg(Color::Yellow),
        );
    }
    total_row.push(Cell::new("").fg(Color::Yellow));
    table.add_row(total_row);

    println!("{table}");
    println!();

    if report.hourly_rate > 0.0 {
        if show_llm {
            println!(
                "{}",
                format!(
                    "Monto: {:.2}h × {}/h = {:.2} {}  +  LLM: {:.2} {}  =  Total: {:.2} {}",
                    report.total_hours(),
                    report.hourly_rate,
                    report.billable_amount(),
                    report.currency,
                    report.llm_cost,
                    report.currency,
                    report.total_amount(),
                    report.currency,
                )
                .green()
                .bold()
            );
        } else {
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
        }
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
                "  Sesión {}:  {} → {}  ({}, {} commits, +{} -{})",
                i + 1,
                session.start.format("%H:%M"),
                session.end.format("%H:%M"),
                format_duration(session.duration_minutes),
                session.commits.len(),
                session.lines_added,
                session.lines_deleted
            );
            for commit in &session.commits {
                let volume = if commit.lines_added + commit.lines_deleted > 0 {
                    format!(" (+{} -{})", commit.lines_added, commit.lines_deleted)
                } else {
                    String::new()
                };
                println!(
                    "    {} {} {}{}",
                    commit.author_date.format("%H:%M").to_string().dimmed(),
                    &commit.hash[..7].yellow(),
                    commit.subject,
                    volume.dimmed()
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
