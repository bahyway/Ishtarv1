// ── KAKI 16-Byte Sovereign Primary Key ───────────────────────────
// Bytes 0-3:   D1 uuid_hash  (Identity)
// Bytes 4-5:   D2 tribe_id   (Belonging)
// Bytes 6-7:   D3 RED        (Domain/Anomaly)
// Bytes 8-9:   D4 GREEN      (Quality/Confidence)
// Bytes 10-11: D5 BLUE       (Freshness/Recency)
// Bytes 12-13: D6 timestamp  (Temporal epoch)
// Bytes 14-15: D7 checksum   (Integrity)
pub type KakiPK = [u8; 16];

/// Void particle — identity element of the Particle Monoid (PA-5)
pub const KAKI_VOID: KakiPK = [0u8; 16];

// ── Hepta 7D Coordinate ──────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hepta {
    pub d1_identity:  f32,  // uuid_hash    [0.0, 1.0]
    pub d2_belonging: f32,  // tribe_id     [0.0, 1.0]
    pub d3_domain:    f32,  // anomaly RED  [0.0, 1.0]
    pub d4_quality:   f32,  // confidence   [0.0, 1.0]
    pub d5_freshness: f32,  // recency BLUE [0.0, 1.0]
    pub d6_temporal:  f32,  // epoch norm   [0.0, 1.0]
    pub d7_integrity: f32,  // checksum     [0.0, 1.0]
}

impl Hepta {
    /// HPS — Hepta Priority Score (Particles Algebra §6.1)
    /// HPS(p) = 0.40×D4 + 0.25×D3 + 0.15×(1−D5) + 0.10×D2 + 0.10×(1−D7)
    pub fn hps(&self) -> f32 {
        0.40 * self.d4_quality
            + 0.25 * self.d3_domain
            + 0.15 * (1.0 - self.d5_freshness)
            + 0.10 * self.d2_belonging
            + 0.10 * (1.0 - self.d7_integrity)
    }

    /// Cosine similarity σ(p1, p2) over 7D Hepta space (PA-7)
    pub fn similarity(&self, other: &Hepta) -> f32 {
        let a = self.vec();
        let b = other.vec();
        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let na: f32  = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let nb: f32  = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        if na < 1e-8 || nb < 1e-8 { 0.0 } else { (dot / (na * nb)).clamp(0.0, 1.0) }
    }

    pub fn vec(&self) -> [f32; 7] {
        [self.d1_identity, self.d2_belonging, self.d3_domain,
         self.d4_quality, self.d5_freshness, self.d6_temporal, self.d7_integrity]
    }
}

// ── Body Tribes ───────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum BodyTribe {
    HeadNeck = 0,
    Torso    = 1,
    Arms     = 2,
    Legs     = 3,
}

impl BodyTribe {
    pub fn norm(&self) -> f32 { *self as u8 as f32 / 3.0 }
}

// ── Body Type ─────────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BodyType { Man, Woman, Child }

// ── Zone Identifiers ─────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ZoneId {
    Head       = 0,
    Neck       = 1,
    Chest      = 2,
    Abdomen    = 3,
    LShoulderZ = 4,
    RShoulderZ = 5,
    LUpperArm  = 6,
    RUpperArm  = 7,
    LForearm   = 8,
    RForearm   = 9,
    LHip       = 10,
    RHip       = 11,
    LThigh     = 12,
    RThigh     = 13,
    LShin      = 14,
    RShin      = 15,
}

impl ZoneId {
    pub const ALL: [ZoneId; 16] = [
        ZoneId::Head,       ZoneId::Neck,
        ZoneId::Chest,      ZoneId::Abdomen,
        ZoneId::LShoulderZ, ZoneId::RShoulderZ,
        ZoneId::LUpperArm,  ZoneId::RUpperArm,
        ZoneId::LForearm,   ZoneId::RForearm,
        ZoneId::LHip,       ZoneId::RHip,
        ZoneId::LThigh,     ZoneId::RThigh,
        ZoneId::LShin,      ZoneId::RShin,
    ];

    pub fn tribe(&self) -> BodyTribe {
        match self {
            ZoneId::Head | ZoneId::Neck => BodyTribe::HeadNeck,
            ZoneId::Chest | ZoneId::Abdomen
            | ZoneId::LShoulderZ | ZoneId::RShoulderZ => BodyTribe::Torso,
            ZoneId::LUpperArm | ZoneId::RUpperArm
            | ZoneId::LForearm | ZoneId::RForearm => BodyTribe::Arms,
            ZoneId::LHip | ZoneId::RHip
            | ZoneId::LThigh | ZoneId::RThigh
            | ZoneId::LShin | ZoneId::RShin => BodyTribe::Legs,
        }
    }

    pub fn baseline(&self, body_type: BodyType) -> f32 {
        match body_type {
            BodyType::Man   => self.baseline_man(),
            BodyType::Woman => self.baseline_woman(),
            BodyType::Child => self.baseline_child(),
        }
    }

    /// Baseline normal temperature (°C) — Man sovereign reference
    pub fn baseline_man(&self) -> f32 {
        match self {
            ZoneId::Head       => 36.8,
            ZoneId::Neck       => 36.4,
            ZoneId::Chest      => 36.6,
            ZoneId::Abdomen    => 36.4,
            ZoneId::LShoulderZ | ZoneId::RShoulderZ => 35.8,
            ZoneId::LUpperArm  | ZoneId::RUpperArm  => 34.5,
            ZoneId::LForearm   | ZoneId::RForearm   => 33.8,
            ZoneId::LHip       | ZoneId::RHip       => 35.6,
            ZoneId::LThigh     | ZoneId::RThigh     => 35.0,
            ZoneId::LShin      | ZoneId::RShin       => 33.5,
        }
    }

    /// Woman baseline (+0.2°C core)
    pub fn baseline_woman(&self) -> f32 { self.baseline_man() + 0.2 }

    /// Child baseline (higher surface temperature: +0.5°C head, +0.4°C core, +0.2°C rest)
    pub fn baseline_child(&self) -> f32 {
        match self {
            ZoneId::Head | ZoneId::Neck          => self.baseline_man() + 0.5,
            ZoneId::Chest | ZoneId::Abdomen      => self.baseline_man() + 0.4,
            _                                    => self.baseline_man() + 0.2,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            ZoneId::Head       => "Head",
            ZoneId::Neck       => "Neck",
            ZoneId::Chest      => "Chest",
            ZoneId::Abdomen    => "Abdomen",
            ZoneId::LShoulderZ => "L-Shoulder",
            ZoneId::RShoulderZ => "R-Shoulder",
            ZoneId::LUpperArm  => "L-UpperArm",
            ZoneId::RUpperArm  => "R-UpperArm",
            ZoneId::LForearm   => "L-Forearm",
            ZoneId::RForearm   => "R-Forearm",
            ZoneId::LHip       => "L-Hip",
            ZoneId::RHip       => "R-Hip",
            ZoneId::LThigh     => "L-Thigh",
            ZoneId::RThigh     => "R-Thigh",
            ZoneId::LShin      => "L-Shin",
            ZoneId::RShin      => "R-Shin",
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zone_discriminants_are_stable_indices() {
        assert_eq!(ZoneId::Head  as usize, 0);
        assert_eq!(ZoneId::Neck  as usize, 1);
        assert_eq!(ZoneId::Chest as usize, 2);
        assert_eq!(ZoneId::RShin as usize, 15);
        assert_eq!(ZoneId::ALL.len(), 16);
    }

    #[test]
    fn zone_names_non_empty() {
        for z in ZoneId::ALL { assert!(!z.name().is_empty()); }
    }

    #[test]
    fn body_tribe_norm_in_range() {
        for t in [BodyTribe::HeadNeck, BodyTribe::Torso, BodyTribe::Arms, BodyTribe::Legs] {
            let n = t.norm();
            assert!(n >= 0.0 && n <= 1.0, "norm={n}");
        }
    }

    #[test]
    fn hepta_hps_in_range() {
        let h = Hepta { d1_identity: 0.5, d2_belonging: 0.5, d3_domain: 0.5,
                        d4_quality: 0.5, d5_freshness: 0.5, d6_temporal: 0.5, d7_integrity: 0.5 };
        let hps = h.hps();
        assert!(hps >= 0.0 && hps <= 1.0, "hps={hps}");
    }

    #[test]
    fn hepta_similarity_self_is_one() {
        let h = Hepta { d1_identity: 0.3, d2_belonging: 0.7, d3_domain: 0.1,
                        d4_quality: 0.9, d5_freshness: 0.5, d6_temporal: 0.4, d7_integrity: 0.8 };
        assert!((h.similarity(&h) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn baseline_man_head_highest() {
        assert!(ZoneId::Head.baseline_man() > ZoneId::LForearm.baseline_man());
    }

    #[test]
    fn baseline_child_higher_than_man() {
        assert!(ZoneId::Head.baseline_child() > ZoneId::Head.baseline_man());
    }
}
