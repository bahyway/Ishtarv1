#![forbid(unsafe_code)]
use std::fmt;

#[derive(Debug)]
pub enum KittuError {
    Io(String),
    Email(String),
    Serialise(String),
}

impl fmt::Display for KittuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KittuError::Io(s) => write!(f, "IO error: {s}"),
            KittuError::Email(s) => write!(f, "Email error: {s}"),
            KittuError::Serialise(s) => write!(f, "Serialise error: {s}"),
        }
    }
}

impl std::error::Error for KittuError {}
