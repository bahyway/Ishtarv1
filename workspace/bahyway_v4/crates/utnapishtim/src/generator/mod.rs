//! UTNAPISHTIM — Full Generation Orchestrator
//! One call generates everything for one client É-DUBBA session

#![forbid(unsafe_code)]
use std::path::PathBuf;
use crate::ClientTopology;
use crate::threejs::generate_threejs_viewer;
use crate::godot::generate_godot_app;
use crate::manifest::generate_manifest;
use crate::repository::TemplateRepository;

pub struct UtnapishtimGenerator {
    pub output_dir: PathBuf,
    pub repo:       TemplateRepository,
}

impl UtnapishtimGenerator {
    pub fn new(output_dir: impl Into<PathBuf>) -> Self {
        let output_dir = output_dir.into();
        let repo = TemplateRepository::new(output_dir.join("tribe.templates"));
        Self { output_dir, repo }
    }

    /// Generate all deliverables for one client.
    /// Called when É-DUBBA session is sealed.
    pub fn generate(&self, topo: &ClientTopology) -> Result<GenerationResult, UtnapishtimError> {
        topo.validate().map_err(UtnapishtimError::TopologyInvalid)?;

        let client_dir = self.output_dir.join(format!("client_{:04X}", topo.client_id));
        std::fs::create_dir_all(&client_dir)
            .map_err(UtnapishtimError::IoError)?;

        // 1. Three.js viewer
        let html    = generate_threejs_viewer(topo);
        let html_path = client_dir.join(format!("dubsar_viewer_{}.html", topo.client_id));
        std::fs::write(&html_path, &html).map_err(UtnapishtimError::IoError)?;
        println!("[UTNAPISHTIM] Three.js viewer → {:?}", html_path);

        // 2. Godot app
        let godot_app = generate_godot_app(topo);
        let godot_dir = client_dir.join(format!("dubsar_pdm_{}", topo.client_id));
        std::fs::create_dir_all(&godot_dir).map_err(UtnapishtimError::IoError)?;
        for file in &godot_app.files {
            let path = godot_dir.join(&file.name);
            std::fs::write(&path, &file.content).map_err(UtnapishtimError::IoError)?;
            println!("[UTNAPISHTIM] Godot file → {:?}", path);
        }

        // 3. manifest.akk
        let manifest = generate_manifest(topo, &client_dir.to_string_lossy());
        let mpath    = client_dir.join("manifest.akk");
        std::fs::write(&mpath, &manifest).map_err(UtnapishtimError::IoError)?;
        println!("[UTNAPISHTIM] manifest.akk → {:?}", mpath);

        // 4. Save topology to repository
        self.repo.save_topology(topo).map_err(UtnapishtimError::IoError)?;

        println!("[UTNAPISHTIM] 𒌓𒍣𒅁𒀭 The flood cannot reach what has been sealed.");

        Ok(GenerationResult {
            html_path,
            godot_dir,
            manifest_path: mpath,
        })
    }
}

pub struct GenerationResult {
    pub html_path:     PathBuf,
    pub godot_dir:     PathBuf,
    pub manifest_path: PathBuf,
}

#[derive(Debug)]
pub enum UtnapishtimError {
    TopologyInvalid(String),
    #[allow(dead_code)]
    GenerationFailed(String),
    IoError(std::io::Error),
}
