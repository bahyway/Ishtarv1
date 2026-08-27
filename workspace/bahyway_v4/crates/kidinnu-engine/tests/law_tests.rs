use kidinnu_engine::types::*;
use kidinnu_engine::minimax::solve;
use kidinnu_engine::fadam::danger_path;

fn shell() -> Vec<Zone> {
    // 126 zones: rings 7·14·21·28·35·21
    let secs = [7u16, 14, 21, 28, 35, 21];
    let radii = [63.0, 105.0, 147.0, 189.0, 231.0, 273.0];
    let mut zs = Vec::new();
    let mut id = 0u16;
    for (ri, (&ns, &rc)) in secs.iter().zip(radii.iter()).enumerate() {
        for s in 0..ns {
            let a = (s as f64 + 0.5) / ns as f64 * std::f64::consts::TAU;
            zs.push(Zone {
                id, ring: ri as u8, sector: s,
                pop: 3000 + (id as u32 * 37) % 11000,
                cx: 380.0 + rc * a.cos(), cy: 350.0 + rc * a.sin(),
                eps: EpsilonFloor::new(0.07).unwrap(),
            });
            id += 1;
        }
    }
    assert_eq!(zs.len(), 126);
    zs
}
fn refuges() -> Vec<Refuge> {
    vec![
        Refuge { id:"S1".into(), name:"SCHOOL BASEMENT".into(),  x:120.0, y:180.0, cap:190_000, shield:0.55 },
        Refuge { id:"S2".into(), name:"MOSQUE COURTYARD".into(), x:560.0, y:520.0, cap:170_000, shield:1.0 },
        Refuge { id:"S3".into(), name:"RAIL UNDERPASS".into(),   x:540.0, y:130.0, cap:320_000, shield:0.55 },
        Refuge { id:"S4".into(), name:"OPEN FIELD WEST".into(),  x: 60.0, y:420.0, cap:600_000, shield:1.0 },
    ]
}
fn threats(sealed: bool) -> Vec<ThreatTemplate> {
    vec![
        ThreatTemplate { id:"T-FIRE".into(), siren:"long".into(),
            src_x:80.0, src_y:600.0, bearing:-0.7, spread:1.9, sealed },
        ThreatTemplate { id:"T-ARTY".into(), siren:"short4".into(),
            src_x:680.0, src_y:380.0, bearing:std::f64::consts::PI, spread:2.6, sealed },
    ]
}

#[test]
fn f4_epsilon_floor_is_a_type() {
    assert!(EpsilonFloor::new(0.0).is_err());
    assert!(EpsilonFloor::new(-0.1).is_err());
    assert!(EpsilonFloor::new(f64::NAN).is_err());
    assert!(EpsilonFloor::new(0.001).is_ok());
}

#[test]
fn f5_unsealed_templates_never_judge() {
    let r = solve(&shell(), &refuges(), &threats(false), None);
    assert!(r.is_err(), "an unsealed assumption may rehearse; it may never judge");
}

#[test]
fn a2_4_no_full_doors() {
    let zs = shell(); let rf = refuges();
    let asg = solve(&zs, &rf, &threats(true), None).unwrap();
    let mut load: std::collections::HashMap<&str, u64> =
        rf.iter().map(|r| (r.id.as_str(), 0)).collect();
    for a in &asg {
        if let Move::ToRefuge { refuge_id, .. } | Move::HoldUnderground { refuge_id }
            = &a.mv
        {
            *load.get_mut(refuge_id.as_str()).unwrap()
                += zs[a.zone_id as usize].pop as u64;
        }
    }
    for r in &rf {
        assert!(load[r.id.as_str()] <= r.cap as u64,
            "capacity honesty violated at {}", r.id);
    }
}

#[test]
fn a2_3_never_averaged_scenario_flip_safety() {
    // The minimax directive's danger under EVERY sealed template must not
    // exceed the best achievable uniform bound + tolerance; a blend that
    // ignores the disfavored template must not beat minimax in worst case.
    let zs = shell(); let rf = refuges(); let ts = threats(true);
    let asg = solve(&zs, &rf, &ts, None).unwrap();
    for a in asg.iter() {
        let z = &zs[a.zone_id as usize];
        if let Move::ToRefuge { refuge_id, .. } = &a.mv {
            let r = rf.iter().find(|r| &r.id == refuge_id).unwrap();
            let worst_actual = ts.iter()
                .map(|t| danger_path((z.cx, z.cy), (r.x, r.y), r.shield, t, z.eps))
                .fold(0.0f64, f64::max);
            // uniform bound: least worst over all refuges (ignoring capacity),
            // plus slack for the capacity constraint's assignment order.
            let least_worst = rf.iter().map(|rr| ts.iter()
                    .map(|t| danger_path((z.cx, z.cy), (rr.x, rr.y), rr.shield, t, z.eps))
                    .fold(0.0f64, f64::max))
                .fold(f64::MAX, f64::min);
            assert!(worst_actual <= least_worst + 0.35 + z.eps.get(),
                "zone {} exceeds scenario-flip bound", z.id);
        }
    }
}

#[test]
fn a2_6_seal_roundtrip_and_spoof_rejection() {
    use kidinnu_engine::seal::*;
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;
    let sk = SigningKey::generate(&mut OsRng);
    let vk = sk.verifying_key();
    let b = card_bytes(47, &None, "MOVE NORTHEAST -> S3", "2026-W32");
    let sig = seal(&sk, &b);
    assert!(verify(&vk, &b, &sig));
    let spoof = card_bytes(47, &None, "MOVE SOUTHWEST -> S1", "2026-W32");
    assert!(!verify(&vk, &spoof, &sig), "a spoofed order must fail the seal");
}

#[test]
fn a2_8_offline_tablet_is_complete() {
    use kidinnu_engine::export::tablet_line;
    let zs = shell(); let rf = refuges(); let ts = threats(true);
    // full table: every zone × (no siren + each siren)
    let mut lines = 0usize;
    for declared in std::iter::once(None).chain(ts.iter().map(Some)) {
        let asg = solve(&zs, &rf, &ts, declared).unwrap();
        let siren = declared.map(|t| t.siren.clone());
        for a in &asg { let _ = tablet_line(a, &siren); lines += 1; }
    }
    assert_eq!(lines, 126 * 3, "the tablet must cover every zone under every declaration");
}
