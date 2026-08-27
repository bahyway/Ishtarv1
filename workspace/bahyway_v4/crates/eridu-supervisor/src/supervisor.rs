//! EriduSupervisor — lifecycle and health management for runtime + scheduler (§11.3).
//!
//! The supervisor owns the runtime and scheduler, coordinates startup/shutdown,
//! and provides a health status for monitoring.

use eridu_runtime::{EriduRuntime, RuntimeState};
use eridu_scheduler::{EriduScheduler, ScheduledJob};

/// Overall system health reported by the supervisor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    /// All subsystems operating normally.
    Healthy,
    /// At least one subsystem has degraded state.
    Degraded,
    /// System is stopped / not started.
    Down,
}

/// The Eridu system supervisor.
pub struct EriduSupervisor {
    pub runtime: EriduRuntime,
    pub scheduler: EriduScheduler,
    running: bool,
}

impl EriduSupervisor {
    pub fn new() -> Self {
        EriduSupervisor {
            runtime: EriduRuntime::new(),
            scheduler: EriduScheduler::new(),
            running: false,
        }
    }

    /// Start the supervisor.
    pub fn start(&mut self) {
        self.running = true;
    }

    /// Graceful shutdown — drains the runtime queue, then stops.
    pub fn shutdown(&mut self) {
        self.runtime.shutdown();
        self.running = false;
    }

    /// Register a scheduled job.
    pub fn register_job(&mut self, job: ScheduledJob) {
        self.scheduler.register(job);
    }

    /// Advance the scheduler by n ticks and submit due jobs to the runtime.
    /// Returns the list of job names that were dispatched this tick.
    pub fn tick<F>(&mut self, n: u64, mut task_fn: F) -> Vec<String>
    where
        F: FnMut(&str) -> Box<dyn eridu_runtime::Task>,
    {
        let due = self.scheduler.tick(n);
        for name in &due {
            let task = task_fn(name.as_str());
            self.runtime.submit(task);
        }
        due
    }

    /// Run all queued tasks.
    pub fn run_pending(&mut self) -> usize {
        self.runtime.run_all()
    }

    /// Health check — evaluates runtime state.
    pub fn health(&self) -> HealthStatus {
        if !self.running {
            return HealthStatus::Down;
        }
        match self.runtime.state() {
            RuntimeState::Shutdown => HealthStatus::Down,
            RuntimeState::Running | RuntimeState::Idle => {
                if self.runtime.failed > 0 {
                    HealthStatus::Degraded
                } else {
                    HealthStatus::Healthy
                }
            }
        }
    }
}

impl Default for EriduSupervisor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eridu_runtime::{Task, TaskResult};

    struct NullTask(String);
    impl Task for NullTask {
        fn name(&self) -> &str {
            &self.0
        }
        fn run(&mut self) -> TaskResult {
            TaskResult::Ok
        }
    }

    #[test]
    fn health_down_before_start() {
        let sup = EriduSupervisor::new();
        assert_eq!(sup.health(), HealthStatus::Down);
    }

    #[test]
    fn health_healthy_after_start() {
        let mut sup = EriduSupervisor::new();
        sup.start();
        assert_eq!(sup.health(), HealthStatus::Healthy);
    }

    #[test]
    fn health_down_after_shutdown() {
        let mut sup = EriduSupervisor::new();
        sup.start();
        sup.shutdown();
        assert_eq!(sup.health(), HealthStatus::Down);
    }

    #[test]
    fn tick_dispatches_due_jobs() {
        let mut sup = EriduSupervisor::new();
        sup.start();
        sup.register_job(ScheduledJob::new("snapshot", 5));
        let dispatched = sup.tick(1, |name| Box::new(NullTask(name.to_string())));
        assert!(dispatched.contains(&"snapshot".to_string()));
        let ok = sup.run_pending();
        assert_eq!(ok, 1);
    }

    #[test]
    fn degraded_when_task_fails() {
        use eridu_runtime::TaskResult;
        struct FailTask;
        impl Task for FailTask {
            fn name(&self) -> &str {
                "fail"
            }
            fn run(&mut self) -> TaskResult {
                TaskResult::Failed("oops".into())
            }
        }
        let mut sup = EriduSupervisor::new();
        sup.start();
        sup.runtime.submit(Box::new(FailTask));
        sup.run_pending();
        assert_eq!(sup.health(), HealthStatus::Degraded);
    }
}
