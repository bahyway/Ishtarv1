//! model_loader.rs — Sovereign model loader for TamuzAI.
//!
//! Handles model discovery, status tracking, and loading from disk.
//! The model file lives at ~/models/ on the VDI.
#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};

/// Default model filename (DeepSeek-Coder-6.7B Q4_K_M).
pub const DEFAULT_MODEL_NAME: &str = "deepseek-coder-6.7b-instruct.Q4_K_M.gguf";

/// Alternative model names to search for.
const MODEL_CANDIDATES: &[&str] = &[
    "deepseek-coder-6.7b-instruct.Q4_K_M.gguf",
    "deepseek-coder-6.7b-instruct-q4_k_m.gguf",
    "DeepSeek-Coder-V2-Lite-Instruct-Q4_K_M.gguf",
    "qwen2.5-coder-7b-instruct-q4_k_m.gguf",
    "Qwen2.5-Coder-7B-Instruct-Q4_K_M.gguf",
    "codellama-7b-instruct.Q4_K_M.gguf",
];

/// Model loading status.
#[derive(Debug, Clone, PartialEq)]
pub enum ModelStatus {
    /// No model found on disk.
    NotFound,
    /// Model file found, not yet loaded.
    Found(PathBuf),
    /// Model is being loaded into memory.
    Loading(String),
    /// Model loaded and ready for inference.
    Ready(String),
    /// Model failed to load.
    Error(String),
}

impl ModelStatus {
    pub fn is_ready(&self) -> bool { matches!(self, Self::Ready(_)) }
    pub fn is_loading(&self) -> bool { matches!(self, Self::Loading(_)) }

    pub fn status_str(&self) -> &str {
        match self {
            Self::NotFound      => "⚫ No model",
            Self::Found(_)      => "🟡 Found",
            Self::Loading(_)    => "🟠 Loading...",
            Self::Ready(_)      => "🟢 Ready",
            Self::Error(_)      => "🔴 Error",
        }
    }

    pub fn model_name(&self) -> Option<&str> {
        match self {
            Self::Found(p)   => p.file_name()?.to_str(),
            Self::Loading(n) => Some(n),
            Self::Ready(n)   => Some(n),
            Self::Error(e)   => Some(e),
            _                => None,
        }
    }
}

/// Sovereign model loader — discovers and tracks model files.
pub struct ModelLoader {
    pub search_dirs: Vec<PathBuf>,
    pub status:      ModelStatus,
}

impl ModelLoader {
    pub fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/bahyway".to_string());
        Self {
            search_dirs: vec![
                PathBuf::from(format!("{home}/models")),
                PathBuf::from(format!("{home}/.local/share/enkidullm/models")),
                PathBuf::from("/opt/enkidullm/models"),
            ],
            status: ModelStatus::NotFound,
        }
    }

    /// Search for a GGUF model file in the configured directories.
    pub fn discover(&mut self) -> &ModelStatus {
        for dir in &self.search_dirs {
            for candidate in MODEL_CANDIDATES {
                let path = dir.join(candidate);
                if path.exists() {
                    self.status = ModelStatus::Found(path);
                    return &self.status;
                }
            }
            // Also check for any .gguf file in the directory
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|e| e.to_str()) == Some("gguf") {
                        self.status = ModelStatus::Found(path);
                        return &self.status;
                    }
                }
            }
        }
        self.status = ModelStatus::NotFound;
        &self.status
    }

    /// Get the download instructions for the recommended model.
    pub fn download_instructions() -> String {
        format!(
            "To install TamuzAI, download the model:\n\n\
             # Install huggingface-cli\n\
             pip install huggingface_hub\n\n\
             # Download DeepSeek-Coder-6.7B (sovereign offline model)\n\
             mkdir -p ~/models\n\
             huggingface-cli download TheBloke/deepseek-coder-6.7B-instruct-GGUF \\\n\
             {DEFAULT_MODEL_NAME} \\\n\
             --local-dir ~/models/\n\n\
             # Verify download\n\
             ls -lh ~/models/*.gguf\n\n\
             Model size: ~3.8GB | VRAM required: ~4GB | Sovereign: 100% offline"
        )
    }
}

impl Default for ModelLoader { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_status_ready_is_ready() {
        let s = ModelStatus::Ready("test".to_string());
        assert!(s.is_ready());
        assert!(!s.is_loading());
    }

    #[test]
    fn model_status_loading_is_loading() {
        let s = ModelStatus::Loading("test".to_string());
        assert!(s.is_loading());
        assert!(!s.is_ready());
    }

    #[test]
    fn model_status_str_correct() {
        assert_eq!(ModelStatus::NotFound.status_str(), "⚫ No model");
        assert_eq!(ModelStatus::Ready("x".into()).status_str(), "🟢 Ready");
        assert_eq!(ModelStatus::Error("e".into()).status_str(), "🔴 Error");
    }

    #[test]
    fn model_loader_has_search_dirs() {
        let loader = ModelLoader::new();
        assert!(!loader.search_dirs.is_empty());
    }

    #[test]
    fn download_instructions_contain_model_name() {
        let inst = ModelLoader::download_instructions();
        assert!(inst.contains("deepseek"));
        assert!(inst.contains("huggingface-cli"));
        assert!(inst.contains("~/models"));
    }

    #[test]
    fn default_model_name_is_gguf() {
        assert!(DEFAULT_MODEL_NAME.ends_with(".gguf"));
    }
}
