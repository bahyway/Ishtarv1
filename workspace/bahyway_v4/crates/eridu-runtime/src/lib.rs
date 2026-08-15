//! eridu-runtime — synchronous cooperative task executor (§11).
//!
//! Also hosts the SchedulerLoop which integrates EriduScheduler with the
//! SDB→ODB→QDB five-tier pipeline routing.

pub mod runtime;
pub mod scheduler_loop;

pub use runtime::{EriduRuntime, Task, TaskResult, RuntimeState};
pub use scheduler_loop::{SchedulerLoop, TickOutcome};
