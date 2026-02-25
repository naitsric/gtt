use std::path::Path;
use std::process::Command;
use anyhow::Result;
use chrono::NaiveDate;
use crate::errors::GttError;

/// Format string for git log: NUL-separated fields to avoid issues with newlines in subjects.
/// Fields: hash, author-date (ISO 8601 strict), author-email, author-name, subject
const GIT_LOG_FORMAT: &str = "--format=%H%x00%aI%x00%ae%x00%an%x00%s%x00END";

pub fn run_git_log(
    repo_path: &Path,
    since: Option<NaiveDate>,
    until: Option<NaiveDate>,
    author_email: Option<&str>,
) -> Result<String> {
    // Verify the path is a git repo
    let git_check = Command::new("git")
        .args(["-C", &repo_path.to_string_lossy(), "rev-parse", "--git-dir"])
        .output()
        .map_err(|e| GttError::GitCommandFailed(e.to_string()))?;

    if !git_check.status.success() {
        return Err(GttError::NotAGitRepo(repo_path.to_string_lossy().to_string()).into());
    }

    let mut args = vec![
        "-C".to_string(),
        repo_path.to_string_lossy().to_string(),
        "log".to_string(),
        GIT_LOG_FORMAT.to_string(),
        "--no-merges".to_string(),
    ];

    if let Some(since_date) = since {
        args.push(format!("--after={}", since_date.format("%Y-%m-%d")));
    }
    if let Some(until_date) = until {
        // Add 1 day to make until inclusive
        let next_day = until_date.succ_opt().unwrap_or(until_date);
        args.push(format!("--before={}", next_day.format("%Y-%m-%d")));
    }
    if let Some(email) = author_email {
        args.push(format!("--author={}", email));
    }

    let output = Command::new("git")
        .args(&args)
        .output()
        .map_err(|e| GttError::GitCommandFailed(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(GttError::GitCommandFailed(stderr.to_string()).into());
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Run git log with only hash and numstat output.
pub fn run_git_log_numstat(
    repo_path: &Path,
    since: Option<NaiveDate>,
    until: Option<NaiveDate>,
    author_email: Option<&str>,
) -> Result<String> {
    let mut args = vec![
        "-C".to_string(),
        repo_path.to_string_lossy().to_string(),
        "log".to_string(),
        "--format=%H".to_string(),
        "--numstat".to_string(),
        "--no-merges".to_string(),
    ];

    if let Some(since_date) = since {
        args.push(format!("--after={}", since_date.format("%Y-%m-%d")));
    }
    if let Some(until_date) = until {
        let next_day = until_date.succ_opt().unwrap_or(until_date);
        args.push(format!("--before={}", next_day.format("%Y-%m-%d")));
    }
    if let Some(email) = author_email {
        args.push(format!("--author={}", email));
    }

    let output = Command::new("git")
        .args(&args)
        .output()
        .map_err(|e| GttError::GitCommandFailed(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(GttError::GitCommandFailed(stderr.to_string()).into());
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Get the git user email for the repo (falls back to global config)
pub fn get_repo_user_email(repo_path: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["-C", &repo_path.to_string_lossy(), "config", "user.email"])
        .output()
        .ok()?;
    if output.status.success() {
        let email = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !email.is_empty() {
            return Some(email);
        }
    }
    None
}
