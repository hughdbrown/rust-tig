# Configuration Guide

rust-tig supports customization through a YAML configuration file.

## Configuration File Location

The default configuration file location depends on your operating system:

- **Linux/macOS**: `~/.config/rust-tig/config.yaml`
- **Windows**: `%APPDATA%\rust-tig\config.yaml`

## Creating Your Configuration

1. Copy the example configuration:
   ```bash
   cp config.example.yaml ~/.config/rust-tig/config.yaml
   ```

2. Edit the file with your preferred settings

3. rust-tig will automatically load the configuration on startup

If no configuration file exists, rust-tig will use sensible defaults.

## Configuration Structure

The configuration file has three main sections:

### 1. Keybindings

Define custom keybindings for different views:

```yaml
keybindings:
  global:        # Available in all views
    quit: q
    help: "?"
    refresh: r

  main:          # Commit history view
    search: /
    status: s
    diff: d
    enter: Enter

  diff:          # Diff view
    scroll_up: k
    scroll_down: j
    page_up: PageUp
    page_down: PageDown

  status:        # Status view
    stage: s
    unstage: u
    commit: c
    diff: d
```

#### Supported Key Values

- Single characters: `a`, `b`, `1`, `/`, etc.
- Special keys: `Enter`, `Tab`, `Esc`, `Space`
- Arrow keys: `Up`, `Down`, `Left`, `Right`
- Function keys: `F1`, `F2`, etc.
- Page keys: `PageUp`, `PageDown`, `Home`, `End`

### 2. Colors

Customize the color scheme for various UI elements:

```yaml
colors:
  added: green              # Added lines in diffs
  deleted: red              # Deleted lines in diffs
  modified: yellow          # Modified content
  unmodified: black         # Unmodified content
  commit_hash: cyan         # Commit hashes
  date: blue                # Dates
  author: magenta           # Authors
  selected: black on white  # Selected items
  status_bar: black on cyan # Status bar
```

#### Supported Colors

**Basic colors:**
- `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`

**Dark variants:**
- `dark red`, `dark green`, `dark yellow`, `dark blue`, `dark magenta`, `dark cyan`

**Grey variants:**
- `grey`, `gray`, `dark grey`, `dark gray`

**Bright variants (for compatibility):**
- `bright red`, `bright green`, etc.

#### Color Format

- Simple foreground: `green`
- Foreground and background: `black on white`

### 3. Settings

General application settings:

```yaml
settings:
  commit_chunk_size: 50         # Number of commits to load at once
  date_format: "%Y-%m-%d %H:%M" # Date display format
  mouse_support: true           # Enable mouse interaction
  show_line_numbers: true       # Show line numbers in diffs
  tab_width: 4                  # Width of tab characters
```

#### Date Format

Uses [chrono format strings](https://docs.rs/chrono/latest/chrono/format/strftime/index.html):

| Format | Example | Description |
|--------|---------|-------------|
| `%Y-%m-%d %H:%M` | 2024-10-07 15:30 | ISO format with time |
| `%Y-%m-%d` | 2024-10-07 | ISO date only |
| `%b %d %Y` | Oct 07 2024 | Month name format |
| `%d/%m/%Y` | 07/10/2024 | European format |
| `%m/%d/%Y` | 10/07/2024 | American format |

## Example Configurations

### Minimal Configuration

```yaml
keybindings:
  global:
    quit: q
colors:
  added: green
  deleted: red
settings:
  commit_chunk_size: 50
```

### Dark Theme

```yaml
colors:
  added: bright green
  deleted: bright red
  modified: bright yellow
  unmodified: grey
  commit_hash: bright cyan
  date: bright blue
  author: bright magenta
  selected: white on dark blue
  status_bar: white on dark grey
```

### Vim-style Keybindings

```yaml
keybindings:
  global:
    quit: q
    help: "?"
  main:
    search: /
    status: s
    diff: d
  diff:
    scroll_up: k
    scroll_down: j
    page_up: Ctrl-b
    page_down: Ctrl-f
  status:
    stage: s
    unstage: u
```

## Testing Your Configuration

You can test your configuration using the example program:

```bash
cargo run --example config_demo
```

This will:
1. Show the default config path
2. Load and display your current configuration
3. Demonstrate modifying and saving configuration
4. Show the generated YAML

## Troubleshooting

### Configuration Not Loading

1. Check the file path: `Config::default_path()` in the config_demo
2. Verify YAML syntax is valid
3. Check file permissions

### Invalid Color Names

If a color name is not recognized, rust-tig will fall back to white. Check the supported colors list above.

### Keybinding Conflicts

If multiple actions are bound to the same key, the behavior is undefined. Ensure each key is unique within a view.

## Configuration API

For programmatic configuration:

```rust
use rust_tig::config::Config;

// Load from default location
let config = Config::load()?;

// Load from specific path
let config = Config::load_from_file("path/to/config.yaml")?;

// Modify configuration
let mut config = Config::default();
config.colors.added = "bright green".to_string();
config.settings.commit_chunk_size = 100;

// Save configuration
config.save()?;
config.save_to_file("path/to/config.yaml")?;

// Initialize default config file
let path = Config::init_default()?;
```
