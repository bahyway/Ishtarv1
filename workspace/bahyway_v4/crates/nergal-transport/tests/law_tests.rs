use nergal_transport::*;

/// Ring-of-sectors mesh: every sector copies to 2 neighbours.
fn ring_mesh(n: usize, mat: Material) -> Mesh {
    Mesh {
        adj: (0..n).map(|i| vec![(i + 1) % n, (i + n - 1) % n]).collect(),
        mat: vec![mat; n],
    }
}

#[test]
fn l45_subcritical_outbreaks_always_die() {
    // p_absorb .5, p_fission .1 nu=2 -> k = .4 + .2 = 0.6 < 1
    let m = Material {
        p_absorb: 0.5,
        p_fission: 0.1,
        nu: 2,
    };
    assert!(k_eff(&m) < 1.0);
    let mesh = ring_mesh(400, m);
    for seed in 1..=30u64 {
        let mut rng = Rng::new(seed);
        let ev = outbreak(&mesh, 7, &mut rng, 100_000);
        assert!(
            ev < 100_000,
            "subcritical chain hit the cap (seed {})",
            seed
        );
    }
}

#[test]
fn l46_the_control_rod_works() {
    // supercritical: p_absorb .1, p_fission .3 nu=2 -> k = .6 + .6 = 1.2
    // A mildly supercritical branching process from a single walker does
    // NOT invade with certainty: the offspring distribution (0 w.p. .1,
    // 1 w.p. .6, 2 w.p. .3) has extinction probability q solving
    // q = .1 + .6q + .3q^2, i.e. q = 1/3 — so only ~2/3 of outbreaks
    // survive to the cap. 20 trials is too few to pin that down (the
    // first 20 SplitMix64 seeds happen to land low, ~35%, well inside
    // normal sampling variance); 200 trials converges close to the true
    // ~66% rate, so the threshold below is set with margin beneath it.
    let hot = Material {
        p_absorb: 0.10,
        p_fission: 0.3,
        nu: 2,
    };
    assert!(k_eff(&hot) > 1.0);
    let mesh = ring_mesh(400, hot);
    let mut invaded = 0;
    let trials = 200u64;
    for seed in 1..=trials {
        if outbreak(&mesh, 7, &mut Rng::new(seed), 50_000) >= 50_000 {
            invaded += 1;
        }
    }
    assert!(
        invaded >= trials / 2,
        "supercritical medium failed to invade often enough: {}/{}",
        invaded,
        trials
    );
    // Nergal adds absorption (scan-kill): p_absorb .45 -> k = .25+.6? recompute:
    // scatter = 1-.45-.3 = .25; k = .25 + .6 = .85 < 1
    let rodded = Material {
        p_absorb: 0.45,
        p_fission: 0.3,
        nu: 2,
    };
    assert!(k_eff(&rodded) < 1.0);
    let mesh2 = ring_mesh(400, rodded);
    for seed in 1..=trials {
        assert!(
            outbreak(&mesh2, 7, &mut Rng::new(seed), 50_000) < 50_000,
            "the control rod failed (seed {})",
            seed
        );
    }
}

#[test]
fn l47_quarantine_transmission_is_exponential() {
    let (sigma, d) = (0.6f64, 5usize);
    let mut rng = Rng::new(99);
    let t = barrier_transmission(sigma, d, 200_000, &mut rng);
    let expect = (-sigma * d as f64).exp();
    assert!(
        (t - expect).abs() < 0.005,
        "breach probability {} != e^-tau {}",
        t,
        expect
    );
}

#[test]
fn l48_importance_scanning_beats_uniform() {
    // 1000 sectors; infections cluster in the hot region 100..120
    let infected: Vec<usize> = (100..120).collect();
    let uniform: Vec<usize> = (0..1000).collect();
    let mut weighted: Vec<usize> = (100..1000).collect();
    weighted.splice(0..0, 100..120); // hot region first
    let a = scans_to_find(&uniform, &infected);
    let b = scans_to_find(&weighted, &infected);
    assert!(
        b * 5 < a,
        "importance scanning not >=5x cheaper: {} vs {}",
        b,
        a
    );
}
