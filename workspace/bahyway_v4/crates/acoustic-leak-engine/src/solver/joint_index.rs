//! Joint-indexed localization for jointed ceramic/concrete/AC lines.
//!
//! These lines are short segments (1-3 m) with a joint every few metres;
//! most leaks occur AT joints, and joint multipath destroys clean
//! correlation peaks. The lawful move is to QUANTIZE the answer space:
//! the verdict is "joint #k at position p_k", and the <=0.10 m margin is
//! inherited from the surveyed joint map, not from timing resolution.

#[derive(Debug, Clone)]
pub struct JointMap {
    /// Joint positions in metres from sensor A, strictly increasing.
    pub positions_m: Vec<f64>,
}

#[derive(Debug, Clone, Copy)]
pub struct JointVerdictCandidate {
    pub joint_index: usize,
    pub position_m: f64,
    /// |coarse estimate - joint position|.
    pub snap_error_m: f64,
    /// True when the coarse estimate is ambiguous between two joints --
    /// mandatory Stage-3 confirmation.
    pub ambiguous: bool,
}

pub fn snap_to_joint(map: &JointMap, coarse_position_m: f64) -> Option<JointVerdictCandidate> {
    if map.positions_m.is_empty() {
        return None;
    }
    let mut best = 0usize;
    let mut best_err = f64::MAX;
    let mut second_err = f64::MAX;
    for (i, &p) in map.positions_m.iter().enumerate() {
        let e = (p - coarse_position_m).abs();
        if e < best_err {
            second_err = best_err;
            best_err = e;
            best = i;
        } else if e < second_err {
            second_err = e;
        }
    }
    Some(JointVerdictCandidate {
        joint_index: best,
        position_m: map.positions_m[best],
        snap_error_m: best_err,
        // Ambiguous when the runner-up joint is nearly as close.
        ambiguous: second_err - best_err < 0.5,
    })
}
