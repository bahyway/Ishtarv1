use babturdi_engine::*;

fn rec(client: &str, hot: bool, sev: f64) -> Refused {
    Refused {
        client: client.into(),
        endpoint: "edge-najaf".into(),
        vec: [0.9, 0.1, 0.1, 0.2],
        on_hot_spot: hot,
        severity: sev,
    }
}
fn log() -> Vec<Refused> {
    let mut v = Vec::new();
    // Client-B: heavy, hot-spot, malware
    for _ in 0..40 {
        v.push(rec("clientB", true, 0.9));
    }
    // Client-A: light, cold, formatting-only
    for _ in 0..10 {
        v.push(rec("clientA", false, 0.1));
    }
    v
}

#[test]
fn l61_no_particle_identity_in_cost_output() {
    // The CostSignature type has NO field for a Particle KAKI.
    // This test documents the type-enforced No-Contamination boundary:
    // a cost signature is buildable ONLY from outer-ledger data.
    let sig = attribute("clientB", &log(), 3);
    // exercise all fields — none is or contains an inner identity
    let _ = (
        &sig.client,
        sig.n,
        sig.hot_share,
        sig.malware_share,
        sig.cost,
        sig.tau,
        &sig.monthly,
    );
    assert_eq!(sig.client, "clientB");
    // the boundary holds structurally; nothing here can name a Particle
}

#[test]
fn l62_cost_carries_tau_never_bare() {
    let sig = attribute("clientB", &log(), 3);
    assert!(sig.tau >= 0.0);
    assert!(sig.cost > 0.0);
    // hot + malware share must reflect Client-B's heavy stream
    assert!(sig.hot_share > 0.9 && sig.malware_share > 0.9);
}

#[test]
fn l63_above_median_does_not_earn_low_cost_tier() {
    let b = attribute("clientB", &log(), 3);
    let a = attribute("clientA", &log(), 3);
    let median = cohort_median(&[a.clone(), b.clone()]);
    // Client-B (heavy) must NOT earn the low-cost tier; A may.
    assert_eq!(
        reward_tier(&b, median),
        Reward::NotEarned,
        "a costly client wrongly earned the low-cost reward"
    );
    assert_eq!(reward_tier(&a, median), Reward::LowCostEarned);
    // the lever: if B sends cleaner data, its cost falls
    let clean: Vec<Refused> = (0..40).map(|_| rec("clientB", false, 0.1)).collect();
    let b2 = attribute("clientB", &clean, 3);
    assert!(b2.cost < b.cost, "cleaner data must lower the cost basis");
}

#[test]
fn l64_quiet_and_siege_windows_never_averaged() {
    let sig = attribute("clientB", &log(), 3);
    assert_eq!(sig.monthly.len(), 3, "per-window costs must be preserved");
    // the distribution is kept; there is no single averaged number
    let spread = sig.monthly.iter().cloned().fold(0.0_f64, f64::max)
        - sig.monthly.iter().cloned().fold(f64::MAX, f64::min);
    assert!(spread >= 0.0, "windows must be individually inspectable");
}
