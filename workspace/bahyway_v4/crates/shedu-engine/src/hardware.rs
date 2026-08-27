//! Hardware telemetry metrics — the raw measurements from the node.
//!
//! Shedu monitors four physical dimensions of the host machine:
//!   D_GPU  — GPU temperature (°C)
//!   D_VRAM — VRAM utilisation fraction
//!   D_CPU  — CPU load fraction
//!   D_NET  — Network latency (ms, inverted)
//!
//! These are the hardware analogue of the 7D Ibn Wahshiyya dimensions.
//! The composite D7 integrity score is computed in node_health.rs.

/// GPU temperature thresholds (°C).
pub const GPU_TEMP_NORMAL_MAX: f32 = 75.0;
pub const GPU_TEMP_WARN: f32 = 85.0;
pub const GPU_TEMP_CRITICAL: f32 = 95.0;

/// VRAM utilisation thresholds (fraction 0..1).
pub const VRAM_WARN_PCT: f32 = 0.85;
pub const VRAM_CRITICAL_PCT: f32 = 0.95;

/// CPU load thresholds (fraction 0..1).
pub const CPU_WARN_PCT: f32 = 0.80;
pub const CPU_CRITICAL_PCT: f32 = 0.95;

/// Network round-trip latency thresholds (ms).
pub const NET_LATENCY_WARN_MS: f32 = 100.0;
pub const NET_LATENCY_CRITICAL_MS: f32 = 300.0;

/// Raw hardware telemetry snapshot for one node at one moment.
#[derive(Debug, Clone)]
pub struct HardwareMetrics {
    /// GPU temperature in Celsius (0..200).
    pub gpu_temp_celsius: f32,
    /// VRAM currently used (MB).
    pub vram_used_mb: u32,
    /// Total VRAM available (MB).
    pub vram_total_mb: u32,
    /// CPU utilisation fraction (0..1).
    pub cpu_load: f32,
    /// RAM currently used (MB).
    pub ram_used_mb: u32,
    /// Total RAM available (MB).
    pub ram_total_mb: u32,
    /// Network round-trip latency (ms, 0 = disconnected sentinel is u32::MAX).
    pub net_latency_ms: f32,
    /// Unix-epoch millisecond timestamp of this reading.
    pub timestamp_ms: u64,
}

impl HardwareMetrics {
    /// VRAM utilisation fraction (0..1).
    pub fn vram_utilisation(&self) -> f32 {
        if self.vram_total_mb == 0 {
            return 1.0;
        } // treat unknown as saturated
        (self.vram_used_mb as f32 / self.vram_total_mb as f32).clamp(0.0, 1.0)
    }

    /// RAM utilisation fraction (0..1).
    pub fn ram_utilisation(&self) -> f32 {
        if self.ram_total_mb == 0 {
            return 1.0;
        }
        (self.ram_used_mb as f32 / self.ram_total_mb as f32).clamp(0.0, 1.0)
    }

    pub fn gpu_critical(&self) -> bool {
        self.gpu_temp_celsius >= GPU_TEMP_CRITICAL
    }
    pub fn vram_critical(&self) -> bool {
        self.vram_utilisation() >= VRAM_CRITICAL_PCT
    }
    pub fn cpu_critical(&self) -> bool {
        self.cpu_load >= CPU_CRITICAL_PCT
    }
    pub fn net_critical(&self) -> bool {
        self.net_latency_ms >= NET_LATENCY_CRITICAL_MS
    }

    /// Any single dimension is critical.
    pub fn any_critical(&self) -> bool {
        self.gpu_critical() || self.vram_critical() || self.cpu_critical() || self.net_critical()
    }

    /// Construct healthy simulated metrics (for testing).
    pub fn simulated_healthy(timestamp_ms: u64) -> Self {
        Self {
            gpu_temp_celsius: 55.0,
            vram_used_mb: 4_096,
            vram_total_mb: 16_384,
            cpu_load: 0.30,
            ram_used_mb: 8_192,
            ram_total_mb: 32_768,
            net_latency_ms: 12.0,
            timestamp_ms,
        }
    }

    /// Construct imminent-failure metrics (for testing).
    pub fn simulated_critical(timestamp_ms: u64) -> Self {
        Self {
            gpu_temp_celsius: 98.0,
            vram_used_mb: 15_900,
            vram_total_mb: 16_384,
            cpu_load: 0.97,
            ram_used_mb: 31_500,
            ram_total_mb: 32_768,
            net_latency_ms: 350.0,
            timestamp_ms,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn healthy_metrics_not_critical() {
        let m = HardwareMetrics::simulated_healthy(0);
        assert!(!m.any_critical());
        assert!(!m.gpu_critical());
        assert!(!m.vram_critical());
    }

    #[test]
    fn critical_metrics_detected() {
        let m = HardwareMetrics::simulated_critical(0);
        assert!(m.any_critical());
        assert!(m.gpu_critical());
        assert!(m.vram_critical());
        assert!(m.cpu_critical());
        assert!(m.net_critical());
    }

    #[test]
    fn vram_utilisation_fraction() {
        let m = HardwareMetrics::simulated_healthy(0);
        let util = m.vram_utilisation();
        assert!(util > 0.0 && util < 1.0);
    }

    #[test]
    fn zero_vram_total_treated_as_saturated() {
        let m = HardwareMetrics {
            vram_total_mb: 0,
            ..HardwareMetrics::simulated_healthy(0)
        };
        assert_eq!(m.vram_utilisation(), 1.0);
    }

    #[test]
    fn gpu_warn_not_critical() {
        let m = HardwareMetrics {
            gpu_temp_celsius: 87.0,
            ..HardwareMetrics::simulated_healthy(0)
        };
        assert!(!m.gpu_critical());
    }
}
