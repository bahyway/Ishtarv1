//! Jaro-Winkler distance — string similarity weighted toward common prefixes.
//!
//! Gives higher scores to strings that agree from the beginning, making it
//! highly effective for human names and entity matching.
//!
//! Algorithm:
//!   1. Compute Jaro similarity: matching characters within window ⌊max_len/2⌋-1
//!   2. Apply Winkler prefix scaling: bonus for shared prefix up to 4 characters
//!
//! Used by: DqmDimension::Uniqueness (name-based entity resolution)

/// Compute Jaro similarity ∈ [0.0, 1.0].
pub fn jaro(a: &str, b: &str) -> f32 {
    if a == b {
        return 1.0;
    }

    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let la = a.len();
    let lb = b.len();

    if la == 0 || lb == 0 {
        return 0.0;
    }

    let match_window = (la.max(lb) / 2).saturating_sub(1);

    let mut a_matched = vec![false; la];
    let mut b_matched = vec![false; lb];
    let mut matches = 0usize;

    for i in 0..la {
        let start = i.saturating_sub(match_window);
        let end = (i + match_window + 1).min(lb);
        for j in start..end {
            if !b_matched[j] && a[i] == b[j] {
                a_matched[i] = true;
                b_matched[j] = true;
                matches += 1;
                break;
            }
        }
    }

    if matches == 0 {
        return 0.0;
    }

    // Count transpositions
    let mut transpositions = 0usize;
    let mut j = 0usize;
    for i in 0..la {
        if a_matched[i] {
            while !b_matched[j] {
                j += 1;
            }
            if a[i] != b[j] {
                transpositions += 1;
            }
            j += 1;
        }
    }

    let m = matches as f32;
    let t = (transpositions / 2) as f32;
    (m / la as f32 + m / lb as f32 + (m - t) / m) / 3.0
}

/// Compute Jaro-Winkler similarity ∈ [0.0, 1.0].
///
/// `p` is the prefix scaling factor — standard value is 0.1.
/// Maximum prefix length considered is 4 characters.
pub fn jaro_winkler(a: &str, b: &str) -> f32 {
    jaro_winkler_p(a, b, 0.1)
}

/// Jaro-Winkler with explicit prefix scaling factor `p`.
pub fn jaro_winkler_p(a: &str, b: &str, p: f32) -> f32 {
    let jaro_sim = jaro(a, b);
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();

    let prefix_len = a_chars
        .iter()
        .zip(b_chars.iter())
        .take(4)
        .take_while(|(ac, bc)| ac == bc)
        .count() as f32;

    jaro_sim + prefix_len * p * (1.0 - jaro_sim)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_strings_score_one() {
        assert!((jaro_winkler("bahaa", "bahaa") - 1.0).abs() < 1e-4);
    }

    #[test]
    fn empty_strings_score_one() {
        assert!((jaro_winkler("", "") - 1.0).abs() < 1e-4);
    }

    #[test]
    fn one_empty_scores_zero() {
        assert!(jaro_winkler("abc", "").abs() < 1e-4);
        assert!(jaro_winkler("", "abc").abs() < 1e-4);
    }

    #[test]
    fn classic_martha_marhta() {
        // Jaro("MARTHA","MARHTA") = 0.944, JW ≈ 0.961
        let jw = jaro_winkler("MARTHA", "MARHTA");
        assert!(jw > 0.94, "expected > 0.94, got {jw:.4}");
    }

    #[test]
    fn classic_dixon_dicksonx() {
        // JW("DIXON","DICKSONX") ≈ 0.813
        let jw = jaro_winkler("DIXON", "DICKSONX");
        assert!(jw > 0.80, "expected > 0.80, got {jw:.4}");
    }

    #[test]
    fn prefix_bonus_favors_common_start() {
        // "john" vs "jonathan" — shares 4-char prefix, gets Winkler bonus
        let jw = jaro_winkler("john", "jonathan");
        let j = jaro("john", "jonathan");
        assert!(
            jw > j,
            "Winkler should exceed Jaro for common-prefix strings"
        );
    }

    #[test]
    fn completely_different_strings_low_score() {
        let jw = jaro_winkler("abc", "xyz");
        assert!(jw < 0.2, "expected < 0.2, got {jw:.4}");
    }

    #[test]
    fn symmetry() {
        let ab = jaro_winkler("fadam", "fadaam");
        let ba = jaro_winkler("fadaam", "fadam");
        assert!((ab - ba).abs() < 1e-5);
    }
}
