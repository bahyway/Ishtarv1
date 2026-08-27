//! bahyway-enlil — real locality/layer-life/field CLI over
//! `enkidb-particle-store`.
//!
//! Closes the *third* gap flagged (not fixed) in
//! `docs/13_changelog/WED0826_BAHYWAY_ENKIDB_LAMASSU_CLI_2026-08-26.md`'s
//! own "Not fixed here" section: `playbook_655`/`playbook_663`/
//! `playbook_670`/`playbook_671` invoke `bahyway-enlil blob-locality`/
//! `locality`/`layer-metrics`/`field`/`prove` (7 call sites across 4
//! playbooks) as an already-installed CLI that, like `bahyway-enkidb`/
//! `bahyway-lamassu` before it, existed nowhere as a buildable binary.
//! Same discovery method (grep every real call site before designing),
//! same honesty rule as those two: every number is either computed from
//! real stored particles/real `bahyway-algebra` orbital math, or a
//! documented real zero/empty — never fabricated to make a downstream
//! gate pass.
//!
//! Subcommands: `blob-locality`, `locality`, `layer-metrics`, `field`,
//! `prove` — the exact surface the 7 real playbook call sites use.

mod args;
mod cmd;

use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    let argv: Vec<String> = env::args().skip(1).collect();
    let Some((sub, rest)) = argv.split_first() else {
        print_usage();
        return ExitCode::from(2);
    };
    let flags = args::parse(rest);
    let result = match sub.as_str() {
        "blob-locality" => cmd::blob_locality(&flags),
        "locality" => cmd::locality(&flags),
        "layer-metrics" => cmd::layer_metrics(&flags),
        "field" => cmd::field(&flags),
        "prove" => cmd::prove(&flags),
        other => {
            eprintln!("bahyway-enlil: unknown subcommand '{other}'");
            print_usage();
            return ExitCode::from(2);
        }
    };
    match result {
        Ok(code) => code,
        Err(msg) => {
            eprintln!("bahyway-enlil: error: {msg}");
            ExitCode::FAILURE
        }
    }
}

fn print_usage() {
    eprintln!("usage: bahyway-enlil <blob-locality|locality|layer-metrics|field|prove> [flags]");
}
