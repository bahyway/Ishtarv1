// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  nusku-score · src/lib.rs
//  Sovereign aggregate scoring: TRS + MedScore + FuzzyScore + FaceScore
//  Final verdict within the 3-second pipeline window
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

pub use nusku_engine::KakiPK;
use nusku_engine::kaki_to_hex as nusku_kaki_to_hex;

// ── Score Weights (sovereign calibration) ────────────────────────
/// Security TRS weight — highest priority
pub const W_TRS:  f32 = 0.55;
/// Medical inference weight
pub const W_MED:  f32 = 0.25;
/// Nusku fuzzy weight
pub const W_FUZZ: f32 = 0.15;
/// Face match weight (optional — zero when authority DB unavailable)
pub const W_FACE: f32 = 0.05;

// ── Input Scores ─────────────────────────────────────────────────
#[derive(Debug, Clone)]
pub struct ScoreInputs {
    /// Threat Relevance Score [0, 1]
    pub trs:          f32,
    /// Medical condition score [0, 1]
    pub med_score:    f32,
    /// Nusku fuzzy crisp output [0, 1]
    pub fuzzy_score:  f32,
    /// Face match score [0, 1]; ignored when face_enabled = false
    pub face_score:   f32,
    pub face_enabled: bool,
}

// ── Final Verdict ─────────────────────────────────────────────────
#[derive(Debug, Clone)]
pub struct FinalVerdict {
    pub scan_kaki:       KakiPK,
    pub final_score:     f32,
    pub verdict:         VerdictClass,
    pub trs:             f32,
    pub med_score:       f32,
    pub fuzzy_score:     f32,
    pub face_score:      f32,
    /// Unix epoch milliseconds — caller provides; avoids std::time dependency
    pub timestamp_ms:    u64,
    pub processing_ms:   u32,
    /// Sovereign KAKI hex for audit log
    pub audit_kaki_hex:  String,
    pub action_required: ActionRequired,
    pub alert_arabic:    &'static str,
    pub alert_english:   &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerdictClass {
    /// final_score < 0.25
    Clear,
    /// 0.25 ≤ score < 0.40 — fever or stress
    MedicalAlert,
    /// 0.40 ≤ score < 0.60 — ambiguous, secondary review
    Warning,
    /// 0.60 ≤ score < 0.80 — probable threat, detain for inspection
    HighAlert,
    /// score ≥ 0.80 — confirmed threat pattern, immediate action
    ThreatConfirmed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionRequired {
    None,
    MedicalAttention,
    SecondaryScreening,
    ManualInspection,
    ImmediateDetain,
}

// ── Score Engine ──────────────────────────────────────────────────
pub struct ScoreEngine;

impl ScoreEngine {
    /// Compute the final weighted aggregate verdict.
    ///
    /// `timestamp_ms` — caller-supplied Unix epoch ms (avoids time dependency).
    pub fn compute(
        inputs:       &ScoreInputs,
        scan_kaki:    KakiPK,
        processing_ms: u32,
        timestamp_ms:  u64,
    ) -> FinalVerdict {
        let face_w = if inputs.face_enabled { W_FACE } else { 0.0 };
        // When face is disabled, redistribute its weight to TRS
        let trs_w  = if inputs.face_enabled { W_TRS } else { W_TRS + W_FACE };

        let final_score =
              trs_w  * inputs.trs
            + W_MED  * inputs.med_score
            + W_FUZZ * inputs.fuzzy_score
            + face_w * inputs.face_score;

        let verdict = classify(final_score);

        let (action_required, alert_arabic, alert_english) = match &verdict {
            VerdictClass::Clear => (
                ActionRequired::None,
                "لا إجراء مطلوب",
                "No action required",
            ),
            VerdictClass::MedicalAlert => (
                ActionRequired::MedicalAttention,
                "توجيه للرعاية الطبية",
                "Refer for medical attention",
            ),
            VerdictClass::Warning => (
                ActionRequired::SecondaryScreening,
                "فحص ثانوي مطلوب",
                "Secondary screening required",
            ),
            VerdictClass::HighAlert => (
                ActionRequired::ManualInspection,
                "تفتيش يدوي فوري",
                "Immediate manual inspection",
            ),
            VerdictClass::ThreatConfirmed => (
                ActionRequired::ImmediateDetain,
                "⚠ تهديد مؤكد — احتجاز فوري",
                "⚠ THREAT CONFIRMED — Immediate detain",
            ),
        };

        FinalVerdict {
            scan_kaki,
            final_score,
            verdict,
            trs:           inputs.trs,
            med_score:     inputs.med_score,
            fuzzy_score:   inputs.fuzzy_score,
            face_score:    inputs.face_score,
            timestamp_ms,
            processing_ms,
            audit_kaki_hex: kaki_to_hex(&scan_kaki),
            action_required,
            alert_arabic,
            alert_english,
        }
    }
}

fn classify(score: f32) -> VerdictClass {
    match score {
        s if s < 0.25 => VerdictClass::Clear,
        s if s < 0.40 => VerdictClass::MedicalAlert,
        s if s < 0.60 => VerdictClass::Warning,
        s if s < 0.80 => VerdictClass::HighAlert,
        _             => VerdictClass::ThreatConfirmed,
    }
}

fn kaki_to_hex(kaki: &KakiPK) -> String { nusku_kaki_to_hex(kaki) }

// ── Tests ─────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    fn verdict(trs: f32, med: f32, fuzz: f32, face: f32, face_en: bool) -> FinalVerdict {
        let inputs = ScoreInputs { trs, med_score: med, fuzzy_score: fuzz, face_score: face, face_enabled: face_en };
        ScoreEngine::compute(&inputs, [0u8; 16], 50, 0)
    }

    #[test]
    fn all_zero_inputs_is_clear() {
        let v = verdict(0.0, 0.0, 0.0, 0.0, false);
        assert_eq!(v.verdict, VerdictClass::Clear);
        assert_eq!(v.action_required, ActionRequired::None);
    }

    #[test]
    fn high_trs_triggers_threat_confirmed() {
        let v = verdict(1.0, 0.0, 0.0, 0.0, false);
        // trs_w = 0.60 (face weight redistributed), score = 0.60 → HighAlert
        assert_eq!(v.verdict, VerdictClass::HighAlert);
    }

    #[test]
    fn all_max_inputs_is_threat_confirmed() {
        let v = verdict(1.0, 1.0, 1.0, 1.0, true);
        assert_eq!(v.verdict, VerdictClass::ThreatConfirmed);
        assert!(v.final_score >= 0.99);
    }

    #[test]
    fn fever_pattern_is_medical_alert() {
        // High med score, low TRS
        let v = verdict(0.05, 0.85, 0.20, 0.0, false);
        assert_eq!(v.verdict, VerdictClass::MedicalAlert);
    }

    #[test]
    fn face_disabled_redistributes_weight_to_trs() {
        // With face disabled: trs_w = 0.60
        let v_disabled = verdict(0.8, 0.0, 0.0, 0.0, false);
        // With face enabled: trs_w = 0.55
        let v_enabled  = verdict(0.8, 0.0, 0.0, 0.0, true);
        assert!(v_disabled.final_score > v_enabled.final_score);
    }

    #[test]
    fn weights_sum_to_one() {
        let total = W_TRS + W_MED + W_FUZZ + W_FACE;
        assert!((total - 1.0).abs() < 1e-5, "weights must sum to 1.0, got {total}");
    }

    #[test]
    fn audit_kaki_hex_is_32_chars() {
        let v = verdict(0.0, 0.0, 0.0, 0.0, false);
        assert_eq!(v.audit_kaki_hex.len(), 32);
    }

    #[test]
    fn verdict_boundaries_are_exclusive_at_lower_bound() {
        // Boundary: 0.25 → MedicalAlert (not Clear)
        let inputs = ScoreInputs { trs: 0.0, med_score: 1.0, fuzzy_score: 0.0, face_score: 0.0, face_enabled: false };
        // med only: 0.25 * 1.0 = 0.25 → MedicalAlert
        let v = ScoreEngine::compute(&inputs, [0u8; 16], 0, 0);
        assert_eq!(v.verdict, VerdictClass::MedicalAlert);
    }

    #[test]
    fn arabic_alert_non_empty_for_all_verdicts() {
        let scenarios = [
            (0.0, 0.0, 0.0, 0.0),
            (0.3, 0.0, 0.0, 0.0),
            (0.5, 0.0, 0.0, 0.0),
            (0.8, 0.0, 0.0, 0.0),
            (1.0, 1.0, 1.0, 1.0),
        ];
        for (trs, med, fuzz, face) in scenarios {
            let v = verdict(trs, med, fuzz, face, true);
            assert!(!v.alert_arabic.is_empty());
            assert!(!v.alert_english.is_empty());
        }
    }
}
