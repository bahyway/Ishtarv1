//! VGCA-Σ — 7D Feature Score Vector for text quality geometry.
//!
//! The FSV embeds every data record as a point in a 7-dimensional quality
//! manifold.  Records near the domain centroid are Clean; records far from
//! it are Outliers or Aliens.  No rule-lists.  Pure geometry.
//!
//! Dimensions:
//!   D1 char_count_norm      — normalised character count   [0,1]
//!   D2 digit_density        — fraction of digit chars      [0,1]
//!   D3 arabic_density       — fraction of Arabic Unicode   [0,1]
//!   D4 latin_density        — fraction of Latin ASCII      [0,1]
//!   D5 punct_density        — fraction of punctuation      [0,1]
//!   D6 word_count_norm      — normalised whitespace tokens [0,1]
//!   D7 shannon_entropy_norm — normalised Shannon entropy   [0,1]
//!
//! DUB.SAR 𒁾 — BahyWay.Ecosystem v4.0 | Pure Rust

/// 7D Feature Score Vector for one text field.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FeatureScoreVector {
    /// D1 — normalised character count (max 100 chars → 1.0)
    pub char_count_norm:      f64,
    /// D2 — fraction of ASCII digit characters
    pub digit_density:        f64,
    /// D3 — fraction of Arabic Unicode characters
    pub arabic_density:       f64,
    /// D4 — fraction of ASCII alphabetic characters
    pub latin_density:        f64,
    /// D5 — fraction of ASCII punctuation characters
    pub punct_density:        f64,
    /// D6 — normalised whitespace-token count (max 20 tokens → 1.0)
    pub word_count_norm:      f64,
    /// D7 — Shannon entropy normalised to [0,1] (max ~5 bits/char)
    pub shannon_entropy_norm: f64,
}

impl FeatureScoreVector {
    /// All-zero FSV — represents an empty or null field.
    pub fn zero() -> Self {
        Self { char_count_norm: 0.0, digit_density: 0.0, arabic_density: 0.0,
               latin_density: 0.0, punct_density: 0.0, word_count_norm: 0.0,
               shannon_entropy_norm: 0.0 }
    }

    /// Perfect FSV — all dimensions = 1.0 (used for tribe ideal point).
    pub fn perfect() -> Self {
        Self { char_count_norm: 1.0, digit_density: 1.0, arabic_density: 1.0,
               latin_density: 1.0, punct_density: 1.0, word_count_norm: 1.0,
               shannon_entropy_norm: 1.0 }
    }

    /// Return dimensions as a `[f64; 7]` array (D1..D7 order).
    pub fn as_array(&self) -> [f64; 7] {
        [self.char_count_norm, self.digit_density, self.arabic_density,
         self.latin_density, self.punct_density, self.word_count_norm,
         self.shannon_entropy_norm]
    }

    /// Euclidean distance to another FSV in 7D space.
    pub fn distance(&self, other: &Self) -> f64 {
        self.as_array().iter()
            .zip(other.as_array().iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f64>()
            .sqrt()
    }

    /// L1 (Manhattan) norm — sum of all dimensions.
    pub fn l1_norm(&self) -> f64 {
        self.as_array().iter().sum()
    }
}

// ── Arabic Unicode range detection ────────────────────────────────────────────

/// Returns `true` if `c` is an Arabic Unicode character.
///
/// Covers:
///   U+0600–U+06FF  Arabic block (core)
///   U+0750–U+077F  Arabic Supplement
///   U+FB50–U+FDFF  Arabic Presentation Forms-A
///   U+FE70–U+FEFF  Arabic Presentation Forms-B
#[inline]
pub fn is_arabic_char(c: char) -> bool {
    let cp = c as u32;
    matches!(cp, 0x0600..=0x06FF | 0x0750..=0x077F | 0xFB50..=0xFDFF | 0xFE70..=0xFEFF)
}

// ── Shannon entropy ───────────────────────────────────────────────────────────

/// Compute Shannon entropy H(X) = -Σ p(x) log₂ p(x) in bits.
///
/// Returns 0.0 for an empty slice.
pub fn shannon_entropy(chars: &[char]) -> f64 {
    let n = chars.len();
    if n == 0 { return 0.0; }

    // Character frequency table — use a fixed 256-entry array for ASCII
    // and a small Vec for supplementary Unicode.
    let mut freq = [0u32; 256];
    let mut unicode_freq: Vec<(char, u32)> = Vec::new();

    for &c in chars {
        if (c as u32) < 256 {
            freq[c as usize] += 1;
        } else {
            if let Some(entry) = unicode_freq.iter_mut().find(|(ch, _)| *ch == c) {
                entry.1 += 1;
            } else {
                unicode_freq.push((c, 1));
            }
        }
    }

    let nf = n as f64;
    let mut h = 0.0f64;

    for &cnt in &freq {
        if cnt > 0 {
            let p = cnt as f64 / nf;
            h -= p * p.log2();
        }
    }
    for (_, cnt) in &unicode_freq {
        let p = *cnt as f64 / nf;
        h -= p * p.log2();
    }
    h
}

// ── Main FSV computation ──────────────────────────────────────────────────────

/// Compute the 7D Feature Score Vector for a text field.
///
/// All dimensions are normalised to `[0.0, 1.0]`.
/// Empty text returns `FeatureScoreVector::zero()`.
pub fn compute_fsv(text: &str) -> FeatureScoreVector {
    let chars: Vec<char> = text.chars().collect();
    let total = chars.len();

    if total == 0 {
        return FeatureScoreVector::zero();
    }

    let total_f = total as f64;
    let digits  = chars.iter().filter(|&&c| c.is_ascii_digit()).count() as f64;
    let arabic  = chars.iter().filter(|&&c| is_arabic_char(c)).count() as f64;
    let latin   = chars.iter().filter(|&&c| c.is_ascii_alphabetic()).count() as f64;
    let punct   = chars.iter().filter(|&&c| c.is_ascii_punctuation()).count() as f64;
    let words   = text.split_whitespace().count() as f64;
    let entropy = shannon_entropy(&chars);

    // Normalisation caps:
    //   char_count: 100 chars → 1.0   (typical Arabic name ≤ 50 chars)
    //   word_count: 20 words → 1.0    (typical address ≤ 10 words)
    //   entropy:    5.0 bits → 1.0    (natural language max ≈ 4.5 bits)
    FeatureScoreVector {
        char_count_norm:      (total_f / 100.0).min(1.0),
        digit_density:        digits  / total_f,
        arabic_density:       arabic  / total_f,
        latin_density:        latin   / total_f,
        punct_density:        punct   / total_f,
        word_count_norm:      (words  / 20.0).min(1.0),
        shannon_entropy_norm: (entropy / 5.0).min(1.0),
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_text_gives_zero_fsv() {
        let fsv = compute_fsv("");
        assert_eq!(fsv.as_array(), [0.0; 7]);
    }

    #[test]
    fn pure_digits_gives_full_digit_density() {
        let fsv = compute_fsv("12345678");
        assert!((fsv.digit_density - 1.0).abs() < 1e-9);
        assert_eq!(fsv.arabic_density, 0.0);
        assert_eq!(fsv.latin_density, 0.0);
    }

    #[test]
    fn arabic_name_gives_high_arabic_density() {
        let text = "محمد أحمد علي";
        let fsv = compute_fsv(text);
        assert!(fsv.arabic_density > 0.5, "arabic_density={}", fsv.arabic_density);
        assert_eq!(fsv.digit_density, 0.0);
    }

    #[test]
    fn latin_text_gives_zero_arabic_density() {
        let fsv = compute_fsv("Hello World");
        assert_eq!(fsv.arabic_density, 0.0);
        assert!(fsv.latin_density > 0.7, "latin_density={}", fsv.latin_density);
    }

    #[test]
    fn all_dimensions_in_unit_interval() {
        for text in ["", "abc", "123", "محمد", "Hello World", "a.b,c;d!"] {
            let fsv = compute_fsv(text);
            for (i, &d) in fsv.as_array().iter().enumerate() {
                assert!(d >= 0.0 && d <= 1.0, "dim[{i}]={d} out of [0,1] for '{text}'");
            }
        }
    }

    #[test]
    fn entropy_zero_for_single_char_repeated() {
        // "aaaaaaa" — all same char → p=1 → H=0
        let fsv = compute_fsv("aaaaaaa");
        assert!(fsv.shannon_entropy_norm < 0.01, "entropy={}", fsv.shannon_entropy_norm);
    }

    #[test]
    fn entropy_nonzero_for_mixed_text() {
        let fsv = compute_fsv("abcdefghij");
        assert!(fsv.shannon_entropy_norm > 0.0);
    }

    #[test]
    fn distance_self_is_zero() {
        let fsv = compute_fsv("محمد أحمد");
        assert!(fsv.distance(&fsv) < 1e-9);
    }

    #[test]
    fn distance_symmetric() {
        let a = compute_fsv("محمد");
        let b = compute_fsv("Hello123");
        let d_ab = a.distance(&b);
        let d_ba = b.distance(&a);
        assert!((d_ab - d_ba).abs() < 1e-9);
    }

    #[test]
    fn word_count_norm_capped_at_one() {
        // 25 words → word_count_norm should be 1.0 (capped at 20 words)
        let text = "a b c d e f g h i j k l m n o p q r s t u v w x y";
        let fsv = compute_fsv(text);
        assert!((fsv.word_count_norm - 1.0).abs() < 1e-9);
    }

    #[test]
    fn is_arabic_char_covers_core_block() {
        assert!(is_arabic_char('م'));
        assert!(is_arabic_char('ح'));
        assert!(!is_arabic_char('a'));
        assert!(!is_arabic_char('1'));
    }
}
