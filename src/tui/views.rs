//! View rendering — Library, Game Detail, and Folder Browser views.

use ratatui::{
    layout::{Constraint, Rect},
    text::{Line, Span},
    widgets::{Cell, List, ListItem, ListState, Row, Table, TableState},
    Frame,
};

use crate::steam::{folders::FolderEntry, library::GameInfo};

use super::theme;
use super::widgets;

// ── Library View ──────────────────────────────────────────

pub fn render_game_table(
    frame: &mut Frame,
    area: Rect,
    games: &[&GameInfo],
    state: &mut TableState,
) {
    let width = area.width;

    // Width-sensitive column layout
    let (constraints, show_lib_path) = if width >= 120 {
        // Wide: Name | ID | Size | Library | Last Played
        (
            vec![
                Constraint::Min(20),
                Constraint::Length(10),
                Constraint::Length(10),
                Constraint::Length(30),
                Constraint::Length(12),
            ],
            true,
        )
    } else if width >= 80 {
        // Medium: Name | ID | Size | Last Played
        (
            vec![
                Constraint::Min(20),
                Constraint::Length(10),
                Constraint::Length(10),
                Constraint::Length(12),
            ],
            false,
        )
    } else {
        // Narrow: Name | Size | Last Played
        (
            vec![
                Constraint::Min(15),
                Constraint::Length(10),
                Constraint::Length(12),
            ],
            false,
        )
    };

    let header_cells = if width >= 120 {
        vec!["Name", "ID", "Size", "Library", "Last Played"]
    } else if width >= 80 {
        vec!["Name", "ID", "Size", "Last Played"]
    } else {
        vec!["Name", "Size", "Last Played"]
    };

    let header = Row::new(
        header_cells
            .into_iter()
            .map(|h| Cell::from(Span::styled(h, theme::dim()))),
    );

    let rows: Vec<Row> = games
        .iter()
        .map(|g| {
            let size = widgets::format_size(g.size_on_disk);
            let last_played = widgets::format_last_played(g.last_played);
            let lib_path = g
                .library_path
                .to_string_lossy()
                .to_string();

            let mut cells = Vec::new();

            if width >= 80 {
                cells.push(Cell::from(Span::styled(&g.name, theme::title())));
                cells.push(Cell::from(Span::styled(
                    g.app_id.to_string(),
                    theme::dim(),
                )));
                cells.push(Cell::from(Span::styled(size, theme::dim())));
                if show_lib_path {
                    cells.push(Cell::from(Span::styled(lib_path, theme::dim())));
                }
                cells.push(Cell::from(Span::styled(last_played, theme::dim())));
            } else {
                cells.push(Cell::from(Span::styled(&g.name, theme::title())));
                cells.push(Cell::from(Span::styled(size, theme::dim())));
                cells.push(Cell::from(Span::styled(last_played, theme::dim())));
            }

            Row::new(cells)
        })
        .collect();

    let table = Table::new(rows, &constraints)
        .header(header)
        .row_highlight_style(theme::selected());

    frame.render_stateful_widget(table, area, state);
}

// ── Game Detail View ──────────────────────────────────────

pub fn render_folder_list(
    frame: &mut Frame,
    area: Rect,
    folders: &[FolderEntry],
    state: &mut ListState,
) {
    let items: Vec<ListItem> = folders
        .iter()
        .map(|f| {
            let line = Line::from(vec![
                Span::styled("📁 ", theme::folder()),
                Span::styled(&f.label, theme::folder()),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).highlight_style(theme::selected());
    frame.render_stateful_widget(list, area, state);
}

// ── Folder Browser View ───────────────────────────────────

pub struct DirEntry {
    pub name: String,
    pub is_dir: bool,
}

pub fn render_dir_listing(
    frame: &mut Frame,
    area: Rect,
    entries: &[DirEntry],
    state: &mut ListState,
) {
    let items: Vec<ListItem> = entries
        .iter()
        .map(|e| {
            let (icon, style) = if e.is_dir {
                ("📂 ", theme::folder())
            } else {
                ("📄 ", theme::file())
            };
            let line = Line::from(vec![
                Span::styled(icon, style),
                Span::styled(&e.name, style),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).highlight_style(theme::selected());
    frame.render_stateful_widget(list, area, state);
}

/// Read a directory and return sorted entries (dirs first, then files).
pub fn read_dir_entries(path: &std::path::Path) -> Vec<DirEntry> {
    let mut dirs = Vec::new();
    let mut files = Vec::new();

    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
            if is_dir {
                dirs.push(DirEntry {
                    name: format!("{name}/"),
                    is_dir: true,
                });
            } else {
                files.push(DirEntry { name, is_dir: false });
            }
        }
    }

    dirs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    dirs.extend(files);
    dirs
}

// ── Help Overlay ──────────────────────────────────────────

pub fn render_help(frame: &mut Frame, area: Rect) {
    let help_lines = vec![
        "Keybindings:",
        "",
        "  j/↓      Move down",
        "  k/↑      Move up",
        "  g/Home   Jump to top",
        "  G/End    Jump to bottom",
        "  Ctrl+d   Page down",
        "  Ctrl+u   Page up",
        "  Enter    Select / dive in / open file",
        "  e        Edit file in $EDITOR",
        "  Esc/BS   Go back",
        "  o        Open in file explorer",
        "  y        Copy path to clipboard",
        "  /        Filter",
        "  s        Cycle sort (library view)",
        "  R        Refresh (rescan Steam)",
        "  ?        Toggle this help",
        "  q        Quit",
    ];

    let items: Vec<ListItem> = help_lines
        .into_iter()
        .map(|l| ListItem::new(Line::from(l)))
        .collect();

    let list = List::new(items);
    frame.render_widget(list, area);
}
