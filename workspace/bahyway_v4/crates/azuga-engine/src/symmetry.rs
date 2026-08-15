//! Left-right body symmetry analysis.
//!
//! Key heuristic:
//!   Fever            → symmetric elevation across bilateral pairs
//!   Security threat  → torso-arm thermal inversion (vest insulates torso)
//!
//! DUB.SAR 𒁾 — BahyWay.Ecosystem v4.0 | Pure Rust

use crate::body::{BodyScan, ZoneId};

/// Per-axis symmetry scores and threat-relevant thermal gradients.
#[derive(Debug, Clone)]
pub struct SymmetryAnalysis {
    /// Shoulder pair symmetry [0=asymmetric, 1=symmetric]
    pub shoulder_symmetry:   f32,
    /// Arm pair symmetry [0=asymmetric, 1=symmetric]
    pub arm_symmetry:        f32,
    /// Hip/leg pair symmetry [0=asymmetric, 1=symmetric]
    pub leg_symmetry:        f32,
    /// Overall body symmetry (mean of three axes)
    pub overall_symmetry:    f32,
    /// Torso-arm thermal inversion: 0.0 = normal, > 0.5 = threat indicator.
    /// Normal: torso > arms (core warmer than periphery).
    /// Threat: torso < arms (vest insulates torso; wiring heats arms).
    pub torso_arm_inversion: f32,
    /// Head-torso thermal gradient — positive = head hotter (fever pattern).
    pub head_torso_gradient: f32,
    pub notes:               Vec<String>,
}

impl SymmetryAnalysis {
    pub fn compute(scan: &BodyScan) -> Self {
        let mut notes = Vec::new();

        // ── Bilateral shoulder symmetry ────────────────────────────────────────
        let shoulder_diff = (scan.zone(ZoneId::LShoulderZ).temperature
            - scan.zone(ZoneId::RShoulderZ).temperature).abs();
        let shoulder_symmetry = (1.0 - shoulder_diff / 2.0).clamp(0.0, 1.0);

        // ── Bilateral arm symmetry ─────────────────────────────────────────────
        let arm_ul_diff = (scan.zone(ZoneId::LUpperArm).temperature
            - scan.zone(ZoneId::RUpperArm).temperature).abs();
        let arm_ll_diff = (scan.zone(ZoneId::LForearm).temperature
            - scan.zone(ZoneId::RForearm).temperature).abs();
        let arm_symmetry = (1.0 - (arm_ul_diff + arm_ll_diff) / 4.0).clamp(0.0, 1.0);

        // ── Bilateral leg symmetry ─────────────────────────────────────────────
        let hip_diff   = (scan.zone(ZoneId::LHip).temperature
            - scan.zone(ZoneId::RHip).temperature).abs();
        let thigh_diff = (scan.zone(ZoneId::LThigh).temperature
            - scan.zone(ZoneId::RThigh).temperature).abs();
        let leg_symmetry = (1.0 - (hip_diff + thigh_diff) / 4.0).clamp(0.0, 1.0);

        let overall_symmetry = (shoulder_symmetry + arm_symmetry + leg_symmetry) / 3.0;

        // ── Torso-arm thermal inversion (security indicator) ───────────────────
        let torso_mean = (scan.zone(ZoneId::Chest).temperature
            + scan.zone(ZoneId::Abdomen).temperature) / 2.0;
        let arm_mean = (scan.zone(ZoneId::LUpperArm).temperature
            + scan.zone(ZoneId::RUpperArm).temperature) / 2.0;
        let inversion = arm_mean - torso_mean;          // positive = arms hotter than torso
        let torso_arm_inversion = inversion.max(0.0) / 3.0;

        if torso_arm_inversion > 0.5 {
            notes.push(format!(
                "⚠ Torso-arm thermal inversion: arms {:.1}°C hotter than torso",
                inversion
            ));
        }

        // ── Head-torso gradient ────────────────────────────────────────────────
        let head_torso_gradient = scan.zone(ZoneId::Head).temperature - torso_mean;

        if head_torso_gradient > 1.5 {
            notes.push(format!(
                "Head {:.1}°C hotter than torso — fever pattern",
                head_torso_gradient
            ));
        }

        if shoulder_diff > 1.0 {
            notes.push(format!("Shoulder asymmetry: {:.2}°C — investigate", shoulder_diff));
        }

        Self {
            shoulder_symmetry,
            arm_symmetry,
            leg_symmetry,
            overall_symmetry,
            torso_arm_inversion,
            head_torso_gradient,
            notes,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use nusku_engine::{BodyScan, BodyType};

    #[test]
    fn uniform_scan_gives_perfect_symmetry() {
        let sym = SymmetryAnalysis::compute(&BodyScan::uniform(BodyType::Man, 0));
        assert!((sym.overall_symmetry - 1.0).abs() < 0.01,
            "overall_symmetry={}", sym.overall_symmetry);
    }

    #[test]
    fn asymmetric_arm_lowers_arm_symmetry() {
        let scan = BodyScan::from_deviations(BodyType::Man, &[(ZoneId::LUpperArm, -2.5)], 0);
        let sym  = SymmetryAnalysis::compute(&scan);
        assert!(sym.arm_symmetry < 0.9, "arm_symmetry={}", sym.arm_symmetry);
    }

    #[test]
    fn torso_arm_inversion_detected_when_arms_hotter() {
        // Arms 3.5°C hotter than torso → strong inversion signal
        let scan = BodyScan::from_deviations(BodyType::Man, &[
            (ZoneId::LUpperArm,  2.0),
            (ZoneId::RUpperArm,  2.0),
            (ZoneId::Chest,     -1.5),
            (ZoneId::Abdomen,   -1.5),
        ], 0);
        let sym = SymmetryAnalysis::compute(&scan);
        assert!(sym.torso_arm_inversion > 0.0,
            "torso_arm_inversion={}", sym.torso_arm_inversion);
    }

    #[test]
    fn normal_scan_has_no_inversion() {
        // Torso warmer than arms (normal physiology)
        let scan = BodyScan::from_deviations(BodyType::Man, &[
            (ZoneId::Chest,     1.0),
            (ZoneId::Abdomen,   1.0),
            (ZoneId::LUpperArm, 0.0),
            (ZoneId::RUpperArm, 0.0),
        ], 0);
        let sym = SymmetryAnalysis::compute(&scan);
        assert!(sym.torso_arm_inversion < 1e-5, "inversion={}", sym.torso_arm_inversion);
    }

    #[test]
    fn fever_pattern_gives_positive_head_torso_gradient() {
        // Head deviation +2.2, Chest/Abdomen +0.5
        // Head temp = 36.8+2.2=39.0; torso_mean = (36.6+0.5 + 36.4+0.5)/2 = 37.0
        // gradient = 39.0 - 37.0 = 2.0 > 1.5 ✓
        let scan = BodyScan::from_deviations(BodyType::Man, &[
            (ZoneId::Head,    2.2),
            (ZoneId::Chest,   0.5),
            (ZoneId::Abdomen, 0.5),
        ], 0);
        let sym = SymmetryAnalysis::compute(&scan);
        assert!(sym.head_torso_gradient > 1.5,
            "gradient={}", sym.head_torso_gradient);
        assert!(!sym.notes.is_empty(), "fever note should be present");
    }

    #[test]
    fn shoulder_asymmetry_note_generated() {
        let scan = BodyScan::from_deviations(BodyType::Man, &[(ZoneId::LShoulderZ, -1.5)], 0);
        let sym  = SymmetryAnalysis::compute(&scan);
        // shoulder_diff = 1.5 > 1.0 → note added
        assert!(sym.notes.iter().any(|n| n.contains("Shoulder")));
    }
}
