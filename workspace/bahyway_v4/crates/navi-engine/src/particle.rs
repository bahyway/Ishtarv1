//! NaviParticle — 7-dimensional sovereign navigation particle.
//!
//! Dimension mapping (v4):
//!   D1 = Latitude    (NaviCoord.lat  — f32 degrees)
//!   D2 = Longitude   (NaviCoord.lon  — f32 degrees)
//!   D3 = Altitude    (NaviCoord.alt  — f32 metres)
//!   D4 = Flow speed  (speed_kmh      — u8 km/h)
//!   D5 = Congestion  (congestion     — u8, 0=clear 255=gridlock)
//!   D6 = Surface     (SurfaceQuality — cost multiplier)
//!   D7 = Sovereignty (tribe          — TribeId)

use bahyway_core::TribeId;

pub type NaviNodeId = u32;

// ── D1–D3: Spatial coordinates ────────────────────────────────────────────────

/// Geographic position in WGS-84 degrees + altitude in metres.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NaviCoord {
    pub lat: f32,
    pub lon: f32,
    pub alt: f32,
}

impl NaviCoord {
    pub fn new(lat: f32, lon: f32, alt: f32) -> Self { NaviCoord { lat, lon, alt } }

    /// Approximate flat-earth distance in metres (fast, ±1% for short distances).
    pub fn flat_distance(&self, other: &NaviCoord) -> f32 {
        let dlat = (self.lat - other.lat) * 111_320.0;
        let dlon = (self.lon - other.lon) * 111_320.0 * self.lat.to_radians().cos();
        (dlat * dlat + dlon * dlon).sqrt()
    }
}

// ── D6: Surface quality ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceQuality {
    Excellent,  // paved, smooth — 0.80× cost
    Good,       // paved, minor wear — 1.00×
    Fair,       // unpaved / maintained — 1.20×
    Poor,       // rough / damaged — 1.50×
}

impl SurfaceQuality {
    pub fn cost_multiplier(self) -> f32 {
        match self {
            SurfaceQuality::Excellent => 0.80,
            SurfaceQuality::Good      => 1.00,
            SurfaceQuality::Fair      => 1.20,
            SurfaceQuality::Poor      => 1.50,
        }
    }

    pub fn from_u8(v: u8) -> Self {
        match v {
            0..=63   => SurfaceQuality::Excellent,
            64..=127 => SurfaceQuality::Good,
            128..=191 => SurfaceQuality::Fair,
            _         => SurfaceQuality::Poor,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            SurfaceQuality::Excellent => "Excellent",
            SurfaceQuality::Good      => "Good",
            SurfaceQuality::Fair      => "Fair",
            SurfaceQuality::Poor      => "Poor",
        }
    }
}

// ── Particle lifecycle state ──────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NaviParticleState {
    /// Fully passable — routing proceeds normally.
    Active,
    /// Route dead — tombstoned by NC3 in NaviCode pipeline.
    Dead,
    /// Sovereign restriction — blocked by access-control rule.
    Restricted,
    /// State not yet determined — treated as dead by the router.
    Unknown,
}

impl NaviParticleState {
    pub fn is_passable(self) -> bool {
        matches!(self, NaviParticleState::Active)
    }
}

// ── Anti-spoofing signal ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NaviSignal {
    /// Signal verified — AkkadiSeal intact.
    Strong,
    /// Signal degraded but usable — minor drift.
    Degraded,
    /// Signal jammed — particle must be treated as Dead.
    Jammed,
}

impl NaviSignal {
    pub fn is_usable(self) -> bool {
        !matches!(self, NaviSignal::Jammed)
    }
}

// ── NaviParticle ──────────────────────────────────────────────────────────────

/// A 7-dimensional sovereign navigation particle.
#[derive(Debug, Clone)]
pub struct NaviParticle {
    pub coord:      NaviCoord,          // D1 D2 D3
    pub speed_kmh:  u8,                 // D4: 0=stopped 255=max speed
    pub congestion: u8,                 // D5: 0=clear 255=gridlock
    pub surface:    SurfaceQuality,     // D6
    pub tribe:      TribeId,            // D7
    pub state:      NaviParticleState,
    pub signal:     NaviSignal,
}

impl NaviParticle {
    /// Construct a default Active particle at a coordinate.
    pub fn new(coord: NaviCoord, tribe: TribeId) -> Self {
        NaviParticle {
            coord,
            tribe,
            speed_kmh:  60,
            congestion: 0,
            surface:    SurfaceQuality::Good,
            state:      NaviParticleState::Active,
            signal:     NaviSignal::Strong,
        }
    }

    /// A particle is passable when Active and not Jammed.
    pub fn is_passable(&self) -> bool {
        self.state.is_passable() && self.signal.is_usable()
    }

    /// Intrinsic traversal cost — congestion penalty × surface multiplier.
    /// Range: [0.80, 7.50]. Used to seed the EdgeCostMatrix.
    pub fn intrinsic_cost(&self) -> f32 {
        let congestion_factor = 1.0 + (self.congestion as f32 / 255.0) * 4.0;
        congestion_factor * self.surface.cost_multiplier()
    }

    pub fn kill(&mut self)       { self.state = NaviParticleState::Dead; }
    pub fn restrict(&mut self)   { self.state = NaviParticleState::Restricted; }
    pub fn jam(&mut self)        { self.signal = NaviSignal::Jammed; }
    pub fn degrade_signal(&mut self) { self.signal = NaviSignal::Degraded; }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn najaf_tribe() -> TribeId { TribeId::from_u16(0x0001) }
    fn coord()       -> NaviCoord { NaviCoord::new(31.99, 44.32, 0.0) }

    #[test]
    fn new_particle_is_active_and_strong() {
        let p = NaviParticle::new(coord(), najaf_tribe());
        assert!(p.is_passable());
        assert_eq!(p.state, NaviParticleState::Active);
        assert_eq!(p.signal, NaviSignal::Strong);
    }

    #[test]
    fn dead_particle_is_not_passable() {
        let mut p = NaviParticle::new(coord(), najaf_tribe());
        p.kill();
        assert!(!p.is_passable());
    }

    #[test]
    fn restricted_particle_is_not_passable() {
        let mut p = NaviParticle::new(coord(), najaf_tribe());
        p.restrict();
        assert!(!p.is_passable());
    }

    #[test]
    fn jammed_particle_is_not_passable() {
        let mut p = NaviParticle::new(coord(), najaf_tribe());
        p.jam();
        assert!(!p.is_passable());
    }

    #[test]
    fn degraded_signal_is_still_passable() {
        let mut p = NaviParticle::new(coord(), najaf_tribe());
        p.degrade_signal();
        assert!(p.is_passable());
    }

    #[test]
    fn intrinsic_cost_clear_good() {
        let p = NaviParticle::new(coord(), najaf_tribe());
        // congestion=0 → factor=1.0; Good surface=1.0 → cost=1.0
        assert!((p.intrinsic_cost() - 1.0).abs() < 0.01);
    }

    #[test]
    fn intrinsic_cost_excellent_surface() {
        let mut p = NaviParticle::new(coord(), najaf_tribe());
        p.surface = SurfaceQuality::Excellent;
        assert!(p.intrinsic_cost() < 1.0);
    }

    #[test]
    fn intrinsic_cost_full_congestion_poor() {
        let mut p = NaviParticle::new(coord(), najaf_tribe());
        p.congestion = 255;
        p.surface    = SurfaceQuality::Poor;
        // congestion_factor ≈ 5.0; surface = 1.50 → ≈ 7.50
        assert!(p.intrinsic_cost() > 7.0);
    }

    #[test]
    fn surface_quality_from_u8_ranges() {
        assert_eq!(SurfaceQuality::from_u8(0),   SurfaceQuality::Excellent);
        assert_eq!(SurfaceQuality::from_u8(64),  SurfaceQuality::Good);
        assert_eq!(SurfaceQuality::from_u8(128), SurfaceQuality::Fair);
        assert_eq!(SurfaceQuality::from_u8(192), SurfaceQuality::Poor);
    }

    #[test]
    fn surface_cost_order() {
        assert!(SurfaceQuality::Excellent.cost_multiplier() < SurfaceQuality::Good.cost_multiplier());
        assert!(SurfaceQuality::Good.cost_multiplier()      < SurfaceQuality::Fair.cost_multiplier());
        assert!(SurfaceQuality::Fair.cost_multiplier()      < SurfaceQuality::Poor.cost_multiplier());
    }

    #[test]
    fn naviparticlestate_passable_only_active() {
        assert!(NaviParticleState::Active.is_passable());
        assert!(!NaviParticleState::Dead.is_passable());
        assert!(!NaviParticleState::Restricted.is_passable());
        assert!(!NaviParticleState::Unknown.is_passable());
    }

    #[test]
    fn navisignal_jammed_not_usable() {
        assert!(NaviSignal::Strong.is_usable());
        assert!(NaviSignal::Degraded.is_usable());
        assert!(!NaviSignal::Jammed.is_usable());
    }

    #[test]
    fn flat_distance_same_point_is_zero() {
        let c = coord();
        assert!(c.flat_distance(&c) < 0.01);
    }

    #[test]
    fn flat_distance_one_degree_lat() {
        let a = NaviCoord::new(31.0, 44.0, 0.0);
        let b = NaviCoord::new(32.0, 44.0, 0.0);
        let d = a.flat_distance(&b);
        // 1° lat ≈ 111 km
        assert!(d > 100_000.0 && d < 120_000.0);
    }

    #[test]
    fn surface_quality_labels_non_empty() {
        for s in [SurfaceQuality::Excellent, SurfaceQuality::Good, SurfaceQuality::Fair, SurfaceQuality::Poor] {
            assert!(!s.label().is_empty());
        }
    }

    #[test]
    fn tribe_stored_correctly() {
        let t = TribeId::from_u16(0xABCD);
        let p = NaviParticle::new(coord(), t);
        assert_eq!(p.tribe, t);
    }

    #[test]
    fn coord_fields_stored() {
        let c = NaviCoord::new(31.99, 44.32, 15.0);
        assert!((c.lat - 31.99).abs() < 0.001);
        assert!((c.lon - 44.32).abs() < 0.001);
        assert!((c.alt - 15.0).abs() < 0.001);
    }
}
