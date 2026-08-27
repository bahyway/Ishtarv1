//! enkidullm-model — Sovereign GGUF model runner for EnkiduLLM.
//!
//! Runs quantized LLM models (GGUF format) entirely in pure Rust.
//! No PyTorch. No candle. No llama-cpp-sys. No external ML dependencies.
//!
//! Architecture:
//!   GgufFile (memory-mapped) → GgufTensor → Q4KMDequant → F32MatMul
//!   → TransformerBlock × N → LogitSampler → TokenStream
//!
//! Supported quantizations: Q4_K_M (primary), Q4_0, Q8_0, F16, F32
//! Supported architectures: LLaMA, Qwen2, Mistral (same transformer core)
//!
//! Model recommendation for GTX 1650 Max-Q (4GB VRAM):
//!   Qwen2.5-Coder-7B-Instruct-Q4_K_M.gguf (~4.0GB)
//!   DeepSeek-Coder-6.7B-Instruct-Q4_K_M.gguf (~3.8GB)
#![forbid(unsafe_code)]

pub mod gguf;
pub mod inference;
pub mod model_config;
pub mod quant;
pub mod sampler;
pub mod tokenizer;

pub use gguf::{GgufError, GgufFile, GgufMetadata, GgufTensor};
pub use inference::{InferenceConfig, InferenceEngine, InferenceResult, StreamToken};
pub use model_config::{ModelArch, ModelConfig, ModelKaki, MODEL_TRIBE_ID};
pub use quant::{dequantize_q4_0, dequantize_q4_k_m, dequantize_q8_0, Quantization};
pub use sampler::{sample_top_p, SamplerConfig, TokenSampler};
pub use tokenizer::{decode, encode, BpeTokenizer, TokenizerError};
