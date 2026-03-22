use loopal_message::{ContentBlock, Message, MessageRole};

/// Remove oldest messages, keeping the system message and the last `keep_last` messages.
pub fn compact_messages(messages: &mut Vec<Message>, keep_last: usize) {
    if messages.len() <= keep_last + 1 {
        return;
    }

    // Separate system messages (always at the front) from the rest
    let system_count = messages
        .iter()
        .take_while(|m| m.role == MessageRole::System)
        .count();

    let non_system_len = messages.len() - system_count;
    if non_system_len <= keep_last {
        return;
    }

    // Keep system messages + last `keep_last` non-system messages
    let remove_count = non_system_len - keep_last;
    messages.drain(system_count..system_count + remove_count);
}

/// Find the largest ToolResult content block across all messages.
/// Returns `(message_index, block_index, byte_size)`.
pub fn find_largest_tool_result(messages: &[Message]) -> Option<(usize, usize, usize)> {
    let mut best: Option<(usize, usize, usize)> = None;
    for (mi, msg) in messages.iter().enumerate() {
        for (bi, block) in msg.content.iter().enumerate() {
            if let ContentBlock::ToolResult { content, .. } = block {
                let size = content.len();
                if best.is_none_or(|(_, _, s)| size > s) {
                    best = Some((mi, bi, size));
                }
            }
        }
    }
    best
}

/// Truncate a ToolResult content block in place, keeping at most `max_lines` lines
/// and `max_bytes` bytes, appending a truncation notice.
pub fn truncate_block_content(block: &mut ContentBlock, max_lines: usize, max_bytes: usize) {
    let ContentBlock::ToolResult { content, .. } = block else {
        return;
    };
    if content.len() <= max_bytes && content.lines().count() <= max_lines {
        return;
    }

    let original_bytes = content.len();
    let original_lines = content.lines().count();
    let mut result = String::new();
    let mut byte_count = 0;

    for (i, line) in content.lines().enumerate() {
        let line_bytes = line.len() + 1;
        if i >= max_lines || byte_count + line_bytes > max_bytes {
            break;
        }
        if i > 0 {
            result.push('\n');
        }
        result.push_str(line);
        byte_count += line_bytes;
    }

    result.push_str(&format!(
        "\n\n[Truncated by context guard: kept {byte_count}/{original_bytes} bytes, \
         {}/{original_lines} lines]",
        result.lines().count()
    ));
    *content = result;
}

/// Strip `ContentBlock::Thinking` blocks from all assistant messages except the last one.
/// The last assistant message's thinking block must be preserved for Anthropic signature
/// verification in multi-turn conversations. Older thinking blocks waste context tokens.
pub fn strip_old_thinking(messages: &mut [Message]) {
    // Find the index of the last assistant message
    let last_assistant_idx = messages
        .iter()
        .rposition(|m| m.role == MessageRole::Assistant);

    for (i, msg) in messages.iter_mut().enumerate() {
        if msg.role != MessageRole::Assistant {
            continue;
        }
        if Some(i) == last_assistant_idx {
            continue; // preserve thinking in the last assistant message
        }
        msg.content
            .retain(|block| !matches!(block, ContentBlock::Thinking { .. }));
    }
}
