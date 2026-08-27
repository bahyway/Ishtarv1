#![forbid(unsafe_code)]
//! Sync connection pool backed by std::sync::Mutex.

use crate::error::ConError;
use std::net::TcpStream;
use std::sync::Mutex;

pub struct PooledConnection {
    pub stream: TcpStream,
    pub session_id: String,
}

pub struct ConnectionPool {
    connections: Mutex<Vec<PooledConnection>>,
    max_size: usize,
}

impl ConnectionPool {
    pub fn new(max_size: usize) -> Self {
        Self {
            connections: Mutex::new(Vec::new()),
            max_size,
        }
    }

    /// Acquire a pooled connection. Returns `PoolExhausted` if none available.
    pub fn acquire(&self) -> Result<PooledConnection, ConError> {
        let mut guard = self
            .connections
            .lock()
            .map_err(|_| ConError::Io("mutex poisoned".to_string()))?;
        guard.pop().ok_or(ConError::PoolExhausted)
    }

    /// Return a connection to the pool. Drops it if the pool is full.
    pub fn release(&self, conn: PooledConnection) {
        if let Ok(mut guard) = self.connections.lock() {
            if guard.len() < self.max_size {
                guard.push(conn);
            }
            // If pool is full, conn is dropped here (socket closed)
        }
    }

    pub fn len(&self) -> usize {
        self.connections.lock().map(|g| g.len()).unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
