use ratatui::style::{Color, Style};

/// Parse a color string and return the corresponding ratatui Color
///
/// Supports standard terminal colors:
/// - Basic: black, red, green, yellow, blue, magenta, cyan, white
/// - Dark variants: dark red, dark green, dark yellow, dark blue, dark magenta, dark cyan
/// - Grey variants: grey, gray, dark grey, dark gray
/// - Aliases: bright black (dark grey)
///
/// Note: In crossterm/ratatui:
/// - "red", "green", etc. are the bright/normal terminal colors
/// - "dark red", "dark green", etc. are the darker variants
pub fn parse_color(color_str: &str) -> Color {
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
        "grey" | "gray" => Color::Gray,
        "dark grey" | "dark gray" | "dark black" => Color::DarkGray,
        // Ratatui doesn't have separate dark colors, so map them to the base colors
        // which are actually the bright variants in most terminals
        "dark red" => Color::Red,
        "dark green" => Color::Green,
        "dark yellow" => Color::Yellow,
        "dark blue" => Color::Blue,
        "dark magenta" => Color::Magenta,
        "dark cyan" => Color::Cyan,
        "dark white" => Color::White,
        // For backwards compatibility, accept "bright" prefix even though it's redundant
        "bright black" => Color::DarkGray,
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

/// Parse a style string that may include foreground and background colors
/// Supports formats like:
/// - "green" - just foreground
/// - "black on white" - foreground on background
pub fn parse_style(style_str: &str) -> Style {
    if let Some(idx) = style_str.find(" on ") {
        let (fg, bg) = style_str.split_at(idx);
        let bg = &bg[4..]; // Skip " on "
        Style::default()
            .fg(parse_color(fg))
            .bg(parse_color(bg))
    } else {
        Style::default().fg(parse_color(style_str))
    }
}

/// Helper struct to hold parsed color scheme
#[derive(Debug, Clone)]
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

impl ColorScheme {
    /// Parse a Colors configuration into a ColorScheme with actual Color types
    pub fn from_config(colors: &crate::config::Colors) -> Self {
        Self {
            added: parse_color(&colors.added),
            deleted: parse_color(&colors.deleted),
            modified: parse_color(&colors.modified),
            unmodified: parse_color(&colors.unmodified),
            commit_hash: parse_color(&colors.commit_hash),
            date: parse_color(&colors.date),
            author: parse_color(&colors.author),
            selected: parse_style(&colors.selected),
            status_bar: parse_style(&colors.status_bar),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_colors() {
        assert_eq!(parse_color("red"), Color::Red);
        assert_eq!(parse_color("green"), Color::Green);
        assert_eq!(parse_color("blue"), Color::Blue);
    }

    #[test]
    fn test_parse_dark_colors() {
        // Ratatui doesn't have DarkRed, DarkGreen, etc.
        // These map to the base colors
        assert_eq!(parse_color("dark red"), Color::Red);
        assert_eq!(parse_color("dark green"), Color::Green);
    }

    #[test]
    fn test_parse_grey_variants() {
        assert_eq!(parse_color("grey"), Color::Gray);
        assert_eq!(parse_color("gray"), Color::Gray);
        assert_eq!(parse_color("dark grey"), Color::DarkGray);
        assert_eq!(parse_color("dark gray"), Color::DarkGray);
    }

    #[test]
    fn test_parse_style_simple() {
        let style = parse_style("green");
        assert_eq!(style.fg, Some(Color::Green));
        assert_eq!(style.bg, None);
    }

    #[test]
    fn test_parse_style_compound() {
        let style = parse_style("black on white");
        assert_eq!(style.fg, Some(Color::Black));
        assert_eq!(style.bg, Some(Color::White));
    }

    #[test]
    fn test_case_insensitive() {
        assert_eq!(parse_color("RED"), Color::Red);
        assert_eq!(parse_color("Green"), Color::Green);
        assert_eq!(parse_color("DARK BLUE"), Color::Blue);
    }
}
