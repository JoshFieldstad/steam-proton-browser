//! Reusable TUI widget helpers.

use ratatui::{
    Frame,
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
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
    let paragraph = Paragraph::new(line).style(theme::status_bar());
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
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    if timestamp == 0 {
        return "Never".to_string();
    }

    let played = UNIX_EPOCH + Duration::from_secs(timestamp);
    let Ok(elapsed) = SystemTime::now().duration_since(played) else {
        return "Unknown".to_string();
    };

    let secs = elapsed.as_secs();
    let hours = secs / 3600;
    let days = secs / 86400;

    if hours < 1 {
        "Just now".to_string()
    } else if hours < 24 {
        format!("{hours}h ago")
    } else if days < 30 {
        format!("{days}d ago")
    } else if days < 365 {
        format!("{}mo ago", days / 30)
    } else {
        // Civil date conversion inlined to avoid a `chrono` or `time` dependency for a single
        // call site. If date formatting is needed elsewhere, replace with the `time` crate.
        // Algorithm: Howard Hinnant's civil_from_days
        // https://howardhinnant.github.io/date_algorithms.html#civil_from_days
        let ts = timestamp as i64;
        let days_since_epoch = ts / 86400;
        let z = days_since_epoch + 719468;
        let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
        let doe = (z - era * 146097) as u64;
        let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
        let y = (yoe as i64) + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = doy - (153 * mp + 2) / 5 + 1;
        let m = if mp < 10 { mp + 3 } else { mp - 9 };
        let y = if m <= 2 { y + 1 } else { y };
        format!("{y:04}-{m:02}-{d:02}")
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: compute the date string from a known epoch timestamp.
    /// This isolates the civil-date algorithm (the >365 days branch)
    /// so the test doesn't depend on "now".
    fn date_from_epoch(ts: u64) -> String {
        let days_since_epoch = ts as i64 / 86400;
        let z = days_since_epoch + 719468;
        let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
        let doe = (z - era * 146097) as u64;
        let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
        let y = (yoe as i64) + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = doy - (153 * mp + 2) / 5 + 1;
        let m = if mp < 10 { mp + 3 } else { mp - 9 };
        let y = if m <= 2 { y + 1 } else { y };
        format!("{y:04}-{m:02}-{d:02}")
    }

    #[test]
    fn test_format_last_played_never() {
        assert_eq!(format_last_played(0), "Never");
    }

    #[test]
    fn test_epoch_to_date_known_dates() {
        // 2020-01-01 00:00:00 UTC = 1577836800
        assert_eq!(date_from_epoch(1577836800), "2020-01-01");
        // 2000-02-29 00:00:00 UTC = 951782400 (leap day)
        assert_eq!(date_from_epoch(951782400), "2000-02-29");
        // 1970-01-01 = epoch
        assert_eq!(date_from_epoch(0), "1970-01-01");
        // 2024-12-31 00:00:00 UTC = 1735603200
        assert_eq!(date_from_epoch(1735603200), "2024-12-31");
        // 2026-03-14 00:00:00 UTC = 1773446400
        assert_eq!(date_from_epoch(1773446400), "2026-03-14");
    }

    #[test]
    fn test_epoch_to_date_leap_years() {
        // 2024 is a leap year: Feb 29
        assert_eq!(date_from_epoch(1709164800), "2024-02-29");
        // 2023 is not: Mar 01
        assert_eq!(date_from_epoch(1677628800), "2023-03-01");
        // 2100 is NOT a leap year (divisible by 100 but not 400)
        // 2100-03-01 = need to verify the algorithm handles century years
        // 2100-02-28 23:59:59 should be Feb 28, next day should be Mar 01
        // 2100-03-01 00:00:00 UTC = 4107542400
        assert_eq!(date_from_epoch(4107542400), "2100-03-01");
    }

    #[test]
    fn test_format_last_played_future_returns_unknown() {
        // A timestamp far in the future should return "Unknown"
        assert_eq!(format_last_played(u64::MAX / 2), "Unknown");
    }
}
