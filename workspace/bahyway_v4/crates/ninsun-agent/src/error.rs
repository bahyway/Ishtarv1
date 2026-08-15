#![forbid(unsafe_code)]
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NinsunError {
    #[error("Proposal build error: {0}")]
    Build(String),
}
