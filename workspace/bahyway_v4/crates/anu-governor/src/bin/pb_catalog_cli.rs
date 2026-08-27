//! pb-catalog-cli — headless face for the multi-location playbook catalog
//! (`anu_governor::pb_catalog`). Drives it from a TOML config and a
//! terminal, no windowing session required — same rationale as
//! `uruinimgina-cli` (Hala's headless face, as of Phase 2
//! reconciliation -- see docs/99_index/BAHYWAY_PHASE2_GLOSSARY.md). Built for PB-289
//! (`playbooks/playbook_289_multi_location_playbook_catalog.yml`).
//!
//! Usage: `pb-catalog-cli <config.toml>` (default: `pb_catalog.toml`).
//!
//! Config shape:
//! ```toml
//! registry_path = "chronicle/pb_catalog_registry.jsonl"
//! enkiddb_output_root = "data/enkiddb"
//!
//! [[locations]]
//! name = "in-repo/playbooks"
//! root = "/home/bfadam/Forge/EnkiDB/playbooks"
//!
//! [[locations]]
//! name = "VMLAB/20260729"
//! root = "/run/media/bfadam/VMLAB/__All_BahyWay/Forge_Daily_Backups/20260729"
//! ```

use anu_governor::pb_catalog::{catalog_playbooks, CatalogLocation};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize)]
struct LocationCfg {
    name: String,
    root: String,
}

#[derive(Deserialize)]
struct CliConfig {
    registry_path: String,
    enkiddb_output_root: String,
    locations: Vec<LocationCfg>,
}

fn main() {
    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "pb_catalog.toml".to_string());

    let text = match std::fs::read_to_string(&config_path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("✗ cannot read config {config_path}: {e}");
            std::process::exit(1);
        }
    };
    let cfg: CliConfig = match toml::from_str(&text) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("✗ cannot parse config {config_path}: {e}");
            std::process::exit(1);
        }
    };
    if cfg.locations.is_empty() {
        eprintln!("✗ config {config_path} lists no [[locations]] -- nothing to catalog");
        std::process::exit(1);
    }

    let locations: Vec<CatalogLocation> = cfg
        .locations
        .into_iter()
        .map(|l| CatalogLocation {
            name: l.name,
            root: PathBuf::from(l.root),
        })
        .collect();

    println!(
        "𒁾 pb-catalog-cli — scanning {} location(s)",
        locations.len()
    );
    for loc in &locations {
        println!("   · {} → {}", loc.name, loc.root.display());
    }
    println!();

    let log = catalog_playbooks(
        &locations,
        &PathBuf::from(&cfg.registry_path),
        &PathBuf::from(&cfg.enkiddb_output_root),
    );
    for line in &log {
        println!("{line}");
    }

    let collisions = log.iter().filter(|l| l.starts_with("⚠ collision")).count();
    let unreachable = log.iter().filter(|l| l.contains("not reachable")).count();
    if unreachable > 0 {
        eprintln!("\n⚠ {unreachable} location(s) were not reachable -- see above, the catalog is incomplete until they are.");
    }
    if collisions > 0 {
        println!("\n⚠ {collisions} same-PB-number collision(s) found -- open Graph Explorer (Ctrl+K) in DubSar Theater and expand a colliding title to review.");
    }
}
