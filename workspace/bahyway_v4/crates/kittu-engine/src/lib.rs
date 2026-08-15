//! kittu-engine v1 — Sovereign notification and delivery engine.
//!
//! Named for Kittu, Mesopotamian god of truth/justice, attendant of Shamash.
//! NOT named Kakka — phonetic collision with KAKI in Arabic (GL-001).
//!
//! v1 delivery channels:
//!   1. ShoWEngine dashboard (JSON alert file drop)
//!   2. Email via local sendmail
//!
//! Receives: Nisaba signed Alert Event KAKI particles (kaki_type=0x02)
//! Emits:    Delivery confirmation KAKI Event (closes loop immutably)
//!
//! SEPARATION OF CONCERNS:
//!   Nisaba  = detection + declaration only
//!   Kittu   = delivery only
#![forbid(unsafe_code)]

pub mod show;
pub mod email;
pub mod confirm;
pub mod poll;
pub mod error;

use serde::{Deserialize, Serialize};

/// Alert Event KAKI as consumed from Nisaba via EnkiSDB poll.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertEvent {
    pub kaki_id: String,
    pub tribe_id: String,
    pub severity: AlertSeverity,
    pub rule_id: String,
    pub message: String,
    pub timestamp: String,
    pub evidence: Vec<String>, // referenced KAKI particle IDs
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
    Erra, // ERRA = highest TIAMAT alert level
}

impl AlertSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            AlertSeverity::Info => "INFO",
            AlertSeverity::Warning => "WARNING",
            AlertSeverity::Critical => "CRITICAL",
            AlertSeverity::Erra => "ERRA",
        }
    }
}

/// Dispatch an alert to all v1 delivery channels.
pub fn dispatch(
    alert: &AlertEvent,
    show_dir: &str,
    email_to: &str,
) -> Result<String, error::KittuError> {
    show::write_show_alert(alert, show_dir)?;
    email::send_alert_email(alert, email_to)?;
    let confirm_id = confirm::emit_delivery_confirmation(alert);

    eprintln!(
        "Kittu: dispatched alert {} (severity={}) -> confirm_id={}",
        alert.kaki_id,
        alert.severity.as_str(),
        confirm_id
    );
    Ok(confirm_id)
}
