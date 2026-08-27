#![forbid(unsafe_code)]
//! 8 CSR (Connection Security Rules) — applied in order to every connection context.

use crate::audit::NaruJournal;
use crate::error::ConError;
use crate::roles::SovereignRole;

/// Operation type being requested.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operation {
    Read,
    Write,
    CrossTribe,
    Admin,
}

/// The organ kinds named, word-for-word, in the sealed CSR-08 law
/// ("Architect Sovereignty"): "crate, engine, agent, template, KAKI,
/// tribe, session, playbook, or configuration." Not invented here — this
/// enum exists only to name exactly that list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrganKind {
    Crate,
    Engine,
    Agent,
    Template,
    Kaki,
    Tribe,
    Session,
    Playbook,
    Configuration,
}

impl OrganKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Crate => "crate",
            Self::Engine => "engine",
            Self::Agent => "agent",
            Self::Template => "template",
            Self::Kaki => "KAKI",
            Self::Tribe => "tribe",
            Self::Session => "session",
            Self::Playbook => "playbook",
            Self::Configuration => "configuration",
        }
    }
}

/// How an organ-affecting particle is appended.
///
/// BahyWay.Ecosystem is append-only (§0.3 Sovereign Constraints): nothing
/// is ever overwritten in place or physically removed. The sealed CSR-08
/// law's own everyday words are "create, modify, or delete" — but this
/// crate never models a literal in-place modify or delete, because no
/// such operation exists to gate. What those words actually mean, under
/// the hood, is always a new append:
///
///   - `Create`    — mint a brand-new organ (a new crate, a new tribe...).
///   - `Supersede` — the "modify" case: append a new version citing the
///                   prior organ's own KAKI as its predecessor. The prior
///                   organ is never edited — e.g. v4.1's code supersedes
///                   v4.0's as a new particle, GL-IMM-001-A1 supersedes
///                   GL-IMM-001 as a new amendment tablet, never an edit
///                   of the original.
///   - `Retire`    — the "delete" case: append an Event marking the organ
///                   DEAD/retired. The organ's own particles remain in
///                   the store forever (the same law that keeps
///                   `ParticleState::Dead` particles in EnkiDB rather
///                   than removing them) — nothing is physically deleted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrganAction {
    Create,
    Supersede,
    Retire,
}

impl OrganAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Supersede => "supersede",
            Self::Retire => "retire",
        }
    }
}

/// Context for evaluating the 8 CSR rules against a single connection request.
pub struct ConContext<'a> {
    pub caller_role: SovereignRole,
    pub caller_tribe: u32,
    pub target_tribe: u32,
    pub operation: Operation,
    pub passport_valid: bool,
    pub credential_valid: bool,
    /// `None` for an ordinary data Read/Write/CrossTribe/Admin request —
    /// CSR-08 does not apply ("diagnosis is autonomous"). `Some((action,
    /// kind))` when this request is itself proposing to append a
    /// Create/Supersede/Retire particle affecting one of the law's named
    /// organ kinds ("prescription is proposed").
    pub organ_mutation: Option<(OrganAction, OrganKind)>,
    /// Whether DUB.SAR has explicitly confirmed this specific proposed
    /// organ mutation. Deliberately a fact about THIS request, not about
    /// `caller_role` — the caller is very often an autonomous agent
    /// (TamuzAI, EaAgent, AdadAI, NuskuAgent), never the Architect
    /// themself; CSR-08 governs whether the Architect signed off on the
    /// action, not who is carrying it out. Mirrors `passport_valid` /
    /// `credential_valid`'s own shape: a separate boolean fact checked
    /// alongside role, not folded into it.
    pub architect_confirmed: bool,
    pub journal: &'a mut NaruJournal,
}

/// Apply all 8 CSR rules in order. Returns Ok(()) if all pass.
pub fn apply_all_rules(ctx: &mut ConContext) -> Result<(), ConError> {
    csr01_sargon_gate(ctx)?;
    csr02_role_check(ctx)?;
    csr03_naru_audit(ctx)?;
    csr04_credential_check(ctx)?;
    csr05_gilgamesh_gate(ctx)?;
    csr06_kibratu_emit(ctx)?;
    csr07_tribe_isolation(ctx)?;
    csr08_architect_sovereignty(ctx)?;
    Ok(())
}

// ── CSR-01: Sargon gate — passport must be valid ──────────────────────────────

fn csr01_sargon_gate(ctx: &mut ConContext) -> Result<(), ConError> {
    if !ctx.passport_valid {
        return Err(ConError::Forbidden(
            "passport invalid — Sargon gate rejected",
        ));
    }
    Ok(())
}

// ── CSR-02: Role check — Write requires TabletWriter or higher ────────────────

fn csr02_role_check(ctx: &mut ConContext) -> Result<(), ConError> {
    if (ctx.operation == Operation::Write || ctx.operation == Operation::Admin)
        && !ctx.caller_role.can_write()
    {
        return Err(ConError::InsufficientRole {
            needed: SovereignRole::TabletWriter,
            got: ctx.caller_role.clone(),
        });
    }
    Ok(())
}

// ── CSR-03: NĀRU audit — journal the operation ────────────────────────────────

fn csr03_naru_audit(ctx: &mut ConContext) -> Result<(), ConError> {
    let op_code = match ctx.operation {
        Operation::Read => 0x01,
        Operation::Write => 0x02,
        Operation::CrossTribe => 0x03,
        Operation::Admin => 0x04,
    };
    ctx.journal.append(ctx.caller_tribe, op_code)
}

// ── CSR-04: Credential check — credential must be valid ───────────────────────

fn csr04_credential_check(ctx: &mut ConContext) -> Result<(), ConError> {
    if !ctx.credential_valid {
        return Err(ConError::CredentialExpired);
    }
    Ok(())
}

// ── CSR-05: Gilgamesh gate — cross-tribe Write blocked unless DubSar ─────────

fn csr05_gilgamesh_gate(ctx: &mut ConContext) -> Result<(), ConError> {
    if ctx.caller_tribe != ctx.target_tribe
        && (ctx.operation == Operation::Write || ctx.operation == Operation::CrossTribe)
        && !ctx.caller_role.cross_tribe_exempt()
    {
        return Err(ConError::GilgameshBlocked(
            "cross-tribe write requires DubSar role",
        ));
    }
    Ok(())
}

// ── CSR-06: Kibratu emit — audit event stub ───────────────────────────────────

fn csr06_kibratu_emit(_ctx: &mut ConContext) -> Result<(), ConError> {
    // Stub: Kibratu event bus is pending sovereign implementation.
    // No-op — returns Ok always.
    Ok(())
}

// ── CSR-07: Tribe isolation — caller_tribe == target_tribe unless exempt ──────

fn csr07_tribe_isolation(ctx: &mut ConContext) -> Result<(), ConError> {
    if ctx.caller_tribe != ctx.target_tribe && !ctx.caller_role.cross_tribe_exempt() {
        return Err(ConError::TribeIsolationViolation {
            caller_tribe: ctx.caller_tribe,
            target_tribe: ctx.target_tribe,
        });
    }
    Ok(())
}

// ── CSR-08: Architect Sovereignty ──────────────────────────────────────────────
//
// No sovereign component of BahyWay.Ecosystem may create, modify, or
// delete any organ (crate, engine, agent, template, KAKI, tribe, session,
// playbook, or configuration) without explicit confirmation from
// DUB.SAR 𒁾.
//
// BahyWay is append-only (§0.3): there is no in-place modify or delete to
// gate. The law's "modify"/"delete" map onto OrganAction::Supersede and
// OrganAction::Retire — both still real appends, never overwrites or
// removals (see OrganAction's own doc comment).
//
// Diagnosis is autonomous.   -- ctx.organ_mutation == None passes untouched.
// Prescription is proposed.  -- ctx.organ_mutation == Some((action, kind))
//                                may be constructed by any caller; it is
//                                not itself the violation.
// Execution is the Architect's alone. -- only ctx.architect_confirmed
//                                         clears the gate.
//
// Cross-cutting by design: unlike CSR-01..07, this rule never looks at
// caller_role. Every agent -- TamuzAI, EaAgent, AdadAI, NuskuAgent, or
// any future one -- proposes organ-affecting appends under its own role;
// CSR-08 asks only whether THIS SPECIFIC request carries the Architect's
// explicit confirmation, not who is asking.

fn csr08_architect_sovereignty(ctx: &mut ConContext) -> Result<(), ConError> {
    if let Some((action, kind)) = ctx.organ_mutation {
        if !ctx.architect_confirmed {
            return Err(ConError::ArchitectConfirmationRequired(action, kind));
        }
    }
    Ok(())
}
