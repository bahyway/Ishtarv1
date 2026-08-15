//! Delivery confirmation KAKI Event.
//! kaki_type=0x02 Event, kaki_role=0x02 ZIKRU
//! Closes the Nisaba -> Kittu alert loop immutably.
#![forbid(unsafe_code)]
use crate::AlertEvent;
use std::time::{SystemTime, UNIX_EPOCH};

/// Emit a delivery confirmation KAKI. Returns the generated confirm_id.
/// TODO: KISPU four-way atomic commit (stub log for now).
pub fn emit_delivery_confirmation(alert: &AlertEvent) -> String {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let confirm_id = format!("kittu-confirm-{ts:x}");
    eprintln!(
        "Kittu CONFIRM KAKI | id={} | alert_ref={} | tribe={}",
        confirm_id, alert.kaki_id, alert.tribe_id
    );
    confirm_id
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AlertSeverity;

    #[test]
    fn confirm_id_is_unique_per_call() {
        let alert = AlertEvent {
            kaki_id: "kaki-1".to_string(),
            tribe_id: "shedu_security".to_string(),
            severity: AlertSeverity::Info,
            rule_id: "R-1".to_string(),
            message: "m".to_string(),
            timestamp: "t".to_string(),
            evidence: vec![],
        };
        let a = emit_delivery_confirmation(&alert);
        let b = emit_delivery_confirmation(&alert);
        assert!(a.starts_with("kittu-confirm-"));
        assert_ne!(a, b);
    }
}
