//! EnkiDW rotor-sector partitioning. Days/particles are grouped by
//! rotor-angle band (change magnitude), NOT calendar proximity.
//! Snapshot when accumulated rotor crosses the band. TRIPLE-O only.

/// A partition keyed by rotor-angle sector.
#[derive(Debug, Clone, PartialEq)]
pub struct RotorPartition {
    pub sector: u32,
    pub member_indices: Vec<usize>,
    pub accumulated_angle: f64,
}

/// Assign an angle to its sector given a band width (radians).
pub fn sector_for(angle: f64, band_rad: f64) -> u32 {
    assert!(band_rad > 0.0, "band must be positive");
    (angle / band_rad).floor() as u32
}

/// Partition a set of (index, angle) samples into rotor sectors.
pub fn partition(samples: &[(usize, f64)], band_rad: f64) -> Vec<RotorPartition> {
    use std::collections::BTreeMap;
    let mut map: BTreeMap<u32, RotorPartition> = BTreeMap::new();
    for (idx, angle) in samples {
        let sec = sector_for(*angle, band_rad);
        let entry = map.entry(sec).or_insert(RotorPartition {
            sector: sec,
            member_indices: Vec::new(),
            accumulated_angle: 0.0,
        });
        entry.member_indices.push(*idx);
        entry.accumulated_angle += *angle;
    }
    map.into_values().collect()
}

/// Should this partition be snapshotted? (accumulated crosses band)
pub fn needs_snapshot(p: &RotorPartition, band_rad: f64) -> bool {
    p.accumulated_angle >= band_rad
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sector_assignment_by_magnitude() {
        assert_eq!(sector_for(0.2, 0.5), 0);
        assert_eq!(sector_for(0.7, 0.5), 1);
        assert_eq!(sector_for(1.2, 0.5), 2);
    }

    #[test]
    fn partition_groups_by_change_not_calendar() {
        // indices 0 and 2 have similar SMALL change -> same sector,
        // even though index 1 (big change) sits between them in time.
        let samples = vec![(0, 0.1), (1, 1.3), (2, 0.15)];
        let parts = partition(&samples, 0.5);
        let sector0 = parts.iter().find(|p| p.sector == 0).unwrap();
        assert!(sector0.member_indices.contains(&0));
        assert!(sector0.member_indices.contains(&2));
        assert!(!sector0.member_indices.contains(&1));
    }

    #[test]
    fn snapshot_triggered_by_accumulation() {
        let p = RotorPartition {
            sector: 0,
            member_indices: vec![0, 1, 2],
            accumulated_angle: 1.2,
        };
        assert!(needs_snapshot(&p, 1.0));
        let q = RotorPartition {
            sector: 0,
            member_indices: vec![0],
            accumulated_angle: 0.3,
        };
        assert!(!needs_snapshot(&q, 1.0));
    }

    #[test]
    fn empty_samples_yield_no_partitions() {
        assert!(partition(&[], 0.5).is_empty());
    }
}
