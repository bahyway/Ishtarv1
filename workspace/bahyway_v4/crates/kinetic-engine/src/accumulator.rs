//! # Accumulator — Sovereign Force Integration
//!
//! Implements the discrete kinetic integrator from Millington's
//! *Game Physics Engine Development* Chapter 2, applied to
//! BahyWay's sovereign 7D Heptagon space.
//!
//! ## The Integration Equation
//!
//! ```text
//! F_total = I_phys + I_mem + I_learn     (force accumulation)
//! a = F_total * inverse_mass              (Newton: F = ma)
//! v(t+1) = v(t) + a * dt                 (velocity integration)
//! v(t+1) = v(t+1) * damping^dt           (health decay)
//! x(t+1) = x(t) + v(t+1) * dt           (position integration)
//! ```

use crate::vec7d::Vec7D;
use crate::force::ForceGenerator;

// ── KineticParticle ─────────────────────────────────────────────────────────

/// A particle in sovereign 7D Heptagon space.
///
/// Represents one KAKI-identified entity undergoing kinetic analysis.
/// May be a SQL record particle, a file particle, or an AkkadianAOL script particle.
#[derive(Debug, Clone)]
pub struct KineticParticle {
    /// Current position in 7D Heptagon space — the particle's health profile
    pub position: Vec7D,

    /// Current velocity — rate of change of health per analysis cycle
    pub velocity: Vec7D,

    /// Inverse mass in 7D space — `1.0 / mass`.
    /// Use `0.0` for immovable (sovereign-sealed) particles.
    pub inverse_mass: f64,

    /// Health decay coefficient per cycle — in `(0.0, 1.0]`.
    /// `1.0` = no decay. `0.95` = typical active SQL workload.
    pub damping: f64,
}

impl KineticParticle {
    /// Construct with explicit mass (converted internally to inverse_mass).
    /// `mass = 0.0` produces an immovable particle (inverse_mass = 0.0).
    pub fn new(position: Vec7D, mass: f64, damping: f64) -> Self {
        let inverse_mass = if mass > f64::EPSILON { 1.0 / mass } else { 0.0 };
        Self {
            position,
            velocity: Vec7D::ZERO,
            inverse_mass,
            damping: damping.clamp(0.0, 1.0),
        }
    }

    /// Construct a sovereign-sealed (immovable) particle.
    /// No force can move it — its health is permanently fixed at its position.
    pub fn sovereign(position: Vec7D) -> Self {
        Self {
            position,
            velocity: Vec7D::ZERO,
            inverse_mass: 0.0,
            damping: 1.0,
        }
    }

    /// Construct with inverse_mass directly (Millington-style).
    pub fn with_inverse_mass(position: Vec7D, inverse_mass: f64, damping: f64) -> Self {
        Self {
            position,
            velocity: Vec7D::ZERO,
            inverse_mass: inverse_mass.max(0.0),
            damping: damping.clamp(0.0, 1.0),
        }
    }

    /// True if this particle has infinite mass — no force can move it.
    pub fn is_sovereign(&self) -> bool {
        self.inverse_mass < f64::EPSILON
    }

    /// True if this particle has fallen below the dead zone threshold.
    pub fn is_in_dead_zone(&self, threshold: f64) -> bool {
        let radius = self.position.orbital_radius(Vec7D::HEALTHY_ORBIT);
        radius > (1.0 - threshold) * 7.0_f64.sqrt()
    }

    /// Health score in `[0.0, 1.0]` — distance from healthy orbit.
    pub fn health_score(&self) -> f64 {
        self.position.health_score(Vec7D::HEALTHY_ORBIT)
    }

    /// Sigma (σ) — uncertainty collapse in `[0.0, 1.0]`.
    pub fn sigma(&self) -> f64 {
        self.position.sigma(Vec7D::HEALTHY_ORBIT)
    }

    /// True if σ has collapsed below the sovereign threshold.
    pub fn is_sigma_sovereign(&self, sigma_threshold: f64) -> bool {
        self.sigma() < sigma_threshold
    }
}

// ── SovereignAccumulator ────────────────────────────────────────────────────

/// Collects all forces acting on a particle and integrates them
/// using the Euler method — Millington Chapter 2 applied to 7D space.
///
/// ```text
/// df/dt = I_phys + I_mem + I_learn
/// ```
pub struct SovereignAccumulator {
    forces:  Vec<Box<dyn ForceGenerator>>,
    pub dt: f64,
}

impl SovereignAccumulator {
    pub fn new(dt: f64) -> Self {
        Self {
            forces: Vec::new(),
            dt: dt.max(f64::EPSILON),
        }
    }

    pub fn default_cycle() -> Self {
        Self::new(1.0)
    }

    pub fn add_force(&mut self, force: Box<dyn ForceGenerator>) {
        self.forces.push(force);
    }

    pub fn clear_forces(&mut self) {
        self.forces.clear();
    }

    pub fn force_count(&self) -> usize {
        self.forces.len()
    }

    pub fn total_force(&self, position: &Vec7D) -> Vec7D {
        self.forces
            .iter()
            .fold(Vec7D::ZERO, |acc, f| acc + f.generate(position))
    }

    /// **Integrate** — advance the particle by one Heptagon analysis cycle.
    ///
    /// ```text
    /// F = sum(all forces)
    /// a = F * inverse_mass
    /// v += a * dt
    /// v *= damping^dt
    /// x += v * dt
    /// ```
    pub fn integrate(&self, particle: &mut KineticParticle) {
        if particle.is_sovereign() {
            return;
        }

        let total_force   = self.total_force(&particle.position);
        let acceleration  = total_force * particle.inverse_mass;
        particle.velocity += acceleration * self.dt;

        let decay = particle.damping.powf(self.dt);
        particle.velocity *= decay;

        particle.position += particle.velocity * self.dt;
    }

    pub fn integrate_cycles(&self, particle: &mut KineticParticle, cycles: u32) {
        for _ in 0..cycles {
            self.integrate(particle);
        }
    }

    /// Compute the rate of health change per cycle (dH/dt).
    /// Negative → health declining (Self-Immune trigger).
    pub fn health_velocity(&self, particle: &KineticParticle) -> f64 {
        let mut probe = particle.clone();
        let before = probe.health_score();
        self.integrate(&mut probe);
        probe.health_score() - before
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::force::{ConstantForce, HeptaDimension, LearningForce, PhysicalForce};

    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn new_particle_has_zero_velocity() {
        let p = KineticParticle::new(Vec7D::ZERO, 1.0, 0.99);
        assert_eq!(p.velocity, Vec7D::ZERO);
    }

    #[test]
    fn zero_mass_produces_immovable_particle() {
        let p = KineticParticle::new(Vec7D::ZERO, 0.0, 1.0);
        assert!(p.is_sovereign());
        assert!(approx(p.inverse_mass, 0.0));
    }

    #[test]
    fn sovereign_particle_is_immovable() {
        let p = KineticParticle::sovereign(Vec7D::HEALTHY_ORBIT);
        assert!(p.is_sovereign());
    }

    #[test]
    fn mass_converts_to_correct_inverse_mass() {
        let p = KineticParticle::new(Vec7D::ZERO, 2.0, 1.0);
        assert!(approx(p.inverse_mass, 0.5));
    }

    #[test]
    fn particle_on_healthy_orbit_has_maximum_health_score() {
        let p = KineticParticle::new(Vec7D::HEALTHY_ORBIT, 1.0, 1.0);
        assert!(approx(p.health_score(), 1.0));
    }

    #[test]
    fn particle_at_zero_has_reduced_health_score() {
        let p = KineticParticle::new(Vec7D::ZERO, 1.0, 1.0);
        assert!(p.health_score() < 0.5, "health={}", p.health_score());
    }

    #[test]
    fn sovereign_particle_has_minimum_sigma() {
        let p = KineticParticle::sovereign(Vec7D::HEALTHY_ORBIT);
        assert!(approx(p.sigma(), 0.0));
    }

    #[test]
    fn sigma_threshold_detection() {
        let p = KineticParticle::sovereign(Vec7D::HEALTHY_ORBIT);
        assert!(p.is_sigma_sovereign(0.05));
    }

    #[test]
    fn immovable_particle_not_moved_by_any_force() {
        let mut p = KineticParticle::sovereign(Vec7D::HEALTHY_ORBIT);
        let initial = p.position;
        let mut acc = SovereignAccumulator::default_cycle();
        acc.add_force(Box::new(ConstantForce::new(Vec7D::UNIT, HeptaDimension::ME)));
        acc.integrate(&mut p);
        assert_eq!(p.position, initial, "sovereign particle must not move");
    }

    #[test]
    fn zero_force_produces_no_acceleration() {
        let mut p = KineticParticle::new(Vec7D::HEALTHY_ORBIT, 1.0, 1.0);
        let acc = SovereignAccumulator::default_cycle();
        acc.integrate(&mut p);
        assert_eq!(p.position, Vec7D::HEALTHY_ORBIT);
    }

    #[test]
    fn positive_learning_force_moves_particle_toward_healthy_orbit() {
        let start = Vec7D::new(0.2, 0.2, 0.2, 0.2, 0.2, 0.2, 0.2);
        let mut p = KineticParticle::new(start, 1.0, 1.0);
        let mut acc = SovereignAccumulator::default_cycle();
        acc.add_force(Box::new(LearningForce::full()));
        acc.integrate(&mut p);
        assert!(p.position.me > start.me, "me should increase");
    }

    #[test]
    fn negative_physical_force_reduces_uru_dimension() {
        let mut p = KineticParticle::new(Vec7D::HEALTHY_ORBIT, 1.0, 1.0);
        let initial_uru = p.position.uru;
        let mut acc = SovereignAccumulator::default_cycle();
        acc.add_force(Box::new(PhysicalForce::new(0.38, 0.0)));
        acc.integrate(&mut p);
        assert!(p.position.uru < initial_uru,
            "uru should decrease under Gray-Rot: initial={initial_uru}, final={}", p.position.uru);
    }

    #[test]
    fn heavy_particle_accelerates_less_than_light_particle() {
        let pos   = Vec7D::new(0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5);
        let force = Vec7D::UNIT;
        let mut light = KineticParticle::new(pos, 1.0, 1.0);
        let mut heavy = KineticParticle::new(pos, 10.0, 1.0);

        let mut acc = SovereignAccumulator::default_cycle();
        acc.add_force(Box::new(ConstantForce::new(force, HeptaDimension::ME)));
        acc.integrate(&mut light);

        let mut acc2 = SovereignAccumulator::default_cycle();
        acc2.add_force(Box::new(ConstantForce::new(force, HeptaDimension::ME)));
        acc2.integrate(&mut heavy);

        assert!(light.velocity.me > heavy.velocity.me,
            "light particle should have higher velocity: light={}, heavy={}",
            light.velocity.me, heavy.velocity.me);
    }

    #[test]
    fn damping_reduces_velocity_over_cycles() {
        let mut p = KineticParticle::new(Vec7D::ZERO, 1.0, 0.90);
        p.velocity = Vec7D::new(1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let acc = SovereignAccumulator::default_cycle();
        acc.integrate(&mut p);
        let v_after_1 = p.velocity.me;
        acc.integrate(&mut p);
        let v_after_2 = p.velocity.me;
        assert!(v_after_2 < v_after_1,
            "velocity should decay: after_1={v_after_1}, after_2={v_after_2}");
        assert!(v_after_1 < 1.0, "initial velocity should already have decayed");
    }

    #[test]
    fn multiple_forces_accumulate_correctly() {
        let mut p = KineticParticle::new(Vec7D::ZERO, 1.0, 1.0);
        let f1 = Vec7D::new(1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let f2 = Vec7D::new(1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let mut acc = SovereignAccumulator::default_cycle();
        acc.add_force(Box::new(ConstantForce::new(f1, HeptaDimension::ME)));
        acc.add_force(Box::new(ConstantForce::new(f2, HeptaDimension::ME)));
        acc.integrate(&mut p);
        assert!(approx(p.velocity.me, 2.0),
            "accumulated forces should sum: velocity.me={}", p.velocity.me);
    }

    #[test]
    fn clear_forces_empties_accumulator() {
        let mut acc = SovereignAccumulator::default_cycle();
        acc.add_force(Box::new(ConstantForce::new(Vec7D::UNIT, HeptaDimension::ME)));
        assert_eq!(acc.force_count(), 1);
        acc.clear_forces();
        assert_eq!(acc.force_count(), 0);
    }

    #[test]
    fn health_velocity_positive_when_learning_force_active() {
        let start = Vec7D::new(0.3, 0.3, 0.3, 0.3, 0.3, 0.3, 0.3);
        let p = KineticParticle::new(start, 1.0, 1.0);
        let mut acc = SovereignAccumulator::default_cycle();
        acc.add_force(Box::new(LearningForce::full()));
        let dh = acc.health_velocity(&p);
        assert!(dh >= 0.0, "dH/dt should be >= 0 under pure learning force: {dh}");
    }

    #[test]
    fn integrate_cycles_advances_particle_multiple_steps() {
        let mut p = KineticParticle::new(Vec7D::ZERO, 1.0, 1.0);
        let force = Vec7D::new(0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let mut acc = SovereignAccumulator::default_cycle();
        acc.add_force(Box::new(ConstantForce::new(force, HeptaDimension::ME)));
        acc.integrate_cycles(&mut p, 5);
        assert!(p.position.me > 0.0,
            "position.me should be positive after 5 cycles: {}", p.position.me);
        assert!(p.velocity.me > 0.0,
            "velocity.me should be positive after 5 cycles: {}", p.velocity.me);
    }
}
