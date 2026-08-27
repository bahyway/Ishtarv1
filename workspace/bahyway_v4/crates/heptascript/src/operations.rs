//! The Five Sovereign Operations — the COMPLETE verb set of
//! BahyWay v4.0. There is no SELECT, INSERT, UPDATE, or DELETE.
//! Every data interaction is one of these five. TRIPLE-O only.
//!
//!   ORBIT   — "I have these particles in this orbit" (read/scan)
//!   EMIT    — "New particles born into orbit"        (birth)
//!   PROVE   — "Verify this particle's history"       (point lookup)
//!   SYNC    — "Compare our orbit states"              (replication)
//!   WITNESS — "I attest to this state"                (consensus)
//!
//! NOTE: EMIT expresses INTENT to birth. Actual KAKI minting is
//! performed ONLY by enkidb-ingest::bridge — never here.

/// The five — and only — sovereign operations.
///
/// Also `HeptaQuery.verb`'s type (`query.rs`) — the tokenizer/parser
/// (`token.rs`'s `Token::VerbOrbit`..`VerbWitness`, `parser.rs`'s
/// `maybe_parse_verb`) recognize an optional leading `ORBIT`/`EMIT`/
/// `PROVE`/`SYNC`/`WITNESS` keyword and produce this same type, rather
/// than a second parallel enum, so "the five sovereign operations" stays
/// one definition read by both this string-level `parse()` (used
/// wherever a verb arrives as free text, e.g. from a CLI arg) and the
/// real query grammar. See `engine.rs::execute_over_histories` for what
/// each verb actually does when parsed as a HeptaScript clause — that
/// realization is deliberately scoped to stay entirely read-only (never
/// bypassing the Write Node / Read Node split), which is narrower than
/// `is_read_only()` below describes for the *conceptual* five operations
/// (EMIT's real minting still only ever happens in `enkidb-ingest`; SYNC
/// and WITNESS as HeptaScript verbs compute a fingerprint/digest over a
/// read, not the fuller replication/consensus roles their names imply
/// elsewhere in the ecosystem).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Operation {
    #[default]
    Orbit,
    Emit,
    Prove,
    Sync,
    Witness,
}

impl Operation {
    pub const ALL: [Operation; 5] = [
        Operation::Orbit,
        Operation::Emit,
        Operation::Prove,
        Operation::Sync,
        Operation::Witness,
    ];

    pub fn verb(&self) -> &'static str {
        match self {
            Operation::Orbit => "ORBIT",
            Operation::Emit => "EMIT",
            Operation::Prove => "PROVE",
            Operation::Sync => "SYNC",
            Operation::Witness => "WITNESS",
        }
    }

    /// Parse a sovereign verb. SQL keywords are REJECTED by design.
    pub fn parse(token: &str) -> Result<Operation, OperationError> {
        match token.to_ascii_uppercase().as_str() {
            "ORBIT" => Ok(Operation::Orbit),
            "EMIT" => Ok(Operation::Emit),
            "PROVE" => Ok(Operation::Prove),
            "SYNC" => Ok(Operation::Sync),
            "WITNESS" => Ok(Operation::Witness),
            "SELECT" | "INSERT" | "UPDATE" | "DELETE" | "FROM" | "WHERE" | "JOIN" | "GROUP"
            | "ORDER" | "TABLE" | "VIEW" => Err(OperationError::SqlForbidden(token.to_string())),
            other => Err(OperationError::Unknown(other.to_string())),
        }
    }

    /// Does this operation mint new identity? Only EMIT — and even
    /// then the mint happens downstream in enkidb-ingest.
    pub fn births_particles(&self) -> bool {
        matches!(self, Operation::Emit)
    }

    /// Is this a read-only operation (safe on the read node)?
    pub fn is_read_only(&self) -> bool {
        matches!(self, Operation::Orbit | Operation::Prove)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperationError {
    /// A SQL keyword was used — absolutely forbidden in Triple-O.
    SqlForbidden(String),
    Unknown(String),
}

impl std::fmt::Display for OperationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperationError::SqlForbidden(t) => write!(
                f,
                "SQL is forbidden in BahyWay v4.0: '{}' — use ORBIT/EMIT/PROVE/SYNC/WITNESS",
                t
            ),
            OperationError::Unknown(t) => write!(f, "unknown sovereign verb: '{}'", t),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exactly_five_operations() {
        assert_eq!(Operation::ALL.len(), 5);
    }

    #[test]
    fn all_sovereign_verbs_parse() {
        for op in Operation::ALL {
            assert_eq!(Operation::parse(op.verb()), Ok(op));
        }
    }

    #[test]
    fn every_sql_keyword_is_rejected() {
        for kw in [
            "SELECT", "INSERT", "UPDATE", "DELETE", "FROM", "WHERE", "JOIN", "GROUP", "ORDER",
            "TABLE", "VIEW", "select", "Join",
        ] {
            match Operation::parse(kw) {
                Err(OperationError::SqlForbidden(_)) => {}
                other => panic!("SQL keyword {} not rejected: {:?}", kw, other),
            }
        }
    }

    #[test]
    fn only_emit_births_particles() {
        assert!(Operation::Emit.births_particles());
        for op in [
            Operation::Orbit,
            Operation::Prove,
            Operation::Sync,
            Operation::Witness,
        ] {
            assert!(!op.births_particles());
        }
    }

    #[test]
    fn read_only_set_is_orbit_and_prove() {
        assert!(Operation::Orbit.is_read_only());
        assert!(Operation::Prove.is_read_only());
        assert!(!Operation::Emit.is_read_only());
        assert!(!Operation::Sync.is_read_only());
        assert!(!Operation::Witness.is_read_only());
    }

    #[test]
    fn error_message_guides_to_sovereign_verbs() {
        let e = Operation::parse("SELECT").unwrap_err();
        assert!(format!("{}", e).contains("ORBIT/EMIT/PROVE/SYNC/WITNESS"));
    }
}
