#![forbid(unsafe_code)]
//! ASHNAN KAKI particle class definitions (BC-AGRI-001 §4).

use crate::provenance::ProvenanceBlock;

/// KAKI particle class for agricultural data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AshnanKakiClass {
    /// 0x01 / KISHIB — Pest Identity particle (trap catch, species ID, outbreak).
    PestIdentity    = 0x01,
    /// 0x02 / ZIKRU — Soil/Crop Event particle (soil moisture, NDVI, weather).
    SoilCropEvent   = 0x02,
    /// 0x02 / PARZU — Livestock Health Event particle (ear-tag biosensor, vet report).
    LivestockEvent  = 0x03,
    /// 0x03 CrossTribe — External Ingestion particle (FAO/WOAH/met with provenance).
    ExternalIngest  = 0x04,
}

/// External ingestion particle — FAO bulletin, WOAH alert, or met-service forecast.
/// Carries mandatory provenance EAV block (BC-AGRI-001 §4.2).
#[derive(Debug, Clone)]
pub struct ExternalIngestionParticle {
    /// Provenance block — mandatory for all CrossTribe particles.
    pub provenance: ProvenanceBlock,
    /// Domain this bulletin addresses.
    pub domain: ExternalDomain,
    /// Sovereign HeptaMap cells covered by this bulletin.
    pub hepta_cells: [u16; 8],
    /// Bulletin reference identifier from the source organisation.
    pub bulletin_ref: u64,
}

/// Domain classification for external ingestion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExternalDomain {
    /// FAO crop prospects or drought bulletin.
    CropDrought,
    /// WOAH animal disease alert.
    AnimalDisease,
    /// National meteorological service forecast.
    Meteorological,
    /// Satellite NDVI / land-surface temperature direct feed.
    SatelliteRemoteSensing,
}
