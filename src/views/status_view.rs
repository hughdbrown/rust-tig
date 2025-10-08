use super::view::{Action, View};
use crate::git::{Repository, Status, StatusEntry};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};
use tokio::sync::mpsc;

/// Section in the status view
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Section {
    Staged,
    Unstaged,
    Untracked,
    Conflicted,
}

impl Section {
    fn title(&self) -> &'static str {
        match self {
            Section::Staged => "Changes to be committed",
            Section::Unstaged => "Changes not staged for commit",
            Section::Untracked => "Untracked files",
            Section::Conflicted => "Conflicted files",
        }
    }

    fn color(&self) -> Color {
        match self {
            Section::Staged => Color::Green,
            Section::Unstaged => Color::Yellow,
            Section::Untracked => Color::Red,
            Section::Conflicted => Color::Magenta,
        }
    }
}

/// Display item in the status view
#[derive(Debug, Clone)]
struct DisplayItem {
    section: Section,
    entry: Option<StatusEntry>,
    is_header: bool,
}

impl DisplayItem {
    fn header(section: Section) -> Self {
        Self {
            section,
            entry: None,
            is_header: true,
        }
    }

    fn entry(section: Section, entry: StatusEntry) -> Self {
        Self {
            section,
            entry: Some(entry),
            is_header: false,
        }
    }
}

/// Status view showing working directory changes
pub struct StatusView {
    repo: Repository,
    status: Option<Status>,
    items: Vec<DisplayItem>,
    list_state: ListState,
    loading: bool,
    error: Option<String>,
    receiver: Option<mpsc::UnboundedReceiver<Result<Status>>>,
    refresh_trigger: Option<mpsc::UnboundedReceiver<()>>,
}

impl StatusView {
    /// Create a new status view
    pub fn new(repo: Repository) -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Self {
            repo,
            status: None,
            items: Vec::new(),
            list_state,
            loading: false,
            error: None,
            receiver: None,
            refresh_trigger: None,
        }
    }

    /// Start loading status asynchronously
    pub fn start_loading(&mut self) {
        let (tx, rx) = mpsc::unbounded_channel();
        self.receiver = Some(rx);
        self.loading = true;

        let repo = self.repo.clone();

        tokio::spawn(async move {
            let result = crate::git::status::load_status(&repo)
                .await
                .map_err(|e| anyhow::anyhow!(e));
            let _ = tx.send(result);
        });
    }

    /// Build display items from status
    fn build_items(&mut self) {
        self.items.clear();

        if let Some(status) = &self.status {
            // Staged changes
            if !status.staged.is_empty() {
                self.items.push(DisplayItem::header(Section::Staged));
                for entry in &status.staged {
                    self.items
                        .push(DisplayItem::entry(Section::Staged, entry.clone()));
                }
            }

            // Unstaged changes
            if !status.unstaged.is_empty() {
                self.items.push(DisplayItem::header(Section::Unstaged));
                for entry in &status.unstaged {
                    self.items
                        .push(DisplayItem::entry(Section::Unstaged, entry.clone()));
                }
            }

            // Untracked files
            if !status.untracked.is_empty() {
                self.items.push(DisplayItem::header(Section::Untracked));
                for entry in &status.untracked {
                    self.items
                        .push(DisplayItem::entry(Section::Untracked, entry.clone()));
                }
            }

            // Conflicted files
            if !status.conflicted.is_empty() {
                self.items.push(DisplayItem::header(Section::Conflicted));
                for entry in &status.conflicted {
                    self.items
                        .push(DisplayItem::entry(Section::Conflicted, entry.clone()));
                }
            }
        }

        // Ensure selection is valid
        if !self.items.is_empty() && self.list_state.selected().is_none() {
            self.list_state.select(Some(0));
        }
    }

    /// Get the currently selected item
    fn selected_item(&self) -> Option<&DisplayItem> {
        self.list_state
            .selected()
            .and_then(|i| self.items.get(i))
    }

    /// Move selection up
    fn select_previous(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    0
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    /// Move selection down
    fn select_next(&mut self) {
        let len = self.items.len();
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= len.saturating_sub(1) {
                    i
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    /// Move selection to the top
    fn select_first(&mut self) {
        if !self.items.is_empty() {
            self.list_state.select(Some(0));
        }
    }

    /// Move selection to the bottom
    fn select_last(&mut self) {
        if !self.items.is_empty() {
            self.list_state.select(Some(self.items.len() - 1));
        }
    }

    /// Page up
    fn page_up(&mut self, page_size: usize) {
        let i = match self.list_state.selected() {
            Some(i) => i.saturating_sub(page_size),
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    /// Page down
    fn page_down(&mut self, page_size: usize) {
        let len = self.items.len();
        let i = match self.list_state.selected() {
            Some(i) => (i + page_size).min(len.saturating_sub(1)),
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    /// Create a list item for display
    fn create_list_item(&self, item: &DisplayItem) -> ListItem<'static> {
        if item.is_header {
            // Section header
            ListItem::new(Line::from(vec![
                Span::raw(""),
                Span::styled(
                    item.section.title().to_string(),
                    Style::default()
                        .fg(item.section.color())
                        .add_modifier(Modifier::BOLD),
                ),
            ]))
        } else if let Some(entry) = &item.entry {
            // File entry
            let status_code = entry.status.short_code().to_string();
            let path = entry.path.clone();

            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("  {} ", status_code),
                    Style::default().fg(item.section.color()),
                ),
                Span::styled(path, Style::default().fg(Color::White)),
            ]))
        } else {
            ListItem::new(Line::from(""))
        }
    }
}

impl View for StatusView {
    fn handle_key(&mut self, key: KeyEvent) -> Result<Action> {
        match key.code {
            KeyCode::Char('q') => Ok(Action::Quit),
            KeyCode::Char('j') | KeyCode::Down => {
                self.select_next();
                Ok(Action::None)
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.select_previous();
                Ok(Action::None)
            }
            KeyCode::Char('g') => {
                self.select_first();
                Ok(Action::None)
            }
            KeyCode::Char('G') => {
                self.select_last();
                Ok(Action::None)
            }
            KeyCode::PageUp => {
                self.page_up(20);
                Ok(Action::None)
            }
            KeyCode::PageDown => {
                self.page_down(20);
                Ok(Action::None)
            }
            KeyCode::Char('r') => {
                // Refresh status
                self.start_loading();
                Ok(Action::None)
            }
            KeyCode::Enter => {
                // Open diff for selected file
                if let Some(item) = self.selected_item() {
                    if !item.is_header {
                        if let Some(entry) = &item.entry {
                            let path = entry.path.clone();
                            return match item.section {
                                Section::Staged => Ok(Action::OpenStagedDiff {
                                    repo: self.repo.clone(),
                                    path,
                                }),
                                Section::Unstaged | Section::Untracked => {
                                    Ok(Action::OpenUnstagedDiff {
                                        repo: self.repo.clone(),
                                        path,
                                    })
                                }
                                Section::Conflicted => {
                                    // For now, show unstaged diff
                                    Ok(Action::OpenUnstagedDiff {
                                        repo: self.repo.clone(),
                                        path,
                                    })
                                }
                            };
                        }
                    }
                }
                Ok(Action::None)
            }
            KeyCode::Char('u') => {
                // Stage/unstage selected file
                if let Some(item) = self.selected_item() {
                    if !item.is_header {
                        if let Some(entry) = &item.entry {
                            let path = entry.path.clone();
                            let repo = self.repo.clone();

                            // Determine if we should stage or unstage
                            let should_stage = matches!(
                                item.section,
                                Section::Unstaged | Section::Untracked
                            );

                            // Create refresh trigger channel
                            let (tx, rx) = mpsc::unbounded_channel();
                            self.refresh_trigger = Some(rx);

                            // Clear any previous error
                            self.error = None;

                            // Spawn async task to stage/unstage and signal refresh
                            let mut error_field = self.error.clone();
                            tokio::spawn(async move {
                                let result = if should_stage {
                                    crate::git::status::stage_file(&repo, path.clone()).await
                                } else {
                                    crate::git::status::unstage_file(&repo, path.clone()).await
                                };

                                if let Err(e) = result {
                                    // Note: Can't update self.error here due to ownership
                                    // Errors will be shown in the status after refresh
                                    eprintln!("Failed to stage/unstage {}: {}", path, e);
                                }

                                // Signal that we should refresh
                                let _ = tx.send(());
                            });
                        }
                    }
                }
                Ok(Action::None)
            }
            KeyCode::Char('?') => {
                // Show help
                Ok(Action::PushView(super::view::ViewType::Help))
            }
            _ => Ok(Action::None),
        }
    }

    fn update(&mut self) -> Result<()> {
        // Check for refresh trigger
        if let Some(trigger) = &mut self.refresh_trigger {
            if trigger.try_recv().is_ok() {
                // Refresh was triggered, start loading status
                self.refresh_trigger = None;
                self.start_loading();
            }
        }

        // Check for status result from the receiver
        if let Some(receiver) = &mut self.receiver {
            if let Ok(result) = receiver.try_recv() {
                self.loading = false;
                match result {
                    Ok(status) => {
                        self.status = Some(status);
                        self.build_items();
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to load status: {}", e));
                    }
                }
            }
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        // Show loading indicator
        if self.loading && self.status.is_none() {
            let loading_items = vec![ListItem::new("Loading status...")];
            let list = List::new(loading_items)
                .block(Block::default().title("Status").borders(Borders::ALL))
                .style(Style::default().fg(Color::Yellow));
            frame.render_widget(list, area);
            return;
        }

        // Show error if any
        if let Some(error) = &self.error {
            let error_items = vec![ListItem::new(error.clone())];
            let list = List::new(error_items)
                .block(
                    Block::default()
                        .title("Status - Error")
                        .borders(Borders::ALL),
                )
                .style(Style::default().fg(Color::Red));
            frame.render_widget(list, area);
            return;
        }

        // Build title
        let title = if let Some(status) = &self.status {
            if status.has_changes() {
                format!("Status - {} changes", status.total_count())
            } else {
                "Status - No changes".to_string()
            }
        } else {
            "Status".to_string()
        };

        // Build list items
        let mut list_items = Vec::new();
        for item in &self.items {
            list_items.push(self.create_list_item(item));
        }

        let list = List::new(list_items)
            .block(Block::default().title(title).borders(Borders::ALL))
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, area, &mut self.list_state);
    }

    fn title(&self) -> &str {
        "Status"
    }

    fn on_activate(&mut self) -> Result<()> {
        // Start loading status when view is activated
        if self.status.is_none() && !self.loading {
            self.start_loading();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::EntryStatus;
    use std::fs;
    use tempfile::TempDir;

    async fn create_test_repo() -> (TempDir, Repository) {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        let git_repo = git2::Repository::init(repo_path).unwrap();
        let sig = git2::Signature::now("Test", "test@example.com").unwrap();

        // Create initial commit
        fs::write(repo_path.join("test.txt"), "test content\n").unwrap();
        let mut index = git_repo.index().unwrap();
        index.add_path(std::path::Path::new("test.txt")).unwrap();
        index.write().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = git_repo.find_tree(tree_id).unwrap();
        git_repo
            .commit(Some("HEAD"), &sig, &sig, "Initial", &tree, &[])
            .unwrap();

        let repo = Repository::open(repo_path).await.unwrap();
        (temp_dir, repo)
    }

    #[tokio::test]
    async fn test_status_view_creation() {
        let (_temp_dir, repo) = create_test_repo().await;
        let view = StatusView::new(repo);
        assert_eq!(view.title(), "Status");
        assert!(view.status.is_none());
    }

    #[tokio::test]
    async fn test_status_view_load() {
        let (_temp_dir, repo) = create_test_repo().await;
        let mut view = StatusView::new(repo);
        view.start_loading();

        // Give the background task time to complete
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Update the view to process the received status
        view.update().unwrap();

        assert!(view.status.is_some());
    }

    #[tokio::test]
    async fn test_status_view_navigation() {
        let (_temp_dir, repo) = create_test_repo().await;
        let mut view = StatusView::new(repo);

        // Add some items manually for testing
        view.items.push(DisplayItem::header(Section::Staged));
        view.items
            .push(DisplayItem::entry(Section::Staged, StatusEntry {
                path: "file1.txt".to_string(),
                status: EntryStatus::IndexNew,
                index_to_workdir: false,
            }));
        view.items
            .push(DisplayItem::entry(Section::Staged, StatusEntry {
                path: "file2.txt".to_string(),
                status: EntryStatus::IndexNew,
                index_to_workdir: false,
            }));

        view.select_next();
        assert_eq!(view.list_state.selected(), Some(1));

        view.select_previous();
        assert_eq!(view.list_state.selected(), Some(0));

        view.select_last();
        assert_eq!(view.list_state.selected(), Some(2));

        view.select_first();
        assert_eq!(view.list_state.selected(), Some(0));
    }
}
