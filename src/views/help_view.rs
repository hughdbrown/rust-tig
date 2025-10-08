use super::view::{Action, View};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

/// Help view showing keybindings and usage information
pub struct HelpView {
    scroll_offset: usize,
    lines: Vec<Line<'static>>,
}

impl HelpView {
    /// Create a new help view
    pub fn new() -> Self {
        let lines = Self::build_help_lines();
        Self {
            scroll_offset: 0,
            lines,
        }
    }

    /// Build the help content
    fn build_help_lines() -> Vec<Line<'static>> {
        vec![
            Line::from(Span::styled(
                "rust-tig - Git TUI Help",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Global Keybindings",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(vec![
                Span::styled("  q         ", Style::default().fg(Color::Green)),
                Span::raw("Quit the application or close current view"),
            ]),
            Line::from(vec![
                Span::styled("  Ctrl+C    ", Style::default().fg(Color::Green)),
                Span::raw("Force quit"),
            ]),
            Line::from(vec![
                Span::styled("  ?         ", Style::default().fg(Color::Green)),
                Span::raw("Show this help"),
            ]),
            Line::from(vec![
                Span::styled("  Esc       ", Style::default().fg(Color::Green)),
                Span::raw("Close current view or exit search mode"),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "Main View (Commit History)",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(vec![
                Span::styled("  j / Down  ", Style::default().fg(Color::Green)),
                Span::raw("Move selection down"),
            ]),
            Line::from(vec![
                Span::styled("  k / Up    ", Style::default().fg(Color::Green)),
                Span::raw("Move selection up"),
            ]),
            Line::from(vec![
                Span::styled("  g         ", Style::default().fg(Color::Green)),
                Span::raw("Jump to first commit"),
            ]),
            Line::from(vec![
                Span::styled("  G         ", Style::default().fg(Color::Green)),
                Span::raw("Jump to last commit"),
            ]),
            Line::from(vec![
                Span::styled("  PageUp    ", Style::default().fg(Color::Green)),
                Span::raw("Page up"),
            ]),
            Line::from(vec![
                Span::styled("  PageDown  ", Style::default().fg(Color::Green)),
                Span::raw("Page down"),
            ]),
            Line::from(vec![
                Span::styled("  Enter     ", Style::default().fg(Color::Green)),
                Span::raw("View commit diff"),
            ]),
            Line::from(vec![
                Span::styled("  /         ", Style::default().fg(Color::Green)),
                Span::raw("Start search (search commit messages)"),
            ]),
            Line::from(vec![
                Span::styled("  s         ", Style::default().fg(Color::Green)),
                Span::raw("Open status view"),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "Search Mode",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(vec![
                Span::styled("  Type      ", Style::default().fg(Color::Green)),
                Span::raw("Enter search query"),
            ]),
            Line::from(vec![
                Span::styled("  Backspace ", Style::default().fg(Color::Green)),
                Span::raw("Delete character"),
            ]),
            Line::from(vec![
                Span::styled("  Enter     ", Style::default().fg(Color::Green)),
                Span::raw("Keep search results and exit search mode"),
            ]),
            Line::from(vec![
                Span::styled("  Esc       ", Style::default().fg(Color::Green)),
                Span::raw("Clear search and exit search mode"),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "Status View",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(vec![
                Span::styled("  j / Down  ", Style::default().fg(Color::Green)),
                Span::raw("Move selection down"),
            ]),
            Line::from(vec![
                Span::styled("  k / Up    ", Style::default().fg(Color::Green)),
                Span::raw("Move selection up"),
            ]),
            Line::from(vec![
                Span::styled("  g         ", Style::default().fg(Color::Green)),
                Span::raw("Jump to first item"),
            ]),
            Line::from(vec![
                Span::styled("  G         ", Style::default().fg(Color::Green)),
                Span::raw("Jump to last item"),
            ]),
            Line::from(vec![
                Span::styled("  PageUp    ", Style::default().fg(Color::Green)),
                Span::raw("Page up"),
            ]),
            Line::from(vec![
                Span::styled("  PageDown  ", Style::default().fg(Color::Green)),
                Span::raw("Page down"),
            ]),
            Line::from(vec![
                Span::styled("  Enter     ", Style::default().fg(Color::Green)),
                Span::raw("View file diff"),
            ]),
            Line::from(vec![
                Span::styled("  u         ", Style::default().fg(Color::Green)),
                Span::raw("Stage/unstage selected file"),
            ]),
            Line::from(vec![
                Span::styled("  r         ", Style::default().fg(Color::Green)),
                Span::raw("Refresh status"),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "Diff View",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(vec![
                Span::styled("  j / Down  ", Style::default().fg(Color::Green)),
                Span::raw("Scroll down"),
            ]),
            Line::from(vec![
                Span::styled("  k / Up    ", Style::default().fg(Color::Green)),
                Span::raw("Scroll up"),
            ]),
            Line::from(vec![
                Span::styled("  g         ", Style::default().fg(Color::Green)),
                Span::raw("Jump to top"),
            ]),
            Line::from(vec![
                Span::styled("  G         ", Style::default().fg(Color::Green)),
                Span::raw("Jump to bottom"),
            ]),
            Line::from(vec![
                Span::styled("  PageUp    ", Style::default().fg(Color::Green)),
                Span::raw("Page up"),
            ]),
            Line::from(vec![
                Span::styled("  PageDown  ", Style::default().fg(Color::Green)),
                Span::raw("Page down"),
            ]),
            Line::from(vec![
                Span::styled("  Esc       ", Style::default().fg(Color::Green)),
                Span::raw("Close diff view"),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "About",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from("rust-tig is a terminal-based Git interface inspired by tig."),
            Line::from("Built with Rust, ratatui, and git2-rs."),
            Line::from(""),
            Line::from("Press q or Esc to close this help."),
        ]
    }

    /// Scroll down
    fn scroll_down(&mut self, amount: usize) {
        let max_scroll = self.lines.len().saturating_sub(1);
        self.scroll_offset = (self.scroll_offset + amount).min(max_scroll);
    }

    /// Scroll up
    fn scroll_up(&mut self, amount: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(amount);
    }

    /// Scroll to top
    fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    /// Scroll to bottom
    fn scroll_to_bottom(&mut self) {
        self.scroll_offset = self.lines.len().saturating_sub(1);
    }
}

impl View for HelpView {
    fn handle_key(&mut self, key: KeyEvent) -> Result<Action> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => Ok(Action::PopView),
            KeyCode::Char('j') | KeyCode::Down => {
                self.scroll_down(1);
                Ok(Action::None)
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.scroll_up(1);
                Ok(Action::None)
            }
            KeyCode::Char('g') => {
                self.scroll_to_top();
                Ok(Action::None)
            }
            KeyCode::Char('G') => {
                self.scroll_to_bottom();
                Ok(Action::None)
            }
            KeyCode::PageDown => {
                self.scroll_down(20);
                Ok(Action::None)
            }
            KeyCode::PageUp => {
                self.scroll_up(20);
                Ok(Action::None)
            }
            _ => Ok(Action::None),
        }
    }

    fn update(&mut self) -> Result<()> {
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let visible_height = area.height.saturating_sub(2) as usize;

        // Calculate scrollbar state
        let scrollbar_state = if self.lines.len() > visible_height {
            let state = ScrollbarState::default()
                .content_length(self.lines.len())
                .position(self.scroll_offset);
            Some(state)
        } else {
            None
        };

        // Get visible lines
        let visible_lines: Vec<Line> = self
            .lines
            .iter()
            .skip(self.scroll_offset)
            .take(visible_height)
            .cloned()
            .collect();

        let title = format!(
            "Help - {} / {} lines",
            self.scroll_offset + visible_lines.len().min(visible_height),
            self.lines.len()
        );

        let paragraph = Paragraph::new(visible_lines)
            .block(Block::default().title(title).borders(Borders::ALL));

        frame.render_widget(paragraph, area);

        // Render scrollbar if needed
        if let Some(mut state) = scrollbar_state {
            let scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"));
            frame.render_stateful_widget(
                scrollbar,
                area.inner(ratatui::layout::Margin {
                    vertical: 1,
                    horizontal: 0,
                }),
                &mut state,
            );
        }
    }

    fn title(&self) -> &str {
        "Help"
    }
}

impl Default for HelpView {
    fn default() -> Self {
        Self::new()
    }
}
