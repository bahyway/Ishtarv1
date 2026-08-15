//! 𒀭𒄿𒍪𒉡 Azugallu — 17 sovereign medical thermal condition definitions.
//!
//! DUB.SAR 𒁾 — BahyWay.Ecosystem v4.0 | Pure Rust

use nusku_engine::{ZoneId, BodyTribe};

// ── MedicalCondition ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MedicalCondition {
    // ── Fever / Infectious ───────────────────────────────────────────────────
    /// Uniform whole-body elevation +1.5–3.5°C; symmetric; forehead/temporal peak
    SystemicFever,
    /// Chest + throat + nasal bridge elevated; respiratory tribe dominant
    RespiratoryInfection,
    /// Single asymmetric zone elevated >1.5°C — wound, abscess, bite
    LocalizedInfection,
    /// Bilateral joint pairs elevated >0.8°C — arthritis, gout, autoimmune
    JointInflammation,
    /// Skin surface patches: irregular hot zones, no depth
    SkinInflammation,

    // ── Circulatory ──────────────────────────────────────────────────────────
    /// Left-right limb temperature asymmetry >1.2°C — DVT, PAD, Raynaud
    CirculatoryAsymmetry,
    /// Cold extremities + warm torso — poor peripheral circulation
    PoorPeripheralCirculation,
    /// Elevated chest + neck; face flushing — hypertension crisis
    HypertensionFlush,

    // ── Neurological / Autonomic ─────────────────────────────────────────────
    /// Cold palms + hot face — anxiety, PTSD, panic.
    /// Overlaps with security adrenaline response — fuzzy disambiguation required.
    AnxietyResponse,
    /// Elevated anterior neck (thyroid zone) — hyperthyroidism
    ThyroidElevation,

    // ── Metabolic ────────────────────────────────────────────────────────────
    /// Cold feet + legs (poor distal perfusion) — diabetic neuropathy
    DiabeticPeripheral,
    /// Elevated neck/upper torso; night-sweat pattern — hormonal
    HormonalFlush,

    // ── Injury / Post-event ──────────────────────────────────────────────────
    /// Zone still warm from recent surgery — post-operative
    PostSurgicalWarm,
    /// Intense local heat, irregular boundary — burn injury
    BurnInjury,
    /// Lower-extremity elevation, single zone — recent fracture/trauma
    TraumaHot,

    // ── Security-overlap ─────────────────────────────────────────────────────
    /// Elevated face+neck + cold palms — pure adrenaline (no vest cold shadow).
    /// Distinguished from security threat by ABSENCE of torso cold shadow.
    AdrenalineOnly,

    // ── Sentinel ─────────────────────────────────────────────────────────────
    NormalVariation,
    Unclassified,
}

impl MedicalCondition {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::SystemicFever             => "Systemic Fever",
            Self::RespiratoryInfection      => "Respiratory Infection",
            Self::LocalizedInfection        => "Localized Infection",
            Self::JointInflammation         => "Joint Inflammation",
            Self::SkinInflammation          => "Skin Inflammation",
            Self::CirculatoryAsymmetry      => "Circulatory Asymmetry",
            Self::PoorPeripheralCirculation => "Poor Peripheral Circulation",
            Self::HypertensionFlush         => "Hypertension Flush",
            Self::AnxietyResponse           => "Anxiety / Stress Response",
            Self::ThyroidElevation          => "Thyroid Elevation",
            Self::DiabeticPeripheral        => "Diabetic Peripheral Neuropathy",
            Self::HormonalFlush             => "Hormonal Flush",
            Self::PostSurgicalWarm          => "Post-Surgical Zone",
            Self::BurnInjury                => "Burn Injury",
            Self::TraumaHot                 => "Trauma / Fracture Zone",
            Self::AdrenalineOnly            => "Adrenaline Elevation (no threat)",
            Self::NormalVariation           => "Normal Variation",
            Self::Unclassified              => "Unclassified",
        }
    }

    /// Arabic display name for Iraq deployment.
    pub fn display_arabic(&self) -> &'static str {
        match self {
            Self::SystemicFever             => "حمى جهازية",
            Self::RespiratoryInfection      => "عدوى تنفسية",
            Self::LocalizedInfection        => "عدوى موضعية",
            Self::JointInflammation         => "التهاب مفاصل",
            Self::SkinInflammation          => "التهاب جلدي",
            Self::CirculatoryAsymmetry      => "عدم تناسق الدورة الدموية",
            Self::PoorPeripheralCirculation => "ضعف الدورة الطرفية",
            Self::HypertensionFlush         => "احتقان ارتفاع ضغط الدم",
            Self::AnxietyResponse           => "استجابة القلق",
            Self::ThyroidElevation          => "ارتفاع حرارة الغدة الدرقية",
            Self::DiabeticPeripheral        => "اعتلال الأعصاب السكري",
            Self::HormonalFlush             => "احتقان هرموني",
            Self::PostSurgicalWarm          => "منطقة ما بعد الجراحة",
            Self::BurnInjury                => "إصابة حرق",
            Self::TraumaHot                 => "منطقة صدمة",
            Self::AdrenalineOnly            => "ارتفاع الأدرينالين فقط",
            Self::NormalVariation           => "تباين طبيعي",
            Self::Unclassified              => "غير مصنف",
        }
    }

    pub fn escalation(&self) -> EscalationLevel {
        match self {
            Self::SystemicFever | Self::RespiratoryInfection    => EscalationLevel::Medical,
            Self::LocalizedInfection | Self::JointInflammation  => EscalationLevel::MedicalMonitor,
            Self::CirculatoryAsymmetry | Self::HypertensionFlush => EscalationLevel::Medical,
            Self::AdrenalineOnly                                => EscalationLevel::SecurityMonitor,
            Self::NormalVariation                               => EscalationLevel::None,
            Self::Unclassified                                  => EscalationLevel::Review,
            _                                                   => EscalationLevel::MedicalMonitor,
        }
    }
}

// ── EscalationLevel ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EscalationLevel {
    None,
    Review,
    MedicalMonitor,
    Medical,
    SecurityMonitor,
    SecurityAlert,
}

// ── ThermalSignature ──────────────────────────────────────────────────────────

/// Expected thermal pattern for a medical condition.
///
/// Used by `AzugaEngine::infer()` to score each catalogue entry against
/// a live `BodyScan`.
#[derive(Debug, Clone)]
pub struct ThermalSignature {
    pub condition:         MedicalCondition,
    /// Per-zone expected deviation from baseline (°C)
    pub zone_deltas:       Vec<(ZoneId, f32)>,
    /// Symmetry requirement: true = symmetric pattern expected
    pub symmetric:         bool,
    /// Dominant body tribe for this condition
    pub dominant_tribe:    BodyTribe,
    /// Minimum deviation magnitude to activate scoring
    pub min_trigger_delta: f32,
    /// Maximum expected total body deviation
    pub max_total_delta:   f32,
}

impl ThermalSignature {
    /// The 9-entry sovereign medical catalogue.
    pub fn catalogue() -> Vec<Self> {
        vec![
            Self {
                condition: MedicalCondition::SystemicFever,
                zone_deltas: vec![
                    (ZoneId::Head,       2.2),
                    (ZoneId::Neck,       1.8),
                    (ZoneId::Chest,      1.6),
                    (ZoneId::Abdomen,    1.5),
                    (ZoneId::LShoulderZ, 1.2),
                    (ZoneId::RShoulderZ, 1.2),
                    (ZoneId::LUpperArm,  0.9),
                    (ZoneId::RUpperArm,  0.9),
                    (ZoneId::LForearm,   0.7),
                    (ZoneId::RForearm,   0.7),
                ],
                symmetric: true,
                dominant_tribe: BodyTribe::HeadNeck,
                min_trigger_delta: 1.2,
                max_total_delta:   3.5,
            },
            Self {
                condition: MedicalCondition::RespiratoryInfection,
                zone_deltas: vec![
                    (ZoneId::Head,  1.4),
                    (ZoneId::Neck,  1.8),
                    (ZoneId::Chest, 1.3),
                ],
                symmetric: true,
                dominant_tribe: BodyTribe::HeadNeck,
                min_trigger_delta: 0.8,
                max_total_delta:   2.5,
            },
            Self {
                condition: MedicalCondition::AnxietyResponse,
                zone_deltas: vec![
                    (ZoneId::Head,     1.5),
                    (ZoneId::Neck,     1.2),
                    (ZoneId::LForearm, -0.6),
                    (ZoneId::RForearm, -0.6),
                ],
                symmetric: true,
                dominant_tribe: BodyTribe::HeadNeck,
                min_trigger_delta: 0.7,
                max_total_delta:   2.0,
            },
            Self {
                condition: MedicalCondition::CirculatoryAsymmetry,
                zone_deltas: vec![
                    (ZoneId::LUpperArm, -1.5),
                    (ZoneId::LForearm,  -1.8),
                    (ZoneId::LShin,     -1.6),
                ],
                symmetric: false,
                dominant_tribe: BodyTribe::Arms,
                min_trigger_delta: 1.0,
                max_total_delta:   2.5,
            },
            Self {
                condition: MedicalCondition::JointInflammation,
                zone_deltas: vec![
                    (ZoneId::LHip,   0.9),
                    (ZoneId::RHip,   0.9),
                    (ZoneId::LThigh, 0.7),
                    (ZoneId::RThigh, 0.7),
                ],
                symmetric: true,
                dominant_tribe: BodyTribe::Legs,
                min_trigger_delta: 0.6,
                max_total_delta:   1.5,
            },
            Self {
                condition: MedicalCondition::ThyroidElevation,
                zone_deltas: vec![
                    (ZoneId::Neck, 1.6),
                ],
                symmetric: true,
                dominant_tribe: BodyTribe::HeadNeck,
                min_trigger_delta: 1.0,
                max_total_delta:   2.0,
            },
            Self {
                condition: MedicalCondition::DiabeticPeripheral,
                zone_deltas: vec![
                    (ZoneId::LShin,  -1.8),
                    (ZoneId::RShin,  -1.8),
                    (ZoneId::LThigh, -0.8),
                    (ZoneId::RThigh, -0.8),
                ],
                symmetric: true,
                dominant_tribe: BodyTribe::Legs,
                min_trigger_delta: 1.2,
                max_total_delta:   2.5,
            },
            Self {
                condition: MedicalCondition::HypertensionFlush,
                zone_deltas: vec![
                    (ZoneId::Head,  1.8),
                    (ZoneId::Neck,  1.4),
                    (ZoneId::Chest, 0.8),
                ],
                symmetric: true,
                dominant_tribe: BodyTribe::HeadNeck,
                min_trigger_delta: 1.0,
                max_total_delta:   2.2,
            },
            Self {
                condition: MedicalCondition::AdrenalineOnly,
                zone_deltas: vec![
                    (ZoneId::Head,     1.2),
                    (ZoneId::Neck,     1.0),
                    (ZoneId::LForearm, -0.5),
                    (ZoneId::RForearm, -0.5),
                    // torso NORMAL — absence of cold shadow distinguishes from threat
                ],
                symmetric: true,
                dominant_tribe: BodyTribe::HeadNeck,
                min_trigger_delta: 0.6,
                max_total_delta:   1.8,
            },
        ]
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const ALL_CONDITIONS: &[MedicalCondition] = &[
        MedicalCondition::SystemicFever,
        MedicalCondition::RespiratoryInfection,
        MedicalCondition::LocalizedInfection,
        MedicalCondition::JointInflammation,
        MedicalCondition::SkinInflammation,
        MedicalCondition::CirculatoryAsymmetry,
        MedicalCondition::PoorPeripheralCirculation,
        MedicalCondition::HypertensionFlush,
        MedicalCondition::AnxietyResponse,
        MedicalCondition::ThyroidElevation,
        MedicalCondition::DiabeticPeripheral,
        MedicalCondition::HormonalFlush,
        MedicalCondition::PostSurgicalWarm,
        MedicalCondition::BurnInjury,
        MedicalCondition::TraumaHot,
        MedicalCondition::AdrenalineOnly,
        MedicalCondition::NormalVariation,
        MedicalCondition::Unclassified,
    ];

    #[test]
    fn all_display_names_non_empty() {
        for c in ALL_CONDITIONS {
            assert!(!c.display_name().is_empty(),  "{c:?} display_name is empty");
            assert!(!c.display_arabic().is_empty(), "{c:?} display_arabic is empty");
        }
    }

    #[test]
    fn catalogue_has_nine_entries() {
        assert_eq!(ThermalSignature::catalogue().len(), 9);
    }

    #[test]
    fn fever_escalates_to_medical() {
        assert_eq!(MedicalCondition::SystemicFever.escalation(), EscalationLevel::Medical);
        assert_eq!(MedicalCondition::RespiratoryInfection.escalation(), EscalationLevel::Medical);
    }

    #[test]
    fn normal_variation_escalates_to_none() {
        assert_eq!(MedicalCondition::NormalVariation.escalation(), EscalationLevel::None);
    }

    #[test]
    fn adrenaline_escalates_to_security_monitor() {
        assert_eq!(MedicalCondition::AdrenalineOnly.escalation(), EscalationLevel::SecurityMonitor);
    }

    #[test]
    fn catalogue_all_have_nonempty_zone_deltas() {
        for sig in ThermalSignature::catalogue() {
            assert!(!sig.zone_deltas.is_empty(),
                "{:?} has no zone deltas", sig.condition);
            assert!(sig.min_trigger_delta > 0.0);
        }
    }
}
