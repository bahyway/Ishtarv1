//! HeptaScore error types — pure Rust, no thiserror dependency.

use std::fmt;

#[derive(Debug)]
pub enum HeptaError {
    /// A dimension value is outside the valid [0.0, 1.0] range.
    DimensionOutOfRange { dim: usize, value: f32 },
    /// The weight vector does not sum to 1.0 (±0.001).
    WeightsDoNotSumToOne(f32),
    /// The ideal point contains values outside [0.0, 1.0].
    InvalidIdealPoint,
    /// A batch scoring call received an empty input slice.
    EmptyBatch,
    /// A dimension index is out of the 0–6 range.
    InvalidDimensionIndex(usize),
}

impl fmt::Display for HeptaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HeptaError::DimensionOutOfRange { dim, value } => {
                write!(f, "dimension {dim} value {value:.4} is outside [0.0, 1.0]")
            }
            HeptaError::WeightsDoNotSumToOne(sum) => {
                write!(f, "weight vector sums to {sum:.4}, must be 1.0 (±0.001)")
            }
            HeptaError::InvalidIdealPoint => {
                write!(f, "ideal point contains values outside [0.0, 1.0]")
            }
            HeptaError::EmptyBatch => write!(f, "batch is empty — cannot compute statistics"),
            HeptaError::InvalidDimensionIndex(i) => {
                write!(f, "dimension index {i} is out of range (must be 0–6)")
            }
        }
    }
}

impl std::error::Error for HeptaError {}
