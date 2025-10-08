use super::view::{Action, View};
use crate::git::{Commit, CommitWalker, Repository};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Row, Table, TableState},
    Frame,
};
use tokio::sync::mpsc;

/// Search mode state
#[derive(Debug, Clone, Copy, PartialEq)]
enum SearchMode {
    Inactive,
    Active,
}

/// Main view showing commit history
pub struct MainView {
    repo: Repository,
    commits: Vec<Commit>,
    filtered_commits: Vec<usize>, // Indices into commits vec
    table_state: TableState,
    loading: bool,
    error: Option<String>,
    receiver: Option<mpsc::UnboundedReceiver<Vec<Commit>>>,
    search_mode: SearchMode,
    search_query: String,
}

impl MainView {
    /// Create a new main view
    pub fn new(repo: Repository) -> Self {
        let mut table_state = TableState::default();
        table_state.select(Some(0));

        Self {
            repo,
            commits: Vec::new(),
            filtered_commits: Vec::new(),
            table_state,
            loading: false,
            error: None,
            receiver: None,
            search_mode: SearchMode::Inactive,
            search_query: String::new(),
        }
    }

    /// Start loading commits asynchronously
    pub fn start_loading(&mut self) {
        let (tx, rx) = mpsc::unbounded_channel();
        self.receiver = Some(rx);
        self.loading = true;

        let walker = CommitWalker::new(self.repo.clone()).with_chunk_size(50);

        tokio::spawn(async move {
            if let Err(e) = walker.walk(tx).await {
                eprintln!("Error walking commits: {}", e);
            }
        });
    }

    /// Get the currently selected commit
    pub fn selected_commit(&self) -> Option<&Commit> {
        self.table_state.selected().and_then(|i| {
            if self.is_searching() {
                self.filtered_commits.get(i).and_then(|&idx| self.commits.get(idx))
            } else {
                self.commits.get(i)
            }
        })
    }

    /// Get the list of commits to display (filtered or all)
    fn displayed_commits(&self) -> Vec<&Commit> {
        if self.is_searching() {
            self.filtered_commits
                .iter()
                .filter_map(|&i| self.commits.get(i))
                .collect()
        } else {
            self.commits.iter().collect()
        }
    }

    /// Check if search mode is active
    fn is_searching(&self) -> bool {
        self.search_mode == SearchMode::Active && !self.search_query.is_empty()
    }

    /// Update the search filter
    fn update_search_filter(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_commits.clear();
            return;
        }

        let query = self.search_query.to_lowercase();
        self.filtered_commits = self
            .commits
            .iter()
            .enumerate()
            .filter(|(_, commit)| {
                commit.summary.to_lowercase().contains(&query)
                    || commit.author.to_lowercase().contains(&query)
                    || commit.short_id.to_lowercase().contains(&query)
            })
            .map(|(i, _)| i)
            .collect();

        // Reset selection to first result
        if !self.filtered_commits.is_empty() {
            self.table_state.select(Some(0));
        }
    }

    /// Enter search mode
    fn enter_search_mode(&mut self) {
        self.search_mode = SearchMode::Active;
        self.search_query.clear();
        self.filtered_commits.clear();
    }

    /// Exit search mode
    fn exit_search_mode(&mut self) {
        self.search_mode = SearchMode::Inactive;
        self.search_query.clear();
        self.filtered_commits.clear();
        self.table_state.select(Some(0));
    }

    /// Add a character to the search query
    fn search_add_char(&mut self, c: char) {
        self.search_query.push(c);
        self.update_search_filter();
    }

    /// Remove the last character from the search query
    fn search_backspace(&mut self) {
        self.search_query.pop();
        self.update_search_filter();
    }

    /// Move selection up
    fn select_previous(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    0
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    /// Move selection down
    fn select_next(&mut self) {
        let len = if self.is_searching() {
            self.filtered_commits.len()
        } else {
            self.commits.len()
        };

        let i = match self.table_state.selected() {
            Some(i) => {
                if i >= len.saturating_sub(1) {
                    i
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    /// Move selection to the top
    fn select_first(&mut self) {
        self.table_state.select(Some(0));
    }

    /// Move selection to the bottom
    fn select_last(&mut self) {
        let len = if self.is_searching() {
            self.filtered_commits.len()
        } else {
            self.commits.len()
        };

        if len > 0 {
            self.table_state.select(Some(len - 1));
        }
    }

    /// Page up
    fn page_up(&mut self, page_size: usize) {
        let i = match self.table_state.selected() {
            Some(i) => i.saturating_sub(page_size),
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    /// Page down
    fn page_down(&mut self, page_size: usize) {
        let len = if self.is_searching() {
            self.filtered_commits.len()
        } else {
            self.commits.len()
        };

        let i = match self.table_state.selected() {
            Some(i) => (i + page_size).min(len.saturating_sub(1)),
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    /// Create a table row for a commit
    fn create_commit_row<'a>(&self, commit: &'a Commit) -> Row<'a> {
        let hash = Span::styled(&commit.short_id, Style::default().fg(Color::Yellow));

        let date = Span::styled(commit.relative_date(), Style::default().fg(Color::Blue));

        let author = Span::styled(&commit.author, Style::default().fg(Color::Green));

        let refs = if commit.refs.is_empty() {
            Span::raw("")
        } else {
            Span::styled(
                format!(" [{}]", commit.refs.join(", ")),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
        };

        let message = Span::raw(&commit.summary);

        Row::new(vec![
            Line::from(hash),
            Line::from(date),
            Line::from(author),
            Line::from(vec![refs, message]),
        ])
    }
}

impl View for MainView {
    fn handle_key(&mut self, key: KeyEvent) -> Result<Action> {
        // Handle search mode separately
        if self.search_mode == SearchMode::Active {
            match key.code {
                KeyCode::Esc => {
                    self.exit_search_mode();
                    return Ok(Action::None);
                }
                KeyCode::Enter => {
                    // Keep the search results but exit search input mode
                    self.search_mode = SearchMode::Inactive;
                    return Ok(Action::None);
                }
                KeyCode::Backspace => {
                    self.search_backspace();
                    return Ok(Action::None);
                }
                KeyCode::Char(c) => {
                    self.search_add_char(c);
                    return Ok(Action::None);
                }
                _ => return Ok(Action::None),
            }
        }

        // Normal mode keybindings
        match key.code {
            KeyCode::Char('q') => Ok(Action::Quit),
            KeyCode::Char('/') => {
                self.enter_search_mode();
                Ok(Action::None)
            }
            KeyCode::Esc if self.is_searching() => {
                self.exit_search_mode();
                Ok(Action::None)
            }
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
            KeyCode::Enter => {
                // Open diff view for selected commit
                if let Some(commit) = self.selected_commit() {
                    Ok(Action::OpenDiff {
                        repo: self.repo.clone(),
                        commit_id: commit.id,
                        summary: commit.summary.clone(),
                    })
                } else {
                    Ok(Action::None)
                }
            }
            KeyCode::Char('s') => {
                // Open status view
                Ok(Action::PushView(super::view::ViewType::Status))
            }
            KeyCode::Char('?') => {
                // Show help
                Ok(Action::PushView(super::view::ViewType::Help))
            }
            _ => Ok(Action::None),
        }
    }

    fn update(&mut self) -> Result<()> {
        // Check for new commits from the receiver
        if let Some(receiver) = &mut self.receiver {
            while let Ok(chunk) = receiver.try_recv() {
                self.commits.extend(chunk);
            }
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        // Create the table rows from appropriate commits
        let rows: Vec<Row> = if self.is_searching() {
            self.filtered_commits
                .iter()
                .filter_map(|&i| self.commits.get(i))
                .map(|commit| {
                    self.create_commit_row(commit)
                })
                .collect()
        } else {
            self.commits
                .iter()
                .map(|commit| {
                    self.create_commit_row(commit)
                })
                .collect()
        };

        let displayed_count = rows.len();

        let widths = [
            Constraint::Length(8),      // Hash
            Constraint::Length(18),     // Date
            Constraint::Length(20),     // Author
            Constraint::Percentage(50), // Message
        ];

        // Title shows search status
        let title = if self.is_searching() {
            format!(
                "Main - {} / {} commits (filtered)",
                displayed_count,
                self.commits.len()
            )
        } else if self.search_mode == SearchMode::Active {
            format!("Search: {}_", self.search_query)
        } else {
            format!("Main - {} commits", self.commits.len())
        };

        let table = Table::new(rows, widths)
            .block(Block::default().title(title).borders(Borders::ALL))
            .row_highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");

        frame.render_stateful_widget(table, area, &mut self.table_state);

        // Show loading indicator
        if self.loading && self.commits.is_empty() {
            let loading_text = "Loading commits...";
            let x = area.x + (area.width.saturating_sub(loading_text.len() as u16)) / 2;
            let y = area.y + area.height / 2;
            if x < area.x + area.width && y < area.y + area.height {
                frame.render_widget(
                    ratatui::widgets::Paragraph::new(loading_text)
                        .style(Style::default().fg(Color::Yellow)),
                    Rect::new(x, y, loading_text.len() as u16, 1),
                );
            }
        }

        // Show error if any
        if let Some(error) = &self.error {
            let error_text = format!("Error: {}", error);
            frame.render_widget(
                ratatui::widgets::Paragraph::new(error_text)
                    .style(Style::default().fg(Color::Red)),
                Rect::new(area.x + 1, area.y + 1, area.width - 2, 1),
            );
        }
    }

    fn title(&self) -> &str {
        "Main"
    }

    fn on_activate(&mut self) -> Result<()> {
        // Start loading commits when the view is activated
        if self.commits.is_empty() && !self.loading {
            self.start_loading();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_test_repo() -> (TempDir, Repository) {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        let git_repo = git2::Repository::init(repo_path).unwrap();
        let sig = git2::Signature::now("Test", "test@example.com").unwrap();
        let tree_id = git_repo.index().unwrap().write_tree().unwrap();
        let tree = git_repo.find_tree(tree_id).unwrap();
        git_repo
            .commit(Some("HEAD"), &sig, &sig, "Initial", &tree, &[])
            .unwrap();

        let repo = Repository::open(repo_path).await.unwrap();
        (temp_dir, repo)
    }

    #[tokio::test]
    async fn test_main_view_creation() {
        let (_temp_dir, repo) = create_test_repo().await;
        let view = MainView::new(repo);
        assert_eq!(view.commits.len(), 0);
    }

    #[tokio::test]
    async fn test_main_view_navigation() {
        let (_temp_dir, repo) = create_test_repo().await;
        let mut view = MainView::new(repo);

        // Add some dummy commits for testing navigation
        for i in 0..10 {
            view.commits.push(Commit {
                id: git2::Oid::zero(),
                short_id: format!("commit{}", i),
                author: "Test".to_string(),
                author_email: "test@example.com".to_string(),
                date: chrono::Local::now(),
                summary: format!("Commit {}", i),
                message: format!("Commit {}", i),
                refs: vec![],
            });
        }

        view.select_next();
        assert_eq!(view.table_state.selected(), Some(1));

        view.select_previous();
        assert_eq!(view.table_state.selected(), Some(0));

        view.select_last();
        assert_eq!(view.table_state.selected(), Some(9));

        view.select_first();
        assert_eq!(view.table_state.selected(), Some(0));
    }

    #[tokio::test]
    async fn test_main_view_search() {
        let (_temp_dir, repo) = create_test_repo().await;
        let mut view = MainView::new(repo);

        // Add test commits
        view.commits.push(Commit {
            id: git2::Oid::zero(),
            short_id: "abc123".to_string(),
            author: "Test".to_string(),
            author_email: "test@example.com".to_string(),
            date: chrono::Local::now(),
            summary: "Fix bug in parser".to_string(),
            message: "Fix bug in parser".to_string(),
            refs: vec![],
        });
        view.commits.push(Commit {
            id: git2::Oid::zero(),
            short_id: "def456".to_string(),
            author: "Test".to_string(),
            author_email: "test@example.com".to_string(),
            date: chrono::Local::now(),
            summary: "Add new feature".to_string(),
            message: "Add new feature".to_string(),
            refs: vec![],
        });
        view.commits.push(Commit {
            id: git2::Oid::zero(),
            short_id: "ghi789".to_string(),
            author: "Test".to_string(),
            author_email: "test@example.com".to_string(),
            date: chrono::Local::now(),
            summary: "Fix typo".to_string(),
            message: "Fix typo".to_string(),
            refs: vec![],
        });

        // Enter search mode
        view.enter_search_mode();
        assert!(matches!(view.search_mode, SearchMode::Active));

        // Search for "fix"
        view.search_add_char('f');
        view.search_add_char('i');
        view.search_add_char('x');

        // Should find 2 commits with "fix" (case-insensitive)
        assert_eq!(view.filtered_commits.len(), 2);

        // Test backspace
        view.search_backspace();
        assert_eq!(view.search_query, "fi");

        // Exit search mode
        view.exit_search_mode();
        assert_eq!(view.filtered_commits.len(), 0);
        assert_eq!(view.search_query, "");
    }

    #[tokio::test]
    async fn test_main_view_search_empty_query() {
        let (_temp_dir, repo) = create_test_repo().await;
        let mut view = MainView::new(repo);

        view.commits.push(Commit {
            id: git2::Oid::zero(),
            short_id: "abc123".to_string(),
            author: "Test".to_string(),
            author_email: "test@example.com".to_string(),
            date: chrono::Local::now(),
            summary: "Test commit".to_string(),
            message: "Test commit".to_string(),
            refs: vec![],
        });

        // Enter search with empty query
        view.enter_search_mode();
        assert_eq!(view.filtered_commits.len(), 0);
    }

    #[tokio::test]
    async fn test_main_view_search_no_matches() {
        let (_temp_dir, repo) = create_test_repo().await;
        let mut view = MainView::new(repo);

        view.commits.push(Commit {
            id: git2::Oid::zero(),
            short_id: "abc123".to_string(),
            author: "Test".to_string(),
            author_email: "test@example.com".to_string(),
            date: chrono::Local::now(),
            summary: "Test commit".to_string(),
            message: "Test commit".to_string(),
            refs: vec![],
        });

        view.enter_search_mode();
        view.search_add_char('x');
        view.search_add_char('y');
        view.search_add_char('z');

        // Should find no matches
        assert_eq!(view.filtered_commits.len(), 0);
    }

    #[tokio::test]
    async fn test_main_view_page_navigation() {
        let (_temp_dir, repo) = create_test_repo().await;
        let mut view = MainView::new(repo);

        // Add 50 commits
        for i in 0..50 {
            view.commits.push(Commit {
                id: git2::Oid::zero(),
                short_id: format!("{:07x}", i),
                author: "Test".to_string(),
                author_email: "test@example.com".to_string(),
                date: chrono::Local::now(),
                summary: format!("Commit {}", i),
                message: format!("Commit {}", i),
                refs: vec![],
            });
        }

        // Test page down
        view.page_down(20);
        assert_eq!(view.table_state.selected(), Some(20));

        // Test page up
        view.page_up(10);
        assert_eq!(view.table_state.selected(), Some(10));

        // Test page down near end
        view.select_last();
        view.page_down(20);
        assert_eq!(view.table_state.selected(), Some(49));
    }
}
