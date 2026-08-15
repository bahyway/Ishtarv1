//! Akkadian Linguistic Root System — Fix 2 applied.
//! Nonce raised from 16 → 32 bytes. Birthday bound: 2^128.

use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};
use crate::{KupruError, KupruResult};

/// Domain separator — prevents cross-protocol commitment reuse.
const COMMITMENT_DOMAIN: &[u8] = b"BahyWay.LinguisticProof.v353";

#[derive(Debug, Clone, Copy, Default, Zeroize, Serialize, Deserialize)]
pub enum VowelPattern { #[default] Aa, Ai, Ia, Uu, Ui, Au }

impl VowelPattern {
    pub fn entropy_weight(self) -> u8 {
        match self {
            Self::Aa => 0x2A, Self::Ai => 0x3C,
            Self::Ia => 0x4E, Self::Uu => 0x5F,
            Self::Ui => 0x71, Self::Au => 0x83,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct AkkadianAffixes {
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub infix:  Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct AkkadianRoot {
    pub radicals: [char; 3],
    pub pattern:  VowelPattern,
    pub affixes:  AkkadianAffixes,
}

impl AkkadianRoot {
    pub fn new(r1: char, r2: char, r3: char, pattern: VowelPattern) -> Self {
        Self { radicals: [r1, r2, r3], pattern, affixes: AkkadianAffixes::default() }
    }

    pub fn sargon() -> Self { Self::new('Š', 'R', 'K', VowelPattern::Ui) }

    pub fn from_phrase(phrase: &str) -> KupruResult<Self> {
        let normalized = phrase.trim().to_lowercase();
        if normalized.is_empty() {
            return Err(KupruError::InvalidInput("LEMNĪ: empty phrase".into()));
        }
        let consonants: Vec<char> = normalized.chars()
            .filter(|c| !matches!(c, 'a'|'e'|'i'|'o'|'u'|' '|'-')).collect();
        if consonants.len() < 3 {
            return Err(KupruError::InvalidInput("LEMNĪ: < 3 consonants".into()));
        }
        let vowels: Vec<char> = normalized.chars()
            .filter(|c| matches!(c, 'a'|'e'|'i'|'o'|'u')).collect();
        let pattern = match (vowels.first(), vowels.get(1)) {
            (Some('a'), Some('a')) => VowelPattern::Aa,
            (Some('a'), Some('i')) => VowelPattern::Ai,
            (Some('i'), Some('a')) => VowelPattern::Ia,
            (Some('u'), Some('u')) => VowelPattern::Uu,
            (Some('u'), Some('i')) => VowelPattern::Ui,
            (Some('a'), Some('u')) => VowelPattern::Au,
            _ => VowelPattern::Aa,
        };
        Ok(Self {
            radicals: [consonants[0], consonants[1], consonants[2]],
            pattern,
            affixes: AkkadianAffixes::default(),
        })
    }

    pub fn to_key_material(&self) -> [u8; 32] {
        use sha3::{Digest, Sha3_256};
        let mut h = Sha3_256::new();
        for &r in &self.radicals {
            let mut buf = [0u8; 4];
            h.update(r.encode_utf8(&mut buf).as_bytes());
        }
        h.update([self.pattern.entropy_weight()]);
        if let Some(p) = &self.affixes.prefix { h.update(p.as_bytes()); }
        if let Some(s) = &self.affixes.suffix { h.update(s.as_bytes()); }
        h.finalize().into()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AkkadianName {
    pub sumu:    String,
    pub abu:     Option<String>,
    pub asaredu: Option<String>,
    pub ilu:     Option<String>,
}

impl AkkadianName {
    pub fn dubsar() -> Self {
        Self {
            sumu:    "DUB.SAR".into(),
            abu:     None,
            asaredu: Some("Šar Kibrāt Arbaim".into()),
            ilu:     Some("Nabû".into()),
        }
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }
}

/// Fix 2 — nonce raised from 16 → 32 bytes (birthday bound: 2^128).
#[derive(Debug, Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct LinguisticProof {
    pub root:       AkkadianRoot,
    pub commitment: [u8; 32],
    pub nonce:      [u8; 32],
    pub mudu:       u8,
}

impl LinguisticProof {
    pub fn create(phrase: &str) -> KupruResult<Self> {
        use rand::RngCore;
        use sha3::{Digest, Sha3_256};

        let root = AkkadianRoot::from_phrase(phrase)?;
        let mut nonce = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut nonce);

        let mut h = Sha3_256::new();
        h.update(COMMITMENT_DOMAIN);
        h.update(phrase.as_bytes());
        h.update(&nonce);
        let commitment: [u8; 32] = h.finalize().into();

        Ok(Self { root, commitment, nonce, mudu: Self::score_morphology(phrase) })
    }

    pub fn verify(&self, phrase: &str) -> bool {
        use sha3::{Digest, Sha3_256};
        use subtle::ConstantTimeEq;
        let mut h = Sha3_256::new();
        h.update(COMMITMENT_DOMAIN);
        h.update(phrase.as_bytes());
        h.update(&self.nonce);
        let expected: [u8; 32] = h.finalize().into();
        self.commitment.ct_eq(&expected).into()
    }

    fn score_morphology(phrase: &str) -> u8 {
        let mut score = 0u8;
        let p = phrase.to_lowercase();
        if p.contains('-')                                       { score += 1; }
        if p.chars().count() > 8                                 { score += 1; }
        if p.chars().any(|c| matches!(c, 'š'|'ṣ'|'ṭ'|'ḫ'|'ā'|'ī'|'ū')) { score += 2; }
        if p.split_whitespace().count() > 1                      { score += 1; }
        if p.contains("um") || p.contains("im") || p.contains("am") { score += 1; }
        if p.len() > 16                                          { score += 1; }
        score.min(7)
    }
}
