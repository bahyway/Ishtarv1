//! SovereignSign — the unified interface over both traditions:
//! MesZL cuneiform (Sumerian) and Ibn Wahshiyya planetary scripts
//! (Shawq al-Mustaham, Baghdad, ~930 CE).
//!
//! Sign identity in v4.0 is (tradition, reference) — never a
//! hand-packed KAKI. KAKI minting belongs to enkidb-ingest.

use kinetic_engine::HeptaDimension;

const FNV_OFFSET: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// Sovereign FNV-1a 64-bit — the pure-std hash used across SumerEngine.
pub fn fnv1a64(bytes: &[u8], seed: u64) -> u64 {
    let mut h = if seed == 0 { FNV_OFFSET } else { seed };
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(FNV_PRIME);
    }
    h
}

/// The two sovereign sign traditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignTradition {
    MesZL,
    IbnWahshiyya,
}

impl SignTradition {
    /// Tradition tag byte — sealed in v3.5.1 and carried into v4.0.
    /// NOTE: this is a HASH-DOMAIN tag, not a KAKI byte.
    pub fn tag(&self) -> u8 {
        match self {
            SignTradition::MesZL => 0x10,
            SignTradition::IbnWahshiyya => 0x11,
        }
    }
}

/// The unified interface every sovereign sign implements.
pub trait SovereignSign {
    fn tradition(&self) -> SignTradition;
    /// Reference number within its tradition (MesZL 1-898 or
    /// Wahshiyya planetary index).
    fn reference(&self) -> u32;
    fn phonetic(&self) -> &str;
    fn dimension(&self) -> Option<HeptaDimension>;

    /// Deterministic sovereign seed for key derivation.
    /// `position` makes the seed order-sensitive (p1 != p2 != p3).
    fn seed(&self, position: u8) -> u64 {
        let mut bytes: Vec<u8> = Vec::with_capacity(16 + self.phonetic().len());
        bytes.push(self.tradition().tag());
        bytes.extend_from_slice(&self.reference().to_be_bytes());
        bytes.extend_from_slice(self.phonetic().as_bytes());
        bytes.push(position);
        fnv1a64(&bytes, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fnv_is_deterministic_and_input_sensitive() {
        let a = fnv1a64(b"DUB.SAR", 0);
        let b = fnv1a64(b"DUB.SAR", 0);
        let c = fnv1a64(b"DUB.SAT", 0);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn tradition_tags_are_distinct() {
        assert_eq!(SignTradition::MesZL.tag(), 0x10);
        assert_eq!(SignTradition::IbnWahshiyya.tag(), 0x11);
        assert_ne!(SignTradition::MesZL.tag(), SignTradition::IbnWahshiyya.tag());
    }
}
