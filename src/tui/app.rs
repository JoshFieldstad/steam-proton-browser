//! Top-level TUI application loop.

use std::io;
use std::path::PathBuf;

use crossterm::{
    event::{self, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{ListState, TableState},
};

use crate::platform::explorer;
use crate::steam::{
    folders::{self, FolderEntry},
    library::{GameInfo, Library},
};

use super::keybindings::{Action, map_key};
use super::views::{self, DirEntry};
use super::widgets;

/// Which view the user is currently in.
#[derive(Debug, Clone, PartialEq, Eq)]
enum View {
    Library,
    GameDetail { game_index: usize },
    FolderBrowser { game_index: usize, dir: PathBuf },
}

/// Sort mode for the library view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SortMode {
    LastPlayed,
    Name,
    AppId,
    Size,
}

impl SortMode {
    fn next(self) -> Self {
        match self {
            SortMode::LastPlayed => SortMode::Name,
            SortMode::Name => SortMode::AppId,
            SortMode::AppId => SortMode::Size,
            SortMode::Size => SortMode::LastPlayed,
        }
    }

    fn label(self) -> &'static str {
        match self {
            SortMode::LastPlayed => "last played",
            SortMode::Name => "name",
            SortMode::AppId => "app id",
            SortMode::Size => "size",
        }
    }
}

struct AppState {
    library: Library,
    view_stack: Vec<View>,
    list_state: ListState,
    table_state: TableState,
    filter_text: String,
    filter_mode: bool,
    show_help: bool,
    sort_mode: SortMode,

    /// Saved cursor positions for parent views (pushed on drill-in, popped on back).
    saved_selections: Vec<SavedSelection>,

    // Cached derived data for current view
    filtered_game_indices: Vec<usize>,
    folder_entries: Vec<FolderEntry>,
    dir_entries: Vec<DirEntry>,
}

/// Snapshot of cursor position + filter state for a view we drilled away from.
#[derive(Debug, Clone)]
struct SavedSelection {
    selected: Option<usize>,
    filter_text: String,
}

impl AppState {
    fn new(library: Library) -> Self {
        let game_count = library.games.len();
        let filtered_game_indices = (0..game_count).collect();
        let mut state = Self {
            library,
            view_stack: vec![View::Library],
            list_state: ListState::default(),
            table_state: TableState::default(),
            filter_text: String::new(),
            filter_mode: false,
            show_help: false,
            sort_mode: SortMode::LastPlayed,
            saved_selections: Vec::new(),
            filtered_game_indices,
            folder_entries: Vec::new(),
            dir_entries: Vec::new(),
        };
        state.sort_games();
        if game_count > 0 {
            state.list_state.select(Some(0));
        }
        state
    }

    fn current_view(&self) -> &View {
        self.view_stack.last().unwrap()
    }

    fn item_count(&self) -> usize {
        match self.current_view() {
            View::Library => self.filtered_game_indices.len(),
            View::GameDetail { .. } => self.folder_entries.len(),
            View::FolderBrowser { .. } => self.dir_entries.len(),
        }
    }

    /// Get the currently selected index across both table and list views.
    fn selected_index(&self) -> Option<usize> {
        match self.current_view() {
            View::Library => self.table_state.selected(),
            _ => self.list_state.selected(),
        }
    }

    /// Set the selected index on the appropriate state widget.
    fn set_selected(&mut self, index: Option<usize>) {
        match self.current_view() {
            View::Library => self.table_state.select(index),
            _ => self.list_state.select(index),
        }
    }

    fn sort_games(&mut self) {
        match self.sort_mode {
            SortMode::LastPlayed => self
                .library
                .games
                .sort_by(|a, b| b.last_played.cmp(&a.last_played)),
            SortMode::Name => self
                .library
                .games
                .sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase())),
            SortMode::AppId => self.library.games.sort_by_key(|g| g.app_id),
            SortMode::Size => self
                .library
                .games
                .sort_by(|a, b| b.size_on_disk.cmp(&a.size_on_disk)),
        }
        self.apply_filter();
    }

    fn apply_filter(&mut self) {
        let matcher = SkimMatcherV2::default();
        self.filtered_game_indices = self
            .library
            .games
            .iter()
            .enumerate()
            .filter(|(_, g)| {
                // Always exclude runtime/tool entries
                if g.is_runtime() {
                    return false;
                }
                // Apply fuzzy text filter if active
                if !self.filter_text.is_empty() {
                    return matcher.fuzzy_match(&g.name, &self.filter_text).is_some();
                }
                true
            })
            .map(|(i, _)| i)
            .collect();
        // Reset selection
        if self.filtered_game_indices.is_empty() {
            self.table_state.select(None);
        } else {
            self.table_state.select(Some(0));
        }
    }

    fn push_view(&mut self, view: View) {
        // Save current cursor position before navigating away
        let current_sel = self.selected_index();
        self.saved_selections.push(SavedSelection {
            selected: current_sel,
            filter_text: self.filter_text.clone(),
        });

        self.filter_text.clear();
        self.filter_mode = false;
        self.list_state = ListState::default();
        self.table_state = TableState::default();

        match &view {
            View::Library => {
                self.apply_filter();
                if !self.filtered_game_indices.is_empty() {
                    self.table_state.select(Some(0));
                }
            }
            View::GameDetail { game_index } => {
                let game = &self.library.games[*game_index];
                self.folder_entries = folders::resolve_folders(
                    game.app_id,
                    &game.install_dir,
                    &game.library_path,
                    &self.library.steam_roots,
                );
                if !self.folder_entries.is_empty() {
                    self.list_state.select(Some(0));
                }
            }
            View::FolderBrowser { dir, .. } => {
                self.dir_entries = views::read_dir_entries(dir);
                if !self.dir_entries.is_empty() {
                    self.list_state.select(Some(0));
                }
            }
        }

        self.view_stack.push(view);
    }

    fn pop_view(&mut self) {
        if self.view_stack.len() <= 1 {
            return;
        }
        self.view_stack.pop();
        self.filter_mode = false;
        self.list_state = ListState::default();
        self.table_state = TableState::default();

        // Restore saved cursor position
        let saved = self.saved_selections.pop();
        if let Some(ref s) = saved {
            self.filter_text = s.filter_text.clone();
        } else {
            self.filter_text.clear();
        }

        // Refresh data for the view we're returning to
        match self.current_view().clone() {
            View::Library => {
                self.apply_filter();
                // Restore selection after filtering (which resets to 0)
                if let Some(ref s) = saved
                    && let Some(sel) = s.selected
                {
                    let count = self.filtered_game_indices.len();
                    self.table_state
                        .select(Some(sel.min(count.saturating_sub(1))));
                }
            }
            View::GameDetail { game_index } => {
                let game = &self.library.games[game_index];
                self.folder_entries = folders::resolve_folders(
                    game.app_id,
                    &game.install_dir,
                    &game.library_path,
                    &self.library.steam_roots,
                );
                if let Some(ref s) = saved {
                    if let Some(sel) = s.selected {
                        let count = self.folder_entries.len();
                        self.list_state
                            .select(Some(sel.min(count.saturating_sub(1))));
                    }
                } else if !self.folder_entries.is_empty() {
                    self.list_state.select(Some(0));
                }
            }
            View::FolderBrowser { ref dir, .. } => {
                self.dir_entries = views::read_dir_entries(dir);
                if let Some(ref s) = saved {
                    if let Some(sel) = s.selected {
                        let count = self.dir_entries.len();
                        self.list_state
                            .select(Some(sel.min(count.saturating_sub(1))));
                    }
                } else if !self.dir_entries.is_empty() {
                    self.list_state.select(Some(0));
                }
            }
        }
    }

    fn move_selection(&mut self, delta: i32) {
        let count = self.item_count();
        if count == 0 {
            return;
        }
        let current = self.selected_index().unwrap_or(0) as i32;
        let next = (current + delta).clamp(0, count as i32 - 1) as usize;
        self.set_selected(Some(next));
    }

    fn breadcrumb_segments(&self) -> Vec<String> {
        let mut segments = vec!["Library".to_string()];
        for view in &self.view_stack[1..] {
            match view {
                View::Library => {}
                View::GameDetail { game_index } => {
                    segments.push(self.library.games[*game_index].name.clone());
                }
                View::FolderBrowser { dir, .. } => {
                    if let Some(name) = dir.file_name() {
                        segments.push(name.to_string_lossy().to_string());
                    }
                }
            }
        }
        segments
    }

    fn selected_path(&self) -> Option<PathBuf> {
        let sel = self.selected_index()?;
        match self.current_view() {
            View::Library => {
                let gi = *self.filtered_game_indices.get(sel)?;
                let game = &self.library.games[gi];
                let steamapps = if game
                    .library_path
                    .file_name()
                    .is_some_and(|n| n.eq_ignore_ascii_case("steamapps"))
                {
                    game.library_path.clone()
                } else {
                    game.library_path.join("steamapps")
                };
                Some(steamapps.join("common").join(&game.install_dir))
            }
            View::GameDetail { .. } => self.folder_entries.get(sel).map(|f| f.path.clone()),
            View::FolderBrowser { dir, .. } => {
                let entry = self.dir_entries.get(sel)?;
                let name = entry.name.trim_end_matches('/');
                Some(dir.join(name))
            }
        }
    }
}

pub fn run(library: Library) -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = AppState::new(library);

    loop {
        terminal.draw(|frame| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(2), // breadcrumb
                    Constraint::Min(3),    // main content
                    Constraint::Length(1), // status bar
                ])
                .split(frame.area());

            // Breadcrumb
            let segments = state.breadcrumb_segments();
            let seg_refs: Vec<&str> = segments.iter().map(|s| s.as_str()).collect();
            widgets::render_breadcrumb(frame, chunks[0], &seg_refs);

            // Main content
            if state.show_help {
                views::render_help(frame, chunks[1]);
            } else {
                match state.current_view().clone() {
                    View::Library => {
                        let games: Vec<&GameInfo> = state
                            .filtered_game_indices
                            .iter()
                            .filter_map(|&i| state.library.games.get(i))
                            .collect();
                        views::render_game_table(frame, chunks[1], &games, &mut state.table_state);
                    }
                    View::GameDetail { .. } => {
                        views::render_folder_list(
                            frame,
                            chunks[1],
                            &state.folder_entries,
                            &mut state.list_state,
                        );
                    }
                    View::FolderBrowser { .. } => {
                        views::render_dir_listing(
                            frame,
                            chunks[1],
                            &state.dir_entries,
                            &mut state.list_state,
                        );
                    }
                }
            }

            // Status bar / filter
            if state.filter_mode {
                widgets::render_filter_bar(frame, chunks[2], &state.filter_text);
            } else {
                let sort_label = format!("Sort: {}", state.sort_mode.label());
                let hints = match state.current_view() {
                    View::Library => vec![
                        ("/", "Filter"),
                        ("Enter", "Select"),
                        ("o", "Explorer"),
                        ("s", sort_label.as_str()),
                        ("?", "Help"),
                        ("q", "Quit"),
                    ],
                    View::GameDetail { .. } => vec![
                        ("/", "Filter"),
                        ("Enter", "Open"),
                        ("o", "Explorer"),
                        ("y", "Copy"),
                        ("Esc", "Back"),
                        ("?", "Help"),
                    ],
                    View::FolderBrowser { .. } => vec![
                        ("Enter", "Dive/Open"),
                        ("e", "Edit"),
                        ("o", "Explorer"),
                        ("y", "Copy"),
                        ("Esc", "Back"),
                        ("?", "Help"),
                    ],
                };
                let hint_refs: Vec<(&str, &str)> = hints.iter().map(|(a, b)| (*a, *b)).collect();
                widgets::render_status_bar(frame, chunks[2], &hint_refs);
            }
        })?;

        // Event handling
        if let Event::Key(key) = event::read()?
            && let Some(action) = map_key(key, state.filter_mode)
        {
            match action {
                Action::Quit => break,
                Action::MoveDown => state.move_selection(1),
                Action::MoveUp => state.move_selection(-1),
                Action::JumpTop => state.set_selected(Some(0)),
                Action::JumpBottom => {
                    let count = state.item_count();
                    if count > 0 {
                        state.set_selected(Some(count - 1));
                    }
                }
                Action::PageDown => state.move_selection(10),
                Action::PageUp => state.move_selection(-10),
                Action::ToggleHelp => state.show_help = !state.show_help,
                Action::EnterFilter => {
                    state.filter_mode = true;
                    state.filter_text.clear();
                }
                Action::ExitFilter => {
                    state.filter_mode = false;
                    if matches!(state.current_view(), View::Library) {
                        state.apply_filter();
                    }
                }
                Action::FilterChar(c) => {
                    state.filter_text.push(c);
                    if matches!(state.current_view(), View::Library) {
                        state.apply_filter();
                    }
                }
                Action::FilterBackspace => {
                    state.filter_text.pop();
                    if matches!(state.current_view(), View::Library) {
                        state.apply_filter();
                    }
                }
                Action::CycleSort => {
                    if matches!(state.current_view(), View::Library) {
                        state.sort_mode = state.sort_mode.next();
                        state.sort_games();
                    }
                }
                Action::Select => {
                    if let Some(sel) = state.selected_index() {
                        match state.current_view().clone() {
                            View::Library => {
                                if let Some(&gi) = state.filtered_game_indices.get(sel) {
                                    state.push_view(View::GameDetail { game_index: gi });
                                }
                            }
                            View::GameDetail { game_index } => {
                                if let Some(folder) = state.folder_entries.get(sel) {
                                    state.push_view(View::FolderBrowser {
                                        game_index,
                                        dir: folder.path.clone(),
                                    });
                                }
                            }
                            View::FolderBrowser {
                                game_index,
                                ref dir,
                            } => {
                                if let Some(entry) = state.dir_entries.get(sel) {
                                    let name = entry.name.trim_end_matches('/');
                                    let full_path = dir.join(name);
                                    if entry.is_dir {
                                        state.push_view(View::FolderBrowser {
                                            game_index,
                                            dir: full_path,
                                        });
                                    } else {
                                        let _ = explorer::open_file(&full_path);
                                    }
                                }
                            }
                        }
                    }
                }
                Action::Back => state.pop_view(),
                Action::OpenExplorer => {
                    if let Some(path) = state.selected_path() {
                        let target = if path.is_dir() {
                            path
                        } else {
                            path.parent().map(|p| p.to_path_buf()).unwrap_or(path)
                        };
                        let _ = explorer::open_in_file_explorer(&target);
                    }
                }
                Action::EditFile => {
                    if let Some(path) = state.selected_path()
                        && path.is_file()
                    {
                        // Suspend TUI, run editor, then restore
                        disable_raw_mode()?;
                        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
                        terminal.show_cursor()?;

                        let _ = explorer::open_in_editor(&path);

                        // Restore TUI
                        enable_raw_mode()?;
                        execute!(terminal.backend_mut(), EnterAlternateScreen)?;
                        terminal.hide_cursor()?;
                        terminal.clear()?;
                    }
                }
                Action::CopyPath => {
                    // Path copying — best-effort, no clipboard crate for now
                    // The path is shown in the breadcrumb, user can also use 'o'
                }
                Action::Refresh => {
                    // Would trigger a rescan — for now, noop
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
