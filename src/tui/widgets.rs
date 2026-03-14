//! Reusable TUI widget helpers.

use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::theme;

/// Render a breadcrumb trail like: Library > Game Name > Folder Category
pub fn render_breadcrumb(frame: &mut Frame, area: Rect, segments: &[&str]) {
    let spans: Vec<Span> = segments
        .iter()
        .enumerate()
        .flat_map(|(i, seg)| {
            let mut v = Vec::new();
            if i > 0 {
                v.push(Span::styled(" > ", theme::dim()));
            }
            v.push(Span::styled(seg.to_string(), theme::breadcrumb()));
            v
        })
        .collect();

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line).block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(paragraph, area);
}

/// Render the status bar with contextual hotkey hints.
pub fn render_status_bar(frame: &mut Frame, area: Rect, hints: &[(&str, &str)]) {
    let spans: Vec<Span> = hints
        .iter()
        .enumerate()
        .flat_map(|(i, (key, desc))| {
            let mut v = Vec::new();
            if i > 0 {
                v.push(Span::styled("  ", theme::status_bar()));
            }
            v.push(Span::styled(format!("{key} "), theme::status_hotkey()));
            v.push(Span::styled(desc.to_string(), theme::status_bar()));
            v
        })
        .collect();

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line)
        .style(theme::status_bar());
    frame.render_widget(paragraph, area);
}

/// Render the filter input line.
pub fn render_filter_bar(frame: &mut Frame, area: Rect, filter_text: &str) {
    let line = Line::from(vec![
        Span::styled("/ ", theme::status_hotkey()),
        Span::styled(filter_text.to_string(), theme::filter_input()),
        Span::styled("█", theme::filter_input()),
    ]);
    let paragraph = Paragraph::new(line).style(theme::status_bar());
    frame.render_widget(paragraph, area);
}

/// Format a Unix timestamp into a relative or absolute date string.
pub fn format_last_played(timestamp: u64) -> String {
    if timestamp == 0 {
        return "Never".to_string();
    }

    let Some(dt) = chrono::DateTime::from_timestamp(timestamp as i64, 0) else {
        return "Unknown".to_string();
    };

    let now = chrono::Utc::now();
    let diff = now.signed_duration_since(dt);

    if diff.num_hours() < 1 {
        "Just now".to_string()
    } else if diff.num_hours() < 24 {
        format!("{}h ago", diff.num_hours())
    } else if diff.num_days() < 30 {
        format!("{}d ago", diff.num_days())
    } else if diff.num_days() < 365 {
        format!("{}mo ago", diff.num_days() / 30)
    } else {
        dt.format("%Y-%m-%d").to_string()
    }
}

/// Format bytes into human-readable size.
pub fn format_size(bytes: u64) -> String {
    const GB: u64 = 1_073_741_824;
    const MB: u64 = 1_048_576;
    const KB: u64 = 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.0} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}
