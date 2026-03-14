//! Color scheme and styling constants.

use ratatui::style::{Color, Modifier, Style};

pub fn selected() -> Style {
    Style::default()
        .bg(Color::Blue)
        .fg(Color::White)
        .add_modifier(Modifier::BOLD)
}

pub fn breadcrumb() -> Style {
    Style::default().fg(Color::Cyan)
}

pub fn folder() -> Style {
    Style::default().fg(Color::Yellow)
}

pub fn file() -> Style {
    Style::default().fg(Color::White)
}

pub fn status_bar() -> Style {
    Style::default().bg(Color::DarkGray).fg(Color::White)
}

pub fn status_hotkey() -> Style {
    Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD)
}

pub fn filter_input() -> Style {
    Style::default().fg(Color::Green)
}

pub fn dim() -> Style {
    Style::default().fg(Color::Gray)
}

pub fn title() -> Style {
    Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD)
}
