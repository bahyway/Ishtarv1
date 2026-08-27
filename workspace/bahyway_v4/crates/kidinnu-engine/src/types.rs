//! GL-ALG-001-A2 §A2.1 — sorts and ground objects.
//! The honest floor is a TYPE: EpsilonFloor cannot exist at zero (F4).

/// ε > 0 by construction. `new` refuses 0.0 and negatives and NaN.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EpsilonFloor(f64);
impl EpsilonFloor {
    pub fn new(v: f64) -> Result<Self, &'static str> {
        if v.is_finite() && v > 0.0 { Ok(Self(v)) }
        else { Err("F4: epsilon must be finite and > 0 — certainty is forbidden by type") }
    }
    pub fn get(self) -> f64 { self.0 }
}

/// A zone of the HeptaShell lattice Z (126 zones: 7·14·21·28·35·21).
#[derive(Clone, Debug)]
pub struct Zone {
    pub id: u16,
    pub ring: u8,
    pub sector: u16,
    pub pop: u32,
    pub cx: f64,
    pub cy: f64,
    pub eps: EpsilonFloor,
}

/// A refuge r ∈ R: capacity-honest, with shielding factor s(r) ∈ (0,1].
#[derive(Clone, Debug)]
pub struct Refuge {
    pub id: String,
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub cap: u32,
    pub shield: f64, // underground < 1.0 discounts worst case
}

/// A sealed threat template Θ ∈ S with its public siren pattern σ(Θ).
/// F5: only sealed templates judge. `sealed` is checked at solver entry.
#[derive(Clone, Debug)]
pub struct ThreatTemplate {
    pub id: String,
    pub siren: String,
    pub src_x: f64,
    pub src_y: f64,
    pub bearing: f64, // direction of push
    pub spread: f64,
    pub sealed: bool,
}

/// A move m ∈ M(z): walk to a refuge, or outward egress.
#[derive(Clone, Debug, PartialEq)]
pub enum Move {
    ToRefuge { refuge_id: String, bearing: f64 },
    Egress { bearing: f64 },
    HoldUnderground { refuge_id: String },
}

/// The resident projection π_z (§A2.5): ONE zone, no aggregates.
#[derive(Clone, Debug)]
pub struct ResidentCard {
    pub zone_id: u16,
    pub siren: Option<String>,
    pub directive: Move,
    pub worst_case: f64,
    pub eps: f64,
    pub minimax: bool,
    pub seal_hex: String,
}
