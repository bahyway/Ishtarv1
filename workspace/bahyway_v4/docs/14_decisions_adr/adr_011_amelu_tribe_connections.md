# ADR-011 — Amelu: Tribe-to-Tribe Connection Particles

> **DubSar Help** | `Decisions > ADR-011` | Architecture Decision Record

<!--META
doc_kaki:
  concept_hash:   "0x00000000"
  tribe:          "DT-14"
  concept_type:   "0x02"
  epoch:          "2026-07-07"
  concept_depth:  235
  riksu_count:    5
  snapshot_epoch: "2026-07-07"

concept:          "Amelu — Tribe-to-Tribe Connection Particles"
summary:          "CrossTribe relationships split into discrete factual anchors (ZIKRU) and evolving tribe-to-tribe connections (PARZU, called Amelu), so tribe reorganization and smart-city relation density never force a CrossTribe-Kaki to be rewritten."
sovereign_laws:   ["§8.3 amended — Amelu Orbit evolves by EMIT, identity payload does not", "ADR-008 Decision 7 Op #7 clarified — governs effective-state storage only, not Amelu's Orbit"]

riksu_bindings:
  - target: "crosstribe_kaki.md"
    concept: "CrossTribe-Kaki mechanism"
    type: "PARENT"
  - target: "adr_003_kaki_sovereignty.md"
    concept: "KAKI byte layout, structural-facts-only rule"
    type: "PEER"
  - target: "adr_008_ooo_foundation_kaki_roles_forbidden_operations.md"
    concept: "Decision 7, Forbidden Operation #7 (Store CrossTribe effective state)"
    type: "PEER"
  - target: "tribe_birthing.md"
    concept: "PA-15 child-tribe emergence pattern"
    type: "PEER"
  - target: "anu_gate.md"
    concept: "cross-tribe authority conflict resolution"
    type: "PEER"

orbit_tags:       ["CrossTribe-KAKI", "Tribe Journal", "Amelu", "Nabu Calculus"]
rag_keywords:     ["AMELU", "TRIBE JOURNAL", "CROSSTRIBE", "PARZU", "ZIKRU", "COVARIANT DERIVATIVE", "PARALLEL TRANSPORT", "NABU CALCULUS", "CONNECTION"]
-->

**Status:** Accepted (Part A) / Planned, blocked on PB-MRD-01→03 (Part B)
**Date:** 2026-07-07
**Author:** Bahaa Fadam
**Amends:** `02_identity/crosstribe_kaki.md`; ADR-008 Decision 7, Forbidden
Operation #7 ("Store CrossTribe effective state")
**Related:** ADR-003 (KAKI Sovereignty), ADR-008 (OOO Forbidden Operations), `tribe_birthing.md` (PA-15), `anu_gate.md`, `BC-MRD-001`/`GL-MRD-001` (MardukEngine / Nabû Calculus, external planning docs)

---

## Context

Two problems surface once CrossTribe-Kaki is used at real scale, neither of
which the current design (`KAKI_v4.0.1_canonical.pdf` §6.3, `crosstribe_kaki.md`)
resolves:

**1. Tribe reorganization invalidates a supposedly-immutable link.** A
CrossTribe-Kaki's mandatory EAV attribute `tribes_id` is specified as "the
vector of identities of the linked tribes and anchor particles... persisted...
never deleted, never recomputed" (canonical §6.3), locked by Forbidden
Operation #9. Tribe *identity* is not actually permanent — `tribe_birthing.md`
already documents that a child Tribe can be born from a parent's outer-rim
particles (PA-15). Nothing analogous exists for CrossTribe-Kaki: if `tribes_id`
bakes in a tribe's identity at link-creation time and a tribe is later renamed,
split, or merged, the link becomes stale, and the system forbids correcting it.

**2. Dense relation counts produce an unreadable, and structurally wrong,
graph.** A Smart City tribe (e.g. a Baghdad utility or population registry)
can have thousands of real cross-tribe relationships to particles sitting at
different orbit radii, states, and gate stages. Representing each as a
discrete CrossTribe-Kaki edge is what `crosstribe_kaki.md` implies ("basis-
transformation matrix P between two Tribe Jordan Blocks") but what
`idu-prober/src/crosstribe.rs` actually ships (`compose_n_anchors`) is a
small discrete state lookup, not a matrix or a field — the doc's own proposed
fix was never built. At smart-city density, a discrete-edge model becomes an
unreadable simplicial complex in DubSar IDE and is also the wrong
mathematical object: the relationship between two Tribes is a property of
their orbit *frames*, not a property of every individual particle pair
within them.

## Decision

CrossTribe-Kaki relationships split into two kinds, distinguished by the
existing `kaki_role` byte (κ[7]) — no new KAKI field is introduced:

| `kaki_role` | Meaning | Cardinality | Content |
|---|---|---|---|
| **ZIKRU** (0x02) | A discrete factual link between two specific anchor particles (citizen ↔ grave, document ↔ record) | One per fact, unbounded but sparse | `tribes_id` = anchor-Kaki pointers only. Permanent, no Orbit needed. |
| **PARZU** (0x03) — **an "Amelu"** | A relational law between two Tribes as a whole | One per Tribe-pair that has a real relationship — bounded by tribe-pair count, not particle-pair count | `tribes_id` = Tribe Journal pointers (§1 below). Substantive content lives in the Amelu's own evolving Orbit (§3). |

### §1 — The Tribe Journal (new)

`tribe_id` (κ[4..5]) becomes a stable *pointer* into a new append-only Tribe
Journal, never a self-describing name. The Journal records Tribe lifecycle
events — birth, rename, split (parent → child per PA-15), merge, deprecate —
as an Event-Kaki-style stream against the Tribe's own identity. A Tribe's
*current* name, parent, children, and status is a StoryEngine-style
projection over this Journal, exactly as a particle's current state is a
projection over its own Event-Kaki history (canonical §3.3). `tribe_id`
values are never reused (mirrors Rule II, §2.2).

This directly formalizes what `tribe_birthing.md` already asserts without
mechanism ("Their KAKIs do not change. Their storytelling journals continue
unbroken.") — κ[4..5] is a *birth-tribe* / partition fact, permanent by
construction; a particle's operational tribe membership after a split is
recoverable by projecting the Tribe Journal, not by mutating κ[4..5].

### §2 — CrossTribe-Kaki identity payload is anchor-pointers-only

For both ZIKRU and PARZU roles, `tribes_id` never stores a tribe's name or a
snapshot of its structure — only stable pointers (anchor Kakis, and/or Tribe
Journal entries for PARZU/Amelu). `KAKI_v4.0.1_canonical.pdf`'s Forbidden
Operation #9 ("Once linked, always linked" — that document's own numbering,
distinct from ADR-008's 17-item list) continues to apply to this identity
payload exactly as written. This is what makes it safe to never delete or
recompute: a pointer
to a Tribe Journal entry does not go stale when the tribe reorganizes,
because "what the tribe currently is" is resolved by projection at read time,
not frozen into the pointer.

### §3 — Amelu defined

An **Amelu** is a CrossTribe-Kaki with `kaki_role = PARZU`, representing a
relational law between two Tribes. Per §7 of the canonical KAKI spec,
PARZU-role particles are already "logic, templates, axioms, or rules" with
their own Orbit — an Amelu is not a new primitive, it is this existing
category applied to inter-tribe geometry instead of intra-tribe validation
rules.

An Amelu is born once (Identity layer: these two Tribes have a real
relationship worth modeling) and never rewritten. Everything that evolves
about the relationship — recalibrated weights, newly detected correlation,
response to a reorg on either side — lives in the Amelu's **Orbit**, EMITted
as new Event-Kaki-style entries over time, current value always a projection,
never a stored snapshot. This is the resolution to the original "EAV Journal
Registration" proposal: it was never `tribes_id` that needed journal
semantics — it was content that `tribes_id` should never have held in the
first place.

## Part B — Planned, blocked on PB-MRD-01→03 (the Nabû spine)

The following requires MardukEngine's Nabû Calculus (`BC-MRD-001`,
`GL-MRD-001`) — the metric `g = diag(w₁…w₇)` and the covariant derivative ∇ —
none of which exists in the repository yet (no `marduk` crate; PB-MRD-01
crate scaffold is unstarted). This section specifies the *schema* and
*contract* now so Part A can be built without rework later; it does not
claim the computation exists.

### §4 — Amelu's Orbit: Connection Journal schema

Mandatory EAV attributes on an Amelu, EMITted whenever the relationship's
geometry changes:

| Attribute | Meaning |
|---|---|
| `metric_weights` | The 7-component diagonal weight vector (from `H(P)`) defining the Riemannian metric on the path between the two Tribes' orbit frames |
| `transport_coeffs` | Discrete parallel-transport coefficients carrying a separation vector from Tribe A's frame into Tribe B's frame across template/weight changes |
| `last_calibrated_at` | Timestamp of the EMIT that produced the current coefficients — an Amelu with a stale calibration is a detectable, queryable fact, not a silent error |

### §5 — Nabû-computed relatedness, probe-time only

"How related is particle P_a (Tribe A) to particle P_b (Tribe B)" is never
stored per particle-pair. It is computed at probe time by transporting P_a's
orbit-position vector along the Amelu's current `transport_coeffs` into
Tribe B's frame and comparing against P_b's actual position — the same
"compute at PROBE, never store" discipline §8.3 already applies to
Gold/Orange/Gray link state, generalized from a 3-value lookup to a
continuous covariant quantity.

This is a genuinely new application of Nabû Calculus, distinct from the four
domain calculi already planned in `GL-MRD-001` (Šazu/Addu/Suhrim/Namtila),
which compute a particle's drift from its *own* bound template, not its
relation to *another Tribe's* frame. It is not assigned a name from the
Fifty-Names namespace by this ADR — that naming decision, if wanted, belongs
to the Architect.

### §6 — DubSar Theater rendering

Render one connection arc per Amelu — bounded by Tribe-pair count — styled
as a field line between Tribe rings, consistent with the existing BIGRING
orbital rendering pattern (`bahyway-algebra::orbital`, wired in PR #16).
Individual particle relatedness (§5) shows as position/highlight along that
arc, never as a separately drawn simplicial edge. This is what eliminates
the spaghetti: O(Tribe-pairs) rendered objects, not O(particles²).

## Consequences

### Positive
- CrossTribe-Kaki identity (`tribes_id`) is safe to leave genuinely immutable
  forever, because it was never asked to hold anything that changes.
- Tribe reorganization (rename, split, merge) never touches an existing
  CrossTribe-Kaki or Amelu — it is entirely a Tribe Journal event.
- Smart-city relation density is handled by a bounded number of Amelu
  particles plus probe-time computation, not by an unbounded, ever-growing
  edge list.
- Zero new KAKI primitives — reuses `kaki_role`, Identity/Orbit, and the
  existing Event-sourcing/Journal pattern three times (particle, Tribe,
  Amelu) instead of inventing a fourth mechanism.

### Constraints introduced
- A Tribe Journal must exist and be consulted by any reader resolving "what
  Tribe is this particle currently in" — a new required projection, alongside
  the existing particle-state projection.
- Part B cannot be built, tested, or verified before the Nabû spine
  (PB-MRD-01→03) exists. Any CrossTribe relationship that genuinely needs
  geometric relatedness (rather than a bare factual link) is blocked until
  then.
- Deciding ZIKRU-vs-PARZU at CrossTribe-Kaki mint time becomes a required
  authoring judgment in DubSar IDE (or its authoring path) — minting a ZIKRU
  link for what is actually a dense systemic relationship reintroduces the
  spaghetti problem this ADR exists to avoid.

## Corrections to prior documents

1. **`crosstribe_kaki.md`** currently states the CrossTribe-Kaki "is never
   persisted as a physical record in EnkiDB... materialised on PROBE only."
   This conflates the link's *identity* (persisted forever, per canonical
   §6.3) with its *effective state/relatedness* (computed at probe time,
   never stored). The document should be corrected to state both halves
   explicitly, and its "basis-transformation matrix P" proposal should be
   superseded by §4–§5 of this ADR (Amelu's Orbit + Nabû-computed
   relatedness), since P/P⁻¹ was never implemented and the Nabû formulation
   is more general (a fixed matrix cannot represent per-particle orbit
   position the way covariant transport can).
2. **Cross-document citation error, corrected here:** an earlier draft of
   this ADR cited "Forbidden Operation #9" as an ADR-008 rule. It is not —
   ADR-008's own 17-item Decision 7 list has no item numbered 9 with this
   content; the "#9 / once linked, always linked" wording belongs to
   `KAKI_v4.0.1_canonical.pdf`'s separate Forbidden-Operations enumeration (a
   different document, different count, different numbering). ADR-008's
   nearest equivalent is **Decision 7, Operation #7** ("Store CrossTribe
   effective state"). ADR-008 Decision 7 (Op #7) and Decision 6 (the IDU
   Probing Rule) should be read with an explicit note: an Amelu's Orbit
   (§3–§4 of this ADR) evolving by EMIT is not a violation of Op #7 or of
   Operations #1/#2 (no DELETE/UPDATE) — it is ordinary Orbit mutation via
   Event-Kaki, identical to how any particle's EAV state evolves without
   touching its KAKI bytes. The two Forbidden-Operations lists (ADR-008's 17
   vs. the canonical PDF's own, overlapping but not identical, count) should
   themselves be reconciled into one canonical list by the Architect — this
   ADR does not attempt that reconciliation.
3. **Unresolved, out of scope for this ADR — and worse than previously
   noted:** ADR-003 and ADR-008 disagree with *each other*, not only with
   the later canonical PDF, and both are dated the same day. ADR-003
   ("Accepted — Extended 2026-06-05") reassigns κ[8..11] to `seq_counter`;
   ADR-008 ("Accepted" 2026-06-05), Decision 4's byte table, still lists
   κ[8..11] as plain "reserved" with no mention of `seq_counter`.
   `KAKI_v4.0.1_canonical.pdf` (2026-07-05) re-declares the same bytes as
   reserved-and-zeroed, agreeing with ADR-008's table but silently dropping
   ADR-003's decision without citing it. This ADR's design does not depend
   on κ[8..11] either way, but three documents now disagree about one byte
   range, and the conflict should be resolved by the Architect — ADR-003's
   `seq_counter` collision-safety argument (§ Decision 3) is substantive and
   should not be silently lost by omission.

## W5H2

| W | Answer |
|---|---|
| **Who** | `enkidb-kaki` (Tribe Journal + anchor-only `tribes_id`), `idu-prober` (probe-time relatedness computation), DubSar IDE (ZIKRU-vs-Amelu authoring judgment), DubSar Theater (rendering) |
| **What** | A Tribe Journal for tribe lifecycle; anchor/pointer-only CrossTribe-Kaki identity; Amelu — a PARZU-role CrossTribe-Kaki carrying an evolving, Nabû-computed connection Orbit |
| **When** | Part A: buildable now. Part B: blocked until PB-MRD-01→03 (Nabû spine) exist |
| **Where** | `crates/enkidb-kaki` (Tribe Journal, identity payload), `crates/idu-prober` (probe-time computation), new `marduk`/Nabû crate (Part B only) |
| **Why** | So CrossTribe-Kaki's immutability guarantee and Smart-City-scale relation density can both hold simultaneously, without forbidding legitimate tribe reorganization |
| **How** | Reuse of existing primitives only: `kaki_role` (ZIKRU vs PARZU), Identity+Orbit, and the Event-Kaki/Journal pattern applied to Tribes and to Amelu instead of only to ordinary particles |
| **How much** | One Tribe Journal; one Amelu per real Tribe-pair relationship (not per particle-pair); ZIKRU links remain one-per-fact as before |

## Sovereign Law Statement

> A CrossTribe-Kaki's identity is what it points at, never what those points
> currently mean. Tribes reorganize; Amelu do not rewrite themselves to
> track them — they project the Tribe Journal at read time, the same way
> every particle's current state is a projection over its own history, never
> a value stored twice.

---

*Bahaa Fadam — BahyWay.Ecosystem v4.0 | ADR-011, 2026-07-07*
