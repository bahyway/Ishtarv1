//! StewardStation — the Data Steward's action queue (§10.6).
//!
//! The station receives Alerts from the AlertEngine, produces Diagnosis
//! records, and maintains an ordered queue for steward review.
//! Zero writes to the database — the steward reads findings and acts
//! by submitting corrective Event-Kakis through AdadGate.

use alert_engine::alert::Alert;
use diagnosis_templates::{Diagnosis, diagnose};

/// The Data Steward Station — accumulates diagnoses for steward review.
pub struct StewardStation {
    queue: Vec<Diagnosis>,
}

impl StewardStation {
    pub fn new() -> Self {
        StewardStation { queue: Vec::new() }
    }

    /// Receive an alert and enqueue its diagnosis.
    pub fn receive(&mut self, alert: &Alert) {
        let d = diagnose(
            alert.uuid_hash,
            alert.severity,
            alert.drift_distance,
        );
        self.queue.push(d);
    }

    /// Receive a batch of alerts at once.
    pub fn receive_batch(&mut self, alerts: &[Alert]) {
        for a in alerts {
            self.receive(a);
        }
    }

    pub fn queue(&self) -> &[Diagnosis] {
        &self.queue
    }

    pub fn queue_len(&self) -> usize {
        self.queue.len()
    }

    pub fn clear(&mut self) {
        self.queue.clear();
    }
}

impl Default for StewardStation {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alert_engine::alert::{Alert, AlertSeverity, DriftCause};

    fn make_alert(drift: f32, severity: AlertSeverity) -> Alert {
        Alert::new(0xDEAD_BEEF, severity, DriftCause::Unknown, drift)
    }

    #[test]
    fn receive_adds_to_queue() {
        let mut s = StewardStation::new();
        s.receive(&make_alert(60.0, AlertSeverity::Watch));
        assert_eq!(s.queue_len(), 1);
    }

    #[test]
    fn receive_batch() {
        let mut s  = StewardStation::new();
        let alerts = vec![
            make_alert(60.0,  AlertSeverity::Watch),
            make_alert(200.0, AlertSeverity::Warning),
            make_alert(400.0, AlertSeverity::Critical),
        ];
        s.receive_batch(&alerts);
        assert_eq!(s.queue_len(), 3);
    }

    #[test]
    fn clear_empties_queue() {
        let mut s = StewardStation::new();
        s.receive(&make_alert(60.0, AlertSeverity::Watch));
        s.clear();
        assert_eq!(s.queue_len(), 0);
    }

    #[test]
    fn critical_alert_diagnosis_is_critical() {
        use diagnosis_templates::DiagnosisKind;
        let mut s = StewardStation::new();
        s.receive(&make_alert(400.0, AlertSeverity::Critical));
        assert_eq!(s.queue()[0].kind, DiagnosisKind::Critical);
    }
}
