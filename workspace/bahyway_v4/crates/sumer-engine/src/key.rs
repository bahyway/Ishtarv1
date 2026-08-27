//! Vec7DKey — the sovereign 256-bit key derivation function.
//!
//! Input:  three SovereignSigns (p1, p2, p3) from either tradition.
//! Stage 1 — order-sensitive seeds via FNV-1a (tradition-tagged).
//! Stage 2 — anchor mixing on kinetic-engine's EXACT Plimpton 322
//!           spreads, quantized on the Babylonian base 240
//!           (mirrors Plimpton 322; authoritative ColourID law
//!           lives in GeoEngine and is not touched here).
//! Stage 3 — sponge stretching, default 65,536 rounds.
//! Output: 256-bit key, deterministic: same signs, same key.
//!
//! HONESTY SEAL: this is a sovereign internal KDF for BahyWay
//! credentials. Externally-facing cryptography routes through
//! AkkadiCipherEngine per BC-SEC-001.

use crate::sign::{fnv1a64, SignTradition, SovereignSign};
use kinetic_engine::PlimptonAnchors;

/// Default sponge stretching rounds — sealed in v3.5.1, kept in v4.0.
pub const DEFAULT_STRETCH_ROUNDS: u32 = 65_536;

/// Babylonian quantizer (Plimpton 322 base). Hash-domain constant.
const KEY_QUANTIZER: f64 = 240.0;

/// Which traditions contributed to a key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraditionProfile {
    PureMesZL,
    PureWahshiyya,
    Mixed,
}

/// A 256-bit sovereign key.
#[derive(Debug, Clone, PartialEq)]
pub struct Vec7DKey {
    pub bytes: [u8; 32],
    pub profile: TraditionProfile,
    /// Which Heptagon dimensions the three signs activated.
    pub dimensions: [bool; 7],
}

impl Vec7DKey {
    /// Generate with the sovereign default of 65,536 rounds.
    pub fn generate<S1, S2, S3>(p1: &S1, p2: &S2, p3: &S3) -> Vec7DKey
    where
        S1: SovereignSign,
        S2: SovereignSign,
        S3: SovereignSign,
    {
        Vec7DKey::generate_with_rounds(p1, p2, p3, DEFAULT_STRETCH_ROUNDS)
    }

    /// Generate with explicit stretching rounds (tests use fewer).
    pub fn generate_with_rounds<S1, S2, S3>(p1: &S1, p2: &S2, p3: &S3, rounds: u32) -> Vec7DKey
    where
        S1: SovereignSign,
        S2: SovereignSign,
        S3: SovereignSign,
    {
        // Stage 1 — order-sensitive sign seeds.
        let s1 = p1.seed(1);
        let s2 = p2.seed(2);
        let s3 = p3.seed(3);

        // Stage 2 — mix with the exact Plimpton anchor spreads.
        let anchors = PlimptonAnchors::sovereign().as_array();
        let mut state: [u64; 4] = [s1, s2, s3, s1 ^ s2.rotate_left(21) ^ s3.rotate_left(42)];
        for (i, spread) in anchors.iter().enumerate() {
            let q = (spread * KEY_QUANTIZER).round() as u64;
            state[i % 4] = state[i % 4]
                .rotate_left(7)
                .wrapping_add(q.wrapping_mul(0x9E37_79B9_7F4A_7C15));
        }

        // Stage 3 — sponge stretching.
        for r in 0..rounds {
            let mut buf = [0u8; 36];
            for i in 0..4 {
                buf[i * 8..i * 8 + 8].copy_from_slice(&state[i].to_be_bytes());
            }
            buf[32..36].copy_from_slice(&r.to_be_bytes());
            let mixed = fnv1a64(&buf, state[(r as usize) % 4]);
            let idx = (r as usize + 1) % 4;
            state[idx] = state[idx].rotate_left(17) ^ mixed;
        }

        // Output — 32 bytes.
        let mut bytes = [0u8; 32];
        for i in 0..4 {
            bytes[i * 8..i * 8 + 8].copy_from_slice(&state[i].to_be_bytes());
        }

        // Profiles.
        let traditions = [p1.tradition(), p2.tradition(), p3.tradition()];
        let all_meszl = traditions.iter().all(|t| *t == SignTradition::MesZL);
        let all_wah = traditions.iter().all(|t| *t == SignTradition::IbnWahshiyya);
        let profile = if all_meszl {
            TraditionProfile::PureMesZL
        } else if all_wah {
            TraditionProfile::PureWahshiyya
        } else {
            TraditionProfile::Mixed
        };

        let mut dimensions = [false; 7];
        if let Some(d) = p1.dimension() {
            dimensions[d.index()] = true;
        }
        if let Some(d) = p2.dimension() {
            dimensions[d.index()] = true;
        }
        if let Some(d) = p3.dimension() {
            dimensions[d.index()] = true;
        }

        Vec7DKey {
            bytes,
            profile,
            dimensions,
        }
    }

    /// Regenerate and compare — same signs must reproduce the key.
    pub fn verify<S1, S2, S3>(&self, p1: &S1, p2: &S2, p3: &S3) -> bool
    where
        S1: SovereignSign,
        S2: SovereignSign,
        S3: SovereignSign,
    {
        Vec7DKey::generate(p1, p2, p3).bytes == self.bytes
    }

    pub fn verify_with_rounds<S1, S2, S3>(&self, p1: &S1, p2: &S2, p3: &S3, rounds: u32) -> bool
    where
        S1: SovereignSign,
        S2: SovereignSign,
        S3: SovereignSign,
    {
        Vec7DKey::generate_with_rounds(p1, p2, p3, rounds).bytes == self.bytes
    }

    pub fn to_hex(&self) -> String {
        let mut s = String::with_capacity(64);
        for b in self.bytes.iter() {
            s.push_str(&format!("{:02x}", b));
        }
        s
    }
}

/// Local helper: PlimptonAnchors as a fixed-order [f64; 7] array,
/// ME..URU — kinetic-engine's struct exposes named fields only.
trait AsArray {
    fn as_array(&self) -> [f64; 7];
}

impl AsArray for PlimptonAnchors {
    fn as_array(&self) -> [f64; 7] {
        [
            self.me, self.gu, self.sag, self.a, self.izi, self.ud, self.uru,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::meszl;
    use crate::wahshiyya;

    const R: u32 = 128; // fast test rounds

    #[test]
    fn same_signs_same_key_deterministic() {
        let a = meszl::by_name("DUB").unwrap();
        let b = meszl::by_name("SAR").unwrap();
        let c = meszl::by_name("AN").unwrap();
        let k1 = Vec7DKey::generate_with_rounds(a, b, c, R);
        let k2 = Vec7DKey::generate_with_rounds(a, b, c, R);
        assert_eq!(k1.bytes, k2.bytes);
        assert_eq!(k1.profile, TraditionProfile::PureMesZL);
    }

    #[test]
    fn order_changes_the_key() {
        let a = meszl::by_name("DUB").unwrap();
        let b = meszl::by_name("SAR").unwrap();
        let c = meszl::by_name("AN").unwrap();
        let k1 = Vec7DKey::generate_with_rounds(a, b, c, R);
        let k2 = Vec7DKey::generate_with_rounds(c, b, a, R);
        assert_ne!(k1.bytes, k2.bytes);
    }

    #[test]
    fn rounds_change_the_key() {
        let a = meszl::by_name("KI").unwrap();
        let b = meszl::by_name("EN").unwrap();
        let c = meszl::by_name("NUN").unwrap();
        let k1 = Vec7DKey::generate_with_rounds(a, b, c, 64);
        let k2 = Vec7DKey::generate_with_rounds(a, b, c, 65);
        assert_ne!(k1.bytes, k2.bytes);
    }

    #[test]
    fn pure_wahshiyya_profile_and_dimension_tracking() {
        let anchors = wahshiyya::planetary_anchor_signs();
        let k = Vec7DKey::generate_with_rounds(&anchors[0], &anchors[1], &anchors[5], R);
        assert_eq!(k.profile, TraditionProfile::PureWahshiyya);
        // Saturn>ME (index 0), Jupiter>GU (index 1), Mercury>UD (index 5)
        assert!(k.dimensions[0] && k.dimensions[1] && k.dimensions[5]);
        assert!(!k.dimensions[2] && !k.dimensions[6]);
    }

    #[test]
    fn verify_roundtrip_and_hex_shape() {
        let a = meszl::by_name("ME").unwrap();
        let s = wahshiyya::planetary_anchor_signs();
        let k = Vec7DKey::generate_with_rounds(a, &s[3], &s[6], R);
        assert_eq!(k.profile, TraditionProfile::Mixed);
        assert!(k.verify_with_rounds(a, &s[3], &s[6], R));
        assert!(!k.verify_with_rounds(a, &s[6], &s[3], R));
        let hex = k.to_hex();
        assert_eq!(hex.len(), 64);
        assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn default_rounds_full_stretch_is_deterministic() {
        let a = meszl::by_name("DUB").unwrap();
        let b = meszl::by_name("SAR").unwrap();
        let c = meszl::by_name("LUGAL").unwrap();
        let k1 = Vec7DKey::generate(a, b, c);
        let k2 = Vec7DKey::generate(a, b, c);
        assert_eq!(k1.bytes, k2.bytes);
        assert!(k1.verify(a, b, c));
    }
}
