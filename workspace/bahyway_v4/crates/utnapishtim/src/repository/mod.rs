//! UTNAPISHTIM — PARZU-KAKI Template Repository
//! tribe.templates — sovereign topology library

#![forbid(unsafe_code)]
use crate::ClientTopology;
use std::path::PathBuf;

pub struct TemplateRepository {
    pub root: PathBuf,
}

impl TemplateRepository {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn save_topology(&self, topo: &ClientTopology) -> std::io::Result<PathBuf> {
        let dir = self.root.join(format!("client_{:04X}", topo.client_id));
        std::fs::create_dir_all(&dir)?;
        let path = dir.join("topology.json");
        let json = serde_json::to_string_pretty(topo).map_err(std::io::Error::other)?;
        std::fs::write(&path, json)?;
        println!("[UTNAPISHTIM] Topology sealed: {:?}", path);
        Ok(path)
    }

    pub fn list_clients(&self) -> Vec<String> {
        std::fs::read_dir(&self.root)
            .map(|rd| {
                rd.filter_map(|e| {
                    e.ok().and_then(|e| {
                        e.file_name()
                            .into_string()
                            .ok()
                            .filter(|n| n.starts_with("client_"))
                    })
                })
                .collect()
            })
            .unwrap_or_default()
    }
}
