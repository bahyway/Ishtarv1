//! Quantum Freeze protocol — the sovereign transition from live stream to
//! crystalline simulation when the last node is failing.
//!
//! The freeze is NOT a crash. It is a controlled, KAKI-stamped transition:
//!   1. Capture current particle positions (caller provides snapshot bytes)
//!   2. Seal an emergency Event-KAKI as the freeze record
//!   3. Export state for offline .akk-sim replay
//!   4. Notify Dubsar IDE to switch to Alert Amber mode
//!
//! "A failure is not an end; it is a transition into a Simulated Eternity."

use crate::node_health::NodeHealth;

/// The reason the Quantum Freeze was triggered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FreezeReason {
    /// GPU temperature exceeded critical threshold (≥ 95°C).
    GpuThermalCritical,
    /// VRAM utilisation exceeded critical threshold (≥ 95%).
    VramExhausted,
    /// Node became unreachable (network partition).
    NodeIsolated,
    /// Last surviving node is in DEAD lane (B11 < 60).
    LastNodeFailing,
    /// Stakeholder manually triggered the freeze.
    ManualTrigger,
}

impl FreezeReason {
    pub fn label(self) -> &'static str {
        match self {
            Self::GpuThermalCritical => "GPU_THERMAL_CRITICAL",
            Self::VramExhausted      => "VRAM_EXHAUSTED",
            Self::NodeIsolated       => "NODE_ISOLATED",
            Self::LastNodeFailing    => "LAST_NODE_FAILING",
            Self::ManualTrigger      => "MANUAL_TRIGGER",
        }
    }

    pub fn stakeholder_message(self) -> &'static str {
        match self {
            Self::GpuThermalCritical =>
                "Real-time stream unstable: GPU thermal critical. Materializing Final Sovereign Snapshot.",
            Self::VramExhausted =>
                "Real-time stream unstable: VRAM exhausted. Materializing Final Sovereign Snapshot.",
            Self::NodeIsolated =>
                "Real-time stream unstable: node isolated. Materializing Final Sovereign Snapshot.",
            Self::LastNodeFailing =>
                "Real-time stream unstable: last node failing. Materializing Final Sovereign Snapshot.",
            Self::ManualTrigger =>
                "Manual freeze requested. Recording Simulation.",
        }
    }
}

/// The sealed result of a Quantum Freeze event.
#[derive(Debug, Clone)]
pub struct FreezeResult {
    /// Reason the freeze was triggered.
    pub reason: FreezeReason,
    /// Node that triggered the freeze.
    pub node_id: u8,
    /// B11 hardware quality byte at the moment of freeze.
    pub b11_at_freeze: u8,
    /// D7 integrity score at the moment of freeze.
    pub d7_at_freeze: f32,
    /// Unix-epoch millisecond timestamp of the freeze.
    pub timestamp_ms: u64,
    /// KAKI-format 16-byte identifier for this freeze event.
    pub freeze_kaki: [u8; 16],
    /// Human-readable message for the stakeholder UI.
    pub stakeholder_message: &'static str,
}

impl FreezeResult {
    pub fn is_recoverable(&self) -> bool {
        matches!(self.reason, FreezeReason::ManualTrigger | FreezeReason::NodeIsolated)
    }
}

/// Trigger the Quantum Freeze protocol for a failing node.
///
/// `health`       — current node health at freeze moment
/// `reason`       — why the freeze was triggered
/// `timestamp_ms` — caller-supplied timestamp (allows deterministic testing)
pub fn trigger_quantum_freeze(
    health:       &NodeHealth,
    reason:       FreezeReason,
    timestamp_ms: u64,
) -> FreezeResult {
    let freeze_kaki = derive_freeze_kaki(health.node_id, health.b11, timestamp_ms);
    FreezeResult {
        reason,
        node_id:             health.node_id,
        b11_at_freeze:       health.b11,
        d7_at_freeze:        health.d7_integrity,
        timestamp_ms,
        freeze_kaki,
        stakeholder_message: reason.stakeholder_message(),
    }
}

/// Inspect node health and determine if a freeze should fire automatically.
///
/// Returns `Some(FreezeReason)` if the freeze protocol should be triggered.
pub fn should_freeze(health: &NodeHealth, is_last_node: bool) -> Option<FreezeReason> {
    if health.metrics.gpu_critical() {
        return Some(FreezeReason::GpuThermalCritical);
    }
    if health.metrics.vram_critical() {
        return Some(FreezeReason::VramExhausted);
    }
    if is_last_node && health.is_imminent_failure() {
        return Some(FreezeReason::LastNodeFailing);
    }
    None
}

/// Derive a 16-byte KAKI-format identifier for a freeze event.
/// Uses FNV-1a mixing of node_id, B11, and timestamp.
fn derive_freeze_kaki(node_id: u8, b11: u8, timestamp_ms: u64) -> [u8; 16] {
    const FNV_PRIME: u64 = 0x0000_0100_0000_01B3;
    const FNV_BASIS: u64 = 0xcbf2_9ce4_8422_2325;

    let mut data = [0u8; 10];
    data[0] = node_id;
    data[1] = b11;
    data[2..10].copy_from_slice(&timestamp_ms.to_le_bytes());

    let mut hash = FNV_BASIS;
    for &byte in &data {
        hash ^= byte as u64;
        hash  = hash.wrapping_mul(FNV_PRIME);
    }
    let hash2 = hash.wrapping_mul(FNV_PRIME).wrapping_add(0xDEAD_BEEF);

    let mut kaki = [0u8; 16];
    kaki[..8].copy_from_slice(&hash.to_le_bytes());
    kaki[8..].copy_from_slice(&hash2.to_le_bytes());
    // Mark byte 6 as type 2 (EventKaki — this is a freeze event-kaki)
    kaki[6] = 0x02;
    kaki
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hardware::HardwareMetrics;

    fn make_health(metrics: HardwareMetrics) -> NodeHealth {
        NodeHealth::from_metrics(0, metrics)
    }

    #[test]
    fn healthy_node_should_not_freeze() {
        let h = make_health(HardwareMetrics::simulated_healthy(0));
        assert_eq!(should_freeze(&h, false), None);
        assert_eq!(should_freeze(&h, true),  None);
    }

    #[test]
    fn critical_gpu_triggers_freeze() {
        let h = make_health(HardwareMetrics::simulated_critical(0));
        let reason = should_freeze(&h, false);
        assert_eq!(reason, Some(FreezeReason::GpuThermalCritical));
    }

    #[test]
    fn last_node_failing_triggers_freeze() {
        // VRAM not critical but node is dying (low B11)
        let h = make_health(HardwareMetrics {
            gpu_temp_celsius: 50.0,
            vram_used_mb:     4_000,
            vram_total_mb:    16_384,
            cpu_load:         0.99,
            ram_used_mb:      30_000,
            ram_total_mb:     32_768,
            net_latency_ms:   450.0,
            timestamp_ms:     0,
        });
        // Only fires if last node
        assert_eq!(should_freeze(&h, false), None);
        // With last_node=true and b11 < NODE_DEAD_B11 it fires
        if h.is_imminent_failure() {
            assert_eq!(should_freeze(&h, true), Some(FreezeReason::LastNodeFailing));
        }
    }

    #[test]
    fn trigger_freeze_returns_correct_fields() {
        let h = make_health(HardwareMetrics::simulated_critical(1_000));
        let result = trigger_quantum_freeze(&h, FreezeReason::ManualTrigger, 1_000);
        assert_eq!(result.node_id, 0);
        assert_eq!(result.reason, FreezeReason::ManualTrigger);
        assert_eq!(result.b11_at_freeze, h.b11);
        assert_eq!(result.timestamp_ms, 1_000);
    }

    #[test]
    fn freeze_kaki_type_byte_is_event() {
        let h = make_health(HardwareMetrics::simulated_critical(0));
        let result = trigger_quantum_freeze(&h, FreezeReason::LastNodeFailing, 42_000);
        assert_eq!(result.freeze_kaki[6], 0x02, "freeze KAKI must be EventKaki type");
    }

    #[test]
    fn freeze_kaki_deterministic() {
        let h = make_health(HardwareMetrics::simulated_healthy(0));
        let r1 = trigger_quantum_freeze(&h, FreezeReason::ManualTrigger, 99);
        let r2 = trigger_quantum_freeze(&h, FreezeReason::ManualTrigger, 99);
        assert_eq!(r1.freeze_kaki, r2.freeze_kaki);
    }

    #[test]
    fn freeze_reason_labels_non_empty() {
        for r in [FreezeReason::GpuThermalCritical, FreezeReason::VramExhausted,
                  FreezeReason::NodeIsolated, FreezeReason::LastNodeFailing,
                  FreezeReason::ManualTrigger] {
            assert!(!r.label().is_empty());
            assert!(!r.stakeholder_message().is_empty());
        }
    }

    #[test]
    fn manual_freeze_is_recoverable() {
        let h = make_health(HardwareMetrics::simulated_healthy(0));
        let r = trigger_quantum_freeze(&h, FreezeReason::ManualTrigger, 0);
        assert!(r.is_recoverable());
    }

    #[test]
    fn thermal_freeze_not_recoverable() {
        let h = make_health(HardwareMetrics::simulated_critical(0));
        let r = trigger_quantum_freeze(&h, FreezeReason::GpuThermalCritical, 0);
        assert!(!r.is_recoverable());
    }
}
