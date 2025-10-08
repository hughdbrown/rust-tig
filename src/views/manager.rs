use super::view::{Action, View, ViewType};
use anyhow::{anyhow, Result};
use crossterm::event::KeyEvent;
use ratatui::{layout::Rect, Frame};

/// Manages a stack of views and handles view switching
pub struct ViewManager {
    view_stack: Vec<Box<dyn View>>,
}

impl ViewManager {
    pub fn new() -> Self {
        Self {
            view_stack: Vec::new(),
        }
    }

    /// Push a view onto the stack
    pub fn push(&mut self, view: Box<dyn View>) -> Result<()> {
        if let Some(current) = self.view_stack.last_mut() {
            current.on_deactivate()?;
        }
        self.view_stack.push(view);
        if let Some(new) = self.view_stack.last_mut() {
            new.on_activate()?;
        }
        Ok(())
    }

    /// Pop the current view from the stack
    pub fn pop(&mut self) -> Result<()> {
        if self.view_stack.len() <= 1 {
            return Err(anyhow!("Cannot pop the last view"));
        }

        if let Some(mut old_view) = self.view_stack.pop() {
            old_view.on_deactivate()?;
        }

        if let Some(current) = self.view_stack.last_mut() {
            current.on_activate()?;
        }

        Ok(())
    }

    /// Replace the current view with a new one
    pub fn switch(&mut self, view: Box<dyn View>) -> Result<()> {
        if let Some(mut old_view) = self.view_stack.pop() {
            old_view.on_deactivate()?;
        }
        self.view_stack.push(view);
        if let Some(new) = self.view_stack.last_mut() {
            new.on_activate()?;
        }
        Ok(())
    }

    /// Get the current view
    pub fn current(&self) -> Option<&dyn View> {
        self.view_stack.last().map(|b| b.as_ref())
    }

    /// Get a mutable reference to the current view
    pub fn current_mut(&mut self) -> Option<&mut Box<dyn View>> {
        self.view_stack.last_mut()
    }

    /// Check if there are views in the stack
    pub fn is_empty(&self) -> bool {
        self.view_stack.is_empty()
    }

    /// Get the number of views in the stack
    pub fn len(&self) -> usize {
        self.view_stack.len()
    }

    /// Handle a key event, delegating to the current view
    pub fn handle_key(&mut self, key: KeyEvent) -> Result<Action> {
        if let Some(view) = self.current_mut() {
            view.handle_key(key)
        } else {
            Ok(Action::None)
        }
    }

    /// Update the current view
    pub fn update(&mut self) -> Result<()> {
        if let Some(view) = self.current_mut() {
            view.update()
        } else {
            Ok(())
        }
    }

    /// Render the current view
    pub fn draw(&mut self, frame: &mut Frame, area: Rect) {
        if let Some(view) = self.current_mut() {
            view.draw(frame, area);
        }
    }

    /// Get the title of the current view
    pub fn current_title(&self) -> &str {
        self.current()
            .map(|v| v.title())
            .unwrap_or("rust-tig")
    }
}

impl Default for ViewManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    // Mock view for testing
    struct MockView {
        title: String,
        activated: bool,
    }

    impl MockView {
        fn new(title: &str) -> Self {
            Self {
                title: title.to_string(),
                activated: false,
            }
        }
    }

    impl View for MockView {
        fn handle_key(&mut self, _key: KeyEvent) -> Result<Action> {
            Ok(Action::None)
        }

        fn update(&mut self) -> Result<()> {
            Ok(())
        }

        fn draw(&mut self, _frame: &mut Frame, _area: Rect) {}

        fn title(&self) -> &str {
            &self.title
        }

        fn on_activate(&mut self) -> Result<()> {
            self.activated = true;
            Ok(())
        }

        fn on_deactivate(&mut self) -> Result<()> {
            self.activated = false;
            Ok(())
        }
    }

    #[test]
    fn test_view_manager_creation() {
        let manager = ViewManager::new();
        assert!(manager.is_empty());
        assert_eq!(manager.len(), 0);
    }

    #[test]
    fn test_push_view() {
        let mut manager = ViewManager::new();
        let view = Box::new(MockView::new("Test"));
        manager.push(view).unwrap();
        assert_eq!(manager.len(), 1);
        assert_eq!(manager.current_title(), "Test");
    }

    #[test]
    fn test_pop_view() {
        let mut manager = ViewManager::new();
        manager.push(Box::new(MockView::new("View1"))).unwrap();
        manager.push(Box::new(MockView::new("View2"))).unwrap();

        assert_eq!(manager.len(), 2);
        manager.pop().unwrap();
        assert_eq!(manager.len(), 1);
        assert_eq!(manager.current_title(), "View1");
    }

    #[test]
    fn test_cannot_pop_last_view() {
        let mut manager = ViewManager::new();
        manager.push(Box::new(MockView::new("View1"))).unwrap();

        let result = manager.pop();
        assert!(result.is_err());
        assert_eq!(manager.len(), 1);
    }

    #[test]
    fn test_switch_view() {
        let mut manager = ViewManager::new();
        manager.push(Box::new(MockView::new("View1"))).unwrap();
        manager.switch(Box::new(MockView::new("View2"))).unwrap();

        assert_eq!(manager.len(), 1);
        assert_eq!(manager.current_title(), "View2");
    }
}
