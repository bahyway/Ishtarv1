//! model_config.rs — Sovereign model configuration and KAKI identity.
#![forbid(unsafe_code)]

use enkidb_kaki::{Kaki, KakiMinter, KakiRole};
use bahyway_core::TribeId;

/// Tribe ID for EnkiduLLM model particles.
pub const MODEL_TRIBE_ID: u16 = 0x1100;

/// Supported transformer architectures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelArch {
    LLaMA,
    Qwen2,
    Mistral,
    Unknown,
}

impl ModelArch {
    pub fn from_str(s: &str) -> Self {
        match s {
            "llama"   => Self::LLaMA,
            "qwen2"   => Self::Qwen2,
            "mistral" => Self::Mistral,
            _         => Self::Unknown,
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LLaMA   => "llama",
            Self::Qwen2   => "qwen2",
            Self::Mistral => "mistral",
            Self::Unknown => "unknown",
        }
    }
}

/// Complete model configuration derived from GGUF metadata.
#[derive(Debug, Clone)]
pub struct ModelConfig {
    pub arch:            ModelArch,
    pub model_name:      String,
    pub context_length:  usize,
    pub embedding_dim:   usize,
    pub num_layers:      usize,
    pub num_heads:       usize,
    pub num_kv_heads:    usize,
    pub feed_forward_dim: usize,
    pub vocab_size:      usize,
    pub rope_theta:      f32,
    pub rms_norm_eps:    f32,
}

impl ModelConfig {
    /// Derive from GGUF metadata.
    pub fn from_metadata(meta: &crate::gguf::GgufMetadata) -> Option<Self> {
        let arch_str = meta.arch()?;
        let arch = ModelArch::from_str(arch_str);
        Some(Self {
            arch,
            model_name:       meta.model_name().unwrap_or("unknown").to_string(),
            context_length:   meta.context_length()? as usize,
            embedding_dim:    meta.embedding_dim()? as usize,
            num_layers:       meta.num_layers()? as usize,
            num_heads:        meta.num_heads()? as usize,
            num_kv_heads:     meta.num_kv_heads().unwrap_or(meta.num_heads()?) as usize,
            feed_forward_dim: meta.feed_forward_dim()? as usize,
            vocab_size:       meta.vocab_size()? as usize,
            rope_theta:       meta.kv.get(&format!("{arch_str}.rope.freq_base"))
                                  .and_then(|v| v.as_f32()).unwrap_or(10000.0),
            rms_norm_eps:     meta.kv.get(&format!("{arch_str}.attention.layer_norm_rms_epsilon"))
                                  .and_then(|v| v.as_f32()).unwrap_or(1e-5),
        })
    }

    /// Head dimension.
    pub fn head_dim(&self) -> usize { self.embedding_dim / self.num_heads }

    /// Is this a Grouped Query Attention model?
    pub fn is_gqa(&self) -> bool { self.num_kv_heads != self.num_heads }
}

/// KAKI-stamped model particle — every loaded model is a sovereign entity.
#[derive(Debug)]
pub struct ModelKaki {
    pub nucleus: Kaki,
    pub config:  ModelConfig,
    pub path:    String,
}

impl ModelKaki {
    pub fn new(config: ModelConfig, path: &str) -> Self {
        let minter = KakiMinter::new(TribeId::from_u16(MODEL_TRIBE_ID));
        // uuid_hash derived from model name + arch
        let content = format!("{}:{}", config.model_name, config.arch.as_str());
        let hash = fnv1a_32(content.as_bytes());
        let nucleus = minter.mint_identity(hash, KakiRole::Kishib);
        Self { nucleus, config, path: path.to_string() }
    }

    pub fn kaki_hex(&self) -> String {
        self.nucleus.bytes().iter().map(|b| format!("{b:02x}")).collect()
    }
}

fn fnv1a_32(data: &[u8]) -> u32 {
    const OFFSET: u32 = 2166136261;
    const PRIME: u32  = 16777619;
    let mut h = OFFSET;
    for &b in data { h ^= b as u32; h = h.wrapping_mul(PRIME); }
    h
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn model_arch_from_str() {
        assert_eq!(ModelArch::from_str("qwen2"), ModelArch::Qwen2);
        assert_eq!(ModelArch::from_str("llama"), ModelArch::LLaMA);
        assert_eq!(ModelArch::from_str("unknown_xyz"), ModelArch::Unknown);
    }
    #[test]
    fn head_dim_correct() {
        let cfg = ModelConfig {
            arch: ModelArch::Qwen2,
            model_name: "test".into(),
            context_length: 4096, embedding_dim: 4096,
            num_layers: 32, num_heads: 32, num_kv_heads: 8,
            feed_forward_dim: 11008, vocab_size: 32000,
            rope_theta: 10000.0, rms_norm_eps: 1e-5,
        };
        assert_eq!(cfg.head_dim(), 128);
        assert!(cfg.is_gqa());
    }
}
