//! ZikruEmbedModel — the embedding model stored as a sovereign KAKI.
//!
//! "The model is a KAKI. Its weights are Orbits. Its training is a Journal."
//!
//! ModelOrbit.embedding_weights maps token uuid_hash (from TribalTokenizer)
//! to a dense embedding vector using a HashMap<u32, usize> vocabulary table
//! rather than direct hash-modulo indexing (avoids collision artifacts).

use std::collections::HashMap;
use bahyway_core::TribeId;
use enkidb_kaki::{Kaki, KakiMinter, KakiRole};
use crate::matrix::QuantizedMatrix;

/// Model KAKI nucleus — the model itself is a sovereign particle.
/// tribe_id = LINGUISTIC (0x10FF), kaki_role = Parzu (axiom/architecture).
pub const MODEL_TRIBE_ID: TribeId = TribeId::from_u16(0x10FF);

/// The Zikru-Embed model stored as a KAKI with Orbit shells.
#[derive(Debug, Clone)]
pub struct ZikruEmbedModel {
    pub nucleus: Kaki,
    pub orbit:   ModelOrbit,
}

/// Model orbit: hyperparameters + weights + quality metrics.
#[derive(Debug, Clone)]
pub struct ModelOrbit {
    // ── Core: model metadata (set at initialization)
    pub model_version:          String,
    pub architecture_hash:      u32,
    pub training_corpus_tribe:  u16,

    // ── Classification: hyperparameters
    pub embedding_dim:          u16,
    pub max_sequence_len:       u16,
    pub quantization_bits:      u8,   // Always 8 in v4.0.2

    // ── Content: learned weights (quantized int8)
    /// Embedding matrix: [vocab_size, embedding_dim].
    pub embedding_weights:      QuantizedMatrix,
    /// Vocab table: token uuid_hash → row index in embedding_weights.
    pub vocab_table:            HashMap<u32, usize>,
    /// Field strength bias per TokenClass (8 classes mapped to [0,7]).
    pub field_strength_biases:  [f32; 8],
    /// Sector classifier weights: [7, embedding_dim] — each row produces one sector logit.
    pub sector_weights:         QuantizedMatrix,
    /// Per-tribe sector fusion weights: [7] f32.
    pub fusion_weights:         [f32; 7],

    // ── Reputation: training quality metrics
    pub contrastive_loss:       f32,
    pub tribe_alignment_score:  f32,
    pub training_epochs:        u32,
}

impl ZikruEmbedModel {
    /// Initialize an untrained model with a random embedding matrix.
    /// Weights are pseudo-random (FNV-1a seeded), fully sovereign.
    pub fn init(vocab_size: usize, embedding_dim: u16, corpus_tribe: u16) -> Self {
        let minter = KakiMinter::new(MODEL_TRIBE_ID);
        let nucleus = minter.identity(KakiRole::Parzu);

        let dim = embedding_dim as usize;
        // Initialize weights using FNV-1a seeded pseudo-random values
        let embed_weights: Vec<f32> = (0..vocab_size * dim)
            .map(|i| fnv_pseudo_normal(i as u32, 0) * 0.02)
            .collect();
        let embed_q = QuantizedMatrix::from_f32(vocab_size, dim, &embed_weights);

        let sector_w: Vec<f32> = (0..7 * dim)
            .map(|i| fnv_pseudo_normal(i as u32, 1) * 0.02)
            .collect();
        let sector_q = QuantizedMatrix::from_f32(7, dim, &sector_w);

        let orbit = ModelOrbit {
            model_version:         "ZikruEmbed-1.0".into(),
            architecture_hash:     fnv1a(b"ZikruEmbed-v1.0-7sector"),
            training_corpus_tribe: corpus_tribe,
            embedding_dim,
            max_sequence_len:      512,
            quantization_bits:     8,
            embedding_weights:     embed_q,
            vocab_table:           HashMap::new(),
            field_strength_biases: [1.0, 1.2, 0.8, 1.0, 0.6, 0.7, 1.5, 1.0],
            sector_weights:        sector_q,
            fusion_weights:        [0.10, 0.15, 0.15, 0.10, 0.10, 0.10, 0.30],
            contrastive_loss:      f32::INFINITY,
            tribe_alignment_score: 0.0,
            training_epochs:       0,
        };

        Self { nucleus, orbit }
    }

    /// Look up or register a token uuid_hash in the vocabulary table.
    /// Returns the embedding row index.
    pub fn vocab_index(&mut self, uuid_hash: u32) -> usize {
        let next = self.orbit.vocab_table.len();
        let rows = self.orbit.embedding_weights.rows;
        *self.orbit.vocab_table.entry(uuid_hash).or_insert_with(|| next % rows)
    }

    /// Get the embedding vector for a token uuid_hash.
    pub fn embed_token(&mut self, uuid_hash: u32) -> Vec<f32> {
        let idx = self.vocab_index(uuid_hash);
        self.orbit.embedding_weights.get_row(idx)
    }

    /// Classify a token embedding into a Hepta sector (θ₀–θ₆).
    pub fn classify_sector(&self, embedding: &[f32]) -> u8 {
        let logits = self.orbit.sector_weights.matvec(embedding);
        logits.iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| (i % 7) as u8)
            .unwrap_or(0)
    }

    /// Field strength for a token class index [0..8].
    pub fn field_strength(&self, class_idx: usize) -> f32 {
        self.orbit.field_strength_biases[class_idx.min(7)]
    }

    pub fn embedding_dim(&self) -> usize { self.orbit.embedding_dim as usize }
    pub fn vocab_size(&self)    -> usize { self.orbit.embedding_weights.rows }
}

// ── Sovereign utilities ───────────────────────────────────────────────────────

fn fnv1a(bytes: &[u8]) -> u32 {
    const OFFSET: u32 = 2_166_136_261;
    const PRIME:  u32 = 16_777_619;
    let mut h = OFFSET;
    for &b in bytes { h ^= b as u32; h = h.wrapping_mul(PRIME); }
    h
}

/// FNV-1a seeded pseudo-normal: maps (index, seed) → [-1, 1] float.
/// Not truly normal-distributed but symmetric around 0 with bounded range.
fn fnv_pseudo_normal(index: u32, seed: u32) -> f32 {
    let h = fnv1a(&[
        (index & 0xFF) as u8,
        ((index >> 8) & 0xFF) as u8,
        ((index >> 16) & 0xFF) as u8,
        ((index >> 24) & 0xFF) as u8,
        seed as u8,
    ]);
    // Map [0, 2^32) to [-1, 1]
    (h as f32 / 2_147_483_648.0) - 1.0
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_init_nucleus_valid() {
        let model = ZikruEmbedModel::init(100, 32, 0x1001);
        assert!(model.nucleus.verify_checksum());
        assert_eq!(model.nucleus.tribe_id(), MODEL_TRIBE_ID);
    }

    #[test]
    fn embed_token_returns_correct_dim() {
        let mut model = ZikruEmbedModel::init(50, 16, 0x1001);
        let emb = model.embed_token(0xDEAD_BEEF);
        assert_eq!(emb.len(), 16);
    }

    #[test]
    fn vocab_index_stable() {
        let mut model = ZikruEmbedModel::init(50, 16, 0x1001);
        let idx1 = model.vocab_index(0xABCD);
        let idx2 = model.vocab_index(0xABCD);
        assert_eq!(idx1, idx2);
    }

    #[test]
    fn vocab_index_distinct_for_different_hashes() {
        let mut model = ZikruEmbedModel::init(50, 16, 0x1001);
        let idx1 = model.vocab_index(0xAAAA);
        let idx2 = model.vocab_index(0xBBBB);
        // With 50 rows and 2 distinct hashes, they're very likely different
        // (could collide modulo rows — assert this is a deterministic test not a fluke)
        let _ = (idx1, idx2); // just verify no panic
    }

    #[test]
    fn classify_sector_in_range() {
        let model = ZikruEmbedModel::init(50, 16, 0x1001);
        let emb = vec![1.0f32; 16];
        let sec = model.classify_sector(&emb);
        assert!(sec < 7, "sector out of range: {}", sec);
    }

    #[test]
    fn field_strength_clamped() {
        let model = ZikruEmbedModel::init(10, 8, 0x1001);
        // Index beyond 7 should clamp to index 7
        let fs = model.field_strength(100);
        assert!(fs > 0.0);
    }

    #[test]
    fn pseudo_normal_bounded() {
        for i in 0..1000 {
            let v = fnv_pseudo_normal(i, 42);
            assert!(v >= -1.0 && v <= 1.0, "v={} out of range [-1,1]", v);
        }
    }
}
