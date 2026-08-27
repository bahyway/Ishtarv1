//! VGCA · Vector Geometric Cleansing Analysis (sealed operators, DRAFT)
//! m ∈ R^6 = (jaro_winkler, levenshtein_sim, minhash_jaccard, soundex, cosine3g, entropy_affinity)
//! Σ = <m,u> agreement with the unanimity axis · Λ = ||m ∧ u|| the wedge of disagreement · Δ = cleansing displacement
//! Law: no auto-merge above the wedge threshold — the wedge protects what the mean would betray.
pub mod fusion;
pub mod persistence;
pub mod verdicts; // 0-dim persistence names σ at the widest gap
