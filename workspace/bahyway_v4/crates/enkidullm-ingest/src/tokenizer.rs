//! TribalTokenizer — native byte-level tokenizer, no BPE, no pretrained vocabulary.
//!
//! Tokens are born as KAKIs in the linguistic tribe (0x10FF).
//! Boundaries: whitespace, camelCase transitions, punctuation, number boundaries.
//! Token identity = FNV-1a hash of normalized byte content.
//!
//! 7 token classes map to the 7 Hepta sectors (θ₀–θ₆):
//!   θ₀ Word/snake_case → syntax structure
//!   θ₁ ProperNoun/ALLCAPS → entities and named concepts
//!   θ₂ Operator/symbol → relationships and actions
//!   θ₃ Number → measurements and quantities
//!   θ₄ Terminal punctuation → temporal boundaries
//!   θ₅ Delimiter → spatial/structural delimiters
//!   θ₆ Quotation/other → abstract/contextual

/// Hepta sector assignment for a token class (θ₀–θ₆).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TokenClass {
    Word       = 0,  // θ₀ — lowercase, snake_case
    ProperNoun = 1,  // θ₁ — CamelCase, ALLCAPS, acronym
    Operator   = 2,  // θ₂ — +−×÷=<>
    Number     = 3,  // θ₃ — digits, decimals
    Terminal   = 4,  // θ₄ — . ! ?
    Delimiter  = 5,  // θ₅ — () [] {} : ;
    Quotation  = 6,  // θ₆ — " ' ` and other
}

impl TokenClass {
    pub fn sector(self) -> u8 { self as u8 }
}

/// A tokenized unit — identity derived from byte content hash.
#[derive(Debug, Clone)]
pub struct TokenUnit {
    /// FNV-1a hash of the normalized token bytes — used as vocabulary index.
    pub uuid_hash:   u32,
    /// Original text content.
    pub text:        String,
    pub class:       TokenClass,
    /// Byte offset in the source text.
    pub byte_offset: usize,
}

/// FNV-1a 32-bit — matches the sovereign pattern in enkidullm-core.
fn fnv1a(bytes: &[u8]) -> u32 {
    const OFFSET: u32 = 2_166_136_261;
    const PRIME:  u32 = 16_777_619;
    let mut h = OFFSET;
    for &b in bytes { h ^= b as u32; h = h.wrapping_mul(PRIME); }
    h
}

/// Normalize a token for vocabulary hashing: lowercase, strip trailing punctuation.
fn normalize(s: &str) -> String {
    s.to_ascii_lowercase()
        .trim_matches(|c: char| !c.is_alphanumeric())
        .to_string()
}

/// Tokenize a text slice into a sequence of `TokenUnit` values.
pub fn tokenize(text: &str) -> Vec<TokenUnit> {
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut tokens = Vec::new();
    let mut i = 0;

    while i < len {
        // Skip whitespace
        while i < len && bytes[i].is_ascii_whitespace() { i += 1; }
        if i >= len { break; }

        let start = i;
        let first = bytes[i];

        let (class, end) = if first.is_ascii_digit() {
            // Number: digits + optional decimal point
            let mut j = i;
            while j < len && (bytes[j].is_ascii_digit() || bytes[j] == b'.') { j += 1; }
            (TokenClass::Number, j)

        } else if first.is_ascii_uppercase() {
            // ProperNoun: ALLCAPS acronym, or CamelCase word
            let mut j = i;
            while j < len && bytes[j].is_ascii_uppercase() { j += 1; }
            // If followed immediately by lowercase, it's CamelCase — continue word
            if j < len && bytes[j].is_ascii_lowercase() {
                while j < len && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_') { j += 1; }
            }
            (TokenClass::ProperNoun, j)

        } else if first.is_ascii_lowercase() || first == b'_' {
            // Word or snake_case
            let mut j = i;
            while j < len && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_') { j += 1; }
            (TokenClass::Word, j)

        } else {
            // Single-character punctuation
            let class = punct_class(first);
            (class, i + 1)
        };

        let raw = &text[start..end];
        if raw.is_empty() { i = end + 1; continue; }
        let norm = normalize(raw);
        let hash = fnv1a(norm.as_bytes());

        tokens.push(TokenUnit {
            uuid_hash:   hash,
            text:        raw.to_string(),
            class,
            byte_offset: start,
        });
        i = end;
    }

    tokens
}

fn punct_class(b: u8) -> TokenClass {
    match b {
        b'.' | b'!' | b'?'                         => TokenClass::Terminal,
        b'(' | b')' | b'[' | b']' | b'{' | b'}' |
        b':' | b';' | b','                         => TokenClass::Delimiter,
        b'+' | b'-' | b'*' | b'/' | b'%' |
        b'=' | b'<' | b'>' | b'&' | b'|'          => TokenClass::Operator,
        b'"' | b'\'' | b'`'                        => TokenClass::Quotation,
        _                                           => TokenClass::Quotation,
    }
}

/// Statistics over a tokenized sequence.
#[derive(Debug, Default)]
pub struct TokenStats {
    pub total:       u32,
    pub words:       u32,
    pub proper_nouns:u32,
    pub numbers:     u32,
    pub operators:   u32,
    pub terminals:   u32,
    pub delimiters:  u32,
    pub quotations:  u32,
    /// Unique token hashes (vocabulary size estimate).
    pub unique:      u32,
}

impl TokenStats {
    pub fn from_tokens(tokens: &[TokenUnit]) -> Self {
        use std::collections::HashSet;
        let mut seen = HashSet::new();
        let mut stats = Self::default();
        for t in tokens {
            stats.total += 1;
            seen.insert(t.uuid_hash);
            match t.class {
                TokenClass::Word       => stats.words += 1,
                TokenClass::ProperNoun => stats.proper_nouns += 1,
                TokenClass::Number     => stats.numbers += 1,
                TokenClass::Operator   => stats.operators += 1,
                TokenClass::Terminal   => stats.terminals += 1,
                TokenClass::Delimiter  => stats.delimiters += 1,
                TokenClass::Quotation  => stats.quotations += 1,
            }
        }
        stats.unique = seen.len() as u32;
        stats
    }

    /// Lexical complexity: ratio of unique tokens to total (0 = repetitive, 1 = fully unique).
    pub fn lexical_complexity(&self) -> f32 {
        if self.total == 0 { 0.0 } else { self.unique as f32 / self.total as f32 }
    }

    /// Technical depth signal: ratio of ProperNouns to total words (named concepts dense = technical).
    pub fn proper_noun_density(&self) -> f32 {
        let denom = (self.words + self.proper_nouns) as f32;
        if denom == 0.0 { 0.0 } else { self.proper_nouns as f32 / denom }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_simple_sentence() {
        let tokens = tokenize("hello world foo");
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].class, TokenClass::Word);
        assert_eq!(tokens[0].text, "hello");
    }

    #[test]
    fn proper_noun_detection() {
        let tokens = tokenize("The CAP Theorem is foundational");
        let pn: Vec<_> = tokens.iter().filter(|t| t.class == TokenClass::ProperNoun).collect();
        // "The", "CAP", "Theorem" should be ProperNoun
        assert!(!pn.is_empty(), "no proper nouns found");
        assert!(pn.iter().any(|t| t.text == "CAP"), "CAP not classified as ProperNoun");
    }

    #[test]
    fn number_detection() {
        let tokens = tokenize("chapter 12 has 3.14 value");
        let nums: Vec<_> = tokens.iter().filter(|t| t.class == TokenClass::Number).collect();
        assert!(nums.iter().any(|t| t.text == "12"));
        assert!(nums.iter().any(|t| t.text == "3.14"));
    }

    #[test]
    fn operator_detection() {
        let tokens = tokenize("a + b = c");
        let ops: Vec<_> = tokens.iter().filter(|t| t.class == TokenClass::Operator).collect();
        assert!(ops.iter().any(|t| t.text == "+"));
        assert!(ops.iter().any(|t| t.text == "="));
    }

    #[test]
    fn hash_stable_for_same_token() {
        let tokens1 = tokenize("distributed_systems");
        let tokens2 = tokenize("distributed_systems");
        assert_eq!(tokens1[0].uuid_hash, tokens2[0].uuid_hash);
    }

    #[test]
    fn hash_case_normalized() {
        // "KAKI" and "kaki" should hash to the same value (both normalize to "kaki")
        let t1 = tokenize("KAKI");
        let t2 = tokenize("kaki");
        assert_eq!(t1[0].uuid_hash, t2[0].uuid_hash,
                   "hash should be case-insensitive: {} vs {}", t1[0].uuid_hash, t2[0].uuid_hash);
    }

    #[test]
    fn byte_offset_correct() {
        let text = "foo bar baz";
        let tokens = tokenize(text);
        assert_eq!(tokens[0].byte_offset, 0);
        assert_eq!(tokens[1].byte_offset, 4);
        assert_eq!(tokens[2].byte_offset, 8);
    }

    #[test]
    fn token_stats_complexity() {
        let tokens = tokenize("the the the the");
        let stats = TokenStats::from_tokens(&tokens);
        assert!(stats.lexical_complexity() < 0.5); // very repetitive
    }

    #[test]
    fn token_stats_high_complexity() {
        let tokens = tokenize("alpha beta gamma delta epsilon zeta eta theta");
        let stats = TokenStats::from_tokens(&tokens);
        assert!(stats.lexical_complexity() > 0.9); // fully unique words
    }

    #[test]
    fn camel_case_word() {
        let tokens = tokenize("DatabaseSystem");
        assert_eq!(tokens[0].class, TokenClass::ProperNoun);
        assert_eq!(tokens[0].text, "DatabaseSystem");
    }

    #[test]
    fn empty_text_no_tokens() {
        let tokens = tokenize("");
        assert!(tokens.is_empty());
    }
}
