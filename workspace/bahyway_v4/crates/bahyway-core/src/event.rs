//! Event kernel types — Ṭupšarrūtu §1 (sort 𝓔) and §2.1.
//!
//! An `Event` is an **immutable StoryWay record** appended to a particle's
//! journal.  Once written, events are never modified or deleted (A2).
//!
//! `EventKind` is the kernel-level classification (Mint, Journal, TribeChange,
//! LaneTransition, Evidence).  Operational causes live in `enkidb-journal::EventCause`.

use crate::lane::Lane;

/// Kernel-level event type discriminant (Ṭupšarrūtu §2.1).
///
/// These are the *algebraic* kinds.  Implementations may extend with richer
/// payload enums, but the kernel only requires these five.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum EventKind {
    /// Particle created — first event in every particle's history.
    Mint = 0x01,
    /// Generic state journal — catches any non-enumerated mutation.
    Journal = 0x02,
    /// Particle moved between tribes (T3: emits the old and new tribe IDs).
    TribeChange = 0x03,
    /// Lane assignment changed (A3: move into Gray MUST be accompanied by Evidence).
    LaneTransition = 0x04,
    /// Evidence payload attached (satisfies the A3 obligation for Gray lane).
    Evidence = 0x05,
}

impl EventKind {
    /// Returns `true` when this kind satisfies the A3 evidence requirement.
    /// Per A3: a LaneTransition *into* Gray is only valid if a concurrent
    /// Evidence event is also appended in the same transaction.
    #[inline]
    pub fn is_evidence(self) -> bool {
        self == EventKind::Evidence
    }

    /// Display name for HeptaScript introspection and STIR output.
    pub fn as_str(self) -> &'static str {
        match self {
            EventKind::Mint => "MINT",
            EventKind::Journal => "JOURNAL",
            EventKind::TribeChange => "TRIBE_CHANGE",
            EventKind::LaneTransition => "LANE_TRANSITION",
            EventKind::Evidence => "EVIDENCE",
        }
    }
}

impl core::fmt::Display for EventKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// An immutable StoryWay event record (sort 𝓔 in the Ṭupšarrūtu algebra).
///
/// The payload is intentionally kept small at the kernel level — rich payloads
/// (bytes, EAV triples, tribe IDs) are layered on top by `enkidb-journal`.
/// The kernel only requires `kind` and a monotonic `sequence`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Event {
    /// Algebraic kind — what type of operation this event records.
    pub kind: EventKind,
    /// Monotonically increasing sequence within a single particle's journal.
    /// Sequence 1 is always a `Mint` event.
    pub sequence: u32,
    /// Optional lane snapshot: the *new* lane after a LaneTransition, or the
    /// lane at which this event applies.  `None` for non-lane events.
    pub lane_snapshot: Option<Lane>,
}

impl Event {
    /// Construct a `Mint` event (sequence = 1, default White lane).
    pub fn mint() -> Self {
        Self {
            kind: EventKind::Mint,
            sequence: 1,
            lane_snapshot: Some(Lane::White),
        }
    }

    /// Construct a generic `Journal` event at the given sequence number.
    pub fn journal(sequence: u32) -> Self {
        Self {
            kind: EventKind::Journal,
            sequence,
            lane_snapshot: None,
        }
    }

    /// Construct a `LaneTransition` event.
    pub fn lane_transition(sequence: u32, new_lane: Lane) -> Self {
        Self {
            kind: EventKind::LaneTransition,
            sequence,
            lane_snapshot: Some(new_lane),
        }
    }

    /// Construct an `Evidence` event (satisfies A3 for a Gray transition).
    pub fn evidence(sequence: u32) -> Self {
        Self {
            kind: EventKind::Evidence,
            sequence,
            lane_snapshot: None,
        }
    }

    /// Construct a `TribeChange` event.
    pub fn tribe_change(sequence: u32) -> Self {
        Self {
            kind: EventKind::TribeChange,
            sequence,
            lane_snapshot: None,
        }
    }
}

/// Validate that an event sequence satisfies A3: no Gray LaneTransition without
/// a preceding or concurrent Evidence event at the same sequence boundary.
///
/// Returns `Ok(())` when valid, or `Err(A3Violation)` when the constraint is broken.
pub fn validate_a3(events: &[Event]) -> Result<(), A3Violation> {
    let mut has_evidence_at_seq = false;
    let mut pending_gray_seq: Option<u32> = None;

    for e in events {
        match e.kind {
            EventKind::Evidence => {
                if pending_gray_seq == Some(e.sequence) {
                    has_evidence_at_seq = true;
                }
            }
            EventKind::LaneTransition if e.lane_snapshot == Some(Lane::Gray) => {
                // Check whether evidence exists at the same sequence
                let evidence_exists = events
                    .iter()
                    .any(|x| x.kind == EventKind::Evidence && x.sequence == e.sequence);
                if !evidence_exists {
                    return Err(A3Violation {
                        sequence: e.sequence,
                    });
                }
                pending_gray_seq = None;
            }
            _ => {}
        }
        let _ = has_evidence_at_seq; // suppress unused warning
        let _ = pending_gray_seq;
    }
    Ok(())
}

/// A3 violation: a LaneTransition into Gray had no accompanying Evidence event.
#[derive(Debug, PartialEq, Eq)]
pub struct A3Violation {
    pub sequence: u32,
}

impl core::fmt::Display for A3Violation {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "A3 violation: Gray lane transition at seq {} has no Evidence",
            self.sequence
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mint_event_defaults() {
        let e = Event::mint();
        assert_eq!(e.kind, EventKind::Mint);
        assert_eq!(e.sequence, 1);
        assert_eq!(e.lane_snapshot, Some(Lane::White));
    }

    #[test]
    fn event_kind_display() {
        assert_eq!(EventKind::Mint.to_string(), "MINT");
        assert_eq!(EventKind::Journal.to_string(), "JOURNAL");
        assert_eq!(EventKind::TribeChange.to_string(), "TRIBE_CHANGE");
        assert_eq!(EventKind::LaneTransition.to_string(), "LANE_TRANSITION");
        assert_eq!(EventKind::Evidence.to_string(), "EVIDENCE");
    }

    #[test]
    fn evidence_flag() {
        assert!(EventKind::Evidence.is_evidence());
        assert!(!EventKind::Mint.is_evidence());
        assert!(!EventKind::LaneTransition.is_evidence());
    }

    #[test]
    fn a3_valid_gray_with_evidence() {
        let events = vec![
            Event::mint(),
            Event::evidence(2),
            Event::lane_transition(2, Lane::Gray),
        ];
        assert!(validate_a3(&events).is_ok());
    }

    #[test]
    fn a3_violation_gray_without_evidence() {
        let events = vec![
            Event::mint(),
            Event::lane_transition(2, Lane::Gray), // no Evidence at seq 2
        ];
        let result = validate_a3(&events);
        assert_eq!(result, Err(A3Violation { sequence: 2 }));
    }

    #[test]
    fn a3_white_transition_no_evidence_needed() {
        let events = vec![Event::mint(), Event::lane_transition(2, Lane::White)];
        assert!(validate_a3(&events).is_ok());
    }

    #[test]
    fn a3_black_transition_no_evidence_needed() {
        let events = vec![Event::mint(), Event::lane_transition(2, Lane::Black)];
        assert!(validate_a3(&events).is_ok());
    }

    #[test]
    fn a3_violation_display() {
        let v = A3Violation { sequence: 5 };
        assert!(v.to_string().contains("seq 5"));
    }
}
