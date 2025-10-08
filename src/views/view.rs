use crate::git::Repository;
use anyhow::Result;
use crossterm::event::KeyEvent;
use git2::Oid;
use ratatui::{layout::Rect, Frame};

/// Actions that views can request
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// No action
    None,
    /// Quit the application
    Quit,
    /// Switch to a different view
    SwitchView(ViewType),
    /// Push a new view onto the stack
    PushView(ViewType),
    /// Pop the current view and return to the previous one
    PopView,
    /// Open a diff view for a specific commit
    OpenDiff {
        repo: Repository,
        commit_id: Oid,
        summary: String,
    },
    /// Open a diff view for staged changes
    OpenStagedDiff {
        repo: Repository,
        path: String,
    },
    /// Open a diff view for unstaged changes
    OpenUnstagedDiff {
        repo: Repository,
        path: String,
    },
}

/// Types of views available
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewType {
    Main,
    Diff,
    Status,
    Help,
}

/// Trait that all views must implement
pub trait View {
    /// Handle a key event, returns an action to perform
    fn handle_key(&mut self, key: KeyEvent) -> Result<Action>;

    /// Update the view state (called on each frame)
    fn update(&mut self) -> Result<()>;

    /// Render the view
    fn draw(&mut self, frame: &mut Frame, area: Rect);

    /// Get the view's title (for status bar)
    fn title(&self) -> &str;

    /// Called when the view is activated (moved to foreground)
    fn on_activate(&mut self) -> Result<()> {
        Ok(())
    }

    /// Called when the view is deactivated (moved to background)
    fn on_deactivate(&mut self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_equality() {
        assert_eq!(Action::Quit, Action::Quit);
        assert_eq!(Action::None, Action::None);
        assert_ne!(Action::Quit, Action::None);
    }

    #[test]
    fn test_view_type_equality() {
        assert_eq!(ViewType::Main, ViewType::Main);
        assert_ne!(ViewType::Main, ViewType::Diff);
    }
}
