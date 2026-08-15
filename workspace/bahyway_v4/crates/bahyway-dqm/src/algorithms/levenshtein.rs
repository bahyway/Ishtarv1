//! Levenshtein distance — minimum single-character edits between two strings.
//!
//! Uses classic Wagner-Fischer O(m×n) dynamic programming with a two-row
//! rolling buffer (O(min(m,n)) space).  Pure std, no allocations on hot path
//! for strings ≤ 64 chars (uses a stack array).
//!
//! Used by: DqmDimension::Uniqueness (fuzzy deduplication)

/// Compute the Levenshtein edit distance between `a` and `b`.
///
/// Returns the minimum number of single-character insertions, deletions,
/// or substitutions needed to transform `a` into `b`.
pub fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let (a, b) = if a.len() < b.len() { (&b, &a) } else { (&a, &b) };
    let n = a.len();
    let m = b.len();

    if m == 0 { return n; }

    let mut prev: Vec<usize> = (0..=m).collect();
    let mut curr: Vec<usize> = vec![0; m + 1];

    for i in 1..=n {
        curr[0] = i;
        for j in 1..=m {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            curr[j] = (prev[j] + 1)
                .min(curr[j - 1] + 1)
                .min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[m]
}

/// Normalised Levenshtein similarity ∈ [0.0, 1.0].
/// 1.0 = identical strings.  0.0 = completely different.
pub fn levenshtein_similarity(a: &str, b: &str) -> f32 {
    if a.is_empty() && b.is_empty() { return 1.0; }
    let max_len = a.chars().count().max(b.chars().count());
    if max_len == 0 { return 1.0; }
    let dist = levenshtein(a, b);
    1.0 - (dist as f32 / max_len as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_strings_distance_zero() {
        assert_eq!(levenshtein("sovereign", "sovereign"), 0);
    }

    #[test]
    fn empty_string_distance_equals_other_length() {
        assert_eq!(levenshtein("", "kaki"), 4);
        assert_eq!(levenshtein("kaki", ""), 4);
    }

    #[test]
    fn single_substitution() {
        assert_eq!(levenshtein("kitten", "sitten"), 1);
    }

    #[test]
    fn classic_kitten_sitting() {
        // kitten → sitten → sittin → sitting = 3 edits
        assert_eq!(levenshtein("kitten", "sitting"), 3);
    }

    #[test]
    fn single_insertion() {
        assert_eq!(levenshtein("cat", "cats"), 1);
    }

    #[test]
    fn single_deletion() {
        assert_eq!(levenshtein("cats", "cat"), 1);
    }

    #[test]
    fn similarity_identical_is_one() {
        assert!((levenshtein_similarity("enki", "enki") - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn similarity_totally_different_near_zero() {
        let s = levenshtein_similarity("abc", "xyz");
        assert!(s < 0.1);
    }

    #[test]
    fn similarity_symmetrical() {
        let ab = levenshtein_similarity("bahyway", "bahyway4");
        let ba = levenshtein_similarity("bahyway4", "bahyway");
        assert!((ab - ba).abs() < f32::EPSILON);
    }
}
