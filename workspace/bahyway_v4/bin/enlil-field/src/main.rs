//! enlil-field — the real GL-FLD-001 wave-field kernel.
//!
//! Closes the gap found live running PB-626's site-seal rite to
//! completion (2026-08-27): `playbook_623_enlil_field_kernel.yml`
//! invokes this binary as already-built (`--gate conservation`,
//! `--gate descent`, `--bench <n> --assert-under-ms <ms>`), but it
//! existed nowhere as a real crate — only as a design-tablet reference
//! in `shala-prototypes/batch36_phase3_final_build/623_enlil_field_kernel/`.
//! Built for real: every gate scatters real particles out of the local
//! `enkidb-particle-store` substrate through the real KAKI-derived
//! orbital math `bahyway-enlil`/`bahyway-lamassu` already use — no
//! fabricated results.

mod config;
mod kernel;

use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let cfg = config::load();

    if let Some(pos) = args.iter().position(|a| a == "--gate") {
        let Some(gate) = args.get(pos + 1) else {
            eprintln!("enlil-field: --gate needs a value (conservation|descent)");
            return ExitCode::from(2);
        };
        let (ok, detail) = match gate.as_str() {
            "conservation" => kernel::gate_conservation(&cfg),
            "descent" => kernel::gate_descent(&cfg),
            other => {
                eprintln!("enlil-field: unknown gate '{other}'");
                return ExitCode::from(2);
            }
        };
        println!("{} {gate}: {detail}", if ok { "PASS" } else { "FAIL" });
        return if ok { ExitCode::SUCCESS } else { ExitCode::FAILURE };
    }

    if let Some(pos) = args.iter().position(|a| a == "--bench") {
        let Some(n) = args.get(pos + 1).and_then(|s| s.parse::<u64>().ok()) else {
            eprintln!("enlil-field: --bench needs a numeric particle count");
            return ExitCode::from(2);
        };
        let threshold_ms = args
            .iter()
            .position(|a| a == "--assert-under-ms")
            .and_then(|p| args.get(p + 1))
            .and_then(|s| s.parse::<u128>().ok());

        let elapsed = kernel::bench_scatter(&cfg, n);
        let ms = elapsed.as_millis();
        println!(
            "BENCH scattered {n} particle(s) into {}x{}x{} grid in {ms}ms",
            cfg.shells, cfg.angles, cfg.radii
        );

        if let Some(limit) = threshold_ms {
            let ok = ms < limit;
            println!("{} assert-under-ms {limit}: {ms}ms", if ok { "PASS" } else { "FAIL" });
            return if ok { ExitCode::SUCCESS } else { ExitCode::FAILURE };
        }
        return ExitCode::SUCCESS;
    }

    eprintln!("usage: enlil-field --gate <conservation|descent> | --bench <n> [--assert-under-ms <ms>]");
    ExitCode::from(2)
}
