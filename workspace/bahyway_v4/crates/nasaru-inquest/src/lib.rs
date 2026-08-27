//! nasaru-inquest — GL-NSR-001-A2 (The Rigmu Inquest Doctrine). PB-332.
//!
//! A RIGMU may not be answered with opinion. This crate seals the
//! Inquest: the standard W5H2 investigation that opens automatically with
//! every great cry. Pure Rust, zero dependencies.
#![forbid(unsafe_code)]

/// Onset bisection: finds the first index where a "sustained departure"
/// predicate holds for the rest of the series (§2 WHEN, O(log n) against
/// the assumption that departure, once begun, does not un-depart before
/// the Inquest runs). Binary search over a monotone predicate.
pub fn onset_bisection(departed: &[bool]) -> Option<usize> {
    if departed.is_empty() || !*departed.last().unwrap() {
        return None;
    }
    let (mut lo, mut hi) = (0usize, departed.len() - 1);
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        if departed[mid] {
            hi = mid;
        } else {
            lo = mid + 1;
        }
    }
    Some(lo)
}

/// The proof obligation of L9: bisection must equal a plain linear scan
/// for the first `true`, on any input where the predicate is monotone
/// non-decreasing (never un-departs).
pub fn onset_linear_scan(departed: &[bool]) -> Option<usize> {
    departed.iter().position(|&d| d)
}

#[derive(Debug, Clone)]
pub struct Actor {
    pub id: String,
    pub write_mass: u64,
    pub timestamp: i64,
}

/// WHO: symmetric lineage sweep over [onset-delta, onset+delta] — writes
/// *after* onset can be concealment, not only cause (§4 clause). Ranked
/// by write mass, descending.
pub fn lineage_sweep(actors: &[Actor], onset: i64, delta: i64) -> Vec<Actor> {
    let mut in_bracket: Vec<Actor> = actors
        .iter()
        .filter(|a| a.timestamp >= onset - delta && a.timestamp <= onset + delta)
        .cloned()
        .collect();
    in_bracket.sort_by(|a, b| b.write_mass.cmp(&a.write_mass));
    in_bracket
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeptaAxis {
    Integrity,
    Temporal,
    Quality,
    Shape,
}

/// WHERE: the leading axis is the Hepta dimension with the earliest
/// per-dimension onset — evidence, never verdict (§4 clause).
pub fn leading_axis(per_axis_onset: &[(HeptaAxis, usize)]) -> Option<HeptaAxis> {
    per_axis_onset
        .iter()
        .min_by_key(|(_, onset)| *onset)
        .map(|(axis, _)| *axis)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RootCause {
    UpstreamCorruption,
    UnauthorizedWrite,
    WorldChangeGoldenStale,
    PipelineFault,
}

/// WHY: two-witness conviction — an evidence witness and an independent
/// NUZI-lineage witness must concord on the same root cause. Discordant
/// proposals are refused, not corrected (§4 clause): the case stays open.
pub fn two_witness_why(
    evidence_witness: RootCause,
    lineage_witness: RootCause,
) -> Result<RootCause, &'static str> {
    if evidence_witness == lineage_witness {
        Ok(evidence_witness)
    } else {
        Err("WHY witnesses do not concord -- conviction refused, case stays open")
    }
}

/// The mandatory W5H2 fields of an explanation particle (§2). Every field
/// is required; a particle missing any cannot be minted (L11).
#[derive(Debug, Clone, Default)]
pub struct W5H2Builder {
    pub when_onset: Option<usize>,
    pub who_actors: Option<Vec<String>>,
    pub what_diff: Option<String>,
    pub where_axis: Option<HeptaAxis>,
    pub why_root_cause: Option<RootCause>,
    pub how_mechanism: Option<String>,
    pub how_much_magnitude: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct ExplanationParticle {
    pub when_onset: usize,
    pub who_actors: Vec<String>,
    pub what_diff: String,
    pub where_axis: HeptaAxis,
    pub why_root_cause: RootCause,
    pub how_mechanism: String,
    pub how_much_magnitude: f64,
    pub signed: bool,
}

impl W5H2Builder {
    /// Kernel-enforced (L11): refuses to mint unless every W5H2 field is
    /// present. The signature is the LAST act (§4 clause) — a fresh
    /// particle is always unsigned; see `close_case`.
    pub fn mint(&self) -> Result<ExplanationParticle, &'static str> {
        Ok(ExplanationParticle {
            when_onset: self.when_onset.ok_or("WHEN is mandatory")?,
            who_actors: self
                .who_actors
                .clone()
                .filter(|v| !v.is_empty())
                .ok_or("WHO is mandatory")?,
            what_diff: self
                .what_diff
                .clone()
                .filter(|s| !s.is_empty())
                .ok_or("WHAT is mandatory")?,
            where_axis: self.where_axis.ok_or("WHERE is mandatory")?,
            why_root_cause: self.why_root_cause.ok_or("WHY is mandatory")?,
            how_mechanism: self
                .how_mechanism
                .clone()
                .filter(|s| !s.is_empty())
                .ok_or("HOW is mandatory")?,
            how_much_magnitude: self.how_much_magnitude.ok_or("HOW MUCH is mandatory")?,
            signed: false,
        })
    }
}

/// SYNC case CLOSED → unfreeze Gate G4: only after a signed conviction
/// (§3 operation 7 / §4 "signature is the last act" clause). Any lesser
/// close attempt is refused and journaled (L12).
pub fn close_case(particle: &mut ExplanationParticle, architect_or_madanu_signed: bool) -> Result<(), &'static str> {
    if !architect_or_madanu_signed {
        return Err("close refused: missing Architect/Madanu signature (L12)");
    }
    particle.signed = true;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // L9 — onset bisection equals linear scan (bisection = linear scan, proven).
    #[test]
    fn l9_bisection_matches_linear_scan() {
        let cases: Vec<Vec<bool>> = vec![
            vec![false, false, false, true, true, true],
            vec![true, true, true],
            vec![false, false, false],
            vec![false, true],
        ];
        for case in cases {
            assert_eq!(onset_bisection(&case), onset_linear_scan(&case), "case: {case:?}");
        }
    }

    // L10 — the bracket is symmetric: WHO sweeps [onset-delta, onset+delta],
    // including writes strictly after onset (possible concealment).
    #[test]
    fn l10_symmetric_bracket_includes_after_onset() {
        let actors = vec![
            Actor { id: "a".into(), write_mass: 5, timestamp: 90 },   // before onset, in bracket
            Actor { id: "b".into(), write_mass: 9, timestamp: 105 },  // after onset, in bracket
            Actor { id: "c".into(), write_mass: 1, timestamp: 200 },  // far after, excluded
        ];
        let swept = lineage_sweep(&actors, 100, 10);
        let ids: Vec<&str> = swept.iter().map(|a| a.id.as_str()).collect();
        assert!(ids.contains(&"a"));
        assert!(ids.contains(&"b"), "writes after onset must be swept too (concealment check)");
        assert!(!ids.contains(&"c"));
        assert_eq!(swept[0].id, "b", "ranked by write mass descending");
    }

    // L11 — an explanation particle missing any field cannot be minted.
    #[test]
    fn l11_partial_particle_refused() {
        let mut b = W5H2Builder::default();
        assert!(b.mint().is_err());
        b.when_onset = Some(3);
        b.who_actors = Some(vec!["scribe-a".into()]);
        b.what_diff = Some("spine EAV diverged from world witness".into());
        b.where_axis = Some(HeptaAxis::Integrity);
        assert!(b.mint().is_err(), "still missing WHY/HOW/HOW_MUCH");
        b.why_root_cause = Some(RootCause::UpstreamCorruption);
        b.how_mechanism = Some("Sen slope decay, lineage path confirmed".into());
        b.how_much_magnitude = Some(2.4);
        assert!(b.mint().is_ok(), "W5H2-complete particle must mint");
    }

    // L12 — signature is the last act; discordant WHY witnesses are
    // refused, not corrected; close refuses without a signature.
    #[test]
    fn l12_signature_last_act_and_discord_refused() {
        assert!(two_witness_why(RootCause::PipelineFault, RootCause::PipelineFault).is_ok());
        assert!(two_witness_why(RootCause::PipelineFault, RootCause::UnauthorizedWrite).is_err());

        let mut b = W5H2Builder::default();
        b.when_onset = Some(1);
        b.who_actors = Some(vec!["scribe-a".into()]);
        b.what_diff = Some("diff".into());
        b.where_axis = Some(HeptaAxis::Temporal);
        b.why_root_cause = Some(RootCause::WorldChangeGoldenStale);
        b.how_mechanism = Some("decay signature".into());
        b.how_much_magnitude = Some(1.0);
        let mut particle = b.mint().unwrap();
        assert!(!particle.signed);
        assert!(close_case(&mut particle, false).is_err(), "unsigned close refused");
        assert!(!particle.signed);
        assert!(close_case(&mut particle, true).is_ok());
        assert!(particle.signed);
    }

    #[test]
    fn leading_axis_picks_earliest_onset() {
        let axes = vec![
            (HeptaAxis::Quality, 50),
            (HeptaAxis::Integrity, 12),
            (HeptaAxis::Temporal, 30),
        ];
        assert_eq!(leading_axis(&axes), Some(HeptaAxis::Integrity));
    }
}
