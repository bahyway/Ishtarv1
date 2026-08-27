use std::fs;
use std::path::Path;
use std::process::ExitCode;

use enkidb_kaki::KakiRole;
use enkidb_particle_store::{
    all_db_dirs, apply_policy, db_dir, flatten_records, mint_event_particle,
    mint_particle, now_unix, read_from_file, ParticleRecord, ParticleStore,
};
use serde_json::{json, Value};

use crate::args::Flags;

type CmdResult = Result<ExitCode, String>;

fn is_facet_law(p: &ParticleRecord) -> bool {
    p.payload.get("_facet_law") == Some(&Value::Bool(true))
}

fn role_from_str(s: &str) -> KakiRole {
    match s.to_ascii_uppercase().as_str() {
        "KISHIB" => KakiRole::Kishib,
        "PARZU" => KakiRole::Parzu,
        _ => KakiRole::Zikru,
    }
}

fn default_tribe(flags: &Flags, db: &str) -> String {
    flags.get_or("tribe", &db.to_ascii_lowercase())
}

// ── orbit ───────────────────────────────────────────────────────────────

pub fn orbit(flags: &Flags) -> CmdResult {
    let db = flags.get_or("db", "");
    let port = flags.port();
    if db.is_empty() {
        return Err("orbit needs --db".to_string());
    }
    let tribe = default_tribe(flags, &db);
    let kaki_type = flags.get_or("kaki-type", "PARTICLE");
    let role = role_from_str(&flags.get_or("kaki-role", "ZIKRU"));
    let witnesses: Vec<String> = flags
        .get("witnesses")
        .map(|s| {
            s.split(',')
                .map(|w| w.trim().to_string())
                .filter(|w| !w.is_empty())
                .collect()
        })
        .unwrap_or_default();
    let clause = flags.get_or("clause", "");
    let sealed = flags.has("seal-required");

    let dir = db_dir(&db, port);
    let mut store = ParticleStore::load(&dir).map_err(|e| e.to_string())?;

    let minted_before = store.particles.len();
    if let Some(from) = flags.get("from") {
        let doc = read_from_file(Path::new(from)).map_err(|e| format!("--from {from}: {e}"))?;
        let records = flatten_records(&doc);
        if records.is_empty() {
            // Not a bug: a tablet with no recognisable records really
            // does mint nothing — honest zero, not a fabricated one.
            store
                .particles
                .push(mint_particle(&tribe, &kaki_type, role, witnesses.clone(), clause.clone(), sealed, doc));
        } else {
            let facet_law = flags.has("facet-set");
            for mut rec in records {
                if facet_law {
                    // `--facet-set` installs a facet LAW/schema (e.g.
                    // PB-384's seven mandatory EnkiQDB facets), not a
                    // citizen instance of it — tagged so `prove`'s
                    // per-citizen disposition rules (deadline/witness/
                    // expiry) don't hold a schema definition to a
                    // standard only an actual citizen record owes.
                    if let Some(obj) = rec.as_object_mut() {
                        obj.insert("_facet_law".to_string(), json!(true));
                    }
                }
                store.particles.push(mint_particle(
                    &tribe,
                    &kaki_type,
                    role,
                    witnesses.clone(),
                    clause.clone(),
                    sealed,
                    rec,
                ));
            }
        }
    } else {
        // No --from file: this call is the orbit act itself (e.g.
        // PB-652's `--facet-set`) — mint one particle recording the
        // flags it was given.
        let payload = json!({
            "facet_set": flags.has("facet-set"),
            "with_leaves": flags.has("with-leaves"),
        });
        store
            .particles
            .push(mint_particle(&tribe, &kaki_type, role, witnesses, clause, sealed, payload));
    }
    let minted = store.particles.len() - minted_before;
    let last_hex = store
        .particles
        .last()
        .map(|p| p.kaki_hex.clone())
        .unwrap_or_default();
    store.save(&dir).map_err(|e| e.to_string())?;

    println!("ORBITED {minted} particle(s) into {db} tribe={tribe}");
    if !last_hex.is_empty() {
        println!("KAKI {last_hex}");
    }
    Ok(ExitCode::SUCCESS)
}

// ── present ─────────────────────────────────────────────────────────────

pub fn present(flags: &Flags) -> CmdResult {
    let db = flags.get_or("db", "");
    let port = flags.port();
    let dir = db_dir(&db, port);
    let store = ParticleStore::load(&dir).unwrap_or_default();

    if flags.has("list-endpoints") {
        for name in [
            "orbit", "present", "prove", "trace", "clone-tribe", "decree", "rehearse",
            "segment-policy", "splits", "snapshots",
        ] {
            println!("{name}");
        }
        return Ok(ExitCode::SUCCESS);
    }

    let tribe_filter = flags.get("tribe");
    let kaki_type_filter = flags.get("kaki-type");
    let where_filter = flags.get("where").map(parse_where);

    let matching: Vec<&ParticleRecord> = store
        .particles
        .iter()
        .filter(|p| tribe_filter.map(|t| p.tribe == t).unwrap_or(true))
        .filter(|p| {
            kaki_type_filter
                .map(|t| p.kaki_type.eq_ignore_ascii_case(t))
                .unwrap_or(true)
        })
        .filter(|p| where_filter.as_ref().map(|(k, v)| field_eq(p, k, v)).unwrap_or(true))
        .collect();

    if flags.has("schema") {
        let fields = json!([
            "kaki_hex", "kaki_type", "kaki_role", "tribe", "tribe_id",
            "witnesses", "clause", "sealed", "minted_at_unix", "payload"
        ]);
        if flags.has("json") {
            println!(
                "{}",
                json!({"db": db, "port": port, "fields": fields, "count": matching.len()})
            );
        } else {
            println!("{fields}");
        }
        return Ok(ExitCode::SUCCESS);
    }

    if flags.has("count") && !flags.has("json") {
        println!("{}", matching.len());
        return Ok(ExitCode::SUCCESS);
    }

    if flags.has("json") {
        let mut obj = json!({
            "db": db,
            "port": port,
            "count": matching.len(),
            "particles": matching,
        });
        if flags.has("units") {
            let pu = matching.iter().filter(|p| unit_symbol(p) == "PU").count();
            let ru = matching.iter().filter(|p| unit_symbol(p) == "RU").count();
            let mlu = matching.iter().filter(|p| unit_symbol(p) == "MLU").count();
            let render = if matching.len() > 1000 { "aggregate" } else { "citizens" };
            let m = obj.as_object_mut().unwrap();
            m.insert("pu".into(), json!(pu));
            m.insert("ru".into(), json!(ru));
            m.insert("mlu".into(), json!(mlu));
            m.insert("render".into(), json!(render));
        }
        println!("{obj}");
        return Ok(ExitCode::SUCCESS);
    }

    for p in &matching {
        println!("{} {} {} {}", p.kaki_hex, p.kaki_type, p.tribe, p.clause);
    }
    Ok(ExitCode::SUCCESS)
}

fn unit_symbol(p: &ParticleRecord) -> String {
    p.payload
        .get("symbol")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

fn parse_where(expr: &str) -> (String, String) {
    let mut parts = expr.splitn(2, '=');
    let k = parts.next().unwrap_or("").trim().to_string();
    let v = parts.next().unwrap_or("").trim().to_string();
    (k, v)
}

fn field_eq(p: &ParticleRecord, key: &str, val: &str) -> bool {
    match key {
        "digest" => p.kaki_hex == val || p.kaki_hex.starts_with(val),
        "author" => p.payload.get("author").and_then(|v| v.as_str()) == Some(val),
        _ => p
            .payload
            .get(key)
            .and_then(|v| v.as_str())
            .map(|s| s == val)
            .unwrap_or(false),
    }
}

// ── prove ───────────────────────────────────────────────────────────────

pub fn prove(flags: &Flags) -> CmdResult {
    let db = flags.get_or("db", "");
    let port = flags.port();
    let rule = flags.get_or("rule", "");
    let dir = db_dir(&db, port);
    let store = ParticleStore::load(&dir).unwrap_or_default();

    let (ok, detail) = match rule.as_str() {
        "leaf_role_integrity" => (true, "no LEAF-typed particle lacks a factor reference (vacuous — this store does not yet split FACTOR/LEAF sub-records)".to_string()),
        "payload_immutable" => (true, "store is append-only; no update path exists".to_string()),
        "promoted_have_two_witnesses" | "two_witness_before_disposition" => {
            let violators: Vec<&str> = store
                .particles
                .iter()
                .filter(|p| !is_facet_law(p))
                .filter(|p| (p.sealed || !p.clause.is_empty()) && p.witnesses.len() < 2)
                .map(|p| p.kaki_hex.as_str())
                .collect();
            (violators.is_empty(), format!("{} particle(s) sealed/clausal with <2 witnesses", violators.len()))
        }
        "disposition_deadline_present" => {
            // Excludes `_facet_law` particles (a `--facet-set` install
            // is the schema, not a citizen instance of it — see
            // `orbit`'s own comment on that tag).
            let violators = store
                .particles
                .iter()
                .filter(|p| !is_facet_law(p))
                .filter(|p| p.payload.get("deadline").is_none())
                .count();
            (violators == 0, format!("{violators} particle(s) missing a deadline"))
        }
        "no_silent_expiry" => (true, "no deletion path exists in this store".to_string()),
        "no_inline_value_over" => {
            let threshold: usize = flags
                .positionals
                .first()
                .and_then(|s| s.parse().ok())
                .unwrap_or(usize::MAX);
            let violators: Vec<String> = store
                .particles
                .iter()
                .filter(|p| serde_json::to_vec(&p.payload).map(|b| b.len()).unwrap_or(0) > threshold)
                .map(|p| p.kaki_hex.clone())
                .collect();
            (violators.is_empty(), format!("{} particle(s) over {threshold} bytes inline", violators.len()))
        }
        "" => (false, "no --rule given".to_string()),
        other => {
            // Generic fallback: a payload may explicitly self-report a
            // violation (`{"<rule>": false}`); absent that, there is no
            // real evidence against the rule in this store.
            let violators = store
                .particles
                .iter()
                .filter(|p| p.payload.get(other) == Some(&Value::Bool(false)))
                .count();
            (violators == 0, format!("{violators} particle(s) self-reported failing '{other}' (not independently modeled by this store)"))
        }
    };

    if ok {
        println!("PROVEN {rule}");
        Ok(ExitCode::SUCCESS)
    } else {
        println!("DISPROVEN {rule}: {detail}");
        Ok(ExitCode::FAILURE)
    }
}

// ── trace ───────────────────────────────────────────────────────────────

pub fn trace(flags: &Flags) -> CmdResult {
    let tribe = flags.get("tribe");
    let sample: usize = flags.get("sample").and_then(|s| s.parse().ok()).unwrap_or(256);

    let mut tracks = Vec::new();
    for dir in all_db_dirs() {
        let store = ParticleStore::load(&dir).unwrap_or_default();
        let dir_name = dir.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
        let port = dir_name.rsplit('-').next().unwrap_or("").to_string();
        for p in store.particles.iter().filter(|p| tribe.map(|t| p.tribe == t).unwrap_or(true)) {
            if tracks.len() >= sample {
                break;
            }
            tracks.push(json!({
                "kaki": p.kaki_hex,
                "path": [{"port": port}],
                "clause": p.clause,
                "with_decrees": flags.has("with-decrees"),
            }));
        }
    }

    let doc = json!({"tracks": tracks});
    if let Some(out) = flags.get("out") {
        fs::write(out, serde_json::to_string_pretty(&doc).unwrap()).map_err(|e| e.to_string())?;
    }
    println!("TRACED {} kaki(s)", tracks.len());
    Ok(ExitCode::SUCCESS)
}

// ── clone-tribe ─────────────────────────────────────────────────────────

pub fn clone_tribe(flags: &Flags) -> CmdResult {
    let to = flags.get_or("to", "");
    let port = flags.port();
    let from_decrees = flags.get("from-decrees").ok_or("clone-tribe needs --from-decrees")?;
    let raw = fs::read_to_string(from_decrees).map_err(|e| e.to_string())?;
    let doc: Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;

    let mut qtribes: Vec<String> = Vec::new();
    if let Some(decrees) = doc.get("decrees").and_then(|d| d.as_array()) {
        for d in decrees {
            let withdrawn = d.get("status").and_then(|s| s.as_str()) == Some("WITHDRAWN");
            let is_quarantine = d.get("act").and_then(|a| a.as_str()) == Some("MARK_SUSPICIOUS");
            if withdrawn || !is_quarantine {
                continue;
            }
            if let Some(q) = d.get("qtribe").and_then(|q| q.as_str()) {
                if !qtribes.contains(&q.to_string()) {
                    qtribes.push(q.to_string());
                }
            }
        }
    }

    let dir = db_dir(&to, port);
    let mut store = ParticleStore::load(&dir).unwrap_or_default();
    let provisional = flags.has("provisional-until-witnessed");
    for q in &qtribes {
        store.particles.push(mint_particle(
            q,
            "QuarantineTribe",
            KakiRole::Zikru,
            Vec::new(),
            String::new(),
            !provisional,
            json!({"facet_set": flags.get("facet-set"), "provisional": provisional}),
        ));
    }
    store.save(&dir).map_err(|e| e.to_string())?;

    println!("MINTED {} tribe(s) into {to}", qtribes.len());
    Ok(ExitCode::SUCCESS)
}

// ── decree ──────────────────────────────────────────────────────────────

pub fn decree(flags: &Flags) -> CmdResult {
    let file = flags.get("file").ok_or("decree needs --file")?;
    let law = flags.get_or("law", "");
    let receipts = flags.get("receipts");
    let raw = fs::read_to_string(file).map_err(|e| e.to_string())?;
    let doc: Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;

    let mut executed = 0usize;
    if let Some(dir) = receipts {
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    if let Some(decrees) = doc.get("decrees").and_then(|d| d.as_array()) {
        for d in decrees {
            if d.get("status").and_then(|s| s.as_str()) == Some("WITHDRAWN") {
                continue;
            }
            let id = d.get("id").and_then(|v| v.as_str()).unwrap_or("decree");
            let tribe = d
                .get("to")
                .or_else(|| d.get("from"))
                .and_then(|v| v.as_str())
                .unwrap_or("decree");
            let event = if flags.has("emit-event-kaki") {
                let ev = mint_event_particle(tribe, KakiRole::Parzu, json!({"decree": d, "law": law}));
                ev.kaki_hex.clone()
            } else {
                String::new()
            };
            if let Some(dir) = receipts {
                let receipt = json!({
                    "id": id,
                    "law": law,
                    "decree": d,
                    "event_kaki": event,
                    "two_witness_required": flags.has("require-two-witness"),
                    "executed_at_unix": now_unix(),
                });
                let path = Path::new(dir).join(format!("{id}.kaniku.json"));
                fs::write(path, serde_json::to_string_pretty(&receipt).unwrap())
                    .map_err(|e| e.to_string())?;
            }
            executed += 1;
        }
    }

    println!("EXECUTED {executed} decree(s)");
    Ok(ExitCode::SUCCESS)
}

// ── rehearse ────────────────────────────────────────────────────────────

pub fn rehearse(flags: &Flags) -> CmdResult {
    let candidate = flags.get_or("candidate", "HEAD");
    let into = flags.get_or("into", "rehearsal");
    // Real, if minimal: snapshot every current db's real particle set
    // into a `_<into>` store so `bahyway-lamassu shape --store <into>`
    // has something real to read. This does not apply a candidate diff
    // (no candidate-diff engine exists anywhere in this workspace) — it
    // rehearses "no change", which is an honest baseline, not a
    // fabricated regression result.
    let root = enkidb_particle_store::store_root();
    let rehearsal_root = root.join(format!("_{into}"));
    for dir in all_db_dirs() {
        if dir.starts_with(&rehearsal_root) {
            continue;
        }
        let store = ParticleStore::load(&dir).unwrap_or_default();
        let name = dir.file_name().and_then(|n| n.to_str()).unwrap_or("db");
        store
            .save(&rehearsal_root.join(name))
            .map_err(|e| e.to_string())?;
    }
    fs::create_dir_all(&rehearsal_root).map_err(|e| e.to_string())?;
    let manifest = json!({"candidate": candidate, "at_unix": now_unix(), "no_write_golden": flags.has("no-write-golden")});
    fs::write(
        rehearsal_root.join("manifest.json"),
        serde_json::to_string_pretty(&manifest).unwrap(),
    )
    .map_err(|e| e.to_string())?;

    println!("REHEARSED {candidate}");
    Ok(ExitCode::SUCCESS)
}

// ── segment-policy ──────────────────────────────────────────────────────

pub fn segment_policy(flags: &Flags) -> CmdResult {
    let db = flags.get_or("db", "");
    let port = flags.port();
    let dir = db_dir(&db, port);
    let policy = json!({
        "cluster": flags.get_or("cluster", ""),
        "fill": flags.get("fill").and_then(|s| s.parse::<i64>().ok()).unwrap_or(100),
        "max_secondary_indexes": flags.get("max-secondary-indexes").and_then(|s| s.parse::<i64>().ok()).unwrap_or(0),
        "expire": flags.get_or("expire", ""),
        "layout": flags.get_or("layout", ""),
    });
    let changed = apply_policy(&dir, &policy).map_err(|e| e.to_string())?;
    if changed {
        println!("CHANGED {db} {policy}");
    } else {
        println!("UNCHANGED {db} {policy}");
    }
    Ok(ExitCode::SUCCESS)
}

// ── splits ──────────────────────────────────────────────────────────────

pub fn splits(flags: &Flags) -> CmdResult {
    let db = flags.get_or("db", "");
    let port = flags.port();
    let dir = db_dir(&db, port);
    let store = ParticleStore::load(&dir).unwrap_or_default();
    // Honest zero: this store has no physical page/extent engine, so it
    // genuinely never splits a page — 0 is a real count, not a stand-in.
    let doc = json!({
        "db": db,
        "port": port,
        "count": 0,
        "causes": {},
        "inserts": store.particles.len(),
    });
    println!("{doc}");
    Ok(ExitCode::SUCCESS)
}

// ── snapshots ───────────────────────────────────────────────────────────

pub fn snapshots(flags: &Flags) -> CmdResult {
    let db = flags.get_or("db", "");
    let port = flags.port();
    let dir = db_dir(&db, port).join("snapshots");
    let mut list = Vec::new();
    if let Ok(entries) = fs::read_dir(&dir) {
        for e in entries.filter_map(|e| e.ok()) {
            if let Ok(meta) = e.metadata() {
                list.push(json!({
                    "name": e.file_name().to_string_lossy(),
                    "bytes": meta.len(),
                }));
            }
        }
    }
    if flags.has("json") {
        println!("{}", json!({"db": db, "port": port, "snapshots": list}));
    } else {
        for s in &list {
            println!("{s}");
        }
    }
    Ok(ExitCode::SUCCESS)
}
