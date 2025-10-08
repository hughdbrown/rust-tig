use super::view::{Action, View};
use crate::config::ColorScheme;
use crate::git::{Diff, DiffFile, DiffHunk, DiffLine, LineType, Repository};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use git2::Oid;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};
use tokio::sync::mpsc;

/// Source of the diff
#[derive(Debug, Clone)]
enum DiffSource {
    Commit { id: Oid, summary: String },
    StagedFile { path: String },
    UnstagedFile { path: String },
}

/// Diff view showing changes for a commit or file
pub struct DiffView {
    repo: Repository,
    source: DiffSource,
    diff: Option<Diff>,
    lines: Vec<Line<'static>>,
    scroll_offset: usize,
    loading: bool,
    error: Option<String>,
    receiver: Option<mpsc::UnboundedReceiver<Result<Diff>>>,
    colors: ColorScheme,
}

impl DiffView {
    /// Create a new diff view for a commit
    pub fn new(repo: Repository, commit_id: Oid, commit_summary: String, colors: ColorScheme) -> Self {
        Self {
            repo,
            source: DiffSource::Commit {
                id: commit_id,
                summary: commit_summary,
            },
            diff: None,
            lines: Vec::new(),
            scroll_offset: 0,
            loading: false,
            error: None,
            receiver: None,
            colors,
        }
    }

    /// Create a new diff view for staged changes
    pub fn new_staged(repo: Repository, path: String, colors: ColorScheme) -> Self {
        Self {
            repo,
            source: DiffSource::StagedFile { path },
            diff: None,
            lines: Vec::new(),
            scroll_offset: 0,
            loading: false,
            error: None,
            receiver: None,
            colors,
        }
    }

    /// Create a new diff view for unstaged changes
    pub fn new_unstaged(repo: Repository, path: String, colors: ColorScheme) -> Self {
        Self {
            repo,
            source: DiffSource::UnstagedFile { path },
            diff: None,
            lines: Vec::new(),
            scroll_offset: 0,
            loading: false,
            error: None,
            receiver: None,
            colors,
        }
    }

    /// Start loading the diff asynchronously
    pub fn start_loading(&mut self) {
        let (tx, rx) = mpsc::unbounded_channel();
        self.receiver = Some(rx);
        self.loading = true;

        let repo = self.repo.clone();
        let source = self.source.clone();

        tokio::spawn(async move {
            let result = match source {
                DiffSource::Commit { id, .. } => {
                    crate::git::diff::load_commit_diff(&repo, id).await
                }
                DiffSource::StagedFile { path } => {
                    crate::git::diff::load_staged_diff(&repo, Some(path)).await
                }
                DiffSource::UnstagedFile { path } => {
                    crate::git::diff::load_unstaged_diff(&repo, Some(path)).await
                }
            }
            .map_err(|e| anyhow::anyhow!(e));
            let _ = tx.send(result);
        });
    }

    /// Convert a Diff into styled lines for rendering
    fn render_diff_to_lines(&self, diff: &Diff) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        // Show header based on source
        match &self.source {
            DiffSource::Commit { id, summary } => {
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("commit {}", id),
                        Style::default().fg(self.colors.commit_hash).add_modifier(Modifier::BOLD),
                    ),
                ]));
                lines.push(Line::from(Span::styled(
                    summary.clone(),
                    // Style::default().fg(Color::White),
                    Style::default().fg(self.colors.modified),
                )));
            }
            DiffSource::StagedFile { path } => {
                lines.push(Line::from(vec![
                    Span::styled(
                        "Staged changes: ",
                        Style::default().fg(self.colors.added).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        path.clone(),
                        // Style::default().fg(Color::White),
                        Style::default().fg(self.colors.modified),
                    ),
                ]));
            }
            DiffSource::UnstagedFile { path } => {
                lines.push(Line::from(vec![
                    Span::styled(
                        "Unstaged changes: ",
                        Style::default().fg(self.colors.modified).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        path.clone(), 
                        // Style::default().fg(Color::White),
                        Style::default().fg(self.colors.modified),
                    ),
                ]));
            }
        }
        lines.push(Line::from(""));

        // Render each file
        for file in &diff.files {
            self.render_file_to_lines(&mut lines, file);
        }

        // Show summary at the bottom
        let (additions, deletions) = diff.total_stats();
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(
                format!("{} file(s) changed, ", diff.files.len()),
                // Style::default().fg(Color::White),
                Style::default().fg(self.colors.modified),
            ),
            Span::styled(
                format!("+{} ", additions),
                Style::default().fg(self.colors.added),
            ),
            Span::styled(
                format!("-{}", deletions),
                Style::default().fg(self.colors.deleted),
            ),
        ]));

        lines
    }

    /// Render a single file to lines
    fn render_file_to_lines(&self, lines: &mut Vec<Line<'static>>, file: &DiffFile) {
        // File header
        let file_line = match (&file.old_path, &file.new_path) {
            (Some(old), Some(new)) if old != new => {
                format!("diff --git a/{} b/{}", old, new)
            }
            (Some(old), None) => format!("diff --git a/{} (deleted)", old),
            (None, Some(new)) => format!("diff --git a/{} (new)", new),
            (Some(path), _) | (None, Some(path)) => {
                format!("diff --git a/{} b/{}", path, path)
            }
            (None, None) => "diff --git (unknown)".to_string(),
        };

        lines.push(Line::from(Span::styled(
            file_line,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )));

        // File statistics
        let stats = file.stats_summary();
        lines.push(Line::from(Span::styled(
            stats,
            Style::default().fg(Color::Cyan),
        )));

        // Binary file indicator
        if file.is_binary {
            lines.push(Line::from(Span::styled(
                "Binary file",
                Style::default().fg(self.colors.modified),
            )));
            lines.push(Line::from(""));
            return;
        }

        // Render hunks
        for hunk in &file.hunks {
            self.render_hunk_to_lines(lines, hunk);
        }

        lines.push(Line::from(""));
    }

    /// Render a single hunk to lines
    fn render_hunk_to_lines(&self, lines: &mut Vec<Line<'static>>, hunk: &DiffHunk) {
        // Hunk header - clone to own the string
        let header = hunk.header.clone();
        lines.push(Line::from(Span::styled(
            header,
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )));

        // Hunk lines
        for line in &hunk.lines {
            lines.push(self.render_diff_line(line));
        }
    }

    /// Render a single diff line
    fn render_diff_line(&self, line: &DiffLine) -> Line<'static> {
        let (style, prefix) = match line.line_type {
            LineType::Addition => (Style::default().fg(self.colors.added), "+"),
            LineType::Deletion => (Style::default().fg(self.colors.deleted), "-"),
            // LineType::Context => (Style::default().fg(Color::White), " "),
            LineType::Context => (Style::default().fg(self.colors.modified), " "),
            LineType::FileHeader => (
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
                "",
            ),
            LineType::HunkHeader => (
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
                "",
            ),
        };

        // Build line with optional line numbers
        let line_num = match (line.old_lineno, line.new_lineno) {
            (Some(old), Some(new)) => format!("{:4} {:4} ", old, new),
            (Some(old), None) => format!("{:4}      ", old),
            (None, Some(new)) => format!("     {:4} ", new),
            (None, None) => "          ".to_string(),
        };

        let mut content = line.content.clone();
        // Remove trailing newline if present
        if content.ends_with('\n') {
            content.pop();
        }

        Line::from(vec![
            Span::styled(line_num, Style::default().fg(Color::DarkGray)),
            Span::styled(prefix.to_string(), style),
            Span::styled(content, style),
        ])
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

    /// Page down
    fn page_down(&mut self, page_size: usize) {
        self.scroll_down(page_size);
    }

    /// Page up
    fn page_up(&mut self, page_size: usize) {
        self.scroll_up(page_size);
    }
}

impl View for DiffView {
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
                self.page_down(20);
                Ok(Action::None)
            }
            KeyCode::PageUp => {
                self.page_up(20);
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
        // Check for diff result from the receiver
        if let Some(receiver) = &mut self.receiver {
            if let Ok(result) = receiver.try_recv() {
                self.loading = false;
                match result {
                    Ok(diff) => {
                        self.lines = self.render_diff_to_lines(&diff);
                        self.diff = Some(diff);
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to load diff: {}", e));
                    }
                }
            }
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        // Calculate visible area (accounting for borders)
        let visible_height = area.height.saturating_sub(2) as usize;

        // Show loading indicator
        if self.loading {
            let loading_text = "Loading diff...";
            let paragraph = Paragraph::new(loading_text)
                .block(Block::default().title("Diff").borders(Borders::ALL))
                .style(Style::default().fg(self.colors.modified));
            frame.render_widget(paragraph, area);
            return;
        }

        // Show error if any
        if let Some(error) = &self.error {
            let paragraph = Paragraph::new(error.as_str())
                .block(Block::default().title("Diff - Error").borders(Borders::ALL))
                .style(Style::default().fg(self.colors.deleted));
            frame.render_widget(paragraph, area);
            return;
        }

        // Calculate scrollbar position
        let scrollbar_state = if self.lines.len() > visible_height {
            let state = ScrollbarState::default()
                .content_length(self.lines.len())
                .position(self.scroll_offset);
            Some(state)
        } else {
            None
        };

        // Show diff content
        let visible_lines: Vec<Line> = self
            .lines
            .iter()
            .skip(self.scroll_offset)
            .take(visible_height)
            .cloned()
            .collect();

        let title = format!(
            "Diff - {} / {} lines",
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
        "Diff"
    }

    fn on_activate(&mut self) -> Result<()> {
        // Start loading diff when view is activated
        if self.diff.is_none() && !self.loading {
            self.start_loading();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_test_repo_with_commit() -> (TempDir, Repository, Oid) {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        let git_repo = git2::Repository::init(repo_path).unwrap();
        let sig = git2::Signature::now("Test", "test@example.com").unwrap();

        // Create initial commit
        std::fs::write(repo_path.join("test.txt"), "line1\nline2\nline3\n").unwrap();
        let mut index = git_repo.index().unwrap();
        index.add_path(std::path::Path::new("test.txt")).unwrap();
        index.write().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = git_repo.find_tree(tree_id).unwrap();
        let commit_id = git_repo
            .commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
            .unwrap();

        let repo = Repository::open(repo_path).await.unwrap();
        (temp_dir, repo, commit_id)
    }

    fn test_color_scheme() -> ColorScheme {
        use crate::config::Config;
        ColorScheme::from_config(&Config::default().colors)
    }

    #[tokio::test]
    async fn test_diff_view_creation() {
        let (_temp_dir, repo, commit_id) = create_test_repo_with_commit().await;
        let view = DiffView::new(repo, commit_id, "Test commit".to_string(), test_color_scheme());
        assert_eq!(view.title(), "Diff");
        assert!(view.diff.is_none());
    }

    #[tokio::test]
    async fn test_diff_view_load() {
        let (_temp_dir, repo, commit_id) = create_test_repo_with_commit().await;
        let mut view = DiffView::new(repo, commit_id, "Test commit".to_string(), test_color_scheme());
        view.start_loading();

        // Give the background task time to complete
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Update the view to process the received diff
        view.update().unwrap();

        assert!(view.diff.is_some());
        assert!(!view.lines.is_empty());
    }

    #[tokio::test]
    async fn test_diff_view_scrolling() {
        let (_temp_dir, repo, commit_id) = create_test_repo_with_commit().await;
        let mut view = DiffView::new(repo, commit_id, "Test commit".to_string(), test_color_scheme());
        view.start_loading();

        // Give the background task time to complete
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        view.update().unwrap();

        // Test scrolling
        let initial_offset = view.scroll_offset;
        view.scroll_down(5);
        assert!(view.scroll_offset > initial_offset);

        view.scroll_to_top();
        assert_eq!(view.scroll_offset, 0);

        view.scroll_to_bottom();
        assert!(view.scroll_offset > 0);
    }
}
