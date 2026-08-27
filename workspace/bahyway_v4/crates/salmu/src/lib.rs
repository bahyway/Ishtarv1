//! salmu — GL-SHP-001 (The Salmu Registry): standard simplicial shapes —
//! names, glyphs, witnesses, and verdicts. PB-335. Pure Rust, zero
//! dependencies.
#![forbid(unsafe_code)]

/// The three witness classes (§3) plus the history-aware ancestry flag
/// (§5 clause) a real caller would read off the Temennu-of-Shape.
#[derive(Debug, Clone, Copy)]
pub struct Witnesses {
    /// W-A: census (particle count in the tribe).
    pub census: usize,
    /// W-A: Betti at standard radius eps.
    pub b0: u32,
    pub b1: u32,
    /// W-B: a hole absent at eps but present under the Baru Sweep radius
    /// filtration -- the crescent's near-loop signature.
    pub ghost_hole_in_sweep: bool,
    /// W-C: core occupancy, mass fraction within 0.45*R_gyration.
    pub kappa: f64,
    /// W-C: elongation e = lambda1/lambda2 of the covariance ellipse.
    pub elongation: f64,
    /// History clause (§5): was the Temennu-of-Shape once a hoop?
    pub had_temennu_ancestry: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SalmuGlyph {
    Riqu,
    Eperu,
    Sibirtu,
    Harranu,
    Kippatu,
    Mastabba,
    Uskaru,
    Kirsu,
}

pub const E_ROAD: f64 = 3.0;
pub const KAPPA_LOW: f64 = 0.10;

/// Classification totality (L20): the decision order is sealed --
/// RIQU -> SIBIRTU/EPERU -> MASTABBA -> KIPPATU -> HARRANU -> USKARU ->
/// KIRSU -- chambers are tested before elongation, because anything with
/// a chamber cannot be a road. Deterministic, total, one glyph per
/// tribe.
pub fn classify(w: &Witnesses) -> SalmuGlyph {
    if w.census == 0 {
        return SalmuGlyph::Riqu;
    }
    if w.b0 > 1 {
        return if w.had_temennu_ancestry { SalmuGlyph::Sibirtu } else { SalmuGlyph::Eperu };
    }
    // b0 == 1 from here: single component. Chambers before elongation.
    if w.b1 >= 2 {
        return SalmuGlyph::Mastabba;
    }
    if w.b1 == 1 && !w.ghost_hole_in_sweep && w.kappa < KAPPA_LOW {
        return SalmuGlyph::Kippatu;
    }
    if w.elongation > E_ROAD {
        return SalmuGlyph::Harranu;
    }
    if w.b1 == 0 && w.ghost_hole_in_sweep && w.kappa < KAPPA_LOW {
        return SalmuGlyph::Uskaru;
    }
    SalmuGlyph::Kirsu
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Witnesses {
        Witnesses {
            census: 100,
            b0: 1,
            b1: 1,
            ghost_hole_in_sweep: false,
            kappa: 0.02,
            elongation: 1.2,
            had_temennu_ancestry: false,
        }
    }

    // L17 — hoop -> KIPPATU; paved hoop (interior filled) -> KIRSU.
    #[test]
    fn l17_hoop_and_paved_hoop() {
        let hoop = base();
        assert_eq!(classify(&hoop), SalmuGlyph::Kippatu);

        let paved = Witnesses { b1: 0, ghost_hole_in_sweep: false, kappa: 0.8, ..base() };
        assert_eq!(classify(&paved), SalmuGlyph::Kirsu, "a paved-over hoop is a collapsed orbit, not a healthy one");
    }

    // L18 — crescent -> USKARU, never KIRSU.
    #[test]
    fn l18_crescent_is_uskaru_not_kirsu() {
        let crescent = Witnesses { b1: 0, ghost_hole_in_sweep: true, kappa: 0.02, ..base() };
        assert_eq!(classify(&crescent), SalmuGlyph::Uskaru);
        assert_ne!(classify(&crescent), SalmuGlyph::Kirsu);
    }

    // L19 — history flips EPERU -> SIBIRTU.
    #[test]
    fn l19_history_flips_eperu_to_sibirtu() {
        let dust = Witnesses { b0: 3, had_temennu_ancestry: false, ..base() };
        assert_eq!(classify(&dust), SalmuGlyph::Eperu);

        let broken = Witnesses { b0: 3, had_temennu_ancestry: true, ..base() };
        assert_eq!(classify(&broken), SalmuGlyph::Sibirtu, "same geometry, different history -> different verdict");
    }

    // L20 — classifier total and deterministic: every glyph is reachable,
    // and the same witnesses always classify the same way.
    #[test]
    fn l20_total_and_deterministic() {
        let empty = Witnesses { census: 0, ..base() };
        assert_eq!(classify(&empty), SalmuGlyph::Riqu);

        let twins = Witnesses { b1: 2, ..base() };
        assert_eq!(classify(&twins), SalmuGlyph::Mastabba);

        let road = Witnesses { b1: 0, elongation: 5.0, kappa: 0.02, ..base() };
        assert_eq!(classify(&road), SalmuGlyph::Harranu);

        // Determinism: repeated classification of the same witnesses.
        let w = base();
        let first = classify(&w);
        for _ in 0..10 {
            assert_eq!(classify(&w), first);
        }
    }
}
