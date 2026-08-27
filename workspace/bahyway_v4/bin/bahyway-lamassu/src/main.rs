//! bahyway-lamassu — real TDA shape CLI over `enkidb-particle-store`.
//!
//! Closes the same gap as `bahyway-enkidb` (see that binary's and
//! `enkidb-particle-store`'s doc comments, and
//! `docs/13_changelog/WED0826_MANIFEST_TRIAGE_2026-08-26.md`) for the
//! 6 real `bahyway-lamassu ...` call sites. Unlike `bahyway-enkidb`,
//! this one is a thin real wrapper: `lamassu-engine::LamassuEngine`
//! already exists and already does real persistent-homology math
//! (Vietoris-Rips via `bahyway-algebra::persistence`) — this CLI's own
//! job is only to sample this store's real particles into the point
//! cloud that engine expects and print its real verdict.

mod args;
mod cmd;

use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    let argv: Vec<String> = env::args().skip(1).collect();
    let Some((sub, rest)) = argv.split_first() else {
        eprintln!("usage: bahyway-lamassu <shape|coherence|orbits> [flags]");
        return ExitCode::from(2);
    };
    let flags = args::parse(rest);
    let result = match sub.as_str() {
        "shape" => cmd::shape(&flags),
        "coherence" => cmd::coherence(&flags),
        "orbits" => cmd::orbits(&flags),
        other => {
            eprintln!("bahyway-lamassu: unknown subcommand '{other}'");
            return ExitCode::from(2);
        }
    };
    match result {
        Ok(code) => code,
        Err(msg) => {
            eprintln!("bahyway-lamassu: error: {msg}");
            ExitCode::FAILURE
        }
    }
}
