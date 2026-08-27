//! Raft log entries — the EAV commands being replicated across the cluster.

#![forbid(unsafe_code)]

/// A 7-tuple EAV command: (Entity, Attr, Value, Time, Shard, ProofCtx, Materialization).
/// Mirrors the 7D index space `(E,A,V,T,S,Z,M)` of EnkiDB.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RaftEav {
    pub entity: u64,
    pub attr: u32,
    pub value_bytes: [u8; 16],
    pub time: u64,
    pub shard: u8,
    pub proof_ctx: u8,
    pub materialization: u8,
}

/// A single entry in the Raft log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
    /// Leader's term when the entry was created.
    pub term: u64,
    /// The EAV command to apply to the state machine.
    pub command: RaftEav,
}

impl LogEntry {
    pub fn new(term: u64, command: RaftEav) -> Self {
        LogEntry { term, command }
    }
}

/// A read-only view of the Raft log for log-matching checks.
pub struct LogView<'a>(pub &'a [LogEntry]);

impl<'a> LogView<'a> {
    /// (prevLogIndex, prevLogTerm) for an AppendEntries RPC.
    pub fn last_log_index_term(&self) -> (usize, u64) {
        match self.0.last() {
            Some(e) => (self.0.len(), e.term),
            None => (0, 0),
        }
    }

    /// Return `true` if the log contains an entry at 1-based `index` with `term`.
    pub fn matches(&self, index: usize, term: u64) -> bool {
        if index == 0 {
            return true; // empty log matches everything
        }
        self.0.get(index - 1).is_some_and(|e| e.term == term)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eav(e: u64) -> RaftEav {
        RaftEav {
            entity: e,
            attr: 0,
            value_bytes: [0; 16],
            time: 0,
            shard: 0,
            proof_ctx: 0,
            materialization: 0,
        }
    }

    #[test]
    fn empty_log_last_is_zero() {
        let log = LogView(&[]);
        assert_eq!(log.last_log_index_term(), (0, 0));
    }

    #[test]
    fn one_entry_last_is_one() {
        let entries = [LogEntry::new(1, eav(42))];
        let log = LogView(&entries);
        assert_eq!(log.last_log_index_term(), (1, 1));
    }

    #[test]
    fn matches_index_zero_always() {
        let log = LogView(&[]);
        assert!(log.matches(0, 999));
    }

    #[test]
    fn matches_checks_term_at_index() {
        let entries = [LogEntry::new(3, eav(1)), LogEntry::new(5, eav(2))];
        let log = LogView(&entries);
        assert!(log.matches(1, 3));
        assert!(!log.matches(1, 5));
        assert!(log.matches(2, 5));
    }
}
