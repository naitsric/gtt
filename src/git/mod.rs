pub mod log;
pub mod parser;

pub use log::{get_repo_user_email, run_git_log};
pub use parser::{parse_git_log, Commit};
