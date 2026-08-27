//! A real, tested subprocess wrapper that invokes WHATEVER external
//! structure-from-motion (SfM) engine a deployment configures (e.g.
//! OpenDroneMap, COLMAP, Meshroom, or a containerized wrapper around
//! one), then locates and parses the point-cloud file it produces.
//!
//! No specific SfM engine ships with or was verified installed in the
//! workspace that built this crate — confirmed absent (`which odm
//! colmap meshroom` and a repo-wide search all came back empty; only
//! the `docker` binary exists, no running engine). This mirrors the
//! ecosystem's existing `girsu-vscodium-src` pattern ("the real build
//! lives outside this repo") rather than fabricating one: `SfmConfig`
//! is fully caller-supplied (command, args, expected output path), and
//! this module's own tests prove the plumbing against a stub script
//! standing in for a real engine, not a fabricated real reconstruction.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::{parse_point_cloud_file, ParseError, PointCloud};

#[derive(Debug, Clone)]
pub struct SfmConfig {
    /// The SfM engine's executable — e.g. `"odm_task"`, `"docker"`, or a
    /// wrapper script. Never assumed by this crate; always supplied by
    /// whoever configured the real engine for a given deployment.
    pub command: String,
    /// Arguments to pass before the photos-directory argument.
    pub args: Vec<String>,
    /// Where the engine is expected to write its output point cloud once
    /// it exits successfully. This crate does not guess an engine's
    /// output-directory convention (e.g. ODM's own layout) — the caller,
    /// who knows which real engine is configured, supplies it.
    pub output_point_cloud_path: PathBuf,
}

#[derive(Debug)]
pub enum SfmError {
    Spawn(String),
    NonZeroExit { code: Option<i32>, stderr: String },
    Parse(ParseError),
}

impl std::fmt::Display for SfmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SfmError::Spawn(e) => write!(f, "failed to launch SfM engine: {e}"),
            SfmError::NonZeroExit { code, stderr } => {
                write!(f, "SfM engine exited with code {code:?}: {stderr}")
            }
            SfmError::Parse(e) => {
                write!(
                    f,
                    "SfM engine finished, but its output point cloud didn't parse: {e}"
                )
            }
        }
    }
}
impl std::error::Error for SfmError {}

/// Run the configured SfM engine against `photos_dir`, then parse
/// whatever point-cloud file it left at `cfg.output_point_cloud_path`.
pub fn run_sfm(photos_dir: &Path, cfg: &SfmConfig) -> Result<PointCloud, SfmError> {
    let output = Command::new(&cfg.command)
        .args(&cfg.args)
        .arg(photos_dir)
        .output()
        .map_err(|e| SfmError::Spawn(e.to_string()))?;

    if !output.status.success() {
        return Err(SfmError::NonZeroExit {
            code: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }

    parse_point_cloud_file(&cfg.output_point_cloud_path).map_err(SfmError::Parse)
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn unique_temp_dir(tag: &str) -> PathBuf {
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!(
            "photogrammetry_ingest_test_{tag}_{}_{n}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_stub_script(dir: &Path, name: &str, body: &str) -> PathBuf {
        let path = dir.join(name);
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(f, "#!/bin/sh").unwrap();
        f.write_all(body.as_bytes()).unwrap();
        let mut perms = std::fs::metadata(&path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&path, perms).unwrap();
        path
    }

    const STUB_PLY: &str = "ply\nformat ascii 1.0\nelement vertex 2\nproperty float x\nproperty float y\nproperty float z\nend_header\n0 0 0\n1 1 1\n";

    #[test]
    fn a_successful_stub_engine_produces_a_parsed_point_cloud() {
        // Stands in for a real SfM engine (OpenDroneMap/COLMAP/etc.):
        // this proves run_sfm's subprocess-then-parse wiring works end
        // to end without needing an actual reconstruction, which this
        // sandbox has no engine installed to perform.
        let dir = unique_temp_dir("ok");
        let out_path = dir.join("cloud.ply");
        let script = write_stub_script(
            &dir,
            "fake_engine.sh",
            &format!(
                "cat > '{}' <<'EOF'\n{STUB_PLY}EOF\nexit 0\n",
                out_path.display()
            ),
        );
        let cfg = SfmConfig {
            command: script.to_string_lossy().to_string(),
            args: vec![],
            output_point_cloud_path: out_path,
        };
        let cloud = run_sfm(&dir, &cfg).unwrap();
        assert_eq!(cloud.points.len(), 2);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_failing_engine_reports_its_nonzero_exit_and_stderr() {
        let dir = unique_temp_dir("fail");
        let script = write_stub_script(
            &dir,
            "fake_engine.sh",
            "echo 'reconstruction failed: no matches found' 1>&2\nexit 1\n",
        );
        let cfg = SfmConfig {
            command: script.to_string_lossy().to_string(),
            args: vec![],
            output_point_cloud_path: dir.join("never_written.ply"),
        };
        let err = run_sfm(&dir, &cfg).unwrap_err();
        match err {
            SfmError::NonZeroExit { code, stderr } => {
                assert_eq!(code, Some(1));
                assert!(stderr.contains("no matches found"));
            }
            other => panic!("expected NonZeroExit, got {other}"),
        }
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn an_engine_that_produces_unparseable_output_reports_a_parse_error() {
        let dir = unique_temp_dir("badout");
        let out_path = dir.join("cloud.ply");
        let script = write_stub_script(
            &dir,
            "fake_engine.sh",
            &format!("echo 'not a real ply' > '{}'\nexit 0\n", out_path.display()),
        );
        let cfg = SfmConfig {
            command: script.to_string_lossy().to_string(),
            args: vec![],
            output_point_cloud_path: out_path,
        };
        let err = run_sfm(&dir, &cfg).unwrap_err();
        assert!(matches!(err, SfmError::Parse(_)), "{err}");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_nonexistent_command_reports_a_spawn_error_not_a_panic() {
        let dir = unique_temp_dir("spawn");
        let cfg = SfmConfig {
            command: "/definitely/not/a/real/sfm/engine/binary".to_string(),
            args: vec![],
            output_point_cloud_path: dir.join("cloud.ply"),
        };
        let err = run_sfm(&dir, &cfg).unwrap_err();
        assert!(matches!(err, SfmError::Spawn(_)), "{err}");
        std::fs::remove_dir_all(&dir).ok();
    }
}
