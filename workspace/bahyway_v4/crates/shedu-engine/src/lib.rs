//! 𒀭𒅆𒇻 shedu-engine — Sovereign Hardware Anomaly Detection v4.0.1
//!
//! Shedu (𒀭𒅆𒇻) — Akkadian protective spirit. The guardian that watches
//! the machine itself, treating the node as a particle with its own B11
//! quality score and triggering the Quantum Freeze when it fails.
//!
//! Architecture:
//!   hardware    — HardwareMetrics: raw GPU/VRAM/CPU/NET readings
//!   node_health — NodeHealth: D7 integrity → B11 hardware quality byte
//!   telemetry   — TelemetryFeed: Event-KAKI equivalent for hardware events
//!   freeze      — Quantum Freeze protocol: sealed FreezeResult + KAKI stamp
//!
//! The node's hardware quality is scored identically to a data particle:
//!   B11_hw = round(D7_integrity × 240)
//!   GEM-NODE ≥ 200 · TRIBE-NODE ≥ 140 · ACTIVE-NODE ≥ 100
//!   FUZZY-NODE ≥ 60 · DEAD-NODE < 60 → Quantum Freeze fires
//!
//! "The Scribe records the final snapshot. Build with Sovereignty." 𒁾
//!
//! DUB.SAR 𒁾 — BahyWay.Ecosystem v4.0.1 | Pure Rust

#![forbid(unsafe_code)]

pub mod hardware;
pub mod node_health;
pub mod telemetry;
pub mod freeze;

pub use hardware::{
    HardwareMetrics,
    GPU_TEMP_WARN, GPU_TEMP_CRITICAL,
    VRAM_WARN_PCT, VRAM_CRITICAL_PCT,
    CPU_WARN_PCT, CPU_CRITICAL_PCT,
    NET_LATENCY_WARN_MS, NET_LATENCY_CRITICAL_MS,
};
pub use node_health::{
    NodeHealth, compute_d7_integrity,
    D7_W_GPU, D7_W_VRAM, D7_W_CPU, D7_W_NET,
    NODE_DEAD_B11, NODE_FUZZY_B11,
};
pub use telemetry::{TelemetryEvent, TelemetryFeed};
pub use freeze::{FreezeReason, FreezeResult, trigger_quantum_freeze, should_freeze};

/// Assess all nodes in the cluster and return any that require a freeze.
///
/// `node_healths` — current health snapshot for each node
///
/// Returns a list of `(NodeHealth, FreezeReason)` for all nodes that should freeze.
pub fn assess_cluster(node_healths: &[NodeHealth]) -> Vec<(&NodeHealth, FreezeReason)> {
    let total_nodes = node_healths.len();
    let alive: Vec<&NodeHealth> = node_healths.iter()
        .filter(|h| !h.is_imminent_failure())
        .collect();
    let is_last = alive.len() <= 1;

    node_healths.iter()
        .filter_map(|h| {
            let last = is_last && alive.first().map(|a| a.node_id) == Some(h.node_id);
            should_freeze(h, last).map(|r| (h, r))
        })
        .collect()
}

/// Harmony coefficient across all nodes — how uniformly healthy the cluster is.
///
/// Returns 1.0 for a perfectly uniform cluster, lower for high variance.
pub fn cluster_harmony(node_healths: &[NodeHealth]) -> f32 {
    if node_healths.is_empty() { return 0.0; }
    let mean = node_healths.iter().map(|h| h.b11 as f32).sum::<f32>()
        / node_healths.len() as f32;
    let variance = node_healths.iter()
        .map(|h| (h.b11 as f32 - mean).powi(2))
        .sum::<f32>()
        / node_healths.len() as f32;
    1.0 / (1.0 + variance.sqrt() / 240.0)
}

// ── Integration Tests ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn healthy_node(id: u8) -> NodeHealth {
        NodeHealth::from_metrics(id, HardwareMetrics::simulated_healthy(0))
    }
    fn critical_node(id: u8) -> NodeHealth {
        NodeHealth::from_metrics(id, HardwareMetrics::simulated_critical(0))
    }

    #[test]
    fn healthy_cluster_no_freeze() {
        let cluster = vec![healthy_node(0), healthy_node(1), healthy_node(2)];
        let freezes = assess_cluster(&cluster);
        assert!(freezes.is_empty(), "healthy cluster should have no freezes");
    }

    #[test]
    fn one_critical_node_triggers_freeze() {
        let cluster = vec![healthy_node(0), critical_node(1), healthy_node(2)];
        let freezes = assess_cluster(&cluster);
        assert!(!freezes.is_empty(), "critical node should trigger freeze");
    }

    #[test]
    fn cluster_harmony_uniform_is_high() {
        let cluster = vec![healthy_node(0), healthy_node(1), healthy_node(2)];
        let h = cluster_harmony(&cluster);
        assert!(h > 0.9, "uniform healthy cluster should have harmony > 0.9, got {h}");
    }

    #[test]
    fn cluster_harmony_mixed_is_lower() {
        let cluster = vec![healthy_node(0), critical_node(1)];
        let h = cluster_harmony(&cluster);
        assert!(h < 0.9, "mixed cluster should have lower harmony, got {h}");
    }

    #[test]
    fn empty_cluster_harmony_zero() {
        assert_eq!(cluster_harmony(&[]), 0.0);
    }

    #[test]
    fn full_pipeline_healthy_to_freeze() {
        let m     = HardwareMetrics::simulated_critical(9_000);
        let health = NodeHealth::from_metrics(0, m.clone());
        let mut feed = TelemetryFeed::new();
        feed.ingest(0, &m);
        assert!(feed.has_critical());
        let reason = should_freeze(&health, true).unwrap();
        let result = trigger_quantum_freeze(&health, reason, 9_000);
        assert_eq!(result.freeze_kaki[6], 0x02);
        assert!(!result.stakeholder_message.is_empty());
    }
}
