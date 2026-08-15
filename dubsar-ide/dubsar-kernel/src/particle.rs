use rand::prelude::*;
use serde::{Deserialize, Serialize};

use crate::projection::Projection7D3D;

/// 7D particle: position(3) + momentum(3) + scalar(1) = 28 bytes
/// Represents a data unit inside a storage block (storage fragmentation sim)
#[repr(C)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Particle7D {
    /// Spatial coordinates in storage address space
    pub position: [f32; 3],
    /// Velocity / migration direction
    pub momentum: [f32; 3],
    /// Fragmentation scalar: 0.0 = contiguous, 1.0 = fully fragmented
    pub scalar: f32,
    /// Block ID (tribe assignment: 0 = Tribe A, 1 = Tribe B, else free)
    pub tribe_id: u32,
}

/// Projected 3D point for rendering (output of Projection7D3D)
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Point3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    /// RGBA color packed as [0..1] floats
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

/// Manages a cloud of 7D particles on the CPU (VM-compatible, no GPU passthrough)
pub struct ParticleCloud {
    pub particles: Vec<Particle7D>,
    step_count: u64,
}

impl ParticleCloud {
    pub fn new(count: usize) -> Self {
        let mut rng = thread_rng();
        let particles = (0..count)
            .map(|i| Particle7D {
                position: [
                    rng.gen_range(-100.0..100.0),
                    rng.gen_range(-100.0..100.0),
                    rng.gen_range(-100.0..100.0),
                ],
                momentum: [
                    rng.gen_range(-1.0..1.0),
                    rng.gen_range(-1.0..1.0),
                    rng.gen_range(-0.1..0.1),
                ],
                scalar: rng.gen(),
                tribe_id: (i % 3) as u32,
            })
            .collect();
        Self { particles, step_count: 0 }
    }

    /// Initialise a storage fragmentation scenario:
    /// - Tribe A (id=0): tightly packed blocks on the left half (position.x < 0)
    /// - Tribe B (id=1): scattered blocks on the right half (position.x > 0)
    /// - Free (id=2): randomly distributed with high scalar (fragmented)
    pub fn new_storage_fragmentation(count: usize) -> Self {
        let mut rng = thread_rng();
        let tribe_a = count / 3;
        let tribe_b = count / 3;
        let free = count - tribe_a - tribe_b;

        let mut particles = Vec::with_capacity(count);

        // Tribe A — contiguous, low fragmentation
        for _ in 0..tribe_a {
            particles.push(Particle7D {
                position: [
                    rng.gen_range(-100.0..-10.0),
                    rng.gen_range(-50.0..50.0),
                    rng.gen_range(-5.0..5.0),
                ],
                momentum: [
                    rng.gen_range(-0.1..0.1),
                    rng.gen_range(-0.1..0.1),
                    0.0,
                ],
                scalar: rng.gen_range(0.0..0.2),
                tribe_id: 0,
            });
        }

        // Tribe B — scattered, high fragmentation
        for _ in 0..tribe_b {
            particles.push(Particle7D {
                position: [
                    rng.gen_range(10.0..100.0),
                    rng.gen_range(-100.0..100.0),
                    rng.gen_range(-20.0..20.0),
                ],
                momentum: [
                    rng.gen_range(-0.5..0.5),
                    rng.gen_range(-0.5..0.5),
                    rng.gen_range(-0.1..0.1),
                ],
                scalar: rng.gen_range(0.6..1.0),
                tribe_id: 1,
            });
        }

        // Free blocks — highest fragmentation, chaotic motion
        for _ in 0..free {
            particles.push(Particle7D {
                position: [
                    rng.gen_range(-100.0..100.0),
                    rng.gen_range(-100.0..100.0),
                    rng.gen_range(-50.0..50.0),
                ],
                momentum: [
                    rng.gen_range(-2.0..2.0),
                    rng.gen_range(-2.0..2.0),
                    rng.gen_range(-0.5..0.5),
                ],
                scalar: rng.gen_range(0.8..1.0),
                tribe_id: 2,
            });
        }

        Self { particles, step_count: 0 }
    }

    /// Advance simulation by `dt` seconds
    pub fn step(&mut self, dt: f32) {
        let gravity = 0.5_f32; // attraction between same-tribe particles

        // Simple O(N) per-particle update (no interaction, just motion + fragmentation)
        // Full Barnes-Hut / spatial hashing would be added for GPU path
        for p in &mut self.particles {
            // Gravity-like pull toward tribe centroid (simplified)
            let (cx, cy) = match p.tribe_id {
                0 => (-50.0_f32, 0.0_f32), // Tribe A centroid
                1 => (50.0_f32, 0.0_f32),  // Tribe B centroid
                _ => (0.0_f32, 0.0_f32),   // Free: attracted to origin
            };

            let dx = cx - p.position[0];
            let dy = cy - p.position[1];
            let dist = (dx * dx + dy * dy).sqrt().max(0.001);

            p.momentum[0] += gravity * dx / dist * dt;
            p.momentum[1] += gravity * dy / dist * dt;

            // Damping — free particles lose momentum faster (fragmentation settling)
            let damping = if p.tribe_id == 2 { 0.98 } else { 0.995 };
            p.momentum[0] *= damping;
            p.momentum[1] *= damping;
            p.momentum[2] *= damping;

            p.position[0] += p.momentum[0] * dt;
            p.position[1] += p.momentum[1] * dt;
            p.position[2] += p.momentum[2] * dt;

            // Fragmentation scalar decays as particles settle into tribes
            if p.tribe_id != 2 {
                p.scalar = (p.scalar - 0.001 * dt).max(0.0);
            }
        }

        self.step_count += 1;
    }

    /// Project all particles to 3D for rendering
    pub fn project(&self, proj: &Projection7D3D) -> Vec<Point3D> {
        self.particles
            .iter()
            .map(|p| {
                let [x, y, z] = proj.project(p);

                // Color by tribe: A=green, B=red, free=yellow
                let (r, g, b) = match p.tribe_id {
                    0 => (0.0, 0.8 + p.scalar * 0.2, 0.2 + p.scalar * 0.3), // green → yellow-green
                    1 => (0.8 + p.scalar * 0.2, 0.1, 0.1),                   // red → bright red
                    _ => (1.0, 0.9 - p.scalar * 0.4, 0.0),                   // orange-yellow
                };

                Point3D { x, y, z, r, g, b, a: 0.7 - p.scalar * 0.2 }
            })
            .collect()
    }

    pub fn active_count(&self) -> usize {
        self.particles.len()
    }

    pub fn step_count(&self) -> u64 {
        self.step_count
    }

    /// Returns fragmentation statistics for analysis cells
    pub fn fragmentation_stats(&self) -> FragmentationStats {
        let tribe_a: Vec<_> = self.particles.iter().filter(|p| p.tribe_id == 0).collect();
        let tribe_b: Vec<_> = self.particles.iter().filter(|p| p.tribe_id == 1).collect();
        let free: Vec<_> = self.particles.iter().filter(|p| p.tribe_id == 2).collect();

        let mean_scalar = |v: &[&Particle7D]| -> f32 {
            if v.is_empty() { return 0.0; }
            v.iter().map(|p| p.scalar).sum::<f32>() / v.len() as f32
        };

        FragmentationStats {
            tribe_a_count: tribe_a.len(),
            tribe_b_count: tribe_b.len(),
            free_count: free.len(),
            tribe_a_fragmentation: mean_scalar(&tribe_a),
            tribe_b_fragmentation: mean_scalar(&tribe_b),
            step: self.step_count,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct FragmentationStats {
    pub tribe_a_count: usize,
    pub tribe_b_count: usize,
    pub free_count: usize,
    pub tribe_a_fragmentation: f32,
    pub tribe_b_fragmentation: f32,
    pub step: u64,
}
