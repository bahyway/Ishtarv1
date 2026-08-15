//! ea_panel_state.rs — EaAgent DubSar panel state.
#![forbid(unsafe_code)]

use crate::ea_model_loader::EaModelStatus;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EaMessageKind {
    User, Agent, System, Math, Error,
}

impl EaMessageKind {
    pub fn prefix(self) -> &'static str {
        match self {
            Self::User   => "You",
            Self::Agent  => "𒂗𒆠 EaAgent",
            Self::System => "⚙ System",
            Self::Math   => "∑ Exact",
            Self::Error  => "⚠ Error",
        }
    }
    pub fn color_rgb(self) -> [u8; 3] {
        match self {
            Self::User   => [100, 180, 255],
            Self::Agent  => [0,   200, 255],  // cyan — math god color
            Self::System => [150, 150, 150],
            Self::Math   => [0,   255, 136],  // green — exact result
            Self::Error  => [255, 80,  80],
        }
    }
}

#[derive(Debug, Clone)]
pub struct EaMessage {
    pub kind:    EaMessageKind,
    pub content: String,
    pub epoch:   u64,
}

impl EaMessage {
    pub fn user(content: &str, epoch: u64) -> Self {
        Self { kind: EaMessageKind::User, content: content.to_string(), epoch }
    }
    pub fn agent(content: &str, epoch: u64) -> Self {
        Self { kind: EaMessageKind::Agent, content: content.to_string(), epoch }
    }
    pub fn system(content: &str) -> Self {
        Self { kind: EaMessageKind::System, content: content.to_string(), epoch: 0 }
    }
    pub fn math(content: &str) -> Self {
        Self { kind: EaMessageKind::Math, content: content.to_string(), epoch: 0 }
    }
    pub fn error(content: &str) -> Self {
        Self { kind: EaMessageKind::Error, content: content.to_string(), epoch: 0 }
    }
}

pub struct EaPanelState {
    pub messages:     Vec<EaMessage>,
    pub input:        String,
    pub model_status: EaModelStatus,
    pub is_thinking:  bool,
    epoch:            u64,
}

impl EaPanelState {
    pub fn new() -> Self {
        let mut s = Self {
            messages: Vec::new(), input: String::new(),
            model_status: EaModelStatus::NotFound,
            is_thinking: false, epoch: 0,
        };
        s.messages.push(EaMessage::system(
            "𒂗𒆠 EaAgent — God of Wisdom & Mathematics\n\
             Sovereign algebraic engine: Pauli · Jordan · Harmony · Exact Solver\n\
             Type !help for commands or ask any algebra question.\n\
             Model: DeepSeek-Math-7B (load ~/models/ to enable full AI)"
        ));
        s
    }

    pub fn push_user(&mut self, content: &str) {
        self.epoch += 1;
        self.messages.push(EaMessage::user(content, self.epoch));
    }
    pub fn push_agent(&mut self, content: &str) {
        self.epoch += 1;
        self.messages.push(EaMessage::agent(content, self.epoch));
    }
    pub fn push_math(&mut self, content: &str) {
        self.messages.push(EaMessage::math(content));
    }
    pub fn push_system(&mut self, content: &str) {
        self.messages.push(EaMessage::system(content));
    }
    pub fn push_error(&mut self, content: &str) {
        self.messages.push(EaMessage::error(content));
    }
    pub fn clear(&mut self) {
        self.messages.clear();
        self.messages.push(EaMessage::system("Conversation cleared. 𒂗𒆠"));
    }
    pub fn take_input(&mut self) -> String {
        let s = self.input.clone();
        self.input.clear();
        s
    }
}

impl Default for EaPanelState { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn new_state_has_welcome() {
        let s = EaPanelState::new();
        assert_eq!(s.messages.len(), 1);
        assert!(s.messages[0].content.contains("EaAgent"));
    }
    #[test]
    fn agent_color_is_cyan() {
        assert_eq!(EaMessageKind::Agent.color_rgb(), [0, 200, 255]);
    }
    #[test]
    fn math_color_is_green() {
        assert_eq!(EaMessageKind::Math.color_rgb(), [0, 255, 136]);
    }
    #[test]
    fn take_input_clears() {
        let mut s = EaPanelState::new();
        s.input = "test".into();
        let t = s.take_input();
        assert_eq!(t, "test");
        assert!(s.input.is_empty());
    }
    #[test]
    fn clear_resets_messages() {
        let mut s = EaPanelState::new();
        s.push_user("q");
        s.push_agent("a");
        s.clear();
        assert_eq!(s.messages.len(), 1);
    }
}
