//! TribalFieldAttention — sovereign attention via Particles Algebra.
//!
//! NOT multi-head self-attention (no Q/K/V matrices, no softmax).
//! Instead: each token emits a field; neighbors interact via inverse-square
//! field strength weighted by sector compatibility.
//!
//! This grounds the model in Triple-O: tokens are particles, field lines
//! are the EAV relationships, sector alignment is the Hepta spatial index.

/// A token particle in the embedding space.
#[derive(Debug, Clone)]
pub struct TokenParticle {
    /// Embedding vector (dim = model.embedding_dim).
    pub position:      Vec<f32>,
    /// Field strength: how strongly this particle influences neighbors.
    /// Set from field_strength_biases indexed by TokenClass.
    pub field_strength: f32,
    /// Tribe affinity — domain-specific relevance weight [0,1].
    pub tribe_affinity: f32,
    /// Hepta sector θ₀–θ₆ for this token.
    pub sector:        u8,
}

/// Compute attended embeddings via Tribal Field Attention.
///
/// For each token i, accumulates the weighted sum of all other tokens j:
///   weight(i,j) = field_j / (cosine_distance(i,j)² + ε) × sector_compat × tribe_j
///
/// Then: output[i] = position[i] + Σ(weight(i,j) × position[j]) / total_weight
pub fn tribal_field_attend(particles: &[TokenParticle]) -> Vec<Vec<f32>> {
    let n = particles.len();
    if n == 0 { return Vec::new(); }
    let dim = particles[0].position.len();
    let mut outputs = vec![vec![0.0f32; dim]; n];

    for i in 0..n {
        let mut accumulator = vec![0.0f32; dim];
        let mut total_weight = 0.0f32;

        for j in 0..n {
            if i == j { continue; }

            let distance = cosine_distance(&particles[i].position, &particles[j].position);
            // Inverse-square field: particles close in semantic space influence more
            let field = particles[j].field_strength / (distance * distance + 1e-6);

            // Same sector = stronger interaction (aligned semantic dimension)
            let sector_compat = if particles[i].sector == particles[j].sector { 1.5f32 } else { 0.5f32 };

            let weight = field * sector_compat * particles[j].tribe_affinity;

            for k in 0..dim {
                accumulator[k] += weight * particles[j].position[k];
            }
            total_weight += weight;
        }

        // Residual connection: output = input + attended
        let norm = (total_weight + 1e-6).recip();
        for k in 0..dim {
            outputs[i][k] = particles[i].position[k] + accumulator[k] * norm;
        }
    }

    outputs
}

/// Cosine distance: 1 − cosine_similarity. Returns [0,1] for non-negative vectors,
/// [0,2] in general.
pub fn cosine_distance(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len());
    let dot:    f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    1.0 - (dot / (norm_a * norm_b + 1e-8))
}

/// Euclidean distance between two vectors.
pub fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len());
    a.iter().zip(b.iter()).map(|(x, y)| (x - y) * (x - y)).sum::<f32>().sqrt()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_particle(pos: Vec<f32>, sector: u8) -> TokenParticle {
        TokenParticle { position: pos, field_strength: 1.0, tribe_affinity: 1.0, sector }
    }

    #[test]
    fn attend_two_identical_sector_particles() {
        let a = make_particle(vec![1.0, 0.0], 0);
        let b = make_particle(vec![0.0, 1.0], 0); // same sector → 1.5 compat
        let out = tribal_field_attend(&[a, b]);
        assert_eq!(out.len(), 2);
        // Residual: each output has the input embedded in it
        assert!(out[0][0] > 0.5, "residual component should be present");
    }

    #[test]
    fn attend_empty_no_panic() {
        let out = tribal_field_attend(&[]);
        assert!(out.is_empty());
    }

    #[test]
    fn attend_single_particle_unchanged() {
        let p = make_particle(vec![1.0, 2.0, 3.0], 1);
        let out = tribal_field_attend(&[p.clone()]);
        // Single particle: no neighbors, output = position (residual only)
        assert!((out[0][0] - 1.0).abs() < 1e-5);
        assert!((out[0][1] - 2.0).abs() < 1e-5);
        assert!((out[0][2] - 3.0).abs() < 1e-5);
    }

    #[test]
    fn cosine_distance_identical_vectors() {
        let v = vec![1.0f32, 2.0, 3.0];
        let d = cosine_distance(&v, &v);
        assert!(d.abs() < 1e-5, "identical vectors have distance 0, got {}", d);
    }

    #[test]
    fn cosine_distance_orthogonal_vectors() {
        let a = vec![1.0f32, 0.0];
        let b = vec![0.0f32, 1.0];
        let d = cosine_distance(&a, &b);
        assert!((d - 1.0).abs() < 1e-5, "orthogonal vectors have distance 1, got {}", d);
    }

    #[test]
    fn euclidean_distance_known() {
        let a = vec![0.0f32, 0.0];
        let b = vec![3.0f32, 4.0];
        let d = euclidean_distance(&a, &b);
        assert!((d - 5.0).abs() < 1e-5, "expected 5.0, got {}", d);
    }

    #[test]
    fn field_strength_affects_output() {
        // Higher field strength on particle B → B influences A more
        let a  = make_particle(vec![1.0, 0.0], 0);
        let mut b_high = make_particle(vec![0.0, 1.0], 0);
        b_high.field_strength = 10.0;
        let b_low  = make_particle(vec![0.0, 1.0], 0); // field=1.0

        let out_high = tribal_field_attend(&[a.clone(), b_high]);
        let out_low  = tribal_field_attend(&[a.clone(), b_low]);

        // High-field B should pull A's output[1] higher
        assert!(out_high[0][1] > out_low[0][1],
                "high field: {}, low field: {}", out_high[0][1], out_low[0][1]);
    }
}
