use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Main configuration structure for rust-tig
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Config {
    /// Keybinding configuration for different views
    pub keybindings: KeyBindings,
    /// Color scheme configuration
    pub colors: Colors,
    /// General application settings
    pub settings: Settings,
}

/// Keybinding configuration organized by view
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KeyBindings {
    /// Global keybindings available in all views
    pub global: HashMap<String, String>,
    /// Main view (commit history) keybindings
    pub main: HashMap<String, String>,
    /// Diff view keybindings
    pub diff: HashMap<String, String>,
    /// Status view keybindings
    pub status: HashMap<String, String>,
}

/// Color configuration for various UI elements
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Colors {
    /// Color for added lines in diffs
    pub added: String,
    /// Color for deleted lines in diffs
    pub deleted: String,
    /// Color for modified content
    pub modified: String,
    /// Color for unmodified content
    pub unmodified: String,
    /// Color for commit hashes
    pub commit_hash: String,
    /// Color for dates
    pub date: String,
    /// Color for authors
    pub author: String,
    /// Color for selected items
    pub selected: String,
    /// Color for status bar
    pub status_bar: String,
}

/// General application settings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Settings {
    /// Number of commits to load per chunk
    pub commit_chunk_size: usize,
    /// Date format string (chrono format)
    pub date_format: String,
    /// Enable/disable mouse support
    pub mouse_support: bool,
    /// Show line numbers in diff view
    pub show_line_numbers: bool,
    /// Tab width for display
    pub tab_width: usize,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            keybindings: KeyBindings::default(),
            colors: Colors::default(),
            settings: Settings::default(),
        }
    }
}

impl Default for KeyBindings {
    fn default() -> Self {
        let mut global = HashMap::new();
        global.insert("quit".to_string(), "q".to_string());
        global.insert("help".to_string(), "?".to_string());
        global.insert("refresh".to_string(), "r".to_string());

        let mut main = HashMap::new();
        main.insert("search".to_string(), "/".to_string());
        main.insert("status".to_string(), "s".to_string());
        main.insert("diff".to_string(), "d".to_string());
        main.insert("enter".to_string(), "Enter".to_string());

        let mut diff = HashMap::new();
        diff.insert("scroll_up".to_string(), "k".to_string());
        diff.insert("scroll_down".to_string(), "j".to_string());
        diff.insert("page_up".to_string(), "PageUp".to_string());
        diff.insert("page_down".to_string(), "PageDown".to_string());

        let mut status = HashMap::new();
        status.insert("stage".to_string(), "s".to_string());
        status.insert("unstage".to_string(), "u".to_string());
        status.insert("commit".to_string(), "c".to_string());
        status.insert("diff".to_string(), "d".to_string());

        KeyBindings {
            global,
            main,
            diff,
            status,
        }
    }
}

impl Default for Colors {
    fn default() -> Self {
        Colors {
            added: "green".to_string(),
            deleted: "red".to_string(),
            modified: "yellow".to_string(),
            unmodified: "black".to_string(),
            commit_hash: "cyan".to_string(),
            date: "blue".to_string(),
            author: "magenta".to_string(),
            selected: "black on white".to_string(),
            status_bar: "black on cyan".to_string(),
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            commit_chunk_size: 50,
            date_format: "%Y-%m-%d %H:%M".to_string(),
            mouse_support: true,
            show_line_numbers: true,
            tab_width: 4,
        }
    }
}

impl Config {
    /// Get the default configuration file path
    /// Returns ~/.config/rust-tig/config.yaml on Unix-like systems
    /// Returns %APPDATA%\rust-tig\config.yaml on Windows
    pub fn default_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Could not determine config directory")?
            .join("rust-tig");

        Ok(config_dir.join("config.yaml"))
    }

    /// Load configuration from a YAML file
    /// If the file doesn't exist, returns the default configuration
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();

        if !path.exists() {
            return Ok(Config::default());
        }

        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        let config: Config = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

        Ok(config)
    }

    /// Load configuration from the default path
    pub fn load() -> Result<Self> {
        let path = Self::default_path()?;
        Self::load_from_file(path)
    }

    /// Save configuration to a YAML file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();

        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {}", parent.display()))?;
        }

        let yaml = serde_yaml::to_string(self)
            .context("Failed to serialize configuration")?;

        fs::write(path, yaml)
            .with_context(|| format!("Failed to write config file: {}", path.display()))?;

        Ok(())
    }

    /// Save configuration to the default path
    pub fn save(&self) -> Result<()> {
        let path = Self::default_path()?;
        self.save_to_file(path)
    }

    /// Create a new default configuration file at the default path
    /// Only creates the file if it doesn't already exist
    pub fn init_default() -> Result<PathBuf> {
        let path = Self::default_path()?;

        if path.exists() {
            return Ok(path);
        }

        let config = Config::default();
        config.save_to_file(&path)?;

        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = Config::default();

        assert_eq!(config.keybindings.global.get("quit"), Some(&"q".to_string()));
        assert_eq!(config.colors.added, "green");
        assert_eq!(config.settings.commit_chunk_size, 50);
    }

    #[test]
    fn test_save_and_load_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");

        let original = Config::default();
        original.save_to_file(&config_path).unwrap();

        let loaded = Config::load_from_file(&config_path).unwrap();
        assert_eq!(original, loaded);
    }

    #[test]
    fn test_load_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("nonexistent.yaml");

        let config = Config::load_from_file(&config_path).unwrap();
        assert_eq!(config, Config::default());
    }

    #[test]
    fn test_custom_keybindings() {
        let mut config = Config::default();
        config.keybindings.global.insert("custom".to_string(), "x".to_string());

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");

        config.save_to_file(&config_path).unwrap();
        let loaded = Config::load_from_file(&config_path).unwrap();

        assert_eq!(loaded.keybindings.global.get("custom"), Some(&"x".to_string()));
    }

    #[test]
    fn test_custom_colors() {
        let mut config = Config::default();
        config.colors.added = "bright green".to_string();

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");

        config.save_to_file(&config_path).unwrap();
        let loaded = Config::load_from_file(&config_path).unwrap();

        assert_eq!(loaded.colors.added, "bright green");
    }

    #[test]
    fn test_custom_settings() {
        let mut config = Config::default();
        config.settings.commit_chunk_size = 100;
        config.settings.mouse_support = false;

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");

        config.save_to_file(&config_path).unwrap();
        let loaded = Config::load_from_file(&config_path).unwrap();

        assert_eq!(loaded.settings.commit_chunk_size, 100);
        assert_eq!(loaded.settings.mouse_support, false);
    }
}
