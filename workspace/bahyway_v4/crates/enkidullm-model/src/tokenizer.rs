//! tokenizer.rs — Sovereign BPE tokenizer for EnkiduLLM.
//!
//! Reads vocabulary from GGUF metadata (tokenizer.ggml.tokens + merges).
//! No tiktoken. No HuggingFace tokenizers. Pure Rust.
#![forbid(unsafe_code)]

use std::collections::HashMap;

#[derive(Debug)]
pub enum TokenizerError {
    VocabNotFound,
    MergesNotFound,
    InvalidToken(String),
    UnknownToken(String),
}

impl std::fmt::Display for TokenizerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::VocabNotFound       => write!(f, "Vocabulary not found in GGUF metadata"),
            Self::MergesNotFound      => write!(f, "BPE merges not found in GGUF metadata"),
            Self::InvalidToken(s)     => write!(f, "Invalid token: {s}"),
            Self::UnknownToken(s)     => write!(f, "Unknown token: {s}"),
        }
    }
}

/// Sovereign BPE tokenizer.
pub struct BpeTokenizer {
    /// token_id → token string
    pub vocab:       Vec<String>,
    /// token string → token_id
    pub vocab_index: HashMap<String, u32>,
    /// BPE merge rules: (left, right) → merged
    pub merges:      Vec<(String, String)>,
    /// Special tokens
    pub bos_id:      u32,
    pub eos_id:      u32,
    pub pad_id:      u32,
    pub unk_id:      u32,
}

impl BpeTokenizer {
    /// Build from GGUF metadata vocabulary arrays.
    pub fn from_vocab(tokens: Vec<String>, merges: Vec<String>,
                      bos_id: u32, eos_id: u32) -> Self {
        let mut vocab_index = HashMap::with_capacity(tokens.len());
        for (i, t) in tokens.iter().enumerate() {
            vocab_index.insert(t.clone(), i as u32);
        }
        let merge_pairs = merges.iter().filter_map(|m| {
            let parts: Vec<&str> = m.splitn(2, ' ').collect();
            if parts.len() == 2 {
                Some((parts[0].to_string(), parts[1].to_string()))
            } else { None }
        }).collect();

        let unk_id = vocab_index.get("<unk>").copied().unwrap_or(0);
        let pad_id = vocab_index.get("<pad>").copied().unwrap_or(eos_id);

        Self { vocab: tokens, vocab_index, merges: merge_pairs, bos_id, eos_id, pad_id, unk_id }
    }

    /// Encode a string to token IDs.
    pub fn encode(&self, text: &str, add_bos: bool) -> Vec<u32> {
        let mut tokens = Vec::new();
        if add_bos { tokens.push(self.bos_id); }

        // Split into UTF-8 characters, then apply BPE merges
        let mut chars: Vec<String> = text.chars()
            .map(|c| {
                // GGML uses Ġ (U+0120) to represent leading spaces
                if c == ' ' { "Ġ".to_string() } else { c.to_string() }
            })
            .collect();

        // Apply BPE merges iteratively
        loop {
            let mut best_pos: Option<usize> = None;
            let mut best_rank: Option<usize> = None;

            for i in 0..chars.len().saturating_sub(1) {
                let pair = (&chars[i], &chars[i+1]);
                if let Some(rank) = self.merge_rank(pair.0, pair.1) {
                    if best_rank.is_none() || rank < best_rank.unwrap() {
                        best_rank = Some(rank);
                        best_pos  = Some(i);
                    }
                }
            }

            match best_pos {
                None => break,
                Some(pos) => {
                    let merged = format!("{}{}", chars[pos], chars[pos+1]);
                    chars[pos] = merged;
                    chars.remove(pos + 1);
                }
            }
        }

        // Map to token IDs
        for ch in &chars {
            let id = self.vocab_index.get(ch)
                .copied()
                .unwrap_or(self.unk_id);
            tokens.push(id);
        }
        tokens
    }

    /// Decode token IDs to string.
    pub fn decode(&self, ids: &[u32]) -> String {
        let mut out = String::new();
        for &id in ids {
            if id == self.bos_id || id == self.eos_id { continue; }
            if let Some(tok) = self.vocab.get(id as usize) {
                // Reverse Ġ → space
                out.push_str(&tok.replace('Ġ', " ").replace('Ċ', "\n"));
            }
        }
        out
    }

    /// Check if token is EOS.
    pub fn is_eos(&self, id: u32) -> bool { id == self.eos_id }

    /// Vocabulary size.
    pub fn vocab_size(&self) -> usize { self.vocab.len() }

    fn merge_rank(&self, left: &str, right: &str) -> Option<usize> {
        self.merges.iter().position(|(l, r)| l == left && r == right)
    }
}

/// Convenience encode function.
pub fn encode(tokenizer: &BpeTokenizer, text: &str) -> Vec<u32> {
    tokenizer.encode(text, true)
}

/// Convenience decode function.
pub fn decode(tokenizer: &BpeTokenizer, ids: &[u32]) -> String {
    tokenizer.decode(ids)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_tokenizer() -> BpeTokenizer {
        let vocab = vec![
            "<unk>".to_string(), "<bos>".to_string(), "<eos>".to_string(),
            "h".to_string(), "e".to_string(), "l".to_string(), "o".to_string(),
            "he".to_string(), "hel".to_string(), "hell".to_string(), "hello".to_string(),
        ];
        let merges = vec![
            "h e".to_string(), "he l".to_string(),
            "hel l".to_string(), "hell o".to_string(),
        ];
        BpeTokenizer::from_vocab(vocab, merges, 1, 2)
    }

    #[test]
    fn vocab_size_correct() {
        let tok = make_test_tokenizer();
        assert_eq!(tok.vocab_size(), 11);
    }

    #[test]
    fn bos_eos_ids_correct() {
        let tok = make_test_tokenizer();
        assert_eq!(tok.bos_id, 1);
        assert_eq!(tok.eos_id, 2);
    }

    #[test]
    fn decode_skips_special_tokens() {
        let tok = make_test_tokenizer();
        let decoded = tok.decode(&[1, 2]); // bos + eos
        assert_eq!(decoded, "");
    }

    #[test]
    fn is_eos_correct() {
        let tok = make_test_tokenizer();
        assert!(tok.is_eos(2));
        assert!(!tok.is_eos(1));
    }
}
