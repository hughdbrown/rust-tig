use super::event::Event;
use crate::{
    git::Repository,
    views::{Action, DiffView, HelpView, MainView, StatusView, ViewManager, ViewType},
};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

/// Main application state
pub struct App {
    running: bool,
    view_manager: ViewManager,
    repo: Option<Repository>,
    branch: Option<String>,
    error: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            running: true,
            view_manager: ViewManager::new(),
            repo: None,
            branch: None,
            error: None,
        }
    }

    /// Initialize the application with a repository
    pub async fn init(&mut self) -> Result<()> {
        match Repository::discover().await {
            Ok(repo) => {
                // Get branch name
                self.branch = repo.current_branch().await.ok().flatten();

                self.repo = Some(repo.clone());
                // Create and push the main view
                let main_view = MainView::new(repo);
                self.view_manager.push(Box::new(main_view))?;
                Ok(())
            }
            Err(e) => {
                self.error = Some(format!("Failed to open repository: {}", e));
                Err(e.into())
            }
        }
    }

    /// Check if the application should keep running
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Handle an event
    pub fn handle_event(&mut self, event: Event) -> Result<()> {
        match event {
            Event::Key(key) => self.handle_key(key),
            Event::Resize(_, _) => Ok(()),
            Event::Tick => Ok(()),
            Event::Mouse(_) => Ok(()),
        }
    }

    /// Handle a key event
    fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        // Global keybindings (Ctrl+C to quit)
        if key.code == KeyCode::Char('c') && key.modifiers == KeyModifiers::CONTROL {
            self.running = false;
            return Ok(());
        }

        // Delegate to view manager and handle actions
        let action = self.view_manager.handle_key(key)?;
        self.handle_action(action)?;

        Ok(())
    }

    /// Handle an action from a view
    fn handle_action(&mut self, action: Action) -> Result<()> {
        match action {
            Action::Quit => {
                self.running = false;
            }
            Action::SwitchView(_view_type) => {
                // TODO: Implement view switching
            }
            Action::PushView(view_type) => {
                match view_type {
                    ViewType::Status => {
                        if let Some(repo) = &self.repo {
                            let status_view = StatusView::new(repo.clone());
                            self.view_manager.push(Box::new(status_view))?;
                        }
                    }
                    ViewType::Help => {
                        let help_view = HelpView::new();
                        self.view_manager.push(Box::new(help_view))?;
                    }
                    _ => {
                        // Ignore other view types for now
                    }
                }
            }
            Action::PopView => {
                self.view_manager.pop().ok(); // Ignore error if can't pop
            }
            Action::OpenDiff {
                repo,
                commit_id,
                summary,
            } => {
                let diff_view = DiffView::new(repo, commit_id, summary);
                self.view_manager.push(Box::new(diff_view))?;
            }
            Action::OpenStagedDiff { repo, path } => {
                let diff_view = DiffView::new_staged(repo, path);
                self.view_manager.push(Box::new(diff_view))?;
            }
            Action::OpenUnstagedDiff { repo, path } => {
                let diff_view = DiffView::new_unstaged(repo, path);
                self.view_manager.push(Box::new(diff_view))?;
            }
            Action::None => {}
        }
        Ok(())
    }

    /// Update application state
    pub fn update(&mut self) -> Result<()> {
        self.view_manager.update()?;
        Ok(())
    }

    /// Render the application
    pub fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();

        // Show error screen if there's an error
        if let Some(error) = &self.error {
            self.render_error(frame, area, error);
            return;
        }

        // Create layout with status bar
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),    // Main content
                Constraint::Length(1), // Status bar
            ])
            .split(area);

        // Render current view
        self.view_manager.draw(frame, chunks[0]);

        // Render status bar
        self.render_status_bar(frame, chunks[1]);
    }

    /// Render error screen
    fn render_error(&self, frame: &mut Frame, area: Rect, error: &str) {
        let text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "Error",
                Style::default().fg(Color::Red),
            )),
            Line::from(""),
            Line::from(error),
            Line::from(""),
            Line::from("Press Ctrl+C to quit"),
        ];

        let paragraph = Paragraph::new(text);
        frame.render_widget(paragraph, area);
    }

    /// Render status bar
    fn render_status_bar(&self, frame: &mut Frame, area: Rect) {
        let branch = self
            .branch
            .as_ref()
            .map(|b| b.clone())
            .unwrap_or_else(|| "No branch".to_string());

        let view_title = self.view_manager.current_title();

        let status = Line::from(vec![
            Span::raw(" "),
            Span::styled(view_title, Style::default().fg(Color::Yellow)),
            Span::raw(" | "),
            Span::styled(format!("\u{e0a0} {}", branch), Style::default().fg(Color::Cyan)),
            Span::raw(" | "),
            Span::raw("q:quit | s:status | ?:help"),
        ]);

        let paragraph = Paragraph::new(status).style(Style::default().bg(Color::DarkGray));
        frame.render_widget(paragraph, area);
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn test_app_creation() {
        let app = App::new();
        assert!(app.is_running());
    }

    #[test]
    fn test_quit_on_ctrl_c() {
        let mut app = App::new();
        let event = Event::Key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
        app.handle_event(event).unwrap();
        assert!(!app.is_running());
    }
}
