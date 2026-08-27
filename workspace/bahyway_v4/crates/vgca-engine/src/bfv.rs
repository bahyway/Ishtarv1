//! VGCA-Δ — 6D Binary Feature Vector for binary block analysis.
//!
//! Applied when a data field contains raw bytes rather than Unicode text.
//! The BFV characterises the byte distribution to detect fragmented,
//! corrupted, or encrypted blocks before HeptaScore runs.
//!
//! Sovereign constant: δ_frag = 0.35 (ADR-008)
//! If fragmentation_score(bfv) > δ_frag → block is SUSPECT.
//!
//! DUB.SAR 𒁾 — BahyWay.Ecosystem v4.0 | Pure Rust

/// δ_frag — fragmentation threshold (ADR-008, sovereign, never change).
pub const DELTA_FRAG: f64 = 0.35;

/// 6D Binary Feature Vector for raw byte blocks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BinaryFeatureVector {
    /// B1 — Shannon byte entropy normalised to [0,1] (max 8 bits)
    pub byte_entropy: f64,
    /// B2 — fraction of null bytes (0x00)
    pub null_density: f64,
    /// B3 — fraction of printable ASCII bytes (0x20–0x7E)
    pub printable_ratio: f64,
    /// B4 — fraction of high bytes (0x80–0xFF)
    pub high_byte_ratio: f64,
    /// B5 — normalised run-length (longest same-byte run / block length)
    pub run_length_norm: f64,
    /// B6 — repeating 4-byte pattern score (0 = no pattern, 1 = fully periodic)
    pub pattern_score: f64,
}

impl BinaryFeatureVector {
    /// All-zero BFV — represents an empty block.
    pub fn zero() -> Self {
        Self {
            byte_entropy: 0.0,
            null_density: 0.0,
            printable_ratio: 0.0,
            high_byte_ratio: 0.0,
            run_length_norm: 0.0,
            pattern_score: 0.0,
        }
    }

    /// Return dimensions as a `[f64; 6]` array.
    pub fn as_array(&self) -> [f64; 6] {
        [
            self.byte_entropy,
            self.null_density,
            self.printable_ratio,
            self.high_byte_ratio,
            self.run_length_norm,
            self.pattern_score,
        ]
    }
}

/// Compute the 6D BFV for a raw byte block.
pub fn compute_bfv(bytes: &[u8]) -> BinaryFeatureVector {
    let n = bytes.len();
    if n == 0 {
        return BinaryFeatureVector::zero();
    }
    let nf = n as f64;

    // Byte frequency table (256 entries)
    let mut freq = [0u32; 256];
    for &b in bytes {
        freq[b as usize] += 1;
    }

    // B1: Shannon byte entropy
    let entropy: f64 = freq
        .iter()
        .filter(|&&cnt| cnt > 0)
        .map(|&cnt| {
            let p = cnt as f64 / nf;
            -p * p.log2()
        })
        .sum::<f64>();
    let byte_entropy = (entropy / 8.0).min(1.0); // max 8 bits

    // B2: null density
    let null_density = freq[0] as f64 / nf;

    // B3: printable ratio (0x20..=0x7E)
    let printable: u32 = freq[0x20..=0x7E].iter().sum();
    let printable_ratio = printable as f64 / nf;

    // B4: high byte ratio (0x80..=0xFF)
    let high: u32 = freq[0x80..=0xFF].iter().sum();
    let high_byte_ratio = high as f64 / nf;

    // B5: longest run-length (same consecutive byte)
    let mut max_run = 1usize;
    let mut cur_run = 1usize;
    for i in 1..n {
        if bytes[i] == bytes[i - 1] {
            cur_run += 1;
            if cur_run > max_run {
                max_run = cur_run;
            }
        } else {
            cur_run = 1;
        }
    }
    let run_length_norm = (max_run as f64 / nf).min(1.0);

    // B6: 4-byte repeating pattern score
    // Check if the block has a dominant 4-byte pattern
    let pattern_score = detect_pattern_score(bytes);

    BinaryFeatureVector {
        byte_entropy,
        null_density,
        printable_ratio,
        high_byte_ratio,
        run_length_norm,
        pattern_score,
    }
}

/// Detect repeating 4-byte pattern strength.
///
/// Returns a score in [0,1]: 1.0 = fully periodic (e.g. padding),
/// 0.0 = no discernible period-4 pattern.
fn detect_pattern_score(bytes: &[u8]) -> f64 {
    let n = bytes.len();
    if n < 8 {
        return 0.0;
    }

    // Count how many positions i where bytes[i] == bytes[i % 4]
    let pattern: [u8; 4] = [bytes[0], bytes[1], bytes[2], bytes[3]];
    let matches = bytes
        .iter()
        .enumerate()
        .filter(|(i, &b)| b == pattern[i % 4])
        .count();

    matches as f64 / n as f64
}

/// Compute the scalar fragmentation score for a BFV.
///
/// Weights: entropy (0.30) + nulls (0.20) + non-printable (0.30) + high bytes (0.20)
/// A fragmented block scores > δ_frag = 0.35.
pub fn fragmentation_score(bfv: &BinaryFeatureVector) -> f64 {
    bfv.byte_entropy * 0.30
        + bfv.null_density * 0.20
        + (1.0 - bfv.printable_ratio) * 0.30
        + bfv.high_byte_ratio * 0.20
}

/// Returns `true` if the block is fragmented (score > δ_frag).
///
/// Fragmented blocks are VGCA-Δ SUSPECT — they require steward review
/// before HeptaScore can run.
pub fn is_fragmented(bfv: &BinaryFeatureVector) -> bool {
    fragmentation_score(bfv) > DELTA_FRAG
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_block_gives_zero_bfv() {
        let bfv = compute_bfv(&[]);
        assert_eq!(bfv.as_array(), [0.0; 6]);
    }

    #[test]
    fn all_null_bytes_max_null_density() {
        let bfv = compute_bfv(&[0u8; 16]);
        assert!((bfv.null_density - 1.0).abs() < 1e-9);
        assert_eq!(bfv.printable_ratio, 0.0);
        assert_eq!(bfv.high_byte_ratio, 0.0);
    }

    #[test]
    fn printable_ascii_gives_full_printable_ratio() {
        let text = b"Hello World!";
        let bfv = compute_bfv(text);
        assert!(
            (bfv.printable_ratio - 1.0).abs() < 1e-9,
            "{}",
            bfv.printable_ratio
        );
        assert_eq!(bfv.null_density, 0.0);
        assert_eq!(bfv.high_byte_ratio, 0.0);
    }

    #[test]
    fn all_dimensions_in_unit_interval() {
        let cases: &[&[u8]] = &[
            &[],
            &[0u8; 4],
            b"hello",
            &[0xFF; 8],
            &[0x41; 16],
            &[0x00, 0x01, 0x02, 0x03, 0xF0, 0xE0, 0xD0, 0xC0],
        ];
        for &bytes in cases {
            let bfv = compute_bfv(bytes);
            for (i, &d) in bfv.as_array().iter().enumerate() {
                assert!(d >= 0.0 && d <= 1.0, "dim[{i}]={d} out of [0,1]");
            }
        }
    }

    #[test]
    fn delta_frag_constant_is_sovereign() {
        assert!(
            (DELTA_FRAG - 0.35).abs() < 1e-9,
            "δ_frag must be 0.35 (ADR-008)"
        );
    }

    #[test]
    fn clean_ascii_not_fragmented() {
        let bfv = compute_bfv(b"Hello World, this is clean ASCII text.");
        assert!(!is_fragmented(&bfv), "score={}", fragmentation_score(&bfv));
    }

    #[test]
    fn null_filled_block_is_fragmented() {
        let bfv = compute_bfv(&[0u8; 64]);
        assert!(is_fragmented(&bfv), "null block should be fragmented");
    }

    #[test]
    fn high_entropy_binary_block_is_fragmented() {
        // Binary-ish block with all 256 byte values represented
        let bytes: Vec<u8> = (0u8..=255).collect();
        let bfv = compute_bfv(&bytes);
        // High entropy + high byte ratio → fragmented
        assert!(is_fragmented(&bfv), "score={}", fragmentation_score(&bfv));
    }

    #[test]
    fn pattern_score_nonzero_for_periodic_block() {
        // Repeating [0xAB, 0xCD, 0xEF, 0x01] pattern
        let bytes: Vec<u8> = (0..32).map(|i| [0xABu8, 0xCD, 0xEF, 0x01][i % 4]).collect();
        let bfv = compute_bfv(&bytes);
        assert!(
            bfv.pattern_score > 0.9,
            "pattern_score={}",
            bfv.pattern_score
        );
    }

    #[test]
    fn run_length_norm_capped_at_one() {
        let bfv = compute_bfv(&[0xAAu8; 32]);
        assert!((bfv.run_length_norm - 1.0).abs() < 1e-9);
    }
}
