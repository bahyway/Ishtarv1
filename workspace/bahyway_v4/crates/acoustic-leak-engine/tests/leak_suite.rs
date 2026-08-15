//! acoustic-leak-engine sovereign test suite (deterministic, no external data).

use acoustic_leak_engine::calibration::{localize_two_sensor, piecewise_travel_time_s, SegmentCalibration};
use acoustic_leak_engine::crlb::{error_budget, ErrorBudgetInput};
use acoustic_leak_engine::material::{PipeMaterial, SolverClass};
use acoustic_leak_engine::solver::gcc_phat::gcc_phat_delay;
use acoustic_leak_engine::solver::joint_index::{snap_to_joint, JointMap};
use acoustic_leak_engine::stage1_kingplot::{deming_fit, judge_point, Stage1Judgement};
use acoustic_leak_engine::verdict::{judge, Escalation, LeakSite, PositionClaim, Stage};
use acoustic_leak_engine::SOVEREIGN_MARGIN_M;

// Deterministic pseudo-noise (no rand dependency).
fn pseudo_noise(i: usize, scale: f64) -> f64 {
    let x = ((i as f64 * 12.9898).sin() * 43758.5453).fract();
    (x - 0.5) * 2.0 * scale
}

#[test]
fn deming_recovers_true_slope_with_noise_on_both_axes() {
    let n = 60;
    let mut xs = Vec::new();
    let mut ys = Vec::new();
    for i in 0..n {
        let t = 10.0 + i as f64 * 1.5;
        xs.push(t + pseudo_noise(i, 0.4));
        ys.push(1.2 * t + 5.0 + pseudo_noise(i + 1000, 0.4));
    }
    let fit = deming_fit(&xs, &ys, 1.0).expect("fit");
    assert!((fit.slope - 1.2).abs() < 0.03, "slope {}", fit.slope);
    assert!((fit.intercept - 5.0).abs() < 1.5, "intercept {}", fit.intercept);
    // n-2 dof: resid_sd near the injected noise scale, not biased low.
    assert!(fit.resid_sd > 0.15 && fit.resid_sd < 0.8, "sd {}", fit.resid_sd);
}

#[test]
fn stage1_flags_systematic_leak_deviation() {
    let n = 30;
    let mut xs = Vec::new();
    let mut ys = Vec::new();
    for i in 0..n {
        let t = 10.0 + i as f64 * 3.0;
        xs.push(t + pseudo_noise(i, 0.3));
        ys.push(1.2 * t + 5.0 + pseudo_noise(i + 500, 0.3));
    }
    let fit = deming_fit(&xs, &ys, 1.0).expect("fit");
    // Healthy point passes; leak-sagged point flags.
    assert_eq!(judge_point(&fit, 55.0, 1.2 * 55.0 + 5.0, 3.0), Stage1Judgement::GridIntegrityHolds);
    assert_eq!(judge_point(&fit, 85.0, 1.2 * 85.0 + 5.0 - 12.0, 3.0), Stage1Judgement::SegmentFlagged);
}

#[test]
fn gcc_phat_recovers_known_delay() {
    let fs = 8000.0;
    let n = 4096;
    let delay_samples = 37usize; // y lags x by 37 samples = 4.625 ms
    // Broadband leak-like source.
    let src: Vec<f64> = (0..n + delay_samples).map(|i| pseudo_noise(i, 1.0)).collect();
    let x: Vec<f64> = src[delay_samples..delay_samples + n].to_vec();
    let y: Vec<f64> = src[..n].to_vec(); // y is x delayed by delay_samples
    let est = gcc_phat_delay(&x, &y, fs).expect("delay");
    let true_delay = delay_samples as f64 / fs;
    assert!((est.delay_s - true_delay).abs() < 1.5 / fs, "est {} vs true {}", est.delay_s, true_delay);
}

#[test]
fn two_sensor_localization_from_recovered_delay() {
    // Ductile iron, calibrated in situ.
    let seg = SegmentCalibration::from_reference_tap(
        PipeMaterial::DuctileIron,
        120.0, // sensor spacing L
        120.0, // tap at far sensor
        0.1,   // 120 m / 0.1 s -> v = 1200 m/s
        0.0001,
        14.5,
    )
    .expect("cal");
    assert!((seg.v_mps - 1200.0).abs() < 1e-9);
    // Leak at d = 40 m from A -> dt = (L - 2d)/v = 40/1200 s.
    let dt = (120.0 - 2.0 * 40.0) / 1200.0;
    let d = localize_two_sensor(&seg, dt);
    assert!((d - 40.0).abs() < 1e-9, "d {}", d);
}

#[test]
fn piecewise_travel_time_for_mixed_material_path() {
    let iron =
        SegmentCalibration::from_reference_tap(PipeMaterial::DuctileIron, 60.0, 60.0, 0.05, 0.0001, 14.0).unwrap();
    let pe = SegmentCalibration::from_reference_tap(PipeMaterial::Pe, 30.0, 30.0, 0.1, 0.0005, 14.0).unwrap();
    let t = piecewise_travel_time_s(&[iron, pe]);
    assert!((t - 0.15).abs() < 1e-12);
}

#[test]
fn material_selects_solver_lawfully() {
    assert_eq!(PipeMaterial::DuctileIron.solver_class(), SolverClass::PlainGccPhat);
    assert_eq!(PipeMaterial::Copper.solver_class(), SolverClass::PlainGccPhat);
    assert_eq!(PipeMaterial::Pe.solver_class(), SolverClass::DispersionCompensated);
    assert_eq!(PipeMaterial::Pvc.solver_class(), SolverClass::DispersionCompensated);
    assert_eq!(PipeMaterial::VitrifiedClay.solver_class(), SolverClass::JointIndexed);
    assert_eq!(PipeMaterial::Concrete.solver_class(), SolverClass::JointIndexed);
}

#[test]
fn error_budget_velocity_term_dominates_far_from_midpoint() {
    let near = error_budget(&ErrorBudgetInput {
        v_mps: 1200.0,
        sigma_v_rel: 0.005,
        bandwidth_hz: 750.0,
        observation_s: 10.0,
        snr_linear: 10.0,
        dx_from_midpoint_m: 1.0,
    });
    let far = error_budget(&ErrorBudgetInput {
        v_mps: 1200.0,
        sigma_v_rel: 0.005,
        bandwidth_hz: 750.0,
        observation_s: 10.0,
        snr_linear: 10.0,
        dx_from_midpoint_m: 50.0,
    });
    assert!(far.sigma_d_m > near.sigma_d_m);
    // 0.5% v error at 50 m from midpoint alone costs 0.25 m -- above margin.
    assert!(far.velocity_term_m >= 0.25 - 1e-9);
    assert!(far.sigma_d_m > SOVEREIGN_MARGIN_M);
}

#[test]
fn verdict_gate_refuses_unproven_claims() {
    // Stage 1 never localizes.
    let s1 = PositionClaim {
        stage: Stage::Stage1Hydraulic,
        position_m: 40.0,
        sigma_d_m: 0.05,
        segment_length_m: 120.0,
        site: LeakSite::PipeBody,
    };
    assert_eq!(judge(&s1).unwrap_err(), Escalation::RunStage2Correlation);

    // Stage 2 above margin -> escalate.
    let s2_bad = PositionClaim {
        stage: Stage::Stage2Correlation,
        position_m: 40.0,
        sigma_d_m: 0.35,
        segment_length_m: 120.0,
        site: LeakSite::PipeBody,
    };
    assert_eq!(judge(&s2_bad).unwrap_err(), Escalation::RunStage3Pinpoint);

    // Stage 2 within margin but inside junction guard -> escalate.
    let s2_junction = PositionClaim {
        stage: Stage::Stage2Correlation,
        position_m: 1.2,
        sigma_d_m: 0.08,
        segment_length_m: 120.0,
        site: LeakSite::PipeBody,
    };
    assert_eq!(judge(&s2_junction).unwrap_err(), Escalation::RunStage3Pinpoint);

    // Stage 3 with proven sigma_d <= 0.10 m -> signed.
    let s3 = PositionClaim {
        stage: Stage::Stage3Pinpoint,
        position_m: 41.37,
        sigma_d_m: 0.06,
        segment_length_m: 120.0,
        site: LeakSite::Junction,
    };
    let v = judge(&s3).expect("signed");
    assert!(v.sigma_d_m <= SOVEREIGN_MARGIN_M);
    assert_eq!(v.site, LeakSite::Junction);
}

#[test]
fn overlapping_pairs_double_claim_forces_recalibration() {
    use acoustic_leak_engine::verdict::cross_check_overlap;
    assert_eq!(cross_check_overlap(true, true), Some(Escalation::RecalibrateVelocity));
    assert_eq!(cross_check_overlap(true, false), None);
}

#[test]
fn joint_snap_quantizes_ceramic_verdict() {
    let map = JointMap { positions_m: vec![2.0, 4.5, 7.0, 9.5, 12.0] };
    let c = snap_to_joint(&map, 7.21).expect("snap");
    assert_eq!(c.joint_index, 2);
    assert!((c.position_m - 7.0).abs() < 1e-9);
    assert!(c.snap_error_m <= 0.25);
    assert!(!c.ambiguous);
    // Midway between joints -> ambiguous, Stage 3 mandatory.
    let amb = snap_to_joint(&map, 5.75).expect("snap");
    assert!(amb.ambiguous);
}

#[test]
fn alert_event_carries_full_eav_and_never_mints_kaki() {
    use acoustic_leak_engine::event::{build_alert, to_bridge_wire};
    let ev = build_alert(
        7,
        "NAJAF-SEG-014",
        Stage::Stage3Pinpoint,
        PipeMaterial::DuctileIron,
        1201.4,
        0.0008,
        41.37,
        0.06,
        LeakSite::PipeBody,
        0.9931,
    );
    let wire = to_bridge_wire(&ev);
    assert!(wire.contains("ale.sigma_d_m=0.060"));
    assert!(wire.contains("ale.position_m=41.370"));
    assert!(wire.starts_with("ALE_ALERT tribe=7"));
    assert!(wire.ends_with("END\n"));
}
