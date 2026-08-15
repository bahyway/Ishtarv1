//! Telemetry events — the Event-KAKI equivalent for hardware state changes.
//!
//! Each TelemetryEvent is emitted when a hardware dimension crosses a threshold.
//! The TelemetryFeed accumulates events chronologically and drives the freeze decision.

use crate::hardware::{
    GPU_TEMP_WARN, GPU_TEMP_CRITICAL,
    VRAM_WARN_PCT, VRAM_CRITICAL_PCT,
    CPU_WARN_PCT, CPU_CRITICAL_PCT,
    NET_LATENCY_WARN_MS, NET_LATENCY_CRITICAL_MS,
};
use crate::hardware::HardwareMetrics;

/// A hardware threshold crossing event.
#[derive(Debug, Clone, PartialEq)]
pub enum TelemetryEvent {
    GpuTempWarn     { node_id: u8, temp_celsius: f32, timestamp_ms: u64 },
    GpuTempCritical { node_id: u8, temp_celsius: f32, timestamp_ms: u64 },
    VramWarn        { node_id: u8, utilisation: f32,  timestamp_ms: u64 },
    VramCritical    { node_id: u8, utilisation: f32,  timestamp_ms: u64 },
    CpuWarn         { node_id: u8, load: f32,         timestamp_ms: u64 },
    CpuCritical     { node_id: u8, load: f32,         timestamp_ms: u64 },
    NetWarn         { node_id: u8, latency_ms: f32,   timestamp_ms: u64 },
    NetCritical     { node_id: u8, latency_ms: f32,   timestamp_ms: u64 },
    NodeDown        { node_id: u8, timestamp_ms: u64 },
    LastNodeFailing { node_id: u8, b11: u8,           timestamp_ms: u64 },
}

impl TelemetryEvent {
    pub fn node_id(&self) -> u8 {
        match self {
            Self::GpuTempWarn     { node_id, .. } => *node_id,
            Self::GpuTempCritical { node_id, .. } => *node_id,
            Self::VramWarn        { node_id, .. } => *node_id,
            Self::VramCritical    { node_id, .. } => *node_id,
            Self::CpuWarn         { node_id, .. } => *node_id,
            Self::CpuCritical     { node_id, .. } => *node_id,
            Self::NetWarn         { node_id, .. } => *node_id,
            Self::NetCritical     { node_id, .. } => *node_id,
            Self::NodeDown        { node_id, .. } => *node_id,
            Self::LastNodeFailing { node_id, .. } => *node_id,
        }
    }

    pub fn is_critical(&self) -> bool {
        matches!(self,
            Self::GpuTempCritical { .. } |
            Self::VramCritical    { .. } |
            Self::CpuCritical     { .. } |
            Self::NetCritical     { .. } |
            Self::NodeDown        { .. } |
            Self::LastNodeFailing { .. }
        )
    }

    pub fn timestamp_ms(&self) -> u64 {
        match self {
            Self::GpuTempWarn     { timestamp_ms, .. } => *timestamp_ms,
            Self::GpuTempCritical { timestamp_ms, .. } => *timestamp_ms,
            Self::VramWarn        { timestamp_ms, .. } => *timestamp_ms,
            Self::VramCritical    { timestamp_ms, .. } => *timestamp_ms,
            Self::CpuWarn         { timestamp_ms, .. } => *timestamp_ms,
            Self::CpuCritical     { timestamp_ms, .. } => *timestamp_ms,
            Self::NetWarn         { timestamp_ms, .. } => *timestamp_ms,
            Self::NetCritical     { timestamp_ms, .. } => *timestamp_ms,
            Self::NodeDown        { timestamp_ms, .. } => *timestamp_ms,
            Self::LastNodeFailing { timestamp_ms, .. } => *timestamp_ms,
        }
    }
}

/// Chronological feed of telemetry events across the cluster.
#[derive(Debug, Default)]
pub struct TelemetryFeed {
    pub events: Vec<TelemetryEvent>,
}

impl TelemetryFeed {
    pub fn new() -> Self { Self::default() }

    /// Ingest hardware metrics and emit any threshold-crossing events.
    pub fn ingest(&mut self, node_id: u8, m: &HardwareMetrics) {
        let t = m.timestamp_ms;

        if m.gpu_temp_celsius >= GPU_TEMP_CRITICAL {
            self.events.push(TelemetryEvent::GpuTempCritical {
                node_id, temp_celsius: m.gpu_temp_celsius, timestamp_ms: t });
        } else if m.gpu_temp_celsius >= GPU_TEMP_WARN {
            self.events.push(TelemetryEvent::GpuTempWarn {
                node_id, temp_celsius: m.gpu_temp_celsius, timestamp_ms: t });
        }

        let vram_util = m.vram_utilisation();
        if vram_util >= VRAM_CRITICAL_PCT {
            self.events.push(TelemetryEvent::VramCritical {
                node_id, utilisation: vram_util, timestamp_ms: t });
        } else if vram_util >= VRAM_WARN_PCT {
            self.events.push(TelemetryEvent::VramWarn {
                node_id, utilisation: vram_util, timestamp_ms: t });
        }

        if m.cpu_load >= CPU_CRITICAL_PCT {
            self.events.push(TelemetryEvent::CpuCritical {
                node_id, load: m.cpu_load, timestamp_ms: t });
        } else if m.cpu_load >= CPU_WARN_PCT {
            self.events.push(TelemetryEvent::CpuWarn {
                node_id, load: m.cpu_load, timestamp_ms: t });
        }

        if m.net_latency_ms >= NET_LATENCY_CRITICAL_MS {
            self.events.push(TelemetryEvent::NetCritical {
                node_id, latency_ms: m.net_latency_ms, timestamp_ms: t });
        } else if m.net_latency_ms >= NET_LATENCY_WARN_MS {
            self.events.push(TelemetryEvent::NetWarn {
                node_id, latency_ms: m.net_latency_ms, timestamp_ms: t });
        }
    }

    /// Emit a NodeDown event (called when a node becomes unreachable).
    pub fn node_down(&mut self, node_id: u8, timestamp_ms: u64) {
        self.events.push(TelemetryEvent::NodeDown { node_id, timestamp_ms });
    }

    /// Emit a LastNodeFailing event (only one node left, and it is failing).
    pub fn last_node_failing(&mut self, node_id: u8, b11: u8, timestamp_ms: u64) {
        self.events.push(TelemetryEvent::LastNodeFailing { node_id, b11, timestamp_ms });
    }

    /// Whether the feed contains any critical events.
    pub fn has_critical(&self) -> bool {
        self.events.iter().any(|e| e.is_critical())
    }

    /// Most recent event for a given node.
    pub fn latest_for_node(&self, node_id: u8) -> Option<&TelemetryEvent> {
        self.events.iter().rev().find(|e| e.node_id() == node_id)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn healthy_metrics_emit_no_events() {
        let mut feed = TelemetryFeed::new();
        feed.ingest(0, &HardwareMetrics::simulated_healthy(1000));
        assert!(feed.events.is_empty());
        assert!(!feed.has_critical());
    }

    #[test]
    fn critical_metrics_emit_critical_events() {
        let mut feed = TelemetryFeed::new();
        feed.ingest(0, &HardwareMetrics::simulated_critical(2000));
        assert!(!feed.events.is_empty());
        assert!(feed.has_critical());
    }

    #[test]
    fn node_down_is_critical() {
        let mut feed = TelemetryFeed::new();
        feed.node_down(1, 3000);
        assert!(feed.has_critical());
        let ev = feed.latest_for_node(1).unwrap();
        assert_eq!(ev.node_id(), 1);
    }

    #[test]
    fn last_node_failing_is_critical() {
        let mut feed = TelemetryFeed::new();
        feed.last_node_failing(2, 45, 4000);
        assert!(feed.has_critical());
    }

    #[test]
    fn latest_for_node_returns_most_recent() {
        let mut feed = TelemetryFeed::new();
        feed.ingest(0, &HardwareMetrics { gpu_temp_celsius: 90.0, timestamp_ms: 100,
            ..HardwareMetrics::simulated_healthy(0) });
        feed.ingest(0, &HardwareMetrics { gpu_temp_celsius: 99.0, timestamp_ms: 200,
            ..HardwareMetrics::simulated_healthy(0) });
        let latest = feed.latest_for_node(0).unwrap();
        assert_eq!(latest.timestamp_ms(), 200);
    }

    #[test]
    fn warn_below_critical_threshold() {
        let mut feed = TelemetryFeed::new();
        feed.ingest(0, &HardwareMetrics { gpu_temp_celsius: 87.0, timestamp_ms: 0,
            ..HardwareMetrics::simulated_healthy(0) });
        let any_critical = feed.events.iter().any(|e| e.is_critical());
        assert!(!any_critical, "87°C should be warn not critical");
        assert!(!feed.events.is_empty(), "87°C should emit a warn event");
    }
}
