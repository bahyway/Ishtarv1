//! agent_kaki.rs — EaAgent KAKI identity.
#![forbid(unsafe_code)]

use enkidb_kaki::{Kaki, KakiMinter, KakiRole};
use bahyway_core::TribeId;

pub const AGENT_TRIBE_ID: u16 = 0x1300;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentTribe {
    EaAgent   = 0x1300,
    TamuzAI   = 0x1200,
    NuskuAgent = 0x1400,
}

impl AgentTribe {
    pub fn name(self) -> &'static str {
        match self {
            Self::EaAgent    => "𒂗𒆠 EaAgent",
            Self::TamuzAI    => "𒀭 TamuzAI",
            Self::NuskuAgent => "𒀭𒁕𒍪 NuskuAgent",
        }
    }
    pub fn tribe_id(self) -> u16 { self as u16 }
}

/// KAKI-stamped EaAgent identity particle.
pub struct AgentKaki {
    pub nucleus: Kaki,
    pub agent:   AgentTribe,
    pub version: &'static str,
}

impl AgentKaki {
    pub fn new(agent: AgentTribe, version: &'static str) -> Self {
        let minter  = KakiMinter::new(TribeId::from_u16(agent.tribe_id()));
        let content = format!("{}:{}", agent.name(), version);
        let hash    = fnv1a_32(content.as_bytes());
        let nucleus = minter.mint_identity(hash, KakiRole::Kishib);
        Self { nucleus, agent, version }
    }

    pub fn kaki_hex(&self) -> String {
        self.nucleus.bytes().iter().map(|b| format!("{b:02x}")).collect()
    }
}

fn fnv1a_32(data: &[u8]) -> u32 {
    const OFFSET: u32 = 2166136261;
    const PRIME: u32  = 16777619;
    let mut h = OFFSET;
    for &b in data { h ^= b as u32; h = h.wrapping_mul(PRIME); }
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ea_agent_kaki_valid_checksum() {
        let k = AgentKaki::new(AgentTribe::EaAgent, "4.1.0");
        assert!(k.nucleus.verify_checksum());
    }

    #[test]
    fn agent_tribe_names_correct() {
        assert!(AgentTribe::EaAgent.name().contains("EaAgent"));
        assert!(AgentTribe::TamuzAI.name().contains("TamuzAI"));
    }

    #[test]
    fn agent_tribe_ids_distinct() {
        assert_ne!(AgentTribe::EaAgent.tribe_id(), AgentTribe::TamuzAI.tribe_id());
        assert_ne!(AgentTribe::EaAgent.tribe_id(), AgentTribe::NuskuAgent.tribe_id());
    }

    #[test]
    fn kaki_hex_is_32_chars() {
        let k = AgentKaki::new(AgentTribe::EaAgent, "4.1.0");
        assert_eq!(k.kaki_hex().len(), 32);
    }
}
