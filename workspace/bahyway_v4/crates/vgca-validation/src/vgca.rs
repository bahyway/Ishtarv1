//! VGCA — Vector Geometric Cleansing Analysis (ADR-008, March 2026).
//!
//! Three sub-algorithms:
//!   VGCA-Σ  Population-level field geometry outlier detection (text values)
//!   VGCA-Δ  Entropy-geometric block trajectory analysis (binary files)
//!   VGCA-Λ  Columnar manifold analysis at scale (Arrow columnar buffers)
//!
//! All algorithms are training-free, no-ML, pure-math geometry.
//! `#![forbid(unsafe_code)]` applies — no raw pointers, no SIMD intrinsics.

// ── Thresholds (§2.4, §3.3) ──────────────────────────────────────────────────

/// VGCA-Σ: GeometricFit::Clean threshold — value is column-consistent.
pub const CLEAN_THRESHOLD: f32 = 0.80;

/// VGCA-Σ: GeometricFit::Suspect lower bound (score < this → Outlier).
pub const SUSPECT_THRESHOLD: f32 = 0.50;

/// VGCA-Δ: Fragment detection threshold — empirically calibrated against ZIP/XLSX.
pub const DELTA_FRAG: f32 = 0.35;

/// VGCA-Σ: Outlier detection uses 3σ as the normalization divisor.
pub const SIGMA_MULTIPLIER: f32 = 3.0;

// ── VGCA-Σ: Feature Signature Vector (§2.2) ───────────────────────────────────

/// 7-dimensional Feature Signature Vector for a single text value.
///
/// Dimensions (all in [0.0, 1.0]):
///   d0  char_count_normalised       len(value) / max_len_in_column
///   d1  digit_density               count(digit) / len
///   d2  arabic_density              count(U+0600–U+06FF) / len
///   d3  latin_density               count(A–Za–z) / len
///   d4  punctuation_density         count(punctuation) / len
///   d5  word_count_normalised        word_count / max_words_in_column
///   d6  shannon_entropy_normalised  H(chars) / log₂(unique_chars + 1)
#[derive(Clone, Debug, PartialEq)]
pub struct FieldSignatureVector(pub [f32; 7]);

impl FieldSignatureVector {
    /// Euclidean distance between two FSVs.
    pub fn distance_to(&self, other: &Self) -> f32 {
        let sum: f32 = self
            .0
            .iter()
            .zip(other.0.iter())
            .map(|(a, b)| (a - b) * (a - b))
            .sum();
        sum.sqrt()
    }

    /// Compute FSV from a string value with column normalisation parameters.
    ///
    /// `max_len_chars` — maximum character count in the column (for d0).
    /// `max_words`     — maximum word count in the column (for d5).
    ///
    /// Returns `None` if `value` is empty.
    pub fn compute(value: &str, max_len_chars: usize, max_words: usize) -> Option<Self> {
        let len = value.chars().count();
        if len == 0 {
            return None;
        }

        let len_f = len as f32;

        // d0: char_count_normalised
        let d0 = if max_len_chars == 0 {
            0.0
        } else {
            (len as f32 / max_len_chars as f32).min(1.0)
        };

        // d1: digit_density
        let digits = value.chars().filter(|c| c.is_ascii_digit()).count();
        let d1 = digits as f32 / len_f;

        // d2: arabic_density (U+0600–U+06FF)
        let arabic = value
            .chars()
            .filter(|c| ('\u{0600}'..='\u{06FF}').contains(c))
            .count();
        let d2 = arabic as f32 / len_f;

        // d3: latin_density (A–Za–z)
        let latin = value.chars().filter(|c| c.is_ascii_alphabetic()).count();
        let d3 = latin as f32 / len_f;

        // d4: punctuation_density
        let punct = value.chars().filter(|c| c.is_ascii_punctuation()).count();
        let d4 = punct as f32 / len_f;

        // d5: word_count_normalised
        let words = value.split_whitespace().count();
        let d5 = if max_words == 0 {
            0.0
        } else {
            (words as f32 / max_words as f32).min(1.0)
        };

        // d6: shannon_entropy_normalised
        let d6 = normalised_shannon_entropy(value);

        Some(FieldSignatureVector([d0, d1, d2, d3, d4, d5, d6]))
    }
}

/// Shannon entropy of character distribution, normalised by log₂(unique_chars + 1).
fn normalised_shannon_entropy(s: &str) -> f32 {
    let mut freq = [0usize; 256];
    let mut total = 0usize;
    for b in s.bytes() {
        freq[b as usize] += 1;
        total += 1;
    }
    if total == 0 {
        return 0.0;
    }
    let unique = freq.iter().filter(|&&c| c > 0).count();
    let norm = (unique as f32 + 1.0).log2();
    if norm == 0.0 {
        return 0.0;
    }
    let h: f32 = freq
        .iter()
        .filter(|&&c| c > 0)
        .map(|&c| {
            let p = c as f32 / total as f32;
            -p * p.log2()
        })
        .sum();
    (h / norm).min(1.0)
}

// ── VGCA-Σ: Geometric Fit Classification (§2.4) ───────────────────────────────

/// Geometric fit verdict for a text value against its column population.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum GeometricFit {
    /// score ≥ 0.80 — column-consistent. KAKI issuance may proceed.
    Clean,
    /// score 0.50–0.79 — 1.5σ–3σ from centroid. Flag for Data Steward.
    Suspect,
    /// score < 0.50 — geometric outlier. Cleansing candidate.
    Outlier,
    /// score ≈ 0.0 — cross-field contamination. Route to BlackBoxWay; skip KAKI.
    Alien,
}

impl GeometricFit {
    pub fn as_str(self) -> &'static str {
        match self {
            GeometricFit::Clean => "CLEAN",
            GeometricFit::Suspect => "SUSPECT",
            GeometricFit::Outlier => "OUTLIER",
            GeometricFit::Alien => "ALIEN",
        }
    }

    pub fn requires_steward_review(self) -> bool {
        matches!(self, GeometricFit::Suspect | GeometricFit::Outlier)
    }

    pub fn blocks_kaki_issuance(self) -> bool {
        self == GeometricFit::Alien
    }
}

impl core::fmt::Display for GeometricFit {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Compute VGCA score from Euclidean distance and column standard deviation.
///
/// Formula: `vgca_score = 1.0 − clamp(distance / (3σ), 0.0, 1.0)`
/// - score = 1.0 → at population core
/// - score = 0.5 → 1.5σ from centroid
/// - score = 0.0 → ≥ 3σ from centroid (geometric outlier)
pub fn vgca_score(distance: f32, sigma: f32) -> f32 {
    if sigma == 0.0 {
        return 1.0;
    } // degenerate: all values identical → perfect fit
    1.0 - (distance / (SIGMA_MULTIPLIER * sigma)).clamp(0.0, 1.0)
}

/// Classify a vgca score into a `GeometricFit` verdict.
pub fn geometric_fit(score: f32) -> GeometricFit {
    if score >= CLEAN_THRESHOLD {
        GeometricFit::Clean
    } else if score >= SUSPECT_THRESHOLD {
        GeometricFit::Suspect
    } else if score > 0.005 {
        GeometricFit::Outlier
    } else {
        GeometricFit::Alien
    }
}

/// Full VGCA-Σ result for a single text value (§5.1).
#[derive(Clone, Debug)]
pub struct VgcaTextResult {
    pub fsv: FieldSignatureVector,
    /// Euclidean distance from the column centroid FSV.
    pub distance: f32,
    /// Distance expressed in σ multiples.
    pub sigma_multiple: f32,
    /// Geometric fit score ∈ [0.0, 1.0].
    pub score: f32,
    pub fit: GeometricFit,
    /// Index of the FSV dimension (0–6) that drove the detection, if identifiable.
    pub outlier_driver: Option<u8>,
}

impl VgcaTextResult {
    /// Compute the full VGCA-Σ result for a value given the column centroid and sigma.
    pub fn compute(fsv: FieldSignatureVector, centroid: &FieldSignatureVector, sigma: f32) -> Self {
        let distance = fsv.distance_to(centroid);
        let sigma_multiple = if sigma == 0.0 { 0.0 } else { distance / sigma };
        let score = vgca_score(distance, sigma);
        let fit = geometric_fit(score);

        // identify the dimension with the largest deviation from centroid
        let outlier_driver = if fit != GeometricFit::Clean {
            fsv.0
                .iter()
                .zip(centroid.0.iter())
                .enumerate()
                .max_by(|(_, (a1, b1)), (_, (a2, b2))| {
                    let d1 = (*a1 - *b1).abs();
                    let d2 = (*a2 - *b2).abs();
                    d1.partial_cmp(&d2).unwrap()
                })
                .map(|(i, _)| i as u8)
        } else {
            None
        };

        VgcaTextResult {
            fsv,
            distance,
            sigma_multiple,
            score,
            fit,
            outlier_driver,
        }
    }
}

// ── VGCA-Δ: Block Feature Vector (§3.2) ───────────────────────────────────────

/// 6-dimensional Block Feature Vector for a binary file block.
///
/// Dimensions:
///   e0  block_entropy              Shannon_H(bytes) / 8.0
///   e1  byte_class_balance         1 − |P(printable) − P(binary)|
///   e2  null_byte_density          count(0x00) / block_size
///   e3  magic_byte_score           match_score(block[0..8], known_magic_headers)
///   e4  high_byte_density          count(bytes > 0x7F) / block_size
///   e5  sequential_prediction_error  ||BFV(i) − BFV(i-1)||
#[derive(Clone, Debug, PartialEq)]
pub struct BlockFeatureVector(pub [f32; 6]);

impl BlockFeatureVector {
    /// Euclidean distance between two BFVs.
    pub fn distance_to(&self, other: &Self) -> f32 {
        let sum: f32 = self
            .0
            .iter()
            .zip(other.0.iter())
            .map(|(a, b)| (a - b) * (a - b))
            .sum();
        sum.sqrt()
    }

    /// Compute BFV from a raw byte block.  `prev_bfv` is for computing e5.
    pub fn compute(block: &[u8], prev_bfv: Option<&Self>) -> Self {
        if block.is_empty() {
            return BlockFeatureVector([0.0; 6]);
        }
        let n = block.len() as f32;

        // e0: block_entropy
        let e0 = block_entropy(block) / 8.0;

        // e1: byte_class_balance
        let printable = block
            .iter()
            .filter(|&&b| (0x20..=0x7E).contains(&b))
            .count();
        let binary = block
            .iter()
            .filter(|&&b| !(0x20..=0x7E).contains(&b))
            .count();
        let e1 = 1.0 - ((printable as f32 / n) - (binary as f32 / n)).abs();

        // e2: null_byte_density
        let nulls = block.iter().filter(|&&b| b == 0x00).count();
        let e2 = nulls as f32 / n;

        // e3: magic_byte_score (ZIP=PK\x03\x04, Parquet=PAR1, XLSX=PK\x03\x04)
        let e3 = magic_score(block);

        // e4: high_byte_density (bytes > 0x7F — multi-byte UTF-8, cuneiform)
        let high = block.iter().filter(|&&b| b > 0x7F).count();
        let e4 = high as f32 / n;

        // e5: sequential prediction error from previous block
        let e5 = if let Some(prev) = prev_bfv {
            let current = BlockFeatureVector([e0, e1, e2, e3, e4, 0.0]);
            let dist = current.distance_to(prev);
            dist.min(1.0)
        } else {
            0.0
        };

        BlockFeatureVector([e0, e1, e2, e3, e4, e5])
    }
}

fn block_entropy(block: &[u8]) -> f32 {
    let mut freq = [0usize; 256];
    for &b in block {
        freq[b as usize] += 1;
    }
    let n = block.len() as f32;
    freq.iter()
        .filter(|&&c| c > 0)
        .map(|&c| {
            let p = c as f32 / n;
            -p * p.log2()
        })
        .sum()
}

fn magic_score(block: &[u8]) -> f32 {
    if block.len() < 4 {
        return 0.0;
    }
    // ZIP/XLSX: PK\x03\x04
    if block[0] == 0x50 && block[1] == 0x4B && block[2] == 0x03 && block[3] == 0x04 {
        return 1.0;
    }
    // Parquet: PAR1
    if block.len() >= 4 && &block[0..4] == b"PAR1" {
        return 1.0;
    }
    // PDF: %PDF
    if block.len() >= 4 && &block[0..4] == b"%PDF" {
        return 0.9;
    }
    // PNG: \x89PNG
    if block.len() >= 4 && block[0] == 0x89 && &block[1..4] == b"PNG" {
        return 0.9;
    }
    0.0
}

/// Full VGCA-Δ result for a binary file (§5.1).
#[derive(Clone, Debug)]
pub struct VgcaBlockResult {
    /// Number of fragment boundaries detected (e5 > DELTA_FRAG).
    pub fragment_count: usize,
    /// Fragmentation ratio (fragment_count / total_blocks). 0.0 = contiguous.
    pub fragmentation_ratio: f32,
    /// `true` if block entropy profile matches ZIP bomb signature.
    pub zip_bomb_suspected: bool,
    /// Block index of the first detected anomaly, if any.
    pub first_anomaly_block: Option<usize>,
}

/// Compute VGCA-Δ result for a sequence of pre-computed BFVs.
pub fn vgca_delta(bfvs: &[BlockFeatureVector]) -> VgcaBlockResult {
    if bfvs.is_empty() {
        return VgcaBlockResult {
            fragment_count: 0,
            fragmentation_ratio: 0.0,
            zip_bomb_suspected: false,
            first_anomaly_block: None,
        };
    }
    let total = bfvs.len();
    let mut fragment_count = 0usize;
    let mut first_anomaly = None;

    for (i, bfv) in bfvs.iter().enumerate().skip(1) {
        let e5 = bfv.0[5];
        if e5 > DELTA_FRAG {
            fragment_count += 1;
            if first_anomaly.is_none() {
                first_anomaly = Some(i);
            }
        }
    }

    // ZIP bomb: mean block_entropy across all blocks is suspiciously high (> 7.9 / 8.0)
    let mean_entropy: f32 = bfvs.iter().map(|b| b.0[0]).sum::<f32>() / total as f32;
    let zip_bomb_suspected = mean_entropy > (7.9 / 8.0);

    let fragmentation_ratio = if total <= 1 {
        0.0
    } else {
        fragment_count as f32 / (total - 1) as f32
    };

    VgcaBlockResult {
        fragment_count,
        fragmentation_ratio,
        zip_bomb_suspected,
        first_anomaly_block: first_anomaly,
    }
}

// ── VGCA-Λ: Column Geometry Descriptor (§4.2) ────────────────────────────────

/// Compressed geometric summary of an entire column's value population (VGCA-Λ).
#[derive(Clone, Debug)]
pub struct ColumnGeometryDescriptor {
    pub column_id: String,
    /// Mean FSV across all column values.
    pub centroid: [f32; 7],
    /// Population standard deviation in FSV space.
    pub spread: f32,
    /// Distribution skewness (0 = symmetric; +ve = right tail; −ve = left tail).
    pub skewness: f32,
    /// Distribution kurtosis (3 = normal; > 3 = heavy tails / outlier-prone).
    pub kurtosis: f32,
    /// Cluster count (1 = homogeneous; > 3 = mixed content, structural warning).
    pub cluster_count: u8,
    /// Count of values with vgca_score < SUSPECT_THRESHOLD.
    pub outlier_count: u64,
    /// Total value count.
    pub total_count: u64,
    /// Autonomously inferred column type from CGD pattern.
    pub inferred_type: InferredColumnType,
}

impl ColumnGeometryDescriptor {
    /// Compute a CGD from a slice of FSVs.
    pub fn from_fsvs(column_id: String, fsvs: &[FieldSignatureVector]) -> Self {
        if fsvs.is_empty() {
            return ColumnGeometryDescriptor {
                column_id,
                centroid: [0.0; 7],
                spread: 0.0,
                skewness: 0.0,
                kurtosis: 0.0,
                cluster_count: 1,
                outlier_count: 0,
                total_count: 0,
                inferred_type: InferredColumnType::Unknown,
            };
        }
        let n = fsvs.len() as f32;

        // centroid
        let mut centroid = [0.0f32; 7];
        for fsv in fsvs {
            for (i, v) in fsv.0.iter().enumerate() {
                centroid[i] += v / n;
            }
        }
        let centroid_fsv = FieldSignatureVector(centroid);

        // spread (population σ)
        let spread = {
            let sum_sq: f32 = fsvs
                .iter()
                .map(|fsv| fsv.distance_to(&centroid_fsv).powi(2))
                .sum();
            (sum_sq / n).sqrt()
        };

        // outlier count (score < SUSPECT_THRESHOLD)
        let outlier_count = fsvs
            .iter()
            .filter(|fsv| {
                let d = fsv.distance_to(&centroid_fsv);
                vgca_score(d, spread) < SUSPECT_THRESHOLD
            })
            .count() as u64;

        // skewness and kurtosis (1D approximation on overall distance)
        let (skewness, kurtosis) = {
            let distances: Vec<f32> = fsvs
                .iter()
                .map(|fsv| fsv.distance_to(&centroid_fsv))
                .collect();
            let mean = distances.iter().sum::<f32>() / n;
            let variance = distances.iter().map(|d| (d - mean).powi(2)).sum::<f32>() / n;
            let std_dev = variance.sqrt();
            if std_dev == 0.0 {
                (0.0, 3.0)
            } else {
                let skew = distances
                    .iter()
                    .map(|d| ((d - mean) / std_dev).powi(3))
                    .sum::<f32>()
                    / n;
                let kurt = distances
                    .iter()
                    .map(|d| ((d - mean) / std_dev).powi(4))
                    .sum::<f32>()
                    / n;
                (skew, kurt)
            }
        };

        // cluster_count: simplified heuristic — 1 if spread < 0.15, else proportional
        let cluster_count = if spread < 0.15 {
            1
        } else if spread < 0.30 {
            2
        } else if spread < 0.40 {
            3
        } else {
            // 2026-08-24, found live (clippy::min_max, hard-deny): this was
            // `4u8.min(x.max(4))`, which always evaluates to the constant 4
            // regardless of x -- min(4, anything >= 4) is always 4, so the
            // intended scaling by spread never actually took effect. Kept
            // the existing (already-constant) runtime behavior rather than
            // invent a new upper bound the Architect hasn't specified.
            4u8
        };

        let inferred_type = infer_column_type(&centroid, spread, cluster_count);

        ColumnGeometryDescriptor {
            column_id,
            centroid,
            spread,
            skewness,
            kurtosis,
            cluster_count,
            outlier_count,
            total_count: fsvs.len() as u64,
            inferred_type,
        }
    }
}

/// Autonomous schema inference output (§4.3).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum InferredColumnType {
    ArabicNameField,
    PhoneNumber,
    IraqiNationalId,
    DateOrTimestamp,
    HijriDate,
    NumericAmount,
    FreeText,
    GeographicCode,
    /// cluster_count > 3 — structural warning, flag for steward.
    MixedContent,
    Unknown,
}

impl InferredColumnType {
    pub fn as_str(self) -> &'static str {
        match self {
            InferredColumnType::ArabicNameField => "ARABIC_NAME_FIELD",
            InferredColumnType::PhoneNumber => "PHONE_NUMBER",
            InferredColumnType::IraqiNationalId => "IRAQI_NATIONAL_ID",
            InferredColumnType::DateOrTimestamp => "DATE_OR_TIMESTAMP",
            InferredColumnType::HijriDate => "HIJRI_DATE",
            InferredColumnType::NumericAmount => "NUMERIC_AMOUNT",
            InferredColumnType::FreeText => "FREE_TEXT",
            InferredColumnType::GeographicCode => "GEOGRAPHIC_CODE",
            InferredColumnType::MixedContent => "MIXED_CONTENT",
            InferredColumnType::Unknown => "UNKNOWN",
        }
    }
}

/// Autonomous column type inference from CGD pattern (§4.3).
pub fn infer_column_type(
    centroid: &[f32; 7],
    spread: f32,
    cluster_count: u8,
) -> InferredColumnType {
    let d1 = centroid[1]; // digit_density
    let d2 = centroid[2]; // arabic_density
    let d3 = centroid[3]; // latin_density (unused directly but available)
    let d4 = centroid[4]; // punctuation_density
    let d0 = centroid[0]; // char_count_normalised

    let _ = d3; // reserved for future rules

    if cluster_count > 3 {
        return InferredColumnType::MixedContent;
    }

    // Phone number: high digit + punctuation separators
    if d1 > 0.80 && d4 > 0.10 {
        return InferredColumnType::PhoneNumber;
    }

    // Iraqi National ID: digit-dense, ~12 chars, no separators
    if d1 > 0.80 && (d0 - 0.12).abs() < 0.05 {
        return InferredColumnType::IraqiNationalId;
    }

    // Numeric amount: near-pure digits, no separators
    if d1 > 0.95 && d4 < 0.05 {
        return InferredColumnType::NumericAmount;
    }

    // Date/timestamp: punctuation separators + digit density
    if d4 > 0.30 && d1 > 0.50 {
        return InferredColumnType::DateOrTimestamp;
    }

    // Arabic name: Arabic-dominant, tight cluster
    if d2 > 0.70 && spread < 0.15 {
        return InferredColumnType::ArabicNameField;
    }

    // Arabic mixed content: Arabic column with wide spread
    if d2 > 0.70 && spread > 0.40 {
        return InferredColumnType::MixedContent;
    }

    // Free text: long values, diverse population
    if d0 > 0.50 {
        return InferredColumnType::FreeText;
    }

    InferredColumnType::Unknown
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── FSV computation ───────────────────────────────────────────────────────

    #[test]
    fn fsv_empty_returns_none() {
        assert!(FieldSignatureVector::compute("", 100, 10).is_none());
    }

    #[test]
    fn fsv_pure_digits_high_d1() {
        let fsv = FieldSignatureVector::compute("0501234567", 20, 5).unwrap();
        assert!(
            fsv.0[1] > 0.8,
            "digit_density should be high, got {}",
            fsv.0[1]
        );
    }

    #[test]
    fn fsv_arabic_name_high_d2() {
        let fsv = FieldSignatureVector::compute("محمد علي", 20, 5).unwrap();
        assert!(
            fsv.0[2] > 0.5,
            "arabic_density should be high, got {}",
            fsv.0[2]
        );
    }

    #[test]
    fn fsv_latin_name_high_d3() {
        let fsv = FieldSignatureVector::compute("John Smith", 20, 5).unwrap();
        assert!(
            fsv.0[3] > 0.5,
            "latin_density should be high, got {}",
            fsv.0[3]
        );
    }

    #[test]
    fn fsv_all_dimensions_in_range() {
        let fsv = FieldSignatureVector::compute("Hello World 123!", 50, 10).unwrap();
        for (i, v) in fsv.0.iter().enumerate() {
            assert!(*v >= 0.0 && *v <= 1.0, "d{i} = {v} out of [0,1]");
        }
    }

    #[test]
    fn fsv_distance_to_self_is_zero() {
        let fsv = FieldSignatureVector::compute("test", 20, 5).unwrap();
        assert!((fsv.distance_to(&fsv) - 0.0).abs() < 1e-6);
    }

    // ── vgca_score ────────────────────────────────────────────────────────────

    #[test]
    fn vgca_score_at_centroid_is_one() {
        assert_eq!(vgca_score(0.0, 1.0), 1.0);
    }

    #[test]
    fn vgca_score_at_3sigma_is_zero() {
        assert_eq!(vgca_score(3.0, 1.0), 0.0);
    }

    #[test]
    fn vgca_score_zero_sigma_returns_one() {
        assert_eq!(vgca_score(5.0, 0.0), 1.0);
    }

    #[test]
    fn vgca_score_clamped() {
        assert!(vgca_score(100.0, 1.0) >= 0.0);
        assert!(vgca_score(0.0, 1.0) <= 1.0);
    }

    // ── geometric_fit ─────────────────────────────────────────────────────────

    #[test]
    fn geometric_fit_classification() {
        assert_eq!(geometric_fit(1.0), GeometricFit::Clean);
        assert_eq!(geometric_fit(0.80), GeometricFit::Clean);
        assert_eq!(geometric_fit(0.79), GeometricFit::Suspect);
        assert_eq!(geometric_fit(0.50), GeometricFit::Suspect);
        assert_eq!(geometric_fit(0.49), GeometricFit::Outlier);
        assert_eq!(geometric_fit(0.01), GeometricFit::Outlier);
        assert_eq!(geometric_fit(0.00), GeometricFit::Alien);
    }

    #[test]
    fn alien_blocks_kaki() {
        assert!(GeometricFit::Alien.blocks_kaki_issuance());
        assert!(!GeometricFit::Clean.blocks_kaki_issuance());
    }

    #[test]
    fn suspect_outlier_require_steward() {
        assert!(GeometricFit::Suspect.requires_steward_review());
        assert!(GeometricFit::Outlier.requires_steward_review());
        assert!(!GeometricFit::Clean.requires_steward_review());
    }

    // ── VGCA-Δ ───────────────────────────────────────────────────────────────

    #[test]
    fn bfv_zip_magic_scores_one() {
        let zip_header: &[u8] = &[0x50, 0x4B, 0x03, 0x04, 0x00, 0x00, 0x00, 0x00];
        let bfv = BlockFeatureVector::compute(zip_header, None);
        assert_eq!(bfv.0[3], 1.0, "magic_byte_score should be 1.0 for ZIP");
    }

    #[test]
    fn bfv_empty_block() {
        let bfv = BlockFeatureVector::compute(&[], None);
        assert_eq!(bfv.0, [0.0; 6]);
    }

    #[test]
    fn vgca_delta_no_blocks() {
        let result = vgca_delta(&[]);
        assert_eq!(result.fragment_count, 0);
        assert_eq!(result.fragmentation_ratio, 0.0);
        assert!(!result.zip_bomb_suspected);
    }

    #[test]
    fn vgca_delta_single_block_no_fragmentation() {
        let bfv = BlockFeatureVector([0.5, 0.5, 0.0, 0.0, 0.0, 0.0]);
        let result = vgca_delta(&[bfv]);
        assert_eq!(result.fragment_count, 0);
        assert_eq!(result.fragmentation_ratio, 0.0);
    }

    // ── VGCA-Λ / CGD ─────────────────────────────────────────────────────────

    #[test]
    fn cgd_empty_column() {
        let cgd = ColumnGeometryDescriptor::from_fsvs("test".to_string(), &[]);
        assert_eq!(cgd.total_count, 0);
        assert_eq!(cgd.inferred_type, InferredColumnType::Unknown);
    }

    #[test]
    fn cgd_homogeneous_zero_spread() {
        let fsv = FieldSignatureVector([0.5; 7]);
        let fsvs = vec![fsv.clone(), fsv.clone(), fsv.clone()];
        let cgd = ColumnGeometryDescriptor::from_fsvs("col".to_string(), &fsvs);
        assert!(cgd.spread < 1e-5);
        assert_eq!(cgd.outlier_count, 0);
        assert_eq!(cgd.cluster_count, 1);
    }

    #[test]
    fn infer_phone_number() {
        // d1=0.90, d4=0.15 → PhoneNumber
        let centroid = [0.1, 0.90, 0.0, 0.0, 0.15, 0.1, 0.5];
        assert_eq!(
            infer_column_type(&centroid, 0.10, 1),
            InferredColumnType::PhoneNumber
        );
    }

    #[test]
    fn infer_arabic_name() {
        // d2=0.80, spread < 0.15 → ArabicNameField
        let centroid = [0.3, 0.0, 0.80, 0.0, 0.02, 0.3, 0.5];
        assert_eq!(
            infer_column_type(&centroid, 0.10, 1),
            InferredColumnType::ArabicNameField
        );
    }

    #[test]
    fn infer_mixed_content_from_cluster_count() {
        let centroid = [0.3; 7];
        assert_eq!(
            infer_column_type(&centroid, 0.30, 4),
            InferredColumnType::MixedContent
        );
    }

    #[test]
    fn inferred_type_display() {
        assert_eq!(
            InferredColumnType::ArabicNameField.as_str(),
            "ARABIC_NAME_FIELD"
        );
        assert_eq!(InferredColumnType::PhoneNumber.as_str(), "PHONE_NUMBER");
        assert_eq!(InferredColumnType::MixedContent.as_str(), "MIXED_CONTENT");
    }
}

// ── CorruptionClass — VGCA-Δ failure modes (ADR-008 §4.4) ────────────────────

/// Corruption failure modes introduced by VGCA-Δ for BlackBoxWay routing.
///
/// These three variants detect errors invisible to CRC32, BLAKE3, and
/// Shannon entropy alone.  When detected, the file is routed to the
/// BlackBoxWay processing path.
///
/// Severity levels follow the ADR-008 CorruptionClass severity scale:
///   3 = DriveFragmentationSevere
///   4 = BlockTrajectoryAnomaly
///   5 = CompressedPayloadGeometricAnomaly (highest — ZIP bomb / payload injection)
#[derive(Clone, Debug)]
pub enum CorruptionClass {
    /// File block sequence has geometric discontinuities inconsistent with the
    /// declared format.  Indicates physical fragmentation or block injection
    /// (ZIP bomb, payload injection).  Severity: 4.
    BlockTrajectoryAnomaly {
        /// Block index of the first detected discontinuity.
        first_discontinuity_block: usize,
        /// Total count of detected fragment boundaries.
        fragment_count: usize,
        /// Maximum fragmentation score observed (0.0–1.0).
        max_fragment_score: f32,
    },

    /// ZIP / XLSX / gzip block entropy profile matches a ZIP bomb signature:
    /// compressed blocks with entropy too high to produce valid decompressed
    /// output.  Severity: 5 (highest).
    CompressedPayloadGeometricAnomaly {
        /// Mean entropy across all compressed data blocks.
        mean_compressed_entropy: f32,
        /// Expected entropy range [min, max] for this format.
        expected_entropy_range: [f32; 2],
    },

    /// Drive block sequence geometry indicates physical storage fragmentation
    /// severe enough to prevent reliable parsing at current read order.
    /// Severity: 3.
    DriveFragmentationSevere {
        /// Estimated fragment count based on discontinuity count.
        fragment_estimate: usize,
        /// Fragmentation ratio (0.0 = contiguous, 1.0 = fully scattered).
        fragmentation_ratio: f32,
    },
}

impl CorruptionClass {
    /// Severity level (3–5) per ADR-008 CorruptionClass severity scale.
    pub fn severity(&self) -> u8 {
        match self {
            CorruptionClass::DriveFragmentationSevere { .. } => 3,
            CorruptionClass::BlockTrajectoryAnomaly { .. } => 4,
            CorruptionClass::CompressedPayloadGeometricAnomaly { .. } => 5,
        }
    }

    /// Returns `true` when this class should trigger immediate BlackBoxWay routing
    /// (severity ≥ 4).
    pub fn requires_blackbox_routing(&self) -> bool {
        self.severity() >= 4
    }

    /// Classify a `VgcaBlockResult` into a `CorruptionClass` if the result
    /// indicates corruption.  Returns `None` if the result is clean.
    pub fn from_vgca_block_result(result: &VgcaBlockResult) -> Option<Self> {
        if result.zip_bomb_suspected {
            // severity 5: ZIP bomb profile
            return Some(CorruptionClass::CompressedPayloadGeometricAnomaly {
                mean_compressed_entropy: 0.99, // near-maximum entropy
                expected_entropy_range: [0.60, 0.95],
            });
        }
        if result.fragmentation_ratio > 0.70 {
            // severity 3: severe drive fragmentation
            return Some(CorruptionClass::DriveFragmentationSevere {
                fragment_estimate: result.fragment_count,
                fragmentation_ratio: result.fragmentation_ratio,
            });
        }
        if result.fragment_count > 0 {
            // severity 4: block trajectory anomaly
            let max_score = if result.fragmentation_ratio > 0.0 {
                result.fragmentation_ratio
            } else {
                DELTA_FRAG + 0.01
            };
            return Some(CorruptionClass::BlockTrajectoryAnomaly {
                first_discontinuity_block: result.first_anomaly_block.unwrap_or(0),
                fragment_count: result.fragment_count,
                max_fragment_score: max_score,
            });
        }
        None
    }
}

#[cfg(test)]
mod corruption_tests {
    use super::*;

    #[test]
    fn severity_ordering() {
        assert_eq!(
            CorruptionClass::DriveFragmentationSevere {
                fragment_estimate: 5,
                fragmentation_ratio: 0.80
            }
            .severity(),
            3
        );

        assert_eq!(
            CorruptionClass::BlockTrajectoryAnomaly {
                first_discontinuity_block: 2,
                fragment_count: 3,
                max_fragment_score: 0.40
            }
            .severity(),
            4
        );

        assert_eq!(
            CorruptionClass::CompressedPayloadGeometricAnomaly {
                mean_compressed_entropy: 0.99,
                expected_entropy_range: [0.60, 0.95]
            }
            .severity(),
            5
        );
    }

    #[test]
    fn blackbox_routing_threshold() {
        let anomaly = CorruptionClass::BlockTrajectoryAnomaly {
            first_discontinuity_block: 0,
            fragment_count: 1,
            max_fragment_score: 0.4,
        };
        assert!(anomaly.requires_blackbox_routing());

        let frag = CorruptionClass::DriveFragmentationSevere {
            fragment_estimate: 3,
            fragmentation_ratio: 0.8,
        };
        assert!(!frag.requires_blackbox_routing());
    }

    #[test]
    fn classify_zip_bomb() {
        let result = VgcaBlockResult {
            fragment_count: 0,
            fragmentation_ratio: 0.0,
            zip_bomb_suspected: true,
            first_anomaly_block: None,
        };
        let class = CorruptionClass::from_vgca_block_result(&result).unwrap();
        assert_eq!(class.severity(), 5);
    }

    #[test]
    fn classify_severe_fragmentation() {
        let result = VgcaBlockResult {
            fragment_count: 50,
            fragmentation_ratio: 0.80,
            zip_bomb_suspected: false,
            first_anomaly_block: Some(2),
        };
        let class = CorruptionClass::from_vgca_block_result(&result).unwrap();
        assert_eq!(class.severity(), 3);
    }

    #[test]
    fn classify_mild_anomaly() {
        let result = VgcaBlockResult {
            fragment_count: 3,
            fragmentation_ratio: 0.15,
            zip_bomb_suspected: false,
            first_anomaly_block: Some(5),
        };
        let class = CorruptionClass::from_vgca_block_result(&result).unwrap();
        assert_eq!(class.severity(), 4);
    }

    #[test]
    fn clean_result_returns_none() {
        let result = VgcaBlockResult {
            fragment_count: 0,
            fragmentation_ratio: 0.0,
            zip_bomb_suspected: false,
            first_anomaly_block: None,
        };
        assert!(CorruptionClass::from_vgca_block_result(&result).is_none());
    }
}
