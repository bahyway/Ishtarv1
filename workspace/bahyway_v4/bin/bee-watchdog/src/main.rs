#![forbid(unsafe_code)]
//! bee-watchdog — BeeMDM FileWatchDog daemon.
//!
//! Full ETL station chain per ZIP batch:
//!
//!   shard/                              (LandingZone root — shared Vagrant folder)
//!   ├── *.zip                           ← DROP HERE
//!   ├── profiles/
//!   │   ├── {batch_name}.template.akk   ← AKK template (preferred — compiled from .akk PARTICLE)
//!   │   ├── {batch_name}.dqprofile      ← client SLA rules (nullable, thresholds, dim-tags)
//!   │   └── {batch_name}.schema.ref     ← compare-tribe-schema reference (auto-saved on first arrival)
//!   ├── Processing/
//!   │   └── {batch}_{ts}/               ← extracted entries + .schema (station 2-5 work area)
//!   └── Moved_To/
//!       └── {batch}_{ts}/               ← completed batches (all Golden Records persisted)
//!
//! Processing order per ZIP:
//! ```text
//!   1. LandingZone::poll()         → detect new ZIPs
//!   2. Musarû security gate        → pre-extraction byte scan (malware signatures)
//!   2b. VGCA-Δ block analysis      → binary trajectory, ZIP-bomb detection (severity ≥4 → reject)
//!   3. ZIP extraction              → ZipEngine (DEFLATE + STORE)
//!   4. ProcessingZone::stage()     → write entries + .schema to Processing/{batch}_{ts}/
//!   5. BatchSchema::infer()        → entity_seed, mandatory/optional attrs, tb_ table name
//!   5b. compare-tribe-schema gate  → compare arriving schema vs SLA/DubSar SDM ref;
//!                                    SchemaMismatch → DeadVerdict, batch rejected
//!   5c. Load or auto-generate ClientDqProfile from shard/profiles/{batch}.dqprofile
//!   6. Per record station chain (Adad Gate as sole KAKI issuer):
//!        a. adad-gate              → ArrivalRecord → IdentityKaki + EventKaki + EAV
//!        b. DataStructureStation   → map raw EAV against template
//!        c. data-cleansing         → DAMA-DMBOK DQ.COMPLETENESS / UNIQUENESS / VALIDITY
//!        d. VGCA beam              → required-field template gate on structured EAV
//!        e. client-dq-profile      → EAV + profile → FuzzyDimensions
//!        f. score-engine           → FuzzyDimensions → B11 → State + Quality + ColorRGB
//!        g. Gem/Tribe (B11≥140)    → ParticleState::Golden → PermanentStore + PersistedDb
//!           Active  (B11 100–139)  → ParticleState::Fuzzy  → PersistedDb + StewardStation
//!           Dead    (B11 < 100)    → ParticleState::Dead   → PersistedDb only
//!   7. Batch-completion event      → commit Entity KAKI to journal (file-level event)
//!   8. ProcessingZone::complete()  → move Processing/{batch}_{ts}/ → Moved_To/{batch}_{ts}/
//! ```
//!
//! Client SLA GUI — first step:
//!   Drop a .dqprofile file in shard/profiles/ to configure per-attribute rules:
//!     nullable, fill_threshold, dim (Name/Identity/Date/Geography/General), weight
//!   Without a profile: auto-generated from BatchSchema (mandatory→non-nullable, optional→nullable)
//!
//! Usage:
//!   bee-watchdog --shard <dir> --data-dir <dir> --tribe-id <hex_u16> [--interval-ms <ms>]

use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use adad_gate::{AdadGate, ArrivalRecord, GateResult};
use akk_loader::{load_akk_file, load_pipeline_file};
use alert_engine::alert::{Alert, AlertSeverity, DriftCause};
use bahyway_core::{ParticleState, TribeId};
use client_dq_profile::{compute_dims, parse_profile, ClientDqProfile};
use compare_tribe_schema::{compare_versions, DeadVerdict, FieldMeta, FieldType, SchemaVersion};
use data_cleansing_station::cleanse as dq_cleanse;
use data_steward_station::StewardStation;
use data_structure_station::structure;
use enkidb_engine::EnkiDb;
use enkidb_journal::entry::EavTriple;
use enkidb_kaki::KakiRole;
use enkidb_persist::PersistedDb;
use enkidb_storage::FsyncPolicy;
use enkidw::kaki_generator::{self, RawRecord};
use enkidw::landing_zone::LandingZone;
use enkidw::zip_engine;
use enkidw::{records_from_entry, BatchSchema, ProcessingZone, StagedEntry};
use media_type_detector::{detect as detect_media_type, MediaCategory};
use musaru_security::zip_scan as musaru_scan;
use permanent_storage::PermanentStore;
use score_engine::freshness::FreshnessDecay;
use score_engine::{score as fuzzy_score, tier_to_state, ScoreInput};
use story_engine::projection::{encode_state, ATTR_STATE};
use template_engine::{FieldSpec, Template};
use vgca_validation::{validate as vgca_beam, vgca_delta, BlockFeatureVector, CorruptionClass};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn strip_ext(fname: &str) -> String {
    match fname.rfind('.') {
        Some(p) => fname[..p].to_string(),
        None => fname.to_string(),
    }
}

// ── Config & arg parsing ──────────────────────────────────────────────────────

struct Config {
    shard: PathBuf,
    data_dir: PathBuf,
    tribe_id: u16,
    interval_ms: u64,
}

fn print_usage() {
    eprintln!(
        "Usage: bee-watchdog \
         --shard <dir> \
         --data-dir <dir> \
         --tribe-id <hex_u16> \
         [--interval-ms <ms>]"
    );
}

fn parse_args() -> Config {
    let mut shard = None::<PathBuf>;
    let mut data_dir = None::<PathBuf>;
    let mut tribe_id = None::<u16>;
    let mut interval_ms = 2_000u64;

    let raw: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < raw.len() {
        match raw[i].as_str() {
            "--shard" => {
                i += 1;
                shard = Some(PathBuf::from(&raw[i]));
            }
            "--data-dir" => {
                i += 1;
                data_dir = Some(PathBuf::from(&raw[i]));
            }
            "--tribe-id" => {
                i += 1;
                let s = raw[i].trim_start_matches("0x").trim_start_matches("0X");
                tribe_id =
                    Some(u16::from_str_radix(s, 16).expect("--tribe-id: expected hex e.g. 0x0001"));
            }
            "--interval-ms" => {
                i += 1;
                interval_ms = raw[i].parse().expect("expected integer");
            }
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            other => {
                eprintln!("[bee-watchdog] unknown argument: {other}");
                print_usage();
                std::process::exit(1);
            }
        }
        i += 1;
    }

    Config {
        shard: shard.unwrap_or_else(|| {
            eprintln!("missing --shard");
            std::process::exit(1);
        }),
        data_dir: data_dir.unwrap_or_else(|| {
            eprintln!("missing --data-dir");
            std::process::exit(1);
        }),
        tribe_id: tribe_id.unwrap_or_else(|| {
            eprintln!("missing --tribe-id");
            std::process::exit(1);
        }),
        interval_ms,
    }
}

// ── Batch result ──────────────────────────────────────────────────────────────

struct BatchResult {
    batch_name: String,
    golden_count: usize,
    fuzzy_count: usize,
    dead_count: usize,
}

// ── Core station chain ────────────────────────────────────────────────────────

/// Build a Template from the inferred BatchSchema so that data-structure-station
/// and VGCA beam enforce the actual column layout, not a static placeholder.
///
/// - Mandatory columns  → `required = true`  (VGCA beam flags absences)
/// - Optional columns   → `required = false` (DQ.COMPLETENESS may warn)
/// - "epoch"/"state" meta-columns are excluded — handled before EAV is built.
/// - ATTR_STATE (0x1A4B) is always required; the state column maps to it.
///
/// `FieldSpec.label` is `&'static str` by design (compile-time templates);
/// runtime columns use the placeholder label "col" — `attr_hash` carries identity.
fn build_template_from_schema(schema: &BatchSchema) -> Template {
    let is_meta = |col: &str| {
        matches!(
            col.to_lowercase().as_str(),
            "epoch" | "ep" | "state" | "status"
        )
    };

    let mut specs: Vec<FieldSpec> =
        Vec::with_capacity(schema.mandatory_attrs.len() + schema.optional_attrs.len() + 1);
    specs.push(FieldSpec::new(ATTR_STATE, "state", true));
    for col in &schema.mandatory_attrs {
        if !is_meta(col) {
            specs.push(FieldSpec {
                attr_hash: fnv1a(col),
                label: "col",
                required: true,
            });
        }
    }
    for col in &schema.optional_attrs {
        if !is_meta(col) {
            specs.push(FieldSpec {
                attr_hash: fnv1a(col),
                label: "col",
                required: false,
            });
        }
    }
    // name/desc are static labels only; fields vec carries the real schema constraints.
    Template::default_template("bee_batch", "BeeMDM batch", &specs)
}

/// Run one CSV file through the full station chain using fuzzy scoring.
/// Returns (golden_committed, fuzzy_committed, dead_committed).
/// Each parameter is a distinct required station dependency (records,
/// gate, template, profile, domain, and the three sovereign stores) --
/// not artificial padding, so bundling into a params struct is a real
/// API-design tradeoff rather than a mechanical fix.
#[allow(clippy::too_many_arguments)]
fn run_station_chain(
    records: Vec<RawRecord>,
    gate: &AdadGate,
    template: &Template,
    profile: &ClientDqProfile,
    domain_byte: u8,
    db: &mut EnkiDb,
    pdb: &mut PersistedDb,
    steward: &mut StewardStation,
) -> (usize, usize, usize) {
    let mut golden = 0usize;
    let mut fuzzy = 0usize;
    let mut dead = 0usize;
    let format_layer = profile.fuzzy_format_layer();
    let decay = FreshnessDecay::civil_registry();

    for record in records {
        // ── Adad Gate: sole KAKI issuer — mints IdentityKaki + EventKaki ──
        let mut ge = adad_ingest_record(gate, &record);

        // ── Station A: Data Structure — EAV normalisation ─────────────────
        let raw_pairs: Vec<(u32, Vec<u8>)> = ge
            .eav
            .iter()
            .map(|t| (t.attr_hash, t.value.clone()))
            .collect();
        ge.eav = structure(template, raw_pairs).eav;

        // ── Station A.5: Data Cleansing — DAMA-DMBOK DQ checks ────────────
        let dq_report = dq_cleanse(template, &ge.eav);
        if !dq_report.is_clean() {
            for issue in dq_report.issues() {
                eprintln!(
                    "  [DQ.{}] epoch={} — {}",
                    issue.term_code, ge.epoch, issue.detail
                );
            }
        }

        // ── Station A.6: VGCA beam — required-field template gate ─────────
        let beam = vgca_beam(template, &ge.eav);
        if !beam.is_valid() {
            eprintln!(
                "  [VGCA-beam] epoch={}: {} missing required / {} unknown attrs",
                ge.epoch,
                beam.missing_required.len(),
                beam.unknown_attrs
            );
        }

        // ── Station B: Client DQ Profile → FuzzyDimensions → Score ────────
        let dims = compute_dims(&ge.eav, profile);
        let input = ScoreInput {
            dims,
            format: format_layer,
            domain_byte,
            elapsed_seconds: 0.0,
            decay: decay.clone(),
            orbital_trust_penalty: 0.0,
        };
        let result = fuzzy_score(&input);
        let state = tier_to_state(result.tier);

        // Replace assessment EAV: STATE + QUALITY + COLOR_RGB + FRESHNESS
        ge.eav.retain(|t| t.attr_hash != ATTR_STATE);
        ge.eav.extend(result.to_eav_triples());

        match state {
            ParticleState::Golden => {
                // ── Gem/Tribe → Golden Record ──────────────────────────────
                let mut ps = PermanentStore::new(db);
                let _ = ps.commit(ge.event_kaki, ge.particle, ge.epoch, ge.eav.clone());
                let _ = pdb.register_particle(&ge.particle);
                let _ = pdb.commit(ge.event_kaki, ge.particle, ge.epoch, ge.eav);
                golden += 1;
            }
            ParticleState::Fuzzy => {
                // ── Active → Fuzzy + StewardStation alert ─────────────────
                let alert = Alert::new(
                    ge.particle.uuid_hash(),
                    AlertSeverity::Watch,
                    DriftCause::Unknown,
                    result.hps.value(),
                );
                steward.receive(&alert);
                let _ = pdb.register_particle(&ge.particle);
                let _ = pdb.commit(ge.event_kaki, ge.particle, ge.epoch, ge.eav);
                fuzzy += 1;
            }
            ParticleState::Dead => {
                // ── NonActive → Dead — journal to PersistedDb for traceability ──
                eprintln!(
                    "  [DEAD] particle {:08x}  B11={} — below Dead threshold",
                    ge.particle.uuid_hash(),
                    result.b11
                );
                let alert = Alert::new(
                    ge.particle.uuid_hash(),
                    AlertSeverity::Critical,
                    DriftCause::Unknown,
                    result.hps.value(),
                );
                steward.receive(&alert);
                let _ = pdb.register_particle(&ge.particle);
                let _ = pdb.commit(ge.event_kaki, ge.particle, ge.epoch, ge.eav);
                dead += 1;
            }
        }
    }

    (golden, fuzzy, dead)
}

// ── Pipeline router ───────────────────────────────────────────────────────────

/// Routes archive entries to their schema template based on PIPELINE stage names.
///
/// Matching: entry filename is checked (case-insensitive contains) against each
/// stage name in order.  First match wins.  Unmatched entries use `default`.
struct PipelineRouter {
    stages: Vec<(String, Template)>, // (stage_name_lowercase, template)
    default: Template,
}

impl PipelineRouter {
    fn template_for(&self, entry_name: &str) -> &Template {
        let lower = entry_name.to_lowercase();
        for (stage, tmpl) in &self.stages {
            if lower.contains(stage.as_str()) {
                return tmpl;
            }
        }
        &self.default
    }
}

/// Build a PipelineRouter for a batch:
///   1. Check for `{batch_name}.pipeline.akk` in profiles_dir.
///   2. For each stage, load `{stage}.template.akk` from profiles_dir.
///   3. Default template: load `{batch_name}.template.akk` or auto-infer.
fn build_pipeline_router(
    profiles_dir: &Path,
    batch_name: &str,
    fname: &str,
    schema: &BatchSchema,
) -> PipelineRouter {
    // Default template
    let default = {
        let akk_path = profiles_dir.join(format!("{}.template.akk", batch_name));
        if akk_path.exists() {
            match load_akk_file(&akk_path) {
                Ok(t) => {
                    println!(
                        "[INFO] {fname}: AKK template loaded → {} ({} req / {} opt)",
                        akk_path.file_name().unwrap_or_default().to_string_lossy(),
                        t.required_fields().count(),
                        t.fields.iter().filter(|f| !f.required).count(),
                    );
                    t
                }
                Err(e) => {
                    eprintln!(
                        "[WARN] {fname}: .template.akk load failed ({e}) — using auto-inferred"
                    );
                    build_template_from_schema(schema)
                }
            }
        } else {
            build_template_from_schema(schema)
        }
    };

    // Pipeline stages
    let pipeline_path = profiles_dir.join(format!("{}.pipeline.akk", batch_name));
    if !pipeline_path.exists() {
        return PipelineRouter {
            stages: vec![],
            default,
        };
    }

    let pipeline = match load_pipeline_file(&pipeline_path) {
        Ok(p) => {
            println!(
                "[INFO] {fname}: PIPELINE '{}' ({} stages)",
                p.name,
                p.stages.len()
            );
            p
        }
        Err(e) => {
            eprintln!("[WARN] {fname}: .pipeline.akk load failed ({e})");
            return PipelineRouter {
                stages: vec![],
                default,
            };
        }
    };

    let mut stages = Vec::new();
    for stage_name in &pipeline.stages {
        let tmpl_path = profiles_dir.join(format!("{}.template.akk", stage_name));
        if tmpl_path.exists() {
            match load_akk_file(&tmpl_path) {
                Ok(t) => {
                    println!(
                        "[INFO] {fname}:   stage '{}' → {} fields",
                        stage_name,
                        t.fields.len()
                    );
                    stages.push((stage_name.to_lowercase(), t));
                }
                Err(e) => eprintln!("[WARN] {fname}: stage '{stage_name}' template failed: {e}"),
            }
        } else {
            eprintln!("[WARN] {fname}: stage '{stage_name}' has no .template.akk (skipped)");
        }
    }

    PipelineRouter { stages, default }
}

/// Process one ZIP file through all stations.
///
/// `zip_path` is where the raw bytes are read from (may already be archived
/// out of the shard root); `shard_root` is always the shard directory itself
/// — profiles/, schema refs, and DQ profiles all live there regardless of
/// where the raw ZIP currently sits.
/// Each parameter is a distinct required dependency (paths, tribe, zone,
/// gate, and the three sovereign stores) -- not artificial padding, so
/// bundling into a params struct is a real API-design tradeoff rather
/// than a mechanical fix.
#[allow(clippy::too_many_arguments)]
fn process_zip(
    zip_path: &Path,
    shard_root: &Path,
    fname: &str,
    _tribe_id: TribeId,
    zone: &ProcessingZone,
    gate: &AdadGate,
    db: &mut EnkiDb,
    pdb: &mut PersistedDb,
    steward: &mut StewardStation,
) -> Option<BatchResult> {
    // ── Step 1: Read raw bytes ─────────────────────────────────────────────
    let data = match std::fs::read(zip_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("[ERR!] {fname}: read error — {e}");
            return None;
        }
    };

    // ── Step 2: Musarû security gate ────────────────────────────────────────
    let scan = musaru_scan(&data);
    if scan.malware_detected {
        eprintln!(
            "[ERR!] {fname}: SECURITY BLOCK — {} (Musarû sig #{})",
            scan.detail,
            scan.matched_sig_index.unwrap_or(0)
        );
        return None;
    }
    println!("[INFO] {fname}: security gate — clean");

    // ── Step 2b: VGCA-Δ — binary block trajectory analysis ──────────────────
    {
        const VGCA_BLOCK: usize = 4096;
        let mut bfvs: Vec<BlockFeatureVector> = Vec::new();
        for chunk in data.chunks(VGCA_BLOCK) {
            bfvs.push(BlockFeatureVector::compute(chunk, bfvs.last()));
        }
        let delta = vgca_delta(&bfvs);
        match CorruptionClass::from_vgca_block_result(&delta) {
            Some(cc) => {
                eprintln!(
                    "[VGCA-Δ] {fname}: severity={} frag={:.0}% zip_bomb={}",
                    cc.severity(),
                    delta.fragmentation_ratio * 100.0,
                    delta.zip_bomb_suspected
                );
                if cc.requires_blackbox_routing() {
                    eprintln!(
                        "[ERR!] {fname}: VGCA-Δ severity {} — blackbox routing, batch rejected",
                        cc.severity()
                    );
                    return None;
                }
            }
            None => {
                println!(
                    "[INFO] {fname}: VGCA-Δ — clean  blocks={} frag={:.0}%",
                    bfvs.len(),
                    delta.fragmentation_ratio * 100.0
                );
            }
        }
    }

    // ── Step 3: ZIP extraction ──────────────────────────────────────────────
    let entries = zip_engine::extract_ok(&data);
    if entries.is_empty() {
        eprintln!("[WARN] {fname}: no extractable entries (use zip -0 for STORE method)");
        return None;
    }
    println!("[INFO] {fname}: extracted {} entries", entries.len());

    // ── Step 4: Stage to Processing/ ───────────────────────────────────────
    let batch_name = strip_ext(fname);
    let timestamp = now_secs();
    let batch_dir_name = format!("{}_{}", batch_name, timestamp);

    let staged: Vec<StagedEntry> = entries
        .iter()
        .map(|e| StagedEntry {
            name: e.name.clone(),
            data: e.data.clone(),
        })
        .collect();
    let _batch_dir = match zone.stage(&batch_dir_name, &staged) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("[ERR!] {fname}: stage error — {e}");
            return None;
        }
    };
    println!("[INFO] {fname}: staged → Processing/{}", batch_dir_name);

    // ── Step 5: Schema inference ────────────────────────────────────────────
    let mut batch_golden = 0usize;
    let mut batch_fuzzy = 0usize;
    let mut batch_dead = 0usize;

    // Detect archive type and separate entries by media category.
    // All formats (CSV, JSON, XML, Parquet, Excel, PDF, images, point clouds …)
    // are routed through record_router → Vec<RawRecord> for the station chain.
    let data_entries: Vec<_> = entries
        .iter()
        .filter(|e| {
            let mt = detect_media_type(&e.data);
            mt.category() != MediaCategory::Archive // skip nested archives (logged separately)
        })
        .collect();

    if data_entries.is_empty() {
        eprintln!("[WARN] {fname}: no ingestible entries found in archive");
    } else {
        let fmt_list: Vec<&str> = data_entries
            .iter()
            .map(|e| detect_media_type(&e.data).label())
            .collect();
        println!(
            "[INFO] {fname}: {} data entries: {}",
            data_entries.len(),
            fmt_list.join(", ")
        );
    }

    // Use the first parseable-structured entry for schema inference.
    // Prefer CSV/JSON/XML (natively parseable) over opaque blobs.
    let first_structured = data_entries
        .iter()
        .find(|e| detect_media_type(&e.data).category() == MediaCategory::StructuredData);

    let (schema, first_csv_headers) = if let Some(first_entry) = first_structured {
        let records = records_from_entry(&first_entry.name, &first_entry.data);
        let hdrs = records
            .first()
            .map(|r| r.headers.clone())
            .unwrap_or_default();
        let s = BatchSchema::infer(&batch_name, timestamp, &records);
        let descriptor = s.to_descriptor();
        let _ = zone.write_in_batch(
            &batch_dir_name,
            &format!("{}.schema", batch_name),
            &descriptor,
        );
        println!(
            "[INFO] {fname}: schema → {}  ({} mandatory / {} optional)",
            s.schema_name,
            s.mandatory_attrs.len(),
            s.optional_attrs.len()
        );
        (s, hdrs)
    } else {
        (BatchSchema::infer(&batch_name, timestamp, &[]), vec![])
    };

    // ── Step 5b: compare-tribe-schema gate ─────────────────────────────────────
    // Compare the arriving file schema against the SLA/DubSar SDM reference.
    // First arrival: save arriving schema as the reference (auto-match).
    // Subsequent arrivals: SchemaMismatch → DeadVerdict, batch rejected before any KAKI is minted.
    {
        let ref_path = shard_root
            .join("profiles")
            .join(format!("{}.schema.ref", batch_name));
        let arriving_sv = build_schema_version(
            &batch_name,
            2,
            timestamp as u32,
            &first_csv_headers,
            &schema.mandatory_attrs,
        );

        let compare_report = if ref_path.exists() {
            match load_ref_schema(&ref_path, &batch_name) {
                Some(ref_sv) => {
                    let r = compare_versions(&ref_sv, &arriving_sv, &batch_name, timestamp as u32);
                    println!("[INFO] {fname}: compare-tribe-schema — {}", r.summary());
                    r
                }
                None => {
                    eprintln!("[WARN] {fname}: corrupted .schema.ref — resetting to first arrival");
                    let _ = save_ref_schema(&ref_path, &arriving_sv);
                    compare_versions(&arriving_sv, &arriving_sv, &batch_name, timestamp as u32)
                }
            }
        } else {
            println!(
                "[INFO] {fname}: first arrival — saving reference schema → {}",
                ref_path.display()
            );
            let _ = save_ref_schema(&ref_path, &arriving_sv);
            compare_versions(&arriving_sv, &arriving_sv, &batch_name, timestamp as u32)
        };

        if !compare_report.is_match() {
            let dead_count: usize = data_entries
                .iter()
                .map(|e| records_from_entry(&e.name, &e.data).len())
                .sum();
            let dead_verdict = DeadVerdict::new(dead_count, compare_report);
            let priority_tag = if dead_verdict.alert.is_high_priority() {
                "HIGH"
            } else {
                "NORMAL"
            };
            eprintln!(
                "[SCHEMA-MISMATCH][{}] {fname}: {}",
                priority_tag, dead_verdict.alert.message
            );
            return None;
        }

        if let Some(ref staging) = compare_report.staging_tribe_name {
            println!("[INFO] {fname}: staging_tribe_name = {}", staging);
        }
    }

    // Mint the Entity KAKI via Adad Gate (sole sovereign KAKI issuer for this batch)
    let entity_particle = gate
        .ingest(ArrivalRecord {
            attrs: vec![(fnv1a("batch_entity"), batch_name.as_bytes().to_vec())],
            epoch: timestamp as u32,
            role: KakiRole::Zikru,
        })
        .unwrap_or_else(|e| {
            eprintln!("[ERR!] adad-gate batch entity: {e}");
            std::process::exit(1);
        })
        .particle;

    // ── Step 5b: Load or auto-generate ClientDqProfile ─────────────────────
    let profile_path = shard_root
        .join("profiles")
        .join(format!("{}.dqprofile", batch_name));

    let profile = if profile_path.exists() {
        match std::fs::read_to_string(&profile_path)
            .ok()
            .and_then(|s| parse_profile(&s).ok())
        {
            Some(p) => {
                println!(
                    "[INFO] {fname}: loaded DQ profile → {}",
                    profile_path.display()
                );
                p
            }
            None => {
                eprintln!("[WARN] {fname}: failed to parse .dqprofile, using auto-generated");
                ClientDqProfile::default_from_schema(
                    &batch_name,
                    &schema.mandatory_attrs,
                    &schema.optional_attrs,
                )
            }
        }
    } else {
        println!("[INFO] {fname}: no .dqprofile found — auto-generated from schema ({} mandatory / {} optional)",
            schema.mandatory_attrs.len(), schema.optional_attrs.len());
        ClientDqProfile::default_from_schema(
            &batch_name,
            &schema.mandatory_attrs,
            &schema.optional_attrs,
        )
    };

    // ── Step 5d: PIPELINE router (AKK template or auto-inferred per entry) ─────
    let profiles_dir = shard_root.join("profiles");
    let router = build_pipeline_router(&profiles_dir, &batch_name, fname, &schema);
    let domain_byte = (schema.entity_seed & 0xFF) as u8;

    // ── Step 6: Per-entry station chain (ALL media types via record_router) ────
    for entry in &data_entries {
        let mt = detect_media_type(&entry.data);
        let records = records_from_entry(&entry.name, &entry.data);
        let count = records.len();
        let template = router.template_for(&entry.name);
        println!(
            "[INFO]   {} ({}) → {} records  template={}",
            entry.name,
            mt.label(),
            count,
            template.name
        );

        let (g, f, d) = run_station_chain(
            records,
            gate,
            template,
            &profile,
            domain_byte,
            db,
            pdb,
            steward,
        );
        batch_golden += g;
        batch_fuzzy += f;
        batch_dead += d;
        println!(
            "[INFO]   {} → {} Golden + {} Fuzzy + {} Dead (steward queue: {})",
            entry.name,
            g,
            f,
            d,
            steward.queue_len()
        );
    }

    // ── Step 7: Batch-completion event (file-level StoryEngine entry) ────────
    // Commit Entity KAKI to journal — this is the "file clear" StoryEngine entry.
    let batch_eav = vec![
        EavTriple::new(ATTR_STATE, encode_state(ParticleState::Golden).to_vec()),
        EavTriple::new(fnv1a("batch_name"), batch_name.as_bytes().to_vec()),
        EavTriple::new(fnv1a("batch_golden"), batch_golden.to_string().into_bytes()),
        EavTriple::new(fnv1a("batch_fuzzy"), batch_fuzzy.to_string().into_bytes()),
        EavTriple::new(fnv1a("batch_dead"), batch_dead.to_string().into_bytes()),
        EavTriple::new(
            fnv1a("batch_records"),
            (batch_golden + batch_fuzzy + batch_dead)
                .to_string()
                .into_bytes(),
        ),
    ];

    // Commit to in-memory EnkiDb (StoryEngine gate)
    {
        let ev_db = gate
            .ingest(ArrivalRecord {
                attrs: vec![],
                epoch: timestamp as u32,
                role: KakiRole::Zikru,
            })
            .unwrap_or_else(|e| {
                eprintln!("[ERR!] adad-gate ev_db: {e}");
                std::process::exit(1);
            })
            .event_kaki;
        let mut ps = PermanentStore::new(db);
        let _ = ps.commit(ev_db, entity_particle, timestamp as u32, batch_eav.clone());
    }
    // Commit to crash-safe PersistedDb (disk journal)
    {
        let ev_pdb = gate
            .ingest(ArrivalRecord {
                attrs: vec![],
                epoch: timestamp as u32,
                role: KakiRole::Zikru,
            })
            .unwrap_or_else(|e| {
                eprintln!("[ERR!] adad-gate ev_pdb: {e}");
                std::process::exit(1);
            })
            .event_kaki;
        let _ = pdb.register_particle(&entity_particle);
        let _ = pdb.commit(ev_pdb, entity_particle, timestamp as u32, batch_eav);
    }

    // StoryEngine gate verification: project entity particle from in-memory EnkiDb
    let projected = db.project(&entity_particle);
    println!(
        "[INFO] {fname}: StoryEngine gate — entity events={} state={:?}",
        projected.events_seen, projected.state
    );

    // ── Step 8: Move Processing → Moved_To ──────────────────────────────────
    let _moved_to = match zone.complete(&batch_dir_name) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("[ERR!] {fname}: move to Moved_To failed — {e}");
            PathBuf::from(&batch_dir_name)
        }
    };
    println!(
        "[INFO] {fname}: → Moved_To/{}  ({} Golden / {} Fuzzy / {} Dead / {} DQ alerts)",
        batch_dir_name,
        batch_golden,
        batch_fuzzy,
        batch_dead,
        steward.queue_len()
    );

    Some(BatchResult {
        batch_name,
        golden_count: batch_golden,
        fuzzy_count: batch_fuzzy,
        dead_count: batch_dead,
    })
}

// ── Live export (for bahyway-api) ────────────────────────────────────────────

/// Write per-batch ring buffer (max 20) to `{data_dir}/batches_export.json`.
fn write_batches_export(
    data_dir: &Path,
    tribe_id: u16,
    history: &[(String, u64, usize, usize, usize)], // (name, ts, golden, fuzzy, dead)
) {
    let mut items = String::new();
    for (i, (name, ts, golden, fuzzy, dead)) in history.iter().enumerate() {
        if i > 0 {
            items.push(',');
        }
        let total = golden + fuzzy + dead;
        items.push_str(&format!(
            concat!(
                r#"{{"name":"{n}","timestamp":{ts},"#,
                r#""golden":{g},"fuzzy":{f},"dead":{d},"total":{t}}}"#
            ),
            n = name,
            ts = ts,
            g = golden,
            f = fuzzy,
            d = dead,
            t = total
        ));
    }
    let json = format!(
        r#"{{"status":"ok","tribe_id":"0x{:04X}","count":{cnt},"batches":[{items}]}}"#,
        tribe_id,
        cnt = history.len()
    );
    let path = data_dir.join("batches_export.json");
    if let Err(e) = std::fs::write(&path, json.as_bytes()) {
        eprintln!("[bee-watchdog] WARN: batches_export.json write failed — {e}");
    }
}

/// Write per-tribe stats to `{data_dir}/tribes_summary.json` after each batch.
/// bahyway-api serves this at /api/v1/tribes so the web UI can show live counts
/// overlaid on the static tribe cards.
fn write_tribes_summary(
    data_dir: &Path,
    tribe_id: u16,
    last_batch: &str,
    golden: usize,
    fuzzy: usize,
    dead: usize,
    batches: usize,
) {
    let total = golden + fuzzy + dead;
    let json = format!(
        concat!(
            r#"{{"status":"ok","tribes":[{{"id":"0x{:04X}","#,
            r#""last_batch":"{bn}","golden":{g},"fuzzy":{f},"dead":{d},"total":{t},"batches":{b}}}]}}"#
        ),
        tribe_id,
        bn = last_batch,
        g = golden,
        f = fuzzy,
        d = dead,
        t = total,
        b = batches
    );
    let path = data_dir.join("tribes_summary.json");
    if let Err(e) = std::fs::write(&path, json.as_bytes()) {
        eprintln!("[bee-watchdog] WARN: tribes_summary.json write failed — {e}");
    }
}

/// Write a summary JSON to `{data_dir}/live_export.json` after each batch.
/// bahyway-api serves this file to the bahyway-web frontend.
fn write_live_export(
    data_dir: &Path,
    last_batch: &str,
    timestamp: u64,
    total_golden: usize,
    total_fuzzy: usize,
    total_dead: usize,
    total_batches: usize,
) {
    let total = total_golden + total_fuzzy + total_dead;
    let json = format!(
        concat!(
            r#"{{"status":"ok","timestamp":{ts},"last_batch":"{bn}","total":{total},"#,
            r#""golden":{golden},"fuzzy":{fuzzy},"dead":{dead},"batches":{batches}}}"#
        ),
        ts = timestamp,
        bn = last_batch,
        total = total,
        golden = total_golden,
        fuzzy = total_fuzzy,
        dead = total_dead,
        batches = total_batches,
    );
    let path = data_dir.join("live_export.json");
    if let Err(e) = std::fs::write(&path, json.as_bytes()) {
        eprintln!("[bee-watchdog] WARN: live_export.json write failed — {e}");
    }
}

// ── compare-tribe-schema helpers ─────────────────────────────────────────────

/// Build a SchemaVersion from the original ordered CSV headers + mandatory set.
/// All fields default to FieldType::Text; BatchSchema does not infer column types.
fn build_schema_version(
    batch_name: &str,
    version: u32,
    epoch: u32,
    headers: &[String],
    mandatory_set: &[String],
) -> SchemaVersion {
    let fields: Vec<FieldMeta> = headers
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let is_mandatory = mandatory_set.contains(name);
            FieldMeta::new(
                name.as_str(),
                FieldType::Text,
                None,
                (i + 1) as u16,
                is_mandatory,
                !is_mandatory,
            )
        })
        .collect();
    SchemaVersion::new(version, batch_name, epoch, fields)
}

/// Load a reference SchemaVersion from `shard/profiles/{batch}.schema.ref`.
/// Returns `None` when the file is absent or unparseable.
fn load_ref_schema(ref_path: &Path, batch_name: &str) -> Option<SchemaVersion> {
    let content = std::fs::read_to_string(ref_path).ok()?;
    let mut version = 1u32;
    let mut epoch = 0u32;
    let mut fields: Vec<FieldMeta> = Vec::new();

    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with("# version=") {
            for part in line[2..].split_whitespace() {
                if let Some(v) = part.strip_prefix("version=") {
                    version = v.parse().unwrap_or(1);
                }
                if let Some(e) = part.strip_prefix("epoch=") {
                    epoch = e.parse().unwrap_or(0);
                }
            }
            continue;
        }
        if line.starts_with('#') {
            continue;
        }
        match FieldMeta::parse(line) {
            Ok(f) => fields.push(f),
            Err(e) => {
                eprintln!("[WARN] schema.ref parse error: {e}");
                return None;
            }
        }
    }

    if fields.is_empty() {
        return None;
    }
    Some(SchemaVersion::new(version, batch_name, epoch, fields))
}

/// Persist a SchemaVersion to `shard/profiles/{batch}.schema.ref` on first arrival.
fn save_ref_schema(ref_path: &Path, sv: &SchemaVersion) -> std::io::Result<()> {
    if let Some(parent) = ref_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut content = format!("# version={} epoch={}\n", sv.version, sv.created_epoch);
    for f in &sv.fields {
        let len_str = match f.max_length {
            Some(l) => l.to_string(),
            None => "-".to_string(),
        };
        content.push_str(&format!(
            "{}:{}:{}:{}:{}:{}\n",
            f.name,
            f.data_type.as_str(),
            len_str,
            f.position,
            f.mandatory,
            f.nullable,
        ));
    }
    std::fs::write(ref_path, content.as_bytes())
}

// ── FNV helper ────────────────────────────────────────────────────────────────

fn fnv1a(s: &str) -> u32 {
    let mut h: u32 = 0x811C_9DC5;
    for &b in s.as_bytes() {
        h ^= b as u32;
        h = h.wrapping_mul(0x0100_0193);
    }
    h
}

// ── Adad Gate per-record adapter ──────────────────────────────────────────────

/// Convert a RawRecord into an ArrivalRecord and submit it to the Adad Gate.
/// The gate is the sole sovereign KAKI issuer — no raw KakiMinter in bee-watchdog.
fn adad_ingest_record(gate: &AdadGate, record: &kaki_generator::RawRecord) -> GateResult {
    let mut epoch = record.seq.max(1);
    let mut attrs: Vec<(u32, Vec<u8>)> = Vec::with_capacity(record.headers.len());

    for (col_name, val) in record.headers.iter().zip(record.values.iter()) {
        match col_name.to_lowercase().as_str() {
            "epoch" | "ep" => {
                if let Ok(s) = std::str::from_utf8(val) {
                    if let Ok(n) = s.trim().parse::<u32>() {
                        epoch = n;
                    }
                }
            }
            "state" | "status" => {
                let state = match std::str::from_utf8(val)
                    .unwrap_or("")
                    .trim()
                    .to_lowercase()
                    .as_str()
                {
                    "golden" | "1" | "active" | "ok" => ParticleState::Golden,
                    "dead" | "0" | "inactive" | "off" => ParticleState::Dead,
                    _ => ParticleState::Fuzzy,
                };
                attrs.push((ATTR_STATE, encode_state(state).to_vec()));
            }
            other => {
                if !val.is_empty() {
                    attrs.push((fnv1a(other), val.clone()));
                }
            }
        }
    }

    if !attrs.iter().any(|(h, _)| *h == ATTR_STATE) {
        attrs.push((ATTR_STATE, encode_state(ParticleState::Fuzzy).to_vec()));
    }

    gate.ingest(ArrivalRecord {
        attrs,
        epoch,
        role: KakiRole::Zikru,
    })
    .unwrap_or_else(|e| {
        eprintln!("[ERR!] adad-gate record ingest failed — {e}");
        std::process::exit(1);
    })
}

// ── Main ──────────────────────────────────────────────────────────────────────

fn main() {
    let cfg = parse_args();

    println!(
        "[bee-watchdog] starting  shard={}  data-dir={}  tribe={:#06x}  poll={}ms",
        cfg.shard.display(),
        cfg.data_dir.display(),
        cfg.tribe_id,
        cfg.interval_ms
    );

    let tid = TribeId::from_u16(cfg.tribe_id);
    let gate = AdadGate::new(tid);
    let mut db = EnkiDb::new(tid);

    let mut pdb =
        PersistedDb::open(&cfg.data_dir, tid, FsyncPolicy::PerCommit).unwrap_or_else(|e| {
            eprintln!("[bee-watchdog] FATAL: PersistedDb open failed — {e}");
            std::process::exit(1);
        });

    let mut landing = LandingZone::new(&cfg.shard).unwrap_or_else(|e| {
        eprintln!("[bee-watchdog] FATAL: LandingZone open failed — {e}");
        std::process::exit(1);
    });

    let zone = ProcessingZone::new(&cfg.shard).unwrap_or_else(|e| {
        eprintln!("[bee-watchdog] FATAL: ProcessingZone init failed — {e}");
        std::process::exit(1);
    });

    let mut steward = StewardStation::new();

    println!(
        "[bee-watchdog] ready — watching {} for ZIP files …",
        cfg.shard.display()
    );
    println!(
        "[bee-watchdog] Processing: {}  Moved_To: {}",
        zone.processing_dir().display(),
        zone.moved_to_dir().display()
    );

    let mut total_golden = 0usize;
    let mut total_fuzzy = 0usize;
    let mut total_dead = 0usize;
    let mut total_batches = 0usize;
    // Ring buffer: last 20 batches for /api/v1/batches
    let mut batch_history: Vec<(String, u64, usize, usize, usize)> = Vec::with_capacity(20);

    loop {
        let new_files = landing.poll();

        for lf in new_files {
            let fname = lf
                .path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            use enkidw::landing_zone::LandingFileKind;
            if lf.kind != LandingFileKind::Zip {
                println!("[SKIP] {fname}: not a ZIP (kind={:?})", lf.kind);
                continue;
            }

            // Archive the raw ZIP out of the shard root *before* processing.
            // LandingZone's dedup state ("seen") is in-memory only — without
            // this, restarting bee-watchdog would rediscover every already-
            // completed ZIP still sitting in the shard root and reprocess it,
            // minting duplicate particles for a batch already committed.
            let archived_path = match landing.archive(&lf) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("[ERR!] {fname}: failed to archive source out of shard root — {e}");
                    continue;
                }
            };

            println!("[INFO] === batch start: {fname} ===");
            if let Some(result) = process_zip(
                &archived_path,
                &cfg.shard,
                &fname,
                tid,
                &zone,
                &gate,
                &mut db,
                &mut pdb,
                &mut steward,
            ) {
                total_golden += result.golden_count;
                total_fuzzy += result.fuzzy_count;
                total_dead += result.dead_count;
                total_batches += 1;
                println!("[INFO] === batch done: {} ===", result.batch_name);

                // Export live stats so bahyway-api can serve them to the web UI
                let export_ts = now_secs();
                write_live_export(
                    &cfg.data_dir,
                    &result.batch_name,
                    export_ts,
                    total_golden,
                    total_fuzzy,
                    total_dead,
                    total_batches,
                );
                // Maintain ring buffer then write batch history
                batch_history.push((
                    result.batch_name.clone(),
                    export_ts,
                    result.golden_count,
                    result.fuzzy_count,
                    result.dead_count,
                ));
                if batch_history.len() > 20 {
                    batch_history.remove(0);
                }
                write_batches_export(&cfg.data_dir, cfg.tribe_id, &batch_history);
                write_tribes_summary(
                    &cfg.data_dir,
                    cfg.tribe_id,
                    &result.batch_name,
                    total_golden,
                    total_fuzzy,
                    total_dead,
                    total_batches,
                );
                println!(
                    "[INFO] live + batches + tribes exports updated → {}",
                    cfg.data_dir.display()
                );
            }
        }

        if total_batches > 0 {
            println!(
                "[bee-watchdog] totals  batches={}  golden={}  fuzzy={}  dead={}  dq_queue={}",
                total_batches,
                total_golden,
                total_fuzzy,
                total_dead,
                steward.queue_len()
            );
        }

        thread::sleep(Duration::from_millis(cfg.interval_ms));
    }
}
