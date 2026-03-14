//! Keybinding definitions.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Actions the user can trigger.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Quit,
    MoveDown,
    MoveUp,
    JumpTop,
    JumpBottom,
    PageDown,
    PageUp,
    Select,
    Back,
    OpenExplorer,
    EditFile,
    CopyPath,
    ToggleHelp,
    EnterFilter,
    ExitFilter,
    Refresh,
    CycleSort,
    FilterChar(char),
    FilterBackspace,
}

/// Map a key event to an action, given whether we're in filter mode.
pub fn map_key(key: KeyEvent, in_filter_mode: bool) -> Option<Action> {
    if in_filter_mode {
        return map_filter_key(key);
    }

    match (key.modifiers, key.code) {
        (KeyModifiers::CONTROL, KeyCode::Char('c')) => Some(Action::Quit),
        (_, KeyCode::Char('q')) => Some(Action::Quit),
        (_, KeyCode::Char('j') | KeyCode::Down) => Some(Action::MoveDown),
        (_, KeyCode::Char('k') | KeyCode::Up) => Some(Action::MoveUp),
        (_, KeyCode::Char('g') | KeyCode::Home) => Some(Action::JumpTop),
        (_, KeyCode::Char('G') | KeyCode::End) => Some(Action::JumpBottom),
        (KeyModifiers::CONTROL, KeyCode::Char('d')) => Some(Action::PageDown),
        (KeyModifiers::CONTROL, KeyCode::Char('u')) => Some(Action::PageUp),
        (_, KeyCode::Enter) => Some(Action::Select),
        (_, KeyCode::Esc | KeyCode::Backspace) => Some(Action::Back),
        (_, KeyCode::Char('o')) => Some(Action::OpenExplorer),
        (_, KeyCode::Char('e')) => Some(Action::EditFile),
        (_, KeyCode::Char('y')) => Some(Action::CopyPath),
        (_, KeyCode::Char('?')) => Some(Action::ToggleHelp),
        (_, KeyCode::Char('/')) => Some(Action::EnterFilter),
        (_, KeyCode::Char('R')) => Some(Action::Refresh),
        (_, KeyCode::Char('s')) => Some(Action::CycleSort),
        _ => None,
    }
}

fn map_filter_key(key: KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Esc => Some(Action::ExitFilter),
        KeyCode::Enter => Some(Action::ExitFilter),
        KeyCode::Backspace => Some(Action::FilterBackspace),
        KeyCode::Char(c) => Some(Action::FilterChar(c)),
        _ => None,
    }
}
