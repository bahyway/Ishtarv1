//! conversation.rs — KAKI-stamped conversation particles.
#![forbid(unsafe_code)]

use enkidb_kaki::{Kaki, KakiMinter, KakiRole};


/// Tribe ID for conversation particles.
pub const CONV_TRIBE_ID: u16 = 0x1200;

/// Who produced this turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnRole {
    User      = 0x01,  // maps to KISHIB
    Assistant = 0x02,  // maps to ZIKRU
    System    = 0x03,  // maps to PARZU
}

impl TurnRole {
    pub fn to_kaki_role(self) -> KakiRole {
        match self {
            Self::User      => KakiRole::Kishib,
            Self::Assistant => KakiRole::Zikru,
            Self::System    => KakiRole::Parzu,
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Self::User      => "user",
            Self::Assistant => "assistant",
            Self::System    => "system",
        }
    }
}

/// A single conversation turn — sovereign particle with full provenance.
#[derive(Debug, Clone)]
pub struct Turn {
    /// Sovereign identity of this turn.
    pub kaki:       Kaki,
    /// Who produced this turn.
    pub role:       TurnRole,
    /// The text content.
    pub content:    String,
    /// Epoch when this turn was created.
    pub epoch:      u64,
    /// Number of tokens in this turn.
    pub token_count: usize,
    /// Session this turn belongs to.
    pub session_id: u64,
    /// Turn index within session (1-based).
    pub turn_index: u32,
}

impl Turn {
    /// Mint a new conversation turn with sovereign KAKI identity.
    pub fn new(
        role:       TurnRole,
        content:    String,
        epoch:      u64,
        session_id: u64,
        turn_index: u32,
        minter:     &KakiMinter,
    ) -> Self {
        // uuid_hash from content + session + index
        let hash_input = format!("{session_id}:{turn_index}:{content}");
        let uuid_hash  = fnv1a_32(hash_input.as_bytes());
        let kaki       = minter.mint_identity(uuid_hash, role.to_kaki_role());

        // Approximate token count (4 chars per token heuristic)
        let token_count = (content.len() / 4).max(1);

        Self { kaki, role, content, epoch, token_count, session_id, turn_index }
    }

    /// Format for display in the DubSar IDE panel.
    pub fn display(&self) -> String {
        let role = self.role.as_str();
        let kaki = self.kaki.bytes()[..4].iter()
            .map(|b| format!("{b:02x}")).collect::<String>();
        format!("[{role}|κ:{kaki}|ep:{:5}|tok:{:4}] {}",
            self.epoch, self.token_count,
            if self.content.len() > 80 { &self.content[..80] } else { &self.content })
    }
}

/// A conversation particle in the EAV store.
#[derive(Debug, Clone)]
pub struct ConversationParticle {
    pub turn:        Turn,
    /// EAV attributes: (attr_hash, value)
    pub attributes:  Vec<(u32, String)>,
}

impl ConversationParticle {
    pub fn from_turn(turn: Turn) -> Self {
        let mut attributes = Vec::new();
        // Standard EAV attributes for every conversation turn
        attributes.push((attr_hash("role"),        turn.role.as_str().to_string()));
        attributes.push((attr_hash("session_id"),  turn.session_id.to_string()));
        attributes.push((attr_hash("turn_index"),  turn.turn_index.to_string()));
        attributes.push((attr_hash("epoch"),       turn.epoch.to_string()));
        attributes.push((attr_hash("token_count"), turn.token_count.to_string()));
        attributes.push((attr_hash("content"),     turn.content.clone()));
        Self { turn, attributes }
    }

    /// Get attribute value by name.
    pub fn get_attr(&self, name: &str) -> Option<&str> {
        let h = attr_hash(name);
        self.attributes.iter()
            .find(|(k, _)| *k == h)
            .map(|(_, v)| v.as_str())
    }
}

fn fnv1a_32(data: &[u8]) -> u32 {
    const OFFSET: u32 = 2166136261;
    const PRIME: u32  = 16777619;
    let mut h = OFFSET;
    for &b in data { h ^= b as u32; h = h.wrapping_mul(PRIME); }
    h
}

fn attr_hash(name: &str) -> u32 { fnv1a_32(name.as_bytes()) }

#[cfg(test)]
mod tests {
    use super::*;
    use bahyway_core::TribeId;

    fn make_minter() -> KakiMinter {
        KakiMinter::new(TribeId::from_u16(CONV_TRIBE_ID))
    }

    #[test]
    fn turn_role_kaki_role_mapping() {
        assert_eq!(TurnRole::User.to_kaki_role(), KakiRole::Kishib);
        assert_eq!(TurnRole::Assistant.to_kaki_role(), KakiRole::Zikru);
        assert_eq!(TurnRole::System.to_kaki_role(), KakiRole::Parzu);
    }

    #[test]
    fn turn_minted_with_valid_checksum() {
        let minter = make_minter();
        let turn = Turn::new(
            TurnRole::User,
            "What does enkidb-kaki do?".to_string(),
            1000, 42, 1, &minter,
        );
        assert!(turn.kaki.verify_checksum(), "KAKI checksum invalid");
    }

    #[test]
    fn two_turns_have_distinct_kakis() {
        let minter = make_minter();
        let t1 = Turn::new(TurnRole::User, "hello".to_string(), 1000, 1, 1, &minter);
        let t2 = Turn::new(TurnRole::User, "world".to_string(), 1001, 1, 2, &minter);
        assert_ne!(
            t1.kaki.bytes(),
            t2.kaki.bytes(),
            "distinct turns must have distinct KAKIs"
        );
    }

    #[test]
    fn conversation_particle_has_content_attr() {
        let minter = make_minter();
        let turn = Turn::new(TurnRole::Assistant, "I am EnkiduLLM".to_string(), 1000, 1, 1, &minter);
        let particle = ConversationParticle::from_turn(turn);
        assert_eq!(particle.get_attr("content"), Some("I am EnkiduLLM"));
        assert_eq!(particle.get_attr("role"), Some("assistant"));
    }

    #[test]
    fn token_count_heuristic() {
        let minter = make_minter();
        let content = "a".repeat(400); // ~100 tokens
        let turn = Turn::new(TurnRole::User, content, 1000, 1, 1, &minter);
        assert!(turn.token_count >= 90 && turn.token_count <= 110,
            "token_count = {}", turn.token_count);
    }

    #[test]
    fn display_truncates_long_content() {
        let minter = make_minter();
        let content = "x".repeat(200);
        let turn = Turn::new(TurnRole::User, content, 1000, 1, 1, &minter);
        let disp = turn.display();
        assert!(disp.len() < 200, "display should truncate");
    }
}
