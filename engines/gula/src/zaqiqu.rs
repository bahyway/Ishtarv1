//! ZAQĪQU — phantom patterns (candidate name, pending CSR-08 seal).
//! The dream-phantom that visits before the real thing arrives.
//! Complement, never collision, with Eṭemmu (EN-DDB-004):
//!   eṭemmu is the ghost of what WAS; zaqīqu is the phantom of what IS NOT YET.
//!
//! Two detection mechanisms:
//!   1. Mendeleev gaps — unoccupied lattice cells surrounded by occupied
//!      neighbors (cells differing in exactly one face). Existence in the
//!      algebra precedes existence in the corpus.
//!   2. VOID-HINT — cycle rank (E − V + C) of the shared-face adjacency
//!      graph. A cycle of existing patterns enclosing a hole means the
//!      system organizes around an absence. Void is Sacred (ṬUPŠARRU).
//!      Full homology adjudication belongs to LamassuEngine, the sealed
//!      TDA sentinel — this module emits HINTS only, never verdicts of β₁.
//!
//! Governance: the Unnamed Particle doctrine, generalized. A phantom
//! receives a placeholder identity at prediction, held PRUNE-EXEMPT,
//! and is RECOVERED when a real engine witnesses it in the flesh.
//! Phantoms are predictions, not dialects: they are journaled, never
//! staged to uruk/, and never sealed by anyone but Bahaa.

use crate::{PatternWitness, Signature};
use std::collections::BTreeSet;

/// Default "surrounded" criterion: a gap needs at least this many occupied
/// one-face neighbors before it may be called a phantom (Mendeleev rule).
pub const MIN_SUPPORT: usize = 2;

#[derive(Debug, Clone)]
pub struct ZaqiquCandidate {
    /// The predicted signature — a position no particle occupies.
    pub signature: Signature,
    /// Occupied neighbor signatures that surround the gap.
    pub support: Vec<String>,
    /// Placeholder identity, prune-exempt until recovery.
    pub phantom_id: String,
}

/// Mechanism 1 — Mendeleev gaps over the occupied face lattice.
/// Occupied signatures are deduplicated first (Hepta uniqueness discipline).
pub fn detect_phantoms(occupied: &[Signature], min_support: usize) -> Vec<ZaqiquCandidate> {
    let mut uniq: Vec<Signature> = Vec::new();
    let mut seen = BTreeSet::new();
    for s in occupied {
        if seen.insert(s.key()) {
            uniq.push(s.clone());
        }
    }
    let ps: BTreeSet<&String> = uniq.iter().map(|s| &s.particle_face).collect();
    let os: BTreeSet<&String> = uniq.iter().map(|s| &s.orbit_face).collect();
    let ts: BTreeSet<&String> = uniq.iter().map(|s| &s.tribe_face).collect();
    let occ: BTreeSet<String> = uniq.iter().map(|s| s.key()).collect();

    let mut out = Vec::new();
    for p in &ps {
        for o in &os {
            for t in &ts {
                let cand = Signature {
                    particle_face: (*p).clone(),
                    orbit_face: (*o).clone(),
                    tribe_face: (*t).clone(),
                };
                let key = cand.key();
                if occ.contains(&key) {
                    continue;
                }
                let mut support: Vec<String> = uniq
                    .iter()
                    .filter(|s| shared_faces(s, &cand) == 2)
                    .map(|s| s.key())
                    .collect();
                support.sort();
                support.dedup();
                if support.len() >= min_support {
                    out.push(ZaqiquCandidate {
                        phantom_id: format!("ZAQIQU-{}", key),
                        signature: cand,
                        support,
                    });
                }
            }
        }
    }
    out
}

fn shared_faces(a: &Signature, b: &Signature) -> u8 {
    (a.particle_face == b.particle_face) as u8
        + (a.orbit_face == b.orbit_face) as u8
        + (a.tribe_face == b.tribe_face) as u8
}

/// Mechanism 2 — VOID-HINT: cycle rank E − V + C of the adjacency graph
/// (edges join signatures sharing exactly two faces). Non-zero rank means
/// at least one cycle of patterns encloses an absence. LamassuEngine owns
/// the final β₁ verdict; this is the hint that summons it.
pub fn void_hint(occupied: &[Signature]) -> usize {
    let mut uniq: Vec<&Signature> = Vec::new();
    let mut seen = BTreeSet::new();
    for s in occupied {
        if seen.insert(s.key()) {
            uniq.push(s);
        }
    }
    let n = uniq.len();
    if n == 0 {
        return 0;
    }
    let mut parent: Vec<usize> = (0..n).collect();
    fn find(parent: &mut [usize], mut i: usize) -> usize {
        while parent[i] != i {
            parent[i] = parent[parent[i]];
            i = parent[i];
        }
        i
    }
    let mut edges = 0usize;
    for i in 0..n {
        for j in (i + 1)..n {
            if shared_faces(uniq[i], uniq[j]) == 2 {
                edges += 1;
                let (ri, rj) = (find(&mut parent, i), find(&mut parent, j));
                if ri != rj {
                    parent[ri] = rj;
                }
            }
        }
    }
    let comps: BTreeSet<usize> = (0..n).map(|i| find(&mut parent, i)).collect();
    (edges + comps.len()).saturating_sub(n)
}

/// Recovery — the Unnamed Particle doctrine fulfilled: a live testimony
/// matching a phantom's signature recovers it. Returns the recovered
/// phantom (removed from the prune-exempt hold) or None.
pub fn recover(
    phantoms: &mut Vec<ZaqiquCandidate>,
    witness: &PatternWitness,
) -> Option<ZaqiquCandidate> {
    let key = witness.signature.key();
    let idx = phantoms.iter().position(|z| z.signature.key() == key)?;
    Some(phantoms.remove(idx))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sig(p: &str, o: &str, t: &str) -> Signature {
        Signature::new(p, o, t)
    }

    #[test]
    fn mendeleev_gap_is_predicted() {
        // Occupied: (a,x,1),(b,x,1),(a,y,1) — the gap (b,y,1) has two
        // one-face neighbors → phantom. Cells with less support are not.
        let occ = vec![sig("a", "x", "1"), sig("b", "x", "1"), sig("a", "y", "1")];
        let ph = detect_phantoms(&occ, MIN_SUPPORT);
        assert!(ph.iter().any(|z| z.signature.key() == sig("b", "y", "1").key()));
        for z in &ph {
            assert!(z.support.len() >= MIN_SUPPORT);
            assert!(z.phantom_id.starts_with("ZAQIQU-"));
        }
    }

    #[test]
    fn occupied_cells_are_never_phantoms() {
        let occ = vec![sig("a", "x", "1"), sig("b", "x", "1")];
        let ph = detect_phantoms(&occ, 1);
        assert!(ph.iter().all(|z| z.signature.key() != sig("a", "x", "1").key()));
    }

    #[test]
    fn void_hint_sees_the_enclosed_absence() {
        // Four signatures forming a 4-cycle in the shared-face graph:
        // (a,x,1)-(b,x,1)-(b,y,1)-(a,y,1)-(a,x,1) → cycle rank 1.
        let cycle = vec![
            sig("a", "x", "1"),
            sig("b", "x", "1"),
            sig("b", "y", "1"),
            sig("a", "y", "1"),
        ];
        assert_eq!(void_hint(&cycle), 1, "one cycle encloses one absence");
        // A path (no cycle) hints nothing:
        let path = vec![sig("a", "x", "1"), sig("b", "x", "1"), sig("b", "y", "1")];
        assert_eq!(void_hint(&path), 0);
        assert_eq!(void_hint(&[]), 0);
    }

    #[test]
    fn phantom_recovery_fulfills_the_prophecy() {
        let occ = vec![sig("a", "x", "1"), sig("b", "x", "1"), sig("a", "y", "1")];
        let mut ph = detect_phantoms(&occ, MIN_SUPPORT);
        let before = ph.len();
        assert!(before >= 1);
        let w = PatternWitness {
            engine: "nanshe".into(),
            signature: sig("b", "y", "1"),
            hubullu_gloss: "the phantom made flesh".into(),
        };
        let rec = recover(&mut ph, &w);
        assert!(rec.is_some(), "matching testimony recovers the phantom");
        assert_eq!(ph.len(), before - 1);
        assert!(recover(&mut ph, &w).is_none(), "a phantom recovers only once");
    }
}
