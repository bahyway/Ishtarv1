//! Istar — Sovereign access control engine.
//! 𒀭𒄑𒆳 *Ištar* — "divine gate / sovereign arbiter".
//!
//! Five constitutional Meta Rules + composable rule engine.

#![forbid(unsafe_code)]
#![allow(dead_code)]

pub mod akk_firewall;

pub use akk_firewall::{
    gates, meta_rules, AkkContext, AkkFirewall, AkkOperation, AkkResource, AkkRule, AkkSubject,
    FirewallVerdict,
};

// ── IstarError / IstarResult ──────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum IstarError {
    AccessDenied(String),
    RuleMissing(String),
    ContextInvalid(String),
    EscalationRequired(String),
}

impl std::fmt::Display for IstarError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AccessDenied(m) => write!(f, "NAKRU (access denied): {m}"),
            Self::RuleMissing(m) => write!(f, "ISTAR rule missing: {m}"),
            Self::ContextInvalid(m) => write!(f, "ISTAR context invalid: {m}"),
            Self::EscalationRequired(m) => write!(f, "ISTAR escalation required: {m}"),
        }
    }
}

impl std::error::Error for IstarError {}

pub type IstarResult<T> = Result<T, IstarError>;
