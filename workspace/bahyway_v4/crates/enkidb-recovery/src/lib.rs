//! enkidb-recovery — Crash Recovery and Abrupt-Termination handler (§11)
//!
//! Recovery procedure (§11.3):
//!   1. Validate the most recent snapshot block (CRC check)
//!   2. Scan forward through the Journal from the snapshot timestamp
//!   3. Find the Journal frontier — first CRC failure or EOF
//!   4. Project state from snapshot + valid events
//!   5. Rebuild derived indexes if their recorded position exceeds the frontier
//!   6. Resume normal operation

pub mod procedure;
pub mod objective;

pub use procedure::{RecoveryProcedure, RecoveryOutcome};
pub use objective::RecoveryObjective;

pub mod prelude {
    pub use super::procedure::{RecoveryProcedure, RecoveryOutcome};
    pub use super::objective::RecoveryObjective;
}
