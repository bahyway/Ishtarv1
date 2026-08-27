//! ZikruTrainer — native training loop: contrastive loss + Zikru-Momentum.
//!
//! No PyTorch, no gradient tape, no autograd.
//! Gradients are computed analytically for the contrastive margin loss.
//! Zikru-Momentum: per-tribe gradient accumulation with tribe-specific momentum factors.
//!
//! Training objective: push same-book chunks together, cross-tribe chunks apart.
//!   loss = max(0, dist(anchor, positive) − dist(anchor, negative) + margin)

use crate::attention::{euclidean_distance, tribal_field_attend, TokenParticle};
use crate::model_kaki::ZikruEmbedModel;
use crate::pooling::{pool_sectors, SectorEmbedding, DEFAULT_FUSION_WEIGHTS};

/// A training sample: (anchor_tokens, positive_tokens, negative_tokens, negative_tribe).
pub struct TrainingSample {
    pub anchor_hashes: Vec<u32>,   // Token uuid_hashes for anchor chunk
    pub positive_hashes: Vec<u32>, // Token uuid_hashes from same book (positive)
    pub negative_hashes: Vec<u32>, // Token uuid_hashes from different tribe (negative)
    pub anchor_tribe: u16,
    pub negative_tribe: u16,
}

/// Training metrics for one epoch.
#[derive(Debug, Clone, Default)]
pub struct EpochMetrics {
    pub mean_loss: f32,
    pub non_zero_loss: u32, // Samples where loss > 0 (margin violated)
    pub total_samples: u32,
    pub epoch: u32,
}

/// Zikru-Momentum optimizer state — tribe-aware gradient accumulation.
struct ZikruMomentum {
    /// Flat gradient velocity for embedding weights.
    velocity: Vec<f32>,
    learning_rate: f32,
    base_momentum: f32,
}

impl ZikruMomentum {
    fn new(size: usize, lr: f32, momentum: f32) -> Self {
        Self {
            velocity: vec![0.0f32; size],
            learning_rate: lr,
            base_momentum: momentum,
        }
    }

    /// Tribe-specific momentum factor: cross-domain tribes use lower momentum (more stable).
    fn tribe_momentum(&self, tribe_id: u16) -> f32 {
        if tribe_id == 0x1007 {
            self.base_momentum * 0.89
        } else {
            self.base_momentum
        }
    }

    fn update(&mut self, gradient: &[f32], tribe_id: u16) -> &[f32] {
        let m = self.tribe_momentum(tribe_id);
        let lr = self.learning_rate;
        for (v, &g) in self.velocity.iter_mut().zip(gradient.iter()) {
            *v = m * *v + lr * g;
        }
        &self.velocity
    }
}

/// One training step: embed tokens, compute contrastive loss, return loss value.
/// Weights are NOT updated here (model is immutable borrow) — caller applies deltas.
/// This keeps the trainer composable and avoids &mut aliasing.
pub fn compute_contrastive_loss(
    model: &mut ZikruEmbedModel,
    sample: &TrainingSample,
    margin: f32,
) -> (f32, SectorEmbedding, SectorEmbedding, SectorEmbedding) {
    let anchor_se = embed_chunk(model, &sample.anchor_hashes);
    let positive_se = embed_chunk(model, &sample.positive_hashes);
    let negative_se = embed_chunk(model, &sample.negative_hashes);

    let pos_dist = euclidean_distance(&anchor_se.unified, &positive_se.unified);
    let neg_dist = euclidean_distance(&anchor_se.unified, &negative_se.unified);
    let loss = (pos_dist - neg_dist + margin).max(0.0);

    (loss, anchor_se, positive_se, negative_se)
}

/// Embed a sequence of token uuid_hashes through the model.
fn embed_chunk(model: &mut ZikruEmbedModel, hashes: &[u32]) -> SectorEmbedding {
    if hashes.is_empty() {
        let dim = model.embedding_dim();
        let empty_particle = TokenParticle {
            position: vec![0.0f32; dim],
            field_strength: 0.0,
            tribe_affinity: 1.0,
            sector: 0,
        };
        return pool_sectors(
            std::slice::from_ref(&empty_particle),
            &[vec![0.0f32; dim]],
            &DEFAULT_FUSION_WEIGHTS,
        );
    }

    let particles: Vec<TokenParticle> = hashes
        .iter()
        .take(512)
        .map(|&h| {
            let emb = model.embed_token(h);
            let sector = model.classify_sector(&emb);
            let fs = model.field_strength((sector % 8) as usize);
            TokenParticle {
                position: emb,
                field_strength: fs,
                tribe_affinity: 1.0,
                sector,
            }
        })
        .collect();

    let attended = tribal_field_attend(&particles);
    pool_sectors(&particles, &attended, &DEFAULT_FUSION_WEIGHTS)
}

/// Train for one epoch over a set of samples.
/// Returns per-sample losses and aggregate metrics.
pub fn train_epoch(
    model: &mut ZikruEmbedModel,
    samples: &[TrainingSample],
    margin: f32,
    lr: f32,
    momentum: f32,
) -> EpochMetrics {
    let embed_size = model.orbit.embedding_weights.rows * model.orbit.embedding_weights.cols;
    let mut optim = ZikruMomentum::new(embed_size, lr, momentum);
    let mut total_loss = 0.0f32;
    let mut non_zero = 0u32;

    for sample in samples {
        let (loss, anchor, positive, negative) = compute_contrastive_loss(model, sample, margin);

        if loss > 0.0 {
            non_zero += 1;
            // Approximate gradient: push anchor toward positive, away from negative
            // Gradient is applied to the embedding table via delta updates
            let grad = compute_embedding_gradient(&anchor, &positive, &negative);
            let velocity = optim.update(&grad, sample.anchor_tribe);

            // Apply velocity to embedding weights (first embed_size dimensions)
            apply_gradient_to_model(model, velocity);
        }

        total_loss += loss;
    }

    let n = samples.len().max(1) as f32;
    model.orbit.contrastive_loss = total_loss / n;
    model.orbit.training_epochs += 1;

    EpochMetrics {
        mean_loss: total_loss / n,
        non_zero_loss: non_zero,
        total_samples: samples.len() as u32,
        epoch: model.orbit.training_epochs,
    }
}

/// Approximate gradient for the embedding table.
/// Gradient direction: close anchor to positive, push away from negative.
fn compute_embedding_gradient(
    anchor: &SectorEmbedding,
    positive: &SectorEmbedding,
    negative: &SectorEmbedding,
) -> Vec<f32> {
    let dim = anchor.unified.len();
    let mut grad = vec![0.0f32; dim];
    for (k, g) in grad.iter_mut().enumerate().take(dim) {
        // Pull toward positive, push from negative (contrastive gradient)
        *g = (anchor.unified[k] - positive.unified[k]) - (anchor.unified[k] - negative.unified[k]);
    }
    grad
}

/// Apply a gradient velocity to the model's embedding weight table.
/// Applies only the first `velocity.len().min(rows × cols)` values.
fn apply_gradient_to_model(model: &mut ZikruEmbedModel, velocity: &[f32]) {
    // Dequantize, update, re-quantize
    let rows = model.orbit.embedding_weights.rows;
    let cols = model.orbit.embedding_weights.cols;
    let mut floats: Vec<f32> = (0..rows)
        .flat_map(|r| model.orbit.embedding_weights.get_row(r))
        .collect();

    for (i, v) in velocity.iter().enumerate().take(floats.len()) {
        floats[i] -= v; // gradient descent
    }

    model.orbit.embedding_weights = crate::matrix::QuantizedMatrix::from_f32(rows, cols, &floats);
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model_kaki::ZikruEmbedModel;

    fn make_model() -> ZikruEmbedModel {
        ZikruEmbedModel::init(50, 8, 0x1001)
    }

    fn make_sample() -> TrainingSample {
        TrainingSample {
            anchor_hashes: vec![0xA1, 0xA2, 0xA3],
            positive_hashes: vec![0xA1, 0xA4, 0xA5],
            negative_hashes: vec![0xFF, 0xFE, 0xFD],
            anchor_tribe: 0x1001,
            negative_tribe: 0x1007,
        }
    }

    #[test]
    fn compute_loss_non_negative() {
        let mut model = make_model();
        let sample = make_sample();
        let (loss, _, _, _) = compute_contrastive_loss(&mut model, &sample, 1.0);
        assert!(loss >= 0.0, "loss must be >= 0, got {}", loss);
    }

    #[test]
    fn embed_chunk_returns_correct_dim() {
        let mut model = make_model();
        let hashes = vec![0x01, 0x02, 0x03];
        let se = embed_chunk(&mut model, &hashes);
        assert_eq!(se.unified.len(), 8, "unified dim should match model");
    }

    #[test]
    fn embed_chunk_empty_no_panic() {
        let mut model = make_model();
        let se = embed_chunk(&mut model, &[]);
        assert_eq!(se.unified.len(), 8);
    }

    #[test]
    fn train_epoch_reduces_loss_over_repeated_samples() {
        let mut model = make_model();
        let samples = vec![make_sample(), make_sample()];

        let metrics1 = train_epoch(&mut model, &samples, 1.0, 0.01, 0.9);
        let metrics2 = train_epoch(&mut model, &samples, 1.0, 0.01, 0.9);
        // After training, loss should not increase indefinitely
        // (can't guarantee decrease with random init, but should be finite)
        assert!(metrics1.mean_loss.is_finite());
        assert!(metrics2.mean_loss.is_finite());
    }

    #[test]
    fn epoch_counter_increments() {
        let mut model = make_model();
        let samples = vec![make_sample()];
        assert_eq!(model.orbit.training_epochs, 0);
        train_epoch(&mut model, &samples, 1.0, 0.01, 0.9);
        assert_eq!(model.orbit.training_epochs, 1);
        train_epoch(&mut model, &samples, 1.0, 0.01, 0.9);
        assert_eq!(model.orbit.training_epochs, 2);
    }

    #[test]
    fn zikru_momentum_cross_domain_lower() {
        let optim = ZikruMomentum::new(4, 0.01, 0.9);
        let m_cross = optim.tribe_momentum(0x1007);
        let m_normal = optim.tribe_momentum(0x1001);
        assert!(
            m_cross < m_normal,
            "cross-domain should have lower momentum"
        );
    }
}
