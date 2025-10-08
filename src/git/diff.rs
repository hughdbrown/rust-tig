use super::{error::Result, repository::Repository};
use git2::{Diff as Git2Diff, DiffDelta, DiffOptions, Oid};

/// Type of diff line
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineType {
    Context,
    Addition,
    Deletion,
    FileHeader,
    HunkHeader,
}

/// A single line in a diff
#[derive(Debug, Clone)]
pub struct DiffLine {
    pub line_type: LineType,
    pub content: String,
    pub old_lineno: Option<u32>,
    pub new_lineno: Option<u32>,
}

impl DiffLine {
    pub fn new(
        line_type: LineType,
        content: String,
        old_lineno: Option<u32>,
        new_lineno: Option<u32>,
    ) -> Self {
        Self {
            line_type,
            content,
            old_lineno,
            new_lineno,
        }
    }
}

/// A hunk in a diff
#[derive(Debug, Clone)]
pub struct DiffHunk {
    pub header: String,
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
    pub lines: Vec<DiffLine>,
}

impl DiffHunk {
    pub fn new(header: String, old_start: u32, old_lines: u32, new_start: u32, new_lines: u32) -> Self {
        Self {
            header,
            old_start,
            old_lines,
            new_start,
            new_lines,
            lines: Vec::new(),
        }
    }
}

/// File status in a diff
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileStatus {
    Added,
    Deleted,
    Modified,
    Renamed,
    Copied,
    Ignored,
    Untracked,
    Typechange,
}

/// A file in a diff
#[derive(Debug, Clone)]
pub struct DiffFile {
    pub old_path: Option<String>,
    pub new_path: Option<String>,
    pub status: FileStatus,
    pub hunks: Vec<DiffHunk>,
    pub additions: usize,
    pub deletions: usize,
    pub is_binary: bool,
}

impl DiffFile {
    pub fn new(old_path: Option<String>, new_path: Option<String>, status: FileStatus) -> Self {
        Self {
            old_path,
            new_path,
            status,
            hunks: Vec::new(),
            additions: 0,
            deletions: 0,
            is_binary: false,
        }
    }

    /// Get the display path for this file
    pub fn path(&self) -> &str {
        self.new_path
            .as_ref()
            .or(self.old_path.as_ref())
            .map(|s| s.as_str())
            .unwrap_or("<unknown>")
    }

    /// Get a summary line for this file (e.g., "+10, -5")
    pub fn stats_summary(&self) -> String {
        format!("+{}, -{}", self.additions, self.deletions)
    }
}

/// A complete diff
#[derive(Debug, Clone)]
pub struct Diff {
    pub files: Vec<DiffFile>,
}

impl Diff {
    pub fn new() -> Self {
        Self { files: Vec::new() }
    }

    /// Get total statistics across all files
    pub fn total_stats(&self) -> (usize, usize) {
        let additions = self.files.iter().map(|f| f.additions).sum();
        let deletions = self.files.iter().map(|f| f.deletions).sum();
        (additions, deletions)
    }

    /// Get the total number of lines in the diff
    pub fn total_lines(&self) -> usize {
        self.files
            .iter()
            .map(|f| f.hunks.iter().map(|h| h.lines.len()).sum::<usize>())
            .sum()
    }
}

impl Default for Diff {
    fn default() -> Self {
        Self::new()
    }
}

/// Load a diff for a commit
pub async fn load_commit_diff(repo: &Repository, commit_id: Oid) -> Result<Diff> {
    let repo_path = repo.path().to_path_buf();

    tokio::task::spawn_blocking(move || {
        let git_repo = git2::Repository::open(repo_path)?;
        let commit = git_repo.find_commit(commit_id)?;

        let mut diff_options = DiffOptions::new();
        diff_options.context_lines(3);

        let diff = if commit.parent_count() == 0 {
            // First commit - diff against empty tree
            let tree = commit.tree()?;
            git_repo.diff_tree_to_tree(None, Some(&tree), Some(&mut diff_options))?
        } else {
            // Normal commit - diff against parent
            let parent = commit.parent(0)?;
            let parent_tree = parent.tree()?;
            let commit_tree = commit.tree()?;
            git_repo.diff_tree_to_tree(
                Some(&parent_tree),
                Some(&commit_tree),
                Some(&mut diff_options),
            )?
        };

        parse_git2_diff(&diff)
    })
    .await
    .map_err(|_| super::error::GitError::RepoNotFound)?
}

/// Load a diff for staged changes (HEAD vs index) for a specific path
pub async fn load_staged_diff(repo: &Repository, path: Option<String>) -> Result<Diff> {
    let repo_path = repo.path().to_path_buf();

    tokio::task::spawn_blocking(move || {
        let git_repo = git2::Repository::open(repo_path)?;
        let head = git_repo.head()?.peel_to_tree()?;

        let mut diff_options = DiffOptions::new();
        diff_options.context_lines(3);
        if let Some(path) = path {
            diff_options.pathspec(path);
        }

        let diff = git_repo.diff_tree_to_index(Some(&head), None, Some(&mut diff_options))?;

        parse_git2_diff(&diff)
    })
    .await
    .map_err(|_| super::error::GitError::RepoNotFound)?
}

/// Load a diff for unstaged changes (index vs workdir) for a specific path
pub async fn load_unstaged_diff(repo: &Repository, path: Option<String>) -> Result<Diff> {
    let repo_path = repo.path().to_path_buf();

    tokio::task::spawn_blocking(move || {
        let git_repo = git2::Repository::open(repo_path)?;

        let mut diff_options = DiffOptions::new();
        diff_options.context_lines(3);
        diff_options.include_untracked(true);
        if let Some(path) = path {
            diff_options.pathspec(path);
        }

        let diff = git_repo.diff_index_to_workdir(None, Some(&mut diff_options))?;

        parse_git2_diff(&diff)
    })
    .await
    .map_err(|_| super::error::GitError::RepoNotFound)?
}

/// Parse a git2 diff into our Diff structure
fn parse_git2_diff(git2_diff: &Git2Diff) -> Result<Diff> {
    use std::cell::RefCell;

    let diff = RefCell::new(Diff::new());

    git2_diff.foreach(
        &mut |delta: DiffDelta, _progress: f32| {
            let old_path = delta.old_file().path().map(|p| p.to_string_lossy().to_string());
            let new_path = delta.new_file().path().map(|p| p.to_string_lossy().to_string());

            let status = match delta.status() {
                git2::Delta::Added => FileStatus::Added,
                git2::Delta::Deleted => FileStatus::Deleted,
                git2::Delta::Modified => FileStatus::Modified,
                git2::Delta::Renamed => FileStatus::Renamed,
                git2::Delta::Copied => FileStatus::Copied,
                git2::Delta::Ignored => FileStatus::Ignored,
                git2::Delta::Untracked => FileStatus::Untracked,
                git2::Delta::Typechange => FileStatus::Typechange,
                _ => FileStatus::Modified,
            };

            let file = DiffFile::new(old_path, new_path, status);
            diff.borrow_mut().files.push(file);
            true
        },
        None,
        Some(&mut |_delta: DiffDelta, hunk: git2::DiffHunk| {
            let mut diff_mut = diff.borrow_mut();
            let file = diff_mut.files.last_mut().unwrap();

            let header = String::from_utf8_lossy(hunk.header()).to_string();
            let diff_hunk = DiffHunk::new(
                header,
                hunk.old_start(),
                hunk.old_lines(),
                hunk.new_start(),
                hunk.new_lines(),
            );
            file.hunks.push(diff_hunk);
            true
        }),
        Some(&mut |_delta: DiffDelta, _hunk: Option<git2::DiffHunk>, line: git2::DiffLine| {
            let mut diff_mut = diff.borrow_mut();
            let file = diff_mut.files.last_mut().unwrap();
            let hunk = file.hunks.last_mut().unwrap();

            let line_type = match line.origin() {
                '+' => {
                    file.additions += 1;
                    LineType::Addition
                }
                '-' => {
                    file.deletions += 1;
                    LineType::Deletion
                }
                ' ' => LineType::Context,
                'F' => LineType::FileHeader,
                'H' => LineType::HunkHeader,
                _ => LineType::Context,
            };

            let content = String::from_utf8_lossy(line.content()).to_string();

            let diff_line = DiffLine::new(
                line_type,
                content,
                line.old_lineno(),
                line.new_lineno(),
            );

            hunk.lines.push(diff_line);
            true
        }),
    )?;

    Ok(diff.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_test_repo_with_diff() -> (TempDir, Repository, Oid) {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        let git_repo = git2::Repository::init(repo_path).unwrap();
        let sig = git2::Signature::now("Test", "test@example.com").unwrap();

        // Create initial commit
        std::fs::write(repo_path.join("test.txt"), "line1\nline2\nline3\n").unwrap();
        let mut index = git_repo.index().unwrap();
        index.add_path(std::path::Path::new("test.txt")).unwrap();
        index.write().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = git_repo.find_tree(tree_id).unwrap();
        git_repo
            .commit(Some("HEAD"), &sig, &sig, "Initial", &tree, &[])
            .unwrap();

        // Create second commit with changes
        std::fs::write(repo_path.join("test.txt"), "line1\nmodified\nline3\nline4\n").unwrap();
        let mut index = git_repo.index().unwrap();
        index.add_path(std::path::Path::new("test.txt")).unwrap();
        index.write().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = git_repo.find_tree(tree_id).unwrap();
        let parent = git_repo.head().unwrap().peel_to_commit().unwrap();
        let commit_id = git_repo
            .commit(Some("HEAD"), &sig, &sig, "Second", &tree, &[&parent])
            .unwrap();

        let repo = Repository::open(repo_path).await.unwrap();
        (temp_dir, repo, commit_id)
    }

    #[tokio::test]
    async fn test_load_commit_diff() {
        let (_temp_dir, repo, commit_id) = create_test_repo_with_diff().await;
        let diff = load_commit_diff(&repo, commit_id).await.unwrap();

        assert_eq!(diff.files.len(), 1);
        assert_eq!(diff.files[0].path(), "test.txt");
        assert_eq!(diff.files[0].status, FileStatus::Modified);
        assert!(diff.files[0].hunks.len() > 0);
    }

    #[tokio::test]
    async fn test_diff_stats() {
        let (_temp_dir, repo, commit_id) = create_test_repo_with_diff().await;
        let diff = load_commit_diff(&repo, commit_id).await.unwrap();

        let (additions, deletions) = diff.total_stats();
        assert!(additions > 0);
        assert!(deletions > 0);
    }

    #[test]
    fn test_diff_line_types() {
        let line = DiffLine::new(LineType::Addition, "+test".to_string(), None, Some(1));
        assert_eq!(line.line_type, LineType::Addition);
        assert_eq!(line.content, "+test");
    }
}
