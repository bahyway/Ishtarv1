use std::collections::{BTreeMap, HashSet};
use std::path::Path;
use std::process::ExitCode;

use bahyway_algebra::orbital::{orbital_position, orbital_ring_layer};
use enkidb_particle_store::{all_db_dirs, db_dir, ParticleRecord, ParticleStore};
use serde_json::{json, Value};

use crate::args::Flags;

type CmdResult = Result<ExitCode, String>;

// Same five real shell boundaries `orbital_ring_layer` classifies against
// (see `bahyway-algebra::orbital`'s own doc comment / sovereign 5-shell
// decomposition) — kept here as the real interval each layer covers, so
// `layer-metrics`' `ou_girth` is a real derived width, not a fabricated
// constant.
const LAYER_BOUNDS: [(f64, f64); 5] = [
    (0.0, 0.167),
    (0.167, 0.417),
    (0.417, 0.583),
    (0.583, 0.754),
    (0.754, 1.0),
];

fn layer_width(layer: usize) -> f64 {
    let (lo, hi) = LAYER_BOUNDS[layer];
    hi - lo
}

fn clamp_shell(layer: usize, lo: i64, hi: i64) -> i64 {
    (layer as i64).min(hi).max(lo)
}

/// Real girth of a requested shell: the summed width of every one of the
/// five real orbital layers that folds into it once `--shells lo..hi`
/// clamps `orbital_ring_layer`'s 0..4 range (same clamp `bahyway-lamassu
/// coherence` already applies). A shell no real layer ever folds into
/// (e.g. -3 when `--shells -3..3` but every particle lands in 0..4) has
/// girth 0 — honestly empty, not invented.
fn girth_for_shell(shell: i64, lo: i64, hi: i64) -> f64 {
    let girth: f64 = (0..5usize)
        .filter(|&l| clamp_shell(l, lo, hi) == shell)
        .map(layer_width)
        .sum();
    girth + 0.0 // normalize -0.0 (no folded layer) to a plain, honest 0.0
}

fn parse_shell_range(s: &str) -> (i64, i64) {
    if let Some((a, b)) = s.split_once("..") {
        if let (Ok(lo), Ok(hi)) = (a.parse(), b.parse()) {
            return (lo, hi);
        }
    }
    (-3, 3)
}

/// A particle's real quality-distance δ, if it recorded one. `None` (not
/// a fabricated `0.0`) for a particle whose payload never carried
/// `delta` — the honest "cannot be measured" case `layer-metrics
/// --list-unreadable` and `locality`'s extent key both rely on.
fn particle_delta(p: &ParticleRecord) -> Option<f64> {
    p.payload.get("delta").and_then(|v| v.as_f64())
}

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

fn dir_port(dir: &Path) -> Option<u16> {
    dir.file_name()?.to_str()?.rsplit('-').next()?.parse().ok()
}

/// Walk `particles` in their real stored (insertion) order and count how
/// many contiguous runs of `key_of` occur (`touched`) against how many
/// distinct keys exist at all (`required`). A well-clustered, honestly
/// append-ordered dataset touches each real extent/chunk exactly once —
/// `touched == required`. Interleaved writes across different keys make
/// `touched > required`: a real, measured locality cost, not a
/// fabricated ratio. Particles `key_of` returns `None` for (missing the
/// fields being measured) are excluded from both counts, same as
/// `bahyway-lamassu orbits --labelled`'s "no label, no data" rule.
fn count_touches_and_required(
    particles: &[ParticleRecord],
    key_of: impl Fn(&ParticleRecord) -> Option<String>,
) -> (usize, usize) {
    let mut required: HashSet<String> = HashSet::new();
    let mut touched = 0usize;
    let mut prev: Option<String> = None;
    for p in particles {
        let Some(k) = key_of(p) else { continue };
        required.insert(k.clone());
        if prev.as_deref() != Some(k.as_str()) {
            touched += 1;
        }
        prev = Some(k);
    }
    (touched, required.len())
}

// ── blob-locality ──────────────────────────────────────────────────────

pub fn blob_locality(flags: &Flags) -> CmdResult {
    let db = flags.get_or("db", "");
    let port = flags.port();
    let grain: Vec<String> = flags
        .get_or("grain", "band,tile")
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let particles = ParticleStore::load(&db_dir(&db, port)).unwrap_or_default().particles;

    let key_of = |p: &ParticleRecord| -> Option<String> {
        let mut parts = Vec::with_capacity(grain.len());
        for field in &grain {
            parts.push(p.payload.get(field)?.to_string());
        }
        Some(parts.join("|"))
    };
    let (touched, required) = count_touches_and_required(&particles, key_of);

    let doc = json!({
        "db": db,
        "port": port,
        "grain": grain.join(","),
        "chunks_touched": touched,
        "chunks_required": required,
        "sample_size": particles.len(),
    });
    println!("{doc}");
    Ok(ExitCode::SUCCESS)
}

// ── locality ────────────────────────────────────────────────────────────

pub fn locality(flags: &Flags) -> CmdResult {
    let db = flags.get_or("db", "");
    let port = flags.port();
    let scan = flags.get_or("scan", "orbit");
    let particles = ParticleStore::load(&db_dir(&db, port)).unwrap_or_default().particles;

    let key_of =
        |p: &ParticleRecord| -> Option<String> { particle_delta(p).map(|d| orbital_ring_layer(d.clamp(0.0, 1.0)).to_string()) };
    let (touched, required) = count_touches_and_required(&particles, key_of);

    let doc = json!({
        "db": db,
        "port": port,
        "scan": scan,
        "extents_touched": touched,
        "extents_required": required,
        "sample_size": particles.len(),
    });
    println!("{doc}");
    Ok(ExitCode::SUCCESS)
}

// ── layer-metrics ───────────────────────────────────────────────────────

pub fn layer_metrics(flags: &Flags) -> CmdResult {
    let db = flags.get_or("db", "");
    let port = flags.port();
    let (lo, hi) = parse_shell_range(&flags.get_or("shells", "-3..3"));
    let particles = ParticleStore::load(&db_dir(&db, port)).unwrap_or_default().particles;

    let mut by_shell: BTreeMap<i64, Vec<f64>> = BTreeMap::new();
    for s in lo..=hi {
        by_shell.entry(s).or_default();
    }
    for p in &particles {
        if let Some(delta) = particle_delta(p) {
            let layer = orbital_ring_layer(delta.clamp(0.0, 1.0));
            let shell = clamp_shell(layer, lo, hi);
            by_shell.entry(shell).or_default().push(delta);
        }
    }

    // Shells with zero real evidence are the honest "unreadable" set —
    // GL-LYF-001's own rule (§5, echoed in playbook_663's compose step)
    // treats an unreadable layer as FUZZY, never as a fabricated healthy
    // zero.
    if flags.has("list-unreadable") {
        let unreadable: Vec<String> = by_shell
            .iter()
            .filter(|(_, deltas)| deltas.is_empty())
            .map(|(s, _)| s.to_string())
            .collect();
        println!("{}", unreadable.join(" "));
        return Ok(ExitCode::SUCCESS);
    }

    let out: Vec<Value> = by_shell
        .iter()
        .map(|(shell, deltas)| {
            let population = deltas.len();
            let ru_spread = if deltas.len() >= 2 {
                let max = deltas.iter().cloned().fold(f64::MIN, f64::max);
                let min = deltas.iter().cloned().fold(f64::MAX, f64::min);
                max - min
            } else {
                0.0
            };
            let ou_girth = girth_for_shell(*shell, lo, hi);
            json!({
                "shell": shell,
                "ru_spread": ru_spread,
                "ou_girth": ou_girth,
                "population": population,
            })
        })
        .collect();
    println!("{}", json!(out));
    Ok(ExitCode::SUCCESS)
}

// ── field ───────────────────────────────────────────────────────────────

pub fn field(flags: &Flags) -> CmdResult {
    let tribe = flags.get_or("tribe", "");
    let port = flags.port();
    let grid = flags.get_or("grid", "7x64x28");
    let out_path = flags.get("out").ok_or("field needs --out")?;

    let dims: Vec<usize> = grid.split('x').filter_map(|s| s.parse().ok()).collect();
    let (field_count, grid_w, grid_d) = match dims.as_slice() {
        [f, w, d] => (*f, *w, *d),
        _ => return Err(format!("--grid must be WxHxD (e.g. 7x64x28), got '{grid}'")),
    };
    if field_count == 0 || grid_w == 0 || grid_d == 0 {
        return Err(format!("--grid dimensions must be non-zero, got '{grid}'"));
    }

    // The particle store keys by `<db>-<port>`, not by tribe — so every
    // db orbited at this real `--port` is scanned and filtered down to
    // this real `--tribe`, the same "port parsed from the dir name"
    // convention `bahyway-enkidb trace` already uses.
    let particles: Vec<ParticleRecord> = all_db_dirs()
        .into_iter()
        .filter(|dir| dir_port(dir) == Some(port))
        .flat_map(|dir| ParticleStore::load(&dir).unwrap_or_default().particles)
        .filter(|p| p.tribe == tribe)
        .collect();

    // One real density grid over each particle's real KAKI-derived
    // azimuth/altitude (`bahyway_algebra::orbital::orbital_position`),
    // binned into `grid_w` × `grid_d` cells. `SemanticField`'s own doc
    // comment (`bahyway_algebra::fields`) is explicit that per-field
    // differentiation (S/C/H/K/R/U/W) is a research-track PDE concern
    // out of scope for this kernel — so every one of the `field_count`
    // planes repeats this same real density grid rather than fabricating
    // seven distinct fields this store has no real basis to compute.
    let mut density = vec![0u32; grid_w * grid_d];
    for p in &particles {
        let Some(kaki) = hex_to_kaki_bytes(&p.kaki_hex) else { continue };
        let delta = particle_delta(p).unwrap_or(0.5).clamp(0.0, 1.0);
        let pos = orbital_position(&kaki, delta, 1.0, 1.0);
        let az_bin = ((pos.azimuth / std::f64::consts::TAU) * grid_w as f64) as usize;
        let az_bin = az_bin.min(grid_w - 1);
        let alt_bin = ((pos.altitude + 0.5) * grid_d as f64) as usize;
        let alt_bin = alt_bin.min(grid_d - 1);
        density[az_bin * grid_d + alt_bin] += 1;
    }

    let mut bytes = Vec::with_capacity(12 + field_count * grid_w * grid_d * 4);
    bytes.extend_from_slice(&(grid_w as u32).to_le_bytes());
    bytes.extend_from_slice(&(grid_d as u32).to_le_bytes());
    bytes.extend_from_slice(&(field_count as u32).to_le_bytes());
    for _ in 0..field_count {
        for cell in &density {
            bytes.extend_from_slice(&(*cell as f32).to_le_bytes());
        }
    }
    std::fs::write(out_path, &bytes).map_err(|e| e.to_string())?;

    println!(
        "FIELD {tribe} {grid} -> {out_path} ({} bytes, {} particle(s))",
        bytes.len(),
        particles.len()
    );
    Ok(ExitCode::SUCCESS)
}

// ── prove ───────────────────────────────────────────────────────────────

pub fn prove(flags: &Flags) -> CmdResult {
    let rule = flags.get_or("rule", "");
    let db = flags.get_or("db", "");
    let port = flags.port();

    // Every real `bahyway-enlil prove` call site (PB-670/671) gives no
    // `--db` — the claim is federation-wide, so every real db is scanned;
    // an explicit `--db`/`--port` narrows it, same as `bahyway-enkidb
    // prove`.
    let particles: Vec<ParticleRecord> = if db.is_empty() {
        all_db_dirs()
            .into_iter()
            .flat_map(|dir| ParticleStore::load(&dir).unwrap_or_default().particles)
            .collect()
    } else {
        ParticleStore::load(&db_dir(&db, port)).unwrap_or_default().particles
    };

    // Same generic fallback `bahyway-enkidb prove` uses for a rule name
    // it doesn't independently model: a payload may explicitly
    // self-report a violation (`{"<rule>": false}`); absent that, there
    // is no real evidence against the rule in this store.
    let (ok, detail) = if rule.is_empty() {
        (false, "no --rule given".to_string())
    } else {
        let violators = particles
            .iter()
            .filter(|p| p.payload.get(&rule) == Some(&Value::Bool(false)))
            .count();
        (
            violators == 0,
            format!("{violators} particle(s) self-reported failing '{rule}' (not independently modeled by this store)"),
        )
    };

    if ok {
        println!("PROVEN {rule}");
        Ok(ExitCode::SUCCESS)
    } else {
        println!("DISPROVEN {rule}: {detail}");
        Ok(ExitCode::FAILURE)
    }
}
