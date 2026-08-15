//! ArtifactKind / ArtifactProfile — what EnkiMDB catalogs.
//!
//! Scope is deliberately narrow: only the two kinds `scan.rs` actually
//! produces from real filesystem state. Extend this enum only when a new
//! scanner backs it — no speculative variants.

/// Fixed tribe id for the generic crate/playbook artifact catalog --
/// single source of truth, matching `enkiddb::DOCS_TRIBE_ID`/
/// `shakkanakku::pb_mint::PB_TRIBE_ID`/`TabletKind::tribe_id`'s own
/// pattern. Was previously only a private default inside
/// `enkimdb-write-server`'s `tribe_id()` (still overridable there via
/// `TRIBE_ID` env) -- centralized here so every caller (that server, and
/// Shakkanakku's own crate-mint pass) mints under the same identity.
pub const ARTIFACT_TRIBE_ID: u16 = 0xFF01;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArtifactKind {
    /// A workspace Rust crate (has its own Cargo.toml under crates/).
    Crate,
    /// An Ansible playbook (a `.yml` file under playbooks/).
    Playbook,
}

impl ArtifactKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ArtifactKind::Crate => "Crate",
            ArtifactKind::Playbook => "Playbook",
        }
    }
}

/// One catalogued artifact, ready to be minted into KAKI-sealed particles.
#[derive(Debug, Clone)]
pub struct ArtifactProfile {
    pub name: String,
    pub kind: ArtifactKind,
    pub path: String,
    /// Crate version, when known (playbooks have none).
    pub version: Option<String>,
}
