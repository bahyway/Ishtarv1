//! memory_store.rs — Sovereign append-only memory store for conversations.
#![forbid(unsafe_code)]

use std::collections::HashMap;
use crate::session::Session;
use crate::conversation::ConversationParticle;

#[derive(Debug)]
pub enum MemoryError {
    SessionNotFound(u64),
    SerializationError(String),
    IoError(String),
}

impl std::fmt::Display for MemoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SessionNotFound(id)    => write!(f, "Session not found: {id}"),
            Self::SerializationError(e)  => write!(f, "Serialization error: {e}"),
            Self::IoError(e)             => write!(f, "IO error: {e}"),
        }
    }
}

/// In-memory sovereign store for all conversation sessions.
/// Persistence via journal file is planned for v4.1.
pub struct MemoryStore {
    sessions: HashMap<u64, Session>,
    next_session_id: u64,
    epoch_counter: u64,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self { sessions: HashMap::new(), next_session_id: 1, epoch_counter: 0 }
    }

    /// Create a new session.
    pub fn new_session(&mut self, title: &str) -> u64 {
        let id    = self.next_session_id;
        let epoch = self.tick();
        self.sessions.insert(id, Session::new(id, title, epoch));
        self.next_session_id += 1;
        id
    }

    /// Get a session by ID.
    pub fn session(&self, id: u64) -> Result<&Session, MemoryError> {
        self.sessions.get(&id).ok_or(MemoryError::SessionNotFound(id))
    }

    /// Get a mutable session by ID.
    pub fn session_mut(&mut self, id: u64) -> Result<&mut Session, MemoryError> {
        self.sessions.get_mut(&id).ok_or(MemoryError::SessionNotFound(id))
    }

    /// Add a user turn to a session.
    pub fn add_user(&mut self, session_id: u64, content: &str) -> Result<(), MemoryError> {
        let epoch = self.tick();
        let session = self.session_mut(session_id)?;
        session.add_user(content, epoch);
        Ok(())
    }

    /// Add an assistant turn to a session.
    pub fn add_assistant(&mut self, session_id: u64, content: &str) -> Result<(), MemoryError> {
        let epoch = self.tick();
        let session = self.session_mut(session_id)?;
        session.add_assistant(content, epoch);
        Ok(())
    }

    /// Get all sessions sorted by creation time (newest first).
    pub fn all_sessions(&self) -> Vec<&Session> {
        let mut sessions: Vec<&Session> = self.sessions.values().collect();
        sessions.sort_by(|a, b| b.created_epoch.cmp(&a.created_epoch));
        sessions
    }

    /// Total number of turns across all sessions.
    pub fn total_turns(&self) -> usize {
        self.sessions.values().map(|s| s.turns.len()).sum()
    }

    /// All particles across all sessions (for search).
    pub fn all_particles(&self) -> Vec<&ConversationParticle> {
        self.sessions.values()
            .flat_map(|s| s.turns.iter())
            .collect()
    }

    fn tick(&mut self) -> u64 {
        self.epoch_counter += 1;
        self.epoch_counter
    }
}

impl Default for MemoryStore { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_session_and_add_turns() {
        let mut store = MemoryStore::new();
        let sid = store.new_session("Test");
        store.add_user(sid, "What is KAKI?").unwrap();
        store.add_assistant(sid, "KAKI is the 16-byte sovereign identity.").unwrap();
        let session = store.session(sid).unwrap();
        assert_eq!(session.turns.len(), 2);
    }

    #[test]
    fn session_not_found_returns_error() {
        let store = MemoryStore::new();
        assert!(matches!(store.session(999), Err(MemoryError::SessionNotFound(999))));
    }

    #[test]
    fn total_turns_across_sessions() {
        let mut store = MemoryStore::new();
        let s1 = store.new_session("S1");
        let s2 = store.new_session("S2");
        store.add_user(s1, "q1").unwrap();
        store.add_user(s2, "q2").unwrap();
        store.add_user(s2, "q3").unwrap();
        assert_eq!(store.total_turns(), 3);
    }

    #[test]
    fn sessions_sorted_newest_first() {
        let mut store = MemoryStore::new();
        let s1 = store.new_session("First");
        let s2 = store.new_session("Second");
        let sessions = store.all_sessions();
        assert_eq!(sessions[0].session_id, s2);
        assert_eq!(sessions[1].session_id, s1);
    }
}
