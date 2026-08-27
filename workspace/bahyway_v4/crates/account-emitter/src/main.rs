//! account-emitter — the real GL-FLD-001 §10 Anti-SQL account emitter.
//!
//! Closes the gap found live running PB-626's site-seal rite to
//! completion (2026-08-27): `playbook_624_account_emitter.yml` invokes
//! this binary as already-built (`--gate ledger-complete`, `--gate
//! betti-reported`), but it existed nowhere as a real crate — only as a
//! design-tablet reference in
//! `shala-prototypes/batch36_phase3_final_build/624_account_emitter/`.
//! Built for real: a stakeholder's account is a real epistemic-classified
//! ledger and a real computed betti-number shape over the six real
//! account vertices — no SELECT, no FROM, no JOIN, ever (enforced by
//! `playbook_624`'s own Gate A1 source grep over this crate).

mod account;
mod gates;

use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();

    let Some(pos) = args.iter().position(|a| a == "--gate") else {
        eprintln!("usage: account-emitter --gate <ledger-complete|betti-reported>");
        return ExitCode::from(2);
    };
    let Some(gate) = args.get(pos + 1) else {
        eprintln!("account-emitter: --gate needs a value (ledger-complete|betti-reported)");
        return ExitCode::from(2);
    };

    let (ok, detail) = match gate.as_str() {
        "ledger-complete" => gates::gate_ledger_complete(),
        "betti-reported" => gates::gate_betti_reported(),
        other => {
            eprintln!("account-emitter: unknown gate '{other}'");
            return ExitCode::from(2);
        }
    };
    println!("{} {gate}: {detail}", if ok { "PASS" } else { "FAIL" });
    if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
