//! misru — GL-LBR-001-A2 (The Misru Rite): contested boundaries, where
//! jugglers are born, and the conditional inevitability of shape
//! collapse. PB-340. Pure Rust, zero dependencies.
#![forbid(unsafe_code)]

struct Xorshift64 {
    state: u64,
}

impl Xorshift64 {
    fn new(seed: u64) -> Self {
        Self { state: if seed == 0 { 0x9E3779B97F4A7C15 } else { seed } }
    }
    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }
    fn next_unit(&mut self) -> f64 {
        ((self.next_u64() >> 11) as f64) / ((1u64 << 53) as f64)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Particle {
    pub id: u64,
    pub position: f64,
}

/// The contested band, computed from the confrontation (Two Scribes Rite
/// III) -- never asserted (§3, step 2).
#[derive(Debug, Clone, Copy)]
pub struct ContestedBand {
    pub lo: f64,
    pub hi: f64,
}

impl ContestedBand {
    pub fn contains(&self, x: f64) -> bool {
        x >= self.lo && x <= self.hi
    }

    pub fn width(&self) -> f64 {
        self.hi - self.lo
    }
}

/// Birth clause (L37): a particle enters the nabalkutu register iff it
/// lies in the contested band -- jugglers are born on the misru, nowhere
/// else. Localization is exact.
pub fn nabalkutu_register(particles: &[Particle], band: &ContestedBand) -> Vec<u64> {
    particles.iter().filter(|p| band.contains(p.position)).map(|p| p.id).collect()
}

/// Simulate one seed's rehearsal: contested particles alternate between
/// rule-homes, costing coherence on each flap. The no-decay clause is
/// structural: nothing outside this flap path ever touches `coherence`
/// -- uncontested moorings stay at θ₀ by construction, not by assertion.
/// Returns the tick at which the shape's classification flips (Shape
/// Horizon T_shape), or `None` if it holds through the window.
pub fn simulate_shape_horizon(
    juggler_fraction: f64,
    max_ticks: usize,
    decree_issued: bool,
    decay_per_flap: f64,
    coherence_threshold: f64,
    seed: u64,
) -> Option<usize> {
    let mut rng = Xorshift64::new(seed);
    let mut coherence = 1.0f64;
    for t in 0..max_ticks {
        if !decree_issued && rng.next_unit() < juggler_fraction {
            coherence -= decay_per_flap;
        }
        if coherence < coherence_threshold {
            return Some(t);
        }
    }
    None
}

#[derive(Debug, Clone)]
pub struct EnsembleResult {
    pub median_t_shape: Option<usize>,
    pub held_through_window: usize,
    pub n_seeds: usize,
}

/// Ensemble: many seeds -> the T_shape distribution, median and the
/// held-through-window count (§3, step 4-5).
pub fn ensemble(
    juggler_fraction: f64,
    max_ticks: usize,
    decree_issued: bool,
    decay_per_flap: f64,
    coherence_threshold: f64,
    n_seeds: usize,
) -> EnsembleResult {
    let mut t_shapes: Vec<usize> = Vec::new();
    let mut held = 0usize;
    for seed in 1..=(n_seeds as u64) {
        match simulate_shape_horizon(
            juggler_fraction,
            max_ticks,
            decree_issued,
            decay_per_flap,
            coherence_threshold,
            seed,
        ) {
            Some(t) => t_shapes.push(t),
            None => held += 1,
        }
    }
    t_shapes.sort_unstable();
    let median = if t_shapes.is_empty() {
        None
    } else {
        Some(t_shapes[t_shapes.len() / 2])
    };
    EnsembleResult { median_t_shape: median, held_through_window: held, n_seeds }
}

#[cfg(test)]
mod tests {
    use super::*;

    // L37 — birth clause: localization is exact.
    #[test]
    fn l37_birth_clause_exact_localization() {
        let band = ContestedBand { lo: 0.4, hi: 0.6 };
        let particles = vec![
            Particle { id: 1, position: 0.1 },
            Particle { id: 2, position: 0.5 },
            Particle { id: 3, position: 0.45 },
            Particle { id: 4, position: 0.9 },
        ];
        let jugglers = nabalkutu_register(&particles, &band);
        assert_eq!(jugglers, vec![2, 3], "only particles inside the band are jugglers");
    }

    // L38 — without a harmonization decree, T_shape is finite for every
    // seed (ensemble test).
    #[test]
    fn l38_conditional_inevitability_without_decree() {
        let result = ensemble(1.0, 500, false, 0.05, 0.5, 30);
        assert_eq!(result.held_through_window, 0, "every seed must collapse without the decree");
        assert!(result.median_t_shape.is_some());
    }

    // L39 — with the decree, the flapping ceases and the shape holds
    // through the window.
    #[test]
    fn l39_decree_holds_shape() {
        let result = ensemble(1.0, 500, true, 0.05, 0.5, 30);
        assert_eq!(result.held_through_window, result.n_seeds, "decree must hold every seed");
        assert!(result.median_t_shape.is_none());
    }

    // L40 — monotonicity: a wider contested band (higher juggler
    // fraction) brings an earlier median T_shape.
    #[test]
    fn l40_monotone_in_contested_fraction() {
        let narrow = ensemble(0.2, 2000, false, 0.02, 0.5, 40);
        let wide = ensemble(0.8, 2000, false, 0.02, 0.5, 40);
        let narrow_median = narrow.median_t_shape.unwrap_or(usize::MAX);
        let wide_median = wide.median_t_shape.unwrap_or(usize::MAX);
        assert!(
            wide_median <= narrow_median,
            "wider contested band ({wide_median}) must not collapse later than a narrow one ({narrow_median})"
        );
    }

    // No-decay clause: an empty juggler fraction never touches coherence,
    // so the shape holds through the window by construction.
    #[test]
    fn no_decay_clause_zero_jugglers_never_decay() {
        let result = ensemble(0.0, 1000, false, 0.05, 0.5, 10);
        assert_eq!(result.held_through_window, result.n_seeds);
    }
}
