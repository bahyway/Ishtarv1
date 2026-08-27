//! Raft consensus state machine — pure logic, no I/O.
//!
//! The caller drives the state machine by:
//!   1. Calling `tick(elapsed_ms)` on each heartbeat/timer interval.
//!   2. Calling `handle(from, msg)` for every inbound message.
//!   3. Acting on the returned `Vec<RaftEffect>` (send messages, write WAL, apply cmds).
//!
//! Election timeouts are randomised in [150, 300] ms (from session requirements).

#![forbid(unsafe_code)]

use crate::log::{LogEntry, LogView, RaftEav};
use crate::message::RaftMessage;

/// Election timeout range (milliseconds).
pub const ELECTION_TIMEOUT_MIN_MS: u64 = 150;
pub const ELECTION_TIMEOUT_MAX_MS: u64 = 300;
/// Leader heartbeat interval.
pub const HEARTBEAT_INTERVAL_MS: u64 = 50;

/// Cluster node role.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Follower,
    Candidate,
    Leader,
}

/// An effect the state machine requests the caller to perform.
#[derive(Debug, Clone)]
pub enum RaftEffect {
    /// Send a message to a specific peer (node id).
    Send { to: u64, msg: RaftMessage },
    /// Broadcast a message to all peers.
    Broadcast { msg: RaftMessage },
    /// Append entries to the durable WAL before applying.
    WriteWal { entries: Vec<LogEntry> },
    /// Apply committed entries to the state machine (EnkiDB EAV store).
    Apply { entries: Vec<LogEntry> },
}

/// Deterministic random using a simple LCG seeded from node id + term.
fn pseudo_timeout(node_id: u64, term: u64) -> u64 {
    let seed = node_id
        .wrapping_mul(0x9e3779b97f4a7c15)
        .wrapping_add(term.wrapping_mul(6364136223846793005));
    let range = ELECTION_TIMEOUT_MAX_MS - ELECTION_TIMEOUT_MIN_MS;
    ELECTION_TIMEOUT_MIN_MS + (seed >> 32) % range
}

/// Full Raft node state.
pub struct RaftNode {
    pub id: u64,
    pub peers: Vec<u64>,

    // Persistent state (must survive crash)
    current_term: u64,
    voted_for: Option<u64>,
    log: Vec<LogEntry>,

    // Volatile state (all nodes)
    commit_index: usize,
    last_applied: usize,

    // Role
    pub role: Role,

    // Candidate state
    votes_received: u64, // count of votes granted this term

    // Leader state (per follower)
    next_index: Vec<usize>,
    match_index: Vec<usize>,

    // Election timer
    elapsed_ms: u64,
    election_timeout: u64,

    // Leader heartbeat timer
    hb_elapsed_ms: u64,
}

impl RaftNode {
    /// Create a new follower node.
    pub fn new(id: u64, peers: Vec<u64>) -> Self {
        let n_peers = peers.len();
        let timeout = pseudo_timeout(id, 0);
        RaftNode {
            id,
            peers,
            current_term: 0,
            voted_for: None,
            log: Vec::new(),
            commit_index: 0,
            last_applied: 0,
            role: Role::Follower,
            votes_received: 0,
            next_index: vec![1; n_peers],
            match_index: vec![0; n_peers],
            elapsed_ms: 0,
            election_timeout: timeout,
            hb_elapsed_ms: 0,
        }
    }

    pub fn current_term(&self) -> u64 {
        self.current_term
    }
    pub fn commit_index(&self) -> usize {
        self.commit_index
    }
    pub fn log_len(&self) -> usize {
        self.log.len()
    }

    /// Advance the election/heartbeat timer by `elapsed_ms` milliseconds.
    /// Returns effects (election start or heartbeats).
    pub fn tick(&mut self, elapsed_ms: u64) -> Vec<RaftEffect> {
        let mut effects = Vec::new();
        match self.role {
            Role::Follower | Role::Candidate => {
                self.elapsed_ms += elapsed_ms;
                if self.elapsed_ms >= self.election_timeout {
                    self.start_election(&mut effects);
                }
            }
            Role::Leader => {
                self.hb_elapsed_ms += elapsed_ms;
                if self.hb_elapsed_ms >= HEARTBEAT_INTERVAL_MS {
                    self.hb_elapsed_ms = 0;
                    self.send_heartbeats(&mut effects);
                }
            }
        }
        effects
    }

    /// Handle an inbound message from peer `from`.
    pub fn handle(&mut self, from: u64, msg: RaftMessage) -> Vec<RaftEffect> {
        let mut effects = Vec::new();

        // Term update rule (§5.1): any message with higher term → revert to Follower.
        if msg.term() > self.current_term {
            self.current_term = msg.term();
            self.voted_for = None;
            self.become_follower();
        }

        match msg {
            RaftMessage::Vote {
                term,
                candidate_id,
                last_log_index,
                last_log_term,
            } => {
                self.handle_vote(
                    from,
                    term,
                    candidate_id,
                    last_log_index,
                    last_log_term,
                    &mut effects,
                );
            }
            RaftMessage::VoteOk { term, vote_granted } => {
                self.handle_vote_ok(term, vote_granted, &mut effects);
            }
            RaftMessage::Append {
                term,
                leader_id,
                prev_log_index,
                prev_log_term,
                entries,
                leader_commit,
            } => {
                self.handle_append(
                    from,
                    term,
                    leader_id,
                    prev_log_index,
                    prev_log_term,
                    entries,
                    leader_commit,
                    &mut effects,
                );
            }
            RaftMessage::AppendOk {
                term: _,
                match_index,
            } => {
                self.handle_append_ok(from, match_index, &mut effects);
            }
            RaftMessage::AppendFail { term: _ } => {
                self.handle_append_fail(from);
            }
            RaftMessage::Heartbeat {
                term,
                leader_id,
                leader_commit,
            } => {
                self.handle_heartbeat(from, term, leader_id, leader_commit, &mut effects);
            }
        }

        // Apply committed entries
        self.advance_state_machine(&mut effects);
        effects
    }

    /// Leader API: propose a new EAV command.
    /// Returns `None` if not leader; `Some(effects)` with AppendEntries RPCs otherwise.
    pub fn propose(&mut self, command: RaftEav) -> Option<Vec<RaftEffect>> {
        if self.role != Role::Leader {
            return None;
        }
        let entry = LogEntry::new(self.current_term, command);
        self.log.push(entry.clone());

        let mut effects = vec![RaftEffect::WriteWal {
            entries: vec![entry],
        }];
        self.replicate_to_peers(&mut effects);
        Some(effects)
    }

    // ─────────────────────────────────────────────────────────────────────
    // Private helpers
    // ─────────────────────────────────────────────────────────────────────

    fn become_follower(&mut self) {
        self.role = Role::Follower;
        self.votes_received = 0;
        self.reset_election_timer();
    }

    fn reset_election_timer(&mut self) {
        self.elapsed_ms = 0;
        self.election_timeout = pseudo_timeout(self.id, self.current_term);
    }

    fn start_election(&mut self, effects: &mut Vec<RaftEffect>) {
        self.current_term += 1;
        self.role = Role::Candidate;
        self.voted_for = Some(self.id);
        self.votes_received = 1; // vote for self
        self.reset_election_timer();

        let (last_idx, last_term) = LogView(&self.log).last_log_index_term();
        effects.push(RaftEffect::Broadcast {
            msg: RaftMessage::Vote {
                term: self.current_term,
                candidate_id: self.id,
                last_log_index: last_idx,
                last_log_term: last_term,
            },
        });
    }

    fn become_leader(&mut self, effects: &mut Vec<RaftEffect>) {
        self.role = Role::Leader;
        let next = self.log.len() + 1;
        for i in 0..self.peers.len() {
            self.next_index[i] = next;
            self.match_index[i] = 0;
        }
        self.send_heartbeats(effects);
    }

    fn send_heartbeats(&mut self, effects: &mut Vec<RaftEffect>) {
        for &peer in &self.peers {
            effects.push(RaftEffect::Send {
                to: peer,
                msg: RaftMessage::Heartbeat {
                    term: self.current_term,
                    leader_id: self.id,
                    leader_commit: self.commit_index,
                },
            });
        }
    }

    fn handle_vote(
        &mut self,
        _from: u64,
        term: u64,
        candidate_id: u64,
        last_log_index: usize,
        last_log_term: u64,
        effects: &mut Vec<RaftEffect>,
    ) {
        let (my_last_idx, my_last_term) = LogView(&self.log).last_log_index_term();
        let log_ok = (last_log_term > my_last_term)
            || (last_log_term == my_last_term && last_log_index >= my_last_idx);

        let can_vote = (self.voted_for.is_none() || self.voted_for == Some(candidate_id))
            && log_ok
            && term == self.current_term;

        if can_vote {
            self.voted_for = Some(candidate_id);
            self.reset_election_timer();
        }
        effects.push(RaftEffect::Send {
            to: candidate_id,
            msg: RaftMessage::VoteOk {
                term: self.current_term,
                vote_granted: can_vote,
            },
        });
    }

    fn handle_vote_ok(&mut self, _term: u64, vote_granted: bool, effects: &mut Vec<RaftEffect>) {
        if self.role != Role::Candidate {
            return;
        }
        if vote_granted {
            self.votes_received += 1;
        }
        let quorum = self.peers.len().div_ceil(2) + 1;
        if self.votes_received as usize >= quorum {
            self.become_leader(effects);
        }
    }

    // Each parameter is a distinct field of the Raft AppendEntries RPC
    // (leader identity, log-consistency check, payload, and commit index) --
    // not artificial padding, so bundling into a params struct is a real
    // API-design tradeoff rather than a mechanical fix.
    #[allow(clippy::too_many_arguments)]
    fn handle_append(
        &mut self,
        from: u64,
        _term: u64,
        _leader_id: u64,
        prev_log_index: usize,
        prev_log_term: u64,
        entries: Vec<LogEntry>,
        leader_commit: usize,
        effects: &mut Vec<RaftEffect>,
    ) {
        self.reset_election_timer();
        self.role = Role::Follower;

        if !LogView(&self.log).matches(prev_log_index, prev_log_term) {
            effects.push(RaftEffect::Send {
                to: from,
                msg: RaftMessage::AppendFail {
                    term: self.current_term,
                },
            });
            return;
        }

        // Truncate conflicting entries and append new ones.
        let insert_from = prev_log_index;
        if insert_from < self.log.len() {
            self.log.truncate(insert_from);
        }
        if !entries.is_empty() {
            effects.push(RaftEffect::WriteWal {
                entries: entries.clone(),
            });
            self.log.extend(entries);
        }

        // Advance commit index.
        if leader_commit > self.commit_index {
            self.commit_index = leader_commit.min(self.log.len());
        }

        effects.push(RaftEffect::Send {
            to: from,
            msg: RaftMessage::AppendOk {
                term: self.current_term,
                match_index: self.log.len(),
            },
        });
    }

    fn handle_append_ok(&mut self, from: u64, match_index: usize, effects: &mut Vec<RaftEffect>) {
        if self.role != Role::Leader {
            return;
        }
        if let Some(pos) = self.peers.iter().position(|&p| p == from) {
            self.match_index[pos] = match_index;
            self.next_index[pos] = match_index + 1;
        }
        self.try_advance_commit(effects);
    }

    fn handle_append_fail(&mut self, from: u64) {
        if self.role != Role::Leader {
            return;
        }
        if let Some(pos) = self.peers.iter().position(|&p| p == from) {
            if self.next_index[pos] > 1 {
                self.next_index[pos] -= 1;
            }
        }
    }

    fn handle_heartbeat(
        &mut self,
        _from: u64,
        _term: u64,
        _leader_id: u64,
        leader_commit: usize,
        _effects: &mut Vec<RaftEffect>,
    ) {
        self.reset_election_timer();
        self.role = Role::Follower;
        if leader_commit > self.commit_index {
            self.commit_index = leader_commit.min(self.log.len());
        }
    }

    /// Leader checks if N+1 peers have match_index ≥ N (quorum commit).
    fn try_advance_commit(&mut self, _effects: &mut Vec<RaftEffect>) {
        if self.role != Role::Leader {
            return;
        }
        let n_total = self.peers.len() + 1; // peers + self
        let quorum = n_total / 2 + 1;

        for n in ((self.commit_index + 1)..=self.log.len()).rev() {
            if self
                .log
                .get(n - 1)
                .is_some_and(|e| e.term == self.current_term)
            {
                let replicated = 1 + self.match_index.iter().filter(|&&m| m >= n).count();
                if replicated >= quorum {
                    self.commit_index = n;
                    break;
                }
            }
        }
    }

    fn replicate_to_peers(&mut self, effects: &mut Vec<RaftEffect>) {
        for (i, &peer) in self.peers.iter().enumerate() {
            let ni = self.next_index[i];
            let prev_index = ni.saturating_sub(1);
            let prev_term = if prev_index == 0 {
                0
            } else {
                self.log.get(prev_index - 1).map_or(0, |e| e.term)
            };
            let entries: Vec<LogEntry> = self.log.get(ni - 1..).unwrap_or(&[]).to_vec();
            effects.push(RaftEffect::Send {
                to: peer,
                msg: RaftMessage::Append {
                    term: self.current_term,
                    leader_id: self.id,
                    prev_log_index: prev_index,
                    prev_log_term: prev_term,
                    entries,
                    leader_commit: self.commit_index,
                },
            });
        }
    }

    fn advance_state_machine(&mut self, effects: &mut Vec<RaftEffect>) {
        if self.last_applied < self.commit_index {
            let new_applied = self.commit_index;
            let apply_entries = self
                .log
                .get(self.last_applied..new_applied)
                .unwrap_or(&[])
                .to_vec();
            if !apply_entries.is_empty() {
                effects.push(RaftEffect::Apply {
                    entries: apply_entries,
                });
            }
            self.last_applied = new_applied;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::RaftEav;

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
    fn new_node_is_follower() {
        let node = RaftNode::new(1, vec![2, 3]);
        assert_eq!(node.role, Role::Follower);
        assert_eq!(node.current_term(), 0);
    }

    #[test]
    fn election_timeout_triggers_candidacy() {
        let mut node = RaftNode::new(1, vec![2, 3]);
        // Tick past the maximum timeout to guarantee election fires.
        let effects = node.tick(ELECTION_TIMEOUT_MAX_MS + 1);
        assert_eq!(node.role, Role::Candidate);
        assert_eq!(node.current_term(), 1);
        assert!(!effects.is_empty());
        // Should have broadcast a VOTE message.
        assert!(effects
            .iter()
            .any(|e| matches!(e, RaftEffect::Broadcast { .. })));
    }

    #[test]
    fn vote_granted_when_log_ok() {
        let mut node = RaftNode::new(1, vec![2, 3]);
        // Simulate receiving a valid VOTE for term 1.
        let effects = node.handle(
            2,
            RaftMessage::Vote {
                term: 1,
                candidate_id: 2,
                last_log_index: 0,
                last_log_term: 0,
            },
        );
        assert!(effects.iter().any(|e| {
            matches!(
                e,
                RaftEffect::Send {
                    msg: RaftMessage::VoteOk {
                        vote_granted: true,
                        ..
                    },
                    ..
                }
            )
        }));
    }

    #[test]
    fn becomes_leader_after_quorum_votes() {
        // 3-node cluster: node 1 is candidate, peers [2, 3]
        let mut node = RaftNode::new(1, vec![2, 3]);
        node.tick(ELECTION_TIMEOUT_MAX_MS + 1); // → Candidate, term=1
        assert_eq!(node.role, Role::Candidate);

        // Receive one VoteOk (self + 1 = quorum in a 3-node cluster)
        let effects = node.handle(
            2,
            RaftMessage::VoteOk {
                term: 1,
                vote_granted: true,
            },
        );
        assert_eq!(node.role, Role::Leader);
        // Should send heartbeats immediately.
        assert!(effects.iter().any(|e| matches!(
            e,
            RaftEffect::Send {
                msg: RaftMessage::Heartbeat { .. },
                ..
            }
        )));
    }

    #[test]
    fn follower_advances_commit_on_heartbeat() {
        let mut follower = RaftNode::new(2, vec![1, 3]);
        let effects = follower.handle(
            1,
            RaftMessage::Heartbeat {
                term: 1,
                leader_id: 1,
                leader_commit: 0,
            },
        );
        // No log entries yet, commit stays 0 — but no errors.
        assert_eq!(follower.commit_index(), 0);
        let _ = effects;
    }

    #[test]
    fn leader_can_propose_entry() {
        let mut node = RaftNode::new(1, vec![2, 3]);
        node.tick(ELECTION_TIMEOUT_MAX_MS + 1);
        node.handle(
            2,
            RaftMessage::VoteOk {
                term: 1,
                vote_granted: true,
            },
        );
        assert_eq!(node.role, Role::Leader);

        let effects = node.propose(eav(99)).expect("must be leader");
        assert!(effects
            .iter()
            .any(|e| matches!(e, RaftEffect::WriteWal { .. })));
        assert_eq!(node.log_len(), 1);
    }

    #[test]
    fn append_rejected_when_prev_log_mismatch() {
        let mut follower = RaftNode::new(2, vec![1, 3]);
        // Leader claims prev_log_index=5, but follower has empty log → mismatch.
        let effects = follower.handle(
            1,
            RaftMessage::Append {
                term: 1,
                leader_id: 1,
                prev_log_index: 5,
                prev_log_term: 1,
                entries: vec![],
                leader_commit: 0,
            },
        );
        assert!(effects.iter().any(|e| matches!(
            e,
            RaftEffect::Send {
                msg: RaftMessage::AppendFail { .. },
                ..
            }
        )));
    }

    #[test]
    fn higher_term_in_message_demotes_leader() {
        let mut node = RaftNode::new(1, vec![2, 3]);
        // Manually make it a leader in term 1.
        node.tick(ELECTION_TIMEOUT_MAX_MS + 1);
        node.handle(
            2,
            RaftMessage::VoteOk {
                term: 1,
                vote_granted: true,
            },
        );
        assert_eq!(node.role, Role::Leader);

        // Receive a message with higher term → must revert to Follower.
        node.handle(
            3,
            RaftMessage::Heartbeat {
                term: 5,
                leader_id: 3,
                leader_commit: 0,
            },
        );
        assert_eq!(node.role, Role::Follower);
        assert_eq!(node.current_term(), 5);
    }
}
