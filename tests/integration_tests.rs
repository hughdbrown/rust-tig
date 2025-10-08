use rust_tig::config::{ColorScheme, Config};
use rust_tig::git::{CommitWalker, Repository};
use rust_tig::views::{DiffView, MainView, StatusView, View};
use std::fs;
use tempfile::TempDir;

/// Create a test color scheme
fn test_color_scheme() -> ColorScheme {
    ColorScheme::from_config(&Config::default().colors)
}

/// Create a test repository with multiple commits
async fn create_test_repo_with_history() -> (TempDir, Repository) {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    let git_repo = git2::Repository::init(repo_path).unwrap();
    let sig = git2::Signature::now("Test User", "test@example.com").unwrap();

    // Create initial commit
    fs::write(repo_path.join("file1.txt"), "Initial content\n").unwrap();
    let mut index = git_repo.index().unwrap();
    index.add_path(std::path::Path::new("file1.txt")).unwrap();
    index.write().unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = git_repo.find_tree(tree_id).unwrap();
    git_repo
        .commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
        .unwrap();

    // Create second commit
    fs::write(repo_path.join("file2.txt"), "Second file\n").unwrap();
    let mut index = git_repo.index().unwrap();
    index.add_path(std::path::Path::new("file2.txt")).unwrap();
    index.write().unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = git_repo.find_tree(tree_id).unwrap();
    let parent = git_repo.head().unwrap().peel_to_commit().unwrap();
    git_repo
        .commit(
            Some("HEAD"),
            &sig,
            &sig,
            "Add second file",
            &tree,
            &[&parent],
        )
        .unwrap();

    // Create third commit
    fs::write(repo_path.join("file1.txt"), "Modified content\n").unwrap();
    let mut index = git_repo.index().unwrap();
    index.add_path(std::path::Path::new("file1.txt")).unwrap();
    index.write().unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = git_repo.find_tree(tree_id).unwrap();
    let parent = git_repo.head().unwrap().peel_to_commit().unwrap();
    git_repo
        .commit(
            Some("HEAD"),
            &sig,
            &sig,
            "Modify first file",
            &tree,
            &[&parent],
        )
        .unwrap();

    let repo = Repository::open(repo_path).await.unwrap();
    (temp_dir, repo)
}

#[tokio::test]
async fn test_repository_discovery_and_branch() {
    let (_temp_dir, repo) = create_test_repo_with_history().await;

    // Test current branch
    let branch = repo.current_branch().await.unwrap();
    assert!(branch.is_some());
    assert_eq!(branch.unwrap(), "master");
}

#[tokio::test]
async fn test_commit_walker_loads_all_commits() {
    let (_temp_dir, repo) = create_test_repo_with_history().await;

    let walker = CommitWalker::new(repo).with_chunk_size(10);
    let commits = walker.load_all().await.unwrap();

    assert_eq!(commits.len(), 3);
    assert_eq!(commits[0].summary, "Modify first file");
    assert_eq!(commits[1].summary, "Add second file");
    assert_eq!(commits[2].summary, "Initial commit");
}

#[tokio::test]
async fn test_commit_walker_chunked_loading() {
    let (_temp_dir, repo) = create_test_repo_with_history().await;

    let walker = CommitWalker::new(repo).with_chunk_size(1);
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    tokio::spawn(async move {
        walker.walk(tx).await.unwrap();
    });

    let mut total_commits = 0;
    while let Some(chunk) = rx.recv().await {
        assert!(!chunk.is_empty());
        assert!(chunk.len() <= 1); // Chunk size is 1
        total_commits += chunk.len();
    }

    assert_eq!(total_commits, 3);
}

#[tokio::test]
async fn test_diff_loading_for_commit() {
    let (_temp_dir, repo) = create_test_repo_with_history().await;

    let walker = CommitWalker::new(repo.clone());
    let commits = walker.load_all().await.unwrap();

    // Load diff for the first commit (most recent)
    let commit = &commits[0];
    let diff = rust_tig::git::diff::load_commit_diff(&repo, commit.id)
        .await
        .unwrap();

    assert!(!diff.files.is_empty());
    assert_eq!(diff.files.len(), 1);
    assert_eq!(diff.files[0].new_path, Some("file1.txt".to_string()));
}

#[tokio::test]
async fn test_status_with_changes() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    let git_repo = git2::Repository::init(repo_path).unwrap();
    let sig = git2::Signature::now("Test", "test@example.com").unwrap();

    // Create initial commit
    fs::write(repo_path.join("existing.txt"), "existing\n").unwrap();
    let mut index = git_repo.index().unwrap();
    index.add_path(std::path::Path::new("existing.txt")).unwrap();
    index.write().unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = git_repo.find_tree(tree_id).unwrap();
    git_repo
        .commit(Some("HEAD"), &sig, &sig, "Initial", &tree, &[])
        .unwrap();

    // Create staged file
    fs::write(repo_path.join("staged.txt"), "staged\n").unwrap();
    let mut index = git_repo.index().unwrap();
    index.add_path(std::path::Path::new("staged.txt")).unwrap();
    index.write().unwrap();

    // Create unstaged change
    fs::write(repo_path.join("existing.txt"), "modified\n").unwrap();

    // Create untracked file
    fs::write(repo_path.join("untracked.txt"), "untracked\n").unwrap();

    let repo = Repository::open(repo_path).await.unwrap();
    let status = rust_tig::git::status::load_status(&repo).await.unwrap();

    assert!(!status.staged.is_empty());
    assert!(!status.unstaged.is_empty());
    assert!(!status.untracked.is_empty());
    assert_eq!(status.total_count(), 3);
}

#[tokio::test]
async fn test_staging_and_unstaging_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    let git_repo = git2::Repository::init(repo_path).unwrap();
    let sig = git2::Signature::now("Test", "test@example.com").unwrap();

    // Create initial commit
    fs::write(repo_path.join("file.txt"), "content\n").unwrap();
    let mut index = git_repo.index().unwrap();
    index.add_path(std::path::Path::new("file.txt")).unwrap();
    index.write().unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = git_repo.find_tree(tree_id).unwrap();
    git_repo
        .commit(Some("HEAD"), &sig, &sig, "Initial", &tree, &[])
        .unwrap();

    // Create untracked file
    fs::write(repo_path.join("new.txt"), "new file\n").unwrap();

    let repo = Repository::open(repo_path).await.unwrap();

    // Verify file is untracked
    let status = rust_tig::git::status::load_status(&repo).await.unwrap();
    assert_eq!(status.untracked.len(), 1);
    assert_eq!(status.staged.len(), 0);

    // Stage the file
    rust_tig::git::status::stage_file(&repo, "new.txt".to_string())
        .await
        .unwrap();

    // Verify file is staged
    let status = rust_tig::git::status::load_status(&repo).await.unwrap();
    assert_eq!(status.untracked.len(), 0);
    assert_eq!(status.staged.len(), 1);

    // Unstage the file
    rust_tig::git::status::unstage_file(&repo, "new.txt".to_string())
        .await
        .unwrap();

    // Verify file is untracked again
    let status = rust_tig::git::status::load_status(&repo).await.unwrap();
    assert_eq!(status.untracked.len(), 1);
    assert_eq!(status.staged.len(), 0);
}

#[tokio::test]
async fn test_main_view_lifecycle() {
    let (_temp_dir, repo) = create_test_repo_with_history().await;
    let mut view = MainView::new(repo, test_color_scheme());

    // Test activation
    view.on_activate().unwrap();

    // Give time for async loading
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Update to process loaded commits
    view.update().unwrap();

    // Should have loaded commits by now
    assert_eq!(view.title(), "Main");
}

#[tokio::test]
async fn test_status_view_lifecycle() {
    let (_temp_dir, repo) = create_test_repo_with_history().await;
    let mut view = StatusView::new(repo, test_color_scheme());

    // Test activation
    view.on_activate().unwrap();

    // Give time for async loading
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Update to process status
    view.update().unwrap();

    assert_eq!(view.title(), "Status");
}

#[tokio::test]
async fn test_diff_view_lifecycle() {
    let (_temp_dir, repo) = create_test_repo_with_history().await;

    let walker = CommitWalker::new(repo.clone());
    let commits = walker.load_all().await.unwrap();
    let commit = &commits[0];

    let mut view = DiffView::new(repo, commit.id, commit.summary.clone(), test_color_scheme());

    // Test activation
    view.on_activate().unwrap();

    // Give time for async loading
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Update to process diff
    view.update().unwrap();

    assert_eq!(view.title(), "Diff");
}

#[tokio::test]
async fn test_staged_and_unstaged_diffs() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    let git_repo = git2::Repository::init(repo_path).unwrap();
    let sig = git2::Signature::now("Test", "test@example.com").unwrap();

    // Create initial commit
    fs::write(repo_path.join("file.txt"), "line1\n").unwrap();
    let mut index = git_repo.index().unwrap();
    index.add_path(std::path::Path::new("file.txt")).unwrap();
    index.write().unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = git_repo.find_tree(tree_id).unwrap();
    git_repo
        .commit(Some("HEAD"), &sig, &sig, "Initial", &tree, &[])
        .unwrap();

    // Make a staged change
    fs::write(repo_path.join("file.txt"), "line1\nline2\n").unwrap();
    let mut index = git_repo.index().unwrap();
    index.add_path(std::path::Path::new("file.txt")).unwrap();
    index.write().unwrap();

    // Make an unstaged change
    fs::write(repo_path.join("file.txt"), "line1\nline2\nline3\n").unwrap();

    let repo = Repository::open(repo_path).await.unwrap();

    // Load staged diff
    let staged_diff = rust_tig::git::diff::load_staged_diff(&repo, Some("file.txt".to_string()))
        .await
        .unwrap();
    assert!(!staged_diff.files.is_empty());

    // Load unstaged diff
    let unstaged_diff =
        rust_tig::git::diff::load_unstaged_diff(&repo, Some("file.txt".to_string()))
            .await
            .unwrap();
    assert!(!unstaged_diff.files.is_empty());
}

#[tokio::test]
async fn test_error_handling_invalid_repo() {
    let temp_dir = TempDir::new().unwrap();
    let non_repo_path = temp_dir.path();

    // Try to open a directory that's not a git repo
    let result = Repository::open(non_repo_path).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_error_handling_nonexistent_path() {
    let result = Repository::open("/nonexistent/path/to/repo").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_commit_with_no_parents() {
    let (_temp_dir, repo) = create_test_repo_with_history().await;

    let walker = CommitWalker::new(repo.clone());
    let commits = walker.load_all().await.unwrap();

    // Last commit should be the initial commit with no parents
    let initial_commit = &commits[commits.len() - 1];
    assert_eq!(initial_commit.summary, "Initial commit");

    // Should be able to load diff for initial commit
    let diff = rust_tig::git::diff::load_commit_diff(&repo, initial_commit.id)
        .await
        .unwrap();
    assert!(!diff.files.is_empty());
}

#[tokio::test]
async fn test_repository_with_no_commits() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    git2::Repository::init(repo_path).unwrap();

    let repo = Repository::open(repo_path).await.unwrap();

    // Check if repository is empty
    let is_empty = repo.is_empty().await.unwrap();
    assert!(is_empty);

    // CommitWalker will fail on empty repo (no HEAD), which is expected
    // In the real app, we'd check is_empty() before trying to walk commits
}
