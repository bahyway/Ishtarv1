#![forbid(unsafe_code)]
//! 7 CSR (Connection Security Rules) — applied in order to every connection context.

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

/// Context for evaluating the 7 CSR rules against a single connection request.
pub struct ConContext<'a> {
    pub caller_role:      SovereignRole,
    pub caller_tribe:     u32,
    pub target_tribe:     u32,
    pub operation:        Operation,
    pub passport_valid:   bool,
    pub credential_valid: bool,
    pub journal:          &'a mut NaruJournal,
}

/// Apply all 7 CSR rules in order. Returns Ok(()) if all pass.
pub fn apply_all_rules(ctx: &mut ConContext) -> Result<(), ConError> {
    csr01_sargon_gate(ctx)?;
    csr02_role_check(ctx)?;
    csr03_naru_audit(ctx)?;
    csr04_credential_check(ctx)?;
    csr05_gilgamesh_gate(ctx)?;
    csr06_kibratu_emit(ctx)?;
    csr07_tribe_isolation(ctx)?;
    Ok(())
}

// ── CSR-01: Sargon gate — passport must be valid ──────────────────────────────

fn csr01_sargon_gate(ctx: &mut ConContext) -> Result<(), ConError> {
    if !ctx.passport_valid {
        return Err(ConError::Forbidden("passport invalid — Sargon gate rejected"));
    }
    Ok(())
}

// ── CSR-02: Role check — Write requires TabletWriter or higher ────────────────

fn csr02_role_check(ctx: &mut ConContext) -> Result<(), ConError> {
    if ctx.operation == Operation::Write || ctx.operation == Operation::Admin {
        if !ctx.caller_role.can_write() {
            return Err(ConError::InsufficientRole {
                needed: SovereignRole::TabletWriter,
                got:    ctx.caller_role.clone(),
            });
        }
    }
    Ok(())
}

// ── CSR-03: NĀRU audit — journal the operation ────────────────────────────────

fn csr03_naru_audit(ctx: &mut ConContext) -> Result<(), ConError> {
    let op_code = match ctx.operation {
        Operation::Read       => 0x01,
        Operation::Write      => 0x02,
        Operation::CrossTribe => 0x03,
        Operation::Admin      => 0x04,
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
            "cross-tribe write requires DubSar role"
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
