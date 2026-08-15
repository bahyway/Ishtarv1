//! eridu-scheduler — tick-based job scheduler (§11.2).

pub mod scheduler;
pub use scheduler::{
    EriduScheduler, ScheduledJob, DueJob, JobKind,
    VALIDATION_SWEEP_JOB, VALIDATION_SWEEP_DEFAULT_TICKS,
};
