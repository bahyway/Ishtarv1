//! Raft RPC messages — all six message types in one enum.
//!
//! Text-wire format (sovereign TCP protocol, no serde):
//!   VOTE <term> <candidate_id> <last_log_index> <last_log_term>
//!   VOTE_OK <term> <granted:0|1>
//!   APPEND <term> <leader_id> <prev_idx> <prev_term> <commit> <entry_count> [entries...]
//!   APPEND_OK <term> <match_index>
//!   APPEND_FAIL <term>
//!   HEARTBEAT <term> <leader_id> <commit>

#![forbid(unsafe_code)]

use crate::log::LogEntry;

/// All Raft RPC messages.
#[derive(Debug, Clone, PartialEq)]
pub enum RaftMessage {
    /// RequestVote — sent by candidates to solicit votes.
    Vote {
        term: u64,
        candidate_id: u64,
        last_log_index: usize,
        last_log_term: u64,
    },
    /// RequestVote reply.
    VoteOk { term: u64, vote_granted: bool },
    /// AppendEntries — sent by the leader to replicate log entries.
    Append {
        term: u64,
        leader_id: u64,
        prev_log_index: usize,
        prev_log_term: u64,
        entries: Vec<LogEntry>,
        leader_commit: usize,
    },
    /// AppendEntries success reply.
    AppendOk { term: u64, match_index: usize },
    /// AppendEntries failure reply.
    AppendFail { term: u64 },
    /// Heartbeat — AppendEntries with no entries (keepalive / commit advance).
    Heartbeat {
        term: u64,
        leader_id: u64,
        leader_commit: usize,
    },
}

impl RaftMessage {
    /// Extract the term from any message variant (used for term update checks).
    pub fn term(&self) -> u64 {
        match self {
            RaftMessage::Vote { term, .. } => *term,
            RaftMessage::VoteOk { term, .. } => *term,
            RaftMessage::Append { term, .. } => *term,
            RaftMessage::AppendOk { term, .. } => *term,
            RaftMessage::AppendFail { term, .. } => *term,
            RaftMessage::Heartbeat { term, .. } => *term,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vote_term_extracted() {
        let m = RaftMessage::Vote {
            term: 7,
            candidate_id: 1,
            last_log_index: 0,
            last_log_term: 0,
        };
        assert_eq!(m.term(), 7);
    }

    #[test]
    fn heartbeat_term_extracted() {
        let m = RaftMessage::Heartbeat {
            term: 3,
            leader_id: 2,
            leader_commit: 5,
        };
        assert_eq!(m.term(), 3);
    }
}
