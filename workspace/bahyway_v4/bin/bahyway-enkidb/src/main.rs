//! bahyway-enkidb — real local KAKI-orbit CLI for the seven EnkiDB Types.
//!
//! Closes the gap found triaging PB-670/675 (see
//! `docs/13_changelog/WED0826_MANIFEST_TRIAGE_2026-08-26.md`): 20
//! playbooks invoke this binary; none of them existed anywhere in the
//! workspace. Built on `enkidb-particle-store` (see that crate's own doc
//! comment for exactly what "real" means here — a real local
//! KAKI-minting store, not a fabricated stand-in for a network protocol
//! nobody asked this session to build).
//!
//! Subcommands: `orbit`, `present`, `prove`, `trace`, `clone-tribe`,
//! `decree`, `rehearse`, `segment-policy`, `splits`, `snapshots` — the
//! exact surface the 20 real playbook call sites use, recovered by
//! grepping every `bahyway-enkidb ...` invocation in `playbooks/*.yml`.

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
        "orbit" => cmd::orbit(&flags),
        "present" => cmd::present(&flags),
        "prove" => cmd::prove(&flags),
        "trace" => cmd::trace(&flags),
        "clone-tribe" => cmd::clone_tribe(&flags),
        "decree" => cmd::decree(&flags),
        "rehearse" => cmd::rehearse(&flags),
        "segment-policy" => cmd::segment_policy(&flags),
        "splits" => cmd::splits(&flags),
        "snapshots" => cmd::snapshots(&flags),
        other => {
            eprintln!("bahyway-enkidb: unknown subcommand '{other}'");
            print_usage();
            return ExitCode::from(2);
        }
    };
    match result {
        Ok(code) => code,
        Err(msg) => {
            eprintln!("bahyway-enkidb: error: {msg}");
            ExitCode::FAILURE
        }
    }
}

fn print_usage() {
    eprintln!(
        "usage: bahyway-enkidb <orbit|present|prove|trace|clone-tribe|decree|rehearse|segment-policy|splits|snapshots> [flags]"
    );
}
