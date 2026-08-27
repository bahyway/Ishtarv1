//! session.rs — Conversation session management.
#![forbid(unsafe_code)]

use crate::conversation::{ConversationParticle, Turn, TurnRole, CONV_TRIBE_ID};
use bahyway_core::TribeId;
use enkidb_kaki::KakiMinter;

/// A conversation session — a sequence of turns with a shared context.
pub struct Session {
    pub session_id: u64,
    pub title: String,
    pub turns: Vec<ConversationParticle>,
    pub created_epoch: u64,
    minter: KakiMinter,
    turn_counter: u32,
}

impl Session {
    /// Create a new session.
    pub fn new(session_id: u64, title: &str, epoch: u64) -> Self {
        Self {
            session_id,
            title: title.to_string(),
            turns: Vec::new(),
            created_epoch: epoch,
            minter: KakiMinter::new(TribeId::from_u16(CONV_TRIBE_ID)),
            turn_counter: 0,
        }
    }

    /// Add a user turn.
    pub fn add_user(&mut self, content: &str, epoch: u64) -> &ConversationParticle {
        self.turn_counter += 1;
        let turn = Turn::new(
            TurnRole::User,
            content.to_string(),
            epoch,
            self.session_id,
            self.turn_counter,
            &self.minter,
        );
        self.turns.push(ConversationParticle::from_turn(turn));
        self.turns.last().unwrap()
    }

    /// Add an assistant turn.
    pub fn add_assistant(&mut self, content: &str, epoch: u64) -> &ConversationParticle {
        self.turn_counter += 1;
        let turn = Turn::new(
            TurnRole::Assistant,
            content.to_string(),
            epoch,
            self.session_id,
            self.turn_counter,
            &self.minter,
        );
        self.turns.push(ConversationParticle::from_turn(turn));
        self.turns.last().unwrap()
    }

    /// Add a system event turn.
    pub fn add_system(&mut self, content: &str, epoch: u64) {
        self.turn_counter += 1;
        let turn = Turn::new(
            TurnRole::System,
            content.to_string(),
            epoch,
            self.session_id,
            self.turn_counter,
            &self.minter,
        );
        self.turns.push(ConversationParticle::from_turn(turn));
    }

    /// Build a context window for the model (last N turns formatted as prompt).
    pub fn context_window(&self, max_tokens: usize) -> String {
        let mut parts = Vec::new();
        let mut total_tokens = 0usize;

        for particle in self.turns.iter().rev() {
            let tok_count = particle.turn.token_count;
            if total_tokens + tok_count > max_tokens {
                break;
            }
            total_tokens += tok_count;
            parts.push(format!(
                "<|{}|>\n{}\n",
                particle.turn.role.as_str(),
                particle.turn.content
            ));
        }

        parts.reverse();
        parts.join("")
    }

    /// Session statistics.
    pub fn stats(&self) -> SessionStats {
        let total_tokens: usize = self.turns.iter().map(|p| p.turn.token_count).sum();
        let user_turns = self
            .turns
            .iter()
            .filter(|p| p.turn.role == TurnRole::User)
            .count();
        let asst_turns = self
            .turns
            .iter()
            .filter(|p| p.turn.role == TurnRole::Assistant)
            .count();
        SessionStats {
            total_turns: self.turns.len(),
            user_turns,
            asst_turns,
            total_tokens,
        }
    }
}

/// Session statistics summary.
#[derive(Debug, Clone)]
pub struct SessionStats {
    pub total_turns: usize,
    pub user_turns: usize,
    pub asst_turns: usize,
    pub total_tokens: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_add_turns() {
        let mut session = Session::new(1, "Test session", 1000);
        session.add_user("What is KAKI?", 1001);
        session.add_assistant("KAKI is the 16-byte sovereign identity.", 1002);
        assert_eq!(session.turns.len(), 2);
        assert_eq!(session.turn_counter, 2);
    }

    #[test]
    fn session_stats_correct() {
        let mut session = Session::new(1, "Stats test", 1000);
        session.add_user("Q1", 1001);
        session.add_assistant("A1", 1002);
        session.add_user("Q2", 1003);
        let stats = session.stats();
        assert_eq!(stats.user_turns, 2);
        assert_eq!(stats.asst_turns, 1);
        assert_eq!(stats.total_turns, 3);
    }

    #[test]
    fn context_window_most_recent_first() {
        let mut session = Session::new(1, "Context test", 1000);
        session.add_user("first question", 1001);
        session.add_assistant("first answer", 1002);
        let ctx = session.context_window(10000);
        assert!(ctx.contains("first question"));
        assert!(ctx.contains("first answer"));
    }

    #[test]
    fn turns_have_distinct_kakis() {
        let mut session = Session::new(42, "KAKI test", 1000);
        session.add_user("hello", 1001);
        session.add_user("world", 1002);
        let k0 = session.turns[0].turn.kaki.bytes().to_vec();
        let k1 = session.turns[1].turn.kaki.bytes().to_vec();
        assert_ne!(k0, k1);
    }
}
