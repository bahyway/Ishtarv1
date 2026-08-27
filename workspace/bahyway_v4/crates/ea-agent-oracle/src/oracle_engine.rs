//! oracle_engine.rs — EaAgent Oracle Engine: unified tribe surveillance.
//!
//! Combines CollapsePredictor + PauliMonitor + GemOracle into one
//! sovereign early warning system. Called every epoch automatically.
#![forbid(unsafe_code)]

use crate::collapse_predictor::{CollapsePredictor, CollapseRisk};
use crate::gem_oracle::GemOracle;
use crate::pauli_monitor::PauliMonitor;
use ea_agent_algebra::JordanAnalyzer;
use ea_agent_core::{AgentDecision, DecisionKind, DecisionResult, ParticleSnapshot};

/// Alert severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlertLevel {
    Info,
    Warning,
    Critical,
    Emergency,
}

impl AlertLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Info => "ℹ INFO",
            Self::Warning => "⚠ WARNING",
            Self::Critical => "🔴 CRITICAL",
            Self::Emergency => "💀 EMERGENCY",
        }
    }
}

/// A single oracle alert.
#[derive(Debug, Clone)]
pub struct OracleAlert {
    pub level: AlertLevel,
    pub source: String,
    pub message: String,
    pub epoch: u64,
}

impl OracleAlert {
    pub fn info(source: &str, msg: &str, epoch: u64) -> Self {
        Self {
            level: AlertLevel::Info,
            source: source.into(),
            message: msg.into(),
            epoch,
        }
    }
    pub fn warning(source: &str, msg: &str, epoch: u64) -> Self {
        Self {
            level: AlertLevel::Warning,
            source: source.into(),
            message: msg.into(),
            epoch,
        }
    }
    pub fn critical(source: &str, msg: &str, epoch: u64) -> Self {
        Self {
            level: AlertLevel::Critical,
            source: source.into(),
            message: msg.into(),
            epoch,
        }
    }
    pub fn emergency(source: &str, msg: &str, epoch: u64) -> Self {
        Self {
            level: AlertLevel::Emergency,
            source: source.into(),
            message: msg.into(),
            epoch,
        }
    }
}

/// Full oracle report for one epoch.
#[derive(Debug)]
pub struct OracleReport {
    pub tribe_id: u16,
    pub epoch: u64,
    pub alerts: Vec<OracleAlert>,
    pub decisions: Vec<AgentDecision>,
    pub sovereign: bool,
    pub summary: String,
}

impl OracleReport {
    pub fn has_emergency(&self) -> bool {
        self.alerts.iter().any(|a| a.level == AlertLevel::Emergency)
    }
    pub fn has_critical(&self) -> bool {
        self.alerts.iter().any(|a| a.level >= AlertLevel::Critical)
    }
    pub fn alert_count(&self) -> usize {
        self.alerts.len()
    }
}

/// The unified EaAgent Oracle Engine.
pub struct OracleEngine {
    tribe_id: u16,
    collapse: CollapsePredictor,
    pauli: PauliMonitor,
    gem: GemOracle,
    jordan: JordanAnalyzer,
    pub epoch: u64,
    pub reports: Vec<OracleReport>,
}

impl OracleEngine {
    pub fn new(tribe_id: u16) -> Self {
        Self {
            tribe_id,
            collapse: CollapsePredictor::new(20),
            pauli: PauliMonitor::new(),
            gem: GemOracle::new(),
            jordan: JordanAnalyzer::new(),
            epoch: 0,
            reports: Vec::new(),
        }
    }

    /// Run one full oracle cycle over the current particle snapshot.
    /// Call this every epoch as particles change.
    pub fn tick(&mut self, particles: &[ParticleSnapshot]) -> &OracleReport {
        self.epoch += 1;
        let epoch = self.epoch;
        let tribe_id = self.tribe_id;
        let mut alerts = Vec::new();
        let mut decisions = Vec::new();

        // ── 1. Jordan Stability ───────────────────────────────────────────
        let jordan_result = self.jordan.analyze(tribe_id, particles);
        self.collapse
            .record(epoch, jordan_result.spectral_radius, particles.len());
        let collapse_pred = self.collapse.predict(tribe_id);

        match collapse_pred.risk {
            CollapseRisk::None => {}
            CollapseRisk::Low => alerts.push(OracleAlert::info(
                "Jordan",
                &collapse_pred.recommendation,
                epoch,
            )),
            CollapseRisk::Medium => alerts.push(OracleAlert::warning(
                "Jordan",
                &collapse_pred.recommendation,
                epoch,
            )),
            CollapseRisk::High => {
                alerts.push(OracleAlert::critical(
                    "Jordan",
                    &collapse_pred.recommendation,
                    epoch,
                ));
                decisions.push(AgentDecision::new(
                    DecisionKind::CollapseWarning,
                    DecisionResult::Critical(collapse_pred.recommendation.clone()),
                    epoch,
                    vec![format!("Tribe-0x{tribe_id:04X}")],
                ));
            }
            CollapseRisk::Critical => {
                alerts.push(OracleAlert::emergency(
                    "Jordan",
                    &collapse_pred.recommendation,
                    epoch,
                ));
                decisions.push(AgentDecision::new(
                    DecisionKind::CollapseWarning,
                    DecisionResult::Critical(format!(
                        "EMERGENCY: {}",
                        collapse_pred.recommendation
                    )),
                    epoch,
                    vec![format!("Tribe-0x{tribe_id:04X}")],
                ));
            }
        }

        // ── 2. Pauli Exclusion ────────────────────────────────────────────
        let pauli_report = self.pauli.scan(tribe_id, epoch, particles);
        if !pauli_report.sovereign {
            let msg = format!(
                "{} Pauli violation(s) detected",
                pauli_report.violation_count
            );
            if pauli_report.critical_count > 0 {
                alerts.push(OracleAlert::emergency("Pauli", &msg, epoch));
                for alert in &pauli_report.alerts {
                    decisions.push(AgentDecision::new(
                        DecisionKind::PauliViolation,
                        DecisionResult::Critical(alert.clone()),
                        epoch,
                        vec![format!("Tribe-0x{tribe_id:04X}")],
                    ));
                }
            } else {
                alerts.push(OracleAlert::warning("Pauli", &msg, epoch));
            }
        }

        // ── 3. GEM Rate ───────────────────────────────────────────────────
        let gem_report = self.gem.evaluate(tribe_id, epoch, particles);
        if !gem_report.meets_target() && !particles.is_empty() {
            alerts.push(OracleAlert::warning("GEM", &gem_report.assessment, epoch));
        } else if gem_report.meets_target() {
            decisions.push(AgentDecision::new(
                DecisionKind::HarmonyReport,
                DecisionResult::Sovereign(gem_report.summary()),
                epoch,
                vec![format!("Tribe-0x{tribe_id:04X}")],
            ));
        }

        // ── 4. Jordan Stability Decision ──────────────────────────────────
        decisions.push(AgentDecision::new(
            DecisionKind::JordanStability,
            if jordan_result.is_stable() {
                DecisionResult::Sovereign(jordan_result.summary())
            } else {
                DecisionResult::Warning(jordan_result.summary())
            },
            epoch,
            vec![format!("Tribe-0x{tribe_id:04X}")],
        ));

        let sovereign = alerts.iter().all(|a| a.level == AlertLevel::Info);
        let summary = self.build_summary(&alerts, particles.len(), epoch);

        self.reports.push(OracleReport {
            tribe_id,
            epoch,
            alerts,
            decisions,
            sovereign,
            summary,
        });
        self.reports.last().unwrap()
    }

    fn build_summary(&self, alerts: &[OracleAlert], particle_count: usize, epoch: u64) -> String {
        let max_level = alerts
            .iter()
            .map(|a| a.level)
            .max()
            .unwrap_or(AlertLevel::Info);
        format!(
            "Epoch {} | Tribe 0x{:04X} | {} particles | {} alerts | {}",
            epoch,
            self.tribe_id,
            particle_count,
            alerts.len(),
            max_level.as_str()
        )
    }

    /// All emergency alerts across all epochs.
    pub fn emergencies(&self) -> Vec<&OracleAlert> {
        self.reports
            .iter()
            .flat_map(|r| r.alerts.iter())
            .filter(|a| a.level == AlertLevel::Emergency)
            .collect()
    }

    /// Total KAKI-stamped decisions made.
    pub fn total_decisions(&self) -> usize {
        self.reports.iter().map(|r| r.decisions.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ea_agent_core::HeptaVector;

    fn make_p(name: &str, d: [f64; 7]) -> ParticleSnapshot {
        ParticleSnapshot::new(name, fnv(name), 0x0001, HeptaVector::new(d))
    }
    fn fnv(s: &str) -> u32 {
        let mut h: u32 = 2166136261;
        for b in s.bytes() {
            h ^= b as u32;
            h = h.wrapping_mul(16777619);
        }
        h
    }

    #[test]
    fn tick_increments_epoch() {
        let mut engine = OracleEngine::new(0x0001);
        let particles = vec![make_p("P1", [0.9; 7])];
        engine.tick(&particles);
        assert_eq!(engine.epoch, 1);
        engine.tick(&particles);
        assert_eq!(engine.epoch, 2);
    }

    #[test]
    fn healthy_tribe_is_sovereign() {
        let mut engine = OracleEngine::new(0x0001);
        let particles: Vec<_> = (0..5)
            .map(|i| {
                make_p(
                    &format!("GEM{i}"),
                    [0.95, 0.90, 0.85, 0.92, 0.88, 0.91, 0.93],
                )
            })
            .collect();
        for _ in 0..3 {
            engine.tick(&particles);
        }
        let last = engine.reports.last().unwrap();
        // Should have no collapse emergency (Jordan must be stable)
        let collapse_emergency = last
            .alerts
            .iter()
            .any(|a| a.level == AlertLevel::Emergency && a.source == "Jordan");
        assert!(
            !collapse_emergency,
            "healthy tribe should have no Jordan emergency"
        );
    }

    #[test]
    fn empty_tribe_handled() {
        let mut engine = OracleEngine::new(0x0001);
        let r = engine.tick(&[]);
        assert_eq!(r.epoch, 1);
    }

    #[test]
    fn decisions_are_kaki_stamped() {
        let mut engine = OracleEngine::new(0x0001);
        let particles: Vec<_> = (0..3).map(|i| make_p(&format!("P{i}"), [0.9; 7])).collect();
        engine.tick(&particles);
        assert!(engine.total_decisions() > 0);
        // Each decision has valid KAKI
        for report in &engine.reports {
            for d in &report.decisions {
                assert!(d.kaki.verify_checksum());
            }
        }
    }

    #[test]
    fn multiple_epochs_build_history() {
        let mut engine = OracleEngine::new(0x0001);
        let particles = vec![make_p("P1", [0.8; 7])];
        for _ in 0..5 {
            engine.tick(&particles);
        }
        assert_eq!(engine.reports.len(), 5);
    }
}
