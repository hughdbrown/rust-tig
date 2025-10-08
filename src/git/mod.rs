// Git operations and repository management

pub mod commit;
pub mod diff;
pub mod error;
pub mod repository;
pub mod status;
pub mod walker;

pub use commit::Commit;
pub use diff::{Diff, DiffFile, DiffHunk, DiffLine, FileStatus, LineType};
pub use error::{GitError, Result};
pub use repository::Repository;
pub use status::{EntryStatus, Status, StatusEntry};
pub use walker::CommitWalker;
