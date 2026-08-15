# SPEC-FDD-001 — Flow Defect Detection Core Boundary

Status: SEALED (ratified by this build — playbook_235).

## Amendment, 2026-07-29 — third domain reserved, not built; corrects a
## shared document's factual errors

A shared document proposed extending the water/electricity trilogy on
`fdd-core` with a third domain — oil & gas pipeline defect detection —
and asked whether this was consistent with what had already been built.
The core idea is sound and is recorded below (§"Domain-specific," new
Oil/Gas entry), but the document made several specific factual claims
about this repository that a direct check does not support, and they are
corrected here plainly rather than silently inherited by this spec:

- It described `fdd-core` itself as having a "barû/King-plot
  separation," a "prediction horizon," and a "Têrtu diagnosis" already
  built in. None of these are in `fdd-core` — this section's own
  "Not yet in `fdd-core`" paragraph (below, unchanged) already says the
  King-plot separation doesn't exist anywhere in this crate; a
  prediction horizon exists only in `massartu-core::Horizon` and
  `egd-engine::horizon`, domain-specific, not core; and "Têrtu" is a
  misspelling of `Terru` (`bahyway_algebra::enbilulu::diagnose`), which
  is Enbilulu-specific (water), not a `fdd-core` facility at all.
- It cited "BC-ENV-001 Rev. 2" as sealing WPDEngine's use of the
  Enbilulu Calculus, "PB-150" as its implementing playbook, and a
  Najaf/Karbala/Basra tribe mapping as WPDEngine's real geography. None
  of these exist: `docs/13_changelog/BC-ENV-001_Enbilulu_Calculus_2026-07-07.md` has
  no "Rev. 2" and its own status is `SEALED (design) — build QUEUED`,
  not implemented; no `playbook_150*` file exists anywhere in this
  repo (`bahyway_algebra::enbilulu`'s own module header says so
  directly); and WPDEngine's real geography is `BaghdadSector`
  (`GreenZone, AlKadhimiya, SadrCity, Karrada, Rashid, AlJadria,
  AlMansour` — Baghdad neighbourhoods), not Najaf/Karbala/Basra.
- It proposed **"Nergal"** as a naming candidate for the new domain's
  calculus, flagging it only as "needs care." Nergal is not free to
  propose — it already collides with the real, shipped
  `steward-lens::AlertSeverity::Nergal`. This exact collision was
  already caught once before, while drafting `docs/07_file_formats/SPEC-MAS-001.md`'s
  worked examples, and resolved there: the ecosystem already reserved
  **Ishum** for precisely this future gas-pipeline calculus (see that
  document's own Naming section, quoted under "Domain-specific" below).
  The document's alternative suggestion, "Isatu"/"Išātu," is confirmed
  free of any collision (checked against `docs/` and `workspace/`), but
  using it would create a naming redundancy against the name already
  reserved for this exact role, not fill a gap.
- It named "PB-160, Phase 2, BeeMDM's 50-zip proof" as the completion
  law gating any new domain engine's build. PB-160 itself
  (`playbook_160_tpl_001_section_e_corrected.yml`) is an unrelated,
  not-yet-run federated-state-scope naming law with nothing to do with
  BeeMDM. The real gate — Phase A (PB-90–98) → Phase B (PB-99–109) →
  `TESTING_PLAYBOOK_PHASE1` (Blocks A–F) → a full BeeMDM 50-zip ETL
  test, per `docs/13_changelog/RM-002_ADDENDUM_VERIFIED_2026-07-07.md` and
  `docs/13_changelog/BC-ENV-001_Enbilulu_Calculus_2026-07-07.md`'s identical
  "Governing law" line — is confirmed **not yet closed**: PB-99, the
  Phase B / `TESTING_PLAYBOOK_PHASE1` gate itself, is still status
  `[ ]` (not run) in `docs/16_runbooks/PLAYBOOK_EXECUTION_TRIAGE.md` as of this
  amendment. The gate still holds. This amendment therefore reserves
  the name and records the domain-neutral design; it builds no engine
  crate and mints no new playbook number, exactly as the shared
  document itself independently advised ("Draft the domain spec tablet
  whenever you like; the gate holds the rest").

None of the above changes anything already sealed above this amendment —
`fdd-core`'s real surface, WPDEngine, and EGDEngine are exactly as
described in the original 2026-07-25 note, untouched.

## Honest source note (2026-07-25)

An earlier draft of this spec described `fdd-core` as "extracted from
WPDEngine." That claim was checked against the real source and does not
hold:

- The real `wpd-engine` crate is a **spectral remote-sensing
  classifier** — `BaghdadSector`×7 heptagram geography, 12-band
  VNIR/SWIR/TIR signature matching (`oil_leak_signature`,
  `sewage_blockage_signature`, `water_leak_signature`), KAKI-keyed
  defect events, and repair routing/scheduling. It has no potential
  field, no per-node conservation residual, no Kirchhoff-analog
  anywhere in it.
- The real Enbilulu Calculus (`bahyway_algebra::enbilulu`) is a
  **5-factor weighted score** (weights 0.20/0.20/0.15/0.30/0.15)
  feeding TIAMAT band thresholds. `baru_residual` (kappa) is real and
  is the heaviest single factor, but it is one input to a weighted sum
  — not the output of solving a network. The module's own header
  flags that 3 of its 5 factors are still placeholders pending a
  source document not present in this repository.

`fdd-core` is therefore **new machinery**, not an extraction: a fresh
abstraction for domains where the physics genuinely IS a conservation
law solved over a graph (Kirchhoff's current law for an electrical
grid). It does not touch, and is not derived from, WPDEngine's sealed
source, and WPDEngine/Enbilulu are left completely untouched by this
work (BLK-1 lesson: no blind edits to sealed source).

One consequence: the "PB-173 (Enbilulu re-seat)" item referenced by
earlier drafts does not make sense as originally scoped — Enbilulu has
no potential field to re-seat onto a KCL-residual computer, being a
weighted score, not a network-solved model. If a physics-based
(as opposed to weighted-score) water defect layer is ever wanted, that
is new work built fresh on `fdd-core`, not a migration of Enbilulu.

## Domain-neutral (lives in `fdd-core`)

- Network graph model with DYNAMIC topology: edges carry an
  `in_service` flag (breaker/valve state is a state variable, not a
  constant).
- Conservation-law residual per node (Kirchhoff/mass — the law is
  injected by the domain via the `Potential` trait).
- Typed `FddError` for dimension mismatches (potential-vector length,
  out-of-range edge references) — computed as a `Result`, never a
  panic.
- Baru residual record (`BaruResidual`).
- Alert emission trait (`AlertSink`; domain names its alert class:
  water → Milu, electricity → Birqu).

Not yet in `fdd-core` (explicitly out of scope for this build):
King-plot systematic-vs-anomalous separation (currently `trend-core`
is a plain least-squares fit with no outlier/bad-data classification —
real for a first slice, not a finished bad-data detector), and a
power-flow/state-estimation solver (`Potential::edge_flow` requires
the potential field `phi` as an input; nothing in this crate computes
`phi` from topology + injections). Both are load-bearing gaps for a
production deployment, not polish, and are tracked as follow-on work,
not silently implied to be present.

## Domain-specific (lives in each engine)

- **WPDEngine**: real, sealed, spectral/weighted-score architecture as
  described above. Untouched by this work.
- **EGDEngine**: **Gibil Calculus** — `Phi_Gibil` COMPLEX potential
  (phasors, reactive power, phase angle), admittance-based KCL
  residuals, Birqu alerts, fast-cadence budget in the MILLISECOND
  class. See `docs/07_file_formats/GL-EGD-001.md`.
- **Oil/Gas pipeline engine (RESERVED, not built — 2026-07-29)**:
  **Ishum Calculus**. The domain physics is genuinely a `fdd-core`
  fit — a `Potential<Q>` implementation for hydraulic head (liquid,
  close cousin of the water head-loss physics WPDEngine's spectral
  approach doesn't itself model) or, for compressible gas flow, a
  distinct `Potential` over pressure-squared (the one genuinely new
  physics this domain needs beyond what EGDEngine's Gibil or a
  liquid-hydraulic potential already cover) — conserved at each node
  exactly like Kirchhoff's current law or mass conservation, with
  pipeline segment `in_service` state (valve/isolation status) as the
  same kind of dynamic-topology state variable `Edge.in_service`
  already is for EGDEngine's breakers. Real, on-theme fits for
  `fdd-core`'s existing shape: negative-pressure-wave leak detection is
  a wave-approaching-a-horizon problem; corrosion growth is a
  `trend-core`-class horizon metric exactly like EGDEngine's own
  weeks-until-alert pattern; illegal tapping is anomaly separation —
  the still-missing King-plot gap this document's "Not yet in
  `fdd-core`" paragraph already discloses, so tapping detection is
  blocked on the same real, load-bearing gap electricity bad-data
  detection is, not a new one. No crate exists yet
  (`crates/*-engine` has no oil/gas member); this is a reservation and
  a design note, gated behind the completion law above, not an
  implementation.

## Naming (NL-001)

- Engine names WPDEngine/EGDEngine: descriptive acronym pattern
  (established precedent, not god-named).
- Calculi are god-named. The original proposal for the electricity
  calculus was "Girra" — checked against the real workspace and
  rejected: `girra-engine` already owns Girra for the ecosystem's
  general sovereign monitoring dashboard, and `nusku-engine` /
  `nusku-score` / `nusku-fuzzy` already own Nusku for body-scan
  security classification (itself the reason `girra-engine` was
  renamed away from Nusku in the first place). The electricity
  calculus is named **Gibil** instead — smith-fire, the refining
  flame that shapes metal — unused anywhere in the workspace at the
  time of this build.
- Alert classes are Akkadian phenomena: Milu (flood), Birqu
  (lightning) — unchanged.
- Both Girra and Gibil are candidates for the Marduk fifty-names
  namespace (BC-MRD-001 §2); Enbilulu is the sealed precedent for a
  domain engine's calculus being retroactively absorbed into that
  namespace without requiring the engine to be rebuilt on Nabû
  Calculus machinery. This document takes no position on which name,
  if either, is eventually claimed there — that is MardukEngine's
  ledger to keep, not this spec's.
- The oil/gas calculus is **Ishum**, not a name proposed fresh here:
  `docs/07_file_formats/SPEC-MAS-001.md`'s own Naming section (amendment A2) already
  reserved it, after catching and rejecting the same "Nergal" collision
  a later document independently proposed again: *"an earlier draft's
  illustrative gas-pipeline calculus was named 'Nergal,' which collides
  with the real, shipped `steward-lens::AlertSeverity::Nergal`... the
  future gas-pipeline calculus is **Ishum**, Erra's restraining
  companion in the Erra Epic... Confirmed unused anywhere in the
  workspace... at the time of this build."* This spec inherits that
  reservation rather than re-deciding it — one name, reserved once, not
  re-litigated per document.
