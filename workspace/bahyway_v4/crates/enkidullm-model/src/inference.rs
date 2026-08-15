//! inference.rs — Sovereign transformer inference engine for EnkiduLLM.
//!
//! Implements the core LLaMA/Qwen2/Mistral transformer forward pass.
//! Pure Rust — no BLAS, no candle, no PyTorch. CPU inference only for now.
//! GPU path (Vulkan compute) is planned for v4.1.
#![forbid(unsafe_code)]

use crate::model_config::ModelConfig;
use crate::quant::{rms_norm, softmax_inplace, silu, matmul_f32};
use crate::sampler::{TokenSampler, SamplerConfig};

/// Configuration for a single inference run.
#[derive(Debug, Clone)]
pub struct InferenceConfig {
    /// Maximum tokens to generate.
    pub max_new_tokens: usize,
    /// System prompt prepended to every conversation.
    pub system_prompt:  Option<String>,
    /// Sampler configuration.
    pub sampler:        SamplerConfig,
    /// Stop at EOS token.
    pub stop_at_eos:    bool,
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
            max_new_tokens: 512,
            system_prompt:  Some("You are EnkiduLLM, the sovereign MetaEngine of BahyWay.Ecosystem v4.0. You know every crate, every ADR, and every sovereign law. You answer in precise technical language. You never suggest external dependencies.".to_string()),
            sampler:        SamplerConfig::code(),
            stop_at_eos:    true,
        }
    }
}

/// A single generated token with metadata.
#[derive(Debug, Clone)]
pub struct StreamToken {
    pub token_id:  u32,
    pub token_str: String,
    pub logprob:   f32,
    pub is_eos:    bool,
}

/// Complete inference result.
#[derive(Debug)]
pub struct InferenceResult {
    pub generated_text:   String,
    pub tokens_generated: usize,
    pub tokens_prompt:    usize,
    pub stopped_by_eos:   bool,
}

/// KV cache entry for one transformer layer.
struct KvCache {
    k: Vec<f32>,  // [seq_len × num_kv_heads × head_dim]
    v: Vec<f32>,  // [seq_len × num_kv_heads × head_dim]
    seq_len: usize,
}

impl KvCache {
    fn new(max_seq: usize, num_kv_heads: usize, head_dim: usize) -> Self {
        let cap = max_seq * num_kv_heads * head_dim;
        Self { k: vec![0.0; cap], v: vec![0.0; cap], seq_len: 0 }
    }

    fn append_kv(&mut self, k_new: &[f32], v_new: &[f32], num_kv_heads: usize, head_dim: usize) {
        let step = num_kv_heads * head_dim;
        let base = self.seq_len * step;
        self.k[base..base + step].copy_from_slice(k_new);
        self.v[base..base + step].copy_from_slice(v_new);
        self.seq_len += 1;
    }
}

/// Pre-loaded weight tensors for one transformer layer.
pub struct LayerWeights {
    pub attn_norm:  Vec<f32>,  // [dim]
    pub ffn_norm:   Vec<f32>,  // [dim]
    pub wq:         Vec<f32>,  // [dim × dim]
    pub wk:         Vec<f32>,  // [dim × kv_dim]
    pub wv:         Vec<f32>,  // [dim × kv_dim]
    pub wo:         Vec<f32>,  // [dim × dim]
    pub w_gate:     Vec<f32>,  // [ffn_dim × dim]
    pub w_up:       Vec<f32>,  // [ffn_dim × dim]
    pub w_down:     Vec<f32>,  // [dim × ffn_dim]
}

/// Complete model weights.
pub struct ModelWeights {
    pub config:       ModelConfig,
    pub token_emb:    Vec<f32>,  // [vocab × dim]
    pub norm_final:   Vec<f32>,  // [dim]
    pub lm_head:      Vec<f32>,  // [vocab × dim]
    pub layers:       Vec<LayerWeights>,
}

/// The sovereign inference engine.
pub struct InferenceEngine {
    weights: ModelWeights,
}

impl InferenceEngine {
    pub fn new(weights: ModelWeights) -> Self {
        Self { weights }
    }

    /// Generate tokens from a prompt string.
    pub fn generate(
        &self,
        tokenizer: &crate::tokenizer::BpeTokenizer,
        prompt: &str,
        config: &InferenceConfig,
    ) -> InferenceResult {
        // Build full prompt with system context
        let full_prompt = match &config.system_prompt {
            Some(sys) => format!("<|system|>\n{sys}\n<|user|>\n{prompt}\n<|assistant|>\n"),
            None      => prompt.to_string(),
        };

        let prompt_tokens = tokenizer.encode(&full_prompt, true);
        let prompt_len    = prompt_tokens.len();

        // Initialize KV caches
        let cfg = &self.weights.config;
        let mut kv_caches: Vec<KvCache> = (0..cfg.num_layers)
            .map(|_| KvCache::new(cfg.context_length, cfg.num_kv_heads, cfg.head_dim()))
            .collect();

        let mut sampler = TokenSampler::new(config.sampler.clone());
        let mut generated_ids: Vec<u32> = Vec::new();
        let mut all_tokens = prompt_tokens.clone();

        // Process prompt tokens (prefill)
        let mut logits = vec![0.0f32; cfg.vocab_size];
        for (pos, &tok_id) in prompt_tokens.iter().enumerate() {
            logits = self.forward_single(tok_id, pos, &mut kv_caches);
        }

        // Autoregressive generation
        let mut stopped_by_eos = false;
        for step in 0..config.max_new_tokens {
            let next_tok = sampler.sample(&logits);
            generated_ids.push(next_tok);
            all_tokens.push(next_tok);

            if config.stop_at_eos && tokenizer.is_eos(next_tok) {
                stopped_by_eos = true;
                break;
            }

            let pos = prompt_len + step;
            if pos >= cfg.context_length { break; }

            logits = self.forward_single(next_tok, pos, &mut kv_caches);
        }

        let generated_text = tokenizer.decode(&generated_ids);
        InferenceResult {
            generated_text,
            tokens_generated: generated_ids.len(),
            tokens_prompt:    prompt_len,
            stopped_by_eos,
        }
    }

    /// Forward pass for a single token at position `pos`.
    fn forward_single(&self, token_id: u32, pos: usize, kv_caches: &mut Vec<KvCache>) -> Vec<f32> {
        let cfg = &self.weights.config;
        let dim = cfg.embedding_dim;
        let head_dim = cfg.head_dim();
        let kv_dim = cfg.num_kv_heads * head_dim;

        // Token embedding lookup
        let emb_start = token_id as usize * dim;
        let mut x: Vec<f32> = self.weights.token_emb[emb_start..emb_start + dim].to_vec();

        // RoPE positional encoding frequencies
        let rope_freqs = compute_rope_freqs(head_dim, cfg.rope_theta);

        // Process each transformer layer
        for (layer_idx, layer) in self.weights.layers.iter().enumerate() {
            // 1. Attention pre-norm
            let x_norm = rms_norm(&x, &layer.attn_norm, cfg.rms_norm_eps);

            // 2. QKV projections
            let q = matmul_f32(&layer.wq, &x_norm, dim, dim);
            let k = matmul_f32(&layer.wk, &x_norm, kv_dim, dim);
            let v = matmul_f32(&layer.wv, &x_norm, kv_dim, dim);

            // 3. Apply RoPE to Q and K
            let q_rope = apply_rope(&q, pos, cfg.num_heads, head_dim, &rope_freqs);
            let k_rope = apply_rope(&k, pos, cfg.num_kv_heads, head_dim, &rope_freqs);

            // 4. Update KV cache
            kv_caches[layer_idx].append_kv(&k_rope, &v, cfg.num_kv_heads, head_dim);

            // 5. Multi-head attention
            let attn_out = grouped_query_attention(
                &q_rope, &kv_caches[layer_idx],
                cfg.num_heads, cfg.num_kv_heads, head_dim,
            );

            // 6. Output projection + residual
            let attn_proj = matmul_f32(&layer.wo, &attn_out, dim, dim);
            for (xi, ai) in x.iter_mut().zip(attn_proj.iter()) { *xi += ai; }

            // 7. FFN pre-norm
            let x_norm2 = rms_norm(&x, &layer.ffn_norm, cfg.rms_norm_eps);

            // 8. SwiGLU FFN: down(silu(gate(x)) * up(x))
            let gate_out = matmul_f32(&layer.w_gate, &x_norm2, cfg.feed_forward_dim, dim);
            let up_out   = matmul_f32(&layer.w_up,   &x_norm2, cfg.feed_forward_dim, dim);
            let swiglu: Vec<f32> = gate_out.iter().zip(up_out.iter())
                .map(|(g, u)| silu(*g) * u)
                .collect();
            let ffn_out = matmul_f32(&layer.w_down, &swiglu, dim, cfg.feed_forward_dim);

            // 9. FFN residual
            for (xi, fi) in x.iter_mut().zip(ffn_out.iter()) { *xi += fi; }
        }

        // Final norm
        let x_final = rms_norm(&x, &self.weights.norm_final, cfg.rms_norm_eps);

        // LM head → logits
        let logits = matmul_f32(&self.weights.lm_head, &x_final, cfg.vocab_size, dim);

        // Temperature is handled in sampler — return raw logits
        logits
    }
}

/// Compute RoPE frequency table for a given head dimension and theta.
fn compute_rope_freqs(head_dim: usize, theta: f32) -> Vec<f32> {
    (0..head_dim/2)
        .map(|i| 1.0 / theta.powf(2.0 * i as f32 / head_dim as f32))
        .collect()
}

/// Apply Rotary Position Encoding to Q or K vectors.
fn apply_rope(x: &[f32], pos: usize, num_heads: usize, head_dim: usize, freqs: &[f32]) -> Vec<f32> {
    let mut out = x.to_vec();
    for head in 0..num_heads {
        let base = head * head_dim;
        for i in 0..head_dim/2 {
            let freq  = freqs[i];
            let angle = pos as f32 * freq;
            let cos   = angle.cos();
            let sin   = angle.sin();
            let x0    = out[base + i];
            let x1    = out[base + i + head_dim/2];
            out[base + i]             = x0 * cos - x1 * sin;
            out[base + i + head_dim/2] = x0 * sin + x1 * cos;
        }
    }
    out
}

/// Grouped Query Attention (GQA) forward pass.
fn grouped_query_attention(
    q: &[f32],
    kv_cache: &KvCache,
    num_heads: usize,
    num_kv_heads: usize,
    head_dim: usize,
) -> Vec<f32> {
    let dim      = num_heads * head_dim;
    let kv_dim   = num_kv_heads * head_dim;
    let seq_len  = kv_cache.seq_len;
    let gqa_factor = num_heads / num_kv_heads;
    let scale    = (head_dim as f32).sqrt().recip();

    let mut out = vec![0.0f32; dim];

    for h in 0..num_heads {
        let kv_head = h / gqa_factor;
        let q_base  = h * head_dim;
        let q_vec   = &q[q_base..q_base + head_dim];

        // Compute attention scores over all cached positions
        let mut scores: Vec<f32> = (0..seq_len).map(|t| {
            let k_base = t * kv_dim + kv_head * head_dim;
            let k_vec  = &kv_cache.k[k_base..k_base + head_dim];
            q_vec.iter().zip(k_vec.iter()).map(|(qi, ki)| qi * ki).sum::<f32>() * scale
        }).collect();

        softmax_inplace(&mut scores);

        // Weighted sum of values
        let out_base = h * head_dim;
        for t in 0..seq_len {
            let v_base = t * kv_dim + kv_head * head_dim;
            let v_vec  = &kv_cache.v[v_base..v_base + head_dim];
            let w      = scores[t];
            for i in 0..head_dim {
                out[out_base + i] += w * v_vec[i];
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rope_freqs_decreasing() {
        let freqs = compute_rope_freqs(128, 10000.0);
        assert_eq!(freqs.len(), 64);
        // Frequencies should be strictly decreasing
        for i in 0..freqs.len()-1 {
            assert!(freqs[i] > freqs[i+1], "freqs not decreasing at {i}");
        }
    }

    #[test]
    fn apply_rope_preserves_norm() {
        let x = vec![1.0f32; 128];
        let freqs = compute_rope_freqs(128, 10000.0);
        let x_rope = apply_rope(&x, 0, 1, 128, &freqs);
        // At position 0, angle=0, cos=1, sin=0 → no change
        let norm_before: f32 = x.iter().map(|v| v*v).sum::<f32>().sqrt();
        let norm_after: f32  = x_rope.iter().map(|v| v*v).sum::<f32>().sqrt();
        assert!((norm_before - norm_after).abs() < 1e-4,
            "RoPE changed norm: {norm_before} → {norm_after}");
    }

    #[test]
    fn inference_config_default_has_system_prompt() {
        let cfg = InferenceConfig::default();
        assert!(cfg.system_prompt.is_some());
        assert!(cfg.system_prompt.unwrap().contains("EnkiduLLM"));
    }

    #[test]
    fn kv_cache_append_increments_seq_len() {
        let mut cache = KvCache::new(16, 4, 32);
        assert_eq!(cache.seq_len, 0);
        let k = vec![0.0f32; 4 * 32];
        let v = vec![0.0f32; 4 * 32];
        cache.append_kv(&k, &v, 4, 32);
        assert_eq!(cache.seq_len, 1);
    }
}
