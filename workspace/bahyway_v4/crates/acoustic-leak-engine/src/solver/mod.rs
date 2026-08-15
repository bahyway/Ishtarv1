//! Stage-2 solvers. Selection is by material (LAW-ALE-2), never by choice.

pub mod dispersion;
pub mod gcc_phat;
pub mod joint_index;

use crate::material::{PipeMaterial, SolverClass};
use dispersion::DispersionModel;

#[derive(Debug, Clone, Copy)]
pub struct DelayEstimate {
    pub delay_s: f64,
    /// Peak sharpness proxy in [0,1]: peak / total correlation energy.
    pub peak_quality: f64,
}

/// Dispatch: the material decides (LAW-ALE-2).
pub fn estimate_delay(
    material: PipeMaterial,
    x: &[f64],
    y: &[f64],
    sample_rate_hz: f64,
    sensor_spacing_m: f64,
    dispersion: Option<&DispersionModel>,
) -> Option<DelayEstimate> {
    match material.solver_class() {
        SolverClass::PlainGccPhat => gcc_phat::gcc_phat_delay(x, y, sample_rate_hz),
        SolverClass::DispersionCompensated => {
            let model = dispersion?; // plastic WITHOUT a calibrated model is refused
            dispersion::iterate_compensated_delay(x, y, sample_rate_hz, model, sensor_spacing_m, 5)
        }
        // Jointed pipe starts from a coarse correlation; quantization to the
        // joint map happens in joint_index::snap_to_joint.
        SolverClass::JointIndexed => gcc_phat::gcc_phat_delay(x, y, sample_rate_hz),
    }
}
