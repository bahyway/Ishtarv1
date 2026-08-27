//! panel_state.rs — TamuzAI DubSar panel state management.
//!
//! Manages the egui panel state for the 𒀭 TamuzAI tab in DubSar IDE.
#![forbid(unsafe_code)]

use crate::model_loader::ModelStatus;

/// Kind of message in the chat display.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageKind {
    User,
    Assistant,
    System,
    Command,
    Error,
}

impl MessageKind {
    pub fn prefix(&self) -> &str {
        match self {
            Self::User => "You",
            Self::Assistant => "𒀭 TamuzAI",
            Self::System => "⚙ System",
            Self::Command => "⚡ Quick",
            Self::Error => "⚠ Error",
        }
    }
    pub fn color_rgb(&self) -> [u8; 3] {
        match self {
            Self::User => [100, 180, 255],    // blue
            Self::Assistant => [255, 215, 0], // gold (𒁾 sovereign color)
            Self::System => [150, 150, 150],  // gray
            Self::Command => [0, 255, 136],   // green (GEM color)
            Self::Error => [255, 80, 80],     // red
        }
    }
}

/// A single message in the panel display.
#[derive(Debug, Clone)]
pub struct PanelMessage {
    pub kind: MessageKind,
    pub content: String,
    pub epoch: u64,
}

impl PanelMessage {
    pub fn user(content: &str, epoch: u64) -> Self {
        Self {
            kind: MessageKind::User,
            content: content.to_string(),
            epoch,
        }
    }
    pub fn assistant(content: &str, epoch: u64) -> Self {
        Self {
            kind: MessageKind::Assistant,
            content: content.to_string(),
            epoch,
        }
    }
    pub fn system(content: &str) -> Self {
        Self {
            kind: MessageKind::System,
            content: content.to_string(),
            epoch: 0,
        }
    }
    pub fn command(content: &str) -> Self {
        Self {
            kind: MessageKind::Command,
            content: content.to_string(),
            epoch: 0,
        }
    }
    pub fn error(content: &str) -> Self {
        Self {
            kind: MessageKind::Error,
            content: content.to_string(),
            epoch: 0,
        }
    }
}

/// Complete state for the 𒀭 TamuzAI DubSar panel.
#[derive(Debug)]
pub struct TamuzPanelState {
    /// Chat message history for display.
    pub messages: Vec<PanelMessage>,
    /// Current input field text.
    pub input: String,
    /// Model loading status.
    pub model_status: ModelStatus,
    /// Whether inference is currently running.
    pub is_thinking: bool,
    /// Current session ID.
    pub session_id: u64,
    /// Epoch counter.
    epoch: u64,
    /// Whether to show the download instructions.
    pub show_download: bool,
}

impl TamuzPanelState {
    pub fn new() -> Self {
        let mut state = Self {
            messages: Vec::new(),
            input: String::new(),
            model_status: ModelStatus::NotFound,
            is_thinking: false,
            session_id: 1,
            epoch: 0,
            show_download: false,
        };
        // Welcome message
        state.messages.push(PanelMessage::system(
            "𒀭 TamuzAI — Sovereign Offline MetaEngine\n\
             Type !help for quick commands or ask anything about BahyWay.Ecosystem v4.0.\n\
             Load DeepSeek-Coder-6.7B-Q4_K_M.gguf to enable full AI responses.",
        ));
        state
    }

    /// Add a user message.
    pub fn push_user(&mut self, content: &str) {
        self.epoch += 1;
        self.messages.push(PanelMessage::user(content, self.epoch));
    }

    /// Add an assistant response.
    pub fn push_assistant(&mut self, content: &str) {
        self.epoch += 1;
        self.messages
            .push(PanelMessage::assistant(content, self.epoch));
    }

    /// Add a system notification.
    pub fn push_system(&mut self, content: &str) {
        self.messages.push(PanelMessage::system(content));
    }

    /// Add a quick command response.
    pub fn push_command(&mut self, content: &str) {
        self.messages.push(PanelMessage::command(content));
    }

    /// Add an error message.
    pub fn push_error(&mut self, content: &str) {
        self.messages.push(PanelMessage::error(content));
    }

    /// Clear all messages except the welcome.
    pub fn clear(&mut self) {
        self.messages.clear();
        self.messages
            .push(PanelMessage::system("Conversation cleared. 𒁾"));
    }

    /// Take the current input and clear the field.
    pub fn take_input(&mut self) -> String {
        let input = self.input.clone();
        self.input.clear();
        input
    }

    /// Recent messages for context window (last N).
    pub fn recent_context(&self, n: usize) -> Vec<(&str, &str)> {
        self.messages
            .iter()
            .rev()
            .take(n)
            .filter(|m| m.kind == MessageKind::User || m.kind == MessageKind::Assistant)
            .map(|m| (m.kind.prefix(), m.content.as_str()))
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    pub fn message_count(&self) -> usize {
        self.messages.len()
    }
}

impl Default for TamuzPanelState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_state_has_welcome_message() {
        let state = TamuzPanelState::new();
        assert_eq!(state.messages.len(), 1);
        assert_eq!(state.messages[0].kind, MessageKind::System);
        assert!(state.messages[0].content.contains("TamuzAI"));
    }

    #[test]
    fn push_user_increments_epoch() {
        let mut state = TamuzPanelState::new();
        state.push_user("hello");
        assert_eq!(state.epoch, 1);
        state.push_user("world");
        assert_eq!(state.epoch, 2);
    }

    #[test]
    fn clear_removes_messages_adds_notification() {
        let mut state = TamuzPanelState::new();
        state.push_user("test");
        state.push_assistant("response");
        state.clear();
        assert_eq!(state.messages.len(), 1);
        assert_eq!(state.messages[0].kind, MessageKind::System);
    }

    #[test]
    fn take_input_clears_field() {
        let mut state = TamuzPanelState::new();
        state.input = "test input".to_string();
        let taken = state.take_input();
        assert_eq!(taken, "test input");
        assert!(state.input.is_empty());
    }

    #[test]
    fn message_kind_colors_are_distinct() {
        let user = MessageKind::User.color_rgb();
        let asst = MessageKind::Assistant.color_rgb();
        let err = MessageKind::Error.color_rgb();
        assert_ne!(user, asst);
        assert_ne!(user, err);
    }

    #[test]
    fn assistant_color_is_gold() {
        let [r, g, b] = MessageKind::Assistant.color_rgb();
        assert_eq!([r, g, b], [255, 215, 0]); // sovereign gold
    }
}
