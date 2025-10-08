use super::{commit::Commit, error::Result, repository::Repository};
use git2::{Oid, Sort};
use std::collections::HashMap;
use tokio::sync::mpsc;

/// Async commit walker that loads commits in chunks
pub struct CommitWalker {
    repo: Repository,
    chunk_size: usize,
}

impl CommitWalker {
    /// Create a new commit walker
    pub fn new(repo: Repository) -> Self {
        Self {
            repo,
            chunk_size: 100,
        }
    }

    /// Set the chunk size for loading commits
    pub fn with_chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = size;
        self
    }

    /// Walk commits starting from HEAD and send them through the channel
    pub async fn walk(&self, tx: mpsc::UnboundedSender<Vec<Commit>>) -> Result<()> {
        let repo = self.repo.clone();
        let chunk_size = self.chunk_size;

        tokio::task::spawn_blocking(move || {
            let git_repo = repo.open_git2()?;

            // Get all references to populate commit refs
            let mut refs_map: HashMap<Oid, Vec<String>> = HashMap::new();
            if let Ok(references) = git_repo.references() {
                for reference in references.flatten() {
                    if let (Some(name), Some(target)) = (reference.shorthand(), reference.target()) {
                        refs_map
                            .entry(target)
                            .or_insert_with(Vec::new)
                            .push(name.to_string());
                    }
                }
            }

            // Set up the revwalk
            let mut revwalk = git_repo.revwalk()?;
            revwalk.set_sorting(Sort::TIME | Sort::TOPOLOGICAL)?;
            revwalk.push_head()?;

            let mut commits = Vec::with_capacity(chunk_size);

            for oid in revwalk {
                let oid = oid?;
                let git_commit = git_repo.find_commit(oid)?;

                let mut commit = Commit::from_git2(&git_commit)?;

                // Add refs if this commit has any
                if let Some(refs) = refs_map.get(&oid) {
                    commit.refs = refs.clone();
                }

                commits.push(commit);

                // Send a chunk when we reach the chunk size
                if commits.len() >= chunk_size {
                    if tx.send(commits.clone()).is_err() {
                        break; // Receiver dropped
                    }
                    commits.clear();
                }
            }

            // Send remaining commits
            if !commits.is_empty() {
                let _ = tx.send(commits);
            }

            Ok::<(), super::error::GitError>(())
        })
        .await
        .map_err(|_| super::error::GitError::RepoNotFound)??;

        Ok(())
    }

    /// Load all commits at once (for small repositories)
    pub async fn load_all(&self) -> Result<Vec<Commit>> {
        let repo = self.repo.clone();

        tokio::task::spawn_blocking(move || {
            let git_repo = repo.open_git2()?;

            // Get all references
            let mut refs_map: HashMap<Oid, Vec<String>> = HashMap::new();
            if let Ok(references) = git_repo.references() {
                for reference in references.flatten() {
                    if let (Some(name), Some(target)) = (reference.shorthand(), reference.target()) {
                        refs_map
                            .entry(target)
                            .or_insert_with(Vec::new)
                            .push(name.to_string());
                    }
                }
            }

            let mut revwalk = git_repo.revwalk()?;
            revwalk.set_sorting(Sort::TIME | Sort::TOPOLOGICAL)?;
            revwalk.push_head()?;

            let mut commits = Vec::new();

            for oid in revwalk {
                let oid = oid?;
                let git_commit = git_repo.find_commit(oid)?;
                let mut commit = Commit::from_git2(&git_commit)?;

                if let Some(refs) = refs_map.get(&oid) {
                    commit.refs = refs.clone();
                }

                commits.push(commit);
            }

            Ok(commits)
        })
        .await
        .map_err(|_| super::error::GitError::RepoNotFound)?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_test_repo_with_commits() -> (TempDir, Repository) {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        let git_repo = git2::Repository::init(repo_path).unwrap();
        let sig = git2::Signature::now("Test User", "test@example.com").unwrap();

        // Create multiple commits
        for i in 0..5 {
            let tree_id = {
                let mut index = git_repo.index().unwrap();
                // Create a file to make commits different
                let path = repo_path.join(format!("file{}.txt", i));
                std::fs::write(&path, format!("content {}", i)).unwrap();
                index.add_path(std::path::Path::new(&format!("file{}.txt", i))).unwrap();
                index.write().unwrap();
                index.write_tree().unwrap()
            };
            let tree = git_repo.find_tree(tree_id).unwrap();
            let message = format!("Commit {}", i);

            if i == 0 {
                git_repo
                    .commit(Some("HEAD"), &sig, &sig, &message, &tree, &[])
                    .unwrap();
            } else {
                let parent = git_repo.head().unwrap().peel_to_commit().unwrap();
                git_repo
                    .commit(Some("HEAD"), &sig, &sig, &message, &tree, &[&parent])
                    .unwrap();
            };
        }

        let our_repo = Repository::open(repo_path).await.unwrap();
        (temp_dir, our_repo)
    }

    #[tokio::test]
    async fn test_load_all_commits() {
        let (_temp_dir, repo) = create_test_repo_with_commits().await;
        let walker = CommitWalker::new(repo);

        let commits = walker.load_all().await.unwrap();
        assert_eq!(commits.len(), 5);

        // Verify commits are in reverse chronological order
        for i in 0..5 {
            assert!(commits[i].summary.contains(&format!("Commit {}", 4 - i)));
        }
    }

    #[tokio::test]
    async fn test_walk_commits_in_chunks() {
        let (_temp_dir, repo) = create_test_repo_with_commits().await;
        let walker = CommitWalker::new(repo).with_chunk_size(2);

        let (tx, mut rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            walker.walk(tx).await.unwrap();
        });

        let mut total_commits = 0;
        while let Some(chunk) = rx.recv().await {
            assert!(chunk.len() <= 2);
            total_commits += chunk.len();
        }

        assert_eq!(total_commits, 5);
    }

    #[tokio::test]
    async fn test_commit_has_refs() {
        let (_temp_dir, repo) = create_test_repo_with_commits().await;
        let walker = CommitWalker::new(repo);

        let commits = walker.load_all().await.unwrap();

        // The most recent commit should have HEAD and the branch ref
        assert!(!commits[0].refs.is_empty());
    }
}
