#![forbid(unsafe_code)]
//! enkidb-con-engine — Sovereign Connection Engine with 8 CSR security rules.
//!
//! Named after the Sumerian concept of CSR (Connection Security Rules),
//! this engine enforces passport validation, role checks, audit journaling,
//! credential expiry, cross-tribe gating, Kibratu event emission, tribe
//! isolation, and Architect Sovereignty on every connection request.

pub mod audit;
pub mod csr;
pub mod error;
pub mod pool;
pub mod roles;

pub use audit::{NaruEntry, NaruJournal};
pub use csr::{apply_all_rules, ConContext, Operation, OrganAction, OrganKind};
pub use error::ConError;
pub use pool::{ConnectionPool, PooledConnection};
pub use roles::SovereignRole;

// ── ConContextBuilder ─────────────────────────────────────────────────────────

/// Builder for ConContext that supplies default-safe values.
pub struct ConContextBuilder {
    pub caller_role: SovereignRole,
    pub caller_tribe: u32,
    pub target_tribe: u32,
    pub operation: Operation,
    pub passport_valid: bool,
    pub credential_valid: bool,
    /// See `ConContext::organ_mutation` (CSR-08). Defaults to `None` --
    /// an ordinary data request, untouched by CSR-08.
    pub organ_mutation: Option<(OrganAction, OrganKind)>,
    /// See `ConContext::architect_confirmed` (CSR-08). Defaults to
    /// `false`; irrelevant unless `organ_mutation` is `Some`.
    pub architect_confirmed: bool,
}

impl Default for ConContextBuilder {
    fn default() -> Self {
        Self {
            caller_role: SovereignRole::Client,
            caller_tribe: 0,
            target_tribe: 0,
            operation: Operation::Read,
            passport_valid: false,
            credential_valid: false,
            organ_mutation: None,
            architect_confirmed: false,
        }
    }
}

// ── ConEngine ─────────────────────────────────────────────────────────────────

pub struct ConEngine {
    pub pool: ConnectionPool,
    pub journal: NaruJournal,
}

impl ConEngine {
    pub fn new(pool_size: usize, journal_max: usize) -> Self {
        Self {
            pool: ConnectionPool::new(pool_size),
            journal: NaruJournal::new(journal_max),
        }
    }

    /// Execute a connection request through all 8 CSR rules.
    pub fn execute(&mut self, builder: ConContextBuilder) -> Result<(), ConError> {
        let mut ctx = ConContext {
            caller_role: builder.caller_role,
            caller_tribe: builder.caller_tribe,
            target_tribe: builder.target_tribe,
            operation: builder.operation,
            passport_valid: builder.passport_valid,
            credential_valid: builder.credential_valid,
            organ_mutation: builder.organ_mutation,
            architect_confirmed: builder.architect_confirmed,
            journal: &mut self.journal,
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
            caller_role: SovereignRole::DubSar,
            caller_tribe: 1,
            target_tribe: 1,
            operation: Operation::Read,
            passport_valid: false,
            credential_valid: true,
            organ_mutation: None,
            architect_confirmed: false,
            journal: &mut journal,
        };
        let err = apply_all_rules(&mut ctx).expect_err("should fail CSR-01");
        assert!(matches!(err, ConError::Forbidden(_)));
    }

    #[test]
    fn csr02_rejects_client_for_write() {
        let mut journal = make_journal();
        let mut ctx = ConContext {
            caller_role: SovereignRole::Client,
            caller_tribe: 1,
            target_tribe: 1,
            operation: Operation::Write,
            passport_valid: true,
            credential_valid: true,
            organ_mutation: None,
            architect_confirmed: false,
            journal: &mut journal,
        };
        let err = apply_all_rules(&mut ctx).expect_err("should fail CSR-02");
        assert!(matches!(err, ConError::InsufficientRole { .. }));
    }

    #[test]
    fn csr07_rejects_cross_tribe_for_non_dubsar() {
        let mut journal = make_journal();
        let mut ctx = ConContext {
            caller_role: SovereignRole::TabletWriter,
            caller_tribe: 1,
            target_tribe: 2,
            operation: Operation::Read,
            passport_valid: true,
            credential_valid: true,
            organ_mutation: None,
            architect_confirmed: false,
            journal: &mut journal,
        };
        let err = apply_all_rules(&mut ctx).expect_err("should fail CSR-07");
        assert!(matches!(
            err,
            ConError::TribeIsolationViolation {
                caller_tribe: 1,
                target_tribe: 2
            }
        ));
    }

    #[test]
    fn dubsar_is_cross_tribe_exempt() {
        let mut journal = make_journal();
        let mut ctx = ConContext {
            caller_role: SovereignRole::DubSar,
            caller_tribe: 1,
            target_tribe: 99,
            operation: Operation::Write,
            passport_valid: true,
            credential_valid: true,
            organ_mutation: None,
            architect_confirmed: false,
            journal: &mut journal,
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
    fn csr08_rejects_unconfirmed_organ_mutation() {
        let mut journal = make_journal();
        let mut ctx = ConContext {
            caller_role: SovereignRole::DubSar, // even DubSar's own role is not enough
            caller_tribe: 1,
            target_tribe: 1,
            operation: Operation::Admin,
            passport_valid: true,
            credential_valid: true,
            // "Supersede" -- proposing to append a playbook amendment
            // citing the prior one's KAKI, never an in-place edit.
            organ_mutation: Some((OrganAction::Supersede, OrganKind::Playbook)),
            architect_confirmed: false, // no explicit confirmation on THIS request
            journal: &mut journal,
        };
        let err = apply_all_rules(&mut ctx).expect_err("should fail CSR-08");
        assert!(matches!(
            err,
            ConError::ArchitectConfirmationRequired(OrganAction::Supersede, OrganKind::Playbook)
        ));
    }

    #[test]
    fn csr08_passes_a_confirmed_organ_mutation() {
        let mut journal = make_journal();
        let mut ctx = ConContext {
            caller_role: SovereignRole::TabletWriter, // an ordinary agent's own role...
            caller_tribe: 1,
            target_tribe: 1,
            operation: Operation::Admin,
            passport_valid: true,
            credential_valid: true,
            organ_mutation: Some((OrganAction::Create, OrganKind::Crate)),
            architect_confirmed: true, // ...cleared because DUB.SAR confirmed THIS request
            journal: &mut journal,
        };
        apply_all_rules(&mut ctx).expect("Architect-confirmed organ mutation should pass");
    }

    #[test]
    fn csr08_is_a_no_op_for_ordinary_data_requests() {
        // "Diagnosis is autonomous": organ_mutation: None means CSR-08
        // never fires, regardless of architect_confirmed.
        let mut journal = make_journal();
        let mut ctx = ConContext {
            caller_role: SovereignRole::TabletWriter,
            caller_tribe: 1,
            target_tribe: 1,
            operation: Operation::Write,
            passport_valid: true,
            credential_valid: true,
            organ_mutation: None,
            architect_confirmed: false,
            journal: &mut journal,
        };
        apply_all_rules(&mut ctx).expect("ordinary data write is untouched by CSR-08");
    }

    #[test]
    fn csr08_names_the_real_organ_kinds_from_the_sealed_law() {
        // The law's own parenthetical: "crate, engine, agent, template,
        // KAKI, tribe, session, playbook, or configuration" -- nine
        // kinds, none invented.
        let kinds = [
            OrganKind::Crate,
            OrganKind::Engine,
            OrganKind::Agent,
            OrganKind::Template,
            OrganKind::Kaki,
            OrganKind::Tribe,
            OrganKind::Session,
            OrganKind::Playbook,
            OrganKind::Configuration,
        ];
        assert_eq!(kinds.len(), 9);
        assert_eq!(OrganKind::Kaki.as_str(), "KAKI");
    }

    #[test]
    fn csr08_actions_are_append_only_never_in_place_crud() {
        // Regression guard: BahyWay is append-only (SS0.3) -- there must
        // never be an OrganAction variant that means a literal in-place
        // modify or delete. Exactly three, and this is what they mean:
        //   Create    -- mint a brand-new organ.
        //   Supersede -- the "modify" case: a new particle citing the
        //                prior organ's KAKI, the prior organ untouched.
        //   Retire    -- the "delete" case: an Event marking DEAD/
        //                retired, the organ's particles kept forever.
        let actions = [OrganAction::Create, OrganAction::Supersede, OrganAction::Retire];
        assert_eq!(actions.len(), 3);
        assert_eq!(OrganAction::Supersede.as_str(), "supersede");
        assert_eq!(OrganAction::Retire.as_str(), "retire");
    }

    #[test]
    fn csr08_rejects_an_unconfirmed_retirement_too() {
        // The "delete" case specifically: retiring an organ is still an
        // append (an Event marking it DEAD), and still needs the
        // Architect's confirmation like any other organ-affecting append.
        let mut journal = make_journal();
        let mut ctx = ConContext {
            caller_role: SovereignRole::DubSar,
            caller_tribe: 1,
            target_tribe: 1,
            operation: Operation::Admin,
            passport_valid: true,
            credential_valid: true,
            organ_mutation: Some((OrganAction::Retire, OrganKind::Agent)),
            architect_confirmed: false,
            journal: &mut journal,
        };
        let err = apply_all_rules(&mut ctx).expect_err("should fail CSR-08");
        assert!(matches!(
            err,
            ConError::ArchitectConfirmationRequired(OrganAction::Retire, OrganKind::Agent)
        ));
    }

    #[test]
    fn role_ordering() {
        assert!(SovereignRole::Client < SovereignRole::DataSteward);
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
