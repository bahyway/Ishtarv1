// ── 12-band spectral scan for buried pipeline defect detection ────
// VNIR (B1-B4): 450–850 nm  | SWIR (B5-B8): 1050–1650 nm
// TIR  (B9-B12): 8000–12000 nm

use crate::defect::DefectClass;
use crate::error::{WpdError, WpdResult};

pub const WPD_SPECTRAL_BANDS: usize = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SpectralBand {
    B1Blue = 0,
    B2Green = 1,
    B3Red = 2,
    B4Nir = 3,
    B5Swir1 = 4,
    B6Swir2 = 5,
    B7Swir3 = 6,
    B8Swir4 = 7,
    B9Tir1 = 8,
    B10Tir2 = 9,
    B11Tir3 = 10,
    B12Tir4 = 11,
}

impl SpectralBand {
    pub fn wavelength_nm(self) -> u32 {
        match self {
            SpectralBand::B1Blue => 450,
            SpectralBand::B2Green => 550,
            SpectralBand::B3Red => 650,
            SpectralBand::B4Nir => 850,
            SpectralBand::B5Swir1 => 1050,
            SpectralBand::B6Swir2 => 1250,
            SpectralBand::B7Swir3 => 1450,
            SpectralBand::B8Swir4 => 1650,
            SpectralBand::B9Tir1 => 8000,
            SpectralBand::B10Tir2 => 9000,
            SpectralBand::B11Tir3 => 10500,
            SpectralBand::B12Tir4 => 12000,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            SpectralBand::B1Blue => "B1-Blue",
            SpectralBand::B2Green => "B2-Green",
            SpectralBand::B3Red => "B3-Red",
            SpectralBand::B4Nir => "B4-NIR",
            SpectralBand::B5Swir1 => "B5-SWIR1",
            SpectralBand::B6Swir2 => "B6-SWIR2",
            SpectralBand::B7Swir3 => "B7-SWIR3",
            SpectralBand::B8Swir4 => "B8-SWIR4",
            SpectralBand::B9Tir1 => "B9-TIR1",
            SpectralBand::B10Tir2 => "B10-TIR2",
            SpectralBand::B11Tir3 => "B11-TIR3",
            SpectralBand::B12Tir4 => "B12-TIR4",
        }
    }
}

// ── Spectral scan ─────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SpectralScan {
    pub segment_id: String,
    pub bands: [f32; WPD_SPECTRAL_BANDS],
    pub timestamp_ms: u64,
    pub altitude_m: f32,
    pub gsd_cm: f32,
}

impl SpectralScan {
    pub fn new(
        segment_id: impl Into<String>,
        bands: [f32; WPD_SPECTRAL_BANDS],
        timestamp_ms: u64,
        altitude_m: f32,
        gsd_cm: f32,
    ) -> WpdResult<Self> {
        for (i, &b) in bands.iter().enumerate() {
            if !(0.0..=1.0).contains(&b) {
                return Err(WpdError::InvalidScan(format!("band[{i}]={b} out of [0,1]")));
            }
        }
        Ok(Self {
            segment_id: segment_id.into(),
            bands,
            timestamp_ms,
            altitude_m,
            gsd_cm,
        })
    }

    /// Mean of TIR bands B9-B12 — heat anomaly from leaking water or soil disturbance.
    pub fn thermal_anomaly_index(&self) -> f32 {
        (self.bands[8] + self.bands[9] + self.bands[10] + self.bands[11]) / 4.0
    }

    /// Mean of B5-SWIR1 and B6-SWIR2 — moisture content indicator.
    pub fn moisture_index(&self) -> f32 {
        (self.bands[4] + self.bands[5]) / 2.0
    }

    /// Mean of B3-Red and B8-SWIR4 — corrosion oxide spectral response.
    pub fn corrosion_index(&self) -> f32 {
        (self.bands[2] + self.bands[7]) / 2.0
    }

    /// NDVI-style: (NIR − Green) / (NIR + Green) — surface vegetation cover.
    pub fn vegetation_index(&self) -> f32 {
        let nir = self.bands[3];
        let green = self.bands[1];
        let denom = nir + green;
        if denom < 1e-6 {
            0.0
        } else {
            (nir - green) / denom
        }
    }

    /// Composite defect score: thermal×0.40 + moisture×0.30 + corrosion×0.20 + veg×0.10
    pub fn composite_defect_score(&self) -> f32 {
        let veg = self.vegetation_index().max(0.0);
        (self.thermal_anomaly_index() * 0.40
            + self.moisture_index() * 0.30
            + self.corrosion_index() * 0.20
            + veg * 0.10)
            .clamp(0.0, 1.0)
    }

    pub fn classify(&self) -> DefectClass {
        DefectClass::from_score(self.composite_defect_score())
    }
}

// ── Seed spectral reference signatures ───────────────────────────

/// High NIR due to saturated soil moisture above a water leak.
pub fn water_leak_signature() -> [f32; WPD_SPECTRAL_BANDS] {
    [
        0.05, 0.10, 0.08, 0.72, 0.65, 0.60, 0.30, 0.20, 0.55, 0.50, 0.45, 0.40,
    ]
}

/// Elevated SWIR from hydrocarbon absorption scattering along the SWIR continuum.
pub fn oil_leak_signature() -> [f32; WPD_SPECTRAL_BANDS] {
    [
        0.04, 0.06, 0.05, 0.10, 0.35, 0.50, 0.60, 0.70, 0.30, 0.28, 0.25, 0.22,
    ]
}

/// Uniform low reflectance across all bands characteristic of sewage blockage sites.
pub fn sewage_blockage_signature() -> [f32; WPD_SPECTRAL_BANDS] {
    [
        0.12, 0.13, 0.14, 0.12, 0.15, 0.13, 0.12, 0.11, 0.14, 0.13, 0.12, 0.12,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn zero_scan(id: &str) -> SpectralScan {
        SpectralScan::new(id, [0.0f32; 12], 0, 100.0, 5.0).unwrap()
    }

    #[test]
    fn band_count_is_twelve() {
        assert_eq!(WPD_SPECTRAL_BANDS, 12);
    }

    #[test]
    fn invalid_band_rejected() {
        let mut bands = [0.0f32; 12];
        bands[5] = 1.5;
        assert!(SpectralScan::new("X", bands, 0, 100.0, 5.0).is_err());
    }

    #[test]
    fn zero_scan_scores_zero() {
        let scan = zero_scan("X");
        assert_eq!(scan.composite_defect_score(), 0.0);
    }

    #[test]
    fn water_leak_scores_high() {
        let scan = SpectralScan::new("X", water_leak_signature(), 0, 100.0, 5.0).unwrap();
        assert!(
            scan.composite_defect_score() > 0.40,
            "water leak should score > 0.40, got {}",
            scan.composite_defect_score()
        );
    }

    #[test]
    fn oil_leak_scores_above_moderate() {
        let scan = SpectralScan::new("X", oil_leak_signature(), 0, 100.0, 5.0).unwrap();
        assert!(scan.composite_defect_score() > 0.20);
    }

    #[test]
    fn wavelengths_monotone_increasing() {
        let bands = [
            SpectralBand::B1Blue,
            SpectralBand::B2Green,
            SpectralBand::B3Red,
            SpectralBand::B4Nir,
            SpectralBand::B5Swir1,
            SpectralBand::B6Swir2,
            SpectralBand::B7Swir3,
            SpectralBand::B8Swir4,
            SpectralBand::B9Tir1,
            SpectralBand::B10Tir2,
            SpectralBand::B11Tir3,
            SpectralBand::B12Tir4,
        ];
        let wl: Vec<u32> = bands.iter().map(|b| b.wavelength_nm()).collect();
        for w in wl.windows(2) {
            assert!(
                w[0] < w[1],
                "wavelengths not monotone: {} >= {}",
                w[0],
                w[1]
            );
        }
    }
}
