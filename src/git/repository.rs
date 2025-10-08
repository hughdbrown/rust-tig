use super::error::{GitError, Result};
use git2::Repository as Git2Repo;
use std::path::{Path, PathBuf};

/// Async wrapper around git2::Repository
///
/// Note: git2::Repository is not Send/Sync, so we store the path
/// and open a fresh repository handle in each async operation
#[derive(Clone, Debug, PartialEq)]
pub struct Repository {
    path: PathBuf,
}

impl Repository {
    /// Open a repository from the given path
    pub async fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let path_clone = path.clone();

        // Verify we can open it
        tokio::task::spawn_blocking(move || Git2Repo::open(path_clone))
            .await
            .map_err(|_| GitError::RepoNotFound)??;

        Ok(Self { path })
    }

    /// Discover and open a repository starting from the current directory
    pub async fn discover() -> Result<Self> {
        Self::discover_from(std::env::current_dir()?).await
    }

    /// Discover and open a repository starting from a specific path
    pub async fn discover_from<P: AsRef<Path>>(start_path: P) -> Result<Self> {
        let start_path = start_path.as_ref().to_path_buf();

        let repo_path = tokio::task::spawn_blocking(move || {
            Git2Repo::discover(&start_path)
        })
        .await
        .map_err(|_| GitError::NotARepo)??;

        Self::open(repo_path.path()).await
    }

    /// Get the path to the repository
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Get the current HEAD reference name (e.g., "refs/heads/main")
    pub async fn head_name(&self) -> Result<Option<String>> {
        let path = self.path.clone();

        tokio::task::spawn_blocking(move || {
            let repo = Git2Repo::open(path)?;
            let head = repo.head()?;
            Ok(head.name().map(|s| s.to_string()))
        })
        .await
        .map_err(|_| GitError::RepoNotFound)?
    }

    /// Get the short name of the current branch (e.g., "main" instead of "refs/heads/main")
    pub async fn current_branch(&self) -> Result<Option<String>> {
        let path = self.path.clone();

        tokio::task::spawn_blocking(move || {
            let repo = Git2Repo::open(path)?;
            let head = repo.head()?;
            if let Some(name) = head.shorthand() {
                Ok(Some(name.to_string()))
            } else {
                Ok(None)
            }
        })
        .await
        .map_err(|_| GitError::RepoNotFound)?
    }

    /// Check if the repository is empty (no commits)
    pub async fn is_empty(&self) -> Result<bool> {
        let path = self.path.clone();

        tokio::task::spawn_blocking(move || {
            let repo = Git2Repo::open(path)?;
            Ok(repo.is_empty()?)
        })
        .await
        .map_err(|_| GitError::RepoNotFound)?
    }

    /// Get the workdir path
    pub async fn workdir(&self) -> Result<Option<PathBuf>> {
        let path = self.path.clone();

        tokio::task::spawn_blocking(move || {
            let repo = Git2Repo::open(path)?;
            Ok(repo.workdir().map(|p| p.to_path_buf()))
        })
        .await
        .map_err(|_| GitError::RepoNotFound)?
    }

    /// Open a git2::Repository for synchronous operations
    /// This is useful for operations that need direct access to the repository
    pub fn open_git2(&self) -> Result<Git2Repo> {
        Ok(Git2Repo::open(&self.path)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_test_repo() -> (TempDir, Repository) {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        // Create a git repository
        let repo = Git2Repo::init(repo_path).unwrap();

        // Create an initial commit
        let sig = git2::Signature::now("Test User", "test@example.com").unwrap();
        let tree_id = {
            let mut index = repo.index().unwrap();
            index.write_tree().unwrap()
        };
        let tree = repo.find_tree(tree_id).unwrap();
        repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            "Initial commit",
            &tree,
            &[],
        ).unwrap();

        let our_repo = Repository::open(repo_path).await.unwrap();
        (temp_dir, our_repo)
    }

    #[tokio::test]
    async fn test_open_repository() {
        let (_temp_dir, repo) = create_test_repo().await;
        assert!(repo.path().exists());
    }

    #[tokio::test]
    async fn test_current_branch() {
        let (_temp_dir, repo) = create_test_repo().await;
        let branch = repo.current_branch().await.unwrap();
        assert!(branch.is_some());
        // Default branch is usually "master" or "main"
        let branch_name = branch.unwrap();
        assert!(branch_name == "master" || branch_name == "main");
    }

    #[tokio::test]
    async fn test_is_empty() {
        let (_temp_dir, repo) = create_test_repo().await;
        // We created a commit, so repo should not be empty
        let is_empty = repo.is_empty().await.unwrap();
        assert!(!is_empty);
    }

    #[tokio::test]
    async fn test_workdir() {
        let (_temp_dir, repo) = create_test_repo().await;
        let workdir = repo.workdir().await.unwrap();
        assert!(workdir.is_some());
    }

    #[tokio::test]
    async fn test_discover() {
        let (temp_dir, _repo) = create_test_repo().await;

        // Create a subdirectory and try to discover from there
        let subdir = temp_dir.path().join("subdir");
        std::fs::create_dir(&subdir).unwrap();

        let discovered = Repository::discover_from(&subdir).await;
        assert!(discovered.is_ok());
    }

    #[tokio::test]
    async fn test_open_nonexistent() {
        let result = Repository::open("/nonexistent/path").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_open_git2() {
        let (_temp_dir, repo) = create_test_repo().await;
        let git2_repo = repo.open_git2();
        assert!(git2_repo.is_ok());
    }
}
