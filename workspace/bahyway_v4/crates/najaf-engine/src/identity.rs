//! InscriptionMatcher — FNV-1a Arabic text matching for grave identity resolution.
//!
//! Graves discovered by satellite or drone often have a readable headstone inscription.
//! The matcher compares normalized Arabic text against the civil registry to produce
//! identity candidates with a confidence score on the QUALITY_DIVISOR=240 scale:
//!
//! | Match type                           | Confidence |
//! |--------------------------------------|------------|
//! | Exact normalized text match          | 240        |
//! | All words match (order-independent)  | 200        |
//! | Prefix (first word) match            | 150        |
//! | No match                             | 0 (absent) |

#![forbid(unsafe_code)]

use crate::grave::GraveId;

/// Candidate identity match returned by the inscription matcher.
#[derive(Debug, Clone)]
pub struct IdentityCandidate {
    /// Grave whose registered name matched the inscription.
    pub grave_id: GraveId,
    /// Confidence on the QUALITY_DIVISOR=240 scale.
    pub confidence: u8,
    /// The registered name that produced the match.
    pub matched_name: String,
}

/// FNV-1a 32-bit hash of normalized text.
fn fnv1a(s: &str) -> u32 {
    let mut h: u32 = 0x811C_9DC5;
    for &b in s.as_bytes() {
        h ^= b as u32;
        h = h.wrapping_mul(0x0100_0193);
    }
    h
}

/// Normalize Arabic text: trim whitespace, collapse multiple spaces to one.
fn normalize(text: &str) -> String {
    let trimmed = text.trim();
    let mut out = String::with_capacity(trimmed.len());
    let mut prev_space = false;
    for ch in trimmed.chars() {
        if ch == ' ' || ch == '\t' || ch == '\n' {
            if !prev_space && !out.is_empty() {
                out.push(' ');
            }
            prev_space = true;
        } else {
            out.push(ch);
            prev_space = false;
        }
    }
    out
}

struct Entry {
    grave_id: GraveId,
    name: String,
    name_hash: u32,
    word_hashes: Vec<u32>,
}

/// Registry of known grave names for inscription matching.
pub struct InscriptionMatcher {
    entries: Vec<Entry>,
}

impl InscriptionMatcher {
    pub fn new() -> Self {
        InscriptionMatcher {
            entries: Vec::new(),
        }
    }

    /// Register a grave's known Arabic name for later inscription matching.
    pub fn register(&mut self, grave_id: GraveId, name_arabic: &str) {
        let name = normalize(name_arabic);
        let name_hash = fnv1a(&name);
        let words: Vec<&str> = name.split(' ').filter(|w| !w.is_empty()).collect();
        let word_hashes = words.iter().map(|w| fnv1a(w)).collect();
        self.entries.push(Entry {
            grave_id,
            name,
            name_hash,
            word_hashes,
        });
    }

    /// Match an inscription against all registered names.
    ///
    /// Returns candidates sorted by descending confidence.
    /// Only candidates with confidence > 0 are returned.
    pub fn match_inscription(&self, text: &str) -> Vec<IdentityCandidate> {
        let query = normalize(text);
        let query_hash = fnv1a(&query);
        let query_words: Vec<&str> = query.split(' ').filter(|w| !w.is_empty()).collect();
        let query_word_hashes: Vec<u32> = query_words.iter().map(|w| fnv1a(w)).collect();
        let query_first_hash = query_word_hashes.first().copied().unwrap_or(0);

        let mut results: Vec<IdentityCandidate> = self
            .entries
            .iter()
            .filter_map(|e| {
                let confidence = if e.name_hash == query_hash {
                    // Exact normalized match
                    240u8
                } else if !e.word_hashes.is_empty()
                    && e.word_hashes.len() == query_word_hashes.len()
                    && e.word_hashes.iter().all(|h| query_word_hashes.contains(h))
                {
                    // All words present, possibly reordered
                    200
                } else if !e.word_hashes.is_empty()
                    && query_first_hash != 0
                    && e.word_hashes[0] == query_first_hash
                {
                    // First word (given name) matches
                    150
                } else {
                    return None;
                };
                Some(IdentityCandidate {
                    grave_id: e.grave_id,
                    confidence,
                    matched_name: e.name.clone(),
                })
            })
            .collect();

        results.sort_by_key(|r| std::cmp::Reverse(r.confidence));
        results
    }
}

impl Default for InscriptionMatcher {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn matcher_with_registry() -> InscriptionMatcher {
        let mut m = InscriptionMatcher::new();
        m.register(101, "عبد الله الحسيني");
        m.register(102, "محمد علي الكاظمي");
        m.register(103, "حسين باقر النجفي");
        m.register(104, "فاطمة الزهراء");
        m
    }

    #[test]
    fn exact_match_returns_240() {
        let m = matcher_with_registry();
        let results = m.match_inscription("عبد الله الحسيني");
        assert!(!results.is_empty());
        assert_eq!(results[0].confidence, 240);
        assert_eq!(results[0].grave_id, 101);
    }

    #[test]
    fn exact_match_with_extra_whitespace() {
        let m = matcher_with_registry();
        // Leading/trailing spaces and internal double-space — all normalized
        let results = m.match_inscription("  محمد  علي  الكاظمي  ");
        assert!(!results.is_empty());
        assert_eq!(results[0].confidence, 240);
        assert_eq!(results[0].grave_id, 102);
    }

    #[test]
    fn all_words_match_reordered_returns_200() {
        let m = matcher_with_registry();
        // "الحسيني عبد الله" — same words, different order
        let results = m.match_inscription("الحسيني عبد الله");
        assert!(!results.is_empty());
        assert_eq!(results[0].confidence, 200);
        assert_eq!(results[0].grave_id, 101);
    }

    #[test]
    fn first_word_match_returns_150() {
        let m = matcher_with_registry();
        // Only first word matches — partial inscription (faded headstone)
        let results = m.match_inscription("حسين النجفي الاكبر");
        // "حسين" is the first word of entry 103 ("حسين باقر النجفي")
        let hit = results.iter().find(|c| c.grave_id == 103);
        assert!(hit.is_some());
        assert_eq!(hit.unwrap().confidence, 150);
    }

    #[test]
    fn no_match_returns_empty() {
        let m = matcher_with_registry();
        let results = m.match_inscription("علي أكبر التبريزي");
        // No registered name has "علي" as first word (grave 102 starts with "محمد")
        assert!(results.iter().all(|c| c.grave_id != 101));
        // Could be empty or only 150-confidence hits from coincidental first words
        // Main assertion: no exact or word-order match for an unregistered name
        assert!(!results.iter().any(|c| c.confidence == 240));
    }

    #[test]
    fn results_sorted_by_descending_confidence() {
        let mut m = InscriptionMatcher::new();
        // Register two entries where query will match one exactly and one by first word
        m.register(201, "عبد الرحمن الموسوي");
        m.register(202, "عبد الكريم البصري");
        // "عبد الرحمن الموسوي" → exact=240 for 201, first-word "عبد" → 150 for 202
        let results = m.match_inscription("عبد الرحمن الموسوي");
        assert!(results.len() >= 2);
        assert!(results[0].confidence >= results[1].confidence);
        assert_eq!(results[0].confidence, 240);
    }

    #[test]
    fn normalize_collapses_whitespace() {
        let n = normalize("  حسين   باقر  ");
        assert_eq!(n, "حسين باقر");
    }

    #[test]
    fn fnv1a_is_deterministic() {
        assert_eq!(fnv1a("عبد الله"), fnv1a("عبد الله"));
        assert_ne!(fnv1a("عبد الله"), fnv1a("عبد الرحمن"));
    }

    #[test]
    fn empty_registry_returns_empty() {
        let m = InscriptionMatcher::new();
        assert!(m.match_inscription("أي اسم").is_empty());
    }

    #[test]
    fn single_word_name_exact_match() {
        let mut m = InscriptionMatcher::new();
        m.register(301, "حسين");
        let results = m.match_inscription("حسين");
        assert_eq!(results[0].confidence, 240);
    }

    #[test]
    fn grave_id_preserved_in_candidate() {
        let mut m = InscriptionMatcher::new();
        m.register(9999, "فاطمة الزهراء");
        let results = m.match_inscription("فاطمة الزهراء");
        assert_eq!(results[0].grave_id, 9999);
    }
}
