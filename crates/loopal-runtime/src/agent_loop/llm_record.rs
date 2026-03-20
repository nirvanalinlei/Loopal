use loopal_message::{ContentBlock, Message, MessageRole};
use tracing::error;

use super::runner::AgentLoopRunner;

impl AgentLoopRunner {
    /// Record the assistant response as a message in the conversation history.
    /// Writes to both persistent storage and in-memory params.messages.
    pub fn record_assistant_message(
        &mut self,
        assistant_text: &str,
        tool_uses: &[(String, String, serde_json::Value)],
    ) {
        let mut assistant_content: Vec<ContentBlock> = Vec::new();
        if !assistant_text.is_empty() {
            assistant_content.push(ContentBlock::Text {
                text: assistant_text.to_string(),
            });
        }
        for (id, name, input) in tool_uses {
            assistant_content.push(ContentBlock::ToolUse {
                id: id.clone(),
                name: name.clone(),
                input: input.clone(),
            });
        }

        if !assistant_content.is_empty() {
            let mut assistant_msg = Message {
                id: None,
                role: MessageRole::Assistant,
                content: assistant_content,
            };
            if let Err(e) = self.params.session_manager.save_message(
                &self.params.session.id,
                &mut assistant_msg,
            ) {
                error!(error = %e, "failed to persist message");
            }
            self.params.messages.push(assistant_msg);
        }
    }
}
