//! # kinetic-engine Integration Tests
//!
//! These tests verify the full sovereign kinetic pipeline from the
//! perspective of an external crate consuming kinetic-engine.
//!
//! They test the complete journey:
//!
//! ```text
//! Forces (I_phys + I_mem + I_learn)
//!     → SovereignAccumulator
//!         → KineticParticle integration
//!             → ScoringEngine classification
//!                 → HealthClassification (Sovereign / Warning / Critical)
//! ```
//!
//! Each test scenario corresponds to a real DMW analytical case.
//! DUB.SAR 𒁾

use kinetic_engine::{
    ConstantForce, HealthClassification, HeptaDimension, KineticParticle, LearningForce,
    MemoryForce, PhysicalForce, ScoringEngine, SovereignAccumulator, Vec7D,
};

#[test]
fn healthy_particle_with_no_forces_remains_sovereign() {
    let particle = KineticParticle::new(Vec7D::HEALTHY_ORBIT, 1.0, 1.0);
    let engine = ScoringEngine::dmw_default();
    assert_eq!(
        engine.classify(&particle.position),
        HealthClassification::Sovereign
    );
    assert!(engine.is_sovereign(&particle.position));
}

#[test]
fn sick_sql_particle_classifies_as_critical_without_oracle() {
    let degraded = Vec7D::new(0.2, 0.2, 0.2, 0.2, 0.2, 0.2, 0.2);
    let particle = KineticParticle::new(degraded, 1.0 / 6069.0, 0.95);
    let engine = ScoringEngine::dmw_default();
    assert_eq!(
        engine.classify(&particle.position),
        HealthClassification::Critical
    );
}

#[test]
fn oracle_learning_force_moves_sick_particle_toward_healthy_orbit() {
    let degraded = Vec7D::new(0.2, 0.2, 0.2, 0.2, 0.2, 0.2, 0.2);
    let mut particle = KineticParticle::new(degraded, 6069.0, 0.95);
    let initial_health = particle.health_score();

    let mut acc = SovereignAccumulator::default_cycle();
    acc.add_force(Box::new(LearningForce::full()));
    acc.integrate_cycles(&mut particle, 3);

    let final_health = particle.health_score();
    assert!(
        final_health > initial_health,
        "learning force must improve health score: before={initial_health:.4}, after={final_health:.4}"
    );
}

#[test]
fn gray_rot_force_degrades_particle_over_cycles() {
    let mut particle = KineticParticle::new(Vec7D::HEALTHY_ORBIT, 1.0, 0.95);
    let initial_health = particle.health_score();

    let mut acc = SovereignAccumulator::default_cycle();
    acc.add_force(Box::new(PhysicalForce::new(0.38, 0.95)));
    acc.integrate_cycles(&mut particle, 5);

    let degraded_health = particle.health_score();
    assert!(
        degraded_health < initial_health,
        "Gray-Rot force must degrade health: before={initial_health:.4}, after={degraded_health:.4}"
    );
}

#[test]
fn declining_trajectory_detected_before_threshold_crossed() {
    let slightly_degraded = Vec7D::new(0.85, 0.85, 0.85, 0.85, 0.85, 0.85, 0.85);
    let particle = KineticParticle::new(slightly_degraded, 1.0, 0.95);
    let engine = ScoringEngine::dmw_default();
    assert!(engine.is_sovereign(&particle.position));

    let mut acc = SovereignAccumulator::default_cycle();
    acc.add_force(Box::new(PhysicalForce::new(0.38, 0.50)));
    let dh_dt = acc.health_velocity(&particle);
    assert!(
        dh_dt < 0.0,
        "health_velocity must be negative under Gray-Rot: dH/dt={dh_dt:.6}"
    );
}

#[test]
fn sovereign_sealed_particle_unmoved_by_all_force_types() {
    let mut particle = KineticParticle::sovereign(Vec7D::HEALTHY_ORBIT);
    let sealed_pos = particle.position;

    let mut acc = SovereignAccumulator::default_cycle();
    acc.add_force(Box::new(PhysicalForce::new(1.0, 1.0)));
    acc.add_force(Box::new(MemoryForce::new(1.0, 1.0)));
    acc.add_force(Box::new(LearningForce::full()));
    acc.add_force(Box::new(ConstantForce::new(
        Vec7D::new(99.0, 99.0, 99.0, 99.0, 99.0, 99.0, 99.0),
        HeptaDimension::ME,
    )));
    acc.integrate_cycles(&mut particle, 10);

    assert_eq!(
        particle.position, sealed_pos,
        "sovereign-sealed particle must be completely unmoved by any force"
    );
}

#[test]
fn dominant_failure_routes_to_uru_for_index_fragmentation() {
    let particle = Vec7D::new(1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.0);
    let engine = ScoringEngine::dmw_default();
    let dominant = engine.dominant_failure(&particle);
    assert_eq!(dominant, Some(HeptaDimension::URU));
}

#[test]
fn dominant_failure_routes_to_gu_for_stale_statistics() {
    let particle = Vec7D::new(1.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0);
    let engine = ScoringEngine::dmw_default();
    let dominant = engine.dominant_failure(&particle);
    assert_eq!(dominant, Some(HeptaDimension::GU));
}

#[test]
fn sigma_collapses_toward_zero_under_balanced_force() {
    let unbalanced = Vec7D::new(1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    let particle = KineticParticle::new(unbalanced, 1.0, 1.0);
    let sigma_initial = particle.sigma();
    assert!(
        sigma_initial > 0.25,
        "directionally unbalanced start must have high sigma: {sigma_initial:.4}"
    );

    let mut after = particle.clone();
    let mut acc = SovereignAccumulator::default_cycle();
    acc.add_force(Box::new(ConstantForce::new(
        Vec7D::UNIT,
        HeptaDimension::ME,
    )));
    acc.integrate(&mut after);

    let sigma_final = after.sigma();
    assert!(sigma_final < sigma_initial,
        "σ must collapse under balanced 7D force: initial={sigma_initial:.4}, final={sigma_final:.4}");
}

#[test]
fn particle_unit_invariant_mass_encodes_cardinality_not_schema_width() {
    let cardinality = 6069.0_f64;
    let force = Vec7D::new(1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);

    let mut wide_particle = KineticParticle::new(Vec7D::ZERO, cardinality, 1.0);
    let mut narrow_particle = KineticParticle::new(Vec7D::ZERO, cardinality, 1.0);

    let mut acc = SovereignAccumulator::default_cycle();
    acc.add_force(Box::new(ConstantForce::new(force, HeptaDimension::ME)));
    acc.integrate(&mut wide_particle);

    let mut acc2 = SovereignAccumulator::default_cycle();
    acc2.add_force(Box::new(ConstantForce::new(force, HeptaDimension::ME)));
    acc2.integrate(&mut narrow_particle);

    assert_eq!(
        wide_particle.position, narrow_particle.position,
        "equal cardinality means equal PU — schema width does not affect kinetics"
    );
}

#[test]
fn threshold_profiles_classify_same_particle_differently() {
    let relaxed = ScoringEngine::new(0.50);
    let standard = ScoringEngine::dmw_default();
    let strict = ScoringEngine::strict();
    let sovereign = ScoringEngine::sovereign();

    assert!(relaxed.threshold < standard.threshold);
    assert!(standard.threshold < strict.threshold);
    assert!(strict.threshold < sovereign.threshold);
}

#[test]
fn full_kinetic_pipeline_end_to_end() {
    let raw_position = Vec7D::new(0.3, 0.2, 0.4, 0.1, 0.3, 0.2, 0.1);
    let mut particle = KineticParticle::new(raw_position, 6069.0, 0.95);

    let engine = ScoringEngine::dmw_default();
    assert_eq!(
        engine.classify(&particle.position),
        HealthClassification::Critical
    );

    let health_before = particle.health_score();

    let mut acc = SovereignAccumulator::default_cycle();
    acc.add_force(Box::new(PhysicalForce::new(0.10, 0.20)));
    acc.add_force(Box::new(MemoryForce::new(0.05, 0.0)));
    acc.add_force(Box::new(LearningForce::full()));
    acc.integrate_cycles(&mut particle, 5);

    let health_after = particle.health_score();
    assert!(
        health_after > health_before,
        "after Oracle fixes, distance-based health must improve: before={health_before:.4}, after={health_after:.4}"
    );

    let dh_dt = acc.health_velocity(&particle);
    assert!(
        dh_dt >= 0.0,
        "after Oracle fixes, trajectory must be improving: dH/dt={dh_dt:.6}"
    );
}
