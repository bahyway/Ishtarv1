#![forbid(unsafe_code)]
//! Sovereign XML format skeleton — Stage 1 parsing interface.
//!
//! Full zero-dependency XML parsing is implemented in Phase 2 (Playbooks 203–206).
//! This module exposes the interface and error surface used by MASHSHARU and the
//! naramsin-integrity NRM_MALFORMED classifier.

use crate::error::FormatError;
use crate::row::Row;

/// Parse an XML byte slice into a row-oriented intermediate representation.
///
/// Expects a document where each repeated child element of the root becomes one Row.
/// Returns `FormatError::InvalidXml` for unclosed tags or malformed documents.
///
/// Phase 1 scaffold: full parser implemented in Playbook 204.
pub fn parse_xml(_src: &str) -> Result<Vec<Row>, FormatError> {
    Err(FormatError::InvalidXml {
        line: 0,
        reason: "XML parser not yet implemented — Phase 2 Playbook 204",
    })
}
