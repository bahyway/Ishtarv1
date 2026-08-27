//! eridu-scheduler — tick-based job scheduler (§11.2).

pub mod scheduler;
pub use scheduler::{
    DueJob, EriduScheduler, JobKind, ScheduledJob, VALIDATION_SWEEP_DEFAULT_TICKS,
    VALIDATION_SWEEP_JOB,
};
