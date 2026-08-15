//! EriduRuntime — synchronous cooperative task executor (§11).
//!
//! Named after Eridu (oldest city in Mesopotamia). The runtime manages
//! a queue of named tasks and dispatches them one at a time.
//! No async, no tokio — pure Rust cooperative execution.

use std::collections::VecDeque;

/// A named unit of work the runtime can dispatch.
pub trait Task: Send {
    fn name(&self) -> &str;
    fn run(&mut self) -> TaskResult;
}

/// Outcome of one task execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskResult {
    Ok,
    Retry,
    Failed(String),
}

/// The runtime's operational state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeState {
    Idle,
    Running,
    Shutdown,
}

/// EriduRuntime — holds a queue of tasks and runs them cooperatively.
pub struct EriduRuntime {
    queue:   VecDeque<Box<dyn Task>>,
    state:   RuntimeState,
    /// How many tasks completed successfully this session.
    pub completed: usize,
    /// How many tasks failed this session.
    pub failed:    usize,
}

impl EriduRuntime {
    pub fn new() -> Self {
        EriduRuntime {
            queue:     VecDeque::new(),
            state:     RuntimeState::Idle,
            completed: 0,
            failed:    0,
        }
    }

    /// Submit a task to the queue.
    pub fn submit(&mut self, task: Box<dyn Task>) {
        self.queue.push_back(task);
    }

    /// Run all queued tasks and return after the queue is empty.
    /// Returns the number of tasks that completed with `TaskResult::Ok`.
    pub fn run_all(&mut self) -> usize {
        self.state = RuntimeState::Running;
        let mut ok_count = 0;
        while let Some(mut task) = self.queue.pop_front() {
            match task.run() {
                TaskResult::Ok => {
                    self.completed += 1;
                    ok_count += 1;
                }
                TaskResult::Retry => {
                    // Re-enqueue at end for one more attempt
                    self.queue.push_back(task);
                }
                TaskResult::Failed(_) => {
                    self.failed += 1;
                }
            }
        }
        self.state = RuntimeState::Idle;
        ok_count
    }

    pub fn state(&self) -> RuntimeState { self.state }
    pub fn queue_len(&self) -> usize     { self.queue.len() }

    pub fn shutdown(&mut self) {
        self.queue.clear();
        self.state = RuntimeState::Shutdown;
    }
}

impl Default for EriduRuntime {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct EchoTask(String, bool);
    impl Task for EchoTask {
        fn name(&self) -> &str { &self.0 }
        fn run(&mut self) -> TaskResult {
            if self.1 { TaskResult::Ok } else { TaskResult::Failed("fail".into()) }
        }
    }

    #[test]
    fn run_single_ok_task() {
        let mut rt = EriduRuntime::new();
        rt.submit(Box::new(EchoTask("t1".into(), true)));
        let ok = rt.run_all();
        assert_eq!(ok, 1);
        assert_eq!(rt.completed, 1);
        assert_eq!(rt.failed, 0);
    }

    #[test]
    fn failed_task_increments_counter() {
        let mut rt = EriduRuntime::new();
        rt.submit(Box::new(EchoTask("t1".into(), false)));
        let ok = rt.run_all();
        assert_eq!(ok, 0);
        assert_eq!(rt.failed, 1);
    }

    #[test]
    fn queue_empty_after_run() {
        let mut rt = EriduRuntime::new();
        rt.submit(Box::new(EchoTask("t1".into(), true)));
        rt.submit(Box::new(EchoTask("t2".into(), true)));
        rt.run_all();
        assert_eq!(rt.queue_len(), 0);
    }

    #[test]
    fn shutdown_clears_queue() {
        let mut rt = EriduRuntime::new();
        rt.submit(Box::new(EchoTask("t1".into(), true)));
        rt.shutdown();
        assert_eq!(rt.queue_len(), 0);
        assert_eq!(rt.state(), RuntimeState::Shutdown);
    }
}
