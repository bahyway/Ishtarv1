//! HEPTA CLEARANCE FIELD — the Higgs-style border law (PB-607).
//! GL-MAP-001's gapless heptagonal tiling carries a clearance field φ:
//! unassessed ground sits in the unbroken phase (φ = 0); a cell whose
//! residual bound is met breaks symmetry locally (φ → v, certified).
//! The border of the clean area is the domain wall where φ transitions —
//! KAKI v4.0 mints identity per border cell (border-witness clause).
//!
//! THE BORDER LAW: a cell may never certify while touching unassessed
//! ground. No certified islands floating in ignorance — the clean region
//! grows only by continuous front. This kills the classic handover
//! failure: certified patches with unassessed seams between them.
//!
//! WITNESS TYPING (Kīnu honesty in the type system):
//!   Surface    — LiDAR micro-topography. Sees the SKIN of the soil:
//!                disturbance texture, subsidence dimples, tripwires.
//!                It can summon; it can NEVER certify alone.
//!   Subsurface — GPR, magnetometry. Sees depth. Required for any
//!                certification.
//!   NoiseForensic — huburu structure verdict on residuals (noise.rs).

use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum WitnessKind {
    Surface,
    Subsurface,
    NoiseForensic,
}

#[derive(Debug, Clone, Default)]
pub struct HeptaCell {
    /// Indices of neighboring cells in the gapless tiling (7 interior;
    /// rim cells of a finite patch may hold fewer).
    pub neighbors: Vec<usize>,
    /// Residual bound (puluḫtu) once assessed; None = unbroken phase.
    pub residual: Option<f64>,
    pub witness_kinds: BTreeSet<WitnessKind>,
    pub certified: bool,
}

#[derive(Debug, Default)]
pub struct HeptaField {
    pub cells: Vec<HeptaCell>,
}

impl HeptaField {
    pub fn new(neighbor_lists: Vec<Vec<usize>>) -> Self {
        Self {
            cells: neighbor_lists
                .into_iter()
                .map(|neighbors| HeptaCell { neighbors, ..Default::default() })
                .collect(),
        }
    }

    /// Record an assessment for a cell: witnesses seen, residual bound.
    pub fn assess(&mut self, i: usize, residual: f64, kinds: &[WitnessKind]) {
        let c = &mut self.cells[i];
        c.residual = Some(residual);
        c.witness_kinds.extend(kinds.iter().cloned());
    }

    /// Attempt local symmetry breaking (certification). Refusals name
    /// their law; there is no silent failure.
    pub fn try_certify(&mut self, i: usize, eps_theta: f64) -> Result<(), String> {
        let (residual, kinds, neighbors) = {
            let c = &self.cells[i];
            (c.residual, c.witness_kinds.clone(), c.neighbors.clone())
        };
        let r = residual.ok_or_else(|| {
            format!("cell {} unassessed — the unbroken phase cannot certify", i)
        })?;
        if !kinds.contains(&WitnessKind::Subsurface) {
            return Err(format!(
                "cell {} has no subsurface witness — LiDAR sees the skin of the \
                 soil, not its depth; it may summon, never certify",
                i
            ));
        }
        if r > eps_theta {
            return Err(format!(
                "cell {} residual {:.2e} exceeds ε(Θ) {:.2e} — the bound is not met",
                i, r, eps_theta
            ));
        }
        for &n in &neighbors {
            if self.cells[n].residual.is_none() {
                return Err(format!(
                    "BORDER LAW · cell {} touches unassessed cell {} — no certified \
                     island may float in ignorance; the front must be continuous",
                    i, n
                ));
            }
        }
        self.cells[i].certified = true;
        Ok(())
    }

    /// The domain wall: certified cells with at least one uncertified
    /// neighbor. These receive KAKI border identities.
    pub fn border(&self) -> Vec<usize> {
        (0..self.cells.len())
            .filter(|&i| {
                self.cells[i].certified
                    && self.cells[i].neighbors.iter().any(|&n| !self.cells[n].certified)
            })
            .collect()
    }
}

/// The honest form of "+1000%": the gain lives in the exponent.
/// Adding independent witness passes multiplies the miss product.
pub fn miss_multiplier(added_witness_p: &[f64]) -> f64 {
    added_witness_p.iter().fold(1.0, |m, p| m * (1.0 - p))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Central cell 0 with 7 neighbors 1..=7 (rim cells neighbor 0 only —
    /// a minimal finite patch of the tiling).
    fn patch() -> HeptaField {
        let mut lists = vec![(1..=7).collect::<Vec<_>>()];
        for _ in 1..=7 {
            lists.push(vec![0]);
        }
        HeptaField::new(lists)
    }

    #[test]
    fn lidar_alone_may_summon_never_certify() {
        let mut f = patch();
        for i in 1..=7 {
            f.assess(i, 1e-4, &[WitnessKind::Subsurface]);
        }
        f.assess(0, 1e-6, &[WitnessKind::Surface, WitnessKind::NoiseForensic]);
        let v = f.try_certify(0, 1e-3);
        assert!(v.is_err());
        assert!(v.unwrap_err().contains("skin of the"));
    }

    #[test]
    fn border_law_refuses_islands_in_ignorance() {
        let mut f = patch();
        f.assess(0, 1e-6, &[WitnessKind::Subsurface]);
        // neighbor 3 left unassessed
        for i in [1, 2, 4, 5, 6, 7] {
            f.assess(i, 1e-4, &[WitnessKind::Subsurface]);
        }
        let v = f.try_certify(0, 1e-3);
        assert!(v.is_err());
        assert!(v.unwrap_err().contains("BORDER LAW"));
        // assess the missing neighbor → the front is continuous → certify
        f.assess(3, 2e-4, &[WitnessKind::Subsurface]);
        assert!(f.try_certify(0, 1e-3).is_ok());
    }

    #[test]
    fn domain_wall_is_computed() {
        let mut f = patch();
        for i in 0..=7 {
            f.assess(i, 1e-6, &[WitnessKind::Subsurface]);
        }
        f.try_certify(0, 1e-3).unwrap();
        assert_eq!(f.border(), vec![0], "certified center bordered by uncertified rim");
        for i in 1..=7 {
            f.try_certify(i, 1e-3).unwrap();
        }
        assert!(f.border().is_empty(), "fully certified patch has no interior wall");
    }

    #[test]
    fn the_gain_lives_in_the_exponent() {
        let m = miss_multiplier(&[0.6, 0.7]);
        assert!((m - 0.12).abs() < 1e-12);
        assert!(m < 0.2, "two added witnesses: nearly an order of magnitude per pass");
    }
}
