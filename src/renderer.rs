//! Interactive TUI renderer using ratatui.
//!
//! Provides the `TuiApp` struct which manages terminal state, input handling,
//! filtering, pagination, and renders an interactive ASCII workflow diagram.

use crate::config::AppConfig;
use crate::model::{FilterMode, FlatStep, SearchFilter, Status, Workflow};
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use std::time::Duration;

/// Number of reserved rows: title (1) + status bar (1) + detail panel header (2).
const RESERVED_ROWS: u16 = 4;

/// Scroll page size for PageUp/PageDown.
const PAGE_SIZE: usize = 10;

/// Application state for the interactive TUI.
pub struct TuiApp {
    /// The parsed workflow model.
    pub workflow: Workflow,
    /// Cursor index into the currently filtered step list.
    cursor: usize,
    /// Scroll offset for viewport pagination.
    scroll_offset: usize,
    /// Current filter mode (cycled with 'f').
    filter_mode: FilterMode,
    /// Optional search filter (activated with '/').
    search: SearchFilter,
    /// Index into the filtered list of the expanded step, if any.
    selected_step: Option<usize>,
    /// Whether the help overlay is shown.
    show_help: bool,
    /// Whether we are in search input mode (inline prompt at bottom).
    search_mode: bool,
    /// Current search input buffer.
    search_buffer: String,
    /// Application configuration.
    pub config: AppConfig,
    /// Timestamp of the last reload for watch mode display.
    pub last_reload: Option<String>,
    /// Flag indicating the app should quit.
    should_quit: bool,
    /// Flag indicating export was requested.
    export_requested: bool,
    /// Flag indicating reload was requested.
    reload_requested: bool,
}

impl TuiApp {
    /// Create a new TUI application from a loaded workflow and config.
    pub fn new(workflow: Workflow, config: AppConfig) -> Self {
        Self {
            workflow,
            cursor: 0,
            scroll_offset: 0,
            filter_mode: FilterMode::All,
            search: SearchFilter::default(),
            selected_step: None,
            show_help: false,
            search_mode: false,
            search_buffer: String::new(),
            config,
            last_reload: None,
            should_quit: false,
            export_requested: false,
            reload_requested: false,
        }
    }

    /// Returns the filtered and searched list of steps.
    fn filtered_steps(&self) -> Vec<FlatStep> {
        let all = self.workflow.flatten_steps();

        // Apply filter mode
        let mut filtered: Vec<FlatStep> = all
            .into_iter()
            .filter(|s| self.filter_mode.matches(&s.status))
            .collect();

        // Apply search on top
        if self.search.query.is_some() {
            filtered.retain(|s| self.search.matches(s));
        }

        filtered
    }

    /// Clamp cursor to the filtered list length.
    fn clamp_cursor(&mut self, len: usize) {
        if len == 0 {
            self.cursor = 0;
        } else if self.cursor >= len {
            self.cursor = len - 1;
        }
    }

    /// Adjust scroll offset so cursor is visible.
    fn adjust_scroll_offset(&mut self, viewport_height: usize) {
        if self.cursor < self.scroll_offset {
            self.scroll_offset = self.cursor;
        } else if self.cursor >= self.scroll_offset + viewport_height {
            self.scroll_offset = self.cursor.saturating_sub(viewport_height - 1);
        }
    }

    /// Run the interactive TUI event loop.
    pub fn run(&mut self) -> Result<()> {
        let mut terminal = ratatui::init();

        let result = self.event_loop(&mut terminal);

        ratatui::restore();
        result
    }

    /// Core event loop: draw then poll for input.
    fn event_loop(
        &mut self,
        terminal: &mut ratatui::Terminal<impl ratatui::backend::Backend>,
    ) -> Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;

            if self.should_quit {
                break;
            }

            if self.export_requested {
                self.export_current_view()?;
                self.export_requested = false;
            }

            // Poll for key events with a short timeout so we can update the display
            if event::poll(Duration::from_millis(200))? {
                if let Event::Key(key) = event::read()? {
                    if self.search_mode {
                        self.handle_search_input(key);
                    } else {
                        self.handle_input(key);
                    }
                }
            }
        }
        Ok(())
    }

    /// Format a simple timestamp string from a Duration since epoch.
    fn format_timestamp(dur: Duration) -> String {
        let secs = dur.as_secs();
        let hours = (secs / 3600) % 24;
        let minutes = (secs / 60) % 60;
        let seconds = secs % 60;
        format!("{hours:02}:{minutes:02}:{seconds:02}")
    }

    /// Export the current view to the default export path.
    fn export_current_view(&self) -> Result<()> {
        // Build a text representation of the currently visible steps
        let filtered = self.filtered_steps();
        let mut lines: Vec<String> = Vec::new();

        lines.push(format!(
            "# Workflow: {} ({})",
            self.workflow.name, self.workflow.framework
        ));
        lines.push(String::new());

        for (i, step) in filtered.iter().enumerate() {
            let indent = "  ".repeat(step.depth);
            let marker = step.status.marker();
            let cursor_mark = if i == self.cursor { ">" } else { " " };
            let collapse = if step.has_children {
                if step.collapsed {
                    "[+]"
                } else {
                    "[-]"
                }
            } else {
                "   "
            };
            let fail_mark = if step.status.is_error() { "*" } else { " " };
            lines.push(format!(
                "{} {} {}{}[{}] {} ({}#{})",
                cursor_mark, fail_mark, indent, collapse, marker, step.name, step.step_type, step.id
            ));
            if let Some(ref err) = step.error {
                lines.push(format!("      Error: {}", err.message));
                if let Some(ref sug) = err.suggestion {
                    lines.push(format!("      Suggestion: {sug}"));
                }
            }
            lines.push(format!("      Source: {}", step.source_location));
            if !step.config_snippet.is_empty() {
                for snippet_line in step.config_snippet.lines().take(6) {
                    lines.push(format!("      | {snippet_line}"));
                }
            }
        }

        lines.push(String::new());
        lines.push(format!(
            "Filter: {} | Search: {}",
            self.filter_mode.label(),
            self.search.query.as_deref().unwrap_or("none")
        ));

        let output = lines.join("\n");

        // Determine export path
        let export_path = if let Some(config_dir) = dirs::config_dir() {
            let dir = config_dir.join("workflow-map");
            std::fs::create_dir_all(&dir)?;
            dir.join("last-export.txt")
        } else {
            std::path::PathBuf::from("last-export.txt")
        };

        std::fs::write(&export_path, output)?;
        Ok(())
    }

    /// Handle key input while in search mode.
    fn handle_search_input(&mut self, key: event::KeyEvent) {
        match key.code {
            KeyCode::Enter => {
                let query = self.search_buffer.trim().to_string();
                if query.is_empty() {
                    self.search.query = None;
                } else {
                    self.search.query = Some(query);
                }
                self.search_mode = false;
                self.search_buffer.clear();
                self.cursor = 0;
                self.scroll_offset = 0;
                self.selected_step = None;
            }
            KeyCode::Esc => {
                self.search_mode = false;
                self.search_buffer.clear();
            }
            KeyCode::Backspace => {
                self.search_buffer.pop();
            }
            KeyCode::Char(c) => {
                self.search_buffer.push(c);
            }
            _ => {}
        }
    }

    /// Handle key input while in normal mode.
    fn handle_input(&mut self, key: event::KeyEvent) {
        match key.code {
            // Quit
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.should_quit = true;
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }

            // Navigation
            KeyCode::Up => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
            }
            KeyCode::Down => {
                let filtered = self.filtered_steps();
                if self.cursor + 1 < filtered.len() {
                    self.cursor += 1;
                }
            }
            KeyCode::PageUp => {
                self.cursor = self.cursor.saturating_sub(PAGE_SIZE);
            }
            KeyCode::PageDown => {
                let filtered = self.filtered_steps();
                self.cursor = (self.cursor + PAGE_SIZE).min(
                    filtered.len().saturating_sub(1),
                );
            }
            KeyCode::Home => {
                self.cursor = 0;
                self.scroll_offset = 0;
            }
            KeyCode::End => {
                let filtered = self.filtered_steps();
                self.cursor = filtered.len().saturating_sub(1);
            }

            // Collapse / expand
            KeyCode::Left => {
                self.collapse_at_cursor();
            }
            KeyCode::Right => {
                self.expand_at_cursor();
            }

            // Toggle detail view
            KeyCode::Enter => {
                let filtered = self.filtered_steps();
                if self.selected_step == Some(self.cursor) {
                    self.selected_step = None;
                } else if self.cursor < filtered.len() {
                    self.selected_step = Some(self.cursor);
                }
            }

            // Cycle filter mode
            KeyCode::Char('f') | KeyCode::Char('F') => {
                self.filter_mode = self.filter_mode.cycle();
                self.cursor = 0;
                self.scroll_offset = 0;
                self.selected_step = None;
            }

            // Enter search mode
            KeyCode::Char('/') => {
                self.search_mode = true;
                self.search_buffer = self
                    .search
                    .query
                    .clone()
                    .unwrap_or_default();
            }

            // Export
            KeyCode::Char('e') | KeyCode::Char('E') => {
                self.export_requested = true;
            }

            // Reload
            KeyCode::Char('r') | KeyCode::Char('R') => {
                self.reload_requested = true;
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default();
                self.last_reload = Some(Self::format_timestamp(now));
            }

            // Toggle help overlay
            KeyCode::Char('?') | KeyCode::Char('h') => {
                self.show_help = !self.show_help;
            }

            _ => {}
        }
    }

    /// Collapse the step at the cursor index in the original workflow tree.
    fn collapse_at_cursor(&mut self) {
        let filtered = self.filtered_steps();
        if self.cursor >= filtered.len() {
            return;
        }
        let target_id = &filtered[self.cursor].id;
        Self::set_collapse_recursive(&mut self.workflow.steps, target_id, true);
    }

    /// Expand the step at the cursor index in the original workflow tree.
    fn expand_at_cursor(&mut self) {
        let filtered = self.filtered_steps();
        if self.cursor >= filtered.len() {
            return;
        }
        let target_id = &filtered[self.cursor].id;
        Self::set_collapse_recursive(&mut self.workflow.steps, target_id, false);
    }

    /// Recursively find a step by id and set its collapsed state.
    fn set_collapse_recursive(
        steps: &mut [crate::model::Step],
        id: &str,
        collapsed: bool,
    ) -> bool {
        for step in steps.iter_mut() {
            if step.id == *id {
                step.set_collapse(collapsed);
                return true;
            }
            if Self::set_collapse_recursive(&mut step.children, id, collapsed) {
                return true;
            }
        }
        false
    }

    /// Draw the entire TUI frame.
    fn draw(&mut self, frame: &mut Frame<'_>) {
        let size = frame.area();
        let viewport_height = size.height.saturating_sub(RESERVED_ROWS) as usize;
        if viewport_height == 0 {
            return;
        }

        let filtered = self.filtered_steps();
        self.clamp_cursor(filtered.len());
        self.adjust_scroll_offset(viewport_height);

        // Main layout: [title] [content] [status]
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),  // title
                Constraint::Min(2),     // content area
                Constraint::Length(1),  // status bar
            ])
            .split(size);

        let title_area = main_chunks[0];
        let content_area = main_chunks[1];
        let status_area = main_chunks[2];

        // Title bar
        let step_pos = if filtered.is_empty() {
            "Step 0/0".to_string()
        } else {
            format!("Step {}/{}", self.cursor + 1, filtered.len())
        };

        let reload_info = self
            .last_reload
            .as_ref()
            .map(|t| format!(" | Reloaded: {t}"))
            .unwrap_or_default();

        let filter_label = self.filter_mode.label();
        let search_label = if self.search.query.is_some() {
            " | Search: active"
        } else {
            ""
        };

        let title_text = format!(
            " {} ({}) | {}{}{} | Filter: {}",
            self.workflow.name,
            self.workflow.framework,
            step_pos,
            reload_info,
            search_label,
            filter_label
        );

        let title = Paragraph::new(title_text).style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );
        frame.render_widget(title, title_area);

        // Content area
        if let Some(selected) = self.selected_step {
            if selected < filtered.len() {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Percentage(60),
                        Constraint::Percentage(40),
                    ])
                    .split(content_area);

                self.draw_steps_list(frame, chunks[0], &filtered, viewport_height);
                self.draw_detail_panel(frame, chunks[1], &filtered[selected]);
            }
        } else {
            self.draw_steps_list(frame, content_area, &filtered, viewport_height);
        }

        // Status bar
        self.draw_status_bar(frame, status_area);

        // Help overlay
        if self.show_help {
            self.draw_help_overlay(frame);
        }

        // Search prompt overlay
        if self.search_mode {
            self.draw_search_prompt(frame);
        }
    }

    /// Draw the main steps list.
    fn draw_steps_list(
        &self,
        frame: &mut Frame<'_>,
        area: Rect,
        steps: &[FlatStep],
        viewport_height: usize,
    ) {
        let mut lines: Vec<Line> = Vec::new();

        let end = (self.scroll_offset + viewport_height).min(steps.len());

        for i in self.scroll_offset..end {
            let step = &steps[i];
            let is_cursor = i == self.cursor;
            let is_failed = step.status.is_error();

            let indent = "  ".repeat(step.depth);
            let marker = step.status.marker();

            let collapse = if step.has_children {
                if step.collapsed {
                    "[+]"
                } else {
                    "[-]"
                }
            } else {
                "   "
            };

            // Truncate name to fit within the area width
            let max_name_width = (area.width as usize)
                .saturating_sub(20 + indent.len() + collapse.len() + 3 + marker.len() + 3 + 1);
            let name_display = if step.name.len() > max_name_width {
                format!(
                    "{}…",
                    &step.name[..max_name_width.saturating_sub(1).max(1)]
                )
            } else {
                step.name.clone()
            };

            let fail_prefix = if is_failed { "*" } else { " " };
            let line_text = format!(
                "{}{}[{}] {}{} {}",
                fail_prefix, indent, marker, collapse, name_display, step.id
            );

            // Style: cursor gets reverse video, failed steps get red
            let style = if is_cursor {
                Style::default()
                    .bg(Color::Gray)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD)
            } else if is_failed {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(match step.status {
                    Status::Ok => Color::Green,
                    Status::Running => Color::Yellow,
                    Status::Waiting => Color::White,
                    Status::Skipped => Color::DarkGray,
                    Status::Error => Color::Red,
                })
            };

            lines.push(Line::from(Span::styled(line_text, style)));
        }

        let paragraph = Paragraph::new(Text::from(lines)).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Workflow Steps "),
        );

        frame.render_widget(paragraph, area);
    }

    /// Draw the detail panel for the selected step.
    fn draw_detail_panel(
        &self,
        frame: &mut Frame<'_>,
        area: Rect,
        step: &FlatStep,
    ) {
        let mut lines: Vec<Line> = Vec::new();

        // Step name and type
        lines.push(Line::from(Span::styled(
            format!("  Name: {}", step.name),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )));

        lines.push(Line::from(Span::styled(
            format!("  Type: {}", step.step_type),
            Style::default().fg(Color::Cyan),
        )));

        // Source location
        lines.push(Line::from(Span::styled(
            format!("  Source: {}", step.source_location),
            Style::default().fg(Color::Blue),
        )));

        // Status
        let status_style = match step.status {
            Status::Ok => Style::default().fg(Color::Green),
            Status::Error => Style::default().fg(Color::Red),
            Status::Running => Style::default().fg(Color::Yellow),
            Status::Waiting => Style::default().fg(Color::White),
            Status::Skipped => Style::default().fg(Color::DarkGray),
        };
        lines.push(Line::from(vec![
            Span::styled("  Status: ", Style::default().fg(Color::White)),
            Span::styled(step.status.marker(), status_style),
        ]));

        // Config snippet (up to 6 lines)
        if !step.config_snippet.is_empty() {
            lines.push(Line::from(Span::styled(
                "  Config:",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            )));
            for snippet_line in step.config_snippet.lines().take(6) {
                let max_width = (area.width as usize).saturating_sub(6);
                let truncated = if snippet_line.len() > max_width {
                    format!(
                        "{}…",
                        &snippet_line[..max_width.saturating_sub(1).max(1)]
                    )
                } else {
                    snippet_line.to_string()
                };
                lines.push(Line::from(Span::styled(
                    format!("    │ {truncated}"),
                    Style::default().fg(Color::DarkGray),
                )));
            }
        }

        // Error detail + suggestion
        if let Some(ref err) = step.error {
            lines.push(Line::from(Span::styled(
                "  Error:",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )));
            let err_text =
                self.wrap_text(&err.message, area.width.saturating_sub(6) as usize);
            for err_line in err_text {
                lines.push(Line::from(Span::styled(
                    format!("    {err_line}"),
                    Style::default().fg(Color::Red),
                )));
            }
            if let Some(ref suggestion) = err.suggestion {
                lines.push(Line::from(Span::styled(
                    "  Suggestion:",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )));
                let sug_text =
                    self.wrap_text(suggestion, area.width.saturating_sub(6) as usize);
                for sug_line in sug_text {
                    lines.push(Line::from(Span::styled(
                        format!("    {sug_line}"),
                        Style::default().fg(Color::Yellow),
                    )));
                }
            }
        }

        let paragraph = Paragraph::new(Text::from(lines))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Detail "),
            )
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, area);
    }

    /// Wrap text to fit within a given width.
    fn wrap_text(&self, text: &str, width: usize) -> Vec<String> {
        if width == 0 {
            return vec![text.to_string()];
        }
        let mut result = Vec::new();
        for line in text.lines() {
            if line.len() <= width {
                result.push(line.to_string());
            } else {
                let mut remaining = line;
                while remaining.len() > width {
                    result.push(remaining[..width].to_string());
                    remaining = &remaining[width..];
                }
                if !remaining.is_empty() {
                    result.push(remaining.to_string());
                }
            }
        }
        if result.is_empty() {
            result.push(String::new());
        }
        result
    }

    /// Draw the status bar at the bottom.
    fn draw_status_bar(&self, frame: &mut Frame<'_>, area: Rect) {
        let status_text = if self.search_mode {
            format!(" / {}", self.search_buffer)
        } else {
            " [q] Quit  [f] Filter  [/] Search  [e] Export  [r] Reload  [?] Help "
                .to_string()
        };

        let status = Paragraph::new(status_text).style(
            Style::default()
                .fg(Color::White)
                .bg(Color::Blue),
        );

        frame.render_widget(status, area);
    }

    /// Draw help overlay as a centered box.
    fn draw_help_overlay(&self, frame: &mut Frame<'_>) {
        let size = frame.area();

        // Center a 60x16 area
        let popup_width = 60.min(size.width.saturating_sub(4));
        let popup_height = 16.min(size.height.saturating_sub(4));
        let popup_x = (size.width - popup_width) / 2;
        let popup_y = (size.height - popup_height) / 2;

        let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

        frame.render_widget(Clear, popup_area);

        let mut lines: Vec<Line> = Vec::new();
        lines.push(Line::from(Span::styled(
            " Keybindings Help",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        let bindings: Vec<(&str, &str)> = vec![
            ("Up/Down", "Move cursor"),
            ("Left/Right", "Collapse/Expand step"),
            ("Enter", "Toggle detail view"),
            ("f", "Cycle filter mode"),
            ("/", "Search steps"),
            ("e", "Export current view"),
            ("r", "Mark reload (watch mode)"),
            ("PgUp/PgDn", "Scroll by 10 steps"),
            ("Home/End", "Jump to first/last"),
            ("?", "Toggle this help"),
            ("q", "Quit"),
        ];

        for (key, desc) in bindings {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {:12}", key),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(desc.to_string(), Style::default().fg(Color::White)),
            ]));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  Press any key to close",
            Style::default().fg(Color::DarkGray),
        )));

        let help = Paragraph::new(Text::from(lines))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Help ")
                    .style(Style::default().fg(Color::White).bg(Color::Black)),
            );
        frame.render_widget(help, popup_area);
    }

    /// Draw search prompt at the bottom of the screen.
    fn draw_search_prompt(&self, frame: &mut Frame<'_>) {
        let size = frame.area();

        let prompt_text = format!(" /{}", self.search_buffer);
        let prompt_area = Rect::new(0, size.height.saturating_sub(1), size.width, 1);

        let prompt = Paragraph::new(prompt_text).style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

        frame.render_widget(Clear, prompt_area);
        frame.render_widget(prompt, prompt_area);
    }
}
