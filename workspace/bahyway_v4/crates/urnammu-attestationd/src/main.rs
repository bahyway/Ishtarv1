//! urnammu-attestationd — UrNammu Engine boot attestation daemon
//! Source of truth: BC-SEC-001 §6
//!
//! Modes (via argv[1]):
//!   provision  — seal current PCR state as baseline (run once, clean boot)
//!   boot       — one-shot boot attestation + gate handoff
//!   runtime    — interval loop (called by systemd timer)
#![forbid(unsafe_code)]

mod tpm;
mod baseline;
mod kaki;
mod gate;

use std::env;
use std::thread;
use std::time::Duration;

const RUNTIME_INTERVAL_SECS: u64 = 300; // 5-minute recheck

fn main() {
    eprintln!("[urnammu-attestationd] starting");

    let mode = env::args().nth(1).unwrap_or_else(|| "boot".to_string());
    match mode.as_str() {
        "provision" => provision_mode(),
        "boot" => boot_mode(),
        "runtime" => runtime_mode(),
        other => {
            eprintln!("Unknown mode: {other}. Use: provision | boot | runtime");
            std::process::exit(1);
        }
    }
}

fn provision_mode() {
    eprintln!("[urnammu] PROVISION: sealing current PCR state as baseline...");
    match tpm::read_pcr_state() {
        Ok(state) => match baseline::provision_baseline(&state) {
            Ok(_) => eprintln!(
                "[urnammu] Baseline sealed to {}",
                baseline::baseline_path().display()
            ),
            Err(e) => {
                eprintln!("[urnammu] Baseline seal FAILED: {e}");
                std::process::exit(2);
            }
        },
        Err(e) => {
            eprintln!("[urnammu] PCR read FAILED: {e}");
            std::process::exit(2);
        }
    }
}

fn attest() -> bool {
    let observed = match tpm::read_pcr_state() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[urnammu] PCR read error: {e}");
            return false;
        }
    };
    let baseline_path = baseline::baseline_path();
    let baseline = match baseline::load_baseline(&baseline_path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("[urnammu] Baseline load error: {e}");
            return false;
        }
    };
    baseline::compare_constant_time(&observed, &baseline)
}

fn boot_mode() {
    eprintln!("[urnammu] BOOT attestation...");
    let result = attest();
    let event = kaki::AttestationEvent {
        result,
        pcr_banks: "sha256:0,4,7".to_string(),
        boot_phase: "boot".to_string(),
    };
    let _ = kaki::emit_attestation_kaki(&event);
    if let Err(e) = gate::gate_boot_handoff(result) {
        eprintln!("[urnammu] Gate write error: {e}");
        std::process::exit(3);
    }
    if result {
        eprintln!("[urnammu] BOOT PASS — sentinel written, handoff clear");
    } else {
        eprintln!("[urnammu] BOOT FAIL — sentinel written FAIL");
        std::process::exit(4);
    }
}

fn runtime_mode() {
    eprintln!("[urnammu] RUNTIME recheck loop (interval: {RUNTIME_INTERVAL_SECS}s)");
    loop {
        let result = attest();
        let event = kaki::AttestationEvent {
            result,
            pcr_banks: "sha256:0,4,7".to_string(),
            boot_phase: "runtime".to_string(),
        };
        let _ = kaki::emit_attestation_kaki(&event);
        if !result {
            eprintln!("[urnammu] RUNTIME FAIL — AkkadiRulesEngine severity policy triggered");
            // TODO: signal AkkadiRulesEngine to apply severity policy
        }
        thread::sleep(Duration::from_secs(RUNTIME_INTERVAL_SECS));
    }
}
