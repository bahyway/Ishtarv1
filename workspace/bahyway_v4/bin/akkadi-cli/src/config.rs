//! Akkadi CLI configuration — ~/.akkadi/config.toml

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AkkadiConfig {
    pub enkidb_url: String,
    pub postgres_dsn: Option<String>,
    pub output_format: String,
    pub sovereign_workspace: PathBuf,
    /// Real target for `CellKind::EnkiddbQuery` (`kernel::AkkadiKernel::
    /// run_enkiddb_query`) -- EnkiDDB's own read node, not the same thing
    /// `enkidb_url` names (that field is unused for any real connection
    /// today; this pair actually gets dialed). Default matches this
    /// ecosystem's real deployed port (`playbook_192`'s `enkiddb-read`).
    #[serde(default = "default_enkiddb_read_host")]
    pub enkiddb_read_host: String,
    #[serde(default = "default_enkiddb_read_port")]
    pub enkiddb_read_port: u16,
}

fn default_enkiddb_read_host() -> String {
    "127.0.0.1".to_string()
}

fn default_enkiddb_read_port() -> u16 {
    7102
}

impl Default for AkkadiConfig {
    fn default() -> Self {
        Self {
            enkidb_url: format!("http://localhost:{}", crate::ENKIDB_PORT),
            postgres_dsn: None,
            output_format: "table".into(),
            sovereign_workspace: dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".akkadi"),
            enkiddb_read_host: default_enkiddb_read_host(),
            enkiddb_read_port: default_enkiddb_read_port(),
        }
    }
}

impl AkkadiConfig {
    pub fn config_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".akkadi")
            .join("config.toml")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path).unwrap_or_default();
            toml::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }
}
