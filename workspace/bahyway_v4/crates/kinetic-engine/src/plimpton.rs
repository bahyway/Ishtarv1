//! # Plimpton — Babylonian Rational Trigonometry Boundaries
//!
//! Maps the 15 rows of the Plimpton 322 tablet to sovereign tier transition
//! boundaries in 7D Heptagon space.
//!
//! Spread values are computed as (short_leg / hypotenuse)² for each
//! Pythagorean triple in the tablet — the rational equivalent of sin²θ
//! from Wildberger's rational trigonometry.
//!
//! The 7 Heptagon dimensions map to odd-indexed rows (1, 3, 5, 7, 9, 11, 13),
//! with Row 11 (the 3-4-5 triple, spread = 0.36) defining canonical health.

// ── PlimptonAnchors ─────────────────────────────────────────────────────────

/// The 7 sovereign spread values from Plimpton 322 — one per Heptagon dimension.
///
/// Each value is (short_leg / hypotenuse)² for the corresponding tablet row.
/// These are the geometric boundary positions in 7D space below which
/// a particle is considered to have entered a lower tier.
///
/// | Row | Dimension | Triple        | Spread  |
/// |-----|-----------|---------------|---------|
/// |   1 | ME        | (119,120,169) | ≈ 0.496 |
/// |   3 | GU        | (4601,4800,6649)| ≈ 0.479|
/// |   5 | SAG       | (65,72,97)    | ≈ 0.449 |
/// |   7 | A         | (2291,2700,3541)| ≈ 0.418|
/// |   9 | IZI       | (481,600,769) | ≈ 0.391 |
/// |  11 | UD        | (3,4,5)       |  = 0.360|
/// |  13 | URU       | (161,240,289) | ≈ 0.310 |
#[derive(Debug, Clone, Copy)]
pub struct PlimptonAnchors {
    pub me:  f64,
    pub gu:  f64,
    pub sag: f64,
    pub a:   f64,
    pub izi: f64,
    pub ud:  f64,
    pub uru: f64,
}

impl PlimptonAnchors {
    /// The sovereign set — all 7 dimensional anchors from Plimpton 322.
    pub fn sovereign() -> Self {
        Self {
            me:  (119.0_f64 / 169.0_f64).powi(2), // Row  1: (119,120,169) ≈ 0.4959
            gu:  (4601.0_f64 / 6649.0_f64).powi(2), // Row  3: (4601,4800,6649) ≈ 0.4786
            sag: (65.0_f64  / 97.0_f64).powi(2),  // Row  5: (65,72,97) ≈ 0.4490
            a:   (2291.0_f64 / 3541.0_f64).powi(2), // Row  7: (2291,2700,3541) ≈ 0.4183
            izi: (481.0_f64 / 769.0_f64).powi(2), // Row  9: (481,600,769) ≈ 0.3909
            ud:  (3.0_f64   / 5.0_f64).powi(2),   // Row 11: (3,4,5) canonical = 0.3600
            uru: (161.0_f64 / 289.0_f64).powi(2), // Row 13: (161,240,289) ≈ 0.3096
        }
    }
}

// ── PlimptonTrigger ─────────────────────────────────────────────────────────

/// A rule that triggers when a particle's orbital spread
/// crosses a Plimpton 322 boundary.
///
/// This replaces arbitrary threshold comparisons with exact
/// geometric positions from Babylonian rational trigonometry.
pub struct PlimptonTrigger {
    /// Which Plimpton row defines this trigger boundary
    pub row: usize,        // 1-15
    /// The exact spread value for this row
    pub spread: f64,
    /// What fires when the particle crosses this boundary
    pub action: TriggerAction,
}

#[derive(Debug, Clone)]
pub enum TriggerAction {
    /// Particle entered this Plimpton tier — log the transition
    TierEntry,
    /// Particle crossed the sovereign boundary — emit .akk file
    EmitAkkadianFix(String),
    /// Particle is approaching dead zone — trigger Self-Immune
    SelfImmuneAlert,
    /// Particle reached the Row 11 (3,4,5) tier — canonical health level
    CanonicalHealthReached,
}

impl PlimptonTrigger {
    /// Check if a particle's spread has crossed this trigger's boundary.
    /// Returns true if crossing occurred between last_spread and current_spread.
    pub fn crossed(&self, last_spread: f64, current_spread: f64) -> bool {
        (last_spread - self.spread) * (current_spread - self.spread) < 0.0
    }

    /// The 7 sovereign triggers — one per Heptagon dimension anchor
    pub fn sovereign_set() -> [PlimptonTrigger; 7] {
        let anchors = PlimptonAnchors::sovereign();
        [
            PlimptonTrigger { row: 1,  spread: anchors.me,  action: TriggerAction::TierEntry },
            PlimptonTrigger { row: 3,  spread: anchors.gu,  action: TriggerAction::TierEntry },
            PlimptonTrigger { row: 5,  spread: anchors.sag, action: TriggerAction::TierEntry },
            PlimptonTrigger { row: 7,  spread: anchors.a,   action: TriggerAction::TierEntry },
            PlimptonTrigger { row: 9,  spread: anchors.izi, action: TriggerAction::SelfImmuneAlert },
            PlimptonTrigger { row: 11, spread: anchors.ud,  action: TriggerAction::CanonicalHealthReached },
            PlimptonTrigger { row: 13, spread: anchors.uru, action: TriggerAction::EmitAkkadianFix("index_rebuild.akk".to_string()) },
        ]
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sovereign_anchors_row_11_ud_is_exactly_0_36() {
        let anchors = PlimptonAnchors::sovereign();
        let expected = (3.0_f64 / 5.0).powi(2); // 0.36 exactly
        assert!((anchors.ud - expected).abs() < f64::EPSILON,
            "Row 11 (3,4,5) spread must be exactly 0.36: {}", anchors.ud);
    }

    #[test]
    fn sovereign_anchors_are_strictly_decreasing() {
        let a = PlimptonAnchors::sovereign();
        let spreads = [a.me, a.gu, a.sag, a.a, a.izi, a.ud, a.uru];
        for i in 1..spreads.len() {
            assert!(spreads[i] < spreads[i - 1],
                "Plimpton spreads must decrease: row[{}]={} >= row[{}]={}",
                i, spreads[i], i - 1, spreads[i - 1]);
        }
    }

    #[test]
    fn sovereign_set_has_seven_triggers() {
        let triggers = PlimptonTrigger::sovereign_set();
        assert_eq!(triggers.len(), 7);
    }

    #[test]
    fn trigger_crossed_detects_downward_crossing() {
        let trigger = PlimptonTrigger {
            row: 11, spread: 0.36, action: TriggerAction::CanonicalHealthReached,
        };
        // Crossing from above 0.36 to below 0.36
        assert!(trigger.crossed(0.40, 0.30), "should detect downward crossing");
    }

    #[test]
    fn trigger_crossed_detects_upward_crossing() {
        let trigger = PlimptonTrigger {
            row: 11, spread: 0.36, action: TriggerAction::CanonicalHealthReached,
        };
        // Crossing from below 0.36 to above 0.36
        assert!(trigger.crossed(0.30, 0.40), "should detect upward crossing");
    }

    #[test]
    fn trigger_not_crossed_when_same_side() {
        let trigger = PlimptonTrigger {
            row: 11, spread: 0.36, action: TriggerAction::CanonicalHealthReached,
        };
        assert!(!trigger.crossed(0.40, 0.50), "should not trigger when both above");
        assert!(!trigger.crossed(0.20, 0.30), "should not trigger when both below");
    }
}
