//! KakiType (κ[6]) and KakiRole (κ[7]) — the taxonomy encoded in the KAKI bytes.

use bahyway_core::BahywayError;

/// Physical KAKI type — encoded at κ[6].  Decided at birth, never changed.
///
/// The 4th type `Pattern` (0x04) is new in v4.0 — it marks KAKIs that are
/// derived deterministically from GA-cluster centroids by NISABA.  Their bytes
/// are computed via `derive_pattern_kaki()` in `crates/enkidb-kaki::pattern`,
/// not minted randomly like identity/event/crosstribe KAKIs.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum KakiType {
    /// 0x01 — Birth certificate of a sovereign particle.
    Identity = 0x01,
    /// 0x02 — Immutable record of a state-transition or operational event.
    Event = 0x02,
    /// 0x03 — Persistent linkage between anchor particles across tribes.
    CrossTribe = 0x03,
    /// 0x04 — Emergent pattern structure derived from GA clustering (NISABA).
    /// Deterministic: same inputs → same KAKI. Never minted randomly.
    Pattern = 0x04,
}

impl KakiType {
    pub fn from_byte(b: u8) -> Result<Self, BahywayError> {
        match b {
            0x01 => Ok(KakiType::Identity),
            0x02 => Ok(KakiType::Event),
            0x03 => Ok(KakiType::CrossTribe),
            0x04 => Ok(KakiType::Pattern),
            other => Err(BahywayError::InvalidKakiType(other)),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            KakiType::Identity => "Identity",
            KakiType::Event => "Event",
            KakiType::CrossTribe => "CrossTribe",
            KakiType::Pattern => "Pattern",
        }
    }

    /// Returns true only for pattern-kakis — used by NISABA to guard the
    /// deterministic derivation path against misuse.
    #[inline]
    pub fn is_pattern(self) -> bool {
        matches!(self, KakiType::Pattern)
    }
}

impl core::fmt::Display for KakiType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Logical KAKI role — encoded at κ[7].  Decided at birth, never changed.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum KakiRole {
    /// 0x01 — External file / blob / source artifact seal (Akkadian: kishib).
    Kishib = 0x01,
    /// 0x02 — Record or entity in a tribe domain (Akkadian: zikru).
    Zikru = 0x02,
    /// 0x03 — Logic, template, axiom, or rule (Akkadian: parṣu).
    Parzu = 0x03,
}

impl KakiRole {
    pub fn from_byte(b: u8) -> Result<Self, BahywayError> {
        match b {
            0x01 => Ok(KakiRole::Kishib),
            0x02 => Ok(KakiRole::Zikru),
            0x03 => Ok(KakiRole::Parzu),
            other => Err(BahywayError::InvalidKakiRole(other)),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            KakiRole::Kishib => "KISHIB",
            KakiRole::Zikru => "ZIKRU",
            KakiRole::Parzu => "PARZU",
        }
    }
}

impl core::fmt::Display for KakiRole {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}
