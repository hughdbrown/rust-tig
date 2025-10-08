# rust-tig Architecture

This document describes the architecture and design decisions for rust-tig.

## Overview

rust-tig is built using a layered architecture with clear separation of concerns:

1. **UI Layer** (`src/ui/`): Terminal management and event handling
2. **View Layer** (`src/views/`): View implementations and rendering
3. **Git Layer** (`src/git/`): Git operations using libgit2

## Core Design Patterns

### View Trait Pattern

All views implement the `View` trait which defines a standard interface:

```rust
pub trait View {
    fn handle_key(&mut self, key: KeyEvent) -> Result<Action>;
    fn update(&mut self) -> Result<()>;
    fn draw(&mut self, frame: &mut Frame, area: Rect);
    fn title(&self) -> &str;
    fn on_activate(&mut self) -> Result<()>;
    fn on_deactivate(&mut self) -> Result<()>;
}
```

This pattern enables:
- **Polymorphism**: Views can be managed generically through the ViewManager
- **Lifecycle Management**: Views can react to activation/deactivation
- **Separation of Concerns**: Each view manages its own state and rendering

### Action-Based Navigation

Views communicate with the application through the `Action` enum:

```rust
pub enum Action {
    None,
    Quit,
    PushView(ViewType),
    PopView,
    OpenDiff { repo, commit_id, summary },
    OpenStagedDiff { repo, path },
    OpenUnstagedDiff { repo, path },
}
```

This pattern:
- **Decouples** views from the application layer
- **Centralizes** navigation logic in the application
- **Enables** view composition and reuse

### Async Git Operations

Git operations run asynchronously to keep the UI responsive:

```rust
// Load commits in background
tokio::spawn(async move {
    let result = walker.walk(tx).await;
    tx.send(result)?;
});

// Update UI when results arrive
if let Ok(chunk) = receiver.try_recv() {
    self.commits.extend(chunk);
}
```

Key techniques:
- **spawn_blocking**: Git2 operations run in blocking thread pool
- **Channels**: mpsc channels communicate between async tasks and UI
- **Non-blocking Updates**: UI checks for new data each frame without blocking

### Repository Handle Workaround

git2::Repository is not Send/Sync, so we use a workaround:

```rust
#[derive(Clone)]
pub struct Repository {
    path: PathBuf,  // Store path instead of Repository handle
}

impl Repository {
    pub fn open_git2(&self) -> Result<Git2Repo> {
        Git2Repo::open(&self.path)  // Reopen for each operation
    }
}
```

This allows Repository to be:
- **Cloneable**: Can be passed to different views and tasks
- **Send + Sync**: Can be sent across threads
- **Safe**: Each operation gets its own repository handle

## Module Architecture

### src/git/

Git operations layer using git2-rs (libgit2 bindings).

**commit.rs**
- Commit data structure
- Relative date formatting
- Short hash generation

**diff.rs**
- Diff data structures (DiffLine, DiffHunk, DiffFile, Diff)
- Three diff types: commit diffs, staged file diffs, unstaged file diffs
- Color-coded line types (Addition, Deletion, Context)

**error.rs**
- Git error types using thiserror
- Conversion from git2::Error

**repository.rs**
- Repository wrapper with async methods
- Repository discovery
- Branch information
- Path-based handle workaround

**status.rs**
- Working directory status
- File staging/unstaging operations
- Status categorization (staged, unstaged, untracked, conflicted)

**walker.rs**
- Async commit history traversal
- Chunked loading for performance
- Reference (branch/tag) association

### src/ui/

Terminal and event management layer.

**app.rs**
- Main application state
- View manager integration
- Action handling
- Status bar rendering

**event.rs**
- Event loop (keyboard, mouse, resize, tick)
- 100ms tick for regular updates
- Non-blocking event polling

**terminal.rs**
- Terminal initialization and cleanup
- Panic handler to restore terminal
- Alternate screen buffer

### src/views/

View implementations using ratatui.

**view.rs**
- View trait definition
- Action enum
- ViewType enum

**manager.rs**
- View stack management
- View lifecycle (activation/deactivation)
- Current view delegation

**main_view.rs**
- Commit history display
- Search functionality
- Chunk-based commit loading
- Table rendering with columns: hash, date, author, message

**diff_view.rs**
- Diff rendering with color coding
- Three sources: commit, staged file, unstaged file
- Scrolling and navigation
- Syntax-highlighted output

**status_view.rs**
- Working directory status display
- Sections: staged, unstaged, untracked, conflicted
- File staging/unstaging
- Integration with diff view

**help_view.rs**
- Help overlay
- Keybinding reference
- Scrollable content

## Async Architecture

### Event Loop

```
┌─────────────┐
│   main()    │
│  tokio::main│
└──────┬──────┘
       │
       ▼
┌─────────────────┐
│  Event Handler  │
│  (crossterm)    │
└────────┬────────┘
         │
         ▼
  ┌──────────────┐
  │  App::handle │
  │    _event    │
  └──────┬───────┘
         │
         ▼
  ┌──────────────┐
  │   View::     │
  │ handle_key   │
  └──────┬───────┘
         │
         ▼
  ┌──────────────┐
  │   Action     │
  └──────────────┘
```

### Async Data Flow

```
┌─────────────┐
│  View calls │
│start_loading│
└──────┬──────┘
       │
       ▼
┌────────────────┐
│ tokio::spawn   │
│ (async task)   │
└────────┬───────┘
         │
         ▼
┌────────────────────┐
│ spawn_blocking     │
│ (git2 operations)  │
└────────┬───────────┘
         │
         ▼
┌────────────────┐
│  Send result   │
│  via channel   │
└────────┬───────┘
         │
         ▼
┌────────────────┐
│ View::update   │
│  try_recv()    │
└────────┬───────┘
         │
         ▼
┌────────────────┐
│  Update UI     │
│  state         │
└────────────────┘
```

## Data Structures

### Commit

```rust
pub struct Commit {
    pub id: Oid,
    pub short_id: String,  // 7-char hash
    pub author: String,
    pub author_email: String,
    pub date: DateTime<Utc>,
    pub summary: String,
    pub message: String,
    pub refs: Vec<String>,  // Branch/tag names
}
```

### Diff

```rust
pub struct Diff {
    pub files: Vec<DiffFile>,
}

pub struct DiffFile {
    pub old_path: Option<String>,
    pub new_path: Option<String>,
    pub status: FileStatus,
    pub is_binary: bool,
    pub hunks: Vec<DiffHunk>,
}

pub struct DiffHunk {
    pub header: String,
    pub lines: Vec<DiffLine>,
}

pub struct DiffLine {
    pub line_type: LineType,
    pub old_lineno: Option<u32>,
    pub new_lineno: Option<u32>,
    pub content: String,
}
```

### Status

```rust
pub struct Status {
    pub staged: Vec<StatusEntry>,
    pub unstaged: Vec<StatusEntry>,
    pub untracked: Vec<StatusEntry>,
    pub conflicted: Vec<StatusEntry>,
}

pub struct StatusEntry {
    pub path: String,
    pub status: EntryStatus,
    pub index_to_workdir: bool,
}
```

## Performance Considerations

### Chunked Loading

Commits are loaded in chunks (default 50) to:
- Display initial results quickly
- Allow scrolling while loading
- Handle large repositories efficiently

### Non-blocking Operations

All Git operations run in background tasks:
- UI remains responsive during operations
- Loading indicators show progress
- Cancellation is possible (close view)

### Efficient Rendering

- Only visible items are rendered (ratatui handles this)
- Scrollbar state calculated efficiently
- Diff lines are pre-rendered and cached

## Error Handling

### Error Types

```rust
pub enum GitError {
    Git2(git2::Error),
    IoError(std::io::Error),
    RepoNotFound,
    InvalidCommit(String),
    InvalidDiff,
    RefNotFound(String),
}
```

### Error Propagation

- Most functions return `Result<T, Error>`
- Errors propagate using `?` operator
- User-facing errors shown in UI
- Background errors logged to stderr

## Testing Strategy

### Unit Tests

Each module has unit tests:
- `git/*`: Test git operations with temporary repositories
- `views/*`: Test view logic and state management
- Async operations tested with `#[tokio::test]`

### Test Fixtures

Tests use `tempfile` crate for temporary repositories:

```rust
async fn create_test_repo() -> (TempDir, Repository) {
    let temp_dir = TempDir::new().unwrap();
    let git_repo = git2::Repository::init(temp_dir.path()).unwrap();
    // ... setup test repo
    let repo = Repository::open(temp_dir.path()).await.unwrap();
    (temp_dir, repo)
}
```

## Future Architecture Considerations

### Configuration System

Planned YAML-based configuration:

```yaml
keybindings:
  global:
    quit: q
    help: "?"
  main:
    search: /
    status: s
colors:
  added: green
  deleted: red
  modified: yellow
```

### Plugin System

Potential plugin architecture:
- Custom views
- Custom keybindings
- Custom data sources
- Event hooks

### Performance Optimizations

- Virtualized rendering for very large diffs
- Index caching for faster status updates
- Parallel commit loading
- Incremental diff parsing

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| ratatui | 0.29 | Terminal UI framework |
| crossterm | 0.28 | Terminal backend |
| git2 | 0.19 | Git operations (libgit2) |
| tokio | 1.40 | Async runtime |
| anyhow | 1.0 | Error handling |
| thiserror | 1.0 | Error derive macros |
| chrono | 0.4 | Date/time |
| tempfile | (dev) | Test fixtures |

## Design Decisions

### Why ratatui over other TUI frameworks?

- **Production-ready**: Mature and well-maintained
- **Performance**: Efficient rendering with minimal redraws
- **Flexibility**: Low-level control when needed
- **Community**: Active community and good documentation

### Why git2-rs over spawning git commands?

- **Performance**: Direct libgit2 calls are faster
- **Control**: Fine-grained control over operations
- **Async-friendly**: Easier to integrate with tokio
- **Type-safe**: Rust bindings provide type safety

### Why Edition 2024?

- **Latest features**: Access to newest Rust language features
- **Future-proof**: Prepared for upcoming improvements
- **User request**: Explicitly requested by project requirements

### Why PathBuf instead of Repository handle?

- **Send + Sync**: Enables passing across threads
- **Cloneable**: Simplifies sharing between views
- **Works around libgit2 limitation**: git2::Repository not thread-safe
