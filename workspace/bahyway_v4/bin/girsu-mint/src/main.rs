//! girsu-mint — Girsu IDE's "mark as final" trigger for the KAKIv4.0
//! Generator. Invoked (via a VS Code task) on one real `.akk`/`.way`/
//! `.tmpl` tablet at a time: profiles it (`enkimdb::profile_tablet`),
//! mints a real Identity-Kaki for it (`enkimdb::WriteNode::ingest_tablet`,
//! role=Parzu, journaled under `EventCause::TabletRegistered`), and
//! records the mint in a durable JSONL registry keyed by canonical path
//! — so re-running "mark as final" on an already-finalized tablet is a
//! no-op (reports "already finalized", does not re-mint), matching
//! `anu_governor::pb_mint`'s idempotent-by-identity convention exactly.
//!
//! On a genuine new mint, also materializes a fresh, HeptaScript-
//! queryable Euphrates generation, so the Identity-Kaki is queryable
//! immediately — not just journaled, but actually visible to a
//! HeptaScript `WHERE` clause the moment Girsu finishes the run.
#![forbid(unsafe_code)]

use std::collections::HashSet;
use std::io::Write;
use std::path::PathBuf;

use bahyway_core::TribeId;
use enkidb_kaki::KakiMinter;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct RegistryLine {
    path: String,
    kind: String,
    kaki_hex: String,
    sha256: String,
    minted_at: String,
}

struct Args {
    file: PathBuf,
    enkimdb_root: PathBuf,
    registry: PathBuf,
}

fn parse_args() -> Args {
    let mut file: Option<PathBuf> = None;
    let mut enkimdb_root = PathBuf::from("enkimdb_data");
    let mut registry = PathBuf::from("chronicle/tablet_kaki_registry.jsonl");

    let mut it = std::env::args().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--enkimdb-root" => {
                enkimdb_root =
                    PathBuf::from(it.next().unwrap_or_else(|| fail("--enkimdb-root needs a value")))
            }
            "--registry" => {
                registry = PathBuf::from(it.next().unwrap_or_else(|| fail("--registry needs a value")))
            }
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            other if file.is_none() && !other.starts_with('-') => file = Some(PathBuf::from(other)),
            other => fail(&format!("unknown argument: {other}")),
        }
    }

    let file = file.unwrap_or_else(|| fail("missing required <file> argument"));
    Args { file, enkimdb_root, registry }
}

fn fail(msg: &str) -> ! {
    eprintln!("girsu-mint: {msg}");
    print_usage();
    std::process::exit(1);
}

fn print_usage() {
    eprintln!(
        "girsu-mint <file.akk|.way|.tmpl> [--enkimdb-root DIR] [--registry PATH]\n\
         Mints a real KAKIv4.0 Identity for a finalized AAOL/WAY/TemplateEngine\n\
         tablet and journals it into EnkiMDB. Idempotent: re-running on an\n\
         already-minted path reports \"already finalized\" and mints nothing."
    );
}

fn main() {
    let args = parse_args();

    let canonical = std::fs::canonicalize(&args.file).unwrap_or_else(|e| {
        eprintln!("girsu-mint: cannot read {}: {e}", args.file.display());
        std::process::exit(1);
    });
    let canonical_str = canonical.display().to_string();

    let already: HashSet<String> = std::fs::read_to_string(&args.registry)
        .unwrap_or_default()
        .lines()
        .filter_map(|l| serde_json::from_str::<RegistryLine>(l).ok())
        .map(|r| r.path)
        .collect();

    if already.contains(&canonical_str) {
        println!("𒁾 already finalized: {canonical_str} (no re-mint)");
        return;
    }

    let profile = enkimdb::profile_tablet(&canonical).unwrap_or_else(|e| {
        eprintln!("girsu-mint: {e}");
        std::process::exit(1);
    });

    let tribe_id = profile.kind.tribe_id();
    let minter = KakiMinter::new(TribeId::from_u16(tribe_id));
    let mut write_node = enkimdb::WriteNode::new(minter, 64);
    let kaki = write_node.ingest_tablet(&profile, 1);

    if let Some(p) = args.registry.parent() {
        let _ = std::fs::create_dir_all(p);
    }
    let line = RegistryLine {
        path: canonical_str.clone(),
        kind: profile.kind.as_str().to_string(),
        kaki_hex: hex::encode(kaki.bytes()),
        sha256: profile.sha256.clone(),
        minted_at: chrono::Utc::now().to_rfc3339(),
    };
    match std::fs::OpenOptions::new().create(true).append(true).open(&args.registry) {
        Ok(mut f) => {
            if let Ok(json) = serde_json::to_string(&line) {
                let _ = writeln!(f, "{json}");
            }
        }
        Err(e) => eprintln!("⚠ girsu-mint: could not write registry {}: {e}", args.registry.display()),
    }

    println!(
        "𒁾 KAKIv4.0 Identity minted for {} ({}) → {kaki}",
        canonical_str,
        profile.kind.as_str()
    );

    // Millisecond resolution, unlike pb_mint's per-batch %Y%m%d_%H%M%S --
    // girsu-mint materializes once per *file* (a human can mark two
    // tablets final inside the same wall-clock second), and
    // enkidb_datafile::DataFileWriter::open always appends, never
    // truncates: a same-second stamp collision would silently duplicate
    // this run's particles into the previous run's generation on disk.
    let stamp = chrono::Utc::now().format("%Y%m%d_%H%M%S%.3f").to_string();
    match enkimdb::materialize_version(&write_node, &args.enkimdb_root, &stamp) {
        Ok((generation, stats)) => println!(
            "𒁾 Euphrates generation {stamp} materialized: {} entities → {}",
            stats.entities,
            generation.entities_path.display()
        ),
        Err(e) => eprintln!("⚠ girsu-mint: EnkiMDB materialize failed: {e}"),
    }
}
