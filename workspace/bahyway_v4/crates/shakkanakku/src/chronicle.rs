//! The chronicle — every run event appended as an immutable JSONL line.
//! This file is the bridge to StoryEngine: each line is ready to become
//! an Event particle (kaki_type 0x02) when ingested into EnkiODB.

use crate::model::{ErrorEvent, RunnerEvent};
use serde::Serialize;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Serialize)]
struct ChronicleLine<'a> {
    ts: String,
    event: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pb_index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<&'a ErrorEvent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    note: Option<&'a str>,
}

pub struct Chronicle {
    path: PathBuf,
}

impl Chronicle {
    pub fn open(dir: &str) -> anyhow::Result<Self> {
        std::fs::create_dir_all(dir)?;
        let ts = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        Ok(Self {
            path: Path::new(dir).join(format!("shakkanakku_chronicle_{ts}.jsonl")),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the I/O error instead of swallowing it — the chronicle is an
    /// audit trail; a caller that cares (the runner, which surfaces this as
    /// a visible `RunnerEvent::Log` line) needs to know when it goes dark,
    /// not have that fact silently disappear along with the write.
    fn append(&self, line: &ChronicleLine) -> std::io::Result<()> {
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        let s = serde_json::to_string(line)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        writeln!(f, "{s}")
    }

    pub fn record(&self, ev: &RunnerEvent) -> std::io::Result<()> {
        let ts = chrono::Utc::now().to_rfc3339();
        let line = match ev {
            RunnerEvent::PbStarted(i) => ChronicleLine {
                ts, event: "pb_started", pb_index: Some(*i), detail: None, note: None,
            },
            RunnerEvent::PbOk(i) => ChronicleLine {
                ts, event: "pb_ok", pb_index: Some(*i), detail: None, note: None,
            },
            RunnerEvent::PbWarned(i, e) => ChronicleLine {
                ts, event: "pb_warned", pb_index: Some(*i), detail: Some(e), note: None,
            },
            RunnerEvent::PbFailed(i, e) => ChronicleLine {
                ts, event: "pb_failed_major", pb_index: Some(*i), detail: Some(e), note: None,
            },
            RunnerEvent::Log(l) => ChronicleLine {
                ts, event: "log", pb_index: None, detail: None, note: Some(l),
            },
            RunnerEvent::Finished => ChronicleLine {
                ts, event: "run_finished", pb_index: None, detail: None, note: None,
            },
            RunnerEvent::Aborted => ChronicleLine {
                ts, event: "run_aborted", pb_index: None, detail: None, note: None,
            },
        };
        self.append(&line)
    }

    pub fn note(&self, text: &str) -> std::io::Result<()> {
        self.append(&ChronicleLine {
            ts: chrono::Utc::now().to_rfc3339(),
            event: "architect_act",
            pb_index: None,
            detail: None,
            note: Some(text),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Severity;

    #[test]
    fn chronicle_appends_valid_jsonl() {
        let dir = std::env::temp_dir().join("shk_chr_test");
        let c = Chronicle::open(dir.to_str().unwrap()).unwrap();
        c.record(&RunnerEvent::PbStarted(0)).unwrap();
        c.record(&RunnerEvent::PbFailed(0, ErrorEvent {
            pb_index: 0,
            playbook: "x.yml".into(),
            task: "t".into(),
            error_type: "E".into(),
            message: "m".into(),
            severity: Severity::Major,
            remedy_id: None,
        })).unwrap();
        c.note("architect skipped PB 1").unwrap();
        let text = std::fs::read_to_string(c.path()).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        for l in lines {
            let v: serde_json::Value = serde_json::from_str(l).unwrap();
            assert!(v.get("ts").is_some() && v.get("event").is_some());
        }
    }

    #[test]
    fn append_failure_is_reported_not_swallowed() {
        // Point the chronicle at a path whose parent directory doesn't
        // exist and was never created (open() normally creates `dir`, so
        // construct the struct directly to simulate the write-time failure
        // this test is actually about).
        let c = Chronicle {
            path: PathBuf::from("/nonexistent/shk_chr_missing_dir/x.jsonl"),
        };
        let err = c.record(&RunnerEvent::PbStarted(0));
        assert!(err.is_err(), "a broken chronicle path must surface an error, not succeed silently");
    }
}
