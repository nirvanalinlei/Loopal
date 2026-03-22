/// Message lines conversion: DisplayMessage → Vec<Line<'static>>.
///
/// Instruction model: user messages are dim commands (`> content`),
/// assistant output flows directly without labels, tool calls are
/// single-line work traces, thinking is a collapsed indicator.
use ratatui::prelude::*;

use crate::markdown;
use loopal_session::types::DisplayMessage;

use super::tool_summary::tool_call_summary;

/// Convert a single DisplayMessage into pre-wrapped styled Lines.
pub fn message_to_lines(msg: &DisplayMessage, width: u16) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    match msg.role.as_str() {
        "user" => render_user(&mut lines, msg, width),
        "assistant" => render_assistant(&mut lines, msg, width),
        "thinking" => render_thinking(&mut lines, msg),
        "error" => render_prefixed(&mut lines, msg, "Error: ", Color::Red, width),
        "system" => render_prefixed(&mut lines, msg, "System: ", Color::Yellow, width),
        _ => render_prefixed(&mut lines, msg, &format!("{}: ", msg.role), Color::White, width),
    }

    // Tool calls — single-line summaries
    for tc in &msg.tool_calls {
        let (summary, color) = tool_call_summary(tc);
        let style = match color {
            "green" => Style::default().fg(Color::Green),
            "red" => Style::default().fg(Color::Red),
            _ => Style::default().fg(Color::Yellow),
        };
        let w = (width as usize).max(1);
        lines.extend(
            textwrap::wrap(&summary, w)
                .into_iter()
                .map(|cow| Line::from(Span::styled(cow.into_owned(), style))),
        );
    }

    lines.push(Line::from(""));
    lines
}

/// User message: `> content` (dim, acts as instruction record).
fn render_user(lines: &mut Vec<Line<'static>>, msg: &DisplayMessage, width: u16) {
    let dim = Style::default()
        .fg(Color::DarkGray)
        .add_modifier(Modifier::DIM);
    if msg.content.is_empty() {
        lines.push(Line::from(Span::styled("> ", dim)));
        return;
    }
    for (i, line) in msg.content.lines().enumerate() {
        let prefix = if i == 0 { "> " } else { "  " };
        let text = format!("{}{}", prefix, line);
        let w = (width as usize).max(1);
        lines.extend(
            textwrap::wrap(&text, w)
                .into_iter()
                .map(|cow| Line::from(Span::styled(cow.into_owned(), dim))),
        );
    }
}

/// Assistant message: direct output, no label. Markdown rendered.
fn render_assistant(lines: &mut Vec<Line<'static>>, msg: &DisplayMessage, width: u16) {
    if !msg.content.is_empty() {
        lines.extend(markdown::render_markdown(&msg.content, width));
    }
}

/// Thinking: collapsed to single-line indicator with token estimate.
fn render_thinking(lines: &mut Vec<Line<'static>>, msg: &DisplayMessage) {
    let token_est = msg.content.len() / 4;
    let label = if token_est >= 1000 {
        format!("Thinking ({}k tokens)", token_est / 1000)
    } else if token_est > 0 {
        format!("Thinking ({} tokens)", token_est)
    } else {
        "Thinking...".to_string()
    };
    lines.push(Line::from(Span::styled(
        label,
        Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::DIM),
    )));
}

/// Generic prefixed message (error, system, unknown roles).
fn render_prefixed(
    lines: &mut Vec<Line<'static>>,
    msg: &DisplayMessage,
    prefix: &str,
    color: Color,
    width: u16,
) {
    let style = Style::default().fg(color).bold();
    lines.push(Line::from(Span::styled(prefix.to_string(), style)));
    if !msg.content.is_empty() {
        for line in msg.content.lines() {
            lines.extend(wrap_line(line, width));
        }
    }
}

/// Convert streaming text into pre-wrapped styled Lines.
///
/// Streaming text is **incomplete** markdown — plain textwrap only.
/// No label prefix (instruction model: agent output IS the content).
pub fn streaming_to_lines(text: &str, width: u16) -> Vec<Line<'static>> {
    if text.is_empty() {
        return Vec::new();
    }
    let mut lines = Vec::new();
    for line in text.lines() {
        lines.extend(wrap_line(line, width));
    }
    lines
}

/// Wrap a single logical line into visual lines using textwrap.
fn wrap_line(line: &str, width: u16) -> Vec<Line<'static>> {
    let w = (width as usize).max(1);
    textwrap::wrap(line, w)
        .into_iter()
        .map(|cow| Line::from(cow.into_owned()))
        .collect()
}
