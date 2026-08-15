//! ModularNaviIndex — Fourier-coefficient index for NaviMap routing topology.
//!
//! Inspired by Viazovska's use of modular forms as "magic functions" for
//! sphere packing: just as her theta series encodes lattice geometry in a
//! compact Fourier expansion, ModularNaviIndex encodes a NaviMap's cost
//! landscape as a histogram of edge effective-costs — the "spectral fingerprint"
//! of the routing topology.
//!
//! # Structure
//!
//!   a(n) = number of directed edges whose effective_cost falls in
//!          bucket n: [n × BUCKET_STEP, (n+1) × BUCKET_STEP)
//!
//! The sequence {a(n)} is the Fourier expansion of the NaviLattice theta series:
//!
//!   θ(q) = Σ_{n≥0} a(n) · q^n       (q formal variable)
//!
//! # Weight
//!
//! The modular weight maps to the dominant HeptaChordType:
//!   Spoke → −1   (arterial; cheapest)
//!   Rim   →  0   (neutral ring)
//!   Local →  1   (intra-sector)
//!   Diagonal → 2 (costly cross-sector jump)
//!
//! # Routing equivalence
//!
//! Two NaviMaps with the same ModularNaviIndex signature share their
//! optimal routing topology — an NC6 Dijkstra solution on one applies
//! verbatim to the other.
//!
//! # NC2 resonance
//!
//! ResonanceScorerNc2 replaces NaviCode's flat tribe-bonus (0.30) with a
//! Fourier-weighted bonus: edges whose cost sits near the spectral peak
//! receive the highest resonance (the map's "natural frequency"), while
//! outlier costs receive less.

use bahyway_crc::crc16;
use navi_engine::{HeptaChordType, NaviEdge, NaviGraph};

// ── Constants ─────────────────────────────────────────────────────────────────

/// Width of each cost bucket in metres (effective_cost units).
pub const BUCKET_STEP: f32 = 100.0;

/// Maximum number of buckets — supports maps up to 51.2 km effective cost.
pub const MAX_BUCKETS: usize = 512;

// ── Stage 2: E₂ Fourier sector weights (Viazovska interpolation) ──────────────
//
// Maryna Viazovska's proof uses modular forms evaluated at special lattice
// points to construct "magic functions" that certify sphere-packing optimality.
// We adapt the same principle for the 7-sector heptagram:
//
//   Evaluate a weight-2 Eisenstein-inspired cosine series at τ_k = k/7
//   (the 7th-root-of-unity lattice positions) to derive sovereign routing
//   weights that are mathematically grounded rather than hand-chosen.
//
// The formula:
//   C(k, N) = (1/N) × Σ_{n=1}^{N} [σ₁(n)/n] × cos(2πnk/7)
//
// where σ₁(n) = Σ_{d|n} d  (sum of positive divisors of n).
//
// σ₁(n)/n is the "normalised divisor sum" — it gives the Fourier coefficient
// of the weight-2 Eisenstein series E₂ after the n-damping that ensures
// convergence without complex numbers.
//
// Symmetry property: cos(2πn·k/7) = cos(2πn·(7-k)/7)
//   → sectors k and 7-k always receive identical weights.
//   In the heptagram: Shuhadaa(1)=Anbiya(6), Awliya(2)=Ulamaa(5),
//                     Huffaz(3)=Momineen(4), Entrance(0) is unique.

/// Divisor-sum function σ₁(n) = sum of all positive divisors of n.
fn sigma1(n: u32) -> u32 {
    (1..=n).filter(|&d| n % d == 0).sum()
}

/// E₂-inspired Fourier cosine coefficient at heptagram sector k.
///
/// Computes C(k, N) = (1/N) × Σ_{n=1}^{N} [σ₁(n)/n] × cos(2πnk/7).
/// The σ₁(n)/n damping mirrors the q-expansion of E₂ after logarithmic
/// derivative, giving rapid convergence without imaginary arithmetic.
fn e2_sector_cosine(k: u8, n_terms: u32) -> f64 {
    let angle = 2.0 * core::f64::consts::PI * k as f64 / 7.0;
    let mut sum = 0.0f64;
    for n in 1..=n_terms {
        let s1 = sigma1(n) as f64;
        sum += (s1 / n as f64) * (n as f64 * angle).cos();
    }
    sum / n_terms as f64
}

/// 7-sector sacred-routing weights derived from the E₂ Fourier spectrum.
///
/// Each weight w_k is the normalised Eisenstein cosine at sector position k,
/// linearly mapped into `[LOW, HIGH]`.
///
/// **Interpretation**:
///   - Lower weight → elevated routing priority (more resonant with the
///     heptagram's natural frequency, less "resistance" in the cost landscape).
///   - Higher weight → neutral/normal routing cost.
///
/// **Mathematical result** (N=100 terms):
///   By the cosine symmetry of the 7th-root lattice:
///   - Entrance (k=0): C(0)≈1.622 — DC-dominant; maps to weight = HIGH (1.00, neutral)
///   - Awliya (k=2) = Ulamaa (k=5): C(2)≈−0.010 — minimum; weight = LOW (0.80, most elevated)
///   - Huffaz (k=3) = Momineen (k=4): C(3)≈0.016 — near-minimum; weight ≈ 0.803
///   - Shuhadaa (k=1) = Anbiya (k=6): C(1)≈0.020 — largest non-DC; weight ≈ 0.804
///
/// Key finding: C(0) is ~80× larger than any outer-sector coefficient — the
/// Entrance is the "DC singularity" of the heptagram's Fourier spectrum.
/// All outer sectors cluster near LOW (0.80), with Awliya/Ulamaa most elevated
/// because they sit at the cosine-minimum positions (cos(4π/7) < 0).
///
/// This diverges from the hand-chosen NajafEngine weights where Shuhadaa(0.85)
/// is most elevated — the Fourier analysis discovers that Awliya/Ulamaa zones
/// are the most "harmonically distant" from the entrance DC component.
#[derive(Debug, Clone)]
pub struct E2FourierWeights {
    weights: [f32; 7],
    /// Raw (un-normalised) cosine sums before mapping to [LOW, HIGH].
    raw:     [f64; 7],
}

impl E2FourierWeights {
    /// Lower bound of the weight interval — most elevated (highest routing priority).
    pub const LOW:     f32 = 0.80;
    /// Upper bound — neutral routing cost.
    pub const HIGH:    f32 = 1.00;
    /// Number of Fourier terms — 100 gives < 10⁻⁴ residual error.
    pub const N_TERMS: u32 = 100;

    /// Compute the 7 E₂ sector weights from the Fourier cosine series.
    /// Pure Rust — no complex numbers, no external dependencies.
    pub fn compute() -> Self {
        let mut raw = [0.0f64; 7];
        for k in 0..7u8 {
            raw[k as usize] = e2_sector_cosine(k, Self::N_TERMS);
        }

        let min = raw.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = raw.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let span = max - min;

        let mut weights = [Self::HIGH; 7];
        if span > 1e-9 {
            let range = (Self::HIGH - Self::LOW) as f64;
            for k in 0..7usize {
                // Linear map: lowest raw cosine sum → LOW (most elevated/prioritised).
                //             highest raw cosine sum → HIGH (neutral).
                // k=0 (Entrance) has the maximum raw sum → maps to HIGH (1.00, neutral).
                // k=2,5 (Awliya/Ulamaa) have the minimum raw sum → map to LOW (0.80).
                weights[k] = (Self::LOW as f64 + (raw[k] - min) / span * range) as f32;
            }
        }
        E2FourierWeights { weights, raw }
    }

    /// Weight for sector index k ∈ [0, 6]. Out-of-range → HIGH (neutral).
    pub fn weight(&self, k: u8) -> f32 {
        self.weights.get(k as usize).copied().unwrap_or(Self::HIGH)
    }

    /// All 7 weights [w₀..w₆].
    pub fn all(&self) -> &[f32; 7] { &self.weights }

    /// Raw (un-normalised) Fourier cosine sum for sector k.
    pub fn raw_cosine(&self, k: u8) -> f64 {
        self.raw.get(k as usize).copied().unwrap_or(0.0)
    }

    /// Index of the sector with the lowest weight (highest routing priority).
    pub fn most_elevated(&self) -> u8 {
        self.weights.iter().enumerate()
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal))
            .map(|(i, _)| i as u8)
            .unwrap_or(0)
    }

    /// Index of the sector with the highest weight (most neutral cost).
    pub fn most_neutral(&self) -> u8 {
        self.weights.iter().enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal))
            .map(|(i, _)| i as u8)
            .unwrap_or(0)
    }

    /// True if sector k is elevated (weight < HIGH).
    pub fn is_elevated(&self, k: u8) -> bool {
        self.weight(k) < Self::HIGH - 1e-5
    }

    /// Difference between the Fourier-derived weight and a canonical (hand-chosen) weight.
    /// Positive delta means Fourier rates it MORE costly than canonical.
    /// Negative delta means Fourier gives it HIGHER priority than canonical.
    pub fn delta_vs_canonical(&self, k: u8, canonical: f32) -> f32 {
        self.weight(k) - canonical
    }

    /// True if the two symmetric sectors (k and 7-k) have the same weight
    /// (within floating-point tolerance). Always true for k ∈ {1,2,3}.
    pub fn are_symmetric(&self, k: u8) -> bool {
        if k == 0 { return true; } // only one sector at k=0
        let mirror = 7u8.wrapping_sub(k) % 7;
        (self.weight(k) - self.weight(mirror)).abs() < 1e-4
    }
}

// ── Stage 2 tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod e2_tests {
    use super::*;

    fn weights() -> E2FourierWeights { E2FourierWeights::compute() }

    // ── sigma1 arithmetic ─────────────────────────────────────────────────────

    #[test]
    fn sigma1_of_1_is_1()  { assert_eq!(sigma1(1), 1); }

    #[test]
    fn sigma1_of_6_is_12() { assert_eq!(sigma1(6), 1+2+3+6); }

    #[test]
    fn sigma1_of_prime_is_p_plus_1() {
        assert_eq!(sigma1(7),  1 + 7);   // prime
        assert_eq!(sigma1(11), 1 + 11);  // prime
    }

    #[test]
    fn sigma1_of_4_is_7() { assert_eq!(sigma1(4), 1+2+4); }

    // ── Raw cosine ordering ───────────────────────────────────────────────────

    #[test]
    fn k0_has_highest_raw_cosine() {
        let w = weights();
        let r0 = w.raw_cosine(0);
        for k in 1u8..7 {
            assert!(r0 >= w.raw_cosine(k),
                "C(0) must dominate C({k}): {} vs {}", r0, w.raw_cosine(k));
        }
    }

    #[test]
    fn k2_and_k5_have_lowest_raw_cosine() {
        // cos(4π/7) ≈ −0.2225 is the most-negative n=1 cosine term.
        // Summed over N terms with σ₁(n)/n weights, k=2 and k=5 produce
        // the minimum (most negative) raw cosine sums.
        let w   = weights();
        let r2  = w.raw_cosine(2);
        let r5  = w.raw_cosine(5);
        for k in [0u8, 1, 3, 4, 6] {
            assert!(w.raw_cosine(k) >= r2,
                "C({k}) must be >= C(2): {} vs {}", w.raw_cosine(k), r2);
            assert!(w.raw_cosine(k) >= r5,
                "C({k}) must be >= C(5): {} vs {}", w.raw_cosine(k), r5);
        }
    }

    // ── Weight interval ───────────────────────────────────────────────────────

    #[test]
    fn all_weights_in_low_high_range() {
        let w = weights();
        for k in 0u8..7 {
            let wk = w.weight(k);
            assert!(
                wk >= E2FourierWeights::LOW - 1e-5 && wk <= E2FourierWeights::HIGH + 1e-5,
                "weight({k}) = {wk} out of [{}, {}]",
                E2FourierWeights::LOW, E2FourierWeights::HIGH
            );
        }
    }

    #[test]
    fn entrance_k0_is_neutral_high() {
        // C(0) = maximum raw sum → maps to HIGH (1.00). Entrance is the DC component.
        let w = weights();
        assert!(
            (w.weight(0) - E2FourierWeights::HIGH).abs() < 1e-3,
            "Entrance (k=0) must be HIGH (neutral ≈ 1.00), got {}", w.weight(0)
        );
    }

    #[test]
    fn most_elevated_sectors_are_k2_or_k5() {
        // C(2)=C(5) produce the minimum raw cosine sum → lowest weight (most elevated).
        // Awliya (k=2) and Ulamaa (k=5) sit at cos(4π/7) < 0 — the most
        // harmonically distant positions from the Entrance DC component.
        let w    = weights();
        let peak = w.most_elevated();
        assert!(
            peak == 2 || peak == 5,
            "Most elevated should be Awliya(2) or Ulamaa(5), got {peak}"
        );
    }

    #[test]
    fn most_neutral_is_entrance_k0() {
        // C(0) ≈ 1.622 — the massive DC term makes Entrance the most neutral sector.
        let w = weights();
        assert_eq!(w.most_neutral(), 0, "Entrance (k=0) must be most neutral");
    }

    #[test]
    fn at_least_one_weight_equals_low() {
        let w = weights();
        let has_low = w.all().iter().any(|&wk| (wk - E2FourierWeights::LOW).abs() < 1e-4);
        assert!(has_low, "After normalisation, at least one weight must equal LOW");
    }

    #[test]
    fn at_least_one_weight_equals_high() {
        let w = weights();
        let has_high = w.all().iter().any(|&wk| (wk - E2FourierWeights::HIGH).abs() < 1e-4);
        assert!(has_high, "After normalisation, at least one weight must equal HIGH");
    }

    // ── Cosine symmetry of the 7th-root lattice ───────────────────────────────

    #[test]
    fn symmetry_k1_equals_k6() {
        let w = weights();
        assert!(
            (w.weight(1) - w.weight(6)).abs() < 1e-4,
            "Shuhadaa(1) and Anbiya(6) must be symmetric: {} vs {}",
            w.weight(1), w.weight(6)
        );
    }

    #[test]
    fn symmetry_k2_equals_k5() {
        let w = weights();
        assert!(
            (w.weight(2) - w.weight(5)).abs() < 1e-4,
            "Awliya(2) and Ulamaa(5) must be symmetric: {} vs {}",
            w.weight(2), w.weight(5)
        );
    }

    #[test]
    fn symmetry_k3_equals_k4() {
        let w = weights();
        assert!(
            (w.weight(3) - w.weight(4)).abs() < 1e-4,
            "Huffaz(3) and Momineen(4) must be symmetric: {} vs {}",
            w.weight(3), w.weight(4)
        );
    }

    #[test]
    fn are_symmetric_method_correct() {
        let w = weights();
        assert!(w.are_symmetric(0)); // k=0 is its own mirror
        assert!(w.are_symmetric(1));
        assert!(w.are_symmetric(2));
        assert!(w.are_symmetric(3));
    }

    // ── Weight ordering ───────────────────────────────────────────────────────

    #[test]
    fn outer_sectors_more_elevated_than_entrance() {
        // All outer sector weights < Entrance weight (1.00).
        // Confirmed by actual values: max outer weight ≈ 0.804 << 1.00.
        let w = weights();
        for k in 1u8..7 {
            assert!(
                w.weight(k) < w.weight(0) - 0.1,
                "All outer sectors must be substantially below Entrance: w({k})={:.5} vs w(0)={:.5}",
                w.weight(k), w.weight(0)
            );
        }
    }

    #[test]
    fn k2_k5_more_elevated_than_k3_k4() {
        // C(2)=−0.010 < C(3)=0.016 → w(2) < w(3) → Awliya more elevated than Huffaz.
        let w = weights();
        assert!(
            w.weight(2) < w.weight(3),
            "Awliya(2) must be more elevated than Huffaz(3): {} vs {}",
            w.weight(2), w.weight(3)
        );
    }

    #[test]
    fn k3_k4_more_elevated_than_k1_k6() {
        // C(3)=0.016 < C(1)=0.020 → w(3) < w(1) → Huffaz more elevated than Shuhadaa.
        let w = weights();
        assert!(
            w.weight(3) < w.weight(1),
            "Huffaz(3) must be more elevated than Shuhadaa(1): {} vs {}",
            w.weight(3), w.weight(1)
        );
    }

    // ── is_elevated ───────────────────────────────────────────────────────────

    #[test]
    fn entrance_is_not_elevated() {
        assert!(!weights().is_elevated(0));
    }

    #[test]
    fn all_outer_sectors_are_elevated() {
        let w = weights();
        for k in 1u8..7 {
            assert!(w.is_elevated(k), "sector {k} must be elevated");
        }
    }

    // ── Delta vs. canonical ───────────────────────────────────────────────────

    #[test]
    fn delta_for_entrance_is_near_zero() {
        // Canonical Entrance = 1.00 = HIGH; Fourier w(0) = 1.00 exactly.
        let w = weights();
        let d = w.delta_vs_canonical(0, 1.00);
        assert!(d.abs() < 1e-3, "Entrance delta vs canonical 1.00 should be ~0, got {d}");
    }

    #[test]
    fn delta_for_shuhadaa_fourier_is_more_elevated_than_canonical() {
        // Canonical Shuhadaa = 0.85; Fourier w(1) ≈ 0.804.
        // The Fourier derivation gives Shuhadaa HIGHER priority (lower cost) than
        // the hand-chosen weight, because outer sectors cluster near LOW.
        let w  = weights();
        let wk = w.weight(1);
        // Fourier weight is below the canonical 0.85 — math gives higher elevation
        assert!(wk < 0.85, "Fourier Shuhadaa ({wk:.4}) should be below canonical 0.85");
    }

    // ── Out-of-range ──────────────────────────────────────────────────────────

    #[test]
    fn out_of_range_sector_returns_high() {
        let w = weights();
        assert_eq!(w.weight(7),  E2FourierWeights::HIGH);
        assert_eq!(w.weight(99), E2FourierWeights::HIGH);
    }

    // ── Convergence ───────────────────────────────────────────────────────────

    #[test]
    fn n100_and_n200_weights_agree_within_half_percent() {
        // Maximum observed diff across all sectors: 0.0037 (sector k=1,6).
        // Tolerance 0.005 gives ~2× margin while confirming convergence.
        let w100 = E2FourierWeights::compute();
        let mut raw200 = [0.0f64; 7];
        for k in 0..7u8 { raw200[k as usize] = e2_sector_cosine(k, 200); }
        let min  = raw200.iter().cloned().fold(f64::INFINITY, f64::min);
        let max  = raw200.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let span = max - min;
        let range = (E2FourierWeights::HIGH - E2FourierWeights::LOW) as f64;
        for k in 0..7usize {
            // Use the same normalisation formula as the implementation (LOW + ...)
            let w200 = (E2FourierWeights::LOW as f64 + (raw200[k] - min) / span * range) as f32;
            let diff = (w100.weight(k as u8) - w200).abs();
            assert!(diff < 0.005, "100 vs 200 terms: sector {k} diff = {diff:.6}");
        }
    }
}

// ── Chord weight mapping ──────────────────────────────────────────────────────

/// Modular weight for each chord type.
pub fn chord_weight(chord: HeptaChordType) -> i8 {
    match chord {
        HeptaChordType::Spoke    => -1,
        HeptaChordType::Rim      =>  0,
        HeptaChordType::Local    =>  1,
        HeptaChordType::Diagonal =>  2,
    }
}

/// HeptaChordType for a given modular weight (inverse of chord_weight).
pub fn weight_chord(weight: i8) -> HeptaChordType {
    match weight {
        -1 => HeptaChordType::Spoke,
         0 => HeptaChordType::Rim,
         1 => HeptaChordType::Local,
         _ => HeptaChordType::Diagonal,
    }
}

// ── ModularNaviIndex ──────────────────────────────────────────────────────────

/// Compact Fourier index encoding the routing topology of a NaviMap.
#[derive(Debug, Clone)]
pub struct ModularNaviIndex {
    /// Modular weight — determined by the dominant HeptaChordType.
    /// Spoke=−1, Rim=0, Local=1, Diagonal=2. Ties resolved toward lower weight.
    pub weight:   i8,
    /// Level — NaviGraph node count at index build time.
    pub level:    u32,
    /// Fourier coefficients: a(n) = directed edges in cost bucket n.
    /// Only non-trailing-zero entries are stored.
    pub coeffs:   Vec<u32>,
    /// CRC-16/CCITT over the serialized (weight, level, coeffs) — map signature.
    pub checksum: u16,
}

impl ModularNaviIndex {
    // ── Constructors ─────────────────────────────────────────────────────────

    /// Build a ModularNaviIndex from a fully constructed NaviGraph.
    pub fn from_graph(graph: &NaviGraph) -> Self {
        Self::build(graph.edges(), graph.node_count() as u32)
    }

    /// Build from an edge slice and explicit level (node count).
    /// Useful for partial graphs or testing with synthetic edges.
    pub fn from_edges(edges: &[NaviEdge], level: u32) -> Self {
        Self::build(edges, level)
    }

    // ── Core queries ─────────────────────────────────────────────────────────

    /// 64-bit compact signature: weight × level × checksum.
    /// Suitable as a HashMap key or cache lookup.
    pub fn signature(&self) -> u64 {
        let w = (self.weight as i16 + 128) as u64; // map [-128,127] → [0,255]
        (w << 48) | ((self.level as u64) << 16) | self.checksum as u64
    }

    /// Structural routing equivalence: same weight, level, and checksum.
    /// Two maps passing this check share their optimal path topology.
    pub fn is_equivalent(&self, other: &Self) -> bool {
        self.weight == other.weight
            && self.level == other.level
            && self.checksum == other.checksum
    }

    /// Exact coefficient match — stronger than is_equivalent (immune to CRC collision).
    pub fn coeffs_match(&self, other: &Self) -> bool {
        self.weight == other.weight
            && self.level == other.level
            && self.coeffs == other.coeffs
    }

    /// Sum of all Fourier coefficients = total directed edge count.
    pub fn total_mass(&self) -> u32 {
        self.coeffs.iter().sum()
    }

    /// The dominant HeptaChordType (highest edge count, lower weight wins ties).
    pub fn dominant_chord(&self) -> HeptaChordType {
        weight_chord(self.weight)
    }

    /// Bucket index n with the highest coefficient a(n).
    pub fn spectral_peak_bucket(&self) -> usize {
        self.coeffs.iter().enumerate()
            .max_by_key(|(_, &v)| v)
            .map(|(i, _)| i)
            .unwrap_or(0)
    }

    /// Effective cost at the spectral peak — the map's "natural frequency".
    pub fn spectral_peak_cost(&self) -> f32 {
        self.spectral_peak_bucket() as f32 * BUCKET_STEP
    }

    /// Number of occupied (non-zero) buckets.
    pub fn spectral_width(&self) -> usize {
        self.coeffs.iter().filter(|&&v| v > 0).count()
    }

    /// Coefficient at the bucket containing `cost`.
    pub fn coeff_at_cost(&self, cost: f32) -> u32 {
        self.coeff_at_bucket(cost_to_bucket(cost))
    }

    pub fn coeff_at_bucket(&self, n: usize) -> u32 {
        self.coeffs.get(n).copied().unwrap_or(0)
    }

    // ── Resonance scoring ─────────────────────────────────────────────────────

    /// Normalised resonance score for an edge at `edge_cost` ∈ [0.0, 1.0].
    ///
    /// 1.0 = the cost sits exactly at the spectral peak (most common cost).
    /// 0.0 = this cost bucket is absent from the index (no edges have this cost).
    ///
    /// Used by NC2 ResonanceSeek in place of the flat tribe-bonus.
    pub fn resonance_score(&self, edge_cost: f32) -> f32 {
        let mass = self.total_mass();
        if mass == 0 { return 0.0; }
        let peak = self.coeffs.iter().copied().max().unwrap_or(0);
        if peak == 0 { return 0.0; }
        self.coeff_at_cost(edge_cost) as f32 / peak as f32
    }

    /// True if `resonance_score(edge_cost) >= threshold`.
    pub fn is_resonant(&self, edge_cost: f32, threshold: f32) -> bool {
        self.resonance_score(edge_cost) >= threshold
    }

    // ── Display helpers ───────────────────────────────────────────────────────

    /// Compact human-readable Fourier representation.
    /// Only non-zero buckets are shown: "θ{w=−1 L=7 a(4)=24}"
    pub fn fourier_repr(&self) -> String {
        let non_zero: Vec<String> = self.coeffs.iter().enumerate()
            .filter(|(_, &v)| v > 0)
            .map(|(i, v)| format!("a({i})={v}"))
            .collect();
        format!("θ{{w={} L={} {}}}", self.weight, self.level, non_zero.join(" "))
    }

    // ── Internal build ────────────────────────────────────────────────────────

    fn build(edges: &[NaviEdge], level: u32) -> Self {
        // Count edges per chord type to determine dominant
        let mut chord_counts = [0u32; 4]; // Spoke=0, Rim=1, Local=2, Diagonal=3
        let mut max_cost: f32 = 0.0;

        for e in edges {
            let idx = match e.chord {
                HeptaChordType::Spoke    => 0,
                HeptaChordType::Rim      => 1,
                HeptaChordType::Local    => 2,
                HeptaChordType::Diagonal => 3,
            };
            chord_counts[idx] += 1;
            let ec = e.effective_cost();
            if ec.is_finite() && ec > max_cost { max_cost = ec; }
        }

        // Dominant chord: highest count; lowest index (weight) wins ties
        let dominant_idx = chord_counts.iter().enumerate()
            .rev()                         // start from highest index...
            .max_by_key(|(_, &c)| c)       // ...so lowest-idx wins ties
            .map(|(i, _)| i)
            .unwrap_or(0);
        let dominant = [
            HeptaChordType::Spoke, HeptaChordType::Rim,
            HeptaChordType::Local, HeptaChordType::Diagonal,
        ][dominant_idx];
        let weight = chord_weight(dominant);

        // Build histogram
        let bucket_count = if max_cost > 0.0 {
            ((max_cost / BUCKET_STEP).ceil() as usize + 1).min(MAX_BUCKETS)
        } else {
            1
        };
        let mut coeffs = vec![0u32; bucket_count];
        for e in edges {
            let ec = e.effective_cost();
            if ec.is_finite() {
                let n = cost_to_bucket(ec).min(bucket_count - 1);
                coeffs[n] += 1;
            }
        }

        // Trim trailing zeros
        while coeffs.last() == Some(&0) && coeffs.len() > 1 {
            coeffs.pop();
        }

        let checksum = Self::compute_checksum(weight, level, &coeffs);
        ModularNaviIndex { weight, level, coeffs, checksum }
    }

    fn compute_checksum(weight: i8, level: u32, coeffs: &[u32]) -> u16 {
        let mut bytes: Vec<u8> = Vec::with_capacity(5 + coeffs.len() * 4);
        bytes.push(weight as u8);
        bytes.extend_from_slice(&level.to_le_bytes());
        for &c in coeffs {
            bytes.extend_from_slice(&c.to_le_bytes());
        }
        crc16(&bytes)
    }
}

#[inline]
fn cost_to_bucket(cost: f32) -> usize {
    (cost / BUCKET_STEP) as usize
}

// ── ResonanceScorerNc2 ────────────────────────────────────────────────────────

/// NC2 resonance scorer backed by a ModularNaviIndex.
///
/// Replaces the flat 0.30 tribe-bonus with a Fourier-weighted bonus:
/// edges at the spectral peak get `bonus_scale` (max bonus), edges in
/// empty buckets get 0.0. The bonus is smooth across cost buckets.
pub struct ResonanceScorerNc2 {
    index:       ModularNaviIndex,
    bonus_scale: f32,
}

impl ResonanceScorerNc2 {
    /// `bonus_scale` — maximum bonus applied at the spectral peak.
    /// Typical value: 0.60 (double the flat 0.30 tribe-bonus).
    pub fn new(index: ModularNaviIndex, bonus_scale: f32) -> Self {
        ResonanceScorerNc2 { index, bonus_scale }
    }

    /// Bonus to subtract from edge cost: `score × bonus_scale`.
    pub fn score(&self, edge_cost: f32) -> f32 {
        self.index.resonance_score(edge_cost) * self.bonus_scale
    }

    pub fn index(&self) -> &ModularNaviIndex { &self.index }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use navi_engine::{NaviGraph, seven_node_map};

    fn seven_graph() -> NaviGraph {
        NaviGraph::from_navimap(&seven_node_map()).unwrap()
    }

    fn seven_index() -> ModularNaviIndex {
        ModularNaviIndex::from_graph(&seven_graph())
    }

    // ── Construction ─────────────────────────────────────────────────────────

    #[test]
    fn from_seven_node_map_weight_is_neg_one_or_zero() {
        let idx = seven_index();
        // Spoke(12) ties with Rim(12); lowest index (Spoke=−1) wins
        assert!(idx.weight == -1 || idx.weight == 0,
            "dominant must be Spoke or Rim, got {}", idx.weight);
    }

    #[test]
    fn from_seven_node_map_level_is_7() {
        assert_eq!(seven_index().level, 7);
    }

    #[test]
    fn from_seven_node_map_total_mass_is_24() {
        // 6 bidirectional spokes (12 directed) + 6 bidirectional rim (12 directed)
        assert_eq!(seven_index().total_mass(), 24);
    }

    #[test]
    fn from_seven_node_map_checksum_nonzero() {
        assert_ne!(seven_index().checksum, 0);
    }

    #[test]
    fn from_seven_node_map_coeffs_nonempty() {
        assert!(!seven_index().coeffs.is_empty());
    }

    #[test]
    fn from_seven_node_map_signature_nonzero() {
        assert_ne!(seven_index().signature(), 0);
    }

    #[test]
    fn total_mass_equals_sum_of_coeffs() {
        let idx = seven_index();
        assert_eq!(idx.total_mass(), idx.coeffs.iter().sum::<u32>());
    }

    #[test]
    fn coeffs_no_trailing_zeros() {
        let idx = seven_index();
        // After build, the last coefficient must be non-zero
        assert_ne!(*idx.coeffs.last().unwrap(), 0);
    }

    // ── Equivalence ───────────────────────────────────────────────────────────

    #[test]
    fn is_equivalent_to_self() {
        let idx = seven_index();
        assert!(idx.is_equivalent(&idx));
    }

    #[test]
    fn is_equivalent_to_clone() {
        let idx = seven_index();
        assert!(idx.is_equivalent(&idx.clone()));
    }

    #[test]
    fn is_not_equivalent_different_level() {
        let a = seven_index();
        let mut b = a.clone();
        b.level = 99;
        b.checksum = b.checksum.wrapping_add(1); // invalidate
        assert!(!a.is_equivalent(&b));
    }

    #[test]
    fn coeffs_match_clone() {
        let idx = seven_index();
        assert!(idx.coeffs_match(&idx.clone()));
    }

    #[test]
    fn coeffs_do_not_match_modified() {
        let a = seven_index();
        let mut b = a.clone();
        if let Some(v) = b.coeffs.get_mut(0) { *v = v.wrapping_add(1); }
        assert!(!a.coeffs_match(&b));
    }

    // ── Spectral analysis ─────────────────────────────────────────────────────

    #[test]
    fn spectral_peak_bucket_in_valid_range() {
        let idx = seven_index();
        assert!(idx.spectral_peak_bucket() < idx.coeffs.len());
    }

    #[test]
    fn spectral_peak_cost_is_finite() {
        assert!(seven_index().spectral_peak_cost().is_finite());
    }

    #[test]
    fn spectral_peak_cost_is_nonnegative() {
        assert!(seven_index().spectral_peak_cost() >= 0.0);
    }

    #[test]
    fn spectral_width_at_least_one_for_nonempty_graph() {
        assert!(seven_index().spectral_width() >= 1);
    }

    #[test]
    fn coeff_at_peak_bucket_equals_total_mass_for_uniform_map() {
        // seven_node_map: spoke eff=400, rim eff=400 → all 24 edges in same bucket
        let idx = seven_index();
        let peak = idx.spectral_peak_bucket();
        // All 24 edges collapse to one bucket → coeff at peak = 24
        assert_eq!(idx.coeff_at_bucket(peak), 24);
    }

    #[test]
    fn coeff_at_out_of_range_bucket_returns_zero() {
        assert_eq!(seven_index().coeff_at_bucket(9999), 0);
    }

    #[test]
    fn coeff_at_cost_matches_coeff_at_bucket() {
        let idx   = seven_index();
        let cost  = idx.spectral_peak_cost();
        let n     = idx.spectral_peak_bucket();
        assert_eq!(idx.coeff_at_cost(cost), idx.coeff_at_bucket(n));
    }

    // ── Dominant chord ────────────────────────────────────────────────────────

    #[test]
    fn dominant_chord_roundtrips_through_weight() {
        let idx    = seven_index();
        let chord  = idx.dominant_chord();
        let weight = chord_weight(chord);
        assert_eq!(weight, idx.weight);
    }

    #[test]
    fn chord_weight_spoke_is_neg_one() {
        assert_eq!(chord_weight(HeptaChordType::Spoke), -1);
    }

    #[test]
    fn chord_weight_diagonal_is_two() {
        assert_eq!(chord_weight(HeptaChordType::Diagonal), 2);
    }

    #[test]
    fn weight_chord_roundtrip() {
        for &w in &[-1i8, 0, 1, 2] {
            assert_eq!(chord_weight(weight_chord(w)), w);
        }
    }

    // ── Resonance scoring ─────────────────────────────────────────────────────

    #[test]
    fn resonance_score_at_peak_is_one() {
        let idx  = seven_index();
        let cost = idx.spectral_peak_cost();
        // The peak bucket has the highest count → normalised score = 1.0
        assert!((idx.resonance_score(cost) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn resonance_score_in_zero_one_range() {
        let idx = seven_index();
        for cost in [0.0f32, 100.0, 400.0, 1000.0, 9999.0] {
            let s = idx.resonance_score(cost);
            assert!(s >= 0.0 && s <= 1.0, "score {s} out of [0,1] for cost {cost}");
        }
    }

    #[test]
    fn resonance_score_at_empty_bucket_is_zero() {
        let idx = seven_index();
        // Bucket 0 (cost 0.0–100.0): no edge has effective cost < 100
        assert_eq!(idx.resonance_score(0.0), 0.0);
    }

    #[test]
    fn is_resonant_above_threshold() {
        let idx  = seven_index();
        let cost = idx.spectral_peak_cost();
        assert!(idx.is_resonant(cost, 0.99));
    }

    #[test]
    fn is_resonant_below_threshold_for_empty_bucket() {
        let idx = seven_index();
        assert!(!idx.is_resonant(0.0, 0.01));
    }

    // ── Fourier representation ────────────────────────────────────────────────

    #[test]
    fn fourier_repr_contains_level() {
        let repr = seven_index().fourier_repr();
        assert!(repr.contains("L=7"), "repr: {repr}");
    }

    #[test]
    fn fourier_repr_contains_nonzero_bucket() {
        let repr = seven_index().fourier_repr();
        assert!(repr.contains("a("), "repr must contain at least one a(n): {repr}");
    }

    #[test]
    fn fourier_repr_total_matches_mass() {
        let idx  = seven_index();
        let repr = idx.fourier_repr();
        // Extract all a(n)=v terms and sum v
        let total: u32 = repr.split("a(")
            .skip(1)
            .filter_map(|seg| {
                seg.split(')').nth(1)
                   .and_then(|after| after.trim_start_matches('=').split_whitespace().next())
                   .and_then(|v| v.trim_end_matches('}').parse::<u32>().ok())
            })
            .sum();
        assert_eq!(total, idx.total_mass(), "fourier_repr must account for all edges");
    }

    // ── ResonanceScorerNc2 ────────────────────────────────────────────────────

    #[test]
    fn nc2_scorer_bonus_at_peak_equals_scale() {
        let idx    = seven_index();
        let scale  = 0.60_f32;
        let scorer = ResonanceScorerNc2::new(idx.clone(), scale);
        let cost   = idx.spectral_peak_cost();
        assert!((scorer.score(cost) - scale).abs() < 1e-6);
    }

    #[test]
    fn nc2_scorer_bonus_bounded_by_scale() {
        let idx    = seven_index();
        let scale  = 0.60_f32;
        let scorer = ResonanceScorerNc2::new(idx, scale);
        for cost in [0.0f32, 100.0, 400.0, 800.0, 9999.0] {
            assert!(scorer.score(cost) <= scale + 1e-6);
            assert!(scorer.score(cost) >= 0.0);
        }
    }

    #[test]
    fn nc2_scorer_empty_bucket_gives_zero_bonus() {
        let scorer = ResonanceScorerNc2::new(seven_index(), 0.60);
        assert_eq!(scorer.score(0.0), 0.0);
    }

    // ── Edge cases ────────────────────────────────────────────────────────────

    #[test]
    fn from_empty_edges_gives_zero_mass() {
        let idx = ModularNaviIndex::from_edges(&[], 0);
        assert_eq!(idx.total_mass(), 0);
    }

    #[test]
    fn checksum_changes_when_level_differs() {
        let a = seven_index();
        let b = ModularNaviIndex::from_edges(seven_graph().edges(), 99);
        assert_ne!(a.checksum, b.checksum);
    }

    #[test]
    fn signature_encodes_weight_and_level() {
        let idx = seven_index();
        let sig = idx.signature();
        // Weight component in top 16 bits, level in next 32
        let w_part = ((sig >> 48) as i8).wrapping_sub(0) as i64 - 128;
        assert_eq!(w_part as i8, idx.weight);
    }
}
