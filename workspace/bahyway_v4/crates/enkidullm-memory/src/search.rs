//! search.rs — Sovereign keyword search over conversation memory.
#![forbid(unsafe_code)]

use crate::conversation::ConversationParticle;
use adapa_recall::{Document, RecallIndex};
use std::collections::HashMap;

/// A search result with relevance score.
#[derive(Debug, Clone)]
pub struct SearchResult<'a> {
    pub particle: &'a ConversationParticle,
    pub score:    f32,
    pub snippet:  String,
}

/// Sovereign memory search — no vector DB, no external index.
/// Uses TF-IDF-inspired keyword scoring over conversation content.
pub struct MemorySearch;

impl MemorySearch {
    /// Search particles by real TF-IDF + cosine similarity (adapa-recall,
    /// 2026-07-10), replacing the previous raw-term-frequency-only
    /// scoring in `search()` below (which is kept, unchanged, for
    /// backward compatibility and its own test coverage). This is the
    /// retrieval path TamuzAI should prefer: rare, distinguishing terms
    /// (a crate name, an error code, a proper noun) now outrank generic
    /// word overlap, which plain TF could not distinguish.
    pub fn semantic_search<'a>(
        particles: &[&'a ConversationParticle],
        query: &str,
        max_results: usize,
    ) -> Vec<SearchResult<'a>> {
        // doc id -> particle, keyed by the same hex id adapa-recall uses,
        // so results map back to real particles without re-scanning.
        let mut by_id: HashMap<String, &'a ConversationParticle> = HashMap::new();
        let docs: Vec<Document> = particles.iter().map(|p| {
            let id = kaki_hex(p);
            by_id.insert(id.clone(), p);
            Document::new(id, p.turn.content.clone())
        }).collect();

        let index = RecallIndex::build(docs);
        index.query(query, max_results).into_iter().filter_map(|(id, score)| {
            let particle = *by_id.get(&id)?;
            let terms: Vec<String> = query.split_whitespace().map(|w| w.to_lowercase()).collect();
            let snippet = extract_snippet(&particle.turn.content, &terms);
            Some(SearchResult { particle, score, snippet })
        }).collect()
    }

    /// Search particles by keyword query.
    pub fn search<'a>(
        particles: &[&'a ConversationParticle],
        query: &str,
        max_results: usize,
    ) -> Vec<SearchResult<'a>> {
        let query_terms: Vec<String> = tokenize_query(query);
        if query_terms.is_empty() { return Vec::new(); }

        let total = particles.len() as f32;
        let mut results: Vec<SearchResult> = particles.iter().filter_map(|p| {
            let content = p.turn.content.to_lowercase();
            let score   = score_document(&content, &query_terms, total);
            if score > 0.0 {
                let snippet = extract_snippet(&p.turn.content, &query_terms);
                Some(SearchResult { particle: p, score, snippet })
            } else {
                None
            }
        }).collect();

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results.truncate(max_results);
        results
    }

    /// Find all turns mentioning a specific crate name.
    pub fn find_crate_mentions<'a>(
        particles: &[&'a ConversationParticle],
        crate_name: &str,
    ) -> Vec<&'a ConversationParticle> {
        let name = crate_name.to_lowercase();
        particles.iter()
            .filter(|p| p.turn.content.to_lowercase().contains(&name))
            .copied()
            .collect()
    }

    /// Find all turns from a specific role.
    pub fn find_by_role<'a>(
        particles: &[&'a ConversationParticle],
        role: crate::conversation::TurnRole,
    ) -> Vec<&'a ConversationParticle> {
        particles.iter()
            .filter(|p| p.turn.role == role)
            .copied()
            .collect()
    }
}

fn tokenize_query(query: &str) -> Vec<String> {
    query.split_whitespace()
        .map(|w| w.to_lowercase().trim_matches(|c: char| !c.is_alphanumeric()).to_string())
        .filter(|w| w.len() > 2)
        .collect()
}

fn score_document(content: &str, terms: &[String], _total_docs: f32) -> f32 {
    let words: Vec<&str> = content.split_whitespace().collect();
    let doc_len = words.len() as f32;
    if doc_len == 0.0 { return 0.0; }

    let mut score = 0.0f32;
    for term in terms {
        let tf = words.iter().filter(|w| w.contains(term.as_str())).count() as f32 / doc_len;
        if tf > 0.0 { score += tf; }
    }
    score
}

fn kaki_hex(p: &ConversationParticle) -> String {
    p.turn.kaki.bytes().iter().map(|b| format!("{b:02x}")).collect()
}

fn extract_snippet(content: &str, terms: &[String]) -> String {
    let lower = content.to_lowercase();
    for term in terms {
        if let Some(pos) = lower.find(term.as_str()) {
            let start = pos.saturating_sub(30);
            let end   = (pos + 80).min(content.len());
            let mut snippet = String::new();
            if start > 0 { snippet.push_str("..."); }
            snippet.push_str(&content[start..end]);
            if end < content.len() { snippet.push_str("..."); }
            return snippet;
        }
    }
    if content.len() > 80 { format!("{}...", &content[..80]) } else { content.to_string() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conversation::{Turn, TurnRole, ConversationParticle, CONV_TRIBE_ID};
    use enkidb_kaki::KakiMinter;
    use bahyway_core::TribeId;

    fn make_particle(content: &str) -> ConversationParticle {
        let minter = KakiMinter::new(TribeId::from_u16(CONV_TRIBE_ID));
        let turn = Turn::new(TurnRole::User, content.to_string(), 1000, 1, 1, &minter);
        ConversationParticle::from_turn(turn)
    }

    #[test]
    fn search_finds_matching_particle() {
        let p1 = make_particle("KAKI is the sovereign identity");
        let p2 = make_particle("HeptaScript is the query language");
        let particles = vec![&p1, &p2];
        let results = MemorySearch::search(&particles, "KAKI sovereign", 5);
        assert!(!results.is_empty());
        assert!(results[0].particle.turn.content.contains("KAKI"));
    }

    #[test]
    fn search_empty_query_returns_empty() {
        let p = make_particle("some content");
        let results = MemorySearch::search(&[&p], "", 5);
        assert!(results.is_empty());
    }

    #[test]
    fn semantic_search_ranks_rare_term_match_first() {
        // "esarhaddon" is rare (appears in one turn); "kaki" is common
        // (appears in all three). A query naming the rare term should
        // rank the rare-term turn first via TF-IDF, same discrimination
        // adapa-recall proved at the crate level.
        let p1 = make_particle("kaki kaki kaki particle identity");
        let p2 = make_particle("kaki kaki particle identity kaki");
        let p3 = make_particle("esarhaddon computes structural mortality for kaki particles");
        let particles = vec![&p1, &p2, &p3];

        let results = MemorySearch::semantic_search(&particles, "esarhaddon structural mortality", 3);
        assert!(!results.is_empty());
        assert!(results[0].particle.turn.content.contains("esarhaddon"));
    }

    #[test]
    fn semantic_search_empty_query_returns_empty() {
        let p = make_particle("some content");
        let results = MemorySearch::semantic_search(&[&p], "", 5);
        assert!(results.is_empty());
    }

    #[test]
    fn find_crate_mentions() {
        let p1 = make_particle("enkidb-kaki mints sovereign identities");
        let p2 = make_particle("hepta-score computes H(P)");
        let particles = vec![&p1, &p2];
        let found = MemorySearch::find_crate_mentions(&particles, "enkidb-kaki");
        assert_eq!(found.len(), 1);
    }

    #[test]
    fn find_by_role_filters_correctly() {
        let minter = KakiMinter::new(TribeId::from_u16(CONV_TRIBE_ID));
        let t_user = Turn::new(TurnRole::User, "question".into(), 1, 1, 1, &minter);
        let t_asst = Turn::new(TurnRole::Assistant, "answer".into(), 2, 1, 2, &minter);
        let p1 = ConversationParticle::from_turn(t_user);
        let p2 = ConversationParticle::from_turn(t_asst);
        let particles = vec![&p1, &p2];
        let user_turns = MemorySearch::find_by_role(&particles, TurnRole::User);
        assert_eq!(user_turns.len(), 1);
        assert_eq!(user_turns[0].turn.role, TurnRole::User);
    }
}
