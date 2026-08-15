//! WpdPipelineMap — water/utility pipeline domain layer.
//!
//! Complements `ParticleMap` with pipeline-specific data:
//! segment physical attributes, flow characteristics, and leak candidates.
//!
//! `WpdPipelineMap` acts as a sidecar to the sovereign `ParticleMap` — every
//! pipeline segment still has a KAKI-bearing `MapParticle` in the map; this
//! module attaches the engineering metadata needed by WpdEngine routing.
//!
//! ## Leak detection
//!
//! Leaks are detected by multiple sources (pressure sensors, drone IR, visual
//! inspection) and ranked by severity on the QUALITY_DIVISOR=240 scale:
//! 240=critical rupture, 120=significant, 60=minor seepage, 0=suspected.

#![forbid(unsafe_code)]

use std::collections::HashMap;
use crate::mapparticle::{FlowDir, PipelineKind};

/// Sovereign quality divisor — must match ADR-001 (never 255).
pub const QUALITY_DIVISOR: u8 = 240;

// ── PipeMaterial ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipeMaterial {
    Steel,
    DuctileIron,
    Concrete,
    PVC,
    Clay,
    HDPE,   // High-Density Polyethylene — modern water mains
    Unknown,
}

impl PipeMaterial {
    /// Estimated remaining life factor (1.0 = new, 0.0 = end of life).
    /// Used to adjust routing cost in maintenance planning.
    pub fn life_factor(self, age_years: u16) -> f32 {
        let lifespan = match self {
            PipeMaterial::Steel       => 50u16,
            PipeMaterial::DuctileIron => 80,
            PipeMaterial::Concrete    => 60,
            PipeMaterial::PVC         => 40,
            PipeMaterial::Clay        => 100,
            PipeMaterial::HDPE        => 50,
            PipeMaterial::Unknown     => 30,
        };
        let age = age_years.min(lifespan);
        1.0 - (age as f32 / lifespan as f32)
    }

    pub fn label(self) -> &'static str {
        match self {
            PipeMaterial::Steel       => "Steel",
            PipeMaterial::DuctileIron => "Ductile Iron",
            PipeMaterial::Concrete    => "Concrete",
            PipeMaterial::PVC         => "PVC",
            PipeMaterial::Clay        => "Clay",
            PipeMaterial::HDPE        => "HDPE",
            PipeMaterial::Unknown     => "Unknown",
        }
    }
}

// ── PipelineSegmentData ───────────────────────────────────────────────────────

/// Engineering attributes for a pipeline segment particle.
#[derive(Debug, Clone)]
pub struct PipelineSegmentData {
    pub kind:         PipelineKind,
    pub flow:         FlowDir,
    pub diameter_mm:  u32,
    pub pressure_kpa: Option<u32>,
    pub material:     PipeMaterial,
    pub age_years:    Option<u16>,
    /// Nominal flow rate in litres per second (operational baseline).
    pub flow_rate_ls: Option<u32>,
}

impl PipelineSegmentData {
    pub fn new(kind: PipelineKind, flow: FlowDir, diameter_mm: u32) -> Self {
        PipelineSegmentData {
            kind,
            flow,
            diameter_mm,
            pressure_kpa: None,
            material:     PipeMaterial::Unknown,
            age_years:    None,
            flow_rate_ls: None,
        }
    }

    pub fn with_pressure(mut self, kpa: u32)          -> Self { self.pressure_kpa = Some(kpa); self }
    pub fn with_material(mut self, m: PipeMaterial)   -> Self { self.material = m; self }
    pub fn with_age(mut self, years: u16)              -> Self { self.age_years = Some(years); self }
    pub fn with_flow_rate(mut self, ls: u32)           -> Self { self.flow_rate_ls = Some(ls); self }

    /// Condition score 0–240: combines material life and age.
    pub fn condition_score(&self) -> u8 {
        let lf = self.material.life_factor(self.age_years.unwrap_or(0));
        (lf * QUALITY_DIVISOR as f32) as u8
    }
}

// ── Leak detection ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeakDetectionSource {
    /// Unexpected pressure drop between two sensors.
    PressureDrop,
    /// Field staff visual inspection report.
    VisualInspection,
    /// Soil moisture sensor adjacent to pipe trench.
    SoilMoisture,
    /// Drone-mounted infrared thermal imaging.
    DroneInfraRed,
    /// Acoustic / vibration sensor on pipe wall.
    AcousticSensor,
}

impl LeakDetectionSource {
    pub fn label(self) -> &'static str {
        match self {
            LeakDetectionSource::PressureDrop      => "Pressure Drop",
            LeakDetectionSource::VisualInspection  => "Visual Inspection",
            LeakDetectionSource::SoilMoisture      => "Soil Moisture",
            LeakDetectionSource::DroneInfraRed     => "Drone IR",
            LeakDetectionSource::AcousticSensor    => "Acoustic Sensor",
        }
    }
}

/// A potential leak event at a pipeline segment.
#[derive(Debug, Clone)]
pub struct LeakCandidate {
    /// The `MapParticle` particle_id of the leaking segment.
    pub particle_id: u32,
    /// Severity on QUALITY_DIVISOR=240 scale (240=critical rupture, 0=suspected).
    pub severity:    u8,
    pub source:      LeakDetectionSource,
    /// Epoch of detection (Unix seconds or Hijri year — caller's domain).
    pub detected_at: u32,
}

impl LeakCandidate {
    pub fn is_critical(&self)     -> bool { self.severity >= 200 }
    pub fn is_significant(&self)  -> bool { self.severity >= 100 }
    pub fn normalized_severity(&self) -> f32 { self.severity as f32 / QUALITY_DIVISOR as f32 }
}

// ── WpdPipelineMap ────────────────────────────────────────────────────────────

/// Domain layer: engineering data for all pipeline segments.
///
/// Keyed by `MapParticle.particle_id`.  Designed to be held alongside
/// a `ParticleMap` — the WpdEngine accesses both in tandem.
pub struct WpdPipelineMap {
    segments: HashMap<u32, PipelineSegmentData>,
    leaks:    Vec<LeakCandidate>,
}

impl WpdPipelineMap {
    pub fn new() -> Self { WpdPipelineMap { segments: HashMap::new(), leaks: Vec::new() } }

    pub fn register_segment(&mut self, particle_id: u32, data: PipelineSegmentData) {
        self.segments.insert(particle_id, data);
    }

    pub fn get_segment(&self, particle_id: u32) -> Option<&PipelineSegmentData> {
        self.segments.get(&particle_id)
    }

    pub fn register_leak(&mut self, leak: LeakCandidate) {
        self.leaks.push(leak);
    }

    /// All leaks with severity ≥ `threshold`.
    pub fn critical_leaks(&self, threshold: u8) -> Vec<&LeakCandidate> {
        self.leaks.iter().filter(|l| l.severity >= threshold).collect()
    }

    /// Segments of a given pipeline kind.
    pub fn segments_by_kind(&self, kind: PipelineKind) -> Vec<(u32, &PipelineSegmentData)> {
        self.segments.iter()
            .filter(|(_, s)| s.kind == kind)
            .map(|(&id, s)| (id, s))
            .collect()
    }

    /// Segments whose condition score is below `threshold` (0–240).
    /// Useful for maintenance prioritisation.
    pub fn deteriorated_segments(&self, threshold: u8) -> Vec<(u32, &PipelineSegmentData)> {
        self.segments.iter()
            .filter(|(_, s)| s.condition_score() < threshold)
            .map(|(&id, s)| (id, s))
            .collect()
    }

    pub fn segment_count(&self) -> usize { self.segments.len() }
    pub fn leak_count(&self)    -> usize { self.leaks.len() }
    pub fn all_leaks(&self)     -> &[LeakCandidate] { &self.leaks }
}

impl Default for WpdPipelineMap {
    fn default() -> Self { Self::new() }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapparticle::{FlowDir, PipelineKind};

    fn water_segment() -> PipelineSegmentData {
        PipelineSegmentData::new(PipelineKind::WaterSupply, FlowDir::DownStream, 200)
            .with_pressure(350)
            .with_material(PipeMaterial::DuctileIron)
            .with_age(15)
    }

    #[test]
    fn segment_registered_and_retrieved() {
        let mut m = WpdPipelineMap::new();
        m.register_segment(101, water_segment());
        assert!(m.get_segment(101).is_some());
        assert_eq!(m.segment_count(), 1);
    }

    #[test]
    fn leak_registered_and_retrieved() {
        let mut m = WpdPipelineMap::new();
        m.register_leak(LeakCandidate {
            particle_id: 101,
            severity:    200,
            source:      LeakDetectionSource::PressureDrop,
            detected_at: 1445_000,
        });
        assert_eq!(m.leak_count(), 1);
    }

    #[test]
    fn critical_leaks_threshold() {
        let mut m = WpdPipelineMap::new();
        m.register_leak(LeakCandidate { particle_id: 1, severity: 240, source: LeakDetectionSource::AcousticSensor, detected_at: 0 });
        m.register_leak(LeakCandidate { particle_id: 2, severity: 80,  source: LeakDetectionSource::SoilMoisture,  detected_at: 0 });
        m.register_leak(LeakCandidate { particle_id: 3, severity: 150, source: LeakDetectionSource::DroneInfraRed, detected_at: 0 });
        assert_eq!(m.critical_leaks(200).len(), 1);
        assert_eq!(m.critical_leaks(100).len(), 2);
        assert_eq!(m.critical_leaks(0).len(),   3);
    }

    #[test]
    fn segments_by_kind_filters() {
        let mut m = WpdPipelineMap::new();
        m.register_segment(1, PipelineSegmentData::new(PipelineKind::WaterSupply, FlowDir::DownStream, 200));
        m.register_segment(2, PipelineSegmentData::new(PipelineKind::Sewage,      FlowDir::DownStream, 150));
        m.register_segment(3, PipelineSegmentData::new(PipelineKind::WaterSupply, FlowDir::DownStream, 100));
        let water = m.segments_by_kind(PipelineKind::WaterSupply);
        assert_eq!(water.len(), 2);
    }

    #[test]
    fn condition_score_new_pipe_near_240() {
        let s = PipelineSegmentData::new(PipelineKind::WaterSupply, FlowDir::DownStream, 200)
            .with_material(PipeMaterial::DuctileIron)
            .with_age(0);
        assert_eq!(s.condition_score(), 240);
    }

    #[test]
    fn condition_score_old_pipe_near_zero() {
        let s = PipelineSegmentData::new(PipelineKind::WaterSupply, FlowDir::DownStream, 200)
            .with_material(PipeMaterial::DuctileIron)
            .with_age(80); // at end of life
        assert_eq!(s.condition_score(), 0);
    }

    #[test]
    fn deteriorated_segments_below_threshold() {
        let mut m = WpdPipelineMap::new();
        let old = PipelineSegmentData::new(PipelineKind::WaterSupply, FlowDir::DownStream, 200)
            .with_material(PipeMaterial::PVC).with_age(38); // ~5% remaining life
        let fresh = PipelineSegmentData::new(PipelineKind::WaterSupply, FlowDir::DownStream, 200)
            .with_material(PipeMaterial::HDPE).with_age(0);
        m.register_segment(1, old);
        m.register_segment(2, fresh);
        let det = m.deteriorated_segments(60);
        assert_eq!(det.len(), 1);
        assert_eq!(det[0].0, 1);
    }

    #[test]
    fn leak_candidate_severity_flags() {
        let critical = LeakCandidate { particle_id: 1, severity: 220, source: LeakDetectionSource::AcousticSensor, detected_at: 0 };
        let minor    = LeakCandidate { particle_id: 2, severity: 50,  source: LeakDetectionSource::SoilMoisture,  detected_at: 0 };
        assert!(critical.is_critical());
        assert!(!minor.is_critical());
        assert!(!minor.is_significant());
    }

    #[test]
    fn normalized_severity_100pct() {
        let l = LeakCandidate { particle_id: 1, severity: 240, source: LeakDetectionSource::PressureDrop, detected_at: 0 };
        assert!((l.normalized_severity() - 1.0).abs() < 0.01);
    }

    #[test]
    fn pipe_material_life_factor_new() {
        assert!((PipeMaterial::Steel.life_factor(0) - 1.0).abs() < 0.01);
    }

    #[test]
    fn pipe_material_labels_non_empty() {
        for m in [PipeMaterial::Steel, PipeMaterial::DuctileIron, PipeMaterial::Concrete,
                  PipeMaterial::PVC, PipeMaterial::Clay, PipeMaterial::HDPE, PipeMaterial::Unknown] {
            assert!(!m.label().is_empty());
        }
    }

    #[test]
    fn detection_source_labels_non_empty() {
        for s in [LeakDetectionSource::PressureDrop, LeakDetectionSource::VisualInspection,
                  LeakDetectionSource::SoilMoisture, LeakDetectionSource::DroneInfraRed,
                  LeakDetectionSource::AcousticSensor] {
            assert!(!s.label().is_empty());
        }
    }
}
