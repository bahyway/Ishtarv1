//! Sovereign Mamdani fuzzy inference engine.
//!
//! Input:  BodyScan + pre-computed symmetry/inversion scores
//! Output: FuzzyVerdict — sovereign state classification
//!
//! Pipeline budget: < 30ms per frame.
//!
//! DUB.SAR 𒁾 — BahyWay.Ecosystem v4.0 | Pure Rust

use nusku_engine::{BodyScan, ZoneId};
use std::collections::HashMap;

use crate::defuzz::centroid_defuzz;
use crate::membership::{
    var_arm_dev, var_head_dev, var_state_output, var_symmetry, var_torso_arm_inversion,
    var_torso_dev, LinguisticVar,
};
use crate::rules::{nusku_rule_base, Connective, FuzzyRule};

// ── FuzzyState ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum FuzzyState {
    Clear,
    Fever,
    Stress,
    MedicalOther,
    Ambiguous,
    SecurityThreat,
}

impl FuzzyState {
    pub fn display(&self) -> &'static str {
        match self {
            Self::Clear => "CLEAR",
            Self::Fever => "FEVER",
            Self::Stress => "STRESS / ANXIETY",
            Self::MedicalOther => "MEDICAL CONDITION",
            Self::Ambiguous => "AMBIGUOUS — REVIEW",
            Self::SecurityThreat => "SECURITY THREAT",
        }
    }

    pub fn display_arabic(&self) -> &'static str {
        match self {
            Self::Clear => "سليم",
            Self::Fever => "حمى",
            Self::Stress => "توتر / قلق",
            Self::MedicalOther => "حالة طبية",
            Self::Ambiguous => "غير محدد — مراجعة",
            Self::SecurityThreat => "تهديد أمني",
        }
    }

    /// Sovereign alert level (0 = no action, 5 = immediate detain).
    pub fn alert_level(&self) -> u8 {
        match self {
            Self::Clear => 0,
            Self::Stress => 1,
            Self::Fever => 2,
            Self::MedicalOther => 2,
            Self::Ambiguous => 3,
            Self::SecurityThreat => 5,
        }
    }
}

// ── FuzzyVerdict ──────────────────────────────────────────────────────────────

/// Full inference result for one scan frame.
#[derive(Debug, Clone)]
pub struct FuzzyVerdict {
    /// Sovereign state classification
    pub state: FuzzyState,
    /// Crisp output score [0, 1] from defuzzification
    pub crisp_score: f32,
    /// Rule firing strengths: (rule_id, strength, description)
    pub rule_activations: Vec<(u32, f32, String)>,
    /// Membership values for all input variables
    pub input_memberships: HashMap<String, Vec<(String, f32)>>,
    /// Reasoning trace for PMPVD audit log
    pub reasoning: Vec<String>,
    /// Confidence in classification [0, 1]
    pub confidence: f32,
    /// Iraq deployment flag — elevated sensitivity
    pub iraq_protocol: bool,
}

// ── FuzzyEngine ───────────────────────────────────────────────────────────────

/// Nusku sovereign Mamdani fuzzy inference engine.
pub struct FuzzyEngine {
    rules: Vec<FuzzyRule>,
    torso_var: LinguisticVar,
    head_var: LinguisticVar,
    arm_var: LinguisticVar,
    symmetry_var: LinguisticVar,
    inversion_var: LinguisticVar,
    output_var: LinguisticVar,
    iraq_mode: bool,
}

impl FuzzyEngine {
    pub fn new(iraq_protocol: bool) -> Self {
        Self {
            rules: nusku_rule_base(),
            torso_var: var_torso_dev(),
            head_var: var_head_dev(),
            arm_var: var_arm_dev(),
            symmetry_var: var_symmetry(),
            inversion_var: var_torso_arm_inversion(),
            output_var: var_state_output(),
            iraq_mode: iraq_protocol,
        }
    }

    /// Run full Mamdani inference on a BodyScan frame.
    ///
    /// `symmetry_score` and `inversion` are pre-computed by `SymmetryAnalysis`
    /// from `azuga-engine`.
    pub fn infer(&self, scan: &BodyScan, symmetry_score: f32, inversion: f32) -> FuzzyVerdict {
        // ── Fuzzify inputs ─────────────────────────────────────────────────────
        let torso_dev =
            (scan.zone(ZoneId::Chest).deviation + scan.zone(ZoneId::Abdomen).deviation) / 2.0;
        let head_dev = scan.zone(ZoneId::Head).deviation;
        let arm_dev =
            (scan.zone(ZoneId::LUpperArm).deviation + scan.zone(ZoneId::RUpperArm).deviation) / 2.0;

        let mut input_memberships = HashMap::new();
        input_memberships.insert("torso_deviation".into(), self.torso_var.eval_all(torso_dev));
        input_memberships.insert("head_deviation".into(), self.head_var.eval_all(head_dev));
        input_memberships.insert("arm_deviation".into(), self.arm_var.eval_all(arm_dev));
        input_memberships.insert(
            "symmetry".into(),
            self.symmetry_var.eval_all(symmetry_score),
        );
        input_memberships.insert(
            "torso_arm_inversion".into(),
            self.inversion_var.eval_all(inversion),
        );

        // ── Fire rules (Mamdani T-norm) ────────────────────────────────────────
        let mut rule_activations: Vec<(u32, f32, String)> = Vec::new();
        let mut output_clips: HashMap<String, f32> = HashMap::new();

        for rule in &self.rules {
            let strength = self.fire_rule(rule, &input_memberships);
            let weighted = strength * rule.weight;

            // Iraq protocol: boost ambiguous → security by 15% on partial signals
            let final_strength =
                if self.iraq_mode && rule.consequent.term == "ambiguous" && weighted > 0.4 {
                    (weighted * 1.15).min(1.0)
                } else {
                    weighted
                };

            if final_strength > 0.01 {
                rule_activations.push((rule.id, final_strength, rule.description.clone()));
                let entry = output_clips
                    .entry(rule.consequent.term.clone())
                    .or_insert(0.0);
                *entry = entry.max(final_strength);
            }
        }

        rule_activations.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // ── Defuzzify (centroid CoG) ───────────────────────────────────────────
        let crisp_score = centroid_defuzz(&output_clips, &self.output_var);

        // ── Classify → FuzzyState ─────────────────────────────────────────────
        let (state, confidence) = self.classify(&output_clips);

        // ── Build reasoning trace ─────────────────────────────────────────────
        let mut reasoning = Vec::new();
        reasoning.push(format!(
            "Inputs: torso_dev={:.2}°C  head_dev={:.2}°C  arm_dev={:.2}°C",
            torso_dev, head_dev, arm_dev
        ));
        reasoning.push(format!(
            "Symmetry={:.3}  Inversion={:.3}",
            symmetry_score, inversion
        ));
        for (id, strength, desc) in rule_activations.iter().take(3) {
            reasoning.push(format!(
                "Rule {:02}: strength={:.3} — {}",
                id, strength, desc
            ));
        }
        reasoning.push(format!("Crisp={:.4} → {}", crisp_score, state.display()));

        FuzzyVerdict {
            state,
            crisp_score,
            rule_activations,
            input_memberships,
            reasoning,
            confidence,
            iraq_protocol: self.iraq_mode,
        }
    }

    fn fire_rule(
        &self,
        rule: &FuzzyRule,
        memberships: &HashMap<String, Vec<(String, f32)>>,
    ) -> f32 {
        let values: Vec<f32> = rule
            .antecedents
            .iter()
            .map(|ant| {
                let mu = memberships
                    .get(&ant.variable)
                    .and_then(|terms| terms.iter().find(|(t, _)| t == &ant.term))
                    .map(|(_, v)| *v)
                    .unwrap_or(0.0);
                if ant.negated {
                    1.0 - mu
                } else {
                    mu
                }
            })
            .collect();

        if values.is_empty() {
            return 0.0;
        }

        match rule.connective {
            Connective::And => values.iter().cloned().fold(f32::MAX, f32::min),
            Connective::Or => values.iter().cloned().fold(f32::MIN, f32::max),
        }
    }

    fn classify(&self, clips: &HashMap<String, f32>) -> (FuzzyState, f32) {
        let dominant = clips.iter().max_by(|a, b| a.1.partial_cmp(b.1).unwrap());
        let state = match dominant.map(|(t, _)| t.as_str()) {
            Some("security_threat") => FuzzyState::SecurityThreat,
            Some("fever") => FuzzyState::Fever,
            Some("stress") => FuzzyState::Stress,
            Some("medical_other") => FuzzyState::MedicalOther,
            Some("ambiguous") => FuzzyState::Ambiguous,
            _ => FuzzyState::Clear,
        };
        let confidence = dominant.map(|(_, v)| *v).unwrap_or(0.1);
        (state, confidence)
    }
}

impl Default for FuzzyEngine {
    fn default() -> Self {
        Self::new(false)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use nusku_engine::{BodyScan, BodyType};

    fn engine() -> FuzzyEngine {
        FuzzyEngine::new(false)
    }
    fn iraq_engine() -> FuzzyEngine {
        FuzzyEngine::new(true)
    }

    #[test]
    fn normal_scan_is_clear() {
        let scan = BodyScan::uniform(BodyType::Man, 0);
        let result = engine().infer(&scan, 1.0, 0.0);
        assert_eq!(
            result.state,
            FuzzyState::Clear,
            "got {:?} crisp={:.4}",
            result.state,
            result.crisp_score
        );
    }

    #[test]
    fn security_threat_pattern_detected() {
        // Cold torso (-2°C) + hot arms (+2°C) + strong inversion (0.8)
        let scan = BodyScan::from_deviations(
            BodyType::Man,
            &[
                (ZoneId::Chest, -2.0),
                (ZoneId::Abdomen, -2.0),
                (ZoneId::LUpperArm, 2.0),
                (ZoneId::RUpperArm, 2.0),
            ],
            0,
        );
        let result = engine().infer(&scan, 0.95, 0.8);
        assert_eq!(
            result.state,
            FuzzyState::SecurityThreat,
            "got {:?} crisp={:.4}",
            result.state,
            result.crisp_score
        );
        assert_eq!(result.state.alert_level(), 5);
    }

    #[test]
    fn fever_pattern_detected() {
        // Hot torso (+2°C) + hot head (+2.5°C) + symmetric
        let scan = BodyScan::from_deviations(
            BodyType::Man,
            &[
                (ZoneId::Chest, 2.0),
                (ZoneId::Abdomen, 2.0),
                (ZoneId::Head, 2.5),
            ],
            0,
        );
        let result = engine().infer(&scan, 0.95, 0.0);
        assert_eq!(
            result.state,
            FuzzyState::Fever,
            "got {:?} crisp={:.4}",
            result.state,
            result.crisp_score
        );
    }

    #[test]
    fn stress_pattern_detected() {
        // Elevated head (+0.9°C) + cold hands (-1.2°C) + normal torso
        let scan = BodyScan::from_deviations(
            BodyType::Man,
            &[
                (ZoneId::Head, 0.9),
                (ZoneId::LForearm, -1.2),
                (ZoneId::RForearm, -1.2),
            ],
            0,
        );
        let result = engine().infer(&scan, 1.0, 0.0);
        assert_eq!(
            result.state,
            FuzzyState::Stress,
            "got {:?} crisp={:.4}",
            result.state,
            result.crisp_score
        );
    }

    #[test]
    fn alert_level_ordering() {
        assert!(FuzzyState::Clear.alert_level() < FuzzyState::Fever.alert_level());
        assert!(FuzzyState::Fever.alert_level() < FuzzyState::Ambiguous.alert_level());
        assert!(FuzzyState::Ambiguous.alert_level() < FuzzyState::SecurityThreat.alert_level());
    }

    #[test]
    fn iraq_mode_flag_preserved_in_verdict() {
        let scan = BodyScan::uniform(BodyType::Man, 0);
        let result = iraq_engine().infer(&scan, 1.0, 0.0);
        assert!(result.iraq_protocol);
    }

    #[test]
    fn reasoning_trace_non_empty() {
        let scan = BodyScan::uniform(BodyType::Man, 0);
        let result = engine().infer(&scan, 1.0, 0.0);
        assert!(!result.reasoning.is_empty());
    }

    #[test]
    fn display_and_arabic_non_empty_for_all_states() {
        let states = [
            FuzzyState::Clear,
            FuzzyState::Fever,
            FuzzyState::Stress,
            FuzzyState::MedicalOther,
            FuzzyState::Ambiguous,
            FuzzyState::SecurityThreat,
        ];
        for s in &states {
            assert!(!s.display().is_empty());
            assert!(!s.display_arabic().is_empty());
        }
    }
}
