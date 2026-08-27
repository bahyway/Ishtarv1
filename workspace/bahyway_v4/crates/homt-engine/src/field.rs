//! OrbitField — the Φ and Ψ field interface for HOMT.
//!
//! `OrbitField` is the pure-Rust interface that every scoring/routing
//! component must implement to participate in the HOMT PDE.
//!
//! Implementors live in separate crates (score-engine, fuzzy-engine, etc.)
//! and are composed by the eridu-runtime during the orbit evolution loop.

use enkidb_kaki::IdentityKaki;

/// The 7D EAV manifold axes: (Entity, Attribute, Value, Tribe, Segment, Zone, Mode).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum HeptaAxis {
    Entity = 0,    // E — KAKI uuid_hash dimension
    Attribute = 1, // A — attr_hash dimension
    Value = 2,     // V — value magnitude dimension
    Tribe = 3,     // T — tribe_id dimension
    Segment = 4,   // S — spatial segment dimension
    Zone = 5,      // Z — temporal zone dimension
    Mode = 6,      // M — operational mode dimension
}

impl HeptaAxis {
    pub fn index(self) -> usize {
        self as usize
    }
}

/// A 7D position vector in the Hepta manifold.
pub type HeptaPosition = [f64; 7];

/// OrbitField — the mathematical interface for a field participating in HOMT.
///
/// Each method corresponds to a term in the core PDE:
///   ∂φ/∂t = −∇·(φV) + G(x,t) − S(x,t) − Δφ
pub trait OrbitField {
    /// φ(x,t) — particle data density at `kaki`'s manifold position at `epoch`.
    ///
    /// Maps to: the left-hand side ∂φ/∂t accumulated over the Journal.
    fn density_at(&self, kaki: &IdentityKaki, epoch: u64) -> f64;

    /// -∇Ψ(x) at `kaki`'s position — the routing velocity vector (7 components).
    ///
    /// Maps to: BEAM/PATH routing = gradient descent on quality potential Ψ.
    fn gradient(&self, kaki: &IdentityKaki, epoch: u64) -> HeptaPosition;

    /// G(x,t) — source injection rate at `epoch`.
    ///
    /// Maps to: REGISTERED / INITIAL_SCAN events entering the manifold.
    fn source_term(&self, epoch: u64) -> f64;

    /// S(x,t) — sink drain rate at `epoch`.
    ///
    /// Maps to: QUARANTINED / DEAD events leaving the active field.
    fn sink_term(&self, epoch: u64) -> f64;

    /// Δφ — Laplacian smoothing contribution from this field at `kaki`, `epoch`.
    ///
    /// Maps to: VGCA-Δ spatial smoothing / SCORED events.
    /// Default implementation returns 0.0 (no smoothing) — override in VGCA.
    fn laplacian_smoothing(&self, _kaki: &IdentityKaki, _epoch: u64) -> f64 {
        0.0
    }

    /// Advance the field by one PDE time step of size `dt`.
    ///
    /// Returns the new density: φ(t+dt) using explicit Euler integration.
    /// For stability, dt must satisfy the convergence condition (see convergence module).
    fn step_euler(&self, kaki: &IdentityKaki, epoch: u64, dt: f64) -> f64 {
        let phi = self.density_at(kaki, epoch);
        let grad = self.gradient(kaki, epoch);
        let g = self.source_term(epoch);
        let s = self.sink_term(epoch);
        let lap = self.laplacian_smoothing(kaki, epoch);

        // Flow divergence: ∇·(φV) ≈ φ × |∇Ψ| (simplified scalar approximation)
        let flow_div: f64 = phi * grad.iter().map(|v| v.abs()).sum::<f64>();

        phi + dt * (-flow_div + g - s - lap)
    }
}

/// A null field — returns zero for all terms.  Used for testing and as a
/// safe default when a field component is not yet wired up.
pub struct NullOrbitField;

impl OrbitField for NullOrbitField {
    fn density_at(&self, _: &IdentityKaki, _: u64) -> f64 {
        0.0
    }
    fn gradient(&self, _: &IdentityKaki, _: u64) -> HeptaPosition {
        [0.0; 7]
    }
    fn source_term(&self, _: u64) -> f64 {
        0.0
    }
    fn sink_term(&self, _: u64) -> f64 {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bahyway_core::TribeId;
    use enkidb_kaki::{IdentityKaki as IK, KakiMinter, KakiRole};

    fn test_kaki() -> IdentityKaki {
        let m = KakiMinter::new(TribeId::from_u16(0x0001));
        let k = m.mint_identity(0xDEAD_BEEF, KakiRole::Zikru);
        IK::try_from_kaki(k).expect("identity kaki")
    }

    #[test]
    fn null_field_is_zero_everywhere() {
        let k = test_kaki();
        let f = NullOrbitField;
        assert_eq!(f.density_at(&k, 0), 0.0);
        assert_eq!(f.gradient(&k, 0), [0.0; 7]);
        assert_eq!(f.source_term(0), 0.0);
        assert_eq!(f.sink_term(0), 0.0);
    }

    #[test]
    fn euler_step_with_source_increases_density() {
        struct SimpleSource;
        impl OrbitField for SimpleSource {
            fn density_at(&self, _: &IdentityKaki, _: u64) -> f64 {
                0.5
            }
            fn gradient(&self, _: &IdentityKaki, _: u64) -> HeptaPosition {
                [0.0; 7]
            }
            fn source_term(&self, _: u64) -> f64 {
                0.1
            }
            fn sink_term(&self, _: u64) -> f64 {
                0.0
            }
        }
        let k = test_kaki();
        let phi_next = SimpleSource.step_euler(&k, 0, 1.0);
        // φ(t+1) = 0.5 + 1.0*(0 + 0.1 - 0 - 0) = 0.6
        assert!((phi_next - 0.6).abs() < 1e-9);
    }

    #[test]
    fn hepta_axis_indices_are_0_through_6() {
        use HeptaAxis::*;
        assert_eq!(Entity.index(), 0);
        assert_eq!(Attribute.index(), 1);
        assert_eq!(Value.index(), 2);
        assert_eq!(Tribe.index(), 3);
        assert_eq!(Segment.index(), 4);
        assert_eq!(Zone.index(), 5);
        assert_eq!(Mode.index(), 6);
    }
}
