//! Soundex — phonetic indexing of English words by pronunciation.
//!
//! American Soundex (US National Archives standard):
//!   1. Retain the first letter
//!   2. Replace consonants with digits (groups: BFPV=1, CGJKQSXYZ=2, DT=3,
//!      L=4, MN=5, R=6)
//!   3. Remove consecutive duplicates and vowels/H/W/Y
//!   4. Pad or truncate to exactly 4 characters
//!
//! Returns a 4-character code: one letter + three digits (e.g. "R163").
//! Identical Soundex codes → words likely sound the same in English.
//!
//! Used by: DqmDimension::Uniqueness (phonetic entity resolution)

/// Compute the American Soundex code for an ASCII word.
/// Returns a 4-byte array: `[letter, digit, digit, digit]`.
/// Non-alphabetic input returns `['Z', '0', '0', '0']` by convention.
pub fn soundex(input: &str) -> [u8; 4] {
    let upper: Vec<u8> = input
        .bytes()
        .filter(|b| b.is_ascii_alphabetic())
        .map(|b| b.to_ascii_uppercase())
        .collect();

    if upper.is_empty() {
        return *b"Z000";
    }

    let first = upper[0];
    let mut code = [first, b'0', b'0', b'0'];
    let mut pos = 1usize;
    let mut prev_digit = soundex_digit(first);

    for &ch in &upper[1..] {
        if pos >= 4 {
            break;
        }
        let d = soundex_digit(ch);
        if d != 0 && d != prev_digit {
            code[pos] = b'0' + d;
            pos += 1;
        }
        if d != 0 {
            prev_digit = d;
        } else if ch != b'H' && ch != b'W' {
            // Vowels and Y separate duplicate-suppression groups
            prev_digit = 0;
        }
    }

    code
}

/// Soundex code as a UTF-8 String (e.g. `"R163"`).
pub fn soundex_str(input: &str) -> String {
    let code = soundex(input);
    String::from_utf8(code.to_vec()).unwrap_or_else(|_| "Z000".into())
}

/// True when two strings have the same Soundex code.
pub fn sounds_like(a: &str, b: &str) -> bool {
    soundex(a) == soundex(b)
}

fn soundex_digit(ch: u8) -> u8 {
    match ch {
        b'B' | b'F' | b'P' | b'V' => 1,
        b'C' | b'G' | b'J' | b'K' | b'Q' | b'S' | b'X' | b'Z' => 2,
        b'D' | b'T' => 3,
        b'L' => 4,
        b'M' | b'N' => 5,
        b'R' => 6,
        _ => 0, // vowels, H, W, Y
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn robert_and_rupert_share_code() {
        // Classic test: Robert R163, Rupert R163
        assert_eq!(soundex_str("Robert"), soundex_str("Rupert"));
    }

    #[test]
    fn robert_code_is_r163() {
        assert_eq!(soundex_str("Robert"), "R163");
    }

    #[test]
    fn sounds_like_is_symmetric() {
        assert_eq!(
            sounds_like("Robert", "Rupert"),
            sounds_like("Rupert", "Robert")
        );
    }

    #[test]
    fn same_word_always_matches() {
        assert!(sounds_like("Sovereign", "Sovereign"));
    }

    #[test]
    fn different_sounding_names_do_not_match() {
        assert!(!sounds_like("Smith", "Jones"));
    }

    #[test]
    fn code_is_always_four_chars() {
        for word in &["A", "Bb", "Hi", "Washington", "Lee", "XYZ"] {
            let code = soundex_str(word);
            assert_eq!(code.len(), 4, "code for '{word}' has wrong length: {code}");
        }
    }

    #[test]
    fn empty_input_returns_z000() {
        assert_eq!(soundex_str(""), "Z000");
        assert_eq!(soundex_str("123"), "Z000");
    }

    #[test]
    fn tymczak_and_tymczak_match() {
        // US NARA example: Tymczak = T522
        assert_eq!(soundex_str("Tymczak"), "T522");
    }
}
