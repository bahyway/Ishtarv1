#![forbid(unsafe_code)]
//! Buffer pool — reuse `Vec<u8>` allocations across TCP connections.
//!
//! In a tight loop, Vec allocation/deallocation becomes the bottleneck.
//! A bounded pool provides backpressure naturally: if the pool empties,
//! the caller falls back to a fresh allocation; when the buffer is
//! released, it goes back into the pool for the next caller.
//!
//! Based on `__Optimize_TCP_Channels_.md` §5 (ArrayQueue pattern).

use std::collections::VecDeque;
use std::sync::Mutex;

/// Pool of reusable byte buffers.
///
/// Each buffer has the same `buf_size` capacity.  The pool holds at most
/// `max_bufs` entries; excess releases are silently dropped (GC'd by the
/// allocator).  Acquire always succeeds: it falls back to a fresh `Vec`
/// when the pool is empty.
pub struct BufferPool {
    pool: Mutex<VecDeque<Vec<u8>>>,
    buf_size: usize,
    max_bufs: usize,
}

impl BufferPool {
    /// Create a pool.
    ///
    /// * `max_bufs`  — maximum number of buffers to keep warm (e.g. 256).
    /// * `buf_size`  — byte capacity of each buffer (e.g. 65 536).
    pub fn new(max_bufs: usize, buf_size: usize) -> Self {
        Self {
            pool: Mutex::new(VecDeque::with_capacity(max_bufs)),
            buf_size,
            max_bufs,
        }
    }

    /// Acquire a buffer: pops from pool or allocates fresh.
    pub fn acquire(&self) -> Vec<u8> {
        self.pool
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or_else(|| vec![0u8; self.buf_size])
    }

    /// Release a buffer back to the pool.  Clears content, trims excess capacity.
    pub fn release(&self, mut buf: Vec<u8>) {
        buf.clear();
        // Discard if grown beyond 2× intended size (avoids memory bloat).
        if buf.capacity() > self.buf_size * 2 {
            return;
        }
        let mut q = self.pool.lock().unwrap();
        if q.len() < self.max_bufs {
            q.push_back(buf);
        }
    }

    /// Number of buffers currently in the pool.
    pub fn idle_count(&self) -> usize {
        self.pool.lock().unwrap().len()
    }

    /// Buffer size this pool was created with.
    pub fn buf_size(&self) -> usize { self.buf_size }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acquire_release_cycle() {
        let pool = BufferPool::new(8, 4096);
        let buf = pool.acquire();
        assert_eq!(buf.len(), 4096);
        assert_eq!(pool.idle_count(), 0);

        pool.release(buf);
        assert_eq!(pool.idle_count(), 1);

        let buf2 = pool.acquire();
        assert_eq!(buf2.len(), 0); // cleared on release
        assert_eq!(pool.idle_count(), 0);
    }

    #[test]
    fn pool_bounded_by_max_bufs() {
        let pool = BufferPool::new(2, 64);
        for _ in 0..5 {
            let b = pool.acquire();
            pool.release(b);
        }
        assert!(pool.idle_count() <= 2);
    }

    #[test]
    fn oversized_buffer_discarded() {
        let pool = BufferPool::new(4, 64);
        let mut big = vec![0u8; 64];
        big.reserve(64 * 10); // capacity >> 2× buf_size
        pool.release(big);
        // Should be discarded
        assert_eq!(pool.idle_count(), 0);
    }
}
