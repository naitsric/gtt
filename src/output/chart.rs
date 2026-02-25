use std::collections::HashMap;
use chrono::{Datelike, Duration, NaiveDate};
use colored::Colorize;
use crate::output::table::format_duration;
use crate::session::types::{ClientReport, DayReport};

type DenseDay = (NaiveDate, u32, usize, u32); // (date, minutes, commits, lines_added)

pub fn print_charts(report: &ClientReport) {
    if report.days.is_empty() {
        return;
    }
    let dense = build_dense_days(&report.days, report.period_start, report.period_end);
    print_hours_chart(&dense);
    print_sparklines(&dense);
    print_multi_metric(&dense);
    print_heatmap(&dense);
}

fn build_dense_days(days: &[DayReport], start: NaiveDate, end: NaiveDate) -> Vec<DenseDay> {
    let map: HashMap<NaiveDate, &DayReport> = days.iter().map(|d| (d.date, d)).collect();
    let mut result = Vec::new();
    let mut cur = start;
    while cur <= end {
        if let Some(day) = map.get(&cur) {
            result.push((cur, day.total_minutes, day.total_commits, day.total_lines_added));
        } else {
            result.push((cur, 0, 0, 0));
        }
        cur += Duration::days(1);
    }
    result
}

fn spark_char(value: f64, max: f64) -> char {
    const SPARKS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    if max == 0.0 || value == 0.0 {
        return SPARKS[0];
    }
    let idx = ((value / max) * 7.0).round() as usize;
    SPARKS[idx.min(7)]
}

fn heat_char(minutes: u32, max: u32) -> char {
    if max == 0 || minutes == 0 {
        return ' ';
    }
    let pct = minutes as f64 / max as f64;
    if pct <= 0.25 {
        '░'
    } else if pct <= 0.50 {
        '▒'
    } else if pct <= 0.75 {
        '▓'
    } else {
        '█'
    }
}

fn filled_len(value: f64, max: f64, width: usize) -> usize {
    if max == 0.0 {
        0
    } else {
        ((value / max) * width as f64).round() as usize
    }
    .min(width)
}

fn section_header(title: &str) -> String {
    let title_chars = title.chars().count();
    let used = 4 + title_chars; // "── " (3) + title + " " (1)
    let total = 44;
    let dashes = if total > used { total - used } else { 2 };
    format!("── {} {}", title, "─".repeat(dashes))
}

// Chart 1: Horizontal bar chart by day (hours)
fn print_hours_chart(dense: &[DenseDay]) {
    println!("{}", section_header("Horas por día").bold());
    let max = dense.iter().map(|d| d.1).max().unwrap_or(0);
    for (date, minutes, _, _) in dense {
        let dur = if *minutes == 0 {
            "0m".to_string()
        } else {
            format_duration(*minutes)
        };
        let label = date.format("%a %d/%m").to_string();
        let filled = filled_len(*minutes as f64, max as f64, 24);
        let bar = format!(
            "{}{}",
            "█".repeat(filled).cyan(),
            "░".repeat(24 - filled).dimmed()
        );
        println!("  {label:10}  {bar}  {dur}");
    }
    println!();
}

// Chart 2: Sparklines (compact, one char per day)
fn print_sparklines(dense: &[DenseDay]) {
    println!("{}", section_header("Sparklines").bold());
    let start_label = dense
        .first()
        .map(|d| d.0.format("%d/%m").to_string())
        .unwrap_or_default();
    let end_label = dense
        .last()
        .map(|d| d.0.format("%d/%m").to_string())
        .unwrap_or_default();

    let max_hours = dense.iter().map(|d| d.1).max().unwrap_or(0);
    let max_commits = dense.iter().map(|d| d.2).max().unwrap_or(0);

    let spark_hours: String = dense
        .iter()
        .map(|d| spark_char(d.1 as f64, max_hours as f64))
        .collect();
    let spark_commits: String = dense
        .iter()
        .map(|d| spark_char(d.2 as f64, max_commits as f64))
        .collect();

    println!("  Horas   {} {}  {}", start_label, spark_hours.cyan(), end_label);
    println!("  Commits {} {}  {}", start_label, spark_commits.yellow(), end_label);
    println!();
}

// Chart 3: Multi-metric per day (only active days)
fn print_multi_metric(dense: &[DenseDay]) {
    println!("{}", section_header("Métricas por día").bold());
    let active: Vec<&DenseDay> = dense.iter().filter(|d| d.1 > 0).collect();
    if active.is_empty() {
        println!("  (sin actividad)");
        println!();
        return;
    }
    let max_minutes = active.iter().map(|d| d.1).max().unwrap_or(0);
    let max_commits = active.iter().map(|d| d.2).max().unwrap_or(0);
    let max_lines = active.iter().map(|d| d.3).max().unwrap_or(0);

    for (date, minutes, commits, lines) in &active {
        let label = date.format("%a %d/%m").to_string();
        println!("  {label}");

        let fh = filled_len(*minutes as f64, max_minutes as f64, 20);
        println!(
            "    Horas   │{}{}│ {}",
            "█".repeat(fh).green(),
            "░".repeat(20 - fh).dimmed(),
            format_duration(*minutes)
        );

        let fc = filled_len(*commits as f64, max_commits as f64, 20);
        println!(
            "    Commits │{}{}│ {}",
            "█".repeat(fc).yellow(),
            "░".repeat(20 - fc).dimmed(),
            commits
        );

        let fl = filled_len(*lines as f64, max_lines as f64, 20);
        println!(
            "    Código  │{}{}│ +{}",
            "█".repeat(fl).blue(),
            "░".repeat(20 - fl).dimmed(),
            lines
        );
    }
    println!();
}

// Chart 4: Weekly heat map (rows = Mon–Sun, columns = ISO weeks)
fn print_heatmap(dense: &[DenseDay]) {
    println!("{}", section_header("Mapa de calor").bold());

    // Collect ISO weeks in chronological order (dense is already sorted)
    let mut weeks: Vec<(i32, u32)> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for (date, _, _, _) in dense {
        let iw = date.iso_week();
        let key = (iw.year(), iw.week());
        if seen.insert(key) {
            weeks.push(key);
        }
    }

    // Build lookup: (iso_year, iso_week, weekday_0=Mon) -> minutes
    let mut heat: HashMap<(i32, u32, u32), u32> = HashMap::new();
    for (date, minutes, _, _) in dense {
        let iw = date.iso_week();
        let wd = date.weekday().num_days_from_monday();
        heat.insert((iw.year(), iw.week(), wd), *minutes);
    }

    let max_minutes = dense.iter().map(|d| d.1).max().unwrap_or(0);

    // Header row
    let header: String = weeks.iter().map(|(_, w)| format!(" W{w:02}")).collect();
    println!("      {}", header.dimmed());

    let day_names = ["Lun", "Mar", "Mié", "Jue", "Vie", "Sáb", "Dom"];
    for wd in 0u32..7 {
        let name = day_names[wd as usize];
        let mut row = String::new();
        for (iy, iw) in &weeks {
            if heat.contains_key(&(*iy, *iw, wd)) {
                let minutes = heat[&(*iy, *iw, wd)];
                row.push_str(&format!("   {}", heat_char(minutes, max_minutes)));
            } else {
                row.push_str("    ");
            }
        }
        println!("  {name}  {row}");
    }
    println!();
}
