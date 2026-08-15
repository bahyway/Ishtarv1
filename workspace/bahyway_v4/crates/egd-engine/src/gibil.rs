//! The Gibil Calculus — smith-fire, the refining flame that shapes metal.
//! (Renamed from the original "Girra" proposal: `girra-engine` already
//! owns Girra for the ecosystem's general sovereign monitoring
//! dashboard, and `nusku-engine`/`nusku-score`/`nusku-fuzzy` already own
//! Nusku for body-scan security classification. Gibil keeps the
//! fire/refining metaphor without colliding with either.)
//!
//! Phi_Gibil: complex nodal potential (voltage phasor). Edge
//! coefficient: complex admittance Y. Edge flow is Kirchhoff current:
//! I = Y (V_from - V_to). Conservation residual at each node is the KCL
//! violation — a nonzero kappa where physics says zero is a candidate
//! defect: ground fault, technical loss, meter fraud, failing joint.

use crate::phasor::Phasor;
use fdd_core::{Edge, Potential, Quantity};

pub struct Gibil;

impl Potential<Phasor> for Gibil {
    fn edge_flow(&self, edge: &Edge<Phasor>, v_from: Phasor, v_to: Phasor) -> Phasor {
        edge.coeff.mul(v_from.sub(v_to)) // I = Y * (Vf - Vt)
    }
}

/// Complex power at a node: S = V * conj(I) -> (P, Q).
pub fn complex_power(v: Phasor, i: Phasor) -> (f64, f64) {
    let s = v.mul(i.conj());
    (s.re, s.im) // active P, reactive Q
}

/// BIRQU — the lightning alert class. Where water floods gave Milu,
/// electricity strikes give Birqu.
pub const ALERT_CLASS: &str = "Birqu";

/// Fast-cadence law: electrical transients live in the millisecond
/// class. The fast homeostatic lane budget for Gibil residual sweeps:
pub const FAST_CADENCE_BUDGET_MICROS: u64 = 1_000; // 1 ms
