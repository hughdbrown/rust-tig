# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Tig is an ncurses-based text-mode interface for Git. It functions as a Git repository browser, assists in staging changes for commit at chunk level, and acts as a pager for output from various Git commands.

**Language**: C
**Version**: 2.6.0
**License**: GPL-2.0

## Build Commands

### Building

```bash
# Basic build
make

# Build with debugging symbols
make clean all-debug

# Build with code coverage
make all-coverage

# Build with address sanitizer
make all-address-sanitizer

# Generate configure script (if building from git repo)
make configure
./configure
make
```

### Installation

```bash
# Install to $HOME/bin (default)
make install

# Install to custom prefix
make prefix=/usr/local
sudo make install prefix=/usr/local
```

### Testing

```bash
# Run all tests
make test

# Run individual test
make test/tigrc/parse-test

# Run TODO tests (normally skipped)
make test-todo

# Run with coverage report
make test-coverage

# Run with address sanitizer
make test-address-sanitizer

# Set test options (see test/README.adoc for details)
TEST_OPTS='filter=*:*default' make test
TEST_OPTS='verbose' make test
TEST_OPTS='debugger=lldb' make test
```

### Cleaning

```bash
make clean          # Remove build artifacts
make distclean      # Also remove configure-generated files
make veryclean      # Also remove documentation
```

### Documentation

```bash
make doc            # Build all documentation
make doc-man        # Build man pages only
make doc-html       # Build HTML documentation only
```

## Architecture Overview

### View-Based Architecture

Tig is built around a **view** abstraction. Each view type (main, diff, log, blame, tree, status, stage, refs, stash, grep, pager, help) is a specialized component that displays Git information.

**Key structures:**
- `struct view` (`include/tig/view.h`): Core view structure containing state, lines, operations
- `struct view_ops` (`include/tig/view.h`): Function pointers for view operations (open, read, draw, request, grep, select, done)
- `struct line` (`include/tig/view.h`): Represents a single line in a view with type, state flags, and user data

**View types are implemented in separate files:**
- `src/main.c` - Main view (commit log)
- `src/diff.c` - Diff view
- `src/log.c` - Log view
- `src/blame.c` - Blame view
- `src/tree.c` - Tree browser
- `src/status.c` - Status view
- `src/stage.c` - Stage view (for staging hunks)
- `src/refs.c` - References view
- `src/stash.c` - Stash view
- `src/grep.c` - Grep results view
- `src/pager.c` - Generic pager
- `src/help.c` - Help view

### Core Subsystems

**Display and Drawing** (`src/display.c`, `src/draw.c`):
- Manages ncurses windows and screen layout
- Handles rendering of view content with proper formatting and colors

**I/O and Process Management** (`src/io.c`, `src/argv.c`):
- Manages execution of Git commands
- Handles input/output pipes and process lifecycle

**Line Management** (`src/line.c`):
- Defines line types and colors
- Manages line state and rendering attributes

**Key Bindings** (`src/keys.c`, `src/request.c`):
- Handles key input and mapping to requests
- Request-based command system for view actions

**Options and Configuration** (`src/options.c`, `src/builtin-config.c`):
- Parses `tigrc` configuration files
- Manages runtime options and settings
- Built-in config is generated from `tigrc` via `tools/make-builtin-config.sh`

**Git Integration** (`include/tig/git.h`):
- Git command generation and output parsing
- Reference database (`src/refdb.c`)
- Repository state tracking (`src/repo.c`)

**Graph Rendering** (`src/graph-v1.c`, `src/graph-v2.c`):
- Commit graph visualization
- Two versions for compatibility

**Search** (`src/search.c`):
- Search functionality across views
- Supports PCRE/PCRE2 for regex when available

**File Watching** (`src/watch.c`):
- Monitors file changes to auto-refresh views

### Code Organization

```
include/tig/        # Header files for all subsystems
src/                # Implementation files
compat/             # Compatibility layer (setenv, mkstemps, wordexp, hashtab, utf8proc)
test/               # Test suite organized by view/feature
tools/              # Build and documentation generation tools
contrib/            # Example configs, completions, and platform-specific config
doc/                # Documentation sources (AsciiDoc format)
```

### Configuration System

Tig uses `tigrc` files for configuration:
- System-wide: `$(sysconfdir)/tigrc` (e.g., `/etc/tigrc`)
- User-specific: `~/.tigrc` (or `$TIG_USER_CONFIG` if defined)
- Fallback: Built-in config compiled from `tigrc` file

Configuration uses a simple command-based syntax supporting:
- View column definitions
- Key bindings
- Color schemes
- Display options
- External command integration

## Development Guidelines

### Code Style

- C89/C90 compatible with some C99 features where supported
- Tab indentation (8 spaces)
- Header copyright template in `tools/header.h`
- Use `make update-headers` to refresh copyright headers
- All source files include appropriate headers from `include/tig/`

### Adding a New View

1. Create `src/newview.c` with view implementation
2. Create `include/tig/newview.h` with public interface
3. Define `struct newview_state` for view-specific state
4. Implement `newview_ops` with required view operations
5. Add view to `src/tig.c` includes and initialization
6. Update `TIG_OBJS` in `Makefile`
7. Add tests in `test/newview/`

### Testing

Tests use shell scripts (`*-test` files) with the `libtest.sh` framework:
- Located in `test/` organized by feature
- Use `make test` to run all tests
- Individual tests can be run directly if `PATH` includes `src/` and `test/tools/`
- Test runner is `test/tools/show-results.sh`

### Platform Compatibility

Tig supports Linux, macOS, BSD, Cygwin, and Windows (via Git for Windows).

**Compatibility layer** (`compat/`):
- `NO_SETENV` - Provide setenv() if missing
- `NO_MKSTEMPS` - Provide mkstemps() if missing
- `NO_WORDEXP` - Provide wordexp() if missing
- `NO_STRNDUP` - Provide strndup() if missing

Platform-specific build configs in `contrib/config.make-$kernel`.

### Optional Dependencies

- **ncursesw**: Wide character support (required for UTF-8)
- **readline**: Command and search history
- **PCRE/PCRE2**: Perl-compatible regex in searches

### Debugging

```bash
# Build with debug symbols
make clean all-debug

# Run with debugger
TEST_OPTS='debugger=lldb' make test/some-test

# Run with valgrind
TEST_OPTS='valgrind' make test
```

## Common Patterns

### View Lifecycle

1. `view_ops->open()` - Initialize view, start Git command
2. `view_ops->read()` - Parse lines from Git command output (called repeatedly)
3. `view_ops->draw()` - Render a line (called for visible lines)
4. `view_ops->request()` - Handle user input/commands
5. `view_ops->select()` - Handle line selection changes
6. `view_ops->done()` - Cleanup when view is closed

### Adding Lines to View

```c
struct line *line = add_line_alloc(view, &data, type, data_size, custom);
```

### Request Handling

Views handle user actions via `enum request` (defined in `include/tig/request.h`). Requests are generated from key bindings and propagated through the view hierarchy.
