# Configuration Implementation Summary

This document describes the technical implementation of the configuration system in rust-tig.

## Architecture

The configuration system is organized into three main modules in `src/config/`:

- `config.rs`: Core configuration structures and file I/O
- `colors.rs`: Color parsing and style handling
- `mod.rs`: Public API exports

## Core Structures

### Config

The root configuration structure that contains all settings:

```rust
pub struct Config {
    pub keybindings: KeyBindings,
    pub colors: Colors,
    pub settings: Settings,
}
```

**File**: `src/config/config.rs:8-16`

### KeyBindings

Organizes keybindings by view context:

```rust
pub struct KeyBindings {
    pub global: HashMap<String, String>,  // Available in all views
    pub main: HashMap<String, String>,    // Main/commit history view
    pub diff: HashMap<String, String>,    // Diff view
    pub status: HashMap<String, String>,  // Status view
}
```

**File**: `src/config/config.rs:19-29`

### Colors

Stores color configuration as strings for serialization:

```rust
pub struct Colors {
    pub added: String,
    pub deleted: String,
    pub modified: String,
    pub unmodified: String,
    pub commit_hash: String,
    pub date: String,
    pub author: String,
    pub selected: String,      // Supports "fg on bg" format
    pub status_bar: String,    // Supports "fg on bg" format
}
```

**File**: `src/config/config.rs:32-52`

### Settings

General application settings:

```rust
pub struct Settings {
    pub commit_chunk_size: usize,
    pub date_format: String,
    pub mouse_support: bool,
    pub show_line_numbers: bool,
    pub tab_width: usize,
}
```

**File**: `src/config/config.rs:55-67`

## Default Values

All structures implement `Default` trait with sensible defaults:

- **Global keybindings**: `q` (quit), `?` (help), `r` (refresh)
- **Main view**: `/` (search), `s` (status), `d` (diff), `Enter` (enter)
- **Diff view**: `k`/`j` (scroll), `PageUp`/`PageDown` (page)
- **Status view**: `s` (stage), `u` (unstage), `c` (commit), `d` (diff)
- **Colors**: Standard terminal colors (green for added, red for deleted, etc.)
- **Settings**: 50 commits per chunk, ISO date format, mouse enabled, line numbers shown, 4-space tabs

**Implementation**: `src/config/config.rs:69-139`

## File I/O

### Configuration Paths

```rust
pub fn default_path() -> Result<PathBuf>
```

Returns the platform-appropriate config path using the `dirs` crate:
- Unix: `~/.config/rust-tig/config.yaml`
- Windows: `%APPDATA%\rust-tig\config.yaml`

**File**: `src/config/config.rs:145-151`

### Loading Configuration

```rust
pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self>
pub fn load() -> Result<Self>
```

- `load_from_file`: Loads from a specific path, returns default if file doesn't exist
- `load`: Convenience method that loads from the default path

**File**: `src/config/config.rs:155-175`

### Saving Configuration

```rust
pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()>
pub fn save(&self) -> Result<()>
```

- Creates parent directories if they don't exist
- Serializes to YAML using `serde_yaml`
- Atomic write operation

**File**: `src/config/config.rs:178-200`

### Initialization

```rust
pub fn init_default() -> Result<PathBuf>
```

Creates a default configuration file at the default path. Safe to call multiple times (won't overwrite existing files).

**File**: `src/config/config.rs:204-216`

## Color Parsing

### parse_color

```rust
pub fn parse_color(color_str: &str) -> Color
```

Parses color names to ratatui `Color` enum:
- Case-insensitive
- Supports basic colors: black, red, green, yellow, blue, magenta, cyan, white
- Supports dark/bright variants
- Supports grey/gray variants
- Fallback to white for unknown colors

**File**: `src/config/colors.rs:14-47`

**Note**: Ratatui's color model differs from some terminals. The base colors (e.g., `Color::Red`) are typically the bright variants in most terminals.

### parse_style

```rust
pub fn parse_style(style_str: &str) -> Style
```

Parses style strings with optional background:
- Simple: `"green"` → foreground only
- Compound: `"black on white"` → foreground on background

**File**: `src/config/colors.rs:53-63`

### ColorScheme

Helper struct that holds parsed colors as ratatui types:

```rust
pub struct ColorScheme {
    pub added: Color,
    pub deleted: Color,
    pub modified: Color,
    pub unmodified: Color,
    pub commit_hash: Color,
    pub date: Color,
    pub author: Color,
    pub selected: Style,
    pub status_bar: Style,
}
```

**File**: `src/config/colors.rs:66-78`

Includes `from_config` method to convert `Colors` → `ColorScheme`:

```rust
pub fn from_config(colors: &Colors) -> Self
```

**File**: `src/config/colors.rs:81-94`

## Serialization

Uses `serde` with `serde_yaml`:
- All configuration structs derive `Serialize` and `Deserialize`
- YAML format for human-readable configuration
- Automatic serialization of `HashMap` for keybindings

## Dependencies

- `serde`: Serialization framework
- `serde_yaml`: YAML serialization
- `dirs`: Cross-platform directory paths
- `anyhow`: Error handling
- `ratatui`: Terminal UI and color types

## Testing

Comprehensive test suite in `src/config/config.rs:218-296`:

- `test_default_config`: Verifies default values
- `test_save_and_load_config`: Round-trip serialization
- `test_load_nonexistent_file`: Graceful handling of missing files
- `test_custom_keybindings`: Custom keybinding persistence
- `test_custom_colors`: Custom color persistence
- `test_custom_settings`: Custom settings persistence

Color parsing tests in `src/config/colors.rs:96-143`:

- Basic color parsing
- Dark color variants
- Grey variants
- Simple and compound styles
- Case insensitivity

## Usage Example

```rust
use rust_tig::config::Config;

// Load configuration
let mut config = Config::load()?;

// Modify settings
config.colors.added = "bright green".to_string();
config.settings.commit_chunk_size = 100;
config.keybindings.global.insert("custom".to_string(), "x".to_string());

// Save changes
config.save()?;

// Parse colors for UI rendering
let color_scheme = ColorScheme::from_config(&config.colors);
```

## Demo Program

An example program demonstrates the configuration system:

```bash
cargo run --example config_demo
```

Shows:
- Default config path
- Loading configuration
- Displaying current settings
- Modifying configuration
- Saving to file
- Verifying saved configuration
- Generated YAML output

**File**: `examples/config_demo.rs`

## Future Enhancements

Potential improvements to consider:

1. **Validation**: Add validation for color names and keybinding values
2. **Migration**: Add configuration version and migration support
3. **Profiles**: Support multiple configuration profiles
4. **Hot Reload**: Watch config file for changes and reload automatically
5. **RGB Colors**: Support RGB and hex color values
6. **Modifiers**: Support key modifiers (Ctrl, Alt, Shift)
7. **Themes**: Pre-defined color themes
8. **Schema**: Generate JSON schema for editor autocompletion

## Error Handling

All I/O operations return `anyhow::Result`:
- Missing files: Returns default configuration (graceful degradation)
- Invalid YAML: Returns error with context
- Permission errors: Returns error with file path
- Directory creation: Automatically creates parent directories
