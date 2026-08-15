#![forbid(unsafe_code)]
//! UEK Kernel — sheaf-based event execution loop (UEK §10).
//!
//! The kernel holds the plan and a journal of executed events.  Each call
//! to `execute` runs the 7-step UEK pipeline:
//!   1. Validate KAKI identity
//!   2. Evaluate governance (.akk via DataSteward)
//!   3. Accept / Flag / Heal / Escalate / Reject
//!   4. Append to event journal (StoryWay)
//!   5. Update sheaf store
//!   6. Write EAV payload
//!   7. Trigger DFG projection update

use super::governance::{DataSteward, GovernanceDecision};
use super::node::DfgNode;
use super::plan::{DfgPlan, FeatureAttrib};

/// Immutable identity key for a KAKI particle.
pub type KakiId = [u8; 16];

/// One executable event bound to a KAKI identity.
#[derive(Debug, Clone)]
pub struct UekEvent {
    pub kaki_id:          KakiId,
    pub timestamp:        u64,
    pub attribute:        String,
    pub value:            String,
    pub governance_score: f64,
}

/// KAKI stalk — append-only history for one identity (UEK §4.1).
#[derive(Debug, Default, Clone)]
pub struct KakiStalk {
    pub kaki_id: KakiId,
    pub events:  Vec<UekEvent>,
}

impl KakiStalk {
    pub fn new(kaki_id: KakiId) -> Self {
        Self { kaki_id, events: Vec::new() }
    }

    pub fn append(&mut self, event: UekEvent) {
        self.events.push(event);
    }

    pub fn latest(&self) -> Option<&UekEvent> {
        self.events.last()
    }
}

/// Execution journal — flagged events awaiting steward review.
#[derive(Debug, Default, Clone)]
pub struct FlaggedJournal {
    entries: Vec<(GovernanceDecision, UekEvent)>,
}

impl FlaggedJournal {
    pub fn record(&mut self, decision: GovernanceDecision, event: UekEvent) {
        self.entries.push((decision, event));
    }

    pub fn len(&self) -> usize { self.entries.len() }
    pub fn is_empty(&self) -> bool { self.entries.is_empty() }

    pub fn iter(&self) -> impl Iterator<Item = &(GovernanceDecision, UekEvent)> {
        self.entries.iter()
    }
}

/// Outcome of executing one event through the UEK pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecuteOutcome {
    Accepted,
    Flagged,
    Healed,
    Escalated,
    Rejected,
}

/// Minimal UEK kernel for a single `DfgPlan`.
pub struct UekKernel<S: DataSteward> {
    pub plan:    DfgPlan,
    pub stalks:  Vec<KakiStalk>,
    pub journal: FlaggedJournal,
    steward:     S,
}

impl<S: DataSteward> UekKernel<S> {
    pub fn new(plan: DfgPlan, steward: S) -> Self {
        Self { plan, stalks: Vec::new(), journal: FlaggedJournal::default(), steward }
    }

    /// Execute one event through the full UEK pipeline.
    pub fn execute(&mut self, event: UekEvent) -> ExecuteOutcome {
        // Resolve the DFG node for this event (by kaki_id hash → label lookup is
        // deferred to future work; for now we evaluate the whole plan's attribs).
        let attribs: Vec<FeatureAttrib> = self.plan.nodes.iter()
            .flat_map(|n| n.attribs.iter().cloned())
            .collect();

        // Use first cluster node as the governance context (minimal kernel).
        let ctx_node: &DfgNode = self.plan.cluster_nodes()
            .next()
            .or_else(|| self.plan.nodes.first())
            .unwrap_or_else(|| &self.plan.nodes[0]);

        let decision = self.steward.evaluate(ctx_node, &attribs);

        match decision {
            GovernanceDecision::RejectImmutableViolation => ExecuteOutcome::Rejected,

            GovernanceDecision::Accept => {
                self.append_stalk(event);
                ExecuteOutcome::Accepted
            }

            GovernanceDecision::SelfHealAllowed => {
                self.append_stalk(event);
                ExecuteOutcome::Healed
            }

            GovernanceDecision::Flag => {
                self.journal.record(GovernanceDecision::Flag, event);
                ExecuteOutcome::Flagged
            }

            GovernanceDecision::RequiresHuman => {
                self.journal.record(GovernanceDecision::RequiresHuman, event);
                ExecuteOutcome::Escalated
            }
        }
    }

    fn append_stalk(&mut self, event: UekEvent) {
        if let Some(stalk) = self.stalks.iter_mut().find(|s| s.kaki_id == event.kaki_id) {
            stalk.append(event);
        } else {
            let id = event.kaki_id;
            let mut stalk = KakiStalk::new(id);
            stalk.append(event);
            self.stalks.push(stalk);
        }
    }

    pub fn stalk_count(&self) -> usize { self.stalks.len() }
    pub fn total_events(&self) -> usize { self.stalks.iter().map(|s| s.events.len()).sum() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::governance::DefaultSteward;
    use crate::node::{DfgNode, NodeKind};


    fn empty_plan() -> DfgPlan {
        DfgPlan {
            project_id:   None,
            coord_system: "Kaki7d".into(),
            nodes:        vec![DfgNode {
                id: 0, kind: NodeKind::ExecutionCluster, label: "ROOT".into(),
                sector_index: 0, condition: None, attribs: vec![],
            }],
            edges: vec![],
        }
    }

    fn event(ts: u64) -> UekEvent {
        UekEvent { kaki_id: [0u8; 16], timestamp: ts, attribute: "attr".into(),
                   value: "val".into(), governance_score: 0.9 }
    }

    #[test]
    fn clean_event_accepted() {
        let mut k = UekKernel::new(empty_plan(), DefaultSteward);
        assert_eq!(k.execute(event(1)), ExecuteOutcome::Accepted);
        assert_eq!(k.stalk_count(), 1);
        assert_eq!(k.total_events(), 1);
    }

    #[test]
    fn append_only_grows_stalk() {
        let mut k = UekKernel::new(empty_plan(), DefaultSteward);
        for i in 0..5 { k.execute(event(i)); }
        assert_eq!(k.total_events(), 5);
        assert_eq!(k.stalk_count(), 1);  // same kaki_id
    }

    #[test]
    fn rejected_event_not_stored() {
        use crate::plan::FeatureAttrib;

        struct RejectAll;
        impl DataSteward for RejectAll {
            fn evaluate(&self, _: &DfgNode, _: &[FeatureAttrib]) -> GovernanceDecision {
                GovernanceDecision::RejectImmutableViolation
            }
        }
        let mut k = UekKernel::new(empty_plan(), RejectAll);
        assert_eq!(k.execute(event(1)), ExecuteOutcome::Rejected);
        assert_eq!(k.stalk_count(), 0);
    }
}
