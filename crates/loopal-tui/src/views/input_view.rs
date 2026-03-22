/// Input view: single-line `> ` prompt with CJK cursor fix.
///
/// No border, no title — just a command input channel.
/// Shows inbox count when messages are queued: `> (2 queued) `.
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;
use unicode_width::UnicodeWidthChar;

/// Render the input area as a single-line `> ` prompt.
pub fn render_input(
    f: &mut Frame,
    input: &str,
    cursor: usize,
    inbox_count: usize,
    area: Rect,
) {
    if area.height == 0 {
        return;
    }

    let prefix = if inbox_count > 0 {
        format!("> ({} queued) ", inbox_count)
    } else {
        "> ".to_string()
    };
    let prefix_width: usize = prefix.chars().map(|c| c.width().unwrap_or(0)).sum();

    let line = Line::from(vec![
        Span::styled(prefix, Style::default().fg(Color::DarkGray)),
        Span::raw(input.to_string()),
    ]);

    f.render_widget(Paragraph::new(line), area);

    // Cursor position: prefix display width + input display width up to cursor
    let input_width = display_width_up_to(input, cursor);
    f.set_cursor_position((
        area.x + (prefix_width + input_width) as u16,
        area.y,
    ));
}

/// Calculate the display width of a string up to byte position `pos`.
/// Uses UAX #11 via unicode-width for accurate CJK/emoji/fullwidth handling.
fn display_width_up_to(s: &str, byte_pos: usize) -> usize {
    let slice = &s[..byte_pos.min(s.len())];
    slice.chars().map(|c| c.width().unwrap_or(0)).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii_width() {
        assert_eq!(display_width_up_to("hello", 5), 5);
    }

    #[test]
    fn test_cjk_width() {
        // Each CJK character is 3 bytes, display width 2
        let s = "你好世界";
        assert_eq!(display_width_up_to(s, 6), 4); // 2 chars = width 4
        assert_eq!(display_width_up_to(s, 12), 8); // 4 chars = width 8
    }

    #[test]
    fn test_mixed_width() {
        let s = "hi你好";
        assert_eq!(display_width_up_to(s, 2), 2); // "hi" = width 2
        assert_eq!(display_width_up_to(s, 5), 4); // "hi你" = width 4
        assert_eq!(display_width_up_to(s, 8), 6); // "hi你好" = width 6
    }

    #[test]
    fn test_empty() {
        assert_eq!(display_width_up_to("", 0), 0);
    }

    #[test]
    fn test_pos_beyond_length() {
        assert_eq!(display_width_up_to("abc", 100), 3);
    }
}
