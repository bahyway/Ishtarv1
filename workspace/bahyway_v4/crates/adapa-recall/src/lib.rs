//! adapa-recall — sovereign TF-IDF + cosine similarity retrieval.
//!
//! HONEST SCOPE (2026-07-10): this is lexical-semantic retrieval, not
//! neural embeddings. Confirmed earlier the same day: no embedding model
//! exists anywhere in this ecosystem (`enkidullm-model` runs GGUF
//! generation models, not embedding models; `enkidullm-core`'s
//! `OrbitValue::from_embedding` slot is real but has zero callers). This
//! crate is the honest v1: it materially improves on the previous state
//! (enkidullm-memory's MemorySearch used raw term-frequency only, no
//! inverse-document-frequency component at all — the `_total_docs`
//! parameter it received was never used). TF-IDF + cosine is a real,
//! well-understood retrieval method that surfaces documents sharing rare,
//! distinguishing terms with a query, rather than just common-word
//! overlap. It is not a replacement for true semantic embeddings if/when
//! a real embedding model is integrated — it's the honest step available
//! without one.
//!
//! Named for Uanna/Adapa, "he who completed the plans of heaven and
//! earth" — the Seven Sages' first and most famous member, proposed
//! (pending Architect confirmation) as the name of the PDM Analyzing &
//! Visualizing tool this crate is the retrieval foundation for.
#![forbid(unsafe_code)]

use std::collections::{HashMap, HashSet};

pub type DocId = String;

/// A retrievable unit of content — a conversation turn, a book chunk, a
/// PDM model note, anything with an id and text content.
#[derive(Debug, Clone)]
pub struct Document {
    pub id: DocId,
    pub content: String,
}

impl Document {
    pub fn new(id: impl Into<DocId>, content: impl Into<String>) -> Self {
        Document {
            id: id.into(),
            content: content.into(),
        }
    }
}

#[derive(Debug, Clone, Default)]
struct TermVector(HashMap<String, f32>);

/// A built TF-IDF index over a fixed document set. Rebuild (via `build`)
/// when the corpus changes — this crate does not support incremental
/// insertion, matching the append-only-batch pattern used elsewhere in
/// the ecosystem (journal projections, NINSUN drift batches).
pub struct RecallIndex {
    docs: Vec<Document>,
    doc_vectors: Vec<TermVector>,
    idf: HashMap<String, f32>,
}

fn tokenize(text: &str) -> Vec<String> {
    text.split_whitespace()
        .map(|w| {
            w.to_lowercase()
                .trim_matches(|c: char| !c.is_alphanumeric())
                .to_string()
        })
        .filter(|w| w.len() > 2)
        .collect()
}

fn term_freq(tokens: &[String]) -> HashMap<String, f32> {
    let mut tf = HashMap::new();
    for t in tokens {
        *tf.entry(t.clone()).or_insert(0.0) += 1.0;
    }
    let total = tokens.len() as f32;
    if total > 0.0 {
        for v in tf.values_mut() {
            *v /= total;
        }
    }
    tf
}

fn norm(v: &HashMap<String, f32>) -> f32 {
    v.values().map(|x| x * x).sum::<f32>().sqrt()
}

fn cosine(a: &HashMap<String, f32>, a_norm: f32, b: &HashMap<String, f32>) -> f32 {
    let b_norm = norm(b);
    if a_norm == 0.0 || b_norm == 0.0 {
        return 0.0;
    }
    let dot: f32 = a
        .iter()
        .map(|(k, v)| v * b.get(k).copied().unwrap_or(0.0))
        .sum();
    dot / (a_norm * b_norm)
}

impl RecallIndex {
    /// Builds the index from a fixed document set. Smoothed IDF
    /// (`ln((n+1)/(df+1)) + 1`) so a term appearing in every document
    /// still contributes a small positive weight rather than zeroing out.
    pub fn build(docs: Vec<Document>) -> Self {
        let n = docs.len() as f32;
        let mut df: HashMap<String, f32> = HashMap::new();
        let mut token_lists: Vec<Vec<String>> = Vec::with_capacity(docs.len());

        for d in &docs {
            let toks = tokenize(&d.content);
            let unique: HashSet<&String> = toks.iter().collect();
            for t in unique {
                *df.entry(t.clone()).or_insert(0.0) += 1.0;
            }
            token_lists.push(toks);
        }

        let idf: HashMap<String, f32> = df
            .into_iter()
            .map(|(term, doc_freq)| {
                let w = ((n + 1.0) / (doc_freq + 1.0)).ln() + 1.0;
                (term, w)
            })
            .collect();

        let doc_vectors = token_lists
            .iter()
            .map(|toks| {
                let tf = term_freq(toks);
                let mut v = HashMap::new();
                for (term, tf_val) in tf {
                    let idf_val = *idf.get(&term).unwrap_or(&1.0);
                    v.insert(term, tf_val * idf_val);
                }
                TermVector(v)
            })
            .collect();

        RecallIndex {
            docs,
            doc_vectors,
            idf,
        }
    }

    /// Ranks documents by cosine similarity to `query`'s TF-IDF vector
    /// (using the corpus's IDF weights, not the query's own). Returns at
    /// most `top_k` (doc_id, score) pairs, score-descending, score > 0
    /// only.
    pub fn query(&self, query: &str, top_k: usize) -> Vec<(DocId, f32)> {
        let q_tokens = tokenize(query);
        if q_tokens.is_empty() {
            return Vec::new();
        }

        let q_tf = term_freq(&q_tokens);
        let mut q_vec = HashMap::new();
        for (term, tf_val) in q_tf {
            let idf_val = *self.idf.get(&term).unwrap_or(&1.0);
            q_vec.insert(term, tf_val * idf_val);
        }
        let q_norm = norm(&q_vec);
        if q_norm == 0.0 {
            return Vec::new();
        }

        let mut scored: Vec<(DocId, f32)> = self
            .docs
            .iter()
            .zip(self.doc_vectors.iter())
            .filter_map(|(doc, dv)| {
                let score = cosine(&q_vec, q_norm, &dv.0);
                if score > 0.0 {
                    Some((doc.id.clone(), score))
                } else {
                    None
                }
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);
        scored
    }

    pub fn len(&self) -> usize {
        self.docs.len()
    }
    pub fn is_empty(&self) -> bool {
        self.docs.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_ranks_exact_topical_match_first() {
        let index = RecallIndex::build(vec![
            Document::new(
                "d1",
                "KAKI is the sovereign 16-byte identity used everywhere in EnkiDB",
            ),
            Document::new("d2", "HeptaScript is the W5H2 anti-SQL query language"),
            Document::new("d3", "The weather today is cloudy with a chance of rain"),
        ]);
        let results = index.query("KAKI sovereign identity", 3);
        assert!(!results.is_empty());
        assert_eq!(results[0].0, "d1");
    }

    #[test]
    fn rare_shared_term_outranks_common_term_overlap() {
        // "particle" appears in every doc (low IDF, low discriminating
        // power). "esarhaddon" appears in only one doc (high IDF).
        // A query naming the rare term should retrieve that doc first
        // even though a common-term-only doc shares more raw words.
        let index = RecallIndex::build(vec![
            Document::new("common1", "particle particle particle data particle"),
            Document::new("common2", "particle particle data particle particle"),
            Document::new(
                "rare",
                "esarhaddon computes structural mortality for particle rescue",
            ),
        ]);
        let results = index.query("esarhaddon structural mortality", 3);
        assert_eq!(results[0].0, "rare");
    }

    #[test]
    fn empty_query_returns_nothing() {
        let index = RecallIndex::build(vec![Document::new("d1", "some content here")]);
        assert!(index.query("", 5).is_empty());
        assert!(index.query("   ", 5).is_empty());
    }

    #[test]
    fn top_k_truncates() {
        let index = RecallIndex::build(vec![
            Document::new("d1", "sovereign identity kaki particle"),
            Document::new("d2", "sovereign identity kaki particle two"),
            Document::new("d3", "sovereign identity kaki particle three"),
        ]);
        let results = index.query("sovereign identity kaki particle", 2);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn unrelated_query_returns_empty_or_low_score() {
        let index = RecallIndex::build(vec![Document::new("d1", "KAKI is the sovereign identity")]);
        let results = index.query("zzz completely unrelated qqq wwww", 5);
        assert!(results.is_empty());
    }

    #[test]
    fn empty_corpus_is_safe() {
        let index = RecallIndex::build(vec![]);
        assert!(index.is_empty());
        assert!(index.query("anything", 5).is_empty());
    }
}
