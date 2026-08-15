# SPEC-MAS-001 — Massartu Pattern Core Boundary

Status: SEALED (ratified by this build — playbook_242).

## What Massartu is

Massartu ("the watch," Akkadian) is a domain-neutral pattern crate
(`massartu-core`) implementing the Analysis-to-Solution Law's own
four-step chain — DETECT → PROVE → PREDICT → PRESCRIBE (GL-MRD-002
Rev.2 §7) — as a literal generic function, `watch_cycle`, rather than
as a name for a shape that individual domains reimplement by
convention. Any Tribe that supplies real `ResidualLaw`, `Prover`,
`Horizon`, `Exposure`, `RiskFis`, `Prescriber`, `Escalator`, and
`Chronicler` implementations gets the four-step law for free, correct
by construction, instead of a fourth hand-written copy of the same
control flow.

## Honest source note (2026-07-25)

An earlier draft of this pattern named the chain but did not execute
it: `watch_cycle` skipped PROVE entirely, and its own test suite drove
two scenarios (`hospital_generator_reaches_herald`,
`desert_line_same_orbit_lower_tier`) off of byte-identical hardcoded
`KappaHistory` data, varying only `exposure`. That is a real and valid
test of exposure-separation (see below), but it is not evidence of
nucleus-exchange invariance (PH-002 Corollary 1) — it never exercised
a genuine domain-specific `ResidualLaw`/`Prover` implementation, so it
could not have caught a bug specific to wiring a real nucleus in.

This build closes that gap:

- `Prover` is now a real trait with two methods, both invoked by
  `watch_cycle` before any Sound tier is escalated or before an
  exclusion is ever prescribed:
  - `residual_trustworthy` — gates DETECT. Defaults to `true`: no
    King-plot-style residual classifier exists anywhere in this
    ecosystem yet (`docs/GL-EGD-001.md` already names this as a
    load-bearing gap, not polish). A domain earns a real answer here
    only by overriding the default with its own classifier.
  - `isolation_proven` — gates PRESCRIBE. Defaults to `false`. A
    domain must wire a real exclusion simulator before Massartu will
    ever prescribe `Prescription::ExcludeProven`/`Repair`; every
    other domain honestly gets `Prescription::MitigateOnly`.
- `Tier::Unproven` — a fifth tier for when `residual_trustworthy`
  returns `false`; the pattern escalates the data-quality problem
  itself rather than guessing at an unknown risk.
- `watch_cycle` now literally performs DETECT → PROVE → PREDICT →
  PRESCRIBE in that order, short-circuiting to `Unproven` if PROVE's
  first gate fails, and consulting PROVE's second gate before
  choosing between `ExcludeProven`/`Repair` and `MitigateOnly`.
- The crate's test suite (`crates/massartu-core/src/lib.rs`,
  `tests::nucleus_exchange::
  same_watch_cycle_drives_a_real_physics_nucleus_and_a_manual_one`)
  now drives the identical, unmodified `watch_cycle` against two
  structurally different nuclei:
  - **Electricity** — `ResidualLaw::kappa_history` computed from a
    real `fdd_core::residuals` call over an actual `egd_engine::
    Network<Phasor>` (a redundant double-feed, admittance-Y edges),
    with current theft simulated across days by mutating the
    monitored node's injection — genuine physics output, not
    fabricated numbers (the test asserts the resulting kappa series
    is actually increasing). `Prover::isolation_proven` is wired to
    the real `egd_engine::exclusion::simulate_exclusion`, which
    returns `Verdict::Proven` here because the redundant spare edge
    genuinely keeps the node served.
  - **Generic/manual** — a domain with no dedicated engine at all
    (e.g. a technician-logged hospital generator). Its `Prover` uses
    the trait's honest defaults: residual trusted, isolation never
    proven, because no exclusion simulator exists for it. The test
    asserts its escalation is correctly capped at `MitigateOnly`/
    `Inspect`, never `ExcludeProven`/`Repair`.

  Both nuclei correctly reach a non-Sound, non-Unproven tier under
  the identical `watch_cycle` call — that is the real evidence PH-002
  Corollary 1 needed and did not previously have.
- The original exposure-separation tests are preserved (adapted to
  the new trait/argument shape, and fixed for a real bug found while
  adapting them — see below) as
  `tests::hospital_generator_reaches_herald_desert_line_does_not`:
  identical kappa trend, different `Exposure`, different `Tier`. This
  remains a distinct and still-valid claim from nucleus-exchange, and
  is kept as its own test rather than folded into the new one.
- A silence-window check (`Escalator::silence_elapsed`) was added so
  `watch_cycle` does not re-alert every cycle for an unchanged
  condition — this existed as a named method in the earlier draft's
  `Escalator` trait but nothing in `watch_cycle` actually called it.

**Bug found and fixed while building the real cross-domain test**: the
first draft of every `KappaHistory` in this crate (including the
adapted originals) ordered samples with ascending non-negative time
values (`day 0..n`). `trend-core`'s own contract
(`crates/trend-core/src/lib.rs`, `Sample` doc comment and its own
test `degrading_series_predicts_correct_crossing`) defines `t=0` as
*now* and requires past samples at negative `t`. Feeding it
non-negative, oldest-first `t` values silently inverted which end of
the series `time_to_threshold` treated as "now," making every
degrading trend in this crate's tests evaluate as already past its
own prediction window and report `Sound` instead of an escalating
tier — caught because the corrected cross-domain test asserted the
electricity nucleus's kappa history was actually increasing and then
asserted the resulting tier was not `Sound`; both original tests
failed until every history was rebuilt with the most recent sample at
`t=0` and earlier samples at negative `t`. This is the same
"round-trip / cross-domain test catches what a hand-derived
single-case assertion hides" pattern already seen twice earlier in
this build (egd-engine's state-estimation sign bug, trend-core's own
flatness-check bug).

## Domain-neutral (lives in `massartu-core`)

- `KappaHistory`, `ResidualLaw`, `Prover` (both gates), `Horizon`,
  `Exposure`, `Tier` (five variants), `Prescription` (four variants),
  `RiskFis`, `Prescriber`, `Escalator`, `Chronicler`, and the
  `watch_cycle` function that wires them together in the mandated
  order.
- `TrendHorizon` — a real `Horizon` built on `trend_core::
  time_to_threshold` (the same least-squares primitive Gibil's own
  horizon and Marduk's T_golden already use — no fourth
  reimplementation).
- `SimpleRiskFis`, `SimplePrescriber`, `InMemoryEscalator`,
  `InMemoryChronicler`, `FixedExposure` — real, usable default
  implementations, not test-only stubs, so a small deployment can use
  the pattern without writing eight trait implementations on day one.

Not yet in `massartu-core` (explicitly out of scope for this build):
any King-plot-style residual classifier (so `residual_trustworthy`
stays at its trusting default everywhere until one exists), and any
concrete exclusion simulator besides electricity's (`egd_engine::
exclusion::simulate_exclusion`) — every other domain's
`isolation_proven` is `false` until it builds one. Both are
load-bearing gaps, tracked as follow-on work, not silently implied to
be present.

## Naming (NL-001, amendment A2 — pattern naming)

NL-001's existing convention (see `docs/SPEC-FDD-001.md` §Naming)
covers engines (descriptive-acronym, e.g. EGDEngine) and calculi
(god-named, e.g. Gibil). Massartu is neither an engine nor a domain
calculus — it is a reusable cross-domain *pattern* (a procedure any
domain's calculus can be run through), a category NL-001 had no rule
for. This spec proposes amendment **A2**: patterns are god-named after
a deity's *role or verb*, not their domain, since a pattern by
definition has no single domain — "Massartu" (the watch) names what
the pattern *does*, exactly as "Gibil" names what a calculus *is* for
one specific domain. (Amendment **A1**, artifact naming, is the
sibling convention already established informally in this ecosystem
for file/format-shaped things; A2 is recorded here as its structural
counterpart for procedure-shaped things.)

One naming collision was caught and resolved while drafting this
pattern's worked examples: an earlier draft's illustrative
gas-pipeline calculus was named "Nergal," which collides with the
real, shipped `steward-lens::AlertSeverity::Nergal`. No gas-pipeline
engine exists in this workspace yet (only `egd-engine` for
electricity and `wpd-engine` for water are real), so this build does
not implement one — but the name is reserved now, the same way Gibil
was reserved for electricity before `egd-engine` existed: the future
gas-pipeline calculus is **Ishum**, Erra's restraining companion in
the Erra Epic (Ishum's role — holding Erra's destructive fire in
check until judgment is warranted — reads directly onto Massartu's
own "the watch" theme). Confirmed unused anywhere in the workspace
(`grep -rli "\bishum\b" --include=*.rs --include=*.md docs crates`)
at the time of this build.

## See also

- `docs/PH-002_Puhu_Law.md` — the TOP Algebra law this pattern's
  nucleus-exchange invariance is Corollary 1 of.
- `crates/massartu-core/src/lib.rs` — the real implementation and
  test suite this spec describes.
