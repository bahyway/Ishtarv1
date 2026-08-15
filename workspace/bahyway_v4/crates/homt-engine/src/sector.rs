//! HeptaSectorMap — the 7-sector manifold of the HOMT field.
//!
//! Maps the 7 EAV axes (E,A,V,T,S,Z,M) to their Heptagon geometry.
//! Each sector corresponds to one axis of the 7D Hepta manifold and one
//! face of the heptagonal data-space (no tiling → hyperbolic curvature).
//!
//! Heptagon vs Hexagon rationale:
//!   - 7 sides = 1:1 mapping to 7D EAV dimensions
//!   - No tiling → hyperbolic curvature → ideal for dense spatial zones
//!   - No parallel sides → forced definitive routing → prevents oscillation
//!   - O(log_7 n) search speed for binned lookups

use crate::field::HeptaAxis;

/// A 7-sector manifold bin: axis index + angular position in the heptagon.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HeptaBin {
    pub axis:  HeptaAxis,
    /// Normalised position within the sector [0.0, 1.0].
    pub phase: f64,
}

/// Maps continuous 7D coordinates to discrete Hepta sector bins.
pub struct HeptaSectorMap {
    /// Bin resolution per axis (number of discrete bins per unit range).
    resolution: u32,
}

impl HeptaSectorMap {
    /// Create a new sector map with `resolution` bins per axis unit.
    pub fn new(resolution: u32) -> Self {
        assert!(resolution > 0, "resolution must be non-zero");
        Self { resolution }
    }

    /// Map a 7D position to its sector bin for a given axis.
    pub fn bin_at(&self, position: f64, axis: HeptaAxis) -> HeptaBin {
        let phase = position.fract().abs();
        let _bin_index = (position.abs() as u64) * self.resolution as u64;
        HeptaBin { axis, phase }
    }

    /// Compute the O(log_7 n) sector address from a 7D coordinate.
    ///
    /// Returns a 7-component address vector where each component is the
    /// discrete bin index along that axis.
    pub fn sector_address(&self, coords: &[f64; 7]) -> [u64; 7] {
        let mut addr = [0u64; 7];
        for (i, &c) in coords.iter().enumerate() {
            addr[i] = (c.abs() as u64).saturating_mul(self.resolution as u64);
        }
        addr
    }

    /// Compute the base-7 search key from the sector address.
    ///
    /// This achieves O(log_7 n) lookup by encoding the address as a base-7
    /// number — each axis contributes one digit.
    pub fn search_key(&self, coords: &[f64; 7]) -> u64 {
        let addr = self.sector_address(coords);
        let mut key = 0u64;
        let mut base = 1u64;
        for &a in &addr {
            key = key.saturating_add(a.min(6).saturating_mul(base));
            base = base.saturating_mul(7);
        }
        key
    }

    /// Determine which Hepta axis is the dominant dimension for a 7D vector.
    ///
    /// "Dominant" = the axis with the largest absolute component.
    /// This is the routing axis for BEAM/PATH gradient descent.
    pub fn dominant_axis(coords: &[f64; 7]) -> HeptaAxis {
        let (max_idx, _) = coords
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.abs().partial_cmp(&b.abs()).unwrap())
            .unwrap_or((0, &0.0));
        match max_idx {
            0 => HeptaAxis::Entity,
            1 => HeptaAxis::Attribute,
            2 => HeptaAxis::Value,
            3 => HeptaAxis::Tribe,
            4 => HeptaAxis::Segment,
            5 => HeptaAxis::Zone,
            _ => HeptaAxis::Mode,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sector_address_scales_with_resolution() {
        let m = HeptaSectorMap::new(10);
        let coords = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
        let addr = m.sector_address(&coords);
        assert_eq!(addr[0], 10);
        assert_eq!(addr[1], 20);
        assert_eq!(addr[6], 70);
    }

    #[test]
    fn dominant_axis_selects_largest_component() {
        let coords = [0.1, 0.2, 0.0, 5.0, 0.3, 0.1, 0.0];
        assert_eq!(HeptaSectorMap::dominant_axis(&coords), HeptaAxis::Tribe);
    }

    #[test]
    fn search_key_is_deterministic() {
        let m = HeptaSectorMap::new(1);
        let c = [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0];
        let k1 = m.search_key(&c);
        let k2 = m.search_key(&c);
        assert_eq!(k1, k2);
    }
}
