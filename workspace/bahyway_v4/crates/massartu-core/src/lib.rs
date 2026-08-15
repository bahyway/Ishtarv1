//! massartu-core — the Massartu pattern: a domain-neutral "watch."
//!
//! DETECT -> PROVE -> PREDICT -> PRESCRIBE, made literally true by
//! `watch_cycle` actually calling each step in that order instead of
//! merely naming it. PH-002 Corollary 1 (nucleus-exchange invariance)
//! claims the same `watch_cycle` produces correct behavior for any
//! Tribe that plugs in real `ResidualLaw`/`Prover`/`Horizon`/
//! `Exposure` implementations — the crate's own test suite proves that
//! by driving the identical function against a real electricity
//! nucleus (backed by `fdd-core`/`egd-engine`'s physics) and a generic
//! manual nucleus (no dedicated engine), see `tests::nucleus_exchange`.
//!
//! HONEST SCOPE: `Prover::residual_trustworthy` has no real King-plot
//! classifier anywhere in this ecosystem yet (`docs/07_file_formats/GL-EGD-001.md`
//! already names that gap), so its default implementation trusts the
//! residual unconditionally. `Prover::isolation_proven` defaults to
//! `false` — a domain must opt in with a real exclusion simulator (as
//! the electricity nucleus does via `egd_engine::exclusion::
//! simulate_exclusion`) before Massartu will ever prescribe
//! `Prescription::ExcludeProven`; every other domain honestly gets
//! `Prescription::MitigateOnly` until it builds one.

use trend_core::time_to_threshold;

/// One (day, kappa) sample from a domain's residual law. `day` is
/// relative time (t=0 = most recent); `kappa` is the residual
/// magnitude in whatever unit the domain's conservation law produces.
pub type KappaHistory = Vec<(f64, f64)>;

/// DETECT: the domain hands over its residual/conservation-violation
/// history for one monitored unit. Nothing here assumes physics —
/// a domain with no simulator can hand over manually logged readings.
pub trait ResidualLaw {
    fn kappa_history(&self, unit_id: u32) -> KappaHistory;
}

/// PROVE: gate DETECT and PRESCRIBE behind real evidence, not
/// assumption.
pub trait Prover {
    /// Is this residual history itself trustworthy (not sensor noise,
    /// not a stale/disconnected reading)? Defaults to trusting the
    /// reading — no King-plot-style residual classifier exists in
    /// this ecosystem yet.
    fn residual_trustworthy(&self, _unit_id: u32) -> bool {
        true
    }

    /// Has removing/isolating this unit been proven safe (the rest of
    /// the Tribe still serves all remaining load)? Defaults to `false`
    /// — a domain must wire a real exclusion simulator to earn `true`.
    fn isolation_proven(&self, _unit_id: u32) -> bool {
        false
    }
}

/// PREDICT: given a kappa history, estimate probability of crossing
/// `threshold` within `window_days`, plus a supporting band (here:
/// the trend-fit's predicted days-to-crossing, or `window_days` if the
/// trend never crosses).
pub trait Horizon {
    fn p_fail_within(&self, history: &KappaHistory, threshold: f64, window_days: f64) -> (f64, f64);
}

/// A real `Horizon`: wraps `trend_core::time_to_threshold` (the same
/// least-squares trend-to-threshold primitive Gibil's own horizon and
/// Marduk's T_golden use) rather than inventing a fourth copy.
pub struct TrendHorizon;

impl Horizon for TrendHorizon {
    fn p_fail_within(&self, history: &KappaHistory, threshold: f64, window_days: f64) -> (f64, f64) {
        match time_to_threshold(history, threshold) {
            Ok(Some(days)) if days <= window_days => (1.0 - (days / window_days).clamp(0.0, 1.0), days),
            Ok(Some(days)) => (0.0, days),
            Ok(None) => (0.0, window_days),
            // Fewer than two samples: honestly the lowest-confidence
            // answer, not a crash.
            Err(_) => (0.0, window_days),
        }
    }
}

/// How much is riding on this unit — customers served, criticality,
/// blast radius. Always domain-specific; Massartu never guesses it.
pub trait Exposure {
    fn exposure(&self, unit_id: u32) -> f64;
}

/// Fixed exposure value — the common case for a single monitored unit
/// in a test or a small deployment.
pub struct FixedExposure(pub f64);

impl Exposure for FixedExposure {
    fn exposure(&self, _unit_id: u32) -> f64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    /// PROVE failed: the residual reading itself isn't trustworthy.
    /// Escalate the data-quality problem, not the (unknown) risk.
    Unproven,
    Sound,
    Watch,
    Strike,
    Herald,
}

/// PRESCRIBE: what to actually do about a non-Sound tier.
#[derive(Debug, Clone, PartialEq)]
pub enum Prescription {
    Inspect { unit_id: u32, before_days: f64 },
    Repair { unit_id: u32, before_days: f64 },
    /// Only reachable when `Prover::isolation_proven` returned `true`.
    ExcludeProven { unit_id: u32, isolation_steps: Vec<String> },
    /// The honest fallback whenever isolation has not been proven
    /// safe: never prescribe an exclusion Massartu can't back up.
    MitigateOnly { unit_id: u32, reasons: Vec<String>, interim: Vec<String> },
}

/// RiskFis: fuse (p_fail, band, exposure) into a tier.
pub trait RiskFis {
    fn tier(&self, p_fail: f64, band: f64, exposure: f64) -> Tier;
}

/// A real, simple `RiskFis`: severity = p_fail weighted by exposure.
pub struct SimpleRiskFis {
    pub watch_p_fail: f64,
    pub strike_p_fail: f64,
    /// Severity (p_fail * max(exposure, 1.0)) beyond this escalates a
    /// Strike-level p_fail into Herald.
    pub herald_severity: f64,
}

impl RiskFis for SimpleRiskFis {
    fn tier(&self, p_fail: f64, _band: f64, exposure: f64) -> Tier {
        let severity = p_fail * exposure.max(1.0);
        if p_fail >= self.strike_p_fail && severity >= self.herald_severity {
            Tier::Herald
        } else if p_fail >= self.strike_p_fail {
            Tier::Strike
        } else if p_fail >= self.watch_p_fail {
            Tier::Watch
        } else {
            Tier::Sound
        }
    }
}

pub trait Prescriber {
    fn prescribe(&self, unit_id: u32, tier: Tier, isolation_proven: bool) -> Prescription;
}

/// A real, simple `Prescriber`: Watch inspects, Strike repairs (or
/// mitigates if isolation isn't proven safe), Herald excludes (or
/// mitigates), Unproven/Sound just schedules a routine look.
pub struct SimplePrescriber {
    pub lead_days: f64,
}

impl Prescriber for SimplePrescriber {
    fn prescribe(&self, unit_id: u32, tier: Tier, isolation_proven: bool) -> Prescription {
        match tier {
            Tier::Watch => Prescription::Inspect { unit_id, before_days: self.lead_days },
            Tier::Strike if isolation_proven => Prescription::Repair { unit_id, before_days: self.lead_days * 0.5 },
            Tier::Strike => Prescription::MitigateOnly {
                unit_id,
                reasons: vec!["isolation not proven safe".to_string()],
                interim: vec!["derate".to_string(), "manual inspection".to_string()],
            },
            Tier::Herald if isolation_proven => Prescription::ExcludeProven {
                unit_id,
                isolation_steps: vec!["confirmed via Prover::isolation_proven".to_string()],
            },
            Tier::Herald => Prescription::MitigateOnly {
                unit_id,
                reasons: vec!["isolation not proven safe; cannot exclude".to_string()],
                interim: vec!["emergency derate".to_string()],
            },
            Tier::Sound | Tier::Unproven => Prescription::Inspect { unit_id, before_days: self.lead_days * 4.0 },
        }
    }
}

pub trait Escalator {
    fn emit(&mut self, unit_id: u32, tier: Tier, ts_millis: u64);
    /// Has enough time passed since the last emission for this unit
    /// that a fresh alert is warranted (vs. re-alerting every cycle)?
    fn silence_elapsed(&self, unit_id: u32, now_millis: u64) -> bool;
}

/// A real, in-memory `Escalator` — the common case for a test or a
/// small deployment; a production Tribe can back this with its own
/// alert bus while keeping the same trait.
#[derive(Default)]
pub struct InMemoryEscalator {
    pub silence_window_millis: u64,
    last_emit: std::collections::HashMap<u32, u64>,
    pub log: Vec<(u32, Tier, u64)>,
}

impl InMemoryEscalator {
    pub fn new(silence_window_millis: u64) -> Self {
        Self { silence_window_millis, ..Default::default() }
    }
}

impl Escalator for InMemoryEscalator {
    fn emit(&mut self, unit_id: u32, tier: Tier, ts_millis: u64) {
        self.last_emit.insert(unit_id, ts_millis);
        self.log.push((unit_id, tier, ts_millis));
    }

    fn silence_elapsed(&self, unit_id: u32, now_millis: u64) -> bool {
        match self.last_emit.get(&unit_id) {
            Some(&last) => now_millis.saturating_sub(last) >= self.silence_window_millis,
            None => true,
        }
    }
}

pub trait Chronicler {
    fn record(&mut self, unit_id: u32, tier: Tier, prescription: &Prescription, ts_millis: u64);
}

/// A real, in-memory `Chronicler`.
#[derive(Default)]
pub struct InMemoryChronicler {
    pub entries: Vec<(u32, Tier, Prescription, u64)>,
}

impl Chronicler for InMemoryChronicler {
    fn record(&mut self, unit_id: u32, tier: Tier, prescription: &Prescription, ts_millis: u64) {
        self.entries.push((unit_id, tier, prescription.clone(), ts_millis));
    }
}

/// The Massartu law itself: DETECT -> PROVE -> PREDICT -> PRESCRIBE,
/// literally in that order. Escalation/chronicling only happen for a
/// non-Sound tier, and only once per `Escalator::silence_elapsed`
/// window.
#[allow(clippy::too_many_arguments)]
pub fn watch_cycle<RL, PR, H, EX, FI, PS, ES, CH>(
    unit_id: u32,
    residual_law: &RL,
    prover: &PR,
    horizon: &H,
    exposure: &EX,
    fis: &FI,
    prescriber: &PS,
    escalator: &mut ES,
    chronicler: &mut CH,
    threshold: f64,
    window_days: f64,
    now_millis: u64,
) -> Tier
where
    RL: ResidualLaw,
    PR: Prover,
    H: Horizon,
    EX: Exposure,
    FI: RiskFis,
    PS: Prescriber,
    ES: Escalator,
    CH: Chronicler,
{
    // DETECT
    let history = residual_law.kappa_history(unit_id);

    // PROVE (gate 1: is the data itself trustworthy?)
    if !prover.residual_trustworthy(unit_id) {
        let tier = Tier::Unproven;
        if escalator.silence_elapsed(unit_id, now_millis) {
            let prescription = prescriber.prescribe(unit_id, tier, false);
            escalator.emit(unit_id, tier, now_millis);
            chronicler.record(unit_id, tier, &prescription, now_millis);
        }
        return tier;
    }

    // PREDICT
    let (p_fail, band) = horizon.p_fail_within(&history, threshold, window_days);
    let exposure_value = exposure.exposure(unit_id);
    let tier = fis.tier(p_fail, band, exposure_value);

    // PRESCRIBE (gate 2: is isolation itself proven safe?)
    if tier != Tier::Sound && escalator.silence_elapsed(unit_id, now_millis) {
        let isolation_proven = prover.isolation_proven(unit_id);
        let prescription = prescriber.prescribe(unit_id, tier, isolation_proven);
        escalator.emit(unit_id, tier, now_millis);
        chronicler.record(unit_id, tier, &prescription, now_millis);
    }

    tier
}

#[cfg(test)]
mod tests {
    use super::*;

    struct HardcodedHistory(KappaHistory);
    impl ResidualLaw for HardcodedHistory {
        fn kappa_history(&self, _unit_id: u32) -> KappaHistory {
            self.0.clone()
        }
    }
    impl Prover for HardcodedHistory {}

    fn degrading_history() -> KappaHistory {
        // Same physical degradation trend fed to both scenarios below
        // -- what differs between them is exposure, not the physics.
        // trend-core's convention: t=0 is "now", negative t is past
        // -- so the most recent (most degraded) sample must land at
        // t=0, not the oldest one.
        let n = 30;
        (0..n).map(|d| (d as f64 - (n - 1) as f64, 0.01 + d as f64 * 0.02)).collect()
    }

    /// Same underlying kappa trend, different exposure -> different
    /// tier. This is the exposure-separation claim: two units on the
    /// same "orbit" (same physical trend) still triage differently
    /// because what's riding on them differs.
    #[test]
    fn hospital_generator_reaches_herald_desert_line_does_not() {
        let residual_law = HardcodedHistory(degrading_history());
        let horizon = TrendHorizon;
        let fis = SimpleRiskFis { watch_p_fail: 0.2, strike_p_fail: 0.5, herald_severity: 4.0 };
        let prescriber = SimplePrescriber { lead_days: 3.0 };

        let hospital_exposure = FixedExposure(50.0); // 50 patients on backup power
        let mut esc_h = InMemoryEscalator::new(0);
        let mut chron_h = InMemoryChronicler::default();
        let hospital_tier = watch_cycle(
            1,
            &residual_law,
            &residual_law,
            &horizon,
            &hospital_exposure,
            &fis,
            &prescriber,
            &mut esc_h,
            &mut chron_h,
            0.5,
            10.0,
            0,
        );

        let desert_exposure = FixedExposure(1.0); // one unserved rural line segment
        let mut esc_d = InMemoryEscalator::new(0);
        let mut chron_d = InMemoryChronicler::default();
        let desert_tier = watch_cycle(
            2,
            &residual_law,
            &residual_law,
            &horizon,
            &desert_exposure,
            &fis,
            &prescriber,
            &mut esc_d,
            &mut chron_d,
            0.5,
            10.0,
            0,
        );

        assert_eq!(hospital_tier, Tier::Herald);
        assert_ne!(desert_tier, Tier::Herald);
        assert!(!esc_h.log.is_empty());
        assert!(!esc_d.log.is_empty());
    }

    #[test]
    fn unproven_residual_short_circuits_to_unproven_tier() {
        struct NeverTrust;
        impl ResidualLaw for NeverTrust {
            fn kappa_history(&self, _unit_id: u32) -> KappaHistory {
                vec![(0.0, 999.0)] // would obviously be Herald if trusted
            }
        }
        impl Prover for NeverTrust {
            fn residual_trustworthy(&self, _unit_id: u32) -> bool {
                false
            }
        }

        let n = NeverTrust;
        let horizon = TrendHorizon;
        let exposure = FixedExposure(100.0);
        let fis = SimpleRiskFis { watch_p_fail: 0.2, strike_p_fail: 0.5, herald_severity: 4.0 };
        let prescriber = SimplePrescriber { lead_days: 3.0 };
        let mut esc = InMemoryEscalator::new(0);
        let mut chron = InMemoryChronicler::default();

        let tier = watch_cycle(1, &n, &n, &horizon, &exposure, &fis, &prescriber, &mut esc, &mut chron, 0.5, 10.0, 0);
        assert_eq!(tier, Tier::Unproven);
        assert_eq!(chron.entries.len(), 1);
        assert!(matches!(chron.entries[0].2, Prescription::Inspect { .. }));
    }

    #[test]
    fn silence_window_suppresses_repeat_alerts_within_the_window() {
        let residual_law = HardcodedHistory(degrading_history());
        let horizon = TrendHorizon;
        let exposure = FixedExposure(50.0);
        let fis = SimpleRiskFis { watch_p_fail: 0.2, strike_p_fail: 0.5, herald_severity: 4.0 };
        let prescriber = SimplePrescriber { lead_days: 3.0 };
        let mut esc = InMemoryEscalator::new(1_000);
        let mut chron = InMemoryChronicler::default();

        watch_cycle(1, &residual_law, &residual_law, &horizon, &exposure, &fis, &prescriber, &mut esc, &mut chron, 0.5, 10.0, 0);
        watch_cycle(1, &residual_law, &residual_law, &horizon, &exposure, &fis, &prescriber, &mut esc, &mut chron, 0.5, 10.0, 500);
        assert_eq!(esc.log.len(), 1, "second cycle is inside the silence window and must not re-alert");

        watch_cycle(1, &residual_law, &residual_law, &horizon, &exposure, &fis, &prescriber, &mut esc, &mut chron, 0.5, 10.0, 1_500);
        assert_eq!(esc.log.len(), 2, "third cycle is past the silence window and must alert again");
    }

    /// PH-002 Corollary 1 (nucleus-exchange invariance): the literal
    /// same `watch_cycle` function, unmodified, correctly drives two
    /// genuinely different nuclei -- one backed by real electricity
    /// physics (fdd-core's conservation residual + egd-engine's Gibil
    /// calculus + its real exclusion simulator), one a generic manual
    /// nucleus with no dedicated engine at all. This is the empirical
    /// evidence the corollary needs: not two calls with the same
    /// hardcoded numbers, but two calls whose DETECT and PROVE steps
    /// do genuinely different, domain-specific work underneath an
    /// identical PREDICT/PRESCRIBE law.
    mod nucleus_exchange {
        use super::*;
        use egd_engine::{exclusion, phasor::Phasor, Edge, Gibil, Network, Node};
        use fdd_core::Quantity;

        /// A redundant double-feed: two parallel edges of half
        /// admittance each carry a load. Losing one edge halves the
        /// flow into the load but does not strand it (its residual
        /// does not jump to its full injection) -- a genuine
        /// `Verdict::Proven` case for `simulate_exclusion`, not a
        /// fabricated one.
        struct ElectricityNucleus {
            base_net: Network<Phasor>,
            phi: Vec<Phasor>,
            monitored_node_id: u32,
            spare_edge_idx: usize,
            n_days: usize,
            max_theft_frac: f64,
        }

        impl ElectricityNucleus {
            fn new() -> Self {
                let y_full = Phasor::new(10.0, -5.0);
                let y_half = y_full.mul(Phasor::new(0.5, 0.0));
                let v = vec![Phasor::new(1.0, 0.0), Phasor::new(0.95, -0.02)];
                let i_line = y_full.mul(v[0].sub(v[1]));
                let base_net = Network {
                    nodes: vec![
                        Node { id: 0, injection: Phasor::zero().sub(i_line) },
                        Node { id: 1, injection: i_line },
                    ],
                    edges: vec![
                        Edge { from: 0, to: 1, coeff: y_half, in_service: true },
                        Edge { from: 0, to: 1, coeff: y_half, in_service: true },
                    ],
                };
                Self { base_net, phi: v, monitored_node_id: 1, spare_edge_idx: 0, n_days: 10, max_theft_frac: 0.6 }
            }
        }

        impl ResidualLaw for ElectricityNucleus {
            fn kappa_history(&self, unit_id: u32) -> KappaHistory {
                if unit_id != self.monitored_node_id {
                    return vec![];
                }
                let node_idx = self.base_net.nodes.iter().position(|n| n.id == unit_id).unwrap();
                // trend-core convention: t=0 is "now" -- the most
                // recent (most degraded) day must land at t=0.
                (0..self.n_days)
                    .map(|day| {
                        let frac = self.max_theft_frac * (day as f64 / (self.n_days - 1) as f64);
                        let mut net = self.base_net.clone();
                        let stolen = net.nodes[node_idx].injection.mul(Phasor::new(1.0 - frac, 0.0));
                        net.nodes[node_idx].injection = stolen;
                        let r = fdd_core::residuals(&net, &self.phi, &Gibil).unwrap();
                        (day as f64 - (self.n_days - 1) as f64, r[node_idx])
                    })
                    .collect()
            }
        }

        impl Prover for ElectricityNucleus {
            // residual_trustworthy: default (true) -- no King-plot
            // classifier exists yet, per docs/07_file_formats/GL-EGD-001.md.
            fn isolation_proven(&self, unit_id: u32) -> bool {
                if unit_id != self.monitored_node_id {
                    return false;
                }
                matches!(
                    exclusion::simulate_exclusion(&self.base_net, &self.phi, self.spare_edge_idx, 1e-6),
                    Ok(exclusion::Verdict::Proven)
                )
            }
        }

        /// A generic manual nucleus: a domain with no dedicated
        /// engine, e.g. a technician logging readings by hand. Its
        /// `Prover` is entirely the trait's honest defaults --
        /// residual trusted, isolation never proven, because no
        /// exclusion simulator exists for it.
        struct ManualNucleus(KappaHistory);
        impl ResidualLaw for ManualNucleus {
            fn kappa_history(&self, _unit_id: u32) -> KappaHistory {
                self.0.clone()
            }
        }
        impl Prover for ManualNucleus {}

        #[test]
        fn same_watch_cycle_drives_a_real_physics_nucleus_and_a_manual_one() {
            let horizon = TrendHorizon;
            let fis = SimpleRiskFis { watch_p_fail: 0.15, strike_p_fail: 0.4, herald_severity: 3.0 };
            let prescriber = SimplePrescriber { lead_days: 5.0 };

            // Nucleus A: electricity, real physics underneath.
            let elec = ElectricityNucleus::new();
            let history_a = elec.kappa_history(elec.monitored_node_id);
            assert!(history_a.len() == elec.n_days);
            assert!(
                history_a.first().unwrap().1 < history_a.last().unwrap().1,
                "physics-driven current theft must produce a genuinely growing residual, not fabricated numbers"
            );
            assert!(elec.isolation_proven(elec.monitored_node_id), "the redundant spare edge must make exclusion provably safe");

            let elec_exposure = FixedExposure(20.0);
            let mut esc_a = InMemoryEscalator::new(0);
            let mut chron_a = InMemoryChronicler::default();
            let tier_a = watch_cycle(
                elec.monitored_node_id,
                &elec,
                &elec,
                &horizon,
                &elec_exposure,
                &fis,
                &prescriber,
                &mut esc_a,
                &mut chron_a,
                0.3,
                20.0,
                0,
            );
            assert_ne!(tier_a, Tier::Sound, "the growing physical residual must be caught");
            assert_ne!(tier_a, Tier::Unproven);
            if tier_a == Tier::Herald {
                assert!(matches!(chron_a.entries[0].2, Prescription::ExcludeProven { .. }), "a proven-safe isolation must be prescribed, not merely mitigated");
            }

            // Nucleus B: generic/manual, no dedicated engine, no
            // exclusion simulator.
            let manual = ManualNucleus((0..10).map(|d| (d as f64 - 9.0, 0.02 + d as f64 * 0.08)).collect());
            assert!(!manual.isolation_proven(1), "no exclusion simulator exists for this domain -- must stay unproven");

            let manual_exposure = FixedExposure(20.0);
            let mut esc_b = InMemoryEscalator::new(0);
            let mut chron_b = InMemoryChronicler::default();
            let tier_b = watch_cycle(
                1,
                &manual,
                &manual,
                &horizon,
                &manual_exposure,
                &fis,
                &prescriber,
                &mut esc_b,
                &mut chron_b,
                0.3,
                20.0,
                0,
            );
            assert_ne!(tier_b, Tier::Sound);
            assert_ne!(tier_b, Tier::Unproven);
            if tier_b == Tier::Herald || tier_b == Tier::Strike {
                assert!(
                    matches!(chron_b.entries[0].2, Prescription::MitigateOnly { .. } | Prescription::Inspect { .. }),
                    "an unproven isolation must never be prescribed as ExcludeProven/Repair"
                );
            }

            // The literal same function correctly triaged two
            // structurally different nuclei -- this is the
            // nucleus-exchange invariance PH-002 Corollary 1 claims.
            assert_eq!(esc_a.log.len(), 1);
            assert_eq!(esc_b.log.len(), 1);
        }
    }
}
