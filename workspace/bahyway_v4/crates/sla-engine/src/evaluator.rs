//! SlaEvaluator — fuzzy compliance scoring against an SlaProfile.
//!
//! Scoring uses the same B11 scale (0–240, QUALITY_DIVISOR) as the rest of
//! the BahyWay quality system.  Domain scores and the overall composite score
//! are produced as B11 values so they can feed directly into `hepta-score`.

use std::collections::HashMap;

use crate::{
    profile::{MaturityLevel, SlaProfile},
    requirements::ComplianceDomain,
    GO_LIVE_THRESHOLD, MANDATORY_THRESHOLD, QUALITY_DIVISOR,
};

// ── ComplianceState ───────────────────────────────────────────────────────────

/// The current implementation state for each selected requirement.
///
/// This is what the SLA GUI collects from the administrator.  Map each
/// requirement ID to the `MaturityLevel` the admin selects.
#[derive(Debug, Clone, Default)]
pub struct ComplianceState {
    /// Map from requirement_id → MaturityLevel.
    levels: HashMap<u16, MaturityLevel>,
}

impl ComplianceState {
    pub fn new() -> Self {
        ComplianceState {
            levels: HashMap::new(),
        }
    }

    pub fn set(&mut self, requirement_id: u16, level: MaturityLevel) {
        self.levels.insert(requirement_id, level);
    }

    /// Level for a given requirement (defaults to `NotStarted` if not set).
    pub fn level(&self, requirement_id: u16) -> MaturityLevel {
        self.levels
            .get(&requirement_id)
            .copied()
            .unwrap_or(MaturityLevel::NotStarted)
    }

    /// Number of requirements explicitly set.
    pub fn set_count(&self) -> usize {
        self.levels.len()
    }
}

// ── ComplianceStatus ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComplianceStatus {
    /// B11 ≥ 200 (83%). Independent verification confirmed.
    Sovereign,
    /// B11 ≥ 180 (75%). Go-live safe.
    Compliant,
    /// B11 ≥ 120 (50%). Partial — proceed with caution.
    Partial,
    /// B11 < 120. Action required before go-live.
    ActionRequired,
    /// A mandatory requirement is below MANDATORY_THRESHOLD.
    /// Blocks go-live regardless of overall score.
    MandatoryGap,
}

impl ComplianceStatus {
    pub fn from_b11(b11: u8, has_mandatory_gap: bool) -> Self {
        if has_mandatory_gap {
            return ComplianceStatus::MandatoryGap;
        }
        if b11 >= 200 {
            ComplianceStatus::Sovereign
        } else if b11 >= 180 {
            ComplianceStatus::Compliant
        } else if b11 >= 120 {
            ComplianceStatus::Partial
        } else {
            ComplianceStatus::ActionRequired
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            ComplianceStatus::Sovereign => "Sovereign",
            ComplianceStatus::Compliant => "Compliant",
            ComplianceStatus::Partial => "Partial",
            ComplianceStatus::ActionRequired => "Action Required",
            ComplianceStatus::MandatoryGap => "Mandatory Gap — BLOCKS GO-LIVE",
        }
    }

    pub fn css_class(self) -> &'static str {
        match self {
            ComplianceStatus::Sovereign => "sovereign",
            ComplianceStatus::Compliant => "compliant",
            ComplianceStatus::Partial => "partial",
            ComplianceStatus::ActionRequired => "action-required",
            ComplianceStatus::MandatoryGap => "mandatory-gap",
        }
    }
}

// ── ComplianceCheckResult ─────────────────────────────────────────────────────

/// Score for one individual requirement.
#[derive(Debug, Clone)]
pub struct ComplianceCheckResult {
    pub requirement_id: u16,
    pub name: &'static str,
    pub domain: ComplianceDomain,
    pub maturity: MaturityLevel,
    /// Fuzzy score [0.0, 1.0].
    pub fuzzy_score: f32,
    /// B11 score (fuzzy × weight × 240 / 240).
    pub b11_score: u8,
    pub mandatory: bool,
    pub blocks_go_live: bool,
    pub gap_action: Option<&'static str>,
    pub akk_policy: Option<&'static str>,
}

// ── DomainScore ───────────────────────────────────────────────────────────────

/// Aggregated compliance score for one domain.
#[derive(Debug, Clone)]
pub struct DomainScore {
    pub domain: ComplianceDomain,
    pub b11_score: u8,
    pub status: ComplianceStatus,
    pub checks: Vec<ComplianceCheckResult>,
    pub mandatory_gap: bool,
    pub top_gap_actions: Vec<&'static str>,
}

impl DomainScore {
    /// Progress as a percentage (0–100).
    pub fn percent(&self) -> u8 {
        ((self.b11_score as f32 / QUALITY_DIVISOR as f32) * 100.0).round() as u8
    }
}

// ── SlaReport ─────────────────────────────────────────────────────────────────

/// Full compliance report produced by `SlaEvaluator::evaluate()`.
#[derive(Debug, Clone)]
pub struct SlaReport {
    pub profile_name: &'static str,
    pub topology: &'static str,
    pub overall_b11: u8,
    pub overall_status: ComplianceStatus,
    pub go_live_ready: bool,
    /// Per-domain breakdown.
    pub domains: Vec<DomainScore>,
    /// Top-priority action items (mandatory gaps first, then by weight desc).
    pub priority_actions: Vec<PriorityAction>,
    /// Epoch at which this report was generated.
    pub generated_epoch: u32,
}

impl SlaReport {
    pub fn overall_percent(&self) -> u8 {
        ((self.overall_b11 as f32 / QUALITY_DIVISOR as f32) * 100.0).round() as u8
    }
}

/// A single prioritised action item.
#[derive(Debug, Clone)]
pub struct PriorityAction {
    pub requirement_id: u16,
    pub name: &'static str,
    pub domain: ComplianceDomain,
    pub priority: ActionPriority,
    pub action: &'static str,
    pub akk_policy: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ActionPriority {
    P0Blocking = 0, // mandatory gap — blocks go-live
    P1Required = 1, // required before go-live
    P2Soon = 2,     // required within 90 days
    P3Roadmap = 3,  // 12-month roadmap
}

impl ActionPriority {
    pub fn as_str(self) -> &'static str {
        match self {
            ActionPriority::P0Blocking => "P0 — GO-LIVE BLOCKER",
            ActionPriority::P1Required => "P1 — Required Before Launch",
            ActionPriority::P2Soon => "P2 — Required Within 90 Days",
            ActionPriority::P3Roadmap => "P3 — 12-Month Roadmap",
        }
    }

    pub fn css_class(self) -> &'static str {
        match self {
            ActionPriority::P0Blocking => "p0",
            ActionPriority::P1Required => "p1",
            ActionPriority::P2Soon => "p2",
            ActionPriority::P3Roadmap => "p3",
        }
    }
}

// ── SlaEvaluator ─────────────────────────────────────────────────────────────

pub struct SlaEvaluator;

impl SlaEvaluator {
    /// Evaluate a `ComplianceState` against an `SlaProfile`.
    /// Returns a full `SlaReport` with fuzzy scores, B11 values, and actions.
    pub fn evaluate(profile: &SlaProfile, state: &ComplianceState, now_epoch: u32) -> SlaReport {
        // ── Evaluate each selected requirement ────────────────────────────────
        let checks: Vec<ComplianceCheckResult> = profile
            .selected_ids
            .iter()
            .filter_map(|&id| {
                let req = crate::requirements::requirement_by_id(id)?;
                let mat = state.level(id);
                // NotApplicable → excluded from scoring
                if mat == MaturityLevel::NotApplicable {
                    return None;
                }
                let fuzzy = mat.fuzzy_score();
                let b11 = ((fuzzy * req.weight as f32 / QUALITY_DIVISOR as f32)
                    * QUALITY_DIVISOR as f32)
                    .round() as u8;
                let blocks = req.mandatory && b11 < MANDATORY_THRESHOLD;
                Some(ComplianceCheckResult {
                    requirement_id: id,
                    name: req.name,
                    domain: req.domain,
                    maturity: mat,
                    fuzzy_score: fuzzy,
                    b11_score: b11,
                    mandatory: req.mandatory,
                    blocks_go_live: blocks,
                    gap_action: if fuzzy < 1.0 {
                        Some(req.gap_action)
                    } else {
                        None
                    },
                    akk_policy: req.akk_policy,
                })
            })
            .collect();

        // ── Aggregate into domain scores ──────────────────────────────────────
        let all_domains = [
            ComplianceDomain::GdprPrivacy,
            ComplianceDomain::EncryptionAtRest,
            ComplianceDomain::KeyManagement,
            ComplianceDomain::NetworkSecurity,
            ComplianceDomain::OperationalSecurity,
            ComplianceDomain::NajafPrivacy,
            ComplianceDomain::WpdInfrastructure,
            ComplianceDomain::GovernanceAssurance,
        ];

        let mut domain_scores: Vec<DomainScore> = Vec::new();

        for domain in all_domains {
            let domain_checks: Vec<&ComplianceCheckResult> =
                checks.iter().filter(|c| c.domain == domain).collect();

            if domain_checks.is_empty() {
                continue;
            }

            let mandatory_gap = domain_checks.iter().any(|c| c.blocks_go_live);

            // Weighted average of b11 scores within this domain
            let domain_weight = profile.domain_weight(domain) as f32;
            let (sum_weighted, sum_weights) =
                domain_checks.iter().fold((0f32, 0f32), |(ws, wt), c| {
                    let req = crate::requirements::requirement_by_id(c.requirement_id);
                    let w = req.map(|r| r.weight as f32).unwrap_or(120.0);
                    (ws + c.b11_score as f32 * w, wt + w)
                });
            let raw_avg = if sum_weights > 0.0 {
                sum_weighted / sum_weights
            } else {
                0.0
            };
            // Scale by domain weight influence
            let scaled = (raw_avg * domain_weight / 240.0).round() as u8;

            let status = ComplianceStatus::from_b11(scaled, mandatory_gap);

            let top_gaps: Vec<&'static str> = domain_checks
                .iter()
                .filter_map(|c| c.gap_action)
                .take(3)
                .collect();

            domain_scores.push(DomainScore {
                domain,
                b11_score: scaled,
                status,
                checks: domain_checks.iter().map(|c| (*c).clone()).collect(),
                mandatory_gap,
                top_gap_actions: top_gaps,
            });
        }

        // ── Overall score ─────────────────────────────────────────────────────
        let (ov_sum, ov_weights) = domain_scores.iter().fold((0f32, 0f32), |(s, w), d| {
            let dw = profile.domain_weight(d.domain) as f32;
            (s + d.b11_score as f32 * dw, w + dw)
        });
        let overall_b11 = if ov_weights > 0.0 {
            (ov_sum / ov_weights).round() as u8
        } else {
            0
        };

        let any_mandatory_gap = domain_scores.iter().any(|d| d.mandatory_gap);
        let go_live_ready = !any_mandatory_gap && overall_b11 >= GO_LIVE_THRESHOLD;
        let overall_status = ComplianceStatus::from_b11(overall_b11, any_mandatory_gap);

        // ── Priority action list ──────────────────────────────────────────────
        let mut actions: Vec<PriorityAction> = checks
            .iter()
            .filter(|c| c.gap_action.is_some())
            .map(|c| {
                let priority = if c.blocks_go_live {
                    ActionPriority::P0Blocking
                } else if c.mandatory {
                    ActionPriority::P1Required
                } else {
                    let req = crate::requirements::requirement_by_id(c.requirement_id);
                    let w = req.map(|r| r.weight).unwrap_or(100);
                    if w >= 200 {
                        ActionPriority::P2Soon
                    } else {
                        ActionPriority::P3Roadmap
                    }
                };
                PriorityAction {
                    requirement_id: c.requirement_id,
                    name: c.name,
                    domain: c.domain,
                    priority,
                    action: c.gap_action.unwrap_or(""),
                    akk_policy: c.akk_policy,
                }
            })
            .collect();
        actions.sort_by_key(|a| (a.priority as u8, a.domain as u8));

        SlaReport {
            profile_name: profile.name,
            topology: profile.topology.as_str(),
            overall_b11,
            overall_status,
            go_live_ready,
            domains: domain_scores,
            priority_actions: actions,
            generated_epoch: now_epoch,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::{AppTopology, SlaProfile};

    fn najaf_profile() -> SlaProfile {
        SlaProfile::for_topology(AppTopology::NajafEngine)
    }

    #[test]
    fn empty_state_produces_low_score_for_najaf() {
        let profile = najaf_profile();
        let state = ComplianceState::new();
        let report = SlaEvaluator::evaluate(&profile, &state, 1_000);
        assert!(
            !report.go_live_ready,
            "empty state must not be go-live ready"
        );
        assert!(report.overall_b11 < GO_LIVE_THRESHOLD);
    }

    #[test]
    fn fully_verified_state_is_go_live_ready() {
        let profile = najaf_profile();
        let mut state = ComplianceState::new();
        for &id in &profile.selected_ids {
            state.set(id, MaturityLevel::Verified);
        }
        let report = SlaEvaluator::evaluate(&profile, &state, 1_000);
        assert!(report.go_live_ready, "fully verified must be go-live ready");
        assert!(report.overall_b11 >= GO_LIVE_THRESHOLD);
    }

    #[test]
    fn mandatory_gap_blocks_go_live_even_if_overall_high() {
        let profile = najaf_profile();
        let mut state = ComplianceState::new();
        // Set everything to Verified except the mandatory DPIA (1001)
        for &id in &profile.selected_ids {
            state.set(id, MaturityLevel::Verified);
        }
        state.set(1001, MaturityLevel::NotStarted); // DPIA not done
        let report = SlaEvaluator::evaluate(&profile, &state, 1_000);
        assert!(!report.go_live_ready, "mandatory gap must block go-live");
        assert!(report.domains.iter().any(|d| d.mandatory_gap));
    }

    #[test]
    fn priority_actions_ordered_p0_first() {
        let profile = najaf_profile();
        let mut state = ComplianceState::new();
        state.set(1001, MaturityLevel::NotStarted); // DPIA — mandatory
        state.set(1007, MaturityLevel::NotStarted); // privacy notice — not mandatory
        let report = SlaEvaluator::evaluate(&profile, &state, 1_000);
        if let Some(first) = report.priority_actions.first() {
            assert_eq!(
                first.priority,
                ActionPriority::P0Blocking,
                "first action must be P0 blocking"
            );
        }
    }

    #[test]
    fn not_applicable_requirements_excluded_from_scoring() {
        let profile = najaf_profile();
        let mut state = ComplianceState::new();
        // Mark DPIA as N/A — should not count against the score
        state.set(1001, MaturityLevel::NotApplicable);
        let report = SlaEvaluator::evaluate(&profile, &state, 1_000);
        // N/A items should not appear in actions
        assert!(!report
            .priority_actions
            .iter()
            .any(|a| a.requirement_id == 1001));
    }

    #[test]
    fn enterprise_profile_evaluates_without_panic() {
        let profile = SlaProfile::for_topology(AppTopology::EnterpriseAll);
        let state = ComplianceState::new();
        let _report = SlaEvaluator::evaluate(&profile, &state, 2_000);
    }
}
