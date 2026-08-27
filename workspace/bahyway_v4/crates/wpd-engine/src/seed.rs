// ── 7 sovereign Baghdad pipeline segments ─────────────────────────
// Heptagram sectors H7-00 through H7-06; coordinates near 33°N 44°E.

use crate::domain::{PipeMaterial, PipelineType};
use crate::sector::BaghdadSector;
use crate::segment::{PipelineRegistry, PipelineSegment, SegmentStatus};

pub fn sovereign_segments() -> Vec<PipelineSegment> {
    vec![
        // H7-00 SUN — Green Zone diplomatic core, modern ductile iron
        PipelineSegment {
            id: "BGH-GZ-001".to_string(),
            sector: BaghdadSector::GreenZone,
            pipeline_type: PipelineType::Water,
            lat_start: 33.3128,
            lon_start: 44.3614,
            lat_end: 33.3250,
            lon_end: 44.3800,
            diameter_mm: 400,
            length_m: 1_800.0,
            material: PipeMaterial::DuctileIron,
            year_installed: 2005,
            last_inspected: Some(2024),
            defect_score: 0.12,
            status: SegmentStatus::Active,
        },
        // H7-01 MOON — Al-Kadhimiya, 1978 cast iron, moderate risk
        PipelineSegment {
            id: "BGH-KZ-001".to_string(),
            sector: BaghdadSector::AlKadhimiya,
            pipeline_type: PipelineType::Water,
            lat_start: 33.3780,
            lon_start: 44.3381,
            lat_end: 33.3950,
            lon_end: 44.3550,
            diameter_mm: 300,
            length_m: 2_200.0,
            material: PipeMaterial::CastIron,
            year_installed: 1978,
            last_inspected: Some(2022),
            defect_score: 0.45,
            status: SegmentStatus::Suspected,
        },
        // H7-02 MERCURY — Sadr City, 1968 asbestos cement, highest density + defect
        PipelineSegment {
            id: "BGH-SC-001".to_string(),
            sector: BaghdadSector::SadrCity,
            pipeline_type: PipelineType::Water,
            lat_start: 33.3614,
            lon_start: 44.4478,
            lat_end: 33.3780,
            lon_end: 44.4650,
            diameter_mm: 250,
            length_m: 3_100.0,
            material: PipeMaterial::AsbestosCement,
            year_installed: 1968,
            last_inspected: Some(2021),
            defect_score: 0.78,
            status: SegmentStatus::Defective,
        },
        // H7-03 VENUS — Karrada, 1995 ductile iron, low-risk residential
        PipelineSegment {
            id: "BGH-KR-001".to_string(),
            sector: BaghdadSector::Karrada,
            pipeline_type: PipelineType::Water,
            lat_start: 33.3017,
            lon_start: 44.3953,
            lat_end: 33.3175,
            lon_end: 44.4120,
            diameter_mm: 350,
            length_m: 1_500.0,
            material: PipeMaterial::DuctileIron,
            year_installed: 1995,
            last_inspected: Some(2023),
            defect_score: 0.28,
            status: SegmentStatus::Active,
        },
        // H7-04 MARS — Rashid, 1972 cast iron, 500mm trunk main
        PipelineSegment {
            id: "BGH-RS-001".to_string(),
            sector: BaghdadSector::Rashid,
            pipeline_type: PipelineType::Water,
            lat_start: 33.2817,
            lon_start: 44.3978,
            lat_end: 33.2980,
            lon_end: 44.4150,
            diameter_mm: 500,
            length_m: 2_800.0,
            material: PipeMaterial::CastIron,
            year_installed: 1972,
            last_inspected: Some(2022),
            defect_score: 0.55,
            status: SegmentStatus::Suspected,
        },
        // H7-05 JUPITER — Al-Jadria, 2012 plastic, newest and lowest risk
        PipelineSegment {
            id: "BGH-JD-001".to_string(),
            sector: BaghdadSector::AlJadria,
            pipeline_type: PipelineType::Water,
            lat_start: 33.2953,
            lon_start: 44.3614,
            lat_end: 33.3100,
            lon_end: 44.3780,
            diameter_mm: 300,
            length_m: 1_200.0,
            material: PipeMaterial::Plastic,
            year_installed: 2012,
            last_inspected: Some(2025),
            defect_score: 0.08,
            status: SegmentStatus::Active,
        },
        // H7-06 SATURN — Al-Mansour, 2001 ductile iron, dense western district
        PipelineSegment {
            id: "BGH-MN-001".to_string(),
            sector: BaghdadSector::AlMansour,
            pipeline_type: PipelineType::Water,
            lat_start: 33.3300,
            lon_start: 44.3150,
            lat_end: 33.3450,
            lon_end: 44.3320,
            diameter_mm: 350,
            length_m: 2_000.0,
            material: PipeMaterial::DuctileIron,
            year_installed: 2001,
            last_inspected: Some(2023),
            defect_score: 0.32,
            status: SegmentStatus::Active,
        },
    ]
}

/// Build a registry pre-loaded with all 7 sovereign Baghdad segments.
pub fn seed_registry() -> PipelineRegistry {
    PipelineRegistry::with_segments(sovereign_segments())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sector::BaghdadSector;

    #[test]
    fn exactly_seven_sovereign_segments() {
        assert_eq!(sovereign_segments().len(), 7);
    }

    #[test]
    fn all_seven_sectors_represented() {
        let segs = sovereign_segments();
        for sector in BaghdadSector::ALL {
            assert!(
                segs.iter().any(|s| s.sector == sector),
                "sector {:?} missing from seed",
                sector
            );
        }
    }

    #[test]
    fn sadr_city_is_defective() {
        let segs = sovereign_segments();
        let sc = segs.iter().find(|s| s.id == "BGH-SC-001").unwrap();
        assert_eq!(sc.status, SegmentStatus::Defective);
        assert!(sc.defect_score > 0.70);
    }

    #[test]
    fn seed_registry_has_seven_segments() {
        let reg = seed_registry();
        assert_eq!(reg.len(), 7);
    }

    #[test]
    fn green_zone_is_lowest_defect_score() {
        let segs = sovereign_segments();
        let gz = segs
            .iter()
            .find(|s| s.sector == BaghdadSector::GreenZone)
            .unwrap();
        for other in &segs {
            if other.sector != BaghdadSector::GreenZone && other.sector != BaghdadSector::AlJadria {
                assert!(
                    gz.defect_score <= other.defect_score,
                    "GreenZone score {} should be <= {} ({})",
                    gz.defect_score,
                    other.defect_score,
                    other.id
                );
            }
        }
    }
}
