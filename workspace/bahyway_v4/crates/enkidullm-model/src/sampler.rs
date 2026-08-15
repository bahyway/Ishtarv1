//! sampler.rs — Sovereign token sampler for EnkiduLLM.
//!
//! Implements top-p (nucleus) sampling and greedy decoding.
//! No external random crate — uses a sovereign LCG PRNG seeded from epoch.
#![forbid(unsafe_code)]

/// Configuration for token sampling.
#[derive(Debug, Clone)]
pub struct SamplerConfig {
    /// Temperature — scales logits before sampling. 0.0 = greedy.
    pub temperature:   f32,
    /// Top-p nucleus sampling threshold (0.0–1.0).
    pub top_p:         f32,
    /// Top-k — only sample from top-k tokens. 0 = disabled.
    pub top_k:         usize,
    /// Repetition penalty — penalize recently seen tokens.
    pub repeat_penalty: f32,
    /// Window for repetition penalty.
    pub repeat_window:  usize,
    /// Random seed.
    pub seed:          u64,
}

impl Default for SamplerConfig {
    fn default() -> Self {
        Self {
            temperature:    0.7,
            top_p:          0.9,
            top_k:          40,
            repeat_penalty: 1.1,
            repeat_window:  64,
            seed:           42,
        }
    }
}

impl SamplerConfig {
    /// Greedy decoding (deterministic).
    pub fn greedy() -> Self {
        Self { temperature: 0.0, top_p: 1.0, top_k: 1, repeat_penalty: 1.0, repeat_window: 0, seed: 0 }
    }
    /// Standard creative sampling.
    pub fn creative() -> Self {
        Self { temperature: 0.8, top_p: 0.95, top_k: 50, ..Default::default() }
    }
    /// Code generation — lower temperature for precision.
    pub fn code() -> Self {
        Self { temperature: 0.2, top_p: 0.9, top_k: 20, repeat_penalty: 1.05, ..Default::default() }
    }
}

/// Sovereign token sampler.
pub struct TokenSampler {
    config: SamplerConfig,
    rng:    LcgRng,
    recent: Vec<u32>,
}

impl TokenSampler {
    pub fn new(config: SamplerConfig) -> Self {
        let seed = config.seed;
        Self { config, rng: LcgRng::new(seed), recent: Vec::new() }
    }

    /// Sample next token from logits.
    pub fn sample(&mut self, logits: &[f32]) -> u32 {
        let mut scores: Vec<(usize, f32)> = logits.iter()
            .cloned()
            .enumerate()
            .collect();

        // Apply repetition penalty
        if self.config.repeat_penalty != 1.0 {
            for (tok_id, score) in scores.iter_mut() {
                if self.recent.contains(&(*tok_id as u32)) {
                    if *score > 0.0 { *score /= self.config.repeat_penalty; }
                    else            { *score *= self.config.repeat_penalty; }
                }
            }
        }

        // Greedy
        if self.config.temperature == 0.0 {
            let tok = scores.iter()
                .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                .map(|(i, _)| *i as u32)
                .unwrap_or(0);
            self.push_recent(tok);
            return tok;
        }

        // Apply temperature
        for (_, s) in scores.iter_mut() { *s /= self.config.temperature; }

        // Top-k filter
        if self.config.top_k > 0 && self.config.top_k < scores.len() {
            scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            scores.truncate(self.config.top_k);
        }

        // Softmax
        let max = scores.iter().map(|(_, s)| *s).fold(f32::NEG_INFINITY, f32::max);
        let mut probs: Vec<(usize, f32)> = scores.iter()
            .map(|(i, s)| (*i, (*s - max).exp()))
            .collect();
        let sum: f32 = probs.iter().map(|(_, p)| p).sum();
        for (_, p) in probs.iter_mut() { *p /= sum; }

        // Top-p nucleus filter
        probs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let tok = sample_top_p(&probs, self.config.top_p, self.rng.next_f32());
        self.push_recent(tok);
        tok
    }

    fn push_recent(&mut self, tok: u32) {
        self.recent.push(tok);
        if self.recent.len() > self.config.repeat_window {
            self.recent.remove(0);
        }
    }
}

/// Top-p nucleus sampling from sorted (token_id, probability) pairs.
pub fn sample_top_p(probs: &[(usize, f32)], top_p: f32, rand: f32) -> u32 {
    let mut cumsum = 0.0f32;
    let mut nucleus: Vec<(usize, f32)> = Vec::new();
    for (i, p) in probs {
        nucleus.push((*i, *p));
        cumsum += p;
        if cumsum >= top_p { break; }
    }
    // Re-normalize
    let total: f32 = nucleus.iter().map(|(_, p)| p).sum();
    let mut r = rand * total;
    for (i, p) in &nucleus {
        r -= p;
        if r <= 0.0 { return *i as u32; }
    }
    nucleus.last().map(|(i, _)| *i as u32).unwrap_or(0)
}

/// Sovereign LCG PRNG — no external rand crate.
struct LcgRng { state: u64 }
impl LcgRng {
    fn new(seed: u64) -> Self { Self { state: seed ^ 0xDEADBEEF_CAFEBABE } }
    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.state
    }
    fn next_f32(&mut self) -> f32 {
        (self.next_u64() >> 11) as f32 / (1u64 << 53) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greedy_picks_max() {
        let mut sampler = TokenSampler::new(SamplerConfig::greedy());
        let logits = vec![0.1f32, 0.5, 9.9, 0.3];
        assert_eq!(sampler.sample(&logits), 2);
    }

    #[test]
    fn top_p_full_distribution() {
        // With top_p=1.0 and rand=0.0, should pick highest prob token
        let probs = vec![(2usize, 0.7f32), (0, 0.2), (1, 0.1)];
        let tok = sample_top_p(&probs, 1.0, 0.0);
        assert_eq!(tok, 2);
    }

    #[test]
    fn lcg_rng_deterministic() {
        let mut r1 = LcgRng::new(42);
        let mut r2 = LcgRng::new(42);
        assert_eq!(r1.next_u64(), r2.next_u64());
    }

    #[test]
    fn lcg_rng_f32_in_range() {
        let mut rng = LcgRng::new(123);
        for _ in 0..1000 {
            let v = rng.next_f32();
            assert!(v >= 0.0 && v < 1.0, "out of range: {v}");
        }
    }

    #[test]
    fn sampler_config_code_low_temp() {
        let cfg = SamplerConfig::code();
        assert!(cfg.temperature < 0.5);
    }
}
