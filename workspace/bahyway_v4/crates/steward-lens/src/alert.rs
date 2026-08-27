#![forbid(unsafe_code)]
//! StewardAlert — the notification record delivered to a DataSteward device.

use crate::anomaly::AnomalyEvent;
use crate::device::DeviceTier;

/// Five-level severity mirroring the TiamatLevel system.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum AlertSeverity {
    /// GREEN — nominal, informational.
    Green = 0,
    /// DILBAT — watch, worth monitoring.
    Dilbat = 1,
    /// KAKKAB — caution, human attention recommended.
    Kakkab = 2,
    /// NERGAL — danger, human decision required.
    Nergal = 3,
    /// MAROON — emergency, system-wide override possible.
    Maroon = 4,
}

impl AlertSeverity {
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => AlertSeverity::Green,
            1 => AlertSeverity::Dilbat,
            2 => AlertSeverity::Kakkab,
            3 => AlertSeverity::Nergal,
            _ => AlertSeverity::Maroon,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            AlertSeverity::Green => "GREEN",
            AlertSeverity::Dilbat => "DILBAT",
            AlertSeverity::Kakkab => "KAKKAB",
            AlertSeverity::Nergal => "NERGAL",
            AlertSeverity::Maroon => "MAROON",
        }
    }

    /// Returns true if this severity warrants interrupting the DataSteward.
    #[inline]
    pub fn is_actionable(self) -> bool {
        self >= AlertSeverity::Kakkab
    }

    /// Returns true if this alert must be routed to ALL connected steward devices.
    #[inline]
    pub fn is_broadcast(self) -> bool {
        self == AlertSeverity::Maroon
    }
}

impl core::fmt::Display for AlertSeverity {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A complete alert record delivered from StewardLens to a steward device.
///
/// Read-only: StewardLens emits these; the steward device displays them.
/// Any resulting action (override, approve, deprecate) is recorded by Šàtamu,
/// not by StewardLens.
#[derive(Clone, Debug)]
pub struct StewardAlert {
    /// Severity level (TiamatLevel mapping).
    pub severity: AlertSeverity,
    /// The anomaly that triggered this alert.
    pub event: AnomalyEvent,
    /// The minimum device tier that can display this alert meaningfully.
    pub min_tier: DeviceTier,
    /// Alert sequence number within this steward's session.
    pub sequence: u64,
    /// Whether this alert requires a Šàtamu decision record before it is cleared.
    pub requires_satamu: bool,
}

impl StewardAlert {
    pub fn from_event(event: AnomalyEvent, sequence: u64) -> Self {
        let severity = AlertSeverity::from_u8(event.code.default_tiamat_level());
        let min_tier = match severity {
            AlertSeverity::Maroon | AlertSeverity::Nergal => DeviceTier::Smartphone,
            _ => DeviceTier::Smartwatch,
        };
        let requires_satamu = matches!(severity, AlertSeverity::Nergal | AlertSeverity::Maroon);
        Self {
            severity,
            event,
            min_tier,
            sequence,
            requires_satamu,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anomaly::{AnomalyCode, AnomalyEvent};

    #[test]
    fn satamu_chain_break_is_maroon_broadcast() {
        let ev = AnomalyEvent::new(AnomalyCode::SatamuChainBreak, None, 100, "chain broken");
        let alert = StewardAlert::from_event(ev, 1);
        assert_eq!(alert.severity, AlertSeverity::Maroon);
        assert!(alert.severity.is_broadcast());
        assert!(alert.requires_satamu);
    }

    #[test]
    fn clock_drift_is_dilbat_not_actionable() {
        let ev = AnomalyEvent::new(AnomalyCode::OrbitalClockDrift, None, 200, "drift = 3 ticks");
        let alert = StewardAlert::from_event(ev, 2);
        assert_eq!(alert.severity, AlertSeverity::Dilbat);
        assert!(!alert.severity.is_actionable());
        assert!(!alert.requires_satamu);
    }

    #[test]
    fn severity_ordering() {
        assert!(AlertSeverity::Maroon > AlertSeverity::Nergal);
        assert!(AlertSeverity::Nergal > AlertSeverity::Kakkab);
        assert!(AlertSeverity::Kakkab > AlertSeverity::Dilbat);
        assert!(AlertSeverity::Dilbat > AlertSeverity::Green);
    }
}
