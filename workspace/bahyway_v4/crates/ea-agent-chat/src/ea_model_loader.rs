//! ea_model_loader.rs — EaAgent model loader for DeepSeek-Math-7B.
#![forbid(unsafe_code)]

use std::path::PathBuf;

pub const DEEPSEEK_MATH_MODEL: &str = "deepseek-math-7b-instruct.Q4_K_M.gguf";

const MODEL_CANDIDATES: &[&str] = &[
    "deepseek-math-7b-instruct.Q4_K_M.gguf",
    "deepseek-math-7b-instruct-q4_k_m.gguf",
    "DeepSeek-Math-7B-Instruct-Q4_K_M.gguf",
    "Qwen2.5-Math-7B-Instruct-Q4_K_M.gguf",
];

#[derive(Debug, Clone, PartialEq)]
pub enum EaModelStatus {
    NotFound,
    Found(PathBuf),
    Loading(String),
    Ready(String),
    Error(String),
}

impl EaModelStatus {
    pub fn is_ready(&self) -> bool { matches!(self, Self::Ready(_)) }
    pub fn status_str(&self) -> &str {
        match self {
            Self::NotFound   => "⚫ No model",
            Self::Found(_)   => "🟡 Found",
            Self::Loading(_) => "🟠 Loading...",
            Self::Ready(_)   => "🟢 Ready",
            Self::Error(_)   => "🔴 Error",
        }
    }
}

pub struct EaModelLoader {
    pub search_dirs: Vec<PathBuf>,
    pub status:      EaModelStatus,
}

impl EaModelLoader {
    pub fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/bahyway".to_string());
        Self {
            search_dirs: vec![
                PathBuf::from(format!("{home}/models")),
                PathBuf::from(format!("{home}/.local/share/ea-agent/models")),
            ],
            status: EaModelStatus::NotFound,
        }
    }

    pub fn discover(&mut self) -> &EaModelStatus {
        for dir in &self.search_dirs {
            for candidate in MODEL_CANDIDATES {
                let path = dir.join(candidate);
                if path.exists() {
                    self.status = EaModelStatus::Found(path);
                    return &self.status;
                }
            }
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|e| e.to_str()) == Some("gguf") {
                        let name = path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("")
                            .to_lowercase();
                        if name.contains("math") || name.contains("deepseek") {
                            self.status = EaModelStatus::Found(path);
                            return &self.status;
                        }
                    }
                }
            }
        }
        self.status = EaModelStatus::NotFound;
        &self.status
    }

    pub fn download_instructions() -> String {
        format!(
            "To enable EaAgent full AI, download the Math model:\n\n\
             mkdir -p ~/models\n\
             wget -O ~/models/{DEEPSEEK_MATH_MODEL} \\\n  \
             \"https://huggingface.co/mradermacher/deepseek-math-7b-instruct-GGUF/\\\nresolve/main/{DEEPSEEK_MATH_MODEL}\"\n\n\
             Model size: ~4.0GB | Sovereign: 100% offline"
        )
    }
}

impl Default for EaModelLoader { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn not_found_status_correct() {
        let s = EaModelStatus::NotFound;
        assert!(!s.is_ready());
        assert_eq!(s.status_str(), "⚫ No model");
    }
    #[test]
    fn ready_status_is_ready() {
        let s = EaModelStatus::Ready("test".into());
        assert!(s.is_ready());
    }
    #[test]
    fn loader_has_search_dirs() {
        let l = EaModelLoader::new();
        assert!(!l.search_dirs.is_empty());
    }
    #[test]
    fn download_instructions_contain_model() {
        let inst = EaModelLoader::download_instructions();
        assert!(inst.contains("deepseek-math"));
        assert!(inst.contains("wget"));
    }
}
