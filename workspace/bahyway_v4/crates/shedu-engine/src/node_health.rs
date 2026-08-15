//! Node health scoring — applies H(P) philosophy to hardware.
//!
//! The node itself is treated as a particle. Its D7 integrity score is the
//! H(P) equivalent for hardware, computed from four physical dimensions:
//!
//!   D_GPU   weight 0.35  — GPU thermal headroom
//!   D_VRAM  weight 0.30  — VRAM availability
//!   D_CPU   weight 0.20  — CPU headroom
//!   D_NET   weight 0.15  — network latency headroom
//!
//! B11_hw = round(D7_integrity × 240)
//! Same QualityLane thresholds as particle scoring apply to the node.

use crate::hardware::HardwareMetrics;

/// D7 dimension weights for hardware health (sum = 1.0).
pub const D7_W_GPU:  f32 = 0.35;
pub const D7_W_VRAM: f32 = 0.30;
pub const D7_W_CPU:  f32 = 0.20;
pub const D7_W_NET:  f32 = 0.15;

/// Below this B11 value the node is considered DEAD → freeze imminent.
pub const NODE_DEAD_B11: u8 = 60;

/// Below this B11 the freeze warning should be issued (FUZZY lane).
pub const NODE_FUZZY_B11: u8 = 100;

/// Health score for a single node.
#[derive(Debug, Clone)]
pub struct NodeHealth {
    /// Numeric node identifier (0-based index in cluster).
    pub node_id: u8,
    /// Hardware quality byte — same scale as particle B11 (0..240).
    pub b11: u8,
    /// Raw D7 integrity score (0.0..1.0).
    pub d7_integrity: f32,
    /// The raw hardware metrics this was derived from.
    pub metrics: HardwareMetrics,
}

impl NodeHealth {
    /// Derive node health from raw hardware metrics.
    pub fn from_metrics(node_id: u8, metrics: HardwareMetrics) -> Self {
        let d7 = compute_d7_integrity(&metrics);
        let b11 = (d7 * 240.0).round() as u8;
        Self { node_id, b11, d7_integrity: d7, metrics }
    }

    /// Node is in DEAD lane — Quantum Freeze should fire immediately.
    pub fn is_imminent_failure(&self) -> bool { self.b11 < NODE_DEAD_B11 }

    /// Node is in FUZZY lane — issue warning to stakeholder.
    pub fn is_warning(&self) -> bool {
        self.b11 >= NODE_DEAD_B11 && self.b11 < NODE_FUZZY_B11
    }

    /// Node is healthy (GEM/TRIBE/ACTIVE).
    pub fn is_healthy(&self) -> bool { self.b11 >= NODE_FUZZY_B11 }

    pub fn label(&self) -> &'static str {
        if self.b11 >= 200      { "GEM-NODE" }
        else if self.b11 >= 140 { "TRIBE-NODE" }
        else if self.b11 >= 100 { "ACTIVE-NODE" }
        else if self.b11 >= 60  { "FUZZY-NODE" }
        else                    { "DEAD-NODE" }
    }
}

/// Compute the composite D7 hardware integrity score (0..1).
pub fn compute_d7_integrity(m: &HardwareMetrics) -> f32 {
    // Each sub-score maps the raw metric to [0,1] headroom remaining
    let gpu_score  = gpu_headroom(m.gpu_temp_celsius);
    let vram_score = 1.0 - m.vram_utilisation();
    let cpu_score  = (1.0 - m.cpu_load).clamp(0.0, 1.0);
    let net_score  = net_headroom(m.net_latency_ms);

    (D7_W_GPU  * gpu_score
   + D7_W_VRAM * vram_score
   + D7_W_CPU  * cpu_score
   + D7_W_NET  * net_score)
        .clamp(0.0, 1.0)
}

/// GPU thermal headroom: 1.0 at 0°C, 0.0 at 100°C.
fn gpu_headroom(temp: f32) -> f32 {
    (1.0 - (temp / 100.0)).clamp(0.0, 1.0)
}

/// Network headroom: 1.0 at 0ms, 0.0 at 500ms+.
fn net_headroom(latency_ms: f32) -> f32 {
    (1.0 - (latency_ms / 500.0)).clamp(0.0, 1.0)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn healthy_node_high_b11() {
        let m = HardwareMetrics::simulated_healthy(0);
        let h = NodeHealth::from_metrics(0, m);
        assert!(h.b11 >= NODE_FUZZY_B11, "healthy node should be above fuzzy: b11={}", h.b11);
        assert!(h.is_healthy());
        assert!(!h.is_warning());
        assert!(!h.is_imminent_failure());
    }

    #[test]
    fn critical_node_low_b11() {
        let m = HardwareMetrics::simulated_critical(0);
        let h = NodeHealth::from_metrics(0, m);
        assert!(h.b11 < NODE_FUZZY_B11, "critical node should be below fuzzy: b11={}", h.b11);
        assert!(h.is_imminent_failure() || h.is_warning());
    }

    #[test]
    fn d7_integrity_bounded() {
        let m = HardwareMetrics::simulated_healthy(0);
        let d7 = compute_d7_integrity(&m);
        assert!(d7 >= 0.0 && d7 <= 1.0, "D7 must be in [0,1], got {d7}");
    }

    #[test]
    fn zero_latency_full_net_score() {
        let m = HardwareMetrics { net_latency_ms: 0.0, ..HardwareMetrics::simulated_healthy(0) };
        let d7_low_net  = compute_d7_integrity(&m);
        let m2 = HardwareMetrics { net_latency_ms: 400.0, ..HardwareMetrics::simulated_healthy(0) };
        let d7_high_net = compute_d7_integrity(&m2);
        assert!(d7_low_net > d7_high_net, "lower latency should give higher D7");
    }

    #[test]
    fn node_label_dead() {
        let m = HardwareMetrics::simulated_critical(0);
        let h = NodeHealth::from_metrics(0, m);
        assert!(h.label() == "DEAD-NODE" || h.label() == "FUZZY-NODE");
    }

    #[test]
    fn b11_proportional_to_d7() {
        let m = HardwareMetrics::simulated_healthy(0);
        let h = NodeHealth::from_metrics(0, m);
        let expected = (h.d7_integrity * 240.0).round() as u8;
        assert_eq!(h.b11, expected);
    }

    #[test]
    fn weights_sum_to_one() {
        let sum = D7_W_GPU + D7_W_VRAM + D7_W_CPU + D7_W_NET;
        assert!((sum - 1.0).abs() < 1e-5, "weights must sum to 1.0, got {sum}");
    }
}
