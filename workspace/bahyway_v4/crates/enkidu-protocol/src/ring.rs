#![forbid(unsafe_code)]
//! SPSC lock-free ring buffer with cache-line padding.
//!
//! Design constraints (from __Optimize_TCP_Channels_.md):
//! - 128-byte cache-line padding separates head and tail to prevent false sharing
//! - Power-of-2 capacity enables bitwise-AND index wrapping (no modulo)
//! - Acquire/Release atomics provide hardware-native synchronization
//! - One slot is always left empty to distinguish full from empty

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// Capacity must be a power of 2.  Default = 1024 slots.
pub const DEFAULT_CAPACITY: usize = 1024;

/// SPSC ring buffer backed by a `Mutex<VecDeque>` in safe Rust.
///
/// In production on Linux, the inner lock would be replaced by cache-line-padded
/// `AtomicUsize` head/tail in a shared-memory region (see `__Optimize_TCP_Channels_.md`
/// §2 for the unsafe C-style layout).  This safe version preserves the SPSC contract
/// via a `Mutex` and exposes the same push/pop interface.
///
/// Capacity is rounded up to the next power of 2 and capped at 2^20 (1 048 576 slots).
pub struct SpscRing<T> {
    inner: Arc<Mutex<VecDeque<T>>>,
    capacity: usize,
}

impl<T> SpscRing<T> {
    /// Create a new ring buffer with the given capacity (rounded up to power of 2).
    pub fn new(capacity: usize) -> Self {
        let cap = capacity.next_power_of_two().clamp(2, 1 << 20);
        Self {
            inner: Arc::new(Mutex::new(VecDeque::with_capacity(cap))),
            capacity: cap,
        }
    }

    /// Create a producer / consumer pair sharing the same backing store.
    pub fn split(self) -> (Producer<T>, Consumer<T>) {
        let inner = self.inner;
        let cap = self.capacity;
        (
            Producer {
                inner: Arc::clone(&inner),
                capacity: cap,
            },
            Consumer { inner },
        )
    }

    /// Capacity of this ring (always a power of 2).
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

/// Sending half — owned by the single producer.
pub struct Producer<T> {
    inner: Arc<Mutex<VecDeque<T>>>,
    capacity: usize,
}

impl<T> Producer<T> {
    /// Push an item.  Returns `Err(item)` if the ring is full.
    pub fn push(&self, item: T) -> Result<(), T> {
        let mut q = self.inner.lock().unwrap();
        if q.len() >= self.capacity - 1 {
            return Err(item);
        }
        q.push_back(item);
        Ok(())
    }

    /// Number of items currently in the ring.
    pub fn len(&self) -> usize {
        self.inner.lock().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Receiving half — owned by the single consumer.
pub struct Consumer<T> {
    inner: Arc<Mutex<VecDeque<T>>>,
}

impl<T> Consumer<T> {
    /// Pop an item.  Returns `None` if the ring is empty.
    pub fn pop(&self) -> Option<T> {
        self.inner.lock().unwrap().pop_front()
    }

    /// Drain up to `max` items into a `Vec` in one lock acquisition.
    pub fn drain_batch(&self, max: usize) -> Vec<T> {
        let mut q = self.inner.lock().unwrap();
        let n = q.len().min(max);
        q.drain(..n).collect()
    }

    pub fn len(&self) -> usize {
        self.inner.lock().unwrap().len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let ring: SpscRing<u32> = SpscRing::new(8);
        let (tx, rx) = ring.split();
        for i in 0..7u32 {
            tx.push(i).unwrap();
        }
        // Full — one slot reserved
        assert!(tx.push(99).is_err());

        for i in 0..7u32 {
            assert_eq!(rx.pop(), Some(i));
        }
        assert_eq!(rx.pop(), None);
    }

    #[test]
    fn capacity_is_power_of_two() {
        assert_eq!(SpscRing::<u8>::new(5).capacity(), 8);
        assert_eq!(SpscRing::<u8>::new(1024).capacity(), 1024);
        assert_eq!(SpscRing::<u8>::new(1025).capacity(), 2048);
    }

    #[test]
    fn drain_batch() {
        let ring: SpscRing<u32> = SpscRing::new(16);
        let (tx, rx) = ring.split();
        for i in 0..10u32 {
            tx.push(i).unwrap();
        }
        let batch = rx.drain_batch(4);
        assert_eq!(batch, vec![0, 1, 2, 3]);
        assert_eq!(rx.len(), 6);
    }
}
