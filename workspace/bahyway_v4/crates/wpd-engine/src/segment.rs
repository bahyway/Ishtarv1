// ── Pipeline segment ──────────────────────────────────────────────

use crate::domain::{PipeMaterial, PipelineType};
use crate::sector::BaghdadSector;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentStatus {
    Active,
    Suspected,
    Defective,
    Decommissioned,
    Recovered,
}

impl SegmentStatus {
    pub fn name(self) -> &'static str {
        match self {
            SegmentStatus::Active         => "Active",
            SegmentStatus::Suspected      => "Suspected",
            SegmentStatus::Defective      => "Defective",
            SegmentStatus::Decommissioned => "Decommissioned",
            SegmentStatus::Recovered      => "Recovered",
        }
    }
}

#[derive(Debug, Clone)]
pub struct PipelineSegment {
    pub id:             String,
    pub sector:         BaghdadSector,
    pub pipeline_type:  PipelineType,
    pub lat_start:      f64,
    pub lon_start:      f64,
    pub lat_end:        f64,
    pub lon_end:        f64,
    pub diameter_mm:    u32,
    pub length_m:       f32,
    pub material:       PipeMaterial,
    pub year_installed: u16,
    pub last_inspected: Option<u16>,
    pub defect_score:   f32,
    pub status:         SegmentStatus,
}

impl PipelineSegment {
    pub fn age_years(&self) -> u16 {
        2026u16.saturating_sub(self.year_installed)
    }

    /// risk = defect_score×0.50 + material_susceptibility×0.30 + age_factor×0.20
    pub fn risk_score(&self) -> f32 {
        let age_factor = (self.age_years() as f32 / 100.0).min(1.0);
        (self.defect_score * 0.50
            + self.material.defect_susceptibility() * 0.30
            + age_factor * 0.20)
            .clamp(0.0, 1.0)
    }

    pub fn needs_urgent_inspection(&self) -> bool {
        self.risk_score() > 0.70
    }

    pub fn midpoint(&self) -> (f64, f64) {
        (
            (self.lat_start + self.lat_end) / 2.0,
            (self.lon_start + self.lon_end) / 2.0,
        )
    }
}

// ── Registry ──────────────────────────────────────────────────────

pub struct PipelineRegistry {
    segments: Vec<PipelineSegment>,
}

impl PipelineRegistry {
    pub fn new() -> Self {
        Self { segments: Vec::new() }
    }

    pub fn with_segments(segments: Vec<PipelineSegment>) -> Self {
        Self { segments }
    }

    pub fn get(&self, id: &str) -> Option<&PipelineSegment> {
        self.segments.iter().find(|s| s.id == id)
    }

    pub fn all(&self) -> &[PipelineSegment] {
        &self.segments
    }

    pub fn by_sector(&self, sector: BaghdadSector) -> Vec<&PipelineSegment> {
        self.segments.iter().filter(|s| s.sector == sector).collect()
    }

    pub fn urgent(&self) -> Vec<&PipelineSegment> {
        self.segments.iter().filter(|s| s.needs_urgent_inspection()).collect()
    }

    pub fn insert(&mut self, seg: PipelineSegment) {
        self.segments.push(seg);
    }

    pub fn len(&self) -> usize {
        self.segments.len()
    }

    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }
}

impl Default for PipelineRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sector::BaghdadSector;
    use crate::domain::{PipeMaterial, PipelineType};

    fn make_segment(id: &str, year: u16, score: f32) -> PipelineSegment {
        PipelineSegment {
            id: id.to_string(),
            sector: BaghdadSector::SadrCity,
            pipeline_type: PipelineType::Water,
            lat_start: 33.36, lon_start: 44.44,
            lat_end:   33.37, lon_end:   44.45,
            diameter_mm: 250, length_m: 1000.0,
            material: PipeMaterial::AsbestosCement,
            year_installed: year,
            last_inspected: None,
            defect_score: score,
            status: SegmentStatus::Active,
        }
    }

    #[test]
    fn age_years_sadr_city_1968() {
        let seg = make_segment("X", 1968, 0.0);
        assert_eq!(seg.age_years(), 58);
    }

    #[test]
    fn high_defect_score_urgent() {
        let seg = make_segment("X", 1968, 0.90);
        assert!(seg.needs_urgent_inspection());
    }

    #[test]
    fn low_defect_score_not_urgent() {
        let seg = make_segment("X", 2022, 0.05);
        assert!(!seg.needs_urgent_inspection());
    }

    #[test]
    fn registry_get_by_id() {
        let mut reg = PipelineRegistry::new();
        reg.insert(make_segment("BGH-TEST-001", 2000, 0.20));
        assert!(reg.get("BGH-TEST-001").is_some());
        assert!(reg.get("BGH-MISSING").is_none());
    }

    #[test]
    fn registry_by_sector() {
        let mut reg = PipelineRegistry::new();
        reg.insert(make_segment("X", 2000, 0.10));
        let results = reg.by_sector(BaghdadSector::SadrCity);
        assert_eq!(results.len(), 1);
        assert!(reg.by_sector(BaghdadSector::GreenZone).is_empty());
    }
}
