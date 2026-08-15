//! Shakkanakku core model — statuses, events, phases.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Major,
    Warning,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Major => write!(f, "MAJOR"),
            Severity::Warning => write!(f, "WARNING"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PbStatus {
    Pending,
    Running,
    Ok,
    Warned,
    Failed,
    Skipped,
}

impl PbStatus {
    pub fn icon(&self) -> &'static str {
        match self {
            PbStatus::Pending => "·",
            PbStatus::Running => "▶",
            PbStatus::Ok => "✓",
            PbStatus::Warned => "⚠",
            PbStatus::Failed => "✖",
            PbStatus::Skipped => "»",
        }
    }
}

/// One error/warning event captured from a playbook run.
#[derive(Debug, Clone, Serialize)]
pub struct ErrorEvent {
    pub pb_index: usize,
    pub playbook: String,
    pub task: String,
    pub error_type: String,
    pub message: String,
    pub severity: Severity,
    /// id of the matched remedy rule, if any
    pub remedy_id: Option<String>,
}

/// Events streamed from the runner thread to the UI (Enmebulugga:
/// the runner writes, the view reads, neither blocks the other).
#[derive(Debug, Clone)]
pub enum RunnerEvent {
    PbStarted(usize),
    PbOk(usize),
    PbWarned(usize, ErrorEvent),
    PbFailed(usize, ErrorEvent),
    Log(String),
    Finished,
    Aborted,
}

/// Control commands from the Architect (UI) to the runner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ctl {
    Abort,
    SkipContinue,
    Retry,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunPhase {
    Idle,
    Running,
    /// Halted on a major error at playbook index — awaiting the Architect (CSR-08).
    Halted(usize),
    Finished,
    Aborted,
}
