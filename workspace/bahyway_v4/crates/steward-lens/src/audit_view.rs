#![forbid(unsafe_code)]
//! AuditView — read-only snapshot of the current anomaly state for a steward session.
//!
//! StewardLens never writes to EnkiDB; AuditView is a pure in-memory summary
//! of what the lens has observed.  It is rebuilt on each tick from live data.

use crate::alert::{AlertSeverity, StewardAlert};
use crate::device::DeviceTier;

/// Read-only anomaly state snapshot for one steward session tick.
#[derive(Debug, Default)]
pub struct AuditView {
    /// All alerts emitted in this session, newest first.
    pub alerts: Vec<StewardAlert>,
    /// The highest severity seen in this session.
    pub peak_severity: Option<AlertSeverity>,
    /// Number of alerts that require a Šàtamu decision and have not been cleared.
    pub pending_satamu_decisions: u32,
    /// The BASE orbital counter at the time this view was built.
    pub snapshot_orbital: u64,
}

impl AuditView {
    pub fn new(snapshot_orbital: u64) -> Self {
        Self {
            snapshot_orbital,
            ..Default::default()
        }
    }

    /// Ingest a new alert into the view.
    pub fn push(&mut self, alert: StewardAlert) {
        let sev = alert.severity;
        if self.peak_severity.is_none_or(|p| sev > p) {
            self.peak_severity = Some(sev);
        }
        if alert.requires_satamu {
            self.pending_satamu_decisions += 1;
        }
        self.alerts.insert(0, alert); // newest first
    }

    /// Alerts visible on the given device tier (filters by `min_tier`).
    pub fn alerts_for_tier(&self, tier: DeviceTier) -> impl Iterator<Item = &StewardAlert> {
        self.alerts.iter().filter(move |a| a.min_tier <= tier)
    }

    /// True if any MAROON alert is present and unacknowledged.
    pub fn has_emergency(&self) -> bool {
        self.peak_severity == Some(AlertSeverity::Maroon)
    }

    /// Count of actionable (KAKKAB+) alerts.
    pub fn actionable_count(&self) -> usize {
        self.alerts
            .iter()
            .filter(|a| a.severity.is_actionable())
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anomaly::{AnomalyCode, AnomalyEvent};

    fn make_alert(code: AnomalyCode, seq: u64) -> StewardAlert {
        let ev = AnomalyEvent::new(code, None, seq * 10, "test");
        StewardAlert::from_event(ev, seq)
    }

    #[test]
    fn push_tracks_peak_severity() {
        let mut view = AuditView::new(100);
        view.push(make_alert(AnomalyCode::OrbitalClockDrift, 1));
        assert_eq!(view.peak_severity, Some(AlertSeverity::Dilbat));
        view.push(make_alert(AnomalyCode::KakiCollision, 2));
        assert_eq!(view.peak_severity, Some(AlertSeverity::Maroon));
    }

    #[test]
    fn has_emergency_only_for_maroon() {
        let mut view = AuditView::new(200);
        assert!(!view.has_emergency());
        view.push(make_alert(AnomalyCode::NashBreaking, 1));
        assert!(!view.has_emergency()); // NERGAL, not MAROON
        view.push(make_alert(AnomalyCode::SatamuChainBreak, 2));
        assert!(view.has_emergency());
    }

    #[test]
    fn alerts_for_tier_filters_correctly() {
        let mut view = AuditView::new(300);
        // SatamuChainBreak → min_tier=Smartphone (MAROON)
        view.push(make_alert(AnomalyCode::SatamuChainBreak, 1));
        let watch_count = view.alerts_for_tier(DeviceTier::Smartwatch).count();
        let phone_count = view.alerts_for_tier(DeviceTier::Smartphone).count();
        // Smartwatch only gets broadcasts — min_tier is Smartphone for this alert
        assert_eq!(watch_count, 0);
        assert_eq!(phone_count, 1);
    }
}
