//! Minimal sovereign complex number for phasor arithmetic.

use fdd_core::Quantity;

#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub struct Phasor {
    pub re: f64,
    pub im: f64,
}

impl Phasor {
    pub fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }
    pub fn from_polar(mag: f64, angle_rad: f64) -> Self {
        Self {
            re: mag * angle_rad.cos(),
            im: mag * angle_rad.sin(),
        }
    }
    // Named to match this crate's own dot-call convention throughout
    // (state_estimate.rs etc. chain .mul()/.div() extensively) rather
    // than std::ops::Mul/Div, which would require renaming every call
    // site or accepting an inherent method that permanently shadows
    // the trait's -- a real API decision, not a mechanical lint fix.
    #[allow(clippy::should_implement_trait)]
    pub fn mul(self, o: Self) -> Self {
        Self {
            re: self.re * o.re - self.im * o.im,
            im: self.re * o.im + self.im * o.re,
        }
    }
    /// Complex division. Like f64 division, yields NaN/Inf components
    /// for a zero divisor rather than panicking -- callers that need
    /// to reject a near-zero divisor (e.g. a Gaussian-elimination
    /// pivot) must check `o.magnitude()` themselves before calling.
    #[allow(clippy::should_implement_trait)] // same reasoning as mul() above
    pub fn div(self, o: Self) -> Self {
        let d = o.re * o.re + o.im * o.im;
        Self {
            re: (self.re * o.re + self.im * o.im) / d,
            im: (self.im * o.re - self.re * o.im) / d,
        }
    }
    pub fn conj(self) -> Self {
        Self {
            re: self.re,
            im: -self.im,
        }
    }
    pub fn angle(self) -> f64 {
        self.im.atan2(self.re)
    }
}

impl Quantity for Phasor {
    fn zero() -> Self {
        Self::default()
    }
    fn add(self, o: Self) -> Self {
        Self {
            re: self.re + o.re,
            im: self.im + o.im,
        }
    }
    fn sub(self, o: Self) -> Self {
        Self {
            re: self.re - o.re,
            im: self.im - o.im,
        }
    }
    fn magnitude(self) -> f64 {
        (self.re * self.re + self.im * self.im).sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multiplication_matches_polar_magnitude_and_angle_sum() {
        let a = Phasor::from_polar(2.0, 0.3);
        let b = Phasor::from_polar(3.0, 0.5);
        let p = a.mul(b);
        assert!((p.magnitude() - 6.0).abs() < 1e-9);
        assert!((p.angle() - 0.8).abs() < 1e-9);
    }

    #[test]
    fn conjugate_negates_imaginary_part_only() {
        let a = Phasor::new(3.0, -4.0);
        let c = a.conj();
        assert_eq!(c, Phasor::new(3.0, 4.0));
    }

    #[test]
    fn division_is_the_inverse_of_multiplication() {
        let a = Phasor::from_polar(6.0, 0.8);
        let b = Phasor::from_polar(3.0, 0.5);
        let q = a.div(b);
        assert!((q.magnitude() - 2.0).abs() < 1e-9);
        assert!((q.angle() - 0.3).abs() < 1e-9);
        let back = q.mul(b);
        assert!((back.re - a.re).abs() < 1e-9);
        assert!((back.im - a.im).abs() < 1e-9);
    }
}
