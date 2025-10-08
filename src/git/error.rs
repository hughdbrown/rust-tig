use thiserror::Error;

#[derive(Error, Debug)]
pub enum GitError {
    #[error("Git repository not found")]
    RepoNotFound,

    #[error("Not in a git repository")]
    NotARepo,

    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid commit ID: {0}")]
    InvalidCommit(String),

    #[error("Reference not found: {0}")]
    RefNotFound(String),

    #[error("Invalid UTF-8 in git data")]
    InvalidUtf8,
}

pub type Result<T> = std::result::Result<T, GitError>;
