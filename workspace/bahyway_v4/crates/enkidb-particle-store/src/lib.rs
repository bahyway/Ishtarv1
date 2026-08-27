//! enkidb-particle-store — the real local substrate behind `bahyway-enkidb`
//! and `bahyway-lamassu`.
//!
//! Closes a gap found triaging the PB-609/PB-672 unattended manifest walks
//! (`docs/13_changelog/WED0826_MANIFEST_TRIAGE_2026-08-26.md`): 26
//! playbooks across this repo invoke `bahyway-enkidb`/`bahyway-lamassu` as
//! already-installed CLIs, but no crate anywhere built either binary --
//! the real engine crates (`enkidb-kaki`, `bahyway-algebra`,
//! `lamassu-engine`) existed with no CLI wrapping them.
//!
//! This is deliberately NOT a client for the real length-prefixed TCP wire
//! protocol `enkiddb-write-server`/`enkiddb-read-server` speak (see
//! `enkiddb-rag-client`) -- that protocol is EnkiDDB-specific (RAG
//! ingest/search) and nothing about the 26 callers' `present`/`orbit`/
//! `prove`/`shape` surface matches it. Every one of the seven EnkiDB
//! Types this CLI addresses (`--db EnkiSDB/EnkiODB/EnkiQDB/EnkiDB/EnkiDW/
//! EnkiMDB/EnkiDDB --port 7001..7007`) is real only as a set of crates
//! (`enkisdb`, `enkiodb`, ...) and a `--port` number in these playbooks --
//! none of them currently runs as a live network service anywhere in this
//! repo. So this store is honest about what it is: a real, KAKI-minting,
//! append-only local substrate at `~/bahyway/enkidb/<db>-<port>/`, keyed
//! by the same `--db`/`--port` pair every playbook already passes. It
//! makes `orbit`/`present`/`prove` genuinely stateful and inspectable
//! today, on this host, without inventing a network protocol nobody has
//! asked this session to build. `--port` is kept as part of the storage
//! key (not ignored) so that a future real per-port network service can
//! adopt the identical on-disk shape without a migration.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use bahyway_core::TribeId;
use enkidb_kaki::{Kaki, KakiMinter, KakiRole};
use serde::{Deserialize, Serialize};

/// `~/bahyway/enkidb` -- sibling of every other real on-host state
/// directory this ecosystem already writes under `~/bahyway/...`
/// (`sala/court_journal.log`, `steward/decrees.json`, `segment/`, ...).
pub fn store_root() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join("bahyway").join("enkidb")
}

/// One `--db`/`--port` pair's own directory, e.g. `EnkiODB-7002/`.
pub fn db_dir(db: &str, port: u16) -> PathBuf {
    store_root().join(format!("{db}-{port}"))
}

fn particles_path(dir: &Path) -> PathBuf {
    dir.join("particles.json")
}

fn policy_path(dir: &Path) -> PathBuf {
    dir.join("policy.json")
}

/// One real, minted particle. `kaki_hex` is `Kaki::bytes()` rendered as
/// plain lowercase hex (no separators) -- deliberately not `Kaki`'s own
/// `Display` (which groups with hyphens): several callers (e.g.
/// `playbook_677_template_registry.yml`'s `KAKI\s+([0-9a-f·]+)` regex)
/// expect an unbroken hex run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleRecord {
    pub kaki_hex: String,
    pub kaki_type: String,
    pub kaki_role: String,
    pub tribe: String,
    pub tribe_id: u16,
    #[serde(default)]
    pub witnesses: Vec<String>,
    #[serde(default)]
    pub clause: String,
    #[serde(default)]
    pub sealed: bool,
    #[serde(default)]
    pub minted_at_unix: u64,
    /// Whatever structured fields the caller supplied (from a `--from`
    /// file, or the flags themselves) -- kept verbatim so `present
    /// --json`/`--schema` and `prove`'s rules have real data to read.
    #[serde(default)]
    pub payload: serde_json::Value,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ParticleStore {
    #[serde(default)]
    pub particles: Vec<ParticleRecord>,
}

impl ParticleStore {
    pub fn load(dir: &Path) -> io::Result<Self> {
        let p = particles_path(dir);
        if !p.exists() {
            return Ok(Self::default());
        }
        let raw = fs::read_to_string(&p)?;
        Ok(serde_json::from_str(&raw).unwrap_or_default())
    }

    pub fn save(&self, dir: &Path) -> io::Result<()> {
        fs::create_dir_all(dir)?;
        let raw = serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string());
        fs::write(particles_path(dir), raw)
    }
}

/// A real, deterministic `TribeId` for a symbolic tribe name (e.g.
/// `"mdb.template"`, `"pipe.reach"`) -- CRC-16/CCITT over the UTF-8
/// bytes, same primitive every KAKI checksum in this ecosystem already
/// uses (`bahyway_crc::crc16`). Deterministic on purpose: the same tribe
/// name must always resolve to the same `TribeId` across separate CLI
/// invocations for `present`/`prove` to see a consistent tribe.
pub fn tribe_id_from_name(name: &str) -> TribeId {
    TribeId::from_u16(bahyway_crc::crc16(name.as_bytes()))
}

pub fn kaki_hex(k: &Kaki) -> String {
    k.bytes().iter().map(|b| format!("{b:02x}")).collect()
}

pub fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Mint one real Identity-KAKI for `tribe` via the real `KakiMinter`
/// gate (no fabricated bytes, no bypass of `Kaki::mint`'s checksum) and
/// wrap it as a `ParticleRecord` ready to append to a store.
pub fn mint_particle(
    tribe: &str,
    kaki_type_label: &str,
    role: KakiRole,
    witnesses: Vec<String>,
    clause: String,
    sealed: bool,
    payload: serde_json::Value,
) -> ParticleRecord {
    let tribe_id = tribe_id_from_name(tribe);
    let minter = KakiMinter::new(tribe_id);
    let kaki = minter.identity(role);
    ParticleRecord {
        kaki_hex: kaki_hex(&kaki),
        kaki_type: kaki_type_label.to_string(),
        kaki_role: role.as_str().to_string(),
        tribe: tribe.to_string(),
        tribe_id: tribe_id.as_u16(),
        witnesses,
        clause,
        sealed,
        minted_at_unix: now_unix(),
        payload,
    }
}

/// Mint one real Event-KAKI for `tribe` (kaki_type=Event) -- used by
/// `decree` for the "Event KAKI per executed decree" receipt.
pub fn mint_event_particle(
    tribe: &str,
    role: KakiRole,
    payload: serde_json::Value,
) -> ParticleRecord {
    let tribe_id = tribe_id_from_name(tribe);
    let minter = KakiMinter::new(tribe_id);
    let kaki = minter.event(role);
    ParticleRecord {
        kaki_hex: kaki_hex(&kaki),
        kaki_type: "Event".to_string(),
        kaki_role: role.as_str().to_string(),
        tribe: tribe.to_string(),
        tribe_id: tribe_id.as_u16(),
        witnesses: Vec::new(),
        clause: String::new(),
        sealed: false,
        minted_at_unix: now_unix(),
        payload,
    }
}

/// List every `<db>-<port>` directory that has ever been orbited into,
/// across every db name -- used by `bahyway-enkidb trace` (no `--db`
/// filter, only `--tribe`) and `bahyway-lamassu orbits` (federation-wide
/// scan across every known tribe).
pub fn all_db_dirs() -> Vec<PathBuf> {
    let root = store_root();
    let Ok(entries) = fs::read_dir(&root) else {
        return Vec::new();
    };
    entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect()
}

/// Read a `--from <file>` ontology tablet (YAML or JSON, either is fine
/// -- this ecosystem's own tablets are almost all `.yml`) as generic
/// JSON, so callers can walk it uniformly regardless of source format.
pub fn read_from_file(path: &Path) -> io::Result<serde_json::Value> {
    let raw = fs::read_to_string(path)?;
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
        return Ok(v);
    }
    serde_yaml::from_str::<serde_json::Value>(&raw)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
}

/// Flatten a tablet's real records into a list of `(record_json,
/// approximate_kaki_type_hint)` pairs. A "record" is any JSON object
/// found anywhere in the tree that carries an `id`, `symbol`, or `name`
/// field -- covers every real tablet shape this ecosystem's playbooks
/// pass (`units: [ {symbol: PU, ...}, ... ]`, `structural: [...]` +
/// `material: { human: [...], robot: [...] }`, a flat list, or a single
/// object). No fabricated schema is imposed beyond "it looks like a
/// record" -- unrecognised shapes simply contribute nothing rather than
/// a made-up entry.
pub fn flatten_records(v: &serde_json::Value) -> Vec<serde_json::Value> {
    let mut out = Vec::new();
    walk_records(v, &mut out);
    out
}

fn walk_records(v: &serde_json::Value, out: &mut Vec<serde_json::Value>) {
    match v {
        serde_json::Value::Object(map) => {
            let looks_like_record =
                map.contains_key("id") || map.contains_key("symbol") || map.contains_key("name");
            if looks_like_record {
                out.push(v.clone());
            }
            for value in map.values() {
                walk_records(value, out);
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                walk_records(item, out);
            }
        }
        _ => {}
    }
}

/// Real policy round-trip for `segment-policy --report`: load whatever
/// was previously stored for this db (if anything), compare, save the
/// new one, and say honestly whether it actually changed.
pub fn apply_policy(dir: &Path, new_policy: &serde_json::Value) -> io::Result<bool> {
    let p = policy_path(dir);
    let prior: Option<serde_json::Value> = if p.exists() {
        fs::read_to_string(&p)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
    } else {
        None
    };
    let changed = prior.as_ref() != Some(new_policy);
    fs::create_dir_all(dir)?;
    fs::write(&p, serde_json::to_string_pretty(new_policy).unwrap())?;
    Ok(changed)
}
