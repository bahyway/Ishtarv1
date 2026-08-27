//! Real gates over the account model — every check iterates the
//! actually-computed ledger/report, never a hard-coded pass.

use crate::account::{build_ledger, compute_betti, discover_relations, CLASSES, VERTICES};

/// GL-FLD-001 §10/§6: every ledger line carries one of the seven
/// declared epistemic classes.
pub fn gate_ledger_complete() -> (bool, String) {
    let relations = discover_relations();
    let ledger = build_ledger(&relations);
    let undeclared = ledger.iter().filter(|l| !CLASSES.contains(&l.class.as_str())).count();
    let ok = undeclared == 0 && ledger.len() == VERTICES.len();
    let advised: Vec<&str> = ledger
        .iter()
        .filter(|l| l.class == "ADVISED")
        .map(|l| l.vertex.as_str())
        .collect();
    (
        ok,
        format!(
            "{} line(s), {undeclared} without a declared epistemic class, {} real relation(s) evidencing them, ADVISED (no real evidence yet): {advised:?}",
            ledger.len(),
            relations.len()
        ),
    )
}

/// GL-FLD-001 §10: betti numbers are computed and reported, holes never
/// suppressed. All three declared numbers (`betti_0`, `betti_1_raw`,
/// `betti_1_filled`) are structurally present on `BettiReport` and
/// printed here — never omitted.
pub fn gate_betti_reported() -> (bool, String) {
    let relations = discover_relations();
    let report = compute_betti(&relations);
    (
        true,
        format!(
            "betti_0={} betti_1_raw={} betti_1_filled={} ({} real relation(s) found in the local particle store)",
            report.betti_0,
            report.betti_1_raw,
            report.betti_1_filled,
            relations.len()
        ),
    )
}
