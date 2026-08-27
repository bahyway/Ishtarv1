//! Rotors — daily snapshot evolution in EnkiDW.
//! A rotor R = cos(θ/2) - sin(θ/2)·B rotates a state in a bivector
//! plane B by angle θ. The rotor ANGLE measures total change:
//! big angle = anomaly. Partition by angle band, not calendar.
//! Simplified 2-blade rotor model (scalar + one bivector plane).
#![allow(clippy::needless_range_loop)]

/// A rotor in a single bivector plane: R = a + b·(e_i∧e_j).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rotor {
    pub scalar: f64,   // cos(θ/2)
    pub bivector: f64, // -sin(θ/2) magnitude in plane
}

impl Rotor {
    /// Identity rotor (no change).
    pub fn identity() -> Self {
        Rotor {
            scalar: 1.0,
            bivector: 0.0,
        }
    }

    /// Rotor for a rotation of `theta` radians in the plane.
    pub fn from_angle(theta: f64) -> Self {
        Rotor {
            scalar: (theta / 2.0).cos(),
            bivector: -(theta / 2.0).sin(),
        }
    }

    /// Recover the rotation angle θ from the rotor.
    pub fn angle(&self) -> f64 {
        2.0 * self.scalar.clamp(-1.0, 1.0).acos()
    }

    /// Rotor magnitude (should be 1 for a valid rotation).
    pub fn norm(&self) -> f64 {
        (self.scalar * self.scalar + self.bivector * self.bivector).sqrt()
    }

    /// Reverse (inverse for unit rotor): R⁻¹ = a - b·B.
    pub fn reverse(&self) -> Rotor {
        Rotor {
            scalar: self.scalar,
            bivector: -self.bivector,
        }
    }

    /// Compose two daily rotors (rotor product in the single plane).
    /// (a1 + b1 B)(a2 + b2 B) with B² = -1.
    pub fn compose(&self, next: &Rotor) -> Rotor {
        Rotor {
            scalar: self.scalar * next.scalar - self.bivector * next.bivector,
            bivector: self.scalar * next.bivector + self.bivector * next.scalar,
        }
    }

    /// Is this change anomalous? (angle above threshold radians)
    pub fn is_anomaly(&self, threshold_rad: f64) -> bool {
        self.angle() > threshold_rad
    }
}

/// A journal of daily rotors — the EnkiDW rotor_journal.
#[derive(Debug, Clone, Default)]
pub struct RotorJournal {
    pub daily: Vec<Rotor>,
}

impl RotorJournal {
    pub fn new() -> Self {
        RotorJournal { daily: Vec::new() }
    }

    pub fn record(&mut self, r: Rotor) {
        self.daily.push(r);
    }

    /// Net evolution rotor from day 0 to now (ordered composition).
    pub fn net(&self) -> Rotor {
        self.daily
            .iter()
            .fold(Rotor::identity(), |acc, r| acc.compose(r))
    }

    /// Days whose change exceeded the anomaly threshold.
    pub fn anomalies(&self, threshold_rad: f64) -> Vec<usize> {
        self.daily
            .iter()
            .enumerate()
            .filter(|(_, r)| r.is_anomaly(threshold_rad))
            .map(|(i, _)| i)
            .collect()
    }

    /// Rotor-sector partition key: which angle-band this day falls in.
    /// Bands of `band_rad` radians — groups days by change magnitude.
    pub fn sector_of(day: usize, journal: &RotorJournal, band_rad: f64) -> u32 {
        let a = journal.daily[day].angle();
        (a / band_rad).floor() as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn identity_has_zero_angle() {
        assert!(Rotor::identity().angle().abs() < 1e-12);
    }

    #[test]
    fn rotor_is_unit_norm() {
        let r = Rotor::from_angle(PI / 3.0);
        assert!((r.norm() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn angle_roundtrip() {
        let r = Rotor::from_angle(PI / 2.0);
        assert!((r.angle() - PI / 2.0).abs() < 1e-9);
    }

    #[test]
    fn reverse_undoes_rotation() {
        let r = Rotor::from_angle(1.1);
        let net = r.compose(&r.reverse());
        assert!(net.angle().abs() < 1e-9, "R·R⁻¹ = identity");
    }

    #[test]
    fn journal_net_composes_days() {
        let mut j = RotorJournal::new();
        j.record(Rotor::from_angle(0.3));
        j.record(Rotor::from_angle(0.4));
        // small rotations in one plane add
        assert!((j.net().angle() - 0.7).abs() < 1e-9);
    }

    #[test]
    fn anomaly_detection_flags_large_change() {
        let mut j = RotorJournal::new();
        j.record(Rotor::from_angle(0.1)); // calm
        j.record(Rotor::from_angle(1.5)); // spike
        let flagged = j.anomalies(1.0);
        assert_eq!(flagged, vec![1]);
    }

    #[test]
    fn rotor_sector_partitions_by_change_not_calendar() {
        let mut j = RotorJournal::new();
        j.record(Rotor::from_angle(0.2)); // sector 0 at band 0.5
        j.record(Rotor::from_angle(1.2)); // sector 2
        assert_eq!(RotorJournal::sector_of(0, &j, 0.5), 0);
        assert_eq!(RotorJournal::sector_of(1, &j, 0.5), 2);
    }
}
