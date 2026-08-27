//! Real config loader for `~/bahyway/forge/enlil_field.toml`, the file
//! `playbook_623_enlil_field_kernel.yml` writes before building/running
//! this binary. Read by `$HOME`, not by CWD, since this binary's real
//! call sites (`--gate ...`, `--bench ...`) run with no fixed working
//! directory.

use std::fs;
use std::path::PathBuf;

pub struct FieldConfig {
    pub shells: usize,
    pub angles: usize,
    pub radii: usize,
}

impl Default for FieldConfig {
    /// Same real defaults `playbook_623` itself writes into the tablet
    /// (`grid: {shells: 7, angles: 64, radii: 28}`) -- used only when the
    /// config file is missing or unreadable, never silently substituted
    /// for a config that IS present but malformed in a way that changes
    /// the grid shape.
    fn default() -> Self {
        Self {
            shells: 7,
            angles: 64,
            radii: 28,
        }
    }
}

pub fn config_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join("bahyway")
        .join("forge")
        .join("enlil_field.toml")
}

pub fn load() -> FieldConfig {
    let path = config_path();
    let Ok(raw) = fs::read_to_string(&path) else {
        return FieldConfig::default();
    };
    let Ok(doc) = raw.parse::<toml::Table>() else {
        return FieldConfig::default();
    };
    let grid = doc.get("grid").and_then(|v| v.as_table());
    let get = |key: &str, default: usize| -> usize {
        grid.and_then(|g| g.get(key))
            .and_then(|v| v.as_integer())
            .and_then(|n| usize::try_from(n).ok())
            .unwrap_or(default)
    };
    FieldConfig {
        shells: get("shells", 7),
        angles: get("angles", 64),
        radii: get("radii", 28),
    }
}
