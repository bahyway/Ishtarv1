//! Shakkanakku run-confirmation records — every Shakkanakku corpus run
//! becomes a real EnkiMDB particle, `shakkanakku_run.*` EAV namespace
//! (mirrors `passport.*`'s shape and read-authorization treatment).
//!
//! WHY THIS EXISTS: the Architect's own instruction — a daily workflow
//! of "run Shakkanakku, get report, debug, re-run" was missing a
//! queryable confirmation registry that also records WHO ran it and at
//! what privilege level, specifically so an authority-related block
//! (operator's vault passport doesn't clear the required privilege
//! level) is distinguishable from a genuine technical failure, without
//! having to re-read the raw report text every time.
//!
//! HONEST SCOPE: this is a per-RUN summary (started/finished, per-status
//! counts, overall outcome, operator identity), not a per-playbook
//! ledger — the sealed report + chronicle (`report.rs`/`chronicle.rs`)
//! already carry the full per-playbook detail; duplicating that into EAV
//! rows would be a second copy of the same truth, not a new capability.
//! `operator_*` fields are `None` when `vault_check_enabled` is off for
//! that run (unauthenticated) — never a fabricated identity.
//!
//! TWO SEPARATE IDENTITY SIGNALS (2026-07-29), never merged into one:
//!   - `operator_*` — the cryptographic vault-passport identity
//!     (`kupru_vault::AuthedIdentity`), portable, requires a real vault
//!     + passphrase, only present when `vault_check_enabled=true`.
//!   - `os_*` — the real Fedora host OS user/group that launched the
//!     `shakkanakku` process (`id -un`/`id -Gn`, cross-referenced
//!     against the 5 `bahyway-*` groups PB-268 creates), always
//!     available (no vault needed), host-local. A mismatch between the
//!     two (e.g. OS group `bahyway-developer` running with a vault
//!     passport claiming `privilege_level=7`) is itself a real,
//!     queryable fact this registry preserves rather than silently
//!     reconciles.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShakkanakkuRunRecordSpec {
    pub run_id: String,
    pub started_at: u64,
    pub finished_at: u64,
    pub playbook_count: u32,
    pub ok_count: u32,
    pub warned_count: u32,
    pub failed_count: u32,
    pub skipped_count: u32,
    /// "Completed" | "Aborted" | "BlockedByAuthority" — real, closed set;
    /// see `registry_emitter::emit_shakkanakku_run`'s own validation.
    pub outcome: String,
    /// Set only when `outcome` isn't a clean "Completed" — e.g. the real
    /// MAJOR-failure message, or "operator privilege_level=3 below
    /// required minimum=5" for an authority block.
    pub blocked_reason: Option<String>,
    pub operator_subject_kaki_hex: Option<String>,
    pub operator_realm: Option<String>,
    pub operator_privilege_level: Option<u8>,
    /// Real Fedora host username (`id -un`) that launched this process.
    /// `None` only if the `id` command itself couldn't be run/parsed —
    /// never a fabricated placeholder.
    pub os_username: Option<String>,
    /// Comma-separated real group names (`id -Gn`), unfiltered — every
    /// group the OS user actually belongs to, not just BahyWay ones.
    pub os_groups_csv: Option<String>,
    /// Which of the 5 `bahyway-*` groups (PB-268) matched, if any —
    /// the STRONGEST match when a user belongs to more than one, same
    /// "hold multiple roles, strongest wins" rule the vault-passport
    /// path already uses. `None` if the user is in none of them.
    pub os_bahyway_role: Option<String>,
    pub report_path: String,
}
