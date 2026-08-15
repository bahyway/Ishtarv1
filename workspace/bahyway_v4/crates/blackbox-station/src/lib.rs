//! blackbox-station — the Error Handling Station between EnkiSDB quarantine
//! and a particle's final jail (§12.3b).
//!
//! `ValidationSweep` marks a particle Quarantined but does not decide WHERE
//! it is jailed.  BlackBox Station performs that scan on every Quarantined
//! particle still sitting in EnkiSDB:
//!
//!   - `malware_flag == true`  → confirmed harmful → StorageSector (hardware-isolated, terminal)
//!   - `malware_flag == false` → fuzzy / unknown   → EnkiQDB (pending Data Steward review)

pub mod route;

pub use route::{BlackBoxStation, BlackBoxReport};

pub mod prelude {
    pub use super::route::{BlackBoxStation, BlackBoxReport};
}
