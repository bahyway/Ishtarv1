//! Hardware health integration — feeds Shedu telemetry into the supervisor.
//!
//! The supervisor exposes a HardwareHealthReport per node so that the
//! Dubsar IDE can show the node's B11 quality byte alongside the particle
//! quality — treating the machine as a first-class sovereign particle.

/// Hardware health report produced by the supervisor for one node.
///
/// Populated by Shedu telemetry; consumed by Dubsar IDE and riksu-engine.
#[derive(Debug, Clone)]
pub struct HardwareHealthReport {
    /// Node identifier.
    pub node_id: u8,
    /// Hardware B11 quality byte (0..240), same scale as particle B11.
    pub b11: u8,
    /// D7 hardware integrity score (0.0..1.0).
    pub d7_integrity: f32,
    /// True when B11 < 60 — freeze should fire immediately.
    pub imminent_failure: bool,
    /// True when B11 < 100 — stakeholder warning should be shown.
    pub warning: bool,
    /// Human-readable node health label.
    pub label: &'static str,
}

impl HardwareHealthReport {
    /// Construct from raw D7 integrity (e.g. from shedu-engine).
    pub fn from_d7(node_id: u8, d7_integrity: f32) -> Self {
        let b11 = (d7_integrity * 240.0).round().clamp(0.0, 240.0) as u8;
        let imminent_failure = b11 < 60;
        let warning = b11 < 100 && !imminent_failure;
        let label = if b11 >= 200 {
            "GEM-NODE"
        } else if b11 >= 140 {
            "TRIBE-NODE"
        } else if b11 >= 100 {
            "ACTIVE-NODE"
        } else if b11 >= 60 {
            "FUZZY-NODE"
        } else {
            "DEAD-NODE"
        };
        Self {
            node_id,
            b11,
            d7_integrity,
            imminent_failure,
            warning,
            label,
        }
    }

    /// Construct a healthy report (for testing / uninitialised state).
    pub fn healthy(node_id: u8) -> Self {
        Self::from_d7(node_id, 0.95)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn healthy_node_report() {
        let r = HardwareHealthReport::healthy(0);
        assert!(
            r.b11 >= 200,
            "healthy should be GEM-NODE, got b11={}",
            r.b11
        );
        assert!(!r.imminent_failure);
        assert!(!r.warning);
        assert_eq!(r.label, "GEM-NODE");
    }

    #[test]
    fn dead_node_report() {
        let r = HardwareHealthReport::from_d7(1, 0.10);
        assert!(r.b11 < 60);
        assert!(r.imminent_failure);
        assert_eq!(r.label, "DEAD-NODE");
    }

    #[test]
    fn fuzzy_node_report() {
        let r = HardwareHealthReport::from_d7(2, 0.30);
        assert!(r.b11 >= 60 && r.b11 < 100);
        assert!(r.warning);
        assert!(!r.imminent_failure);
    }

    #[test]
    fn b11_proportional_to_d7() {
        let r = HardwareHealthReport::from_d7(0, 0.5);
        let expected = (0.5f32 * 240.0).round() as u8;
        assert_eq!(r.b11, expected);
    }

    #[test]
    fn d7_zero_is_dead() {
        let r = HardwareHealthReport::from_d7(0, 0.0);
        assert_eq!(r.b11, 0);
        assert!(r.imminent_failure);
    }
}
