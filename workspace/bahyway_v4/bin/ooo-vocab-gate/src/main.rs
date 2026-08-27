//! ooo-vocab-gate — CLI front end for `musaru_security::vocab_gate`.
//!
//! Scans one or more paths (files or directories) for SQL/relational/
//! foreign-database vocabulary and ports before a design surface (a
//! Godot wizard scene, an adopted HTML prototype, a doc) is trusted to
//! carry it into `godot/dubsar-theater/` for real. Exits non-zero on any
//! violation, same PASS/FAIL sentinel-style convention
//! `urnammu-attestationd::gate::gate_boot_handoff` uses for its own gate,
//! adapted to a process exit code since there is no boot-handoff sentinel
//! file to write here — a CI/Ansible step just checks the exit status.
#![forbid(unsafe_code)]

use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

use musaru_security::{vocab_scan_dir, vocab_scan_file, VocabScanReport, GATED_EXTENSIONS};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("usage: ooo-vocab-gate <path> [path...]");
        eprintln!("  scans each path (file or directory) for forbidden");
        eprintln!("  SQL/relational/foreign-database vocabulary and ports.");
        eprintln!("  gated extensions: {}", GATED_EXTENSIONS.join(", "));
        return ExitCode::FAILURE;
    }

    let mut reports: Vec<VocabScanReport> = Vec::new();
    for arg in &args {
        let path = PathBuf::from(arg);
        if path.is_dir() {
            match vocab_scan_dir(&path, GATED_EXTENSIONS) {
                Ok(mut r) => reports.append(&mut r),
                Err(e) => {
                    eprintln!("error scanning directory {}: {e}", path.display());
                    return ExitCode::FAILURE;
                }
            }
        } else {
            match vocab_scan_file(&path) {
                Ok(r) => reports.push(r),
                Err(e) => {
                    eprintln!("error scanning file {}: {e}", path.display());
                    return ExitCode::FAILURE;
                }
            }
        }
    }

    let mut total_violations = 0usize;
    let mut files_scanned = 0usize;
    let mut files_dirty = 0usize;

    for report in &reports {
        files_scanned += 1;
        if report.is_clean() {
            continue;
        }
        files_dirty += 1;
        let path_str = report
            .path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_default();
        for v in &report.violations {
            total_violations += 1;
            println!(
                "{path_str}:{}: forbidden vocabulary '{}' -- {}",
                v.line,
                v.matched,
                v.context.trim()
            );
        }
    }

    println!(
        "ooo-vocab-gate: {files_scanned} file(s) scanned, {files_dirty} dirty, {total_violations} violation(s)"
    );

    if total_violations > 0 {
        println!("GATE: FAIL -- forbidden vocabulary found. BahyWay's Seven EnkiDB Types are Triple-O (OOO); they are not SQL, not relational, and never will be.");
        ExitCode::FAILURE
    } else {
        println!("GATE: PASS");
        ExitCode::SUCCESS
    }
}
