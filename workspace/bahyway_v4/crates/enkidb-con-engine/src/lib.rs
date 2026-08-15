#![forbid(unsafe_code)]
//! enkidb-con-engine — Sovereign Connection Engine with 7 CSR security rules.
//!
//! Named after the Sumerian concept of CSR (Connection Security Rules),
//! this engine enforces passport validation, role checks, audit journaling,
//! credential expiry, cross-tribe gating, Kibratu event emission, and
//! tribe isolation on every connection request.

pub mod error;
pub mod roles;
pub mod audit;
pub mod csr;
pub mod pool;

pub use error::ConError;
pub use roles::SovereignRole;
pub use audit::{NaruEntry, NaruJournal};
pub use csr::{ConContext, Operation, apply_all_rules};
pub use pool::{ConnectionPool, PooledConnection};

// ── ConContextBuilder ─────────────────────────────────────────────────────────

/// Builder for ConContext that supplies default-safe values.
pub struct ConContextBuilder {
    pub caller_role:      SovereignRole,
    pub caller_tribe:     u32,
    pub target_tribe:     u32,
    pub operation:        Operation,
    pub passport_valid:   bool,
    pub credential_valid: bool,
}

impl Default for ConContextBuilder {
    fn default() -> Self {
        Self {
            caller_role:      SovereignRole::Client,
            caller_tribe:     0,
            target_tribe:     0,
            operation:        Operation::Read,
            passport_valid:   false,
            credential_valid: false,
        }
    }
}

// ── ConEngine ─────────────────────────────────────────────────────────────────

pub struct ConEngine {
    pub pool:    ConnectionPool,
    pub journal: NaruJournal,
}

impl ConEngine {
    pub fn new(pool_size: usize, journal_max: usize) -> Self {
        Self {
            pool:    ConnectionPool::new(pool_size),
            journal: NaruJournal::new(journal_max),
        }
    }

    /// Execute a connection request through all 7 CSR rules.
    pub fn execute(&mut self, builder: ConContextBuilder) -> Result<(), ConError> {
        let mut ctx = ConContext {
            caller_role:      builder.caller_role,
            caller_tribe:     builder.caller_tribe,
            target_tribe:     builder.target_tribe,
            operation:        builder.operation,
            passport_valid:   builder.passport_valid,
            credential_valid: builder.credential_valid,
            journal:          &mut self.journal,
        };
        apply_all_rules(&mut ctx)
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_journal() -> NaruJournal {
        NaruJournal::new(1024)
    }

    #[test]
    fn csr01_rejects_invalid_passport() {
        let mut journal = make_journal();
        let mut ctx = ConContext {
            caller_role:      SovereignRole::DubSar,
            caller_tribe:     1,
            target_tribe:     1,
            operation:        Operation::Read,
            passport_valid:   false,
            credential_valid: true,
            journal:          &mut journal,
        };
        let err = apply_all_rules(&mut ctx).expect_err("should fail CSR-01");
        assert!(matches!(err, ConError::Forbidden(_)));
    }

    #[test]
    fn csr02_rejects_client_for_write() {
        let mut journal = make_journal();
        let mut ctx = ConContext {
            caller_role:      SovereignRole::Client,
            caller_tribe:     1,
            target_tribe:     1,
            operation:        Operation::Write,
            passport_valid:   true,
            credential_valid: true,
            journal:          &mut journal,
        };
        let err = apply_all_rules(&mut ctx).expect_err("should fail CSR-02");
        assert!(matches!(err, ConError::InsufficientRole { .. }));
    }

    #[test]
    fn csr07_rejects_cross_tribe_for_non_dubsar() {
        let mut journal = make_journal();
        let mut ctx = ConContext {
            caller_role:      SovereignRole::TabletWriter,
            caller_tribe:     1,
            target_tribe:     2,
            operation:        Operation::Read,
            passport_valid:   true,
            credential_valid: true,
            journal:          &mut journal,
        };
        let err = apply_all_rules(&mut ctx).expect_err("should fail CSR-07");
        assert!(matches!(err, ConError::TribeIsolationViolation { caller_tribe: 1, target_tribe: 2 }));
    }

    #[test]
    fn dubsar_is_cross_tribe_exempt() {
        let mut journal = make_journal();
        let mut ctx = ConContext {
            caller_role:      SovereignRole::DubSar,
            caller_tribe:     1,
            target_tribe:     99,
            operation:        Operation::Write,
            passport_valid:   true,
            credential_valid: true,
            journal:          &mut journal,
        };
        apply_all_rules(&mut ctx).expect("DubSar should pass all rules");
    }

    #[test]
    fn journal_verify_all_passes_after_operations() {
        let mut journal = make_journal();
        journal.append(1, 0x01).unwrap();
        journal.append(2, 0x02).unwrap();
        assert!(journal.verify_all());
        assert_eq!(journal.len(), 2);
    }

    #[test]
    fn role_ordering() {
        assert!(SovereignRole::Client      < SovereignRole::DataSteward);
        assert!(SovereignRole::DataSteward < SovereignRole::TabletWriter);
        assert!(SovereignRole::TabletWriter < SovereignRole::DubSar);
        assert!(!SovereignRole::Client.can_write());
        assert!(!SovereignRole::DataSteward.can_write());
        assert!(SovereignRole::TabletWriter.can_write());
        assert!(SovereignRole::DubSar.can_write());
        assert!(!SovereignRole::TabletWriter.cross_tribe_exempt());
        assert!(SovereignRole::DubSar.cross_tribe_exempt());
    }
}
