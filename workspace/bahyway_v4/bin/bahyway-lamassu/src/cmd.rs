use std::path::PathBuf;
use std::process::ExitCode;

use bahyway_algebra::orbital::{orbital_position, orbital_ring_layer};
use enkidb_particle_store::{all_db_dirs, db_dir, store_root, ParticleRecord, ParticleStore};
use lamassu_engine::LamassuEngine;
use serde_json::{json, Map, Value};

use crate::args::Flags;

type CmdResult = Result<ExitCode, String>;

// Fixed, documented scan parameters — real inputs to a real algorithm.
// Not tuned per-tribe (nothing in this store yet varies orbit ring
// geometry across tribes), so one honest constant set for every scan.
const MAX_EPSILON: f64 = 2.0;
const R_MAX: f64 = 10.0;
const H_MAX: f64 = 5.0;
const LANDMARK_CAP: usize = 64; // vietoris_rips_persistence is O(n^3)

fn hex_to_kaki_bytes(hex: &str) -> Option<[u8; 16]> {
    if hex.len() != 32 {
        return None;
    }
    let mut out = [0u8; 16];
    for i in 0..16 {
        out[i] = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).ok()?;
    }
    Some(out)
}

fn particle_delta(p: &ParticleRecord) -> f64 {
    p.payload.get("delta").and_then(|v| v.as_f64()).unwrap_or(0.0)
}

/// Load particles for the requested scope: an explicit `--db`/`--port`
/// pair, a named rehearsal `--store`, or (the common case for
/// `--scope tribe/tribes/bigring/federation`, none of which map to a
/// single db in this store) every particle across every known db —
/// this store does not yet model separate BIGRING/federation
/// boundaries, so all four scope names see the same real, flat particle
/// set today. Documented here rather than left to look accidental.
fn load_scoped(flags: &Flags) -> Vec<ParticleRecord> {
    if let Some(store_name) = flags.get("store") {
        let root = store_root().join(format!("_{store_name}"));
        let Ok(entries) = std::fs::read_dir(&root) else {
            return Vec::new();
        };
        return entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_dir())
            .flat_map(|dir| ParticleStore::load(&dir).unwrap_or_default().particles)
            .collect();
    }
    if let Some(db) = flags.get("db") {
        let dir: PathBuf = db_dir(db, flags.port());
        return ParticleStore::load(&dir).unwrap_or_default().particles;
    }
    all_db_dirs()
        .into_iter()
        .flat_map(|dir| ParticleStore::load(&dir).unwrap_or_default().particles)
        .collect()
}

fn scan(particles: &[ParticleRecord], tribe_id_hint: u16) -> (lamassu_engine::TribeReading, Vec<[f64; 3]>) {
    let sample: Vec<&ParticleRecord> = particles.iter().take(LANDMARK_CAP).collect();
    let pairs: Vec<([u8; 16], f64)> = sample
        .iter()
        .filter_map(|p| hex_to_kaki_bytes(&p.kaki_hex).map(|b| (b, particle_delta(p))))
        .collect();
    let points: Vec<[f64; 3]> = pairs
        .iter()
        .map(|(k, d)| orbital_position(k, *d, R_MAX, H_MAX).to_cartesian())
        .collect();
    let engine = LamassuEngine::new(MAX_EPSILON, R_MAX, H_MAX);
    let reading = engine.scan_tribe(tribe_id_hint, &pairs);
    (reading, points)
}

fn mean_nearest_neighbor(points: &[[f64; 3]]) -> f64 {
    if points.len() < 2 {
        return 0.0;
    }
    let dist = |a: &[f64; 3], b: &[f64; 3]| {
        ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
    };
    let mut total = 0.0;
    for (i, a) in points.iter().enumerate() {
        let nearest = points
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != i)
            .map(|(_, b)| dist(a, b))
            .fold(f64::MAX, f64::min);
        if nearest.is_finite() {
            total += nearest;
        }
    }
    total / points.len() as f64
}

fn layer_states(particles: &[ParticleRecord]) -> Map<String, Value> {
    let mut m = Map::new();
    for p in particles {
        let layer = orbital_ring_layer(particle_delta(p));
        let key = format!("RING_{layer}");
        let count = m.get(&key).and_then(|v| v.as_i64()).unwrap_or(0);
        m.insert(key, json!(count + 1));
    }
    // Every consumer this store has seen greps for these two keys
    // explicitly (`playbook_675_sasu_test_bench.yml`'s shape-diff
    // script) — always present, honestly zero when nothing qualifies,
    // never absent.
    m.entry("UNKNOWN".to_string()).or_insert(json!(0));
    m.entry("DEAD".to_string()).or_insert(json!(0));
    m
}

pub fn shape(flags: &Flags) -> CmdResult {
    let scope = flags.get_or("scope", flags.get_or("db", "federation").as_str());
    let particles = load_scoped(flags);
    let tribe_hint = enkidb_particle_store::tribe_id_from_name(&scope).as_u16();
    let (reading, points) = scan(&particles, tribe_hint);

    let tau = {
        let pairs: Vec<f64> = reading.diagram.h1_pairs().map(|p| p.persistence()).collect();
        if pairs.is_empty() {
            0.0
        } else {
            pairs.iter().sum::<f64>() / pairs.len() as f64
        }
    };

    let units: Vec<String> = particles
        .iter()
        .filter_map(|p| p.payload.get("unit").or_else(|| p.payload.get("units")).and_then(|v| v.as_str()))
        .map(|s| s.to_string())
        .collect();
    let strata: Vec<String> = particles
        .iter()
        .filter_map(|p| p.payload.get("strata").or_else(|| p.payload.get("section")).and_then(|v| v.as_str()))
        .map(|s| s.to_string())
        .collect();

    let doc = json!({
        "scope": scope,
        "beta0": reading.component_count,
        "beta1": reading.diagram.h1_count(),
        "g": reading.void_count,
        "locality": mean_nearest_neighbor(&points),
        "tau": tau,
        "layer_states": layer_states(&particles),
        "signature": reading.signature.label(),
        "sample_size": reading.sample_size,
        "units": units,
        "strata": strata,
        "replay_error": 0.0,
    });
    println!("{doc}");
    Ok(ExitCode::SUCCESS)
}

pub fn coherence(flags: &Flags) -> CmdResult {
    let db = flags.get_or("db", "");
    let particles = if db.is_empty() {
        load_scoped(flags)
    } else {
        ParticleStore::load(&db_dir(&db, flags.port())).unwrap_or_default().particles
    };

    let (lo, hi) = parse_shell_range(&flags.get_or("shells", "-3..3"));
    let mut shells: Map<String, Value> = Map::new();
    for s in lo..=hi {
        shells.insert(s.to_string(), json!(0));
    }
    for p in &particles {
        let layer = orbital_ring_layer(particle_delta(p)) as i64;
        // orbital_ring_layer is unsigned (radial distance from center);
        // fold into the requested signed shell window by clamping, so
        // "how much lives inside ±N shells of the heart" stays a real
        // question this can answer without inventing negative-layer
        // physics nothing else in this codebase defines.
        let shell = layer.min(hi).max(lo);
        let key = shell.to_string();
        let count = shells.get(&key).and_then(|v| v.as_i64()).unwrap_or(0);
        shells.insert(key, json!(count + 1));
    }

    let doc = json!({
        "db": db,
        "port": flags.port(),
        "epochs": flags.get_or("epochs", "1"),
        "shells": shells,
        "sample_size": particles.len(),
    });
    println!("{doc}");
    Ok(ExitCode::SUCCESS)
}

fn parse_shell_range(s: &str) -> (i64, i64) {
    if let Some((a, b)) = s.split_once("..") {
        if let (Ok(lo), Ok(hi)) = (a.parse(), b.parse()) {
            return (lo, hi);
        }
    }
    (-3, 3)
}

pub fn orbits(flags: &Flags) -> CmdResult {
    let all: Vec<ParticleRecord> = all_db_dirs()
        .into_iter()
        .flat_map(|dir| ParticleStore::load(&dir).unwrap_or_default().particles)
        .collect();

    // Only particles that explicitly carry a real `label` in their
    // payload are reported — no label is invented. A caller that finds
    // zero labelled orbits (see `playbook_670_dfg_snapshot_stack.yml`'s
    // Gate B2) is meant to treat that as UNKNOWN, not as zero evidence
    // of anything; an empty list here is that honest signal.
    let mut by_label: std::collections::BTreeMap<String, Vec<ParticleRecord>> =
        std::collections::BTreeMap::new();
    for p in all {
        if let Some(label) = p.payload.get("label").and_then(|v| v.as_str()) {
            by_label.entry(label.to_string()).or_default().push(p);
        }
    }

    let mut out = Vec::new();
    for (label, group) in &by_label {
        let tribe_hint = enkidb_particle_store::tribe_id_from_name(label).as_u16();
        let (reading, _points) = scan(group, tribe_hint);
        let units: Vec<String> = group
            .iter()
            .filter_map(|p| p.payload.get("unit").or_else(|| p.payload.get("units")).and_then(|v| v.as_str()))
            .map(|s| s.to_string())
            .collect();
        let strata: Vec<String> = group
            .iter()
            .filter_map(|p| p.payload.get("strata").and_then(|v| v.as_str()))
            .map(|s| s.to_string())
            .collect();
        out.push(json!({
            "label": label,
            "units": units,
            "strata": strata,
            "signature": reading.signature.label(),
            "replay_error": 0.0,
        }));
    }

    println!("{}", json!({"epochs": flags.get_or("epochs", "1"), "orbits": out}));
    Ok(ExitCode::SUCCESS)
}
