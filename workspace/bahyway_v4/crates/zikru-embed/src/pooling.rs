//! HeptaSectorPool — 7-sector concept pooling for Zikru-Embed.
//!
//! Each of the 7 Hepta sectors captures one semantic dimension:
//!   θ₀ syntax/structure    θ₁ entities/concepts   θ₂ relationships
//!   θ₃ numbers/metrics     θ₄ temporal            θ₅ spatial
//!   θ₆ abstract/conceptual
//!
//! Tokens are assigned to sectors based on their TokenClass (from enkidullm-ingest).
//! Sector embeddings are averaged per sector, then fused into a unified vector.

use crate::attention::TokenParticle;

pub const NUM_SECTORS: usize = 7;

/// Default fusion weights: θ₆ (abstract) is weighted highest for book content.
pub const DEFAULT_FUSION_WEIGHTS: [f32; NUM_SECTORS] = [0.10, 0.15, 0.15, 0.10, 0.10, 0.10, 0.30];

/// Output of the 7-sector pooling operation.
#[derive(Debug, Clone)]
pub struct SectorEmbedding {
    /// Per-sector averaged embedding: [7 × dim].
    pub sectors: [Vec<f32>; NUM_SECTORS],
    /// Weighted fusion of all sectors: [dim].
    pub unified: Vec<f32>,
    /// Number of tokens pooled into each sector.
    pub sector_counts: [u32; NUM_SECTORS],
}

impl SectorEmbedding {
    /// Cosine similarity between two unified embeddings, in [−1, 1].
    pub fn cosine_similarity(&self, other: &SectorEmbedding) -> f32 {
        let dot: f32 = self
            .unified
            .iter()
            .zip(&other.unified)
            .map(|(a, b)| a * b)
            .sum();
        let na: f32 = self.unified.iter().map(|x| x * x).sum::<f32>().sqrt();
        let nb: f32 = other.unified.iter().map(|x| x * x).sum::<f32>().sqrt();
        dot / (na * nb + 1e-8)
    }

    /// Sector-level cosine similarity for sector s (used by plagiarism probe).
    pub fn sector_similarity(&self, other: &SectorEmbedding, sector: usize) -> f32 {
        let a = &self.sectors[sector];
        let b = &other.sectors[sector];
        if a.is_empty() || b.is_empty() {
            return 0.0;
        }
        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let na: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let nb: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        dot / (na * nb + 1e-8)
    }

    /// Abstract sector (θ₆) similarity — the deepest plagiarism fingerprint.
    pub fn abstract_similarity(&self, other: &SectorEmbedding) -> f32 {
        self.sector_similarity(other, 6)
    }
}

/// Pool a sequence of attended token embeddings into a SectorEmbedding.
pub fn pool_sectors(
    particles: &[TokenParticle],
    attended: &[Vec<f32>],
    fusion_weights: &[f32; NUM_SECTORS],
) -> SectorEmbedding {
    assert_eq!(particles.len(), attended.len());

    let dim = attended.first().map(|v| v.len()).unwrap_or(0);
    let mut sectors: [Vec<f32>; NUM_SECTORS] = std::array::from_fn(|_| vec![0.0f32; dim]);
    let mut sector_counts: [u32; NUM_SECTORS] = [0; NUM_SECTORS];

    for (particle, attn) in particles.iter().zip(attended.iter()) {
        let s = (particle.sector as usize) % NUM_SECTORS;
        for k in 0..dim {
            sectors[s][k] += attn[k];
        }
        sector_counts[s] += 1;
    }

    // Average per sector (mean pooling)
    for s in 0..NUM_SECTORS {
        if sector_counts[s] > 0 {
            let n = sector_counts[s] as f32;
            for x in sectors[s].iter_mut().take(dim) {
                *x /= n;
            }
        }
    }

    // Weighted fusion
    let unified = fuse_sectors(&sectors, fusion_weights, dim);

    SectorEmbedding {
        sectors,
        unified,
        sector_counts,
    }
}

fn fuse_sectors(
    sectors: &[Vec<f32>; NUM_SECTORS],
    weights: &[f32; NUM_SECTORS],
    dim: usize,
) -> Vec<f32> {
    let mut unified = vec![0.0f32; dim];
    let mut weight_sum = 0.0f32;
    for s in 0..NUM_SECTORS {
        if !sectors[s].is_empty() {
            let w = weights[s];
            for k in 0..dim {
                unified[k] += w * sectors[s][k];
            }
            weight_sum += w;
        }
    }
    if weight_sum > 0.0 {
        let inv = weight_sum.recip();
        for x in unified.iter_mut().take(dim) {
            *x *= inv;
        }
    }
    unified
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attention::TokenParticle;

    fn make_particles(sectors: &[u8], dim: usize) -> Vec<TokenParticle> {
        sectors
            .iter()
            .map(|&s| TokenParticle {
                position: vec![1.0f32; dim],
                field_strength: 1.0,
                tribe_affinity: 1.0,
                sector: s,
            })
            .collect()
    }

    fn make_attended(n: usize, dim: usize, val: f32) -> Vec<Vec<f32>> {
        vec![vec![val; dim]; n]
    }

    #[test]
    fn pool_empty_sequence_returns_zero_embedding() {
        let se = pool_sectors(&[], &[], &DEFAULT_FUSION_WEIGHTS);
        assert!(se.unified.is_empty());
    }

    #[test]
    fn sector_counts_correct() {
        let particles = make_particles(&[0, 0, 1, 6, 6, 6], 4);
        let attended = make_attended(6, 4, 1.0);
        let se = pool_sectors(&particles, &attended, &DEFAULT_FUSION_WEIGHTS);
        assert_eq!(se.sector_counts[0], 2);
        assert_eq!(se.sector_counts[1], 1);
        assert_eq!(se.sector_counts[6], 3);
    }

    #[test]
    fn unified_embedding_non_zero_when_tokens_present() {
        let particles = make_particles(&[0, 1, 2], 8);
        let attended = make_attended(3, 8, 0.5);
        let se = pool_sectors(&particles, &attended, &DEFAULT_FUSION_WEIGHTS);
        assert!(se.unified.iter().any(|&v| v > 0.0));
    }

    #[test]
    fn cosine_similarity_self_is_one() {
        let particles = make_particles(&[0, 3, 6], 16);
        let attended = make_attended(3, 16, 1.0);
        let se = pool_sectors(&particles, &attended, &DEFAULT_FUSION_WEIGHTS);
        let sim = se.cosine_similarity(&se);
        assert!((sim - 1.0).abs() < 1e-5, "self-similarity = {}", sim);
    }

    #[test]
    fn sector_sector_wraps_to_valid_index() {
        // sector = 9 should map to 9 % 7 = 2
        let p = make_particles(&[9], 4);
        let a = make_attended(1, 4, 1.0);
        let se = pool_sectors(&p, &a, &DEFAULT_FUSION_WEIGHTS);
        assert_eq!(se.sector_counts[2], 1);
    }

    #[test]
    fn abstract_sector_similarity_identical() {
        let particles = make_particles(&[6], 8);
        let attended = make_attended(1, 8, 1.0);
        let se = pool_sectors(&particles, &attended, &DEFAULT_FUSION_WEIGHTS);
        let sim = se.abstract_similarity(&se);
        assert!((sim - 1.0).abs() < 1e-5);
    }

    #[test]
    fn fusion_weights_affect_unified() {
        // Weights that emphasize sector 0
        let w_s0: [f32; 7] = [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        // Weights that emphasize sector 6
        let w_s6: [f32; 7] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0];

        let particles = make_particles(&[0, 6], 4);
        let attended = vec![
            vec![1.0f32, 0.0, 0.0, 0.0], // sector 0 embedding
            vec![0.0f32, 0.0, 0.0, 1.0], // sector 6 embedding
        ];

        let se_s0 = pool_sectors(&particles, &attended, &w_s0);
        let se_s6 = pool_sectors(&particles, &attended, &w_s6);

        // With s0 weights: unified[0] should be high
        assert!(
            se_s0.unified[0] > se_s0.unified[3],
            "sector 0 should dominate"
        );
        // With s6 weights: unified[3] should be high
        assert!(
            se_s6.unified[3] > se_s6.unified[0],
            "sector 6 should dominate"
        );
    }
}
