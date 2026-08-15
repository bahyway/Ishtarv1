//! NarrativeBuilder — StoryWay plain-language output for a DMW analysis.
//!
//! Converts the technical bottleneck and journey data into a self-contained
//! human narrative that non-technical stakeholders can read and act on.
//!
//! # Example output
//!
//! > Query 'SalesOrder' reached Stable state at 77.3% alignment.
//! > The particle passed 6 of 7 gates but was blocked by: 2 chained outer joins
//! > threatening deadlock under concurrent writes.
//! > Priority action: Stage Outer Joins — expected gain +15% alignment.
//! > Verdict: 1 rotation(s) will elevate this query to Golden orbit.

use crate::bottleneck::{Bottleneck, BottleneckKind};
use crate::journey::{DmwState, JourneyLevelId, SevenLevelJourney};
use crate::oracle::Recommendation;

/// Human-readable narrative derived from a DMW analysis.
#[derive(Debug, Clone)]
pub struct Narrative {
    /// One-line summary of the query's health state.
    pub headline: String,
    /// Story of the 7-gate journey — what passed and what blocked the particle.
    pub journey_story: String,
    /// The single highest-priority recommended action (`None` when clean).
    pub top_action: Option<String>,
    /// Final verdict — what must happen next.
    pub verdict: String,
}

impl Narrative {
    /// Full narrative as a single block of text.
    pub fn full_text(&self) -> String {
        let mut parts = vec![self.headline.as_str(), self.journey_story.as_str()];
        if let Some(ref a) = self.top_action {
            parts.push(a.as_str());
        }
        parts.push(self.verdict.as_str());
        parts.join("  ")
    }
}

/// Stateless builder that converts DMW analysis components into a `Narrative`.
pub struct NarrativeBuilder;

impl NarrativeBuilder {
    /// Build a narrative from the raw analysis components.
    pub fn build(
        query_id: &str,
        state: DmwState,
        alignment_pct: f32,
        journey: &SevenLevelJourney,
        bottlenecks: &[Bottleneck],
        recommendations: &[Recommendation],
    ) -> Narrative {
        Narrative {
            headline:      Self::headline(query_id, state, alignment_pct),
            journey_story: Self::journey_story(journey, bottlenecks),
            top_action:    Self::top_action(recommendations),
            verdict:       Self::verdict(state, recommendations.len()),
        }
    }

    fn headline(query_id: &str, state: DmwState, pct: f32) -> String {
        format!(
            "Query '{}' reached {} state at {:.1}% alignment.",
            query_id,
            state.label(),
            pct
        )
    }

    fn journey_story(journey: &SevenLevelJourney, bottlenecks: &[Bottleneck]) -> String {
        let passed = journey.passed_levels();
        let failed = journey.failed_levels();

        if failed == 0 {
            return "The particle passed all 7 gates cleanly — orbit is stable and sovereign."
                .to_string();
        }

        let fragments: Vec<String> = bottlenecks
            .iter()
            .map(|b| Self::describe_bottleneck(b))
            .collect();

        format!(
            "The particle passed {} of 7 gates but was blocked by: {}.",
            passed,
            fragments.join("; ")
        )
    }

    fn describe_bottleneck(b: &Bottleneck) -> String {
        match &b.kind {
            BottleneckKind::NestedLoopOnLargeSet { inner_rows } => format!(
                "a nested loop working against {} rows causing O(n\u{00B2}) work",
                inner_rows
            ),
            BottleneckKind::CartesianProduct => {
                "a missing join condition causing catastrophic row explosion".to_string()
            }
            BottleneckKind::UnsargablePredicate { table } => format!(
                "a non-sargable filter blocking index seek on table '{}'",
                table
            ),
            BottleneckKind::KeyLookupBottleneck { lookup_count } => format!(
                "{} key lookup(s) returning to the clustered index per row",
                lookup_count
            ),
            BottleneckKind::StaleStatistics { ratio, .. } => format!(
                "statistics off by {:.0}\u{00D7} forcing the optimizer to choose the wrong plan",
                ratio
            ),
            BottleneckKind::ExcessiveSort => {
                "a blocking sort operator with no supporting index".to_string()
            }
            BottleneckKind::SpoolRecompute => {
                "an eager spool materialising the inner set on every outer row".to_string()
            }
            BottleneckKind::DeadlockRisk { join_count } => format!(
                "{} chained outer joins threatening deadlock under concurrent writes",
                join_count
            ),
        }
    }

    fn top_action(recommendations: &[Recommendation]) -> Option<String> {
        recommendations.first().map(|r| {
            format!(
                "Priority action: {} — expected gain +{:.0}% alignment.",
                r.title, r.alignment_gain_pct
            )
        })
    }

    fn verdict(state: DmwState, rec_count: usize) -> String {
        match state {
            DmwState::Golden => {
                "Verdict: This particle is sovereign — no intervention required.".to_string()
            }
            DmwState::Stable => format!(
                "Verdict: {} rotation(s) will elevate this query to Golden orbit.",
                rec_count
            ),
            DmwState::Turbulent => format!(
                "Verdict: {} rotation(s) needed — apply in priority order to restore stable orbit.",
                rec_count
            ),
            DmwState::Critical => format!(
                "Verdict: CRITICAL — {} rotation(s) required immediately to prevent system degradation.",
                rec_count
            ),
        }
    }

    /// Convenience: build narrative directly from a `DmwReport`.
    pub fn from_report(report: &crate::DmwReport) -> Narrative {
        Self::build(
            &report.query_id,
            report.state,
            report.alignment_pct,
            &report.journey,
            &report.bottlenecks,
            &report.recommendations,
        )
    }
}

/// Gate-by-gate summary for display in diagnostics dashboards.
///
/// Returns one line per Journey level: pass/fail indicator, level name, and finding.
pub fn gate_summary(journey: &SevenLevelJourney) -> Vec<String> {
    JourneyLevelId::all()
        .iter()
        .map(|id| {
            let icon    = if journey.level_score(*id) >= 1.0 { "✓" } else { "✗" };
            let finding = journey.level_finding(*id);
            format!("[{}] {} — {}", icon, id.label(), finding)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bottleneck::BottleneckDetector;
    use crate::journey::SevenLevelJourney;
    use crate::oracle::TheOracle;
    use crate::plan::{sales_order_plan, simple_nested_loop_plan, OpKind, PlanNode, QueryPlan, ScanType};
    use crate::DmwAnalyzer;

    fn narrative_for(plan: &QueryPlan) -> Narrative {
        let bns   = BottleneckDetector::detect(plan);
        let j     = SevenLevelJourney::analyze(plan, &bns);
        let recs  = TheOracle::advise(plan, &bns, &j);
        let state = j.state();
        let pct   = j.alignment_score();
        NarrativeBuilder::build(plan.query_id.as_str(), state, pct, &j, &bns, &recs)
    }

    #[test]
    fn headline_contains_query_id() {
        let n = narrative_for(&sales_order_plan());
        assert!(n.headline.contains("SalesOrder"), "headline={}", n.headline);
    }

    #[test]
    fn headline_contains_state_label() {
        let n = narrative_for(&sales_order_plan());
        let known_states = ["Golden", "Stable", "Turbulent", "Critical"];
        assert!(
            known_states.iter().any(|s| n.headline.contains(s)),
            "headline must name the state: {}", n.headline
        );
    }

    #[test]
    fn headline_contains_alignment_pct() {
        let n = narrative_for(&sales_order_plan());
        assert!(n.headline.contains('%'), "headline must include alignment %: {}", n.headline);
    }

    #[test]
    fn journey_story_mentions_gates() {
        let n = narrative_for(&sales_order_plan());
        assert!(n.journey_story.contains("7"), "story must mention 7 gates: {}", n.journey_story);
    }

    #[test]
    fn top_action_present_when_bottlenecks_exist() {
        let n = narrative_for(&simple_nested_loop_plan());
        assert!(n.top_action.is_some(), "simple nested-loop plan has bottlenecks → top_action expected");
    }

    #[test]
    fn top_action_contains_gain() {
        let n = narrative_for(&simple_nested_loop_plan());
        if let Some(ref a) = n.top_action {
            assert!(a.contains('%'), "top_action must mention alignment gain: {a}");
        }
    }

    #[test]
    fn perfect_plan_no_top_action() {
        let root = PlanNode::new(
            OpKind::IndexSeek { name: "PK_T".into(), scan_type: ScanType::Clustered },
            10.0, 100, 50,
        );
        let plan = QueryPlan::new("perfect", root, 10.0).with_actual_rows(100);
        let n = narrative_for(&plan);
        assert!(n.top_action.is_none(), "perfect plan has no bottlenecks → no top action");
    }

    #[test]
    fn perfect_plan_all_gates_story() {
        let root = PlanNode::new(
            OpKind::IndexSeek { name: "PK_T".into(), scan_type: ScanType::Clustered },
            10.0, 100, 50,
        );
        let plan = QueryPlan::new("perfect", root, 10.0).with_actual_rows(100);
        let n = narrative_for(&plan);
        assert!(
            n.journey_story.contains("all 7 gates"),
            "perfect plan story: {}", n.journey_story
        );
    }

    #[test]
    fn verdict_contains_rec_count_for_imperfect() {
        let n = narrative_for(&simple_nested_loop_plan());
        assert!(!n.verdict.is_empty());
        // Must mention number of rotations
        let has_digit = n.verdict.chars().any(|c| c.is_ascii_digit());
        assert!(has_digit, "verdict must contain rotation count: {}", n.verdict);
    }

    #[test]
    fn golden_verdict_says_sovereign() {
        let root = PlanNode::new(
            OpKind::IndexSeek { name: "PK_T".into(), scan_type: ScanType::Clustered },
            10.0, 100, 50,
        );
        let plan = QueryPlan::new("perfect", root, 10.0).with_actual_rows(100);
        let n = narrative_for(&plan);
        assert!(n.verdict.contains("sovereign"), "verdict={}", n.verdict);
    }

    #[test]
    fn full_text_non_empty() {
        let n = narrative_for(&sales_order_plan());
        assert!(!n.full_text().is_empty());
    }

    #[test]
    fn full_text_contains_headline() {
        let n = narrative_for(&sales_order_plan());
        assert!(n.full_text().contains(n.headline.as_str()));
    }

    #[test]
    fn from_report_matches_manual_build() {
        let plan   = sales_order_plan();
        let report = DmwAnalyzer::analyze(&plan);
        let n      = NarrativeBuilder::from_report(&report);
        assert!(!n.headline.is_empty());
        assert!(!n.journey_story.is_empty());
    }

    #[test]
    fn gate_summary_has_seven_entries() {
        let plan = sales_order_plan();
        let bns  = BottleneckDetector::detect(&plan);
        let j    = SevenLevelJourney::analyze(&plan, &bns);
        let gs   = gate_summary(&j);
        assert_eq!(gs.len(), 7);
    }

    #[test]
    fn gate_summary_entries_non_empty() {
        let plan = sales_order_plan();
        let bns  = BottleneckDetector::detect(&plan);
        let j    = SevenLevelJourney::analyze(&plan, &bns);
        for line in gate_summary(&j) {
            assert!(!line.is_empty());
        }
    }

    #[test]
    fn deadlock_risk_described_in_story() {
        let n = narrative_for(&sales_order_plan());
        assert!(
            n.journey_story.contains("outer join") || n.journey_story.contains("deadlock"),
            "SalesOrder story must mention outer joins or deadlock: {}", n.journey_story
        );
    }
}
