//! Color drift computation for the three-layer ColorID model.
//!
//! Drift = Euclidean distance in RGB space between a particle's current
//! color_id_snapshot and its tribe's root color.
//!
//! Channel semantics:
//!   R = severity / alert level
//!   G = quality score
//!   B = freshness / time decay

/// Compute Euclidean distance between two RGB triples.
pub fn rgb_drift(a: [u8; 3], b: [u8; 3]) -> f32 {
    let dr = a[0] as f32 - b[0] as f32;
    let dg = a[1] as f32 - b[1] as f32;
    let db = a[2] as f32 - b[2] as f32;
    (dr * dr + dg * dg + db * db).sqrt()
}

/// Which color channel dominates the drift (highest absolute delta).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriftAxis {
    /// R channel dominant — severity/alert degradation.
    Severity,
    /// G channel dominant — quality degradation.
    Quality,
    /// B channel dominant — freshness decay.
    Freshness,
    /// Balanced across all channels.
    Balanced,
}

/// Break down the drift vector per channel.
pub fn drift_axis(snapshot: [u8; 3], root: [u8; 3]) -> DriftAxis {
    let dr = (snapshot[0] as i16 - root[0] as i16).unsigned_abs() as u32;
    let dg = (snapshot[1] as i16 - root[1] as i16).unsigned_abs() as u32;
    let db = (snapshot[2] as i16 - root[2] as i16).unsigned_abs() as u32;

    let max = dr.max(dg).max(db);
    let tied = [dr == max, dg == max, db == max]
        .iter()
        .filter(|&&x| x)
        .count()
        > 1;

    if tied || max == 0 {
        DriftAxis::Balanced
    } else if dr == max {
        DriftAxis::Severity
    } else if dg == max {
        DriftAxis::Quality
    } else {
        DriftAxis::Freshness
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_drift_for_identical_colors() {
        assert_eq!(rgb_drift([100, 200, 150], [100, 200, 150]), 0.0);
    }

    #[test]
    fn drift_is_euclidean() {
        // 3-4-5 right triangle in RGB space
        let a = [0u8, 0, 0];
        let b = [3u8, 4, 0];
        assert!((rgb_drift(a, b) - 5.0).abs() < 1e-4);
    }

    #[test]
    fn drift_axis_r_dominant() {
        assert_eq!(drift_axis([255, 0, 0], [0, 0, 0]), DriftAxis::Severity);
    }

    #[test]
    fn drift_axis_g_dominant() {
        assert_eq!(drift_axis([0, 200, 0], [0, 0, 0]), DriftAxis::Quality);
    }

    #[test]
    fn drift_axis_b_dominant() {
        assert_eq!(drift_axis([0, 0, 180], [0, 0, 0]), DriftAxis::Freshness);
    }

    #[test]
    fn drift_axis_balanced_on_equal_channels() {
        assert_eq!(drift_axis([50, 50, 50], [0, 0, 0]), DriftAxis::Balanced);
    }
}
