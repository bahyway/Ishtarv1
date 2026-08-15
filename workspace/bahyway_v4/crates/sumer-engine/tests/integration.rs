//! SumerEngine v4.0 — external consumption, the way other crates
//! (DubSar Theater, ZeroEngine, etc.) will actually use it.

use sumer_engine::{
    meszl, wahshiyya, HeptaDimension, NajafSumerianContext, PlimptonAnchors,
    SovereignSign, TraditionProfile, Vec7DKey,
};

#[test]
fn the_three_projections_are_one_crate() {
    // Script
    assert_eq!(meszl::PHASE1_SIGNS.len(), 15);
    // Mathematics (imported single-source from kinetic-engine)
    let anchors = PlimptonAnchors::sovereign();
    assert!((anchors.ud - 0.36).abs() < 1e-12);
    // Geography
    let ctx = NajafSumerianContext::wadi_al_salam();
    assert!(ctx.distance_to_nippur_km() > 0.0);
}

#[test]
fn sovereign_parameter_space_phase1() {
    // Phase 1: 15 MesZL + 28 Wahshiyya = 43 signs.
    let total = meszl::PHASE1_SIGNS.len() + wahshiyya::all_signs().len();
    assert_eq!(total, 43);
}

#[test]
fn mixed_tradition_key_across_both_civilizations() {
    // 3,800 years apart, one key: Sumerian DUB + SAR with the
    // Wahshiyya Moon anchor (URU).
    let dub = meszl::by_name("DUB").unwrap();
    let sar = meszl::by_name("SAR").unwrap();
    let moon = wahshiyya::planetary_anchor_signs().into_iter().last().unwrap();
    assert_eq!(moon.dimension(), Some(HeptaDimension::URU));
    let key = Vec7DKey::generate_with_rounds(dub, sar, &moon, 256);
    assert_eq!(key.profile, TraditionProfile::Mixed);
    assert!(key.dimensions[HeptaDimension::URU.index()]);
    assert!(key.verify_with_rounds(dub, sar, &moon, 256));
}
