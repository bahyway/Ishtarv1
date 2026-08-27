//! najaf-ingest — Wadi Al-Salaam civil registry ingestion (§10, PA-15).
//!
//! Ingests civil registry records for Najaf (sovereign Iraqi identity).
//! Each record becomes a KAKI particle committed to the Journal via
//! the full pipeline: AdadGate → MusaruCheck → VGCA → PersistedDb.
//!
//! Input format (stdin or --file): tab-separated lines
//!   <name>\t<epoch>\t<state>
//!   e.g.  Ali_Karim\t1\tGolden
//!
//! Usage: najaf-ingest [--tribe <hex>] [--file <path>] [--data-dir <dir>]

use std::env;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::PathBuf;

use adad_gate::{AdadGate, ArrivalRecord};
use bahyway_core::{ParticleState, TribeId};
use enkidb_kaki::KakiRole;
use enkidb_persist::PersistedDb;
use enkidb_storage::FsyncPolicy;
use musaru_security::check_sovereignty;
use story_engine::projection::{encode_state, ATTR_STATE};
use template_library::civil_registry_template;
use vgca_validation::validate;

fn parse_state(s: &str) -> ParticleState {
    match s.trim() {
        "Golden" | "golden" => ParticleState::Golden,
        "Dead" | "dead" => ParticleState::Dead,
        _ => ParticleState::Fuzzy,
    }
}

fn ingest_lines<R: BufRead>(reader: R, gate: &AdadGate, pdb: &mut PersistedDb) {
    let template = civil_registry_template();
    let tid = gate.tribe_id();
    let mut ok = 0usize;
    let mut skip = 0usize;

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        let parts: Vec<&str> = line.splitn(3, '\t').collect();
        if parts.len() < 2 {
            skip += 1;
            continue;
        }

        let epoch = parts[1].trim().parse::<u32>().unwrap_or(1);
        let state = if parts.len() >= 3 {
            parse_state(parts[2])
        } else {
            ParticleState::Fuzzy
        };

        let record = ArrivalRecord {
            attrs: vec![(ATTR_STATE, encode_state(state).to_vec())],
            epoch,
            role: KakiRole::Zikru,
        };

        let gr = match gate.ingest(record) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("[skip] ingest error: {e}");
                skip += 1;
                continue;
            }
        };

        let sec = check_sovereignty(tid, &gr.particle);
        if !sec.is_approved() {
            eprintln!("[skip] sovereignty rejected: {sec:?}");
            skip += 1;
            continue;
        }

        let vr = validate(&template, &gr.eav);
        if !vr.is_valid() {
            eprintln!(
                "[skip] validation failed — missing: {:?}",
                vr.missing_required
            );
            skip += 1;
            continue;
        }

        if let Err(e) = pdb.register_particle(&gr.particle) {
            eprintln!("[skip] register error: {e}");
            skip += 1;
            continue;
        }
        if let Err(e) = pdb.commit(gr.event_kaki, gr.particle, gr.epoch, gr.eav) {
            eprintln!("[skip] commit error: {e}");
            skip += 1;
        } else {
            ok += 1;
        }
    }

    eprintln!(
        "𒁾 ingested={ok}  skipped={skip}  on-disk={}",
        pdb.stats().entries_written
    );
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut tribe_id = 0x0001u16;
    let mut file_path: Option<String> = None;
    let mut data_dir = env::var("HOME")
        .map(|h| format!("{h}/.bahyway"))
        .unwrap_or_else(|_| "/tmp/.bahyway".to_string());
    let mut i = 1;

    while i < args.len() {
        match args[i].as_str() {
            "--tribe" if i + 1 < args.len() => {
                let hex = args[i + 1].trim_start_matches("0x");
                tribe_id = u16::from_str_radix(hex, 16).unwrap_or(0x0001);
                i += 2;
            }
            "--file" if i + 1 < args.len() => {
                file_path = Some(args[i + 1].clone());
                i += 2;
            }
            "--data-dir" if i + 1 < args.len() => {
                data_dir = args[i + 1].clone();
                i += 2;
            }
            "--help" | "-h" => {
                eprintln!("Usage: najaf-ingest [--tribe <hex>] [--file <path>] [--data-dir <dir>]");
                eprintln!("Input: <name>\\t<epoch>\\t<state>  (Golden/Fuzzy/Dead)");
                std::process::exit(0);
            }
            _ => {
                i += 1;
            }
        }
    }

    eprintln!("𒁾 najaf-ingest — Wadi Al-Salaam civil registry v4.0");
    eprintln!("  tribe    : {tribe_id:#06x}");
    eprintln!("  data-dir : {data_dir}");

    let tid = TribeId::from_u16(tribe_id);
    let gate = AdadGate::new(tid);
    let mut pdb = PersistedDb::open(&PathBuf::from(&data_dir), tid, FsyncPolicy::PerCommit)
        .unwrap_or_else(|e| {
            eprintln!("error: cannot open database: {e}");
            std::process::exit(1);
        });

    if pdb.stats().entries_replayed > 0 {
        eprintln!(
            "  replayed : {} existing entries",
            pdb.stats().entries_replayed
        );
    }

    match file_path {
        Some(path) => match File::open(&path) {
            Ok(f) => ingest_lines(io::BufReader::new(f), &gate, &mut pdb),
            Err(e) => {
                eprintln!("Cannot open {path}: {e}");
                std::process::exit(1);
            }
        },
        None => {
            eprintln!("  reading from stdin (Ctrl-D to finish)");
            ingest_lines(io::stdin().lock(), &gate, &mut pdb);
        }
    }
}
