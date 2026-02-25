use chrono::DateTime;
use std::path::PathBuf;
use gtt::git::Commit;
use gtt::config::Settings;
use gtt::session::{analyze, group_by_day};

fn make_commit(hash: &str, date_str: &str, repo: &str) -> Commit {
    Commit {
        hash: hash.to_string(),
        author_date: DateTime::parse_from_rfc3339(date_str).unwrap(),
        author_email: "dev@example.com".to_string(),
        author_name: "Dev".to_string(),
        subject: format!("commit {}", hash),
        repo_path: PathBuf::from(format!("/repos/{}", repo)),
        repo_name: repo.to_string(),
        lines_added: 0,
        lines_deleted: 0,
    }
}

fn make_commit_with_volume(hash: &str, date_str: &str, repo: &str, added: u32, deleted: u32) -> Commit {
    Commit {
        hash: hash.to_string(),
        author_date: DateTime::parse_from_rfc3339(date_str).unwrap(),
        author_email: "dev@example.com".to_string(),
        author_name: "Dev".to_string(),
        subject: format!("commit {}", hash),
        repo_path: PathBuf::from(format!("/repos/{}", repo)),
        repo_name: repo.to_string(),
        lines_added: added,
        lines_deleted: deleted,
    }
}

fn default_settings() -> Settings {
    Settings {
        session_gap_minutes: 120,
        first_commit_minutes: 30,
        exclude_weekends: false,
        bot_authors: vec![],
        volume_adjustment: false,
        volume_factor: 5.0,
        volume_scale: 50.0,
    }
}

fn volume_settings() -> Settings {
    Settings {
        volume_adjustment: true,
        ..default_settings()
    }
}

#[test]
fn test_single_commit_gets_base_time() {
    let commits = vec![make_commit("a1", "2026-01-05T10:00:00+00:00", "proj")];
    let sessions = analyze(commits, &default_settings());
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].duration_minutes, 30);
}

#[test]
fn test_two_close_commits_same_session() {
    let commits = vec![
        make_commit("a1", "2026-01-05T10:00:00+00:00", "proj"),
        make_commit("a2", "2026-01-05T10:45:00+00:00", "proj"),
    ];
    let sessions = analyze(commits, &default_settings());
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].duration_minutes, 75); // 30 base + 45 gap
}

#[test]
fn test_large_gap_creates_new_session() {
    let commits = vec![
        make_commit("a1", "2026-01-05T09:00:00+00:00", "proj"),
        make_commit("a2", "2026-01-05T15:00:00+00:00", "proj"), // 6 hour gap
    ];
    let sessions = analyze(commits, &default_settings());
    assert_eq!(sessions.len(), 2);
    assert_eq!(sessions[0].duration_minutes, 30);
    assert_eq!(sessions[1].duration_minutes, 30);
}

#[test]
fn test_midnight_crossing_always_new_session() {
    // 40 min gap but crosses midnight — must be new session
    let commits = vec![
        make_commit("a1", "2026-01-05T23:30:00+00:00", "proj"),
        make_commit("a2", "2026-01-06T00:10:00+00:00", "proj"),
    ];
    let sessions = analyze(commits, &default_settings());
    assert_eq!(sessions.len(), 2);
}

#[test]
fn test_empty_input() {
    let sessions = analyze(vec![], &default_settings());
    assert!(sessions.is_empty());
}

#[test]
fn test_three_sessions_in_one_day() {
    let commits = vec![
        make_commit("a1", "2026-01-05T08:00:00+00:00", "proj"),
        make_commit("a2", "2026-01-05T08:30:00+00:00", "proj"),
        // gap 3h → new session
        make_commit("a3", "2026-01-05T12:00:00+00:00", "proj"),
        make_commit("a4", "2026-01-05T12:20:00+00:00", "proj"),
        // gap 4h → new session
        make_commit("a5", "2026-01-05T17:00:00+00:00", "proj"),
    ];
    let sessions = analyze(commits, &default_settings());
    assert_eq!(sessions.len(), 3);

    let days = group_by_day(sessions);
    assert_eq!(days.len(), 1);
    assert_eq!(days[0].sessions.len(), 3);
}

#[test]
fn test_group_by_day_chronological_order() {
    let commits = vec![
        make_commit("a1", "2026-01-07T10:00:00+00:00", "proj"),
        make_commit("a2", "2026-01-05T10:00:00+00:00", "proj"),
        make_commit("a3", "2026-01-06T10:00:00+00:00", "proj"),
    ];
    let days = group_by_day(analyze(commits, &default_settings()));
    assert_eq!(days.len(), 3);
    assert!(days[0].date < days[1].date);
    assert!(days[1].date < days[2].date);
}

#[test]
fn test_total_minutes_per_day() {
    // Two sessions: 30 + 30 + 45 = 105 min total (30 base + 45 gap for session 1, 30 base for session 2)
    let commits = vec![
        make_commit("a1", "2026-01-05T09:00:00+00:00", "proj"),
        make_commit("a2", "2026-01-05T09:45:00+00:00", "proj"),
        make_commit("a3", "2026-01-05T14:00:00+00:00", "proj"),
    ];
    let sessions = analyze(commits, &default_settings());
    let days = group_by_day(sessions);
    assert_eq!(days.len(), 1);
    // Session 1: 30 + 45 = 75 min; Session 2: 30 min → total 105
    assert_eq!(days[0].total_minutes, 105);
}

#[test]
fn test_custom_gap_threshold() {
    let settings = Settings {
        session_gap_minutes: 60, // 1 hour threshold instead of 2
        ..default_settings()
    };
    let commits = vec![
        make_commit("a1", "2026-01-05T10:00:00+00:00", "proj"),
        make_commit("a2", "2026-01-05T11:30:00+00:00", "proj"), // 90 min gap > 60 min threshold
    ];
    let sessions = analyze(commits, &settings);
    assert_eq!(sessions.len(), 2);
}

#[test]
fn test_custom_first_commit_minutes() {
    let settings = Settings {
        first_commit_minutes: 60, // 1 hour base
        ..default_settings()
    };
    let commits = vec![make_commit("a1", "2026-01-05T10:00:00+00:00", "proj")];
    let sessions = analyze(commits, &settings);
    assert_eq!(sessions[0].duration_minutes, 60);
}

#[test]
fn test_multi_repo_commits_merged_into_sessions() {
    let commits = vec![
        make_commit("a1", "2026-01-05T10:00:00+00:00", "frontend"),
        make_commit("a2", "2026-01-05T10:30:00+00:00", "backend"),
        make_commit("a3", "2026-01-05T10:50:00+00:00", "frontend"),
    ];
    let sessions = analyze(commits, &default_settings());
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].commits.len(), 3);
    let repos = &sessions[0].repos;
    assert!(repos.contains(&"frontend".to_string()));
    assert!(repos.contains(&"backend".to_string()));
}

#[test]
fn test_exact_gap_boundary_stays_same_session() {
    // Exactly session_gap_minutes (120) → same session (strictly greater required)
    let commits = vec![
        make_commit("a1", "2026-01-05T10:00:00+00:00", "proj"),
        make_commit("a2", "2026-01-05T12:00:00+00:00", "proj"),
    ];
    let sessions = analyze(commits, &default_settings());
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].duration_minutes, 30 + 120);
}

// --- Volume adjustment tests ---

#[test]
fn test_volume_disabled_no_bonus() {
    let commits = vec![make_commit_with_volume("a1", "2026-01-05T10:00:00+00:00", "proj", 200, 50)];
    let sessions = analyze(commits, &default_settings());
    assert_eq!(sessions[0].duration_minutes, 30); // no bonus when disabled
}

#[test]
fn test_volume_enabled_adds_bonus() {
    let commits = vec![make_commit_with_volume("a1", "2026-01-05T10:00:00+00:00", "proj", 200, 50)];
    let sessions = analyze(commits, &volume_settings());
    // 250 lines, bonus = 5 * ln(1 + 250/50) = 5 * ln(6) ≈ 8.96 → round to 9
    assert_eq!(sessions[0].duration_minutes, 30 + 9);
}

#[test]
fn test_volume_zero_lines_no_bonus() {
    let commits = vec![make_commit_with_volume("a1", "2026-01-05T10:00:00+00:00", "proj", 0, 0)];
    let sessions = analyze(commits, &volume_settings());
    assert_eq!(sessions[0].duration_minutes, 30);
}

#[test]
fn test_volume_lines_aggregated_in_session() {
    let commits = vec![
        make_commit_with_volume("a1", "2026-01-05T10:00:00+00:00", "proj", 10, 5),
        make_commit_with_volume("a2", "2026-01-05T10:30:00+00:00", "proj", 20, 10),
    ];
    let sessions = analyze(commits, &default_settings());
    assert_eq!(sessions[0].lines_added, 30);
    assert_eq!(sessions[0].lines_deleted, 15);
}

#[test]
fn test_volume_per_commit_bonus() {
    // Two commits with volume — bonus computed per commit, not on total
    let commits = vec![
        make_commit_with_volume("a1", "2026-01-05T10:00:00+00:00", "proj", 50, 0),
        make_commit_with_volume("a2", "2026-01-05T10:30:00+00:00", "proj", 50, 0),
    ];
    let sessions = analyze(commits, &volume_settings());
    // Per commit: 5 * ln(1 + 50/50) = 5 * ln(2) ≈ 3.47, two commits = 6.93 → round to 7
    // Base: 30 + 30 gap + 7 volume = 67
    assert_eq!(sessions[0].duration_minutes, 30 + 30 + 7);
}

#[test]
fn test_volume_lines_aggregated_in_day_report() {
    let commits = vec![
        make_commit_with_volume("a1", "2026-01-05T10:00:00+00:00", "proj", 100, 20),
        make_commit_with_volume("a2", "2026-01-05T15:00:00+00:00", "proj", 50, 10),
    ];
    let days = group_by_day(analyze(commits, &default_settings()));
    assert_eq!(days[0].total_lines_added, 150);
    assert_eq!(days[0].total_lines_deleted, 30);
}
