# Views Specification

This document provides detailed specifications for views not yet implemented in rust-tig. These specifications serve as a roadmap for future development.

## Implemented Views

- ✅ **Main View**: Commit history
- ✅ **Diff View**: Commit and file diffs
- ✅ **Status View**: Working directory status
- ✅ **Help View**: Keybinding reference

## Views to Implement

### 1. Stage View (Interactive Staging)

**Purpose**: Allow line-by-line staging of changes (similar to `git add -p`).

**Features**:
- Display hunks from unstaged changes
- Allow staging/unstaging individual hunks
- Allow splitting hunks for fine-grained control
- Navigate between hunks with j/k
- Preview staged changes

**Keybindings**:
- `j/k` - Navigate hunks
- `s` - Stage current hunk
- `u` - Unstage current hunk
- `a` - Stage all hunks in file
- `d` - Discard current hunk
- `Enter` - Toggle hunk
- `/` - Split hunk (if possible)
- `q/Esc` - Close view

**Data Structures**:
```rust
pub struct StageView {
    repo: Repository,
    hunks: Vec<Hunk>,
    selected: usize,
    staged_hunks: HashSet<HunkId>,
}

pub struct Hunk {
    id: HunkId,
    file_path: String,
    old_start: u32,
    old_lines: u32,
    new_start: u32,
    new_lines: u32,
    lines: Vec<DiffLine>,
    is_staged: bool,
}
```

**Implementation Notes**:
- Use git2's apply_to_tree for staging hunks
- Track which hunks are staged separately
- Refresh after each stage/unstage operation
- Show visual indicator for staged hunks

### 2. Blame View

**Purpose**: Show line-by-line authorship and commit information for a file.

**Features**:
- Display each line with commit hash, author, date
- Color-code by age (recent = bright, old = dim)
- Navigate to commit that introduced line
- Search for specific author or commit
- Show full commit message on hover/expand

**Keybindings**:
- `j/k` - Navigate lines
- `g/G` - Jump to top/bottom
- `Enter` - Open commit diff for selected line
- `b` - Blame parent commit (go back in history)
- `/` - Search
- `q/Esc` - Close view

**Data Structures**:
```rust
pub struct BlameView {
    repo: Repository,
    file_path: String,
    lines: Vec<BlameLine>,
    scroll_offset: usize,
    selected: usize,
}

pub struct BlameLine {
    line_number: u32,
    commit_id: Oid,
    commit_short_id: String,
    author: String,
    date: DateTime<Utc>,
    content: String,
    is_boundary: bool,
}
```

**Implementation Notes**:
- Use git2::Blame::file()
- Cache blame information for performance
- Use color gradient based on commit age
- Support "blame parent" to trace history

### 3. Tree View

**Purpose**: Browse repository file tree at specific commit/ref.

**Features**:
- Display directory structure
- Expand/collapse directories
- Show file sizes
- Navigate to blob view for files
- Show git attributes (executable, symlink)
- Search for files by name

**Keybindings**:
- `j/k` - Navigate items
- `Enter` - Open file/expand directory
- `h/l` - Collapse/expand directory
- `/` - Search files
- `g/G` - Jump to top/bottom
- `q/Esc` - Close view

**Data Structures**:
```rust
pub struct TreeView {
    repo: Repository,
    commit_id: Oid,
    root: TreeNode,
    current_path: Vec<String>,
    selected: usize,
    expanded: HashSet<PathBuf>,
}

pub enum TreeNode {
    Directory {
        name: String,
        children: Vec<TreeNode>,
        expanded: bool,
    },
    File {
        name: String,
        size: u64,
        mode: FileMode,
    },
}
```

**Implementation Notes**:
- Use git2::Tree for repository tree
- Lazy-load directory contents
- Track expanded state
- Show tree structure with unicode box characters

### 4. Blob View

**Purpose**: Display file contents at specific commit/ref.

**Features**:
- Syntax highlighting (if possible)
- Line numbers
- Search within file
- Jump to specific line number
- Copy content
- Show file metadata (size, mode, mime type)

**Keybindings**:
- `j/k` - Scroll up/down
- `g/G` - Jump to top/bottom
- `/` - Search in file
- `n/N` - Next/previous search result
- `:` - Jump to line number
- `PageUp/PageDown` - Page navigation
- `q/Esc` - Close view

**Data Structures**:
```rust
pub struct BlobView {
    repo: Repository,
    commit_id: Oid,
    file_path: String,
    content: Vec<String>,  // Lines
    scroll_offset: usize,
    search_query: String,
    search_matches: Vec<usize>,  // Line numbers
}
```

**Implementation Notes**:
- Use git2::Blob for file content
- Detect binary files and show hex dump
- Implement basic syntax highlighting with syntect
- Handle large files efficiently (lazy loading)

### 5. Refs View

**Purpose**: Display and manage branches, tags, and remotes.

**Features**:
- List all branches (local and remote)
- List all tags
- Show commit each ref points to
- Checkout branches
- Create/delete branches
- Create/delete tags
- Push/pull branches

**Keybindings**:
- `j/k` - Navigate refs
- `Enter` - Checkout branch or show tag commit
- `c` - Create new branch
- `d` - Delete branch/tag
- `m` - Merge branch
- `p` - Push branch
- `f` - Fetch
- `q/Esc` - Close view

**Data Structures**:
```rust
pub struct RefsView {
    repo: Repository,
    branches: Vec<BranchInfo>,
    tags: Vec<TagInfo>,
    remotes: Vec<RemoteInfo>,
    selected: usize,
    filter: RefFilter,
}

pub struct BranchInfo {
    name: String,
    is_head: bool,
    is_remote: bool,
    upstream: Option<String>,
    commit_id: Oid,
    commit_summary: String,
}

pub struct TagInfo {
    name: String,
    commit_id: Oid,
    message: Option<String>,
    tagger: Option<String>,
}

pub enum RefFilter {
    All,
    Branches,
    Tags,
    Remotes,
}
```

**Implementation Notes**:
- Use git2::Branch::lookup and git2::Tag
- Support filtering (show only branches, tags, etc.)
- Show ahead/behind counts for branches
- Implement safe delete (check if merged)

### 6. Log View (Custom Log)

**Purpose**: Display commit history with custom filters and options.

**Features**:
- Filter by author
- Filter by date range
- Filter by file/path
- Filter by grep pattern
- Show graph visualization
- Follow renames
- Show merge commits differently

**Keybindings**:
- `j/k` - Navigate commits
- `Enter` - Open commit diff
- `a` - Filter by author
- `d` - Filter by date
- `p` - Filter by path
- `g` - Filter by grep
- `c` - Clear filters
- `q/Esc` - Close view

**Data Structures**:
```rust
pub struct LogView {
    repo: Repository,
    commits: Vec<Commit>,
    filters: LogFilters,
    selected: usize,
    loading: bool,
}

pub struct LogFilters {
    author: Option<String>,
    since: Option<DateTime<Utc>>,
    until: Option<DateTime<Utc>>,
    path: Option<PathBuf>,
    grep: Option<String>,
}
```

**Implementation Notes**:
- Use git2::Revwalk with custom sorting
- Apply filters during walk
- Show graph with unicode characters
- Cache filtered results

### 7. Stash View

**Purpose**: Manage Git stashes.

**Features**:
- List all stashes
- Show stash diff
- Apply stash
- Pop stash
- Drop stash
- Create new stash
- Show stash details (message, author, date)

**Keybindings**:
- `j/k` - Navigate stashes
- `Enter` - Show stash diff
- `a` - Apply stash
- `p` - Pop stash (apply and drop)
- `d` - Drop stash
- `s` - Create new stash
- `q/Esc` - Close view

**Data Structures**:
```rust
pub struct StashView {
    repo: Repository,
    stashes: Vec<StashInfo>,
    selected: usize,
}

pub struct StashInfo {
    index: usize,
    message: String,
    commit_id: Oid,
    author: String,
    date: DateTime<Utc>,
}
```

**Implementation Notes**:
- Use git2::Repository::stash_foreach
- Show stash diff using git2::Diff
- Implement safe operations with confirmation
- Refresh list after operations

### 8. Grep View

**Purpose**: Search repository content across all files.

**Features**:
- Search with regex
- Show matching lines with context
- Jump to file location
- Filter by file pattern
- Show line numbers
- Case-sensitive/insensitive toggle

**Keybindings**:
- `j/k` - Navigate results
- `Enter` - Open file at line
- `/` - New search
- `i` - Toggle case sensitivity
- `n/N` - Next/previous result
- `q/Esc` - Close view

**Data Structures**:
```rust
pub struct GrepView {
    repo: Repository,
    query: String,
    results: Vec<GrepResult>,
    selected: usize,
    case_sensitive: bool,
}

pub struct GrepResult {
    file_path: String,
    line_number: u32,
    line_content: String,
    match_start: usize,
    match_end: usize,
}
```

**Implementation Notes**:
- Use git2::Repository::index to get tracked files
- Implement parallel search with rayon
- Highlight matches in results
- Support regex patterns

### 9. Reflog View

**Purpose**: Display Git reflog (reference history).

**Features**:
- Show reflog entries
- Display operation (commit, checkout, merge, etc.)
- Show old and new commit hashes
- Navigate to commit
- Cherry-pick from reflog
- Restore lost commits

**Keybindings**:
- `j/k` - Navigate entries
- `Enter` - Show commit diff
- `r` - Reset to entry
- `c` - Cherry-pick entry
- `q/Esc` - Close view

**Data Structures**:
```rust
pub struct ReflogView {
    repo: Repository,
    entries: Vec<ReflogEntry>,
    selected: usize,
}

pub struct ReflogEntry {
    old_id: Oid,
    new_id: Oid,
    message: String,
    committer: String,
    time: DateTime<Utc>,
}
```

**Implementation Notes**:
- Use git2::Reflog
- Show HEAD reflog by default
- Support viewing reflog for other refs
- Implement safe reset with confirmation

## Common Patterns

All views should follow these patterns:

### View Lifecycle
```rust
impl View for CustomView {
    fn on_activate(&mut self) -> Result<()> {
        // Start loading data
        self.start_loading();
        Ok(())
    }

    fn update(&mut self) -> Result<()> {
        // Check for async results
        if let Some(receiver) = &mut self.receiver {
            if let Ok(data) = receiver.try_recv() {
                self.handle_data(data)?;
            }
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        // Render UI
    }
}
```

### Async Operations
- Use channels for async communication
- Show loading indicators
- Handle errors gracefully
- Cancel operations when view closes

### Navigation
- Support vim-style keybindings (j/k/g/G)
- Implement page up/down
- Support search where appropriate
- Show current position indicator

### Error Handling
- Display errors in UI
- Log to stderr
- Provide actionable error messages
- Implement retry mechanisms

## Implementation Priority

Based on user value and implementation complexity:

1. **High Priority** (Most useful, moderate complexity):
   - Refs View (branch/tag management)
   - Blame View (code archaeology)
   - Tree View (file browsing)

2. **Medium Priority** (Useful, higher complexity):
   - Stage View (interactive staging)
   - Stash View (stash management)
   - Log View (advanced filtering)

3. **Lower Priority** (Specialized use cases):
   - Blob View (file viewing - can use external viewer)
   - Grep View (can use external grep)
   - Reflog View (advanced git operations)

## Testing Strategy

Each new view should include:

### Unit Tests
- View creation and initialization
- Navigation (up/down/page)
- State management
- Search functionality (if applicable)

### Integration Tests
- Data loading from git repository
- Git operations (create, delete, modify)
- Error handling
- View lifecycle (activate, deactivate)

### Manual Testing
- UI rendering
- Keybinding responsiveness
- Performance with large datasets
- Error scenarios

## Performance Considerations

### Large Repositories
- Lazy loading of data
- Pagination for large lists
- Background loading with progress indicators
- Caching of expensive operations

### Memory Usage
- Stream processing for large files
- Release memory when views close
- Limit cache size
- Use weak references where appropriate

## Accessibility

All views should:
- Support keyboard-only navigation
- Provide clear visual feedback
- Show help text for keybindings
- Use high-contrast colors
- Support terminal color schemes
