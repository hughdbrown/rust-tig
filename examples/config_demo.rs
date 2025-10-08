// Example demonstrating how to use the rust-tig configuration system
// Run with: cargo run --example config_demo

use rust_tig::config::Config;
use anyhow::Result;
use crossterm::style::{Color, Stylize};

/// Parse a color string and return the corresponding crossterm Color
fn parse_color(color_str: &str) -> Color {
    let color_lower = color_str.trim().to_lowercase();
    match color_lower.as_str() {
        "black" => Color::Black,
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "white" => Color::White,
        "dark black" => Color::Black,
        "dark red" => Color::DarkRed,
        "dark green" => Color::DarkGreen,
        "dark yellow" => Color::DarkYellow,
        "dark blue" => Color::DarkBlue,
        "dark magenta" => Color::DarkMagenta,
        "dark cyan" => Color::DarkCyan,
        "dark white" | "grey" | "gray" => Color::Grey,
        "bright black" | "dark grey" | "dark gray" => Color::DarkGrey,
        "bright red" => Color::Red,
        "bright green" => Color::Green,
        "bright yellow" => Color::Yellow,
        "bright blue" => Color::Blue,
        "bright magenta" => Color::Magenta,
        "bright cyan" => Color::Cyan,
        "bright white" => Color::White,
        _ => Color::White, // fallback
    }
}

/// Print a color item with the text colored according to the color name
fn print_color_item(label: &str, color_str: &str) {
    // Handle compound colors like "black on white"
    if let Some(idx) = color_str.find(" on ") {
        let (fg, bg) = color_str.split_at(idx);
        let bg = &bg[4..]; // Skip " on "
        let fg_color = parse_color(fg);
        let bg_color = parse_color(bg);
        println!("  {}: {}", label, color_str.with(fg_color).on(bg_color));
    } else {
        let color = parse_color(color_str);
        println!("  {}: {}", label, color_str.with(color));
    }
}

fn main() -> Result<()> {
    println!("=== rust-tig Configuration Demo ===\n");

    // 1. Get the default config path
    let config_path = Config::default_path()?;
    println!("Default config path: {}", config_path.display());

    // 2. Load the configuration (returns default if file doesn't exist)
    println!("\nLoading configuration...");
    let mut config = Config::load()?;
    println!("Configuration loaded successfully!");

    // 3. Display current configuration
    println!("\n--- Current Configuration ---");

    println!("\nKeybindings:");
    println!("  Global:");
    for (action, key) in &config.keybindings.global {
        println!("    {}: {}", action, key);
    }
    println!("  Main view:");
    for (action, key) in &config.keybindings.main {
        println!("    {}: {}", action, key);
    }

    println!("\nColors:");
    print_color_item("added", &config.colors.added);
    print_color_item("deleted", &config.colors.deleted);
    print_color_item("modified", &config.colors.modified);
    print_color_item("commit_hash", &config.colors.commit_hash);
    print_color_item("date", &config.colors.date);
    print_color_item("author", &config.colors.author);
    print_color_item("selected", &config.colors.selected);
    print_color_item("status_bar", &config.colors.status_bar);

    println!("\nSettings:");
    println!("  commit_chunk_size: {}", config.settings.commit_chunk_size);
    println!("  date_format: {}", config.settings.date_format);
    println!("  mouse_support: {}", config.settings.mouse_support);
    println!("  show_line_numbers: {}", config.settings.show_line_numbers);
    println!("  tab_width: {}", config.settings.tab_width);

    // 4. Modify configuration
    println!("\n--- Modifying Configuration ---");
    config.keybindings.global.insert("custom_action".to_string(), "x".to_string());
    config.colors.added = "bright green".to_string();
    config.settings.commit_chunk_size = 100;
    println!("Modified:");
    println!("  - Added custom keybinding: custom_action -> x");
    println!("  - Changed added color to: {}", "bright green".with(parse_color("bright green")));
    println!("  - Changed commit_chunk_size to: 100");

    // 5. Save configuration to a temporary location
    let temp_path = std::env::temp_dir().join("rust-tig-demo-config.yaml");
    println!("\nSaving modified config to: {}", temp_path.display());
    config.save_to_file(&temp_path)?;
    println!("Configuration saved successfully!");

    // 6. Load the saved configuration to verify
    println!("\nVerifying saved configuration...");
    let loaded_config = Config::load_from_file(&temp_path)?;

    if let Some(custom_key) = loaded_config.keybindings.global.get("custom_action") {
        println!("  ✓ Custom keybinding loaded: {}", custom_key);
    }
    let added_color = &loaded_config.colors.added;
    println!("  ✓ Added color: {}", added_color.clone().with(parse_color(added_color)));
    println!("  ✓ Commit chunk size: {}", loaded_config.settings.commit_chunk_size);

    // 7. Display YAML content
    println!("\n--- Generated YAML ---");
    let yaml_content = serde_yaml::to_string(&config)?;
    println!("{}", yaml_content);

    // 8. Initialize default config (safe to call multiple times)
    println!("\n--- Initialize Default Config ---");
    match Config::init_default() {
        Ok(path) => {
            if path.exists() {
                println!("Config file exists at: {}", path.display());
                println!("(File was already present or just created)");
            }
        }
        Err(e) => println!("Note: Could not initialize default config: {}", e),
    }

    // Cleanup temporary file
    let _ = std::fs::remove_file(&temp_path);

    println!("\n=== Demo Complete ===");
    Ok(())
}
