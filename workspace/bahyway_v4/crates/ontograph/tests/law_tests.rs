use ontograph::rites::reading::TopoClass;
use ontograph::*;

fn kaki(seq: u8, tribe: u16) -> Kaki {
    let mut b = [0u8; 16];
    b[3] = seq;
    b[4..6].copy_from_slice(&tribe.to_be_bytes());
    b[6] = 0x01;
    b[7] = 0x01;
    let c = Kaki::compute_crc(&b);
    b[14..16].copy_from_slice(&c.to_be_bytes());
    Kaki(b)
}

#[test]
fn kaki_crc_verifies_and_rejects_tamper() {
    let k = kaki(1, 7);
    assert!(k.verify());
    let mut t = k.0;
    t[8] ^= 0xFF; // reserved byte tampered
    assert!(!Kaki(t).verify());
}

#[test]
fn mandatory_write_is_a_law_breach() {
    let m = Attribute {
        layer: Layer::Mandatory,
        name: "state.class",
    };
    assert!(eav::assert_writable(&m).is_err());
    let o = Attribute {
        layer: Layer::Optional,
        name: "onto.concept_id",
    };
    assert!(eav::assert_writable(&o).is_ok());
}

#[test]
fn lattice_has_stable_spine_and_closed_extents() {
    let mut ctx = FormalContext::new();
    ctx.add_particle(
        kaki(1, 7),
        &[
            "w5h2.who",
            "state.class",
            TopoClass::Golden.attribute_name(),
            "dmbok.steward",
        ],
    )
    .unwrap();
    ctx.add_particle(
        kaki(2, 7),
        &[
            "w5h2.who",
            "state.class",
            TopoClass::Golden.attribute_name(),
        ],
    )
    .unwrap();
    ctx.add_particle(
        kaki(3, 7),
        &["w5h2.who", "state.class", TopoClass::Fuzzy.attribute_name()],
    )
    .unwrap();
    let lat = Lattice::compute(&ctx);
    assert!(lat.is_closed(&ctx));
    assert!(lat.concepts.len() >= 3);
    // spine: the top concept holds all particles
    assert!(lat.concepts.iter().any(|c| c.extent.len() == 3));
}

#[test]
fn nebuchadnezzar_writes_optional_only_and_proposes_universals() {
    let mut ctx = FormalContext::new();
    ctx.add_particle(kaki(1, 7), &["w5h2.who", "state.class", "topo.golden"])
        .unwrap();
    ctx.add_particle(kaki(2, 7), &["w5h2.who", "state.class", "topo.dead"])
        .unwrap();
    let lat = Lattice::compute(&ctx);
    let neb = Nebuchadnezzar::mint(&ctx, lat).unwrap();
    assert_eq!(neb.name, "Nebuchadnezzar");
    assert!(neb.writes.iter().all(|w| w.attribute.starts_with("onto.")));
    let uni = neb.propose_universal(&ctx);
    assert!(uni.contains(&"w5h2.who".to_string()));
    assert!(!uni.contains(&"topo.golden".to_string()));
}
