//! 𒀭𒄿𒍪𒉡 Azuga inference engine — scores medical conditions against a BodyScan.
//!
//! Must complete in < 50ms (NuskuEngine pipeline budget).
//!
//! DUB.SAR 𒁾 — BahyWay.Ecosystem v4.0 | Pure Rust

use crate::body::{BodyScan, ZoneId};
use crate::conditions::{EscalationLevel, MedicalCondition, ThermalSignature};
use crate::symmetry::SymmetryAnalysis;

// ── Result types ──────────────────────────────────────────────────────────────

/// Full medical inference result for one BodyScan frame.
#[derive(Debug, Clone)]
pub struct MedicalInferenceResult {
    /// Primary inferred condition
    pub primary:          MedicalCondition,
    /// Confidence in primary condition [0, 1]
    pub confidence:       f32,
    /// All candidates with scores, sorted descending
    pub candidates:       Vec<CandidateScore>,
    pub symmetry:         SymmetryAnalysis,
    /// Aggregate medical risk score [0, 1]
    pub med_score:        f32,
    /// True when torso cold-shadow is present — triggers fuzzy-engine disambiguation.
    pub security_overlap: bool,
    pub escalation:       EscalationLevel,
    /// Reasoning chain for PMPVD audit log
    pub reasoning:        Vec<String>,
}

/// One scored catalogue candidate.
#[derive(Debug, Clone)]
pub struct CandidateScore {
    pub condition: MedicalCondition,
    /// Match score [0, 1]
    pub score:    f32,
    pub evidence: Vec<String>,
}

// ── AzugaEngine ───────────────────────────────────────────────────────────────

/// Sovereign medical thermal inference engine (𒀭𒄿𒍪𒉡 Azugallu).
pub struct AzugaEngine {
    catalogue: Vec<ThermalSignature>,
}

impl AzugaEngine {
    pub fn new() -> Self {
        Self { catalogue: ThermalSignature::catalogue() }
    }

    /// Run full medical inference on a BodyScan frame.
    pub fn infer(&self, scan: &BodyScan) -> MedicalInferenceResult {
        let symmetry = SymmetryAnalysis::compute(scan);
        let mut candidates: Vec<CandidateScore> = Vec::new();
        let mut reasoning:  Vec<String>         = Vec::new();

        // ── Score each catalogue entry ─────────────────────────────────────────
        for sig in &self.catalogue {
            let (score, evidence) = self.score_signature(scan, sig, &symmetry);
            if score > 0.05 {
                candidates.push(CandidateScore {
                    condition: sig.condition.clone(),
                    score,
                    evidence,
                });
            }
        }

        candidates.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        // ── Security overlap: cold shadow on torso ────────────────────────────
        let chest_dev   = scan.zone(ZoneId::Chest).deviation;
        let abdomen_dev = scan.zone(ZoneId::Abdomen).deviation;
        let security_overlap = chest_dev < -1.2 || abdomen_dev < -1.5;

        if security_overlap {
            reasoning.push(format!(
                "⚠ Torso cold shadow: chest_dev={:.2}°C, abdomen_dev={:.2}°C",
                chest_dev, abdomen_dev
            ));
            reasoning.push("Security overlap — fuzzy-engine disambiguation required".to_string());
        }

        // ── Primary finding ───────────────────────────────────────────────────
        let (primary, confidence) = if let Some(top) = candidates.first() {
            reasoning.push(format!(
                "Primary: {} (score={:.3})", top.condition.display_name(), top.score
            ));
            (top.condition.clone(), top.score)
        } else {
            (MedicalCondition::NormalVariation, 0.92)
        };

        let mean_dev  = scan.mean_deviation();
        let med_score = compute_med_score(&primary, confidence, mean_dev, security_overlap);

        let escalation = if security_overlap {
            EscalationLevel::SecurityAlert
        } else {
            primary.escalation()
        };

        MedicalInferenceResult {
            primary, confidence, candidates, symmetry,
            med_score, security_overlap, escalation, reasoning,
        }
    }

    fn score_signature(
        &self,
        scan: &BodyScan,
        sig:  &ThermalSignature,
        sym:  &SymmetryAnalysis,
    ) -> (f32, Vec<String>) {
        let mut hits  = 0u32;
        let mut total = 0u32;
        let mut evidence = Vec::new();

        for &(zone, expected_delta) in &sig.zone_deltas {
            total += 1;
            let actual_dev = scan.zone(zone).deviation;

            // Direction match: same sign within tolerance
            let direction_ok = (actual_dev * expected_delta) > 0.0;
            // Magnitude match: within 60% of expected
            let mag_ratio    = if expected_delta.abs() > 0.01 {
                (actual_dev.abs() / expected_delta.abs()).min(2.0)
            } else { 1.0 };
            let magnitude_ok = mag_ratio > 0.4;

            if direction_ok && magnitude_ok {
                hits += 1;
                evidence.push(format!(
                    "{}: actual={:.2}°C expected≈{:.2}°C ✓",
                    zone.name(), actual_dev, expected_delta
                ));
            } else {
                evidence.push(format!(
                    "{}: actual={:.2}°C expected≈{:.2}°C ✗",
                    zone.name(), actual_dev, expected_delta
                ));
            }
        }

        let zone_score = if total > 0 { hits as f32 / total as f32 } else { 0.0 };

        let sym_score = if sig.symmetric {
            sym.overall_symmetry * 0.3 + zone_score * 0.7
        } else {
            (1.0 - sym.overall_symmetry) * 0.3 + zone_score * 0.7
        };

        let any_above_trigger = sig.zone_deltas.iter()
            .any(|&(z, _)| scan.zone(z).deviation.abs() >= sig.min_trigger_delta);

        // Zero zone hits → condition doesn't apply regardless of symmetry
        let final_score = if hits == 0 {
            0.0
        } else if any_above_trigger {
            sym_score
        } else {
            sym_score * 0.3
        };
        (final_score.clamp(0.0, 1.0), evidence)
    }
}

impl Default for AzugaEngine {
    fn default() -> Self { Self::new() }
}

fn compute_med_score(
    condition:        &MedicalCondition,
    confidence:       f32,
    mean_dev:         f32,
    security_overlap: bool,
) -> f32 {
    let base = match condition {
        MedicalCondition::SystemicFever        => 0.75,
        MedicalCondition::RespiratoryInfection => 0.65,
        MedicalCondition::HypertensionFlush    => 0.70,
        MedicalCondition::CirculatoryAsymmetry => 0.60,
        MedicalCondition::AnxietyResponse      => 0.40,
        MedicalCondition::AdrenalineOnly       => 0.35,
        MedicalCondition::NormalVariation      => 0.05,
        _                                      => 0.45,
    };
    let dev_boost       = (mean_dev.abs() / 3.0).min(0.25);
    let overlap_penalty = if security_overlap { -0.15 } else { 0.0 };
    (base * confidence + dev_boost + overlap_penalty).clamp(0.0, 1.0)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use nusku_engine::{BodyScan, BodyType};
    use crate::conditions::MedicalCondition;

    fn engine() -> AzugaEngine { AzugaEngine::new() }

    #[test]
    fn normal_scan_infers_normal_variation() {
        let result = engine().infer(&BodyScan::uniform(BodyType::Man, 0));
        assert_eq!(result.primary, MedicalCondition::NormalVariation);
        assert!(!result.security_overlap);
    }

    #[test]
    fn systemic_fever_scan_detected() {
        // Neck=0.5 → ThyroidElevation magnitude check fails (0.5/1.6=0.31 < 0.4).
        // SystemicFever hits 9/10 (Neck misses), RespiratoryInfection hits 2/3 → fever wins.
        let scan = BodyScan::from_deviations(BodyType::Man, &[
            (ZoneId::Head,       2.2),
            (ZoneId::Neck,       0.5),  // too small for ThyroidElevation (needs 0.64+)
            (ZoneId::Chest,      1.6),
            (ZoneId::Abdomen,    1.5),
            (ZoneId::LShoulderZ, 1.2),
            (ZoneId::RShoulderZ, 1.2),
            (ZoneId::LUpperArm,  0.9),
            (ZoneId::RUpperArm,  0.9),
            (ZoneId::LForearm,   0.7),
            (ZoneId::RForearm,   0.7),
        ], 0);
        let result = engine().infer(&scan);
        assert_eq!(result.primary, MedicalCondition::SystemicFever,
            "got {:?} score={:.3}", result.primary,
            result.candidates.first().map_or(0.0, |c| c.score));
    }

    #[test]
    fn cold_torso_triggers_security_overlap_and_alert() {
        let scan = BodyScan::from_deviations(BodyType::Man, &[
            (ZoneId::Chest,   -1.5),
            (ZoneId::Abdomen, -1.8),
        ], 0);
        let result = engine().infer(&scan);
        assert!(result.security_overlap, "cold torso must trigger security_overlap");
        assert_eq!(result.escalation, EscalationLevel::SecurityAlert);
    }

    #[test]
    fn normal_scan_has_no_security_overlap() {
        let result = engine().infer(&BodyScan::uniform(BodyType::Man, 0));
        assert!(!result.security_overlap);
    }

    #[test]
    fn candidates_sorted_descending() {
        let scan = BodyScan::from_deviations(BodyType::Man, &[
            (ZoneId::Head, 2.2), (ZoneId::Neck, 1.8),
        ], 0);
        let result = engine().infer(&scan);
        for i in 0..result.candidates.len().saturating_sub(1) {
            assert!(result.candidates[i].score >= result.candidates[i + 1].score,
                "candidates not sorted at index {i}");
        }
    }

    #[test]
    fn reasoning_chain_non_empty_on_security_overlap() {
        let scan = BodyScan::from_deviations(BodyType::Man, &[(ZoneId::Chest, -2.0)], 0);
        let result = engine().infer(&scan);
        assert!(!result.reasoning.is_empty());
        assert!(result.reasoning.iter().any(|r| r.contains("cold shadow")));
    }

    #[test]
    fn med_score_bounded_zero_to_one() {
        for dev in [-2.0f32, -0.5, 0.0, 1.0, 3.0] {
            // Build a scan where all zones have the requested deviation
            let overrides: Vec<(ZoneId, f32)> = ZoneId::ALL.iter().map(|&z| (z, dev)).collect();
            let scan   = BodyScan::from_deviations(BodyType::Man, &overrides, 0);
            let result = engine().infer(&scan);
            assert!(result.med_score >= 0.0 && result.med_score <= 1.0,
                "med_score={} for dev={dev}", result.med_score);
        }
    }
}
