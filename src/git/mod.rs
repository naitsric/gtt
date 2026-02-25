pub mod log;
pub mod parser;

pub use log::{get_repo_user_email, run_git_log, run_git_log_numstat};
pub use parser::{parse_git_log, parse_numstat, merge_numstat, Commit};
