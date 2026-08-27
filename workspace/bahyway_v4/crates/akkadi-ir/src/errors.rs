//! 𒁾 IR Errors

use crate::span::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum IrError {
    DuplicateNode {
        name: String,
        span: Span,
    },
    UndefinedReference {
        name: String,
        span: Span,
    },
    QualityOutOfRange {
        dim: String,
        value: f32,
        span: Span,
    },
    InvalidWeights {
        sum: f32,
        span: Span,
    },
    UnknownFlowSource {
        source: String,
        span: Span,
    },
    UnknownEmitTarget {
        target: String,
        span: Span,
    },
    EmptyIr,
    BackendError {
        backend: String,
        message: String,
    },
    UnknownField {
        particle: String,
        field: String,
        span: Span,
    },
    InvalidKineticTerm {
        term: String,
        reason: String,
    },
}

impl std::fmt::Display for IrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateNode { name, span } => {
                write!(f, "𒁾 IR: duplicate node '{name}' at {span}")
            }
            Self::UndefinedReference { name, span } => {
                write!(f, "𒁾 IR: undefined reference '{name}' at {span}")
            }
            Self::QualityOutOfRange { dim, value, span } => write!(
                f,
                "𒁾 IR: quality '{dim}' = {value:.3} outside [0.0,1.0] at {span}"
            ),
            Self::InvalidWeights { sum, span } => {
                write!(f, "𒁾 IR: weights sum to {sum:.4} ≠ 1.0 at {span}")
            }
            Self::UnknownFlowSource { source, span } => {
                write!(f, "𒁾 IR: unknown flow source '{source}' at {span}")
            }
            Self::UnknownEmitTarget { target, span } => {
                write!(f, "𒁾 IR: unknown emit target '{target}' at {span}")
            }
            Self::EmptyIr => write!(f, "𒁾 IR: tree is empty"),
            Self::BackendError { backend, message } => {
                write!(f, "𒁾 Backend '{backend}': {message}")
            }
            Self::UnknownField {
                particle,
                field,
                span,
            } => write!(f, "𒁾 IR: '{particle}' has no field '{field}' at {span}"),
            Self::InvalidKineticTerm { term, reason } => {
                write!(f, "𒁾 IR: kinetic term '{term}' invalid: {reason}")
            }
        }
    }
}

impl std::error::Error for IrError {}
