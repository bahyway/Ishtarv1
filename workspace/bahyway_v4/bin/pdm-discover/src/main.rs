//! pdm-discover — CLI front end for `pdm-discovery`.
//!
//! Usage: pdm-discover <table_name=path.csv> [<table_name=path.csv> ...]
//!
//! Reads each named CSV file as one table, runs the v1 discovery pipeline
//! (`pdm_discovery::discover_schema`), and prints the resulting
//! `SchemaProposal` as JSON to stdout. This is a CLI, not a network
//! endpoint: no server/wire-protocol integration exists yet (DubSar PDM
//! IDE has no GDExtension bridge into any Rust crate in this workspace —
//! confirmed absent when this crate was designed). Wiring an actual
//! Conceptual-tab node to invoke this is separate, tracked follow-up
//! work, not something this binary claims to already do.
use std::env;
use std::process::ExitCode;

use pdm_discovery::{discover_schema, parse_csv_table};

const DEFAULT_MIN_CONFIDENCE: f64 = 0.9;
const DEFAULT_MAX_WEIGHT: f64 = 1.0;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("usage: pdm-discover <table_name=path.csv> [<table_name=path.csv> ...]");
        return ExitCode::FAILURE;
    }

    let mut tables = Vec::with_capacity(args.len());
    for arg in &args {
        let Some((name, path)) = arg.split_once('=') else {
            eprintln!("bad argument (expected table_name=path.csv): {arg}");
            return ExitCode::FAILURE;
        };
        let text = match std::fs::read_to_string(path) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("failed to read {path}: {e}");
                return ExitCode::FAILURE;
            }
        };
        tables.push(parse_csv_table(name, &text));
    }

    let proposal = discover_schema(&tables, DEFAULT_MIN_CONFIDENCE, DEFAULT_MAX_WEIGHT);
    println!("{}", proposal.to_json());
    ExitCode::SUCCESS
}
