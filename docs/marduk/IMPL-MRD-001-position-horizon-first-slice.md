# IMPL-MRD-001 — MardukEngine First Slice: Position + Horizon + Topology

Status: implementation status note for `crates/marduk-engine`, not the
full GL-MRD-001 glossary/playbook sheet BC-MRD-001 reserves that name
for (GL-MRD-001 does not exist in this repository yet — only
GL-MRD-002, the Nēberu Slicer extension, and GL-ADU-002, the Addu
cyclone extension, are present under `docs/marduk/`). This note is
scoped to what `crates/marduk-engine` actually contains as of
playbook_236, nothing wider.

## Scope decision

BC-MRD-001 defines five invariant verbs: Position, Motion, Curvature,
Topology, Horizon. playbook_236 implemented Position and Horizon;
playbook_238 added Topology. This build implements **three of five**:
Position, Horizon, Topology.

**Why not all five in one pass:** MardukEngine is positioned by its own
roadmap (BC-MRD-001 §9) as gated behind the full multi-phase testing
regime, and GL-MRD-002 already extends it further (Nēberu Slicer:
section-based Betti β₀/β₁ classification, the same primitive `topology.rs`
now provides for plain relation graphs). Building all five verbs plus
four domain calculi (Šazu/Addu/Suhrim/Namtila) in one playbook would
bundle a large, multi-part flagship deliverable with work whose size
and risk profile don't match. Position, Horizon, and Topology are the
three verbs with no open mathematical questions against what's already
built elsewhere in the workspace; Motion and Curvature are deferred
with the reasons below, not silently skipped.

## What's implemented

- **Position** (`marduk_engine::position`): `radial_coordinate` and
  `template_score`, i.e. H(P) = 1/(1+r) under the Hepta Space metric
  g = diag(w1..w7). At a fixed instant with a fixed weight vector this
  metric is flat, so the geodesic distance is exactly the direct
  weighted-Euclidean distance — no ODE integration needed to compute r
  itself. This claim is not asserted untested: `position_matches_geodesic_on_flat_metric`
  integrates the same case through `vgca_engine::riemannian::RiemannianManifold::geodesic`
  (an independently-built, independently-validated covariant-geometry
  engine — christoffel/covariant_derivative/geodesic/ricci already
  exist there, verified against both BahyWay's own metric and an
  analytic 2-sphere case) and confirms the two agree.
- **Horizon** (`marduk_engine::horizon`): `golden_horizon`, i.e.
  T_golden, delegated to `trend-core` (the same least-squares
  trend-to-threshold primitive `egd-engine`'s Gibil horizon uses,
  rather than a third independent reimplementation of the same math).
  B11's sign convention (falling = degrading, opposite of
  Gibil/Enbilulu where kappa rises) is handled by mirroring the series
  before calling the shared primitive — a direct application of the
  Sign-Convention Law (§3.6), which is also recorded in code as
  `marduk_engine::SignConvention`.
- **Topology** (`marduk_engine::topology`): `betti_0` (connected
  components, via union-find) and `betti_1` (circuit rank,
  |E| − |V| + β0) on a plain `RelationGraph` — the exact scope
  BC-MRD-001/GL-MRD-002 ask for ("β0/β1 of relation graphs"), not a
  full simplicial-complex homology engine. β1 > 0 is the graph-theoretic
  signature of a mule ring (Šazu, §4.1) or an organized illegal-
  connection ring: an independent loop in the relation graph. Tested
  against a figure-eight (two overlapping cycles, β1=2) as the closest
  cheap analog to that signature.

## What's deliberately NOT implemented here

- **Motion** (∇r/dt, covariant, across a *recalibrated* metric/template
  — §3.2-§3.3): this is the verb that actually needs
  `vgca_engine::riemannian`'s covariant derivative and parallel
  transport (Position does not, being a same-instant, fixed-metric
  calculation). Not built in this slice.
- **Curvature** (geodesic deviation as systemic cause, §3.4, Jacobi
  equation): needs the full Riemann tensor, not just the Ricci
  contraction `vgca-engine` currently exposes. Not built.
- **Domain calculi** (Šazu/Addu/Suhrim/Namtila, §4): none implemented.
  This crate is domain-agnostic core only.

## Naming note

"Nabu Calculus" (the covariant-geometry mathematics named for the god
Nabû, BC-MRD-001 §3) is unrelated to `nabu`, the real BahyWay CLI
(`docs/11_tooling/nabu_cli.md`, commands like `nabu probe`/`nabu tribe`/
`nabu ingest`). Not a Rust-identifier collision — this crate claims no
`nabu` binary name — but both are legitimately "Nabu" in conversation
and playbook prose, so it's recorded once in `marduk-engine`'s crate
doc comment to preempt confusion.
