# rust-tig - Claude Documentation

This document provides comprehensive information about rust-tig for Claude Code or other AI assistants working on this project.

## Project Overview

**rust-tig** is a terminal-based Git interface inspired by [tig](https://jonas.github.io/tig/), written in Rust using modern async patterns and the ratatui TUI framework.

- **Language**: Rust (Edition 2024)
- **Minimum Rust Version**: 1.90
- **License**: MIT OR Apache-2.0
- **Status**: MVP Complete ✅

## Quick Start

```bash
# Build the project
cargo build --release

# Run tests (58 total: 44 unit + 14 integration)
cargo test

# Run the application (must be in a git repository)
cargo run

# Check release binary size
ls -lh target/release/rust-tig  # ~1.3MB optimized
```

## Project Structure

```
rust-tig/
├── src/
│   ├── main.rs              # Entry point with tokio runtime
│   ├── lib.rs               # Library interface for tests
│   ├── git/                 # Git operations layer
│   │   ├── mod.rs           # Module exports
│   │   ├── commit.rs        # Commit data structures
│   │   ├── diff.rs          # Diff loading and parsing
│   │   ├── error.rs         # Git error types
│   │   ├── repository.rs    # Repository wrapper
│   │   ├── status.rs        # Status and staging
│   │   └── walker.rs        # Commit history walker
│   ├── ui/                  # Terminal UI layer
│   │   ├── mod.rs           # Module exports
│   │   ├── app.rs           # Application state
│   │   ├── event.rs         # Event handling
│   │   └── terminal.rs      # Terminal setup
│   └── views/               # View implementations
│       ├── mod.rs           # Module exports
│       ├── view.rs          # View trait & Action enum
│       ├── manager.rs       # View stack management
│       ├── main_view.rs     # Commit history view
│       ├── diff_view.rs     # Diff display view
│       ├── status_view.rs   # Working directory status
│       └── help_view.rs     # Help overlay
├── tests/
│   └── integration_tests.rs # Integration test suite
├── Cargo.toml               # Dependencies and build config
├── README.md                # User documentation
├── ARCHITECTURE.md          # Architecture deep-dive
└── VIEWS_SPECIFICATION.md   # Future views specification
```

## Core Technologies

| Crate | Version | Purpose |
|-------|---------|---------|
| ratatui | 0.29 | Terminal UI framework |
| crossterm | 0.28 | Cross-platform terminal backend |
| git2 | 0.19 | Rust bindings for libgit2 |
| tokio | 1.40 | Async runtime (multi-threaded) |
| anyhow | 1.0 | Error handling |
| thiserror | 1.0 | Error derive macros |
| chrono | 0.4 | Date/time handling |
| tempfile | 3.10 | Test fixtures (dev-only) |

## Architecture Patterns

### 1. View Trait Pattern

All views implement the `View` trait for polymorphic management:

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

### 2. Action-Based Communication

Views return `Action` enum to request navigation/operations:

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

### 3. Async Git Operations

Git operations run asynchronously to keep UI responsive:

```rust
// Pattern 1: Channel-based async loading
let (tx, rx) = mpsc::unbounded_channel();
tokio::spawn(async move {
    let result = load_data().await;
    tx.send(result).unwrap();
});

// Pattern 2: Check for results in update()
if let Ok(data) = receiver.try_recv() {
    self.process_data(data);
}
```

### 4. Repository Handle Workaround

git2::Repository is not Send/Sync, so we store the path and reopen:

```rust
#[derive(Clone)]
pub struct Repository {
    path: PathBuf,  // Store path, not handle
}

impl Repository {
    pub fn open_git2(&self) -> Result<Git2Repo> {
        Git2Repo::open(&self.path)
    }
}
```

## Implemented Features (MVP)

### Main View
- ✅ Commit history with async loading
- ✅ Search functionality (case-insensitive)
- ✅ Vim-style navigation (j/k/g/G)
- ✅ Page up/down
- ✅ Open commit diff
- ✅ Open status view
- ✅ Reference display (branches/tags)

### Diff View
- ✅ Commit diffs
- ✅ Staged file diffs
- ✅ Unstaged file diffs
- ✅ Color-coded lines (green/red/white)
- ✅ Line numbers
- ✅ Scrolling and navigation
- ✅ File statistics

### Status View
- ✅ Staged/unstaged/untracked/conflicted sections
- ✅ File staging (u key)
- ✅ File unstaging (u key)
- ✅ Refresh (r key)
- ✅ View file diffs
- ✅ Color-coded sections

### Help View
- ✅ Comprehensive keybinding reference
- ✅ Scrollable content
- ✅ Organized by view

## Key Design Decisions

### Edition 2024
**Why**: User explicitly requested it, provides access to latest language features.

### PathBuf Storage
**Why**: git2::Repository is not Send/Sync, so we store path and reopen for each operation.

### Chunked Commit Loading
**Why**: Large repositories need progressive loading to show initial results quickly.

### RefCell in Diff Parsing
**Why**: git2's foreach API requires multiple closures with mutable access, RefCell allows this.

### Thin LTO
**Why**: Full LTO caused out-of-memory during build; thin LTO provides good optimization with less memory.

## Testing

### Test Coverage
- **Unit Tests**: 44 tests across all modules
- **Integration Tests**: 14 end-to-end tests
- **Total**: 58 tests (all passing ✅)

### Test Categories

**Unit Tests** (`src/**/*.rs`):
- Git operations (commit, diff, status, walker)
- View lifecycle and navigation
- Search functionality
- Date formatting
- Error handling

**Integration Tests** (`tests/integration_tests.rs`):
- End-to-end workflows
- Repository operations
- Staging/unstaging
- View activation
- Error scenarios

### Running Tests

```bash
# All tests
cargo test

# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test integration_tests

# Specific test
cargo test test_main_view_search

# With output
cargo test -- --nocapture
```

## Build Optimization

### Release Profile (Cargo.toml)

```toml
[profile.release]
opt-level = "z"     # Optimize for size
lto = "thin"        # Thin LTO (less memory than full)
codegen-units = 1   # Better optimization
strip = true        # Strip symbols
panic = "abort"     # Smaller panic handling
```

**Result**: Binary size reduced to ~1.3MB

### Build Commands

```bash
# Debug build (fast, unoptimized)
cargo build

# Release build (optimized)
cargo build --release

# Clean build directory
cargo clean
```

## Common Development Tasks

### Adding a New View

1. Create view file in `src/views/`
2. Implement `View` trait
3. Add to `src/views/mod.rs` exports
4. Add `ViewType` variant in `view.rs`
5. Handle in `App::handle_action()` (src/ui/app.rs)
6. Add keybinding in appropriate view
7. Write unit tests
8. Update help view with new keybindings

### Adding Git Operation

1. Add function to appropriate module in `src/git/`
2. Make it async with `spawn_blocking` for git2 operations
3. Return `Result<T, GitError>`
4. Add unit tests with tempfile
5. Add integration test if needed

### Debugging

```bash
# Run with Rust backtrace
RUST_BACKTRACE=1 cargo run

# Run with full backtrace
RUST_BACKTRACE=full cargo run

# Check for issues
cargo clippy

# Format code
cargo fmt
```

## Error Handling

### Error Types

```rust
// src/git/error.rs
pub enum GitError {
    Git2(git2::Error),
    IoError(std::io::Error),
    RepoNotFound,
    InvalidDiff,
}
```

### Error Propagation

- Use `?` operator for error propagation
- Convert errors with `map_err` where needed
- Display user-facing errors in views
- Log internal errors to stderr

## Performance Considerations

### Memory
- Views release data when deactivated
- Commit loading is chunked (50 per chunk)
- Diffs are cached per view instance

### Speed
- Async operations prevent UI blocking
- git2 operations in spawn_blocking thread pool
- Efficient rendering with ratatui
- Minimal redraws

### Large Repositories
- Chunked loading works well
- Search is client-side (all commits loaded)
- Future: Add server-side search for very large repos

## Known Limitations

1. **Search**: All commits must be loaded before search works
2. **Diff Size**: Very large diffs (>100k lines) may be slow
3. **Binary Files**: Shown as "Binary file" without hex viewer
4. **Merge Commits**: Shown as regular commits (no special handling)
5. **Submodules**: Not yet supported

## Future Enhancements

See `VIEWS_SPECIFICATION.md` for detailed specs on:
- Stage View (interactive staging)
- Blame View (line-by-line history)
- Tree View (file browser)
- Blob View (file viewer)
- Refs View (branch/tag management)
- Log View (filtered history)
- Stash View (stash management)
- Grep View (content search)
- Reflog View (reference history)

Also planned:
- Configuration file (YAML)
- Custom keybindings
- Color schemes
- Mouse support
- Performance improvements

## Troubleshooting

### Build Issues

**Problem**: LTO causes out-of-memory
**Solution**: Use `lto = "thin"` instead of `lto = true`

**Problem**: Edition 2024 not recognized
**Solution**: Update Rust: `rustup update`

### Runtime Issues

**Problem**: "reference 'refs/heads/master' not found"
**Solution**: Repository is empty (no commits), check with `is_empty()`

**Problem**: Async operations not completing
**Solution**: Call `view.update()` regularly to process channel messages

### Test Issues

**Problem**: Integration tests fail on file operations
**Solution**: Ensure tempfile creates directories correctly

**Problem**: Async tests hang
**Solution**: Use `#[tokio::test]` attribute, not `#[test]`

## Contributing Guidelines

1. **Code Style**: Run `cargo fmt` before committing
2. **Linting**: Fix `cargo clippy` warnings
3. **Tests**: Add tests for new features
4. **Documentation**: Update docs for API changes
5. **Commits**: Use conventional commits (feat:, fix:, docs:, etc.)

## Git Workflow

```bash
# Create feature branch
git checkout -b feature/my-feature

# Make changes and test
cargo test
cargo clippy
cargo fmt

# Commit changes
git add .
git commit -m "feat: add new feature"

# Push and create PR
git push origin feature/my-feature
```

## Resources

- [tig source](https://github.com/jonas/tig) - Original inspiration
- [ratatui docs](https://docs.rs/ratatui) - TUI framework
- [git2-rs docs](https://docs.rs/git2) - Git bindings
- [tokio docs](https://docs.rs/tokio) - Async runtime

## File Locations

- **Main code**: `src/`
- **Tests**: `tests/` and `src/**/*_tests` modules
- **Documentation**: `*.md` files in root
- **Configuration**: `Cargo.toml`
- **Build artifacts**: `target/` (gitignored)

## Important Files

| File | Purpose |
|------|---------|
| `src/main.rs` | Application entry point |
| `src/lib.rs` | Library interface for tests |
| `src/git/repository.rs` | Core git operations |
| `src/views/view.rs` | View trait definition |
| `src/ui/app.rs` | Application state & rendering |
| `Cargo.toml` | Dependencies & build config |
| `README.md` | User documentation |
| `ARCHITECTURE.md` | Technical deep-dive |
| `VIEWS_SPECIFICATION.md` | Future features spec |

## Quick Reference

### Key Keybindings
- `q` - Quit
- `?` - Help
- `s` - Status view
- `/` - Search
- `j/k` - Navigate
- `Enter` - Select/Open

### Common Commands
```bash
cargo build --release    # Build optimized binary
cargo test              # Run all tests
cargo run               # Run in current repo
cargo clippy            # Lint code
cargo fmt               # Format code
```

### Test Patterns
```rust
#[tokio::test]
async fn test_something() {
    let (temp_dir, repo) = create_test_repo().await;
    // ... test code
}
```

## Version Information

- **Current Version**: 0.1.0
- **Rust Edition**: 2024
- **Minimum Rust**: 1.90
- **Binary Size**: ~1.3MB (release, stripped)
- **Test Count**: 58 (44 unit + 14 integration)

## Status Summary

✅ **Complete**:
- Main view with search
- Diff view (commit, staged, unstaged)
- Status view with staging/unstaging
- Help view
- Async git operations
- View management
- Error handling
- Tests (unit + integration)
- Documentation
- Build optimization

🚧 **Future**:
- Additional views (9 specified)
- Configuration system
- Custom keybindings
- Mouse support
- Performance optimizations

---

Last Updated: 2025-10-07
Project Status: MVP Complete ✅
