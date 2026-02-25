use std::path::PathBuf;
use gtt::git::parse_git_log;

fn repo() -> PathBuf {
    PathBuf::from("/home/user/project")
}

#[test]
fn test_parse_single_commit() {
    let raw = "abc123\x002026-01-05T10:30:00+00:00\x00dev@example.com\x00Dev User\x00Fix bug\x00END\n";
    let commits = parse_git_log(raw, &repo(), &[]).unwrap();
    assert_eq!(commits.len(), 1);
    let c = &commits[0];
    assert_eq!(c.hash, "abc123");
    assert_eq!(c.author_email, "dev@example.com");
    assert_eq!(c.author_name, "Dev User");
    assert_eq!(c.subject, "Fix bug");
    assert_eq!(c.repo_name, "project");
}

#[test]
fn test_parse_empty_output() {
    let commits = parse_git_log("", &repo(), &[]).unwrap();
    assert!(commits.is_empty());
}

#[test]
fn test_skip_dependabot() {
    let raw = "bot1\x002026-01-05T10:00:00+00:00\x00dependabot[bot]@users.noreply.github.com\x00dependabot[bot]\x00Bump dep\x00END\n";
    let bots = vec!["dependabot[bot]".to_string()];
    let commits = parse_git_log(raw, &repo(), &bots).unwrap();
    assert!(commits.is_empty());
}

#[test]
fn test_skip_github_actions() {
    let raw = "ci1\x002026-01-05T10:00:00+00:00\x00github-actions[bot]@users.noreply.github.com\x00github-actions[bot]\x00Auto release\x00END\n";
    let bots = vec!["github-actions[bot]".to_string()];
    let commits = parse_git_log(raw, &repo(), &bots).unwrap();
    assert!(commits.is_empty());
}

#[test]
fn test_bot_auto_detected_by_pattern() {
    // Even without explicit bot list, names ending in [bot] should be skipped
    let raw = "ci1\x002026-01-05T10:00:00+00:00\x00somebot[bot]@example.com\x00somebot[bot]\x00Auto commit\x00END\n";
    let commits = parse_git_log(raw, &repo(), &[]).unwrap();
    assert!(commits.is_empty());
}

#[test]
fn test_parse_multiple_commits() {
    let raw = concat!(
        "a1\x002026-01-05T10:00:00+00:00\x00dev@ex.com\x00Dev\x00First\x00END\n",
        "a2\x002026-01-05T11:00:00+00:00\x00dev@ex.com\x00Dev\x00Second\x00END\n",
        "a3\x002026-01-05T12:00:00+00:00\x00dev@ex.com\x00Dev\x00Third\x00END\n",
    );
    let commits = parse_git_log(raw, &repo(), &[]).unwrap();
    assert_eq!(commits.len(), 3);
}

#[test]
fn test_repo_name_extracted_from_path() {
    let repo = PathBuf::from("/home/user/startupx-web");
    let raw = "abc1\x002026-01-05T10:00:00+00:00\x00dev@ex.com\x00Dev\x00Commit\x00END\n";
    let commits = parse_git_log(raw, &repo, &[]).unwrap();
    assert_eq!(commits[0].repo_name, "startupx-web");
}

#[test]
fn test_subject_with_special_characters() {
    let raw = "abc1\x002026-01-05T10:00:00+00:00\x00dev@ex.com\x00Dev\x00fix: handle edge case with 'quotes' & symbols\x00END\n";
    let commits = parse_git_log(raw, &repo(), &[]).unwrap();
    assert_eq!(commits[0].subject, "fix: handle edge case with 'quotes' & symbols");
}

#[test]
fn test_author_date_timezone_preserved() {
    let raw = "abc1\x002026-01-05T10:00:00-05:00\x00dev@ex.com\x00Dev\x00Commit\x00END\n";
    let commits = parse_git_log(raw, &repo(), &[]).unwrap();
    assert_eq!(commits[0].author_date.offset().local_minus_utc(), -5 * 3600);
}
