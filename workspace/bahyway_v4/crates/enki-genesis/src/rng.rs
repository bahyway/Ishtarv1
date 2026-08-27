#![forbid(unsafe_code)]
//! XorShift64 — deterministic no_std PRNG for Monte Carlo simulation.
//! Period: 2^64 - 1.  State: 8 bytes.  Cost: ~3 CPU cycles per call.

/// XorShift64 PRNG seeded from a u64 base (must not be zero).
pub struct XorShift64 {
    state: u64,
}

impl XorShift64 {
    const DEFAULT_SEED: u64 = 0xDEAD_BEEF_CAFE_BABE;

    /// Create a seeded RNG.  If `seed` is 0, the default seed is used.
    pub fn new(seed: u64) -> Self {
        let state = if seed == 0 { Self::DEFAULT_SEED } else { seed };
        Self { state }
    }

    /// Create with the default seed XOR'd with an orbital counter.
    pub fn from_orbital(orbital: u64) -> Self {
        Self::new(Self::DEFAULT_SEED ^ orbital)
    }

    /// Generate next u64.
    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    /// Generate a usize in [0, max).
    #[inline]
    pub fn next_usize(&mut self, max: usize) -> usize {
        if max == 0 {
            return 0;
        }
        (self.next_u64() % max as u64) as usize
    }

    /// Generate a float in [0.0, 1.0).
    #[inline]
    pub fn next_f64(&mut self) -> f64 {
        // Use 53 bits of mantissa
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Generate a float in [lo, hi).
    #[inline]
    pub fn next_f64_range(&mut self, lo: f64, hi: f64) -> f64 {
        lo + self.next_f64() * (hi - lo)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic() {
        let mut a = XorShift64::new(42);
        let mut b = XorShift64::new(42);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn non_zero_output() {
        let mut rng = XorShift64::new(1);
        // At least one of the next 100 values should be non-zero
        let all_zero = (0..100).all(|_| rng.next_u64() == 0);
        assert!(!all_zero);
    }

    #[test]
    fn f64_in_range() {
        let mut rng = XorShift64::new(99);
        for _ in 0..1_000 {
            let v = rng.next_f64();
            assert!((0.0..1.0).contains(&v));
        }
    }

    #[test]
    fn orbital_seed_differs() {
        let mut a = XorShift64::from_orbital(1);
        let mut b = XorShift64::from_orbital(2);
        assert_ne!(a.next_u64(), b.next_u64());
    }
}
