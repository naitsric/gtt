use thiserror::Error;

#[derive(Error, Debug)]
pub enum GttError {
    #[error("Configuration file not found. Run `gtt init` to get started.")]
    ConfigNotFound,

    #[error("Configuration error: {0}")]
    ConfigParse(String),

    #[error("Client '{0}' not found in configuration.")]
    ClientNotFound(String),

    #[allow(dead_code)]
    #[error("Repository not found: {0}")]
    RepoNotFound(String),

    #[error("Not a git repository: {0}")]
    NotAGitRepo(String),

    #[error("Git command failed: {0}")]
    GitCommandFailed(String),

    #[error("Failed to parse git log output: {0}")]
    GitParseFailed(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Date parse error: {0}")]
    DateParse(String),

    #[error("Invalid date range: {0}")]
    InvalidDateRange(String),
}
