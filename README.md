# rust-tig

A terminal-based Git interface inspired by [tig](https://jonas.github.io/tig/), written in Rust using ratatui and git2-rs.

## Features

✅ **Main View**: Browse commit history with search functionality
✅ **Diff View**: View color-coded diffs for commits and files
✅ **Status View**: See and manage staged/unstaged/untracked files
✅ **Help**: Built-in help system with keybinding reference
✅ **Async Operations**: Non-blocking UI with async git operations
✅ **Vim-style Navigation**: Familiar keybindings (j/k, g/G, etc.)

## Installation

### Prerequisites

- Rust 1.90 or later
- Git (libgit2 is used internally)

### Build from Source

```bash
cd rust-tig
cargo build --release
```

The binary will be available at `target/release/rust-tig`.

## Usage

Navigate to any git repository and run:

```bash
rust-tig
```

## Keybindings

### Global

- `q` - Quit application or close current view
- `Ctrl+C` - Force quit
- `?` - Show help
- `Esc` - Close current view or exit search mode

### Main View (Commit History)

- `j` / `↓` - Move selection down
- `k` / `↑` - Move selection up
- `g` - Jump to first commit
- `G` - Jump to last commit
- `PageUp` / `PageDown` - Page navigation
- `Enter` - View commit diff
- `/` - Start search (search commit messages)
- `s` - Open status view

### Search Mode

- Type to enter search query
- `Backspace` - Delete character
- `Enter` - Keep search results and exit search mode
- `Esc` - Clear search and exit search mode

### Status View

- `j` / `↓` - Move selection down
- `k` / `↑` - Move selection up
- `g` - Jump to first item
- `G` - Jump to last item
- `PageUp` / `PageDown` - Page navigation
- `Enter` - View file diff
- `u` - Stage/unstage selected file
- `r` - Refresh status

### Diff View

- `j` / `↓` - Scroll down
- `k` / `↑` - Scroll up
- `g` - Jump to top
- `G` - Jump to bottom
- `PageUp` / `PageDown` - Page navigation
- `Esc` - Close diff view

## Architecture

See [ARCHITECTURE.md](./ARCHITECTURE.md) for detailed architecture documentation.

## Development

### Running Tests

```bash
cargo test
```

### Running the Application

```bash
cargo run
```

### Project Structure

```
rust-tig/
├── src/
│   ├── main.rs              # Entry point, tokio runtime
│   ├── git/                 # Git operations
│   │   ├── commit.rs        # Commit data structures
│   │   ├── diff.rs          # Diff loading and parsing
│   │   ├── error.rs         # Git error types
│   │   ├── repository.rs    # Repository wrapper
│   │   ├── status.rs        # Status and staging operations
│   │   └── walker.rs        # Commit history walker
│   ├── ui/                  # Terminal UI
│   │   ├── app.rs           # Application state and rendering
│   │   ├── event.rs         # Event handling loop
│   │   └── terminal.rs      # Terminal initialization
│   └── views/               # View implementations
│       ├── diff_view.rs     # Commit/file diff view
│       ├── help_view.rs     # Help overlay
│       ├── main_view.rs     # Commit history view
│       ├── manager.rs       # View stack management
│       ├── status_view.rs   # Working directory status
│       └── view.rs          # View trait and actions
└── Cargo.toml
```

## Technology Stack

- **ratatui 0.29**: Terminal user interface framework
- **crossterm 0.28**: Cross-platform terminal manipulation
- **git2 0.19**: Rust bindings for libgit2
- **tokio 1.40**: Async runtime for non-blocking operations
- **anyhow 1.0**: Error handling
- **chrono 0.4**: Date/time handling

## Roadmap

### MVP (✅ Completed)

- [x] Main view with commit history
- [x] Diff view for commits and files
- [x] Status view with staging/unstaging
- [x] Search functionality
- [x] Help system
- [x] Async git operations
- [x] View navigation and management

### Future Enhancements

- [ ] Additional views: Stage, Blame, Tree, Blob, Refs, Log, Stash, Grep, Reflog
- [ ] Configuration file support (YAML)
- [ ] Custom keybindings
- [ ] Color scheme customization
- [ ] Mouse support
- [ ] Line staging (interactive staging)
- [ ] Commit creation from UI
- [ ] Branch operations
- [ ] Performance optimizations for large repositories
- [ ] Plugin system

## License

MIT OR Apache-2.0
