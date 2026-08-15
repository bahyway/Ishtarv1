#![forbid(unsafe_code)]
//! Trajectory buffering — sliding-window 7D movement records per particle.

use enkidb_kaki::{Kaki, FixedCoord7D};

/// A single trajectory segment: one particle's position at one orbital tick.
#[derive(Copy, Clone, Debug)]
pub struct TrajectorySegment {
    pub particle_kaki: Kaki,
    /// 7D position in fixed-point millimetres.
    pub position: FixedCoord7D,
    /// BASE orbital counter at this observation.
    pub orbital: u64,
}

/// Sliding-window buffer of trajectory segments for one particle.
/// Window size is fixed at 16 orbitals (TRAJ-WINDOW-001).
pub struct TrajectoryBuffer {
    segments: [Option<TrajectorySegment>; 16],
    head: usize,
    count: usize,
}

impl TrajectoryBuffer {
    const WINDOW: usize = 16;

    pub fn new() -> Self {
        Self { segments: [None; 16], head: 0, count: 0 }
    }

    /// Push a new observation, evicting the oldest if the window is full.
    pub fn push(&mut self, seg: TrajectorySegment) {
        self.segments[self.head] = Some(seg);
        self.head = (self.head + 1) % Self::WINDOW;
        if self.count < Self::WINDOW { self.count += 1; }
    }

    /// Return all buffered segments in chronological order (oldest first).
    pub fn segments(&self) -> impl Iterator<Item = &TrajectorySegment> {
        let start = if self.count < Self::WINDOW {
            0
        } else {
            self.head // head points to oldest when window full
        };
        (0..self.count).map(move |i| {
            self.segments[(start + i) % Self::WINDOW].as_ref().unwrap()
        })
    }

    pub fn len(&self) -> usize { self.count }
    pub fn is_full(&self) -> bool { self.count == Self::WINDOW }
}

impl Default for TrajectoryBuffer {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use enkidb_kaki::{derive_pattern_kaki, PatternType};

    fn sample_kaki() -> Kaki {
        derive_pattern_kaki(PatternType::CrowdFlow, FixedCoord7D::zero(), [0u8; 32], 8_000, 1)
    }

    fn seg(orbital: u64) -> TrajectorySegment {
        TrajectorySegment {
            particle_kaki: sample_kaki(),
            position: FixedCoord7D { d: [orbital as i32, 0, 0, 0, 0, 0, 0] },
            orbital,
        }
    }

    #[test]
    fn window_fills_and_evicts() {
        let mut buf = TrajectoryBuffer::new();
        for i in 0..20u64 {
            buf.push(seg(i));
        }
        assert_eq!(buf.len(), 16);
        // oldest surviving segment should be orbital 4 (20 - 16 = 4)
        let segs: Vec<_> = buf.segments().collect();
        assert_eq!(segs[0].orbital, 4);
        assert_eq!(segs[15].orbital, 19);
    }

    #[test]
    fn partial_window() {
        let mut buf = TrajectoryBuffer::new();
        for i in 0..5u64 { buf.push(seg(i)); }
        assert_eq!(buf.len(), 5);
        assert!(!buf.is_full());
        let segs: Vec<_> = buf.segments().collect();
        assert_eq!(segs[0].orbital, 0);
        assert_eq!(segs[4].orbital, 4);
    }
}
