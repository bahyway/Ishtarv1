//! zikru-embed — Sovereign concept embedder for EnkiduLLM.
//!
//! "The model is a KAKI. Its weights are Orbits. Its training is a Journal."
//! "No tensor flows from outside. No gradient descends from the cloud."
//!
//! Architecture:
//!   TribalTokenizer (enkidullm-ingest) → TokenParticles → TribalFieldAttention
//!   → HeptaSectorPool → SectorEmbedding[7 × dim + unified]
//!
//! Weights: native int8-quantized QuantizedMatrix — no PyTorch, no candle, no ort.
//! Training: contrastive loss + Zikru-Momentum (tribe-aware SGD with momentum).

#![forbid(unsafe_code)]

pub mod attention;
pub mod matrix;
pub mod model_kaki;
pub mod pooling;
pub mod trainer;

pub use attention::{cosine_distance, euclidean_distance, tribal_field_attend, TokenParticle};
pub use matrix::QuantizedMatrix;
pub use model_kaki::{ModelOrbit, ZikruEmbedModel, MODEL_TRIBE_ID};
pub use pooling::{pool_sectors, SectorEmbedding, DEFAULT_FUSION_WEIGHTS, NUM_SECTORS};
pub use trainer::{compute_contrastive_loss, train_epoch, EpochMetrics, TrainingSample};
