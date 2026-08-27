//! Compiles a `heptascript::HeptaQuery`'s WHERE and WHY clauses into Z3
//! `Real`/`Bool` constraints and checks satisfiability.
//!
//! ## Encoding
//!
//! Every `(entity var, attribute)` pair touched by a numeric comparison
//! gets one Z3 `Real` constant. Text/lane/bool literals are not modeled
//! with Z3's string theory (unnecessary complexity for equality-only
//! comparisons) — each distinct literal seen for a given `(var, attr)`
//! is assigned a unique synthetic integer id, and equality/inequality on
//! that literal becomes equality/inequality on the id. This is enough to
//! prove `WHERE E[status] = "active" AND E[status] = "closed"` UNSAT
//! without needing a full string theory.
//!
//! Every attribute reference also gets a Z3 `Bool` "exists" constant,
//! because the real engine (`heptascript::engine::eval_where_cond`)
//! treats a missing attribute as `Cmp` failing but `NotExists` passing —
//! so a `Cmp` condition compiles to `exists ∧ <value constraint>`, letting
//! `WHERE E[x] > 5 AND E[x] NOT EXISTS` correctly compile to UNSAT (no
//! entity can simultaneously have and lack the same attribute).
//!
//! WHERE and WHY conditions fold left-to-right exactly like
//! `heptascript::engine::eval_where`/`eval_why` do at runtime — each
//! condition's `Combinator` says how it joins with the *running* result,
//! not with its immediate neighbor pairwise, so this compiler mirrors
//! that fold instruction-for-instruction rather than assuming AND-only
//! grouping.
//!
//! Per Gate G4, this module (and this whole crate) is dev-time-only: it
//! is never a dependency of any shipped sovereign server binary.
//!
//! NOTE (z3 crate 0.12 -> 0.20 migration): the old `Config`/`Context`
//! objects a caller had to construct and thread through every AST node
//! (`Bool<'ctx>`, `Real<'ctx>`, `ctx: &'ctx Context` on every function)
//! are gone in 0.20 -- the crate now manages an ambient context
//! internally (`Context::new` is private; `Solver::new()` takes no
//! arguments). This file's functions and types dropped that threading
//! accordingly; the logic itself is unchanged.

use std::collections::HashMap;

use heptascript::{
    Combinator, ConditionTest, GravityClause, GravityMode, HeptaQuery, HeptaValue, LaneValue, Op,
    WhyCondition,
};
use z3::ast::{Bool, Real};
use z3::{SatResult, Solver};

/// Result of checking one query's WHERE/WHY clauses for satisfiability.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SatOutcome {
    /// At least one hypothetical entity could satisfy every clause.
    Satisfiable,
    /// No entity could ever satisfy the clauses — the query is
    /// self-contradictory and would return empty no matter what data
    /// exists.
    Unsatisfiable,
    /// Z3 could not decide within its default resource bounds.
    Unknown,
}

/// A validated query report: the decision plus, on `Unsatisfiable`, which
/// labeled parts of the query participated in the contradiction (as
/// reported by Z3's unsat core).
#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub outcome: SatOutcome,
    /// Human-readable labels of the assertions that caused UNSAT.
    /// Empty when `outcome != Unsatisfiable`.
    pub contradiction_sources: Vec<String>,
    /// Non-empty when the query's `GRAVITY` clause is structurally safe
    /// per the parser (a `BAND`/`MAX_GROUPS` bound is always present —
    /// enforced at parse time, not here) but the chosen `MAX_GROUPS` value
    /// is large enough that it no longer functions as a meaningful safety
    /// cap in practice. Independent of `outcome` — a query can be
    /// perfectly satisfiable and still carry an unsafe `GRAVITY` cap.
    pub gravity_warnings: Vec<String>,
}

/// Above this many exact-match groups, `MAX_GROUPS n` stops functioning as
/// a meaningful safety cap in practice: the per-group accumulator memory
/// itself stays small (a `GravityGroup` is on the order of 150 bytes with
/// `HashMap` overhead), but a result set this wide is not something any
/// real consumer (a Read Node response frame, a DubSar PDM table) can
/// usefully consume — the same reason no database's `GROUP BY` is expected
/// to usefully return millions of rows to one caller. A query needing to
/// partition at this cardinality wants `GRAVITY BAND` (bounded by
/// attribute *range*, not value cardinality), not a bigger cap.
const MAX_REASONABLE_GROUPS: usize = 100_000;

/// Structural GRAVITY check — not a Z3 satisfiability question, so it runs
/// as a plain magnitude check rather than an SMT assertion. Kept separate
/// from `check_query`'s SAT/UNSAT decision so a caller can act on it
/// (reject, or just surface a warning) independently of whether the
/// query's WHERE/WHY is itself satisfiable.
fn check_gravity_cap(query: &HeptaQuery) -> Vec<String> {
    match &query.gravity {
        Some(GravityClause {
            mode: GravityMode::ExactMatchCapped(n),
            attr,
            ..
        }) if *n > MAX_REASONABLE_GROUPS => {
            vec![format!(
                "GRAVITY E[{attr}] MAX_GROUPS {n} exceeds the practical cap of {MAX_REASONABLE_GROUPS} groups \
                 -- MAX_GROUPS is present (satisfying the parser's mandatory-cap rule) but too large to bound \
                 the result usefully; use GRAVITY BAND for a high-cardinality numeric attribute instead."
            )]
        }
        _ => Vec::new(),
    }
}

/// Compile `query`'s WHERE and WHY clauses into Z3 constraints and check
/// whether they could ever simultaneously hold for some entity.
///
/// This does not touch WHO/WHAT/WHEN/HOW/HOW_MUCH or any v2.0 routing
/// clause (NODE/ACROSS/TIER/...) — those select or shape rows, they do
/// not constrain a single entity's attribute values, so there is nothing
/// for a satisfiability check to contradict.
pub fn check_query(query: &HeptaQuery) -> ValidationReport {
    let solver = Solver::new();
    let mut vars = VarPool::new();

    // `assert_and_track` requires a plain constant as the tracking atom;
    // Z3's SMT-LIB printer bar-quotes any name containing whitespace or
    // brackets, which would make parsing `get_unsat_core()`'s output
    // fragile. So tracking atoms get simple sequential names ("t0",
    // "t1", ...) and the human-readable description lives in `track_
    // labels`, indexed the same way — looked up after the fact instead
    // of parsed out of Z3's own printer.
    let mut track_labels: Vec<String> = Vec::new();
    let mut track = |solver: &Solver, formula: &Bool, label: String| {
        let name = format!("t{}", track_labels.len());
        solver.assert_and_track(formula, &Bool::new_const(name));
        track_labels.push(label);
    };

    // Always-true domain fact: QUALITY_BYTE is a real u8 in the AST
    // already (see heptascript::WhyCondition::QualityByte), so this bound
    // can never itself be violated by a parsed query — it is asserted
    // anyway so any future WHY clause built directly against this
    // compiler (bypassing the parser's u8 typing) is still caught.
    let quality_byte = vars.real(&EntityAttr::why("quality.byte"));
    let quality_lo = quality_byte.ge(Real::from_rational(0, 1));
    let quality_hi = quality_byte.le(Real::from_rational(255, 1));
    track(
        &solver,
        &Bool::and(&[&quality_lo, &quality_hi]),
        "quality_byte_domain[0,255]".to_string(),
    );

    // WHERE — assert individually when every condition after the first
    // is AND-joined (safe to decompose for a fine-grained unsat core);
    // otherwise assert the single left-to-right fold as one opaque fact
    // (still correct, just coarser on which part is blamed).
    if !query.r#where.is_empty() {
        if is_pure_and_chain(query.r#where.iter().map(|c| &c.combinator)) {
            for (i, cond) in query.r#where.iter().enumerate() {
                let f = compile_where_leaf(&mut vars, cond);
                track(
                    &solver,
                    &f,
                    format!("WHERE[{i}] {}[{}]", cond.var, cond.attr),
                );
            }
        } else {
            let f = fold_where(&mut vars, &query.r#where);
            track(&solver, &f, "WHERE (mixed AND/OR fold)".to_string());
        }
    }

    // WHY — same decomposition strategy.
    if !query.why.is_empty() {
        if is_pure_and_chain(query.why.iter().map(why_combinator)) {
            for (i, cond) in query.why.iter().enumerate() {
                let f = compile_why_leaf(&mut vars, cond);
                track(&solver, &f, format!("WHY[{i}]"));
            }
        } else {
            let f = fold_why(&mut vars, &query.why);
            track(&solver, &f, "WHY (mixed AND/OR fold)".to_string());
        }
    }

    let gravity_warnings = check_gravity_cap(query);

    match solver.check() {
        SatResult::Sat => ValidationReport {
            outcome: SatOutcome::Satisfiable,
            contradiction_sources: Vec::new(),
            gravity_warnings,
        },
        SatResult::Unknown => ValidationReport {
            outcome: SatOutcome::Unknown,
            contradiction_sources: Vec::new(),
            gravity_warnings,
        },
        SatResult::Unsat => {
            let core = solver.get_unsat_core();
            let contradiction_sources: Vec<String> = core
                .iter()
                .filter_map(|b| {
                    let name = b.to_string();
                    name.strip_prefix('t')
                        .and_then(|n| n.parse::<usize>().ok())
                        .and_then(|i| track_labels.get(i).cloned())
                })
                .collect();
            // An empty/unmapped core must never be reported as "no
            // contradiction" when the solver already said Unsat.
            let contradiction_sources = if contradiction_sources.is_empty() {
                track_labels
            } else {
                contradiction_sources
            };
            ValidationReport {
                outcome: SatOutcome::Unsatisfiable,
                contradiction_sources,
                gravity_warnings,
            }
        }
    }
}

fn why_combinator(c: &WhyCondition) -> &Combinator {
    match c {
        WhyCondition::Lane { combinator, .. } => combinator,
        WhyCondition::QualityByte { combinator, .. } => combinator,
        WhyCondition::AttrExists { combinator, .. } => combinator,
        WhyCondition::AttrNotExists { combinator, .. } => combinator,
    }
}

/// True when every combinator (the first condition has none — it is
/// vacuously fine) is `And`, meaning the left-to-right fold is equivalent
/// to a plain conjunction of every leaf and can be decomposed into
/// independent asserts without changing meaning.
fn is_pure_and_chain<'a, I: Iterator<Item = &'a Combinator>>(mut combinators: I) -> bool {
    combinators.all(|c| matches!(c, Combinator::And))
}

fn fold_where(vars: &mut VarPool, conditions: &[heptascript::WhereCondition]) -> Bool {
    let mut acc = compile_where_leaf(vars, &conditions[0]);
    for cond in &conditions[1..] {
        let this = compile_where_leaf(vars, cond);
        acc = match cond.combinator {
            Combinator::And => Bool::and(&[&acc, &this]),
            Combinator::Or => Bool::or(&[&acc, &this]),
        };
    }
    acc
}

fn fold_why(vars: &mut VarPool, conditions: &[WhyCondition]) -> Bool {
    let mut acc = compile_why_leaf(vars, &conditions[0]);
    for cond in &conditions[1..] {
        let this = compile_why_leaf(vars, cond);
        acc = match why_combinator(cond) {
            Combinator::And => Bool::and(&[&acc, &this]),
            Combinator::Or => Bool::or(&[&acc, &this]),
        };
    }
    acc
}

fn compile_where_leaf(vars: &mut VarPool, cond: &heptascript::WhereCondition) -> Bool {
    let key = EntityAttr::var(&cond.var, &cond.attr);
    match &cond.test {
        ConditionTest::Exists => vars.exists(&key).clone(),
        ConditionTest::NotExists => vars.exists(&key).not(),
        ConditionTest::Cmp { op, val } => {
            let exists = vars.exists(&key).clone();
            let value_ok = compile_cmp(vars, &key, op, val);
            Bool::and(&[&exists, &value_ok])
        }
    }
}

fn compile_why_leaf(vars: &mut VarPool, cond: &WhyCondition) -> Bool {
    match cond {
        WhyCondition::Lane { op, val, .. } => {
            let key = EntityAttr::why("lane");
            let exists = vars.exists(&key).clone();
            let lit = vars.text_literal(&key, lane_str(val));
            let cmp = eq_or_neq(&vars.real(&key), &lit, op);
            Bool::and(&[&exists, &cmp])
        }
        WhyCondition::QualityByte { op, val, .. } => {
            let key = EntityAttr::why("quality.byte");
            let exists = vars.exists(&key).clone();
            let lit = Real::from_rational(i64::from(*val), 1);
            let cmp = ordered_cmp(&vars.real(&key), &lit, op);
            Bool::and(&[&exists, &cmp])
        }
        WhyCondition::AttrExists { attr, .. } => vars.exists(&EntityAttr::why(attr)).clone(),
        WhyCondition::AttrNotExists { attr, .. } => vars.exists(&EntityAttr::why(attr)).not(),
    }
}

fn compile_cmp(vars: &mut VarPool, key: &EntityAttr, op: &Op, val: &HeptaValue) -> Bool {
    let slot = vars.real(key);
    match val {
        HeptaValue::Int(n) => {
            let lit = Real::from_rational_str(&n.to_string(), "1").expect("integer literal");
            ordered_cmp(&slot, &lit, op)
        }
        HeptaValue::Float(f) => {
            let lit = Real::from_rational_str(&format!("{f}"), "1").expect("float literal");
            ordered_cmp(&slot, &lit, op)
        }
        HeptaValue::Text(s) => {
            let lit = vars.text_literal(key, s);
            eq_or_neq(&slot, &lit, op)
        }
        HeptaValue::Bool(b) => {
            let lit = vars.text_literal(key, if *b { "true" } else { "false" });
            eq_or_neq(&slot, &lit, op)
        }
        HeptaValue::Null => {
            let lit = vars.text_literal(key, "\0null");
            eq_or_neq(&slot, &lit, op)
        }
    }
}

/// Full ordering comparison (`Eq/Neq/Gt/Lt/Gte/Lte`) — valid for genuinely
/// numeric slots (ints, floats, `quality.byte`).
fn ordered_cmp(a: &Real, b: &Real, op: &Op) -> Bool {
    match op {
        Op::Eq => a.eq(b),
        Op::Neq => a.eq(b).not(),
        Op::Gt => a.gt(b),
        Op::Lt => a.lt(b),
        Op::Gte => a.ge(b),
        Op::Lte => a.le(b),
    }
}

/// Equality-only comparison for synthetic-id slots (text/bool/null/lane).
/// Ordering operators on these are not meaningful (there is no natural
/// order over arbitrary text in HeptaScript's own `apply_op_ord`, which
/// orders by the *stored* type, never mixed with a synthetic id space),
/// so `Gt/Lt/Gte/Lte` degrade to `Bool::from_bool(true)` — permissive
/// rather than a false contradiction, since this compiler's job is to
/// prove genuine impossibility, never to invent one.
fn eq_or_neq(a: &Real, b: &Real, op: &Op) -> Bool {
    match op {
        Op::Eq => a.eq(b),
        Op::Neq => a.eq(b).not(),
        _ => Bool::from_bool(true),
    }
}

fn lane_str(lane: &LaneValue) -> &'static str {
    match lane {
        LaneValue::Gold => "Gold",
        LaneValue::Silver => "Silver",
        LaneValue::White => "White",
        LaneValue::Gray => "Gray",
        LaneValue::Red => "Red",
        LaneValue::Black => "Black",
    }
}

/// Identifies one `(entity var, attribute)` slot. WHY conditions have no
/// entity var of their own (they apply to whichever entity WHO/WHERE
/// already resolved), so they share a fixed sentinel var name distinct
/// from anything a WHO clause could bind (`Citizens.E` etc. are always
/// alphabetic identifiers).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct EntityAttr {
    var: String,
    attr: String,
}

impl EntityAttr {
    fn var(var: &str, attr: &str) -> Self {
        Self {
            var: var.to_string(),
            attr: attr.to_string(),
        }
    }

    fn why(attr: &str) -> Self {
        Self {
            var: "$WHY".to_string(),
            attr: attr.to_string(),
        }
    }
}

/// Caches the Z3 constants built for each `(var, attr)` slot and each
/// distinct text literal seen, so repeated references to the same slot
/// or literal across a query reuse the identical Z3 AST node.
#[derive(Default)]
struct VarPool {
    reals: HashMap<EntityAttr, Real>,
    exists: HashMap<EntityAttr, Bool>,
    text_ids: HashMap<(EntityAttr, String), i64>,
    next_text_id: i64,
}

impl VarPool {
    fn new() -> Self {
        Self {
            reals: HashMap::new(),
            exists: HashMap::new(),
            text_ids: HashMap::new(),
            // Offset well clear of any plausible numeric business value so
            // a text-literal id can never accidentally coincide with a
            // real numeric comparison against the same slot.
            next_text_id: 1_000_000_000,
        }
    }

    fn real(&mut self, key: &EntityAttr) -> Real {
        self.reals
            .entry(key.clone())
            .or_insert_with(|| Real::new_const(format!("{}.{}", key.var, key.attr)))
            .clone()
    }

    fn exists(&mut self, key: &EntityAttr) -> &Bool {
        self.exists
            .entry(key.clone())
            .or_insert_with(|| Bool::new_const(format!("exists({}.{})", key.var, key.attr)))
    }

    fn text_literal(&mut self, key: &EntityAttr, literal: &str) -> Real {
        let map_key = (key.clone(), literal.to_string());
        let id = match self.text_ids.get(&map_key) {
            Some(id) => *id,
            None => {
                let id = self.next_text_id;
                self.next_text_id += 1;
                self.text_ids.insert(map_key, id);
                id
            }
        };
        Real::from_rational_str(&id.to_string(), "1").expect("synthetic literal id")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use heptascript::parse_query;

    #[test]
    fn satisfiable_numeric_where_reports_sat() {
        let q = parse_query("WHO T.E\nWHERE E[age] > 10 AND E[age] < 50").unwrap();
        let report = check_query(&q);
        assert_eq!(report.outcome, SatOutcome::Satisfiable);
        assert!(report.contradiction_sources.is_empty());
    }

    #[test]
    fn contradictory_numeric_where_reports_unsat_with_both_conditions_named() {
        let q = parse_query("WHO T.E\nWHERE E[age] > 10 AND E[age] < 5").unwrap();
        let report = check_query(&q);
        assert_eq!(report.outcome, SatOutcome::Unsatisfiable);
        assert!(report
            .contradiction_sources
            .iter()
            .any(|s| s.contains("WHERE[0]")));
        assert!(report
            .contradiction_sources
            .iter()
            .any(|s| s.contains("WHERE[1]")));
    }

    #[test]
    fn contradictory_text_equality_reports_unsat() {
        let q = parse_query("WHO T.E\nWHERE E[status] = \"active\" AND E[status] = \"closed\"")
            .unwrap();
        let report = check_query(&q);
        assert_eq!(report.outcome, SatOutcome::Unsatisfiable);
    }

    #[test]
    fn same_text_equality_twice_is_satisfiable() {
        let q = parse_query("WHO T.E\nWHERE E[status] = \"active\" AND E[status] = \"active\"")
            .unwrap();
        let report = check_query(&q);
        assert_eq!(report.outcome, SatOutcome::Satisfiable);
    }

    #[test]
    fn cmp_and_not_exists_on_same_attribute_is_unsatisfiable() {
        // Mirrors heptascript::engine::eval_where_cond: a Cmp condition
        // requires the attribute to exist (returns false via `get_val`
        // returning None otherwise), so ANDing it with NOT EXISTS on the
        // same attribute can never hold for any real entity.
        let q = parse_query("WHO T.E\nWHERE E[age] > 10 AND E[age] NOT EXISTS").unwrap();
        let report = check_query(&q);
        assert_eq!(report.outcome, SatOutcome::Unsatisfiable);
    }

    #[test]
    fn exists_and_not_exists_on_same_attribute_is_unsatisfiable() {
        let q = parse_query("WHO T.E\nWHERE E[phone] EXISTS AND E[phone] NOT EXISTS").unwrap();
        let report = check_query(&q);
        assert_eq!(report.outcome, SatOutcome::Unsatisfiable);
    }

    #[test]
    fn mixed_or_can_rescue_an_otherwise_impossible_chain() {
        // (age > 10 AND age < 5) is impossible alone, but ORing in a
        // reachable third condition makes the whole formula satisfiable
        // — exactly matching eval_where's real left-to-right fold, where
        // `Or` does not distribute the way naive AND-grouping would
        // assume.
        let q = parse_query("WHO T.E\nWHERE E[age] > 10 AND E[age] < 5 OR E[age] = 100").unwrap();
        let report = check_query(&q);
        assert_eq!(report.outcome, SatOutcome::Satisfiable);
    }

    #[test]
    fn why_quality_byte_contradiction_is_unsatisfiable() {
        let q = parse_query(
            "WHO T.E\nWHERE E[x] EXISTS\nWHY QUALITY_BYTE > 200 AND QUALITY_BYTE < 100",
        )
        .unwrap();
        let report = check_query(&q);
        assert_eq!(report.outcome, SatOutcome::Unsatisfiable);
        assert!(report
            .contradiction_sources
            .iter()
            .any(|s| s.starts_with("WHY[")));
    }

    #[test]
    fn why_lane_contradiction_is_unsatisfiable() {
        let q =
            parse_query("WHO T.E\nWHERE E[x] EXISTS\nWHY LANE = Gold AND LANE = Black").unwrap();
        let report = check_query(&q);
        assert_eq!(report.outcome, SatOutcome::Unsatisfiable);
    }

    #[test]
    fn query_with_no_where_or_why_is_trivially_satisfiable() {
        let q = parse_query("WHO T.E").unwrap();
        let report = check_query(&q);
        assert_eq!(report.outcome, SatOutcome::Satisfiable);
        assert!(report.gravity_warnings.is_empty());
    }

    // ── GRAVITY safety check ──────────────────────────────────────────────────

    #[test]
    fn gravity_within_reasonable_cap_has_no_warning() {
        let q = parse_query("WHO T.E\nGRAVITY E[station] MAX_GROUPS 1000\nMEASURE DENSE").unwrap();
        let report = check_query(&q);
        assert_eq!(report.outcome, SatOutcome::Satisfiable);
        assert!(report.gravity_warnings.is_empty());
    }

    #[test]
    fn gravity_max_groups_past_the_practical_ceiling_is_flagged() {
        // The parser's own mandatory-cap rule is satisfied (MAX_GROUPS is
        // present), but a cap this large no longer bounds anything
        // meaningfully -- Gate G4 must still flag it.
        let q = parse_query("WHO T.E\nGRAVITY E[national_id] MAX_GROUPS 5000000\nMEASURE DENSE")
            .unwrap();
        let report = check_query(&q);
        assert_eq!(
            report.outcome,
            SatOutcome::Satisfiable,
            "SAT/UNSAT is independent of the GRAVITY cap"
        );
        assert_eq!(report.gravity_warnings.len(), 1);
        assert!(report.gravity_warnings[0].contains("MAX_GROUPS 5000000"));
        assert!(report.gravity_warnings[0].contains("national_id"));
    }

    #[test]
    fn gravity_band_mode_is_never_flagged_regardless_of_width() {
        // BAND is bounded by (attribute range / width), never by value
        // cardinality, so it has no analogous "too large" failure mode.
        let q = parse_query("WHO T.E\nGRAVITY E[pressure] BAND 0.001\nMEASURE DENSE").unwrap();
        let report = check_query(&q);
        assert!(report.gravity_warnings.is_empty());
    }

    #[test]
    fn no_gravity_clause_has_no_warning() {
        let q = parse_query("WHO T.E\nMEASURE DENSE").unwrap();
        let report = check_query(&q);
        assert!(report.gravity_warnings.is_empty());
    }
}
