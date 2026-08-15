//! ashnan-ingest — CLI runner
//! Usage: ashnan-ingest <path/to/archive>
#![forbid(unsafe_code)]

use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <archive_path>", args[0]);
        std::process::exit(1);
    }
    let path = &args[1];
    let bytes = fs::read(path).unwrap_or_else(|e| {
        eprintln!("Failed to read {path}: {e}");
        std::process::exit(1);
    });

    let filename = std::path::Path::new(path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let report = ashnan_ingest::ingest(&bytes, filename);
    println!("ashnan-ingest complete");
    println!("  file       : {}", report.filename);
    println!("  outcome    : {}", report.outcome_label);
    println!("  nrm_code   : {}", report.nrm_code);
    println!("  rows       : {}", report.row_count);
    println!("  archive_id : {}", report.archive_id);
    println!("  blocked    : {}", report.blocked);
    println!("  journal    : {}", report.journal_line);
    ashnan_ingest::hand_to_mashsharu(&report);

    if report.blocked {
        std::process::exit(2);
    }
}
