//! NinsunAdvisoryQueue — urgency-ordered Data Steward inbox for NINSUN's
//! `RefineProposal`s.
//!
//! Ordering was pure confidence-descending until 2026-07-10. It is now
//! urgency-first, confidence-second (see hazard.rs::UrgencyKey): a case
//! tagged with a real hazard domain (ESARHADDON's SMI structural-risk
//! state today) always outranks an ordinary case, regardless of how
//! confident NINSUN is about either. "Priority is not high/low, priority
//! is prediction" (DUB.SAR).
//!
//! `RefineProposal.target_kaki` is a string identifier (NINSUN reasons over
//! `KakiSummary` batches, not raw KAKI bytes), so confirming or rejecting a
//! case here requires the caller to supply the real `[u8; 16]` identity
//! bytes of the particle the proposal refers to — the same bytes the
//! caller already used to build the `KakiSummary` NINSUN analyzed. This
//! keeps the journal entry a real, auditable record rather than a
//! best-effort string lookup.

use enkidb_kaki::{Kaki, EventKaki, IdentityKaki, KakiMinter, KakiRole};
use enkidb_journal::{Journal, JournalEntry, EventCause};
use ninsun_agent::RefineProposal;
use esarhaddon::smi::Smi;
use crate::hazard::{classify_hazard, HazardDomain, UrgencyKey};

/// What the Data Steward decided about a proposal.  `Pending` cases are
/// still waiting for review.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdvisoryDecision {
    Pending,
    Confirmed,
    Rejected,
}

/// One NINSUN proposal sitting in the Steward's inbox, plus its decision
/// state and its hazard-derived urgency.
#[derive(Debug, Clone)]
pub struct AdvisoryCase {
    pub proposal: RefineProposal,
    pub decision: AdvisoryDecision,
    pub hazard:   HazardDomain,
    /// Present only for hazard-tagged cases where a caller supplied a
    /// computed SMI (ESARHADDON does not itself carry tribe/KAKI state,
    /// so this must be threaded through by whoever ran the SMI computation
    /// upstream of this queue).
    pub smi:      Option<Smi>,
}

impl AdvisoryCase {
    fn urgency_key(&self) -> UrgencyKey {
        match &self.smi {
            Some(smi) => UrgencyKey::from_smi(smi, self.proposal.confidence),
            None => UrgencyKey::ordinary(self.proposal.confidence),
        }
    }
}

/// The Data Steward's NINSUN advisory inbox. Highest-urgency proposals
/// sort to the front (see hazard.rs::UrgencyKey) so the Steward triages
/// real emergencies before ordinary data-quality issues, never the
/// reverse, regardless of relative confidence.
pub struct NinsunAdvisoryQueue {
    cases: Vec<AdvisoryCase>,
}

impl NinsunAdvisoryQueue {
    pub fn new() -> Self {
        NinsunAdvisoryQueue { cases: Vec::new() }
    }

    /// Receive a batch of NINSUN proposals with no hazard SMI data and
    /// re-sort the whole inbox. Equivalent to `receive_with_smi` where
    /// every item's SMI is `None` — kept for callers with no hazard
    /// context. Hazard domain is still classified from `tribe_id` even
    /// here, but without an SMI score a hazard-tagged case sorts as
    /// ordinary (urgency_rank 0) until a real SMI is supplied.
    pub fn receive(&mut self, proposals: Vec<RefineProposal>) {
        self.receive_with_smi(proposals.into_iter().map(|p| (p, None)).collect());
    }

    /// Receive a batch of (proposal, optional computed SMI) pairs and
    /// re-sort the whole inbox by urgency first, confidence second.
    pub fn receive_with_smi(&mut self, items: Vec<(RefineProposal, Option<Smi>)>) {
        for (proposal, smi) in items {
            let hazard = classify_hazard(&proposal.tribe_id);
            self.cases.push(AdvisoryCase { proposal, decision: AdvisoryDecision::Pending, hazard, smi });
        }
        self.cases.sort_by(|a, b| {
            b.urgency_key()
                .partial_cmp(&a.urgency_key())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    /// All still-undecided cases, highest confidence first.
    pub fn pending(&self) -> Vec<&AdvisoryCase> {
        self.cases.iter().filter(|c| c.decision == AdvisoryDecision::Pending).collect()
    }

    pub fn cases(&self)    -> &[AdvisoryCase] { &self.cases }
    pub fn len(&self)      -> usize           { self.cases.len() }
    pub fn is_empty(&self) -> bool            { self.cases.is_empty() }

    /// Steward confirms the case at `case_idx`: NINSUN's read was right.
    /// Journals a `NinsunAdvisoryConfirmed` entry against the real particle
    /// the proposal refers to.  The proposal itself, and the particle it
    /// targets, are never mutated — only the decision state changes.
    pub fn confirm(
        &mut self,
        case_idx:         usize,
        target_kaki_bytes: [u8; 16],
        epoch:            u32,
        journal:          &mut Journal,
        minter:           &KakiMinter,
    ) -> bool {
        self.decide(case_idx, target_kaki_bytes, epoch, journal, minter, AdvisoryDecision::Confirmed, EventCause::NinsunAdvisoryConfirmed)
    }

    /// Steward rejects the case at `case_idx`: NINSUN's flag was a false
    /// positive. Journals a `NinsunAdvisoryRejected` entry.
    pub fn reject(
        &mut self,
        case_idx:         usize,
        target_kaki_bytes: [u8; 16],
        epoch:            u32,
        journal:          &mut Journal,
        minter:           &KakiMinter,
    ) -> bool {
        self.decide(case_idx, target_kaki_bytes, epoch, journal, minter, AdvisoryDecision::Rejected, EventCause::NinsunAdvisoryRejected)
    }

    fn decide(
        &mut self,
        case_idx:          usize,
        target_kaki_bytes: [u8; 16],
        epoch:             u32,
        journal:           &mut Journal,
        minter:            &KakiMinter,
        decision:          AdvisoryDecision,
        cause:             EventCause,
    ) -> bool {
        let Some(case) = self.cases.get_mut(case_idx) else { return false; };
        if case.decision != AdvisoryDecision::Pending { return false; }
        case.decision = decision;

        if let Some(target_kaki) = reconstruct_identity(&target_kaki_bytes) {
            let event_kaki = EventKaki::try_from_kaki(minter.event(KakiRole::Zikru))
                .expect("KakiMinter always produces valid event KAKIs");
            let entry = JournalEntry::new(event_kaki, target_kaki, epoch, Vec::new())
                .with_event_cause(cause);
            let _ = journal.append(entry);
        }
        true
    }
}

impl Default for NinsunAdvisoryQueue {
    fn default() -> Self { Self::new() }
}

fn reconstruct_identity(bytes: &[u8; 16]) -> Option<IdentityKaki> {
    Kaki::from_bytes(*bytes).ok()
        .and_then(|k| IdentityKaki::try_from_kaki(k).ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bahyway_core::TribeId;

    fn make_minter() -> KakiMinter {
        KakiMinter::new(TribeId::from_u16(0x0001))
    }

    fn make_proposal(confidence: f64, target_kaki: &str) -> RefineProposal {
        RefineProposal {
            proposal_id:   "p1".into(),
            target_kaki:   target_kaki.into(),
            tribe_id:      "test_tribe".into(),
            drift_pattern: "STUCK_SENSOR_6HR".into(),
            explanation:   "value unchanged for 6 hours".into(),
            suggested_eav: vec![("ninsun_reviewed".into(), "true".into())],
            confidence,
        }
    }

    #[test]
    fn receive_sorts_by_descending_confidence() {
        let mut queue = NinsunAdvisoryQueue::new();
        queue.receive(vec![
            make_proposal(0.4, "k1"),
            make_proposal(0.9, "k2"),
            make_proposal(0.7, "k3"),
        ]);

        let ordered: Vec<f64> = queue.cases().iter().map(|c| c.proposal.confidence).collect();
        assert_eq!(ordered, vec![0.9, 0.7, 0.4]);
    }

    #[test]
    fn pending_excludes_decided_cases() {
        let minter = make_minter();
        let ik = enkidb_kaki::IdentityKaki::try_from_kaki(minter.identity(KakiRole::Zikru)).unwrap();
        let mut jnl = Journal::new(64);

        let mut queue = NinsunAdvisoryQueue::new();
        queue.receive(vec![make_proposal(0.9, "k1"), make_proposal(0.8, "k2")]);

        assert_eq!(queue.pending().len(), 2);
        queue.confirm(0, *ik.bytes(), 1, &mut jnl, &minter);
        assert_eq!(queue.pending().len(), 1);
    }

    #[test]
    fn confirm_writes_journal_entry_with_correct_cause() {
        let minter = make_minter();
        let ik = enkidb_kaki::IdentityKaki::try_from_kaki(minter.identity(KakiRole::Zikru)).unwrap();
        let mut jnl = Journal::new(64);

        let mut queue = NinsunAdvisoryQueue::new();
        queue.receive(vec![make_proposal(0.9, "k1")]);

        let ok = queue.confirm(0, *ik.bytes(), 1, &mut jnl, &minter);
        assert!(ok);
        assert_eq!(jnl.entry_count(), 1);
        assert_eq!(queue.cases()[0].decision, AdvisoryDecision::Confirmed);
    }

    #[test]
    fn reject_writes_journal_entry_with_correct_cause() {
        let minter = make_minter();
        let ik = enkidb_kaki::IdentityKaki::try_from_kaki(minter.identity(KakiRole::Zikru)).unwrap();
        let mut jnl = Journal::new(64);

        let mut queue = NinsunAdvisoryQueue::new();
        queue.receive(vec![make_proposal(0.5, "k1")]);

        let ok = queue.reject(0, *ik.bytes(), 1, &mut jnl, &minter);
        assert!(ok);
        assert_eq!(jnl.entry_count(), 1);
        assert_eq!(queue.cases()[0].decision, AdvisoryDecision::Rejected);
    }

    #[test]
    fn double_decision_is_rejected_as_noop() {
        let minter = make_minter();
        let ik = enkidb_kaki::IdentityKaki::try_from_kaki(minter.identity(KakiRole::Zikru)).unwrap();
        let mut jnl = Journal::new(64);

        let mut queue = NinsunAdvisoryQueue::new();
        queue.receive(vec![make_proposal(0.9, "k1")]);

        assert!(queue.confirm(0, *ik.bytes(), 1, &mut jnl, &minter));
        assert!(!queue.confirm(0, *ik.bytes(), 1, &mut jnl, &minter)); // already decided
        assert_eq!(jnl.entry_count(), 1); // second call wrote nothing
    }

    #[test]
    fn unknown_index_returns_false() {
        let minter = make_minter();
        let ik = enkidb_kaki::IdentityKaki::try_from_kaki(minter.identity(KakiRole::Zikru)).unwrap();
        let mut jnl = Journal::new(64);
        let mut queue = NinsunAdvisoryQueue::new();

        assert!(!queue.confirm(0, *ik.bytes(), 1, &mut jnl, &minter));
    }

    fn make_proposal_tribe(confidence: f64, target_kaki: &str, tribe_id: &str) -> RefineProposal {
        RefineProposal {
            proposal_id:   "p1".into(),
            target_kaki:   target_kaki.into(),
            tribe_id:      tribe_id.into(),
            drift_pattern: "STUCK_SENSOR_6HR".into(),
            explanation:   "value unchanged for 6 hours".into(),
            suggested_eav: vec![],
            confidence,
        }
    }

    // ── Hazard urgency integration (2026-07-10) ─────────────────────────
    #[test]
    fn low_confidence_earthquake_case_still_outranks_high_confidence_ordinary_case() {
        let dead_critical_smi = Smi::compute(1.0, 0.9, 0.9, 0.9, 0.9).unwrap();
        assert_eq!(dead_critical_smi.state, esarhaddon::smi::SmiState::DeadCritical);

        let mut queue = NinsunAdvisoryQueue::new();
        queue.receive_with_smi(vec![
            (make_proposal_tribe(0.99, "ordinary-k1", "5"), None),          // ordinary tribe, very confident
            (make_proposal_tribe(0.10, "quake-k1", "8192"), Some(dead_critical_smi)), // 0x2000, DeadCritical, low confidence
        ]);

        // DeadCritical structural case must sort first despite far lower confidence.
        assert_eq!(queue.cases()[0].proposal.target_kaki, "quake-k1");
        assert_eq!(queue.cases()[0].hazard, HazardDomain::Earthquake);
        assert_eq!(queue.cases()[1].proposal.target_kaki, "ordinary-k1");
        assert_eq!(queue.cases()[1].hazard, HazardDomain::None);
    }

    #[test]
    fn hazard_tagged_without_smi_yet_does_not_falsely_jump_the_queue() {
        // A case in the ESARHADDON tribe range but with no SMI computed
        // yet (e.g. sensors still fusing) must not be treated as urgent
        // by tribe membership alone -- urgency comes from the real SMI
        // score, not the tag.
        let mut queue = NinsunAdvisoryQueue::new();
        queue.receive_with_smi(vec![
            (make_proposal_tribe(0.95, "ordinary-high-conf", "5"), None),
            (make_proposal_tribe(0.50, "quake-no-smi-yet", "8192"), None),
        ]);
        assert_eq!(queue.cases()[0].proposal.target_kaki, "ordinary-high-conf");
    }

    #[test]
    fn receive_backward_compatible_with_plain_confidence_ordering() {
        // No hazard tribes involved: behaviour must match the pre-2026-07-10
        // pure-confidence sort exactly.
        let mut queue = NinsunAdvisoryQueue::new();
        queue.receive(vec![
            make_proposal(0.4, "k1"),
            make_proposal(0.9, "k2"),
            make_proposal(0.7, "k3"),
        ]);
        let ordered: Vec<f64> = queue.cases().iter().map(|c| c.proposal.confidence).collect();
        assert_eq!(ordered, vec![0.9, 0.7, 0.4]);
    }
}
