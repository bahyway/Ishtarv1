//! SlaProfile — stakeholder-configurable compliance requirement set.
//!
//! `AppTopology` provides pre-built profiles for each BahyWay app.
//! Administrators can adjust the selected requirements via the SLA GUI.

use crate::requirements::ComplianceDomain;

// ── MaturityLevel ─────────────────────────────────────────────────────────────

/// The implementation maturity of a single requirement.
/// Maps directly to a fuzzy membership value [0.0, 1.0].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MaturityLevel {
    /// Requirement not considered for this deployment.
    NotApplicable,
    /// Nothing started — 0% credit.
    NotStarted,
    /// Formally planned — architecture decision made, not yet built.  20%.
    Planned,
    /// Work in progress — partial implementation, not yet verified.  50%.
    InProgress,
    /// Fully implemented but not independently verified.  80%.
    Implemented,
    /// Implemented and independently verified (pen test, audit, etc.).  100%.
    Verified,
}

impl MaturityLevel {
    /// Fuzzy membership value [0.0, 1.0] for this maturity level.
    pub fn fuzzy_score(self) -> f32 {
        match self {
            MaturityLevel::NotApplicable => 1.0, // excluded — counts as neutral
            MaturityLevel::NotStarted    => 0.0,
            MaturityLevel::Planned       => 0.2,
            MaturityLevel::InProgress    => 0.5,
            MaturityLevel::Implemented   => 0.8,
            MaturityLevel::Verified      => 1.0,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            MaturityLevel::NotApplicable => "N/A",
            MaturityLevel::NotStarted    => "Not Started",
            MaturityLevel::Planned       => "Planned",
            MaturityLevel::InProgress    => "In Progress",
            MaturityLevel::Implemented   => "Implemented",
            MaturityLevel::Verified      => "Verified",
        }
    }

    pub fn css_class(self) -> &'static str {
        match self {
            MaturityLevel::NotApplicable => "na",
            MaturityLevel::NotStarted    => "not-started",
            MaturityLevel::Planned       => "planned",
            MaturityLevel::InProgress    => "in-progress",
            MaturityLevel::Implemented   => "implemented",
            MaturityLevel::Verified      => "verified",
        }
    }

    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => MaturityLevel::NotApplicable,
            1 => MaturityLevel::NotStarted,
            2 => MaturityLevel::Planned,
            3 => MaturityLevel::InProgress,
            4 => MaturityLevel::Implemented,
            5 => MaturityLevel::Verified,
            _ => MaturityLevel::NotStarted,
        }
    }

    pub fn as_u8(self) -> u8 {
        match self {
            MaturityLevel::NotApplicable => 0,
            MaturityLevel::NotStarted    => 1,
            MaturityLevel::Planned       => 2,
            MaturityLevel::InProgress    => 3,
            MaturityLevel::Implemented   => 4,
            MaturityLevel::Verified      => 5,
        }
    }
}

// ── AppTopology ───────────────────────────────────────────────────────────────

/// Identifies which BahyWay application topology is being governed.
/// Each topology selects a default set of requirements.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppTopology {
    /// Wadi al-Salam cemetery records — GDPR Article 9 special category.
    NajafEngine,
    /// Water pipeline data — NIS2 critical infrastructure.
    WpdPipeline,
    /// HeptaScript website — HTTP gateway security.
    WebGateway,
    /// EnkiDB core — replication, key management, CQRS isolation.
    EnkiDb,
    /// Full enterprise — all requirements from all topologies.
    EnterpriseAll,
}

impl AppTopology {
    pub fn as_str(self) -> &'static str {
        match self {
            AppTopology::NajafEngine   => "NajafEngine",
            AppTopology::WpdPipeline   => "WPD Pipeline",
            AppTopology::WebGateway    => "Web Gateway",
            AppTopology::EnkiDb        => "EnkiDB Core",
            AppTopology::EnterpriseAll => "BahyWay Enterprise (All)",
        }
    }
}

// ── SlaProfile ────────────────────────────────────────────────────────────────

/// A named compliance profile — the stakeholder's selected requirement set.
///
/// Created from an `AppTopology` default, then adjusted by the administrator
/// through the SLA GUI.
#[derive(Debug, Clone)]
pub struct SlaProfile {
    pub name:             &'static str,
    pub topology:         AppTopology,
    /// IDs of requirements selected for this profile.
    pub selected_ids:     Vec<u16>,
    /// Target B11 score (0–240) for this profile.
    pub target_b11:       u8,
    /// Domain-level weight overrides (domain → weight 0–240).
    /// Empty = use domain's default weight.
    pub domain_weights:   Vec<(ComplianceDomain, u8)>,
}

impl SlaProfile {
    /// Build the default profile for a given application topology.
    pub fn for_topology(topology: AppTopology) -> Self {
        match topology {
            AppTopology::NajafEngine => Self {
                name:           "NajafEngine Compliance Profile",
                topology,
                selected_ids:   vec![
                    // GDPR — all mandatory
                    1001, 1002, 1003, 1004, 1005, 1006, 1007,
                    // Encryption — all mandatory
                    2001, 2002, 2003, 2004,
                    // Key management
                    2102,
                    // Network
                    3001, 3002, 3003, 3004, 3005,
                    // Operational
                    4001, 4002, 4003,
                    // NajafEngine-specific — all mandatory
                    5001, 5002, 5003, 5004,
                    // Governance
                    7001,
                ],
                target_b11:     200, // GEM threshold — 83%
                domain_weights: vec![
                    (ComplianceDomain::GdprPrivacy,  240),
                    (ComplianceDomain::NajafPrivacy, 240),
                ],
            },

            AppTopology::WpdPipeline => Self {
                name:           "WPD Pipeline Compliance Profile",
                topology,
                selected_ids:   vec![
                    2002, 2003,          // volume + key encryption
                    3003, 3004, 3005,    // TLS, network isolation
                    4001, 4002,          // IR plan, pen test
                    6001, 6002, 6003,    // WPD-specific
                    7001,                // threat model
                ],
                target_b11:     180,
                domain_weights: vec![
                    (ComplianceDomain::WpdInfrastructure, 240),
                ],
            },

            AppTopology::WebGateway => Self {
                name:           "Web Gateway Compliance Profile",
                topology,
                selected_ids:   vec![
                    1007,                // privacy notice
                    3001, 3002, 3003,   // hepta-sec-web, rate limiting, TLS
                    4001, 4004,          // IR plan, disclosure policy
                    7001,                // threat model
                ],
                target_b11:     192,
                domain_weights: vec![
                    (ComplianceDomain::NetworkSecurity, 240),
                ],
            },

            AppTopology::EnkiDb => Self {
                name:           "EnkiDB Core Compliance Profile",
                topology,
                selected_ids:   vec![
                    2001, 2002, 2003, 2004, // encryption
                    2101, 2102,              // key management
                    3004, 3005,              // network isolation
                    4001, 4002, 4003,        // operational
                ],
                target_b11:     192,
                domain_weights: vec![
                    (ComplianceDomain::EncryptionAtRest, 240),
                    (ComplianceDomain::KeyManagement,    220),
                ],
            },

            AppTopology::EnterpriseAll => Self {
                name:           "BahyWay Enterprise Full Compliance",
                topology,
                selected_ids:   crate::requirements::ALL_REQUIREMENTS.iter().map(|r| r.id).collect(),
                target_b11:     180, // 75% — realistic for phased rollout
                domain_weights: vec![],
            },
        }
    }

    /// Add a requirement ID to the selection.
    pub fn add(&mut self, id: u16) {
        if !self.selected_ids.contains(&id) {
            self.selected_ids.push(id);
        }
    }

    /// Remove a requirement ID from the selection.
    pub fn remove(&mut self, id: u16) {
        self.selected_ids.retain(|&x| x != id);
    }

    /// Check if a requirement is in the selection.
    pub fn contains(&self, id: u16) -> bool {
        self.selected_ids.contains(&id)
    }

    /// Domain weight, using override if set, or domain default.
    pub fn domain_weight(&self, domain: ComplianceDomain) -> u8 {
        self.domain_weights.iter()
            .find(|(d, _)| *d == domain)
            .map(|(_, w)| *w)
            .unwrap_or_else(|| domain.weight())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maturity_level_fuzzy_scores_ordered() {
        assert!(MaturityLevel::NotStarted.fuzzy_score() < MaturityLevel::Planned.fuzzy_score());
        assert!(MaturityLevel::Planned.fuzzy_score() < MaturityLevel::InProgress.fuzzy_score());
        assert!(MaturityLevel::Implemented.fuzzy_score() < MaturityLevel::Verified.fuzzy_score());
    }

    #[test]
    fn maturity_level_roundtrip() {
        for i in 0u8..=5 {
            assert_eq!(MaturityLevel::from_u8(i).as_u8(), i);
        }
    }

    #[test]
    fn not_applicable_scores_neutral() {
        assert!((MaturityLevel::NotApplicable.fuzzy_score() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn najaf_profile_contains_mandatory_requirements() {
        let p = SlaProfile::for_topology(AppTopology::NajafEngine);
        // Must include all GDPR mandatory reqs
        assert!(p.contains(1001)); // DPIA
        assert!(p.contains(1004)); // erasure mechanism
        assert!(p.contains(5001)); // PII sidecar
    }

    #[test]
    fn enterprise_profile_contains_all_requirements() {
        let p = SlaProfile::for_topology(AppTopology::EnterpriseAll);
        assert!(p.contains(1001));
        assert!(p.contains(6001));
        assert!(p.contains(7003));
    }

    #[test]
    fn profile_add_remove() {
        let mut p = SlaProfile::for_topology(AppTopology::WebGateway);
        p.add(9999);
        assert!(p.contains(9999));
        p.remove(9999);
        assert!(!p.contains(9999));
    }
}
