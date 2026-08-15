//! EnkiSDB poll loop — watches for new Alert Event KAKI particles
//! from Nisaba in the SHEDU security tribe.
//! Poll interval: 10 seconds. Full KISPU query wired when EnkiSDB live.
#![forbid(unsafe_code)]
use crate::AlertEvent;
use std::thread;
use std::time::Duration;

pub const POLL_INTERVAL_SECS: u64 = 10;

/// Poll EnkiSDB for pending Alert Events.
/// Stub: returns empty until KISPU integration is wired.
pub fn poll_pending_alerts(enkidb_addr: &str) -> Vec<AlertEvent> {
    // TODO: HeptaScript query against tribe_id="shedu_security",
    // EAV.alert_type="NISABA_ALERT", EAV.delivered IS NULL, LIMIT 50
    eprintln!("Kittu poll: checking EnkiSDB {enkidb_addr} for alerts");
    vec![] // stub — returns empty until KISPU integration
}

pub fn poll_loop(enkidb_addr: &str, show_dir: &str, email_to: &str) {
    loop {
        let alerts = poll_pending_alerts(enkidb_addr);
        for alert in &alerts {
            match crate::dispatch(alert, show_dir, email_to) {
                Ok(confirm) => eprintln!("Delivered alert {} -> {}", alert.kaki_id, confirm),
                Err(e) => eprintln!("Delivery failed for {}: {}", alert.kaki_id, e),
            }
        }
        thread::sleep(Duration::from_secs(POLL_INTERVAL_SECS));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_poll_returns_empty() {
        assert!(poll_pending_alerts("192.168.122.107:7001").is_empty());
    }
}
