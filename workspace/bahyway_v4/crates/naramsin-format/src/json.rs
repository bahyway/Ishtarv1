#![forbid(unsafe_code)]
//! Sovereign JSON format skeleton — Stage 1 parsing interface.
//!
//! Full zero-dependency JSON parsing is implemented in Phase 2 (Playbooks 203–206).
//! This module exposes the interface and error surface used by MASHSHARU and the
//! naramsin-integrity NRM_MALFORMED classifier.

use crate::error::FormatError;
use crate::row::Row;

/// Parse a JSON byte slice into a row-oriented intermediate representation.
///
/// Expects a JSON array of objects at the top level. Each object becomes one Row.
/// Returns `FormatError::InvalidJson` for any syntax error.
///
/// Phase 1 scaffold: full parser implemented in Playbook 203.
pub fn parse_json(_src: &str) -> Result<Vec<Row>, FormatError> {
    // Phase 2 implementation placeholder.
    // The interface is sealed here so MASHSHARU can reference it without
    // waiting for the full parser. The NRM_MALFORMED path below shows the
    // error contract.
    Err(FormatError::InvalidJson {
        byte_offset: 0,
        reason: "JSON parser not yet implemented — Phase 2 Playbook 203",
    })
}
