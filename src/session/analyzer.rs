use std::collections::BTreeMap;
use chrono::NaiveDate;
use crate::config::Settings;
use crate::git::Commit;
use super::types::{DayReport, Session};

/// Core session detection algorithm.
/// Takes a flat list of commits (from one or multiple repos) and groups them into sessions.
///
/// Rules:
/// 1. Sort commits by author_date ASC
/// 2. For each consecutive pair:
///    - If dates differ (crosses midnight) → new session
///    - If gap > session_gap_minutes → new session
///    - Otherwise → same session (gap time counts as work time)
/// 3. Each new session gets first_commit_minutes as a base
pub fn analyze(mut commits: Vec<Commit>, settings: &Settings) -> Vec<Session> {
    if commits.is_empty() {
        return vec![];
    }

    // Sort by author date ascending
    commits.sort_by_key(|c| c.author_date);

    let mut sessions: Vec<Session> = Vec::new();
    let mut current_commits: Vec<Commit> = vec![commits[0].clone()];
    let mut current_minutes: u32 = settings.first_commit_minutes;

    for i in 1..commits.len() {
        let prev = &commits[i - 1];
        let curr = &commits[i];

        let gap_seconds = (curr.author_date - prev.author_date).num_seconds();
        let gap_minutes = (gap_seconds / 60) as u32;

        let crosses_midnight = prev.author_date.date_naive() != curr.author_date.date_naive();
        let long_gap = gap_minutes > settings.session_gap_minutes;

        if crosses_midnight || long_gap {
            // Finalize current session
            let session = build_session(current_commits, current_minutes, settings);
            sessions.push(session);

            // Start new session
            current_commits = vec![curr.clone()];
            current_minutes = settings.first_commit_minutes;
        } else {
            // Continue same session — add gap time as work time
            current_minutes += gap_minutes;
            current_commits.push(curr.clone());
        }
    }

    // Finalize last session
    if !current_commits.is_empty() {
        let session = build_session(current_commits, current_minutes, settings);
        sessions.push(session);
    }

    sessions
}

fn build_session(commits: Vec<Commit>, duration_minutes: u32, settings: &Settings) -> Session {
    let start = commits.first().unwrap().author_date;
    let end = commits.last().unwrap().author_date;

    let mut repos: Vec<String> = commits.iter().map(|c| c.repo_name.clone()).collect();
    repos.dedup();
    repos.sort();
    repos.dedup();

    let lines_added: u32 = commits.iter().map(|c| c.lines_added).sum();
    let lines_deleted: u32 = commits.iter().map(|c| c.lines_deleted).sum();

    let volume_bonus = if settings.volume_adjustment {
        let mut bonus = 0.0_f64;
        for commit in &commits {
            let total_lines = (commit.lines_added + commit.lines_deleted) as f64;
            if total_lines > 0.0 {
                bonus += settings.volume_factor * (1.0 + total_lines / settings.volume_scale).ln();
            }
        }
        bonus.round() as u32
    } else {
        0
    };

    Session {
        start,
        end,
        duration_minutes: duration_minutes + volume_bonus,
        commits,
        repos,
        lines_added,
        lines_deleted,
    }
}

/// Group sessions by calendar date into DayReport structs.
/// Uses BTreeMap to guarantee chronological order.
pub fn group_by_day(sessions: Vec<Session>) -> Vec<DayReport> {
    let mut map: BTreeMap<NaiveDate, Vec<Session>> = BTreeMap::new();

    for session in sessions {
        map.entry(session.date()).or_default().push(session);
    }

    map.into_iter()
        .map(|(date, day_sessions)| {
            let total_minutes = day_sessions.iter().map(|s| s.duration_minutes).sum();
            let total_commits = day_sessions.iter().map(|s| s.commits.len()).sum();
            let total_lines_added = day_sessions.iter().map(|s| s.lines_added).sum();
            let total_lines_deleted = day_sessions.iter().map(|s| s.lines_deleted).sum();
            let mut repos: Vec<String> = day_sessions
                .iter()
                .flat_map(|s| s.repos.iter().cloned())
                .collect();
            repos.sort();
            repos.dedup();

            DayReport {
                date,
                sessions: day_sessions,
                total_minutes,
                total_commits,
                repos,
                total_lines_added,
                total_lines_deleted,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::DateTime;
    use std::path::PathBuf;

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

    #[test]
    fn test_single_commit_gets_base_time() {
        let commits = vec![make_commit("a1", "2026-01-05T10:00:00+00:00", "proj")];
        let settings = default_settings();
        let sessions = analyze(commits, &settings);
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].duration_minutes, 30);
    }

    #[test]
    fn test_two_close_commits_same_session() {
        let commits = vec![
            make_commit("a1", "2026-01-05T10:00:00+00:00", "proj"),
            make_commit("a2", "2026-01-05T10:45:00+00:00", "proj"),
        ];
        let settings = default_settings();
        let sessions = analyze(commits, &settings);
        assert_eq!(sessions.len(), 1);
        // 30 (base) + 45 (gap) = 75 minutes
        assert_eq!(sessions[0].duration_minutes, 75);
        assert_eq!(sessions[0].commits.len(), 2);
    }

    #[test]
    fn test_gap_exceeds_threshold_creates_new_session() {
        let commits = vec![
            make_commit("a1", "2026-01-05T10:00:00+00:00", "proj"),
            make_commit("a2", "2026-01-05T14:00:00+00:00", "proj"),
        ];
        let settings = default_settings(); // gap = 120 min, actual gap = 240 min
        let sessions = analyze(commits, &settings);
        assert_eq!(sessions.len(), 2);
        assert_eq!(sessions[0].duration_minutes, 30);
        assert_eq!(sessions[1].duration_minutes, 30);
    }

    #[test]
    fn test_midnight_crossing_creates_new_session() {
        let commits = vec![
            make_commit("a1", "2026-01-05T23:30:00+00:00", "proj"),
            make_commit("a2", "2026-01-06T00:10:00+00:00", "proj"),
        ];
        let settings = Settings {
            session_gap_minutes: 120,
            ..default_settings()
        };
        // Gap is only 40 min but crosses midnight — must be separate sessions
        let sessions = analyze(commits, &settings);
        assert_eq!(sessions.len(), 2);
    }

    #[test]
    fn test_empty_commits_returns_empty() {
        let sessions = analyze(vec![], &default_settings());
        assert!(sessions.is_empty());
    }

    #[test]
    fn test_multiple_sessions_in_same_day() {
        let commits = vec![
            make_commit("a1", "2026-01-05T09:00:00+00:00", "proj"),
            make_commit("a2", "2026-01-05T09:30:00+00:00", "proj"),
            make_commit("a3", "2026-01-05T15:00:00+00:00", "proj"),
            make_commit("a4", "2026-01-05T15:30:00+00:00", "proj"),
        ];
        let settings = default_settings();
        let sessions = analyze(commits, &settings);
        assert_eq!(sessions.len(), 2);

        let days = group_by_day(sessions);
        assert_eq!(days.len(), 1);
        assert_eq!(days[0].sessions.len(), 2);
    }

    #[test]
    fn test_commits_across_multiple_days() {
        let commits = vec![
            make_commit("a1", "2026-01-05T10:00:00+00:00", "proj"),
            make_commit("a2", "2026-01-06T10:00:00+00:00", "proj"),
            make_commit("a3", "2026-01-07T10:00:00+00:00", "proj"),
        ];
        let days = group_by_day(analyze(commits, &default_settings()));
        assert_eq!(days.len(), 3);
    }

    #[test]
    fn test_exact_gap_threshold_stays_in_session() {
        // Gap exactly = session_gap_minutes (120 min): should be SAME session
        // because condition is gap > threshold (strictly greater)
        let commits = vec![
            make_commit("a1", "2026-01-05T10:00:00+00:00", "proj"),
            make_commit("a2", "2026-01-05T12:00:00+00:00", "proj"),
        ];
        let settings = default_settings();
        let sessions = analyze(commits, &settings);
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].duration_minutes, 30 + 120);
    }

    #[test]
    fn test_commits_sorted_correctly_regardless_of_input_order() {
        let commits = vec![
            make_commit("a3", "2026-01-05T12:00:00+00:00", "proj"),
            make_commit("a1", "2026-01-05T10:00:00+00:00", "proj"),
            make_commit("a2", "2026-01-05T10:45:00+00:00", "proj"),
        ];
        let settings = default_settings();
        let sessions = analyze(commits, &settings);
        // a1 + a2 should be same session (45 min gap), a3 at 12:00 is 75 min after a2
        // 75 min < 120 min threshold, so all 3 are one session
        assert_eq!(sessions.len(), 1);
    }

    #[test]
    fn test_multi_repo_session_tracks_repos() {
        let commits = vec![
            make_commit("a1", "2026-01-05T10:00:00+00:00", "frontend"),
            make_commit("a2", "2026-01-05T10:30:00+00:00", "backend"),
        ];
        let sessions = analyze(commits, &default_settings());
        assert_eq!(sessions.len(), 1);
        assert!(sessions[0].repos.contains(&"frontend".to_string()));
        assert!(sessions[0].repos.contains(&"backend".to_string()));
    }
}
