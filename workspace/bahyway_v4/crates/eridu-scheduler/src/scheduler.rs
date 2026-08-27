//! EriduScheduler — interval-based job scheduling (§11.2).
//!
//! The scheduler maintains a list of named jobs, each with an interval
//! (in logical "ticks"). Callers advance time via `tick(n)` and retrieve
//! the list of due jobs. No wall-clock dependency — fully testable.

/// Well-known job identifier for the SDB validation sweep.
/// Registered via `EriduScheduler::register_validation_sweep()`.
pub const VALIDATION_SWEEP_JOB: &str = "enkisdb.validation_sweep";

/// Default sweep interval matching §12.1: 15 minutes at 1 tick/second.
pub const VALIDATION_SWEEP_DEFAULT_TICKS: u64 = 900;

/// Semantic kind of a job — used by the runtime to dispatch correctly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobKind {
    /// SDB particle validation sweep (§12.1): promotes or quarantines pending particles.
    ValidationSweep,
    /// Generic named job with no special runtime semantics.
    Generic,
}

/// A due job returned by `tick_typed()`.
#[derive(Debug, Clone)]
pub struct DueJob {
    pub name: String,
    pub kind: JobKind,
}

/// A job registered with the scheduler.
#[derive(Debug, Clone)]
pub struct ScheduledJob {
    pub name: String,
    /// Minimum ticks between executions.
    pub interval_ticks: u64,
    /// Semantic kind — used by the runtime dispatcher.
    pub kind: JobKind,
    /// Tick count at last execution. 0 = never run.
    last_run: u64,
}

impl ScheduledJob {
    pub fn new(name: impl Into<String>, interval_ticks: u64) -> Self {
        ScheduledJob {
            name: name.into(),
            interval_ticks,
            kind: JobKind::Generic,
            last_run: 0,
        }
    }

    pub fn with_kind(mut self, kind: JobKind) -> Self {
        self.kind = kind;
        self
    }

    fn is_due(&self, current_tick: u64) -> bool {
        self.last_run == 0 || current_tick.saturating_sub(self.last_run) >= self.interval_ticks
    }

    fn mark_run(&mut self, tick: u64) {
        self.last_run = tick;
    }
}

/// The scheduler — advances a logical tick counter and fires due jobs.
pub struct EriduScheduler {
    jobs: Vec<ScheduledJob>,
    current: u64,
}

impl EriduScheduler {
    pub fn new() -> Self {
        EriduScheduler {
            jobs: Vec::new(),
            current: 0,
        }
    }

    /// Register a new job.
    pub fn register(&mut self, job: ScheduledJob) {
        self.jobs.push(job);
    }

    /// Advance by `n` ticks and return names of jobs that became due.
    pub fn tick(&mut self, n: u64) -> Vec<String> {
        self.tick_typed(n).into_iter().map(|d| d.name).collect()
    }

    /// Advance by `n` ticks and return `DueJob` records (name + kind).
    pub fn tick_typed(&mut self, n: u64) -> Vec<DueJob> {
        self.current += n;
        let current = self.current;
        let mut due = Vec::new();
        for job in &mut self.jobs {
            if job.is_due(current) {
                due.push(DueJob {
                    name: job.name.clone(),
                    kind: job.kind.clone(),
                });
                job.mark_run(current);
            }
        }
        due
    }

    /// Register the sovereign SDB validation sweep job.
    /// `interval_ticks` defaults to 900 (15 minutes); admin can override.
    pub fn register_validation_sweep(&mut self, interval_ticks: u64) {
        self.register(
            ScheduledJob::new(VALIDATION_SWEEP_JOB, interval_ticks)
                .with_kind(JobKind::ValidationSweep),
        );
    }

    /// Change the interval of an already-registered job by name.
    /// Returns true if the job was found, false if unknown.
    pub fn set_job_interval(&mut self, name: &str, new_interval_ticks: u64) -> bool {
        assert!(new_interval_ticks > 0, "interval must be > 0 ticks");
        if let Some(job) = self.jobs.iter_mut().find(|j| j.name == name) {
            job.interval_ticks = new_interval_ticks;
            true
        } else {
            false
        }
    }

    pub fn current_tick(&self) -> u64 {
        self.current
    }
    pub fn job_count(&self) -> usize {
        self.jobs.len()
    }
}

impl Default for EriduScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn job_fires_on_first_tick() {
        let mut s = EriduScheduler::new();
        s.register(ScheduledJob::new("snapshot", 10));
        let due = s.tick(1);
        assert!(
            due.contains(&"snapshot".to_string()),
            "job should fire immediately on first tick (last_run=0)"
        );
    }

    #[test]
    fn job_fires_after_interval() {
        let mut s = EriduScheduler::new();
        s.register(ScheduledJob::new("snapshot", 10));
        s.tick(1); // first run at tick 1
        let due2 = s.tick(9); // now at tick 10 — interval since last_run=1 is 9 < 10
        assert!(
            due2.is_empty(),
            "not yet due at tick 10 (last_run=1, need 10 ticks)"
        );
        let due3 = s.tick(1); // tick 11: 11-1=10 >= 10 — due
        assert!(due3.contains(&"snapshot".to_string()));
    }

    #[test]
    fn multiple_jobs_independent() {
        let mut s = EriduScheduler::new();
        s.register(ScheduledJob::new("fast", 5));
        s.register(ScheduledJob::new("slow", 20));
        s.tick(1); // both fire on first tick
        let at_5 = s.tick(4); // tick=5, fast due (5-1=4 < 5), slow not (4 < 20)
        assert!(at_5.is_empty());
        let at_6 = s.tick(1); // tick=6, fast due (6-1=5 >= 5)
        assert!(at_6.contains(&"fast".to_string()));
        assert!(!at_6.contains(&"slow".to_string()));
    }

    #[test]
    fn no_jobs_empty_due() {
        let mut s = EriduScheduler::new();
        assert!(s.tick(100).is_empty());
    }

    #[test]
    fn validation_sweep_job_registers_with_correct_kind() {
        let mut s = EriduScheduler::new();
        s.register_validation_sweep(VALIDATION_SWEEP_DEFAULT_TICKS);
        assert_eq!(s.job_count(), 1);

        let due = s.tick_typed(1); // fires on first tick
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].name, VALIDATION_SWEEP_JOB);
        assert_eq!(due[0].kind, JobKind::ValidationSweep);
    }

    #[test]
    fn validation_sweep_fires_every_900_ticks() {
        let mut s = EriduScheduler::new();
        s.register_validation_sweep(VALIDATION_SWEEP_DEFAULT_TICKS);

        let first = s.tick_typed(1);
        assert_eq!(first.len(), 1); // fires immediately (last_run=0)

        // advance 899 more ticks → total 900, interval since last_run=1 is 899 < 900
        let mid = s.tick_typed(899);
        assert!(
            mid.is_empty(),
            "should not fire at tick 900 (899 ticks since last)"
        );

        // one more tick → 900 ticks since last_run
        let next = s.tick_typed(1);
        assert_eq!(next.len(), 1);
        assert_eq!(next[0].kind, JobKind::ValidationSweep);
    }

    #[test]
    fn admin_can_reconfigure_interval() {
        let mut s = EriduScheduler::new();
        s.register_validation_sweep(900);
        s.tick(1); // fire once to set last_run

        let changed = s.set_job_interval(VALIDATION_SWEEP_JOB, 300);
        assert!(changed);

        // At tick 301 (300 after last_run=1) it should fire
        let due = s.tick_typed(300);
        assert_eq!(due.len(), 1);
    }

    #[test]
    fn set_job_interval_returns_false_for_unknown_job() {
        let mut s = EriduScheduler::new();
        assert!(!s.set_job_interval("nonexistent", 100));
    }

    #[test]
    fn tick_typed_and_tick_agree() {
        let mut s1 = EriduScheduler::new();
        let mut s2 = EriduScheduler::new();
        s1.register(ScheduledJob::new("job", 5));
        s2.register(ScheduledJob::new("job", 5));

        for n in [1u64, 2, 3, 4, 5, 1, 5] {
            let plain = s1.tick(n);
            let typed = s2.tick_typed(n);
            let names: Vec<String> = typed.into_iter().map(|d| d.name).collect();
            assert_eq!(plain, names, "tick and tick_typed must agree at n={n}");
        }
    }
}
