use super::{error::Result, repository::Repository};
use git2::{Status as Git2Status, StatusOptions};

/// Status of a file in the working directory
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryStatus {
    /// File is new in the index
    IndexNew,
    /// File is modified in the index
    IndexModified,
    /// File is deleted in the index
    IndexDeleted,
    /// File is renamed in the index
    IndexRenamed,
    /// File is type-changed in the index
    IndexTypeChange,
    /// File is new in the working tree
    WorktreeNew,
    /// File is modified in the working tree
    WorktreeModified,
    /// File is deleted in the working tree
    WorktreeDeleted,
    /// File is renamed in the working tree
    WorktreeRenamed,
    /// File is type-changed in the working tree
    WorktreeTypeChange,
    /// File is ignored
    Ignored,
    /// File is conflicted
    Conflicted,
}

impl EntryStatus {
    /// Check if this status represents a staged change
    pub fn is_staged(&self) -> bool {
        matches!(
            self,
            EntryStatus::IndexNew
                | EntryStatus::IndexModified
                | EntryStatus::IndexDeleted
                | EntryStatus::IndexRenamed
                | EntryStatus::IndexTypeChange
        )
    }

    /// Check if this status represents an unstaged change
    pub fn is_unstaged(&self) -> bool {
        matches!(
            self,
            EntryStatus::WorktreeNew
                | EntryStatus::WorktreeModified
                | EntryStatus::WorktreeDeleted
                | EntryStatus::WorktreeRenamed
                | EntryStatus::WorktreeTypeChange
        )
    }

    /// Get a short status code (like git status --short)
    pub fn short_code(&self) -> &'static str {
        match self {
            EntryStatus::IndexNew => "A ",
            EntryStatus::IndexModified => "M ",
            EntryStatus::IndexDeleted => "D ",
            EntryStatus::IndexRenamed => "R ",
            EntryStatus::IndexTypeChange => "T ",
            EntryStatus::WorktreeNew => "??",
            EntryStatus::WorktreeModified => " M",
            EntryStatus::WorktreeDeleted => " D",
            EntryStatus::WorktreeRenamed => " R",
            EntryStatus::WorktreeTypeChange => " T",
            EntryStatus::Ignored => "!!",
            EntryStatus::Conflicted => "UU",
        }
    }

    /// Get a description of the status
    pub fn description(&self) -> &'static str {
        match self {
            EntryStatus::IndexNew => "new file",
            EntryStatus::IndexModified => "modified",
            EntryStatus::IndexDeleted => "deleted",
            EntryStatus::IndexRenamed => "renamed",
            EntryStatus::IndexTypeChange => "typechange",
            EntryStatus::WorktreeNew => "untracked",
            EntryStatus::WorktreeModified => "modified",
            EntryStatus::WorktreeDeleted => "deleted",
            EntryStatus::WorktreeRenamed => "renamed",
            EntryStatus::WorktreeTypeChange => "typechange",
            EntryStatus::Ignored => "ignored",
            EntryStatus::Conflicted => "conflicted",
        }
    }
}

/// A single status entry
#[derive(Debug, Clone)]
pub struct StatusEntry {
    pub path: String,
    pub status: EntryStatus,
    pub index_to_workdir: bool,
}

impl StatusEntry {
    pub fn new(path: String, status: EntryStatus, index_to_workdir: bool) -> Self {
        Self {
            path,
            status,
            index_to_workdir,
        }
    }
}

/// Repository status information
#[derive(Debug, Clone)]
pub struct Status {
    pub staged: Vec<StatusEntry>,
    pub unstaged: Vec<StatusEntry>,
    pub untracked: Vec<StatusEntry>,
    pub conflicted: Vec<StatusEntry>,
}

impl Status {
    pub fn new() -> Self {
        Self {
            staged: Vec::new(),
            unstaged: Vec::new(),
            untracked: Vec::new(),
            conflicted: Vec::new(),
        }
    }

    /// Get total count of changes
    pub fn total_count(&self) -> usize {
        self.staged.len() + self.unstaged.len() + self.untracked.len() + self.conflicted.len()
    }

    /// Check if there are any changes
    pub fn has_changes(&self) -> bool {
        !self.staged.is_empty()
            || !self.unstaged.is_empty()
            || !self.untracked.is_empty()
            || !self.conflicted.is_empty()
    }
}

impl Default for Status {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert git2 status flags to our EntryStatus
fn parse_status_flags(flags: Git2Status) -> Vec<EntryStatus> {
    let mut statuses = Vec::new();

    // Check for conflicts first
    if flags.is_conflicted() {
        statuses.push(EntryStatus::Conflicted);
        return statuses;
    }

    // Index (staged) changes
    if flags.is_index_new() {
        statuses.push(EntryStatus::IndexNew);
    }
    if flags.is_index_modified() {
        statuses.push(EntryStatus::IndexModified);
    }
    if flags.is_index_deleted() {
        statuses.push(EntryStatus::IndexDeleted);
    }
    if flags.is_index_renamed() {
        statuses.push(EntryStatus::IndexRenamed);
    }
    if flags.is_index_typechange() {
        statuses.push(EntryStatus::IndexTypeChange);
    }

    // Working tree (unstaged) changes
    if flags.is_wt_new() {
        statuses.push(EntryStatus::WorktreeNew);
    }
    if flags.is_wt_modified() {
        statuses.push(EntryStatus::WorktreeModified);
    }
    if flags.is_wt_deleted() {
        statuses.push(EntryStatus::WorktreeDeleted);
    }
    if flags.is_wt_renamed() {
        statuses.push(EntryStatus::WorktreeRenamed);
    }
    if flags.is_wt_typechange() {
        statuses.push(EntryStatus::WorktreeTypeChange);
    }

    if flags.is_ignored() {
        statuses.push(EntryStatus::Ignored);
    }

    statuses
}

/// Load the repository status asynchronously
pub async fn load_status(repo: &Repository) -> Result<Status> {
    let repo_path = repo.path().to_path_buf();

    tokio::task::spawn_blocking(move || {
        let git_repo = git2::Repository::open(repo_path)?;
        let mut status = Status::new();

        let mut opts = StatusOptions::new();
        opts.include_untracked(true);
        opts.recurse_untracked_dirs(true);
        opts.exclude_submodules(true);

        let statuses = git_repo.statuses(Some(&mut opts))?;

        for entry in statuses.iter() {
            let path = entry
                .path()
                .unwrap_or("<unknown>")
                .to_string();

            let flags = entry.status();
            let entry_statuses = parse_status_flags(flags);

            for entry_status in entry_statuses {
                let status_entry = StatusEntry::new(path.clone(), entry_status, false);

                if flags.is_conflicted() {
                    status.conflicted.push(status_entry);
                } else if entry_status.is_staged() {
                    status.staged.push(status_entry);
                } else if entry_status == EntryStatus::WorktreeNew {
                    status.untracked.push(status_entry);
                } else if entry_status.is_unstaged() {
                    status.unstaged.push(status_entry);
                }
            }
        }

        Ok(status)
    })
    .await
    .map_err(|_| super::error::GitError::RepoNotFound)?
}

/// Stage a file (add to index)
pub async fn stage_file(repo: &Repository, path: String) -> Result<()> {
    let repo_path = repo.path().to_path_buf();

    tokio::task::spawn_blocking(move || {
        let git_repo = git2::Repository::open(repo_path)?;
        let mut index = git_repo.index()?;

        // Add the file to the index
        index.add_path(std::path::Path::new(&path))?;
        index.write()?;

        Ok(())
    })
    .await
    .map_err(|_| super::error::GitError::RepoNotFound)?
}

/// Unstage a file (reset from index to HEAD)
pub async fn unstage_file(repo: &Repository, path: String) -> Result<()> {
    let repo_path = repo.path().to_path_buf();

    tokio::task::spawn_blocking(move || {
        let git_repo = git2::Repository::open(repo_path)?;

        // Get HEAD tree
        let head = git_repo.head()?;
        let head_commit = head.peel_to_commit()?;
        let head_tree = head_commit.tree()?;

        // Get the object from HEAD for this path
        let head_entry = head_tree.get_path(std::path::Path::new(&path));

        // Reset the index entry to match HEAD
        let mut index = git_repo.index()?;

        if let Ok(entry) = head_entry {
            // File exists in HEAD, restore it to index
            index.add(&git2::IndexEntry {
                ctime: git2::IndexTime::new(0, 0),
                mtime: git2::IndexTime::new(0, 0),
                dev: 0,
                ino: 0,
                mode: entry.filemode() as u32,
                uid: 0,
                gid: 0,
                file_size: 0,
                id: entry.id(),
                flags: 0,
                flags_extended: 0,
                path: path.as_bytes().to_vec(),
            })?;
        } else {
            // File doesn't exist in HEAD, remove from index
            index.remove_path(std::path::Path::new(&path))?;
        }

        index.write()?;

        Ok(())
    })
    .await
    .map_err(|_| super::error::GitError::RepoNotFound)?
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    async fn create_test_repo_with_changes() -> (TempDir, Repository) {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        let git_repo = git2::Repository::init(repo_path).unwrap();
        let sig = git2::Signature::now("Test", "test@example.com").unwrap();

        // Create initial commit
        fs::write(repo_path.join("existing.txt"), "existing content\n").unwrap();
        let mut index = git_repo.index().unwrap();
        index.add_path(std::path::Path::new("existing.txt")).unwrap();
        index.write().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = git_repo.find_tree(tree_id).unwrap();
        git_repo
            .commit(Some("HEAD"), &sig, &sig, "Initial", &tree, &[])
            .unwrap();

        // Create staged changes
        fs::write(repo_path.join("staged.txt"), "staged content\n").unwrap();
        let mut index = git_repo.index().unwrap();
        index.add_path(std::path::Path::new("staged.txt")).unwrap();
        index.write().unwrap();

        // Create unstaged changes
        fs::write(repo_path.join("existing.txt"), "modified content\n").unwrap();

        // Create untracked file
        fs::write(repo_path.join("untracked.txt"), "untracked content\n").unwrap();

        let repo = Repository::open(repo_path).await.unwrap();
        (temp_dir, repo)
    }

    #[tokio::test]
    async fn test_load_status() {
        let (_temp_dir, repo) = create_test_repo_with_changes().await;
        let status = load_status(&repo).await.unwrap();

        assert!(status.has_changes());
        assert!(status.staged.len() > 0);
        assert!(status.untracked.len() > 0);
    }

    #[tokio::test]
    async fn test_status_counts() {
        let (_temp_dir, repo) = create_test_repo_with_changes().await;
        let status = load_status(&repo).await.unwrap();

        let total = status.total_count();
        assert_eq!(
            total,
            status.staged.len() + status.unstaged.len() + status.untracked.len()
        );
    }

    #[test]
    fn test_entry_status_short_code() {
        assert_eq!(EntryStatus::IndexNew.short_code(), "A ");
        assert_eq!(EntryStatus::WorktreeModified.short_code(), " M");
        assert_eq!(EntryStatus::WorktreeNew.short_code(), "??");
    }

    #[test]
    fn test_entry_status_is_staged() {
        assert!(EntryStatus::IndexNew.is_staged());
        assert!(EntryStatus::IndexModified.is_staged());
        assert!(!EntryStatus::WorktreeModified.is_staged());
    }

    #[test]
    fn test_entry_status_is_unstaged() {
        assert!(EntryStatus::WorktreeModified.is_unstaged());
        assert!(EntryStatus::WorktreeDeleted.is_unstaged());
        assert!(!EntryStatus::IndexModified.is_unstaged());
    }
}
