//! Nergal (GL-NRG-001): the plague-lord as control rod.
//! Discrete-event MC on a sector graph. k_eff decides invasion;
//! quarantine is shielding; scanning is importance-sampled.
#![forbid(unsafe_code)]

pub struct Rng(u64);
impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng(seed.wrapping_mul(0x9E3779B97F4A7C15) | 1)
    }
    pub fn uniform(&mut self) -> f64 {
        // SplitMix64
        self.0 = self.0.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        ((z ^ (z >> 31)) >> 11) as f64 / (1u64 << 53) as f64
    }
}

/// A sector's material: probabilities per collision (sum <= 1;
/// remainder = scatter). fission spawns `nu` new walkers.
#[derive(Clone, Copy)]
pub struct Material {
    pub p_absorb: f64,  // dead ends + Nergal scan-kill
    pub p_fission: f64, // dropper fan-out
    pub nu: usize,
}

/// Sector graph: adjacency (copy targets) + per-sector material.
pub struct Mesh {
    pub adj: Vec<Vec<usize>>,
    pub mat: Vec<Material>,
}

/// One outbreak from `seed`: returns total collisions until the
/// chain dies, capped (cap hit = "invasion" in tests).
pub fn outbreak(mesh: &Mesh, seed_sector: usize, rng: &mut Rng, cap: usize) -> usize {
    let mut frontier = vec![seed_sector];
    let mut events = 0usize;
    while let Some(s) = frontier.pop() {
        events += 1;
        if events >= cap {
            return events;
        }
        let m = mesh.mat[s];
        let u = rng.uniform();
        if u < m.p_absorb {
            continue;
        } // absorbed
        let targets = &mesh.adj[s];
        if targets.is_empty() {
            continue;
        }
        if u < m.p_absorb + m.p_fission {
            for _ in 0..m.nu {
                // fission
                let t = targets[(rng.uniform() * targets.len() as f64) as usize % targets.len()];
                frontier.push(t);
            }
        } else {
            // scatter
            let t = targets[(rng.uniform() * targets.len() as f64) as usize % targets.len()];
            frontier.push(t);
        }
    }
    events
}

/// k_eff estimate: mean offspring per collision (analog estimator).
pub fn k_eff(mat: &Material) -> f64 {
    // scatter keeps 1 walker; fission keeps nu; absorb keeps 0
    let p_scatter = 1.0 - mat.p_absorb - mat.p_fission;
    p_scatter * 1.0 + mat.p_fission * mat.nu as f64
}

/// Shielding: barrier of `d` layers each with kill prob per layer
/// derived from Sigma_t; transmission = product of survivals.
pub fn barrier_transmission(sigma_t: f64, d: usize, trials: usize, rng: &mut Rng) -> f64 {
    let p_survive_layer = (-sigma_t).exp();
    let mut through = 0usize;
    for _ in 0..trials {
        let mut alive = true;
        for _ in 0..d {
            if rng.uniform() >= p_survive_layer {
                alive = false;
                break;
            }
        }
        if alive {
            through += 1;
        }
    }
    through as f64 / trials as f64
}

/// Importance-weighted scanning: scan sectors in order of prior
/// risk weight; return scans needed to find all `infected`.
pub fn scans_to_find(order: &[usize], infected: &[usize]) -> usize {
    let mut remaining: Vec<usize> = infected.to_vec();
    for (n, s) in order.iter().enumerate() {
        remaining.retain(|x| x != s);
        if remaining.is_empty() {
            return n + 1;
        }
    }
    order.len()
}
