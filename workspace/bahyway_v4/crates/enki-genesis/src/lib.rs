#![forbid(unsafe_code)]
//! # ENKI-GENESIS — Synthetic Particle Seeder
//!
//! Generates synthetic indoor movement patterns using Monte Carlo simulation
//! with the Social Force model.  Produces `GaCluster`s for NISABA without
//! requiring any prior real-world observations.
//!
//! ## Simulation parameters (GEN-PARAMS-001):
//! - Iterations:   1 000 Monte Carlo runs
//! - Agents:       100 per run
//! - Steps:        1 000 per run (time_step = 0.01s → 10s total)
//! - attraction_coeff: 2.0
//! - repulsion_coeff:  5.0
//! - wall_repulsion:   10.0
//! - neighbor_radius:  2.0m
//! - wall_radius:      1.0m
//! - damping:          0.95
//!
//! ## RNG: XorShift64 (no_std, no external crates)
//! Seed: `0xDEAD_BEEF_CAFE_BABE` XOR `orbital_seed`.

pub mod rng;
pub mod simulation;
pub mod social_force;

pub use rng::XorShift64;
pub use simulation::{GenesisSimulation, SimulationResult};
pub use social_force::{Agent2D, SocialForceParams};
