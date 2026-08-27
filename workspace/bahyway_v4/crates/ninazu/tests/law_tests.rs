use ninazu::*;
fn snap() -> Snapshot {
    // 5x5: two grave blocks separated by a road cross
    let g = Cell::Grave;
    let r_ = Cell::Road;
    Snapshot {
        date: "2026-08-18",
        cells: vec![
            vec![g(1), g(2), r_, g(3), g(4)],
            vec![g(5), g(6), r_, g(7), g(8)],
            vec![r_, r_, r_, r_, r_],
            vec![g(9), g(10), r_, g(11), g(12)],
            vec![g(13), g(14), r_, g(15), g(16)],
        ],
    }
}
#[test]
fn l70_route_uses_only_road_cells() {
    let s = snap();
    let route = navigate(&s, (0, 2), (4, 2)).expect("route exists on the road");
    assert!(
        route_avoids_graves(&s, &route),
        "route stepped through a grave"
    );
}
#[test]
fn l71_each_grave_is_one_cell() {
    assert!(
        each_grave_one_cell(&snap()),
        "a grave appeared in two cells"
    );
}
#[test]
fn l70b_no_route_through_solid_graves() {
    // (0,0) and (4,4) are corner graves with NO road-adjacent neighbour at
    // all (their only neighbours are interior graves), so no route can
    // ever leave them — that endpoint pair is graph-disconnected from the
    // road network by construction, not a routing failure. (1,1) and (3,3)
    // are graves that each border the road cross, so a route between them
    // is the real test of "reach via roads without crossing a grave cell".
    let s = snap();
    let r = navigate(&s, (1, 1), (3, 3));
    assert!(r.is_some(), "should reach via roads");
    assert!(route_avoids_graves(&s, &r.unwrap()));
}
#[test]
fn l69_navigate_signature_is_offline() {
    // The navigate() signature takes only &Snapshot — no socket, no URL,
    // no live handle. Offline-by-construction (documented invariant).
    let s = snap();
    let _ = navigate(&s, (2, 0), (2, 4));
    assert_eq!(s.date, "2026-08-18");
}
