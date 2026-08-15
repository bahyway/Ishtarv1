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

pub mod matrix;
pub mod attention;
pub mod pooling;
pub mod model_kaki;
pub mod trainer;

pub use matrix::QuantizedMatrix;
pub use attention::{TokenParticle, tribal_field_attend, cosine_distance, euclidean_distance};
pub use pooling::{SectorEmbedding, pool_sectors, NUM_SECTORS, DEFAULT_FUSION_WEIGHTS};
pub use model_kaki::{ZikruEmbedModel, ModelOrbit, MODEL_TRIBE_ID};
pub use trainer::{TrainingSample, EpochMetrics, compute_contrastive_loss, train_epoch};
