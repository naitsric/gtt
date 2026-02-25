use std::collections::HashMap;
use std::path::{Path, PathBuf};
use chrono::{DateTime, FixedOffset};
use anyhow::Result;
use crate::errors::GttError;

#[derive(Debug, Clone)]
pub struct Commit {
    pub hash: String,
    /// Always author date, never commit date — robust against rebase/amend
    pub author_date: DateTime<FixedOffset>,
    #[allow(dead_code)]
    pub author_email: String,
    #[allow(dead_code)]
    pub author_name: String,
    pub subject: String,
    #[allow(dead_code)]
    pub repo_path: PathBuf,
    pub repo_name: String,
    pub lines_added: u32,
    pub lines_deleted: u32,
}

/// Parse raw git log output (NUL-separated records ending with END).
/// Format: hash\x00author_date\x00author_email\x00author_name\x00subject\x00END
pub fn parse_git_log(raw: &str, repo_path: &Path, bot_authors: &[String]) -> Result<Vec<Commit>> {
    let repo_name = repo_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| repo_path.to_string_lossy().to_string());

    let mut commits = Vec::new();

    // Split on the END marker to get individual commit records
    for record in raw.split("\x00END") {
        let record = record.trim_matches('\n').trim();
        if record.is_empty() {
            continue;
        }

        let fields: Vec<&str> = record.splitn(5, '\x00').collect();
        if fields.len() < 5 {
            // Skip incomplete records
            continue;
        }

        let hash = fields[0].trim().to_string();
        let date_str = fields[1].trim();
        let author_email = fields[2].trim().to_string();
        let author_name = fields[3].trim().to_string();
        let subject = fields[4].trim().to_string();

        if hash.is_empty() {
            continue;
        }

        // Skip bot authors
        if is_bot(author_name.as_str(), author_email.as_str(), bot_authors) {
            continue;
        }

        let author_date = DateTime::parse_from_rfc3339(date_str)
            .map_err(|e| GttError::GitParseFailed(format!("Invalid date '{}': {}", date_str, e)))?;

        commits.push(Commit {
            hash,
            author_date,
            author_email,
            author_name,
            subject,
            repo_path: repo_path.to_path_buf(),
            repo_name: repo_name.clone(),
            lines_added: 0,
            lines_deleted: 0,
        });
    }

    Ok(commits)
}

/// Parse `git log --format=%H --numstat` output into a map of hash → (added, deleted).
pub fn parse_numstat(raw: &str) -> HashMap<String, (u32, u32)> {
    let mut map: HashMap<String, (u32, u32)> = HashMap::new();
    let mut current_hash: Option<String> = None;

    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Try numstat line: added\tdeleted\tfilename (tab-separated, 3+ fields)
        let parts: Vec<&str> = line.splitn(3, '\t').collect();
        if parts.len() == 3 {
            if let (Ok(added), Ok(deleted)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                if let Some(ref hash) = current_hash {
                    let entry = map.entry(hash.clone()).or_insert((0, 0));
                    entry.0 += added;
                    entry.1 += deleted;
                }
                continue;
            }
            // Binary files show "-\t-\tfilename" — parse fails, fall through to hash check
        }

        // If not numstat, treat as a hash line (hex chars, at least 4)
        if line.len() >= 4 && line.chars().all(|c| c.is_ascii_hexdigit()) {
            current_hash = Some(line.to_string());
            map.entry(line.to_string()).or_insert((0, 0));
        }
    }

    map
}

/// Merge numstat data into already-parsed commits by hash.
pub fn merge_numstat(commits: &mut [Commit], numstat: &HashMap<String, (u32, u32)>) {
    for commit in commits.iter_mut() {
        if let Some(&(added, deleted)) = numstat.get(&commit.hash) {
            commit.lines_added = added;
            commit.lines_deleted = deleted;
        }
    }
}

fn is_bot(name: &str, email: &str, bot_authors: &[String]) -> bool {
    for bot in bot_authors {
        let bot_lower = bot.to_lowercase();
        if name.to_lowercase().contains(&bot_lower) || email.to_lowercase().contains(&bot_lower) {
            return true;
        }
    }
    // Also skip common CI patterns
    if name.ends_with("[bot]") || email.ends_with("[bot]") {
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_parse_basic_commit() {
        let raw = "abc123\x002026-01-05T10:30:00+00:00\x00dev@example.com\x00Dev User\x00Fix bug\x00END\n";
        let repo = PathBuf::from("/home/user/project");
        let commits = parse_git_log(raw, &repo, &[]).unwrap();
        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].hash, "abc123");
        assert_eq!(commits[0].author_email, "dev@example.com");
        assert_eq!(commits[0].subject, "Fix bug");
    }

    #[test]
    fn test_skip_bot_commits() {
        let raw = "abc123\x002026-01-05T10:30:00+00:00\x00dependabot[bot]@users.noreply.github.com\x00dependabot[bot]\x00Bump version\x00END\n";
        let repo = PathBuf::from("/home/user/project");
        let bot_authors = vec!["dependabot[bot]".to_string()];
        let commits = parse_git_log(raw, &repo, &bot_authors).unwrap();
        assert_eq!(commits.len(), 0);
    }

    #[test]
    fn test_parse_multiple_commits() {
        let raw = concat!(
            "abc1\x002026-01-05T10:30:00+00:00\x00dev@ex.com\x00Dev\x00Commit 1\x00END\n",
            "abc2\x002026-01-05T11:00:00+00:00\x00dev@ex.com\x00Dev\x00Commit 2\x00END\n",
            "abc3\x002026-01-05T14:00:00+00:00\x00dev@ex.com\x00Dev\x00Commit 3\x00END\n",
        );
        let repo = PathBuf::from("/home/user/project");
        let commits = parse_git_log(raw, &repo, &[]).unwrap();
        assert_eq!(commits.len(), 3);
    }
}
