#![forbid(unsafe_code)]
//! Atmospheric threat intelligence — gas kill-zone classification.
//!
//! Earthquake ruptures gas mains, sewage lines, and industrial storage.
//! This module classifies atmospheric threat levels per agent species
//! (BC-QUAKE-001 §9), feeding the NANSHE Gaussian dispersion bridge.

use crate::error::EsarhaddonError;

/// Gas species tracked by ESARHADDON atmospheric threat model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GasSpecies {
    /// CH₄ — methane from ruptured gas mains. LEL 5% v/v.
    Methane,
    /// CO — carbon monoxide from generators and fire. Lethal >1200 ppm.
    CarbonMonoxide,
    /// H₂S — hydrogen sulphide from sewage and industrial. Lethal >500 ppm.
    HydrogenSulphide,
    /// PM2.5 — concrete dust from collapse. Acute risk >500 µg/m³.
    ParticulateMatter,
}

impl GasSpecies {
    /// Sovereign lethal threshold for this species (species-specific units).
    /// CH₄: LEL fraction (0–1.0); CO/H₂S: ppm; PM2.5: µg/m³.
    pub fn lethal_threshold(&self) -> f32 {
        match self {
            Self::Methane            => 0.05,   // 5% v/v LEL
            Self::CarbonMonoxide     => 1200.0, // ppm
            Self::HydrogenSulphide   => 500.0,  // ppm
            Self::ParticulateMatter  => 500.0,  // µg/m³
        }
    }
}

/// Classified atmospheric threat for a HeptaMap cell.
#[derive(Debug, Clone)]
pub struct AtmosphericThreat {
    /// Gas species.
    pub species: GasSpecies,
    /// Measured or modelled concentration (species units).
    pub concentration: f32,
    /// Fraction of lethal threshold (0.0 = safe, ≥1.0 = lethal).
    pub lethality_fraction: f32,
    /// HeptaMap E7 cell.
    pub hepta_cell: u16,
}

impl AtmosphericThreat {
    pub fn new(
        species: GasSpecies,
        concentration: f32,
        hepta_cell: u16,
    ) -> Result<Self, EsarhaddonError> {
        if concentration < 0.0 {
            return Err(EsarhaddonError::InvalidConcentration(concentration));
        }
        let lethality_fraction = concentration / species.lethal_threshold();
        Ok(Self { species, concentration, lethality_fraction, hepta_cell })
    }

    /// Returns true if concentration exceeds the sovereign lethal threshold.
    pub fn is_lethal(&self) -> bool {
        self.lethality_fraction >= 1.0
    }
}
