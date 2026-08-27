//! bwvl — GL-VSL-001 (The BWVL Verb Law): WITNESS / REHEARSE / PROPOSE —
//! and the missing verb. PB-337. Pure Rust, zero dependencies.
//!
//! §2, the law of this tablet: **BWVL has no APPLY.** This crate defines
//! no function anywhere that takes a mutable reference to real tribe
//! state; a `Proposal` can only be consumed by `send_to_bench`, which
//! returns an inert `BenchSubmission` — the only lawful next step is
//! GL-DST-004's own approval path, outside this crate entirely (L24,
//! enforced at the type level by this crate's own API surface, not by a
//! runtime check).
#![forbid(unsafe_code)]

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Watermark {
    Simulation,
}

/// A REHEARSE frame: every rehearsed frame carries the SIMULATION
/// watermark and epsilon; rehearsed futures never enter NARU as events,
/// only as forecasts (§1).
#[derive(Debug, Clone, Copy)]
pub struct RehearsedFrame {
    pub tick: usize,
    pub theta_hat: f64,
    pub salmu: SalmuGlyph,
    pub watermark: Watermark,
    pub epsilon: f64,
}

/// OU decay stepper with a juggler multiplier: more nabalkutu particles,
/// faster decay (§1).
pub fn ou_decay_step(theta: f64, juggler_fraction: f64, decay_rate: f64) -> f64 {
    theta * (1.0 - decay_rate * juggler_fraction).max(0.0)
}

fn reclassify_salmu(theta: f64, theta0: f64) -> SalmuGlyph {
    let ratio = theta / theta0;
    if ratio > 0.8 {
        SalmuGlyph::Kippatu
    } else if ratio > 0.4 {
        SalmuGlyph::Uskaru
    } else {
        SalmuGlyph::Kirsu
    }
}

/// Rehearse a full run: theta decays tick by tick under the juggler
/// fraction, the Salmu is resampled each tick, every frame is watermarked
/// SIMULATION (L25).
pub fn rehearse(theta0: f64, juggler_fraction: f64, decay_rate: f64, ticks: usize, epsilon: f64) -> Vec<RehearsedFrame> {
    let mut theta = theta0;
    let mut frames = Vec::with_capacity(ticks);
    for t in 0..ticks {
        theta = ou_decay_step(theta, juggler_fraction, decay_rate);
        frames.push(RehearsedFrame {
            tick: t,
            theta_hat: theta,
            salmu: reclassify_salmu(theta, theta0),
            watermark: Watermark::Simulation,
            epsilon,
        });
    }
    frames
}

/// Shape Horizon T_shape: the first tick the classification flips away
/// from the healthy KIPPATU orbit.
pub fn shape_horizon(frames: &[RehearsedFrame]) -> Option<usize> {
    frames.iter().find(|f| f.salmu != SalmuGlyph::Kippatu).map(|f| f.tick)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProposalKind {
    Mend,
    ExciseUnwitnessed,
    Reweave,
    ReMoor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct W5H2Explanation {
    pub text: String,
}

/// A ghost: a structurally immutable preview. Every field is read-only
/// after construction (no `pub` field, only getters) and the ONLY
/// consuming method is `send_to_bench` — there is no `apply` method on
/// this type, anywhere, ever (L24).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proposal {
    kind: ProposalKind,
    forecast_glyph: SalmuGlyph,
    explanation: W5H2Explanation,
}

impl Proposal {
    pub fn kind(&self) -> ProposalKind {
        self.kind
    }
    pub fn forecast_glyph(&self) -> SalmuGlyph {
        self.forecast_glyph
    }
    pub fn explanation(&self) -> &str {
        &self.explanation.text
    }

    /// Every BWVL proposal terminates in SEND-TO-BENCH; application
    /// happens only through GL-DST-004, outside this crate.
    pub fn send_to_bench(self) -> BenchSubmission {
        BenchSubmission { proposal: self }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchSubmission {
    pub proposal: Proposal,
}

/// MEND (USKARU → KIPPATU): bridge the missing arc.
pub fn propose_mend(current: SalmuGlyph) -> Option<Proposal> {
    if current != SalmuGlyph::Uskaru {
        return None;
    }
    Some(Proposal {
        kind: ProposalKind::Mend,
        forecast_glyph: SalmuGlyph::Kippatu,
        explanation: W5H2Explanation {
            text: "MEND bridges the missing arc of a crescent (USKARU) toward KIPPATU".into(),
        },
    })
}

/// EXCISE-UNWITNESSED (KIRSU): remove interior particles lacking
/// observation witnesses, per the Apsu Vacancy Principle.
pub fn propose_excise_unwitnessed(current: SalmuGlyph) -> Option<Proposal> {
    if current != SalmuGlyph::Kirsu {
        return None;
    }
    Some(Proposal {
        kind: ProposalKind::ExciseUnwitnessed,
        forecast_glyph: SalmuGlyph::Uskaru,
        explanation: W5H2Explanation {
            text: "EXCISE-UNWITNESSED removes unwitnessed interior particles (Apsu Vacancy Principle)".into(),
        },
    })
}

/// REWEAVE (SIBIRTU): bridge the fragments of a former whole.
pub fn propose_reweave(current: SalmuGlyph) -> Option<Proposal> {
    if current != SalmuGlyph::Sibirtu {
        return None;
    }
    Some(Proposal {
        kind: ProposalKind::Reweave,
        forecast_glyph: SalmuGlyph::Kippatu,
        explanation: W5H2Explanation { text: "REWEAVE bridges the fragments of a former whole".into() },
    })
}

/// RE-MOOR — the only proposal that is ITSELF a decree act, restoring
/// theta0 via a decree-path Temennu relay.
pub fn propose_re_moor(current_theta: f64, theta0: f64) -> Option<Proposal> {
    if current_theta >= theta0 {
        return None;
    }
    Some(Proposal {
        kind: ProposalKind::ReMoor,
        forecast_glyph: SalmuGlyph::Kippatu,
        explanation: W5H2Explanation { text: "RE-MOOR restores theta0 via a decree-path Temennu relay".into() },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // L24 — a proposal cannot mutate the tribe. Behaviorally: consuming a
    // proposal through its only exit (send_to_bench) never changes its
    // own recorded forecast, and no other function in this crate accepts
    // a live tribe by mutable reference to begin with.
    #[test]
    fn l24_proposal_is_immutable_preview() {
        let p = propose_mend(SalmuGlyph::Uskaru).unwrap();
        let forecast_before = p.forecast_glyph();
        let submission = p.send_to_bench();
        assert_eq!(submission.proposal.forecast_glyph(), forecast_before);
        assert_eq!(submission.proposal.kind(), ProposalKind::Mend);
    }

    // L25 — every rehearsed frame carries the SIMULATION watermark flag.
    #[test]
    fn l25_every_frame_watermarked() {
        let frames = rehearse(1.0, 0.5, 0.05, 50, 0.1);
        assert!(!frames.is_empty());
        assert!(frames.iter().all(|f| f.watermark == Watermark::Simulation));
        assert!(frames.iter().all(|f| f.epsilon == 0.1));
    }

    // L26 — a MEND preview of a crescent forecasts KIPPATU.
    #[test]
    fn l26_mend_forecasts_kippatu() {
        let p = propose_mend(SalmuGlyph::Uskaru).expect("MEND applies to USKARU");
        assert_eq!(p.forecast_glyph(), SalmuGlyph::Kippatu);
        assert!(propose_mend(SalmuGlyph::Kirsu).is_none(), "MEND does not apply to KIRSU");
    }

    // L27 — T_shape is monotone non-increasing in juggler fraction.
    #[test]
    fn l27_shape_horizon_monotone_in_juggler_fraction() {
        let low = rehearse(1.0, 0.1, 0.05, 200, 0.05);
        let high = rehearse(1.0, 0.9, 0.05, 200, 0.05);
        let t_low = shape_horizon(&low).unwrap_or(usize::MAX);
        let t_high = shape_horizon(&high).unwrap_or(usize::MAX);
        assert!(t_high <= t_low, "more jugglers must not delay the shape horizon (low={t_low}, high={t_high})");
    }

    #[test]
    fn re_moor_is_decree_gated() {
        assert!(propose_re_moor(0.9, 1.0).is_some());
        assert!(propose_re_moor(1.0, 1.0).is_none(), "no decree needed when already at theta0");
    }
}
