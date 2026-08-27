//! ReproManifest — the "configuration file" half of a client debug
//! delivery. The Architect's own framing: when a client's query on
//! EnkiDDB-as-a-Service crashes, degrades, or fails silently, reproduce
//! it in a private test environment, then send the client the config
//! file plus the notebook -- never environment access itself.
//!
//! This is a genuine gap this ecosystem never had a struct for before
//! (confirmed by direct search: no `SessionConfig`/`EnvironmentManifest`
//! exists anywhere; `enkiddb-write-server`/`enkiddb-read-server` only
//! ever read their own env vars inline, nothing captures what a specific
//! reproduction run actually used). Deliberately minimal: what the
//! client needs to understand HOW their issue was reproduced, not a
//! full environment dump (no credentials, no internal paths beyond the
//! target host:port the reproduction itself used).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReproManifest {
    /// Short human title, matches the companion notebook's own title.
    pub title: String,
    /// What the client reported -- crash, silent error, degraded
    /// performance -- in the Architect's own words, not a category enum
    /// (this is read by a human, not machine-dispatched).
    pub issue: String,
    /// `host:port` the reproduction actually ran against.
    pub target: String,
    /// The exact request frame sent (`QUERY:...`/`SEARCH:...`), so the
    /// client can see precisely what was tested, byte for byte.
    pub query: String,
    /// RFC 3339 timestamp of when the reproduction ran.
    pub captured_at: String,
    /// Round-trip time for the reproduction, in milliseconds -- present
    /// even when the outcome was an error, since "how long before it
    /// failed" is itself diagnostic for a performance complaint.
    pub elapsed_ms: u128,
    /// Whether the reproduction's own EAV materialization was the
    /// version the Architect targeted -- filled in by the caller from
    /// whatever `readnode::list_versions`/generation tag applies, left
    /// as a plain string here since this crate has no dependency on
    /// `enkiddb` and shouldn't gain one just for a manifest field.
    pub dataset_generation: Option<String>,
    /// The companion `.akknb` notebook's file name (not a full path --
    /// the manifest and notebook are meant to travel together in one
    /// folder/zip, not reference each other's disk location).
    pub notebook_file: String,
}

impl ReproManifest {
    pub fn save(&self, path: &std::path::Path) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(path, json).map_err(|e| e.to_string())
    }

    pub fn load(path: &std::path::Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> ReproManifest {
        ReproManifest {
            title: "Slow WHERE clause on components collection".to_string(),
            issue: "Client reports the query takes >5s and sometimes times out.".to_string(),
            target: "127.0.0.1:7102".to_string(),
            query: "QUERY:WHO T.E\nWHAT E[meta.title]\nWHERE E[meta.collection] = \"components\""
                .to_string(),
            captured_at: "2026-07-19T12:00:00Z".to_string(),
            elapsed_ms: 42,
            dataset_generation: Some("Tigris v4.0".to_string()),
            notebook_file: "debug_20260719.akknb".to_string(),
        }
    }

    #[test]
    fn round_trips_through_save_and_load() {
        let dir = std::env::temp_dir().join("akkadi_repro_manifest_test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("test.repro.json");

        let original = sample();
        original.save(&path).unwrap();
        let loaded = ReproManifest::load(&path).unwrap();

        assert_eq!(loaded.title, original.title);
        assert_eq!(loaded.query, original.query);
        assert_eq!(loaded.elapsed_ms, original.elapsed_ms);
        assert_eq!(loaded.dataset_generation, original.dataset_generation);
    }

    #[test]
    fn serializes_as_readable_json_a_client_can_open_in_a_text_editor() {
        let json = serde_json::to_string_pretty(&sample()).unwrap();
        assert!(json.contains("\"issue\""));
        assert!(json.contains("\"target\": \"127.0.0.1:7102\""));
    }
}
