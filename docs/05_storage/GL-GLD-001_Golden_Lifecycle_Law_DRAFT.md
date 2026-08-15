# GL-GLD-001 (candidate) — THE GOLDEN LIFECYCLE LAW
## Golden = Truth = Fact = Golden Record · finality of essence, never of position · Gravity Slop is position-only drift · Golden State Debugging = EnkiDW Analyzing
### BahyWay.Ecosystem v4.0 · binds GL-VIZ-004 (ColourID Lifecycle) §5, GL-MEM-001 (Mašḫalu Elastic Membrane), GL-ONT-001 (OntoGraph state trichotomy), MANDATORY_VS_OPTIONAL_ATTRIBUTES.md `state` attribute · Status: DRAFT — pending CSR-08 by DUB.SAR 𒁾

---

## 0 · Why this tablet exists

Three earlier tablets each hold a fragment of this law without ever stating
it whole. `GL-ONT-001` defines the `GOLDEN/FUZZY/DEAD` Mandatory state
attribute topologically (persistent vs. short-lived vs. diagonal-clustered
Betti classes). `GL-VIZ-004` §5 governs *Aged/Decay* as a Steward-witnessed
ColourID event — a display-layer rule, correct as far as it goes, but silent
on what Aged/Decay actually **mean**. `GL-MEM-001` describes membranes
trilling and drifting under particle-gravity, correct as physics, but never
states whether that drift touches the particle's truth.

This tablet closes the gap the Architect named directly, mid-session
(2026-08-15): a Golden Particle's rendered position on the Membrane Field
can shift after it is saved into a Golden Store (EnkiDB, EnkiDW) — density,
neighbour pressure, and membrane tension move it — and that shift is not a
bug, not corruption, and not evidence the particle's essence changed. It is
the one thing GOLDEN was never a claim about.

---

## 1 · The Law

**GOLDEN is a claim about finality of essence, not permanence of position.
A Golden Particle's content — what it asserts as true — is sealed the
instant it reaches GOLDEN state and never transforms again. Its rendered
position, the Membrane it sits in, and the store that holds it (EnkiDB,
EnkiDW, EnkiMDB, EnkiDDB) may all change without that particle disintegrating,
decomposing, or losing authority. Location, time, and space are not truth;
they are where and when a truth is currently found. This is Triple-O stated
plainly: everything is a particle; what matters is how it is used.**

---

## 2 · Golden = Truth = Fact = Golden Record

BahyWay does not coin a new concept here — it names, precisely, the same
thing the MDM industry calls a **Golden Record**: the single, authoritative,
trusted version of a data entity, reconciled and sealed, that every
downstream consumer defers to instead of re-deriving. `GL-ONT-001`'s
`GOLDEN` state and the MDM industry's Golden Record are the same claim in
two vocabularies:

| MDM industry term | BahyWay term | What both assert |
|---|---|---|
| Golden Record | Golden Particle | This is the authoritative version — trust it, don't re-derive it |
| Survivorship / merge | GOLDEN transition (`GL-VIZ-004` §4) | Reconciliation happened once, bounded, witnessed |
| Record lineage | Birth Root Shade (`GL-VIZ-004` §2a) | Origin is immutable and always recoverable |
| Data steward override | Aged/Decay marking (`GL-VIZ-004` §5) | A human, not a drift, moves a record's status |

A Golden Particle can therefore relocate — between orbits, between stores,
between renders — and remain exactly as authoritative as the day it was
sealed, for the same reason a Golden Record in any MDM system does not stop
being golden because a data warehouse migrated it to a new table.

---

## 3 · Gravity Slop (sealed term)

**Gravity Slop** names the drift of a particle's *rendered position* —
where it sits on its Membrane, how far its orbit is pulled, how the wall
around it dents — under the real gravity of neighbouring particles and
membrane tension. It is independent of, and never a cause of, either:

- **ontological drift** — a change in the particle's W5H2/Mandatory facts
  (`GL-ONT-001` §2), or
- **topological drift** — a change in its Betti-class state trichotomy
  (`GL-ONT-001`'s `GOLDEN/FUZZY/DEAD`, `MANDATORY_VS_OPTIONAL_ATTRIBUTES.md`
  row `0x1A4B`).

Gravity Slop is real, physical, and already implemented — not a proposed
mechanic. The reference implementation (`shala-prototypes/batch3_streaming_
pu_ctg_pb310_320/shala_layered_organism_v2.html`, rehearsal-only, DRAFT)
computes a membrane radius as:

```
taut(v) = 1 + 0.05 * (v / VMAX)^2
dent   += 0.055 * gaussBump(du, v - vcn) * (1 + spring * 6)   // clamped to 0.10
r       = s.r * taut(v) * (1 - dent)
```

`s.r` is the membrane's rest radius — the particle's *sealed* position under
zero load. `taut(v)` and `dent` are load-responsive multipliers: faster
particles taut the membrane; attached/processing particles dent it inward.
**Neither term ever touches `s.r` itself, and neither term writes to any
EAV attribute.** Gravity Slop is a render-time deformation of the wall a
particle is currently pictured against — never a rewrite of what the
particle is. This is the formal guarantee the law in §1 rests on: the
formula itself has no path from position to essence.

**Clause 3.1 — Scope boundary.** Any future engine that reads `dent`,
`taut(v)`, or a particle's current Hepta Space coordinates and writes back
to a Mandatory or KAKI attribute is out of Gravity Slop's scope and needs
its own sealed law, under design-time review (Gate G4), before it ships.

---

## 4 · Aged / Decay — an epistemic gap, not corruption

**Aged** and **Decay** name the growing distance between a sealed fact and
current reality — never structural damage to the particle. A Golden
Particle asserting "this address is correct" does not corrode; the world
around it moves. `GL-VIZ-004` §5 already governs the mechanics correctly
(a Steward witnesses the marking, an Event-KAKI records it, the Birth Root
Shade is preserved) — this tablet supplies the meaning underneath that
mechanic:

- **GOLDEN** — the particle's content is trusted as current fact.
- **AGED** — the particle's content is still true as *sealed*, but a newer
  fact has since appeared and been witnessed (`GL-VIZ-004`'s alert
  trigger). The old version is not wrong; it is superseded.
- **DECAY** — the gap between the sealed fact and current reality has grown
  wide enough that the Steward has marked the particle as no longer
  reliable for current decisions, without ever un-sealing or rewriting it.

None of the three is a claim about the particle's structural integrity,
storage health, or position. All three are claims about **how far the
particle's truth has drifted from the world**, which is a fact *about the
world*, witnessed by a Steward — never a fact the particle discovers about
itself, and never automatic state-drift (`GL-VIZ-004` §5's own clause,
reaffirmed here).

---

## 5 · Golden State Debugging = EnkiDW Analyzing (the procedure, named)

When a Golden Particle's rendered position moves unexpectedly — before
anyone has determined *why* — the investigation that follows has a name:
**Golden State Debugging**, performed as **EnkiDW Analyzing**. It is the
standing procedure for the pre-investigation state described in §3: a
position or membrane shift with an as-yet-unknown cause.

**Procedure:**

1. **Rule out essence change first.** Query the particle's Mandatory EAV
   facts and KAKI (immutable) directly — if they are unchanged, the
   particle's truth never moved; only §3 applies.
2. **Attribute the position change to Gravity Slop mechanics.** Check
   neighbour density, membrane tension, and `dent`/`taut(v)` inputs at the
   time of the shift — this is expected, lawful behavior under §3, not a
   defect.
3. **If Mandatory facts or state (`GOLDEN/FUZZY/DEAD`) DID change,** this is
   no longer Golden State Debugging — escalate to the relevant sealed law
   (`GL-ONT-001` for topological reclassification, `GL-VIZ-004` for a
   Steward-witnessed Aged/Decay event) and record which one applied.
4. **Witness the finding.** Whatever the cause, the investigation's
   conclusion is written to the NĀRU journal — "position shift, essence
   unchanged, Gravity Slop" is itself a real, checkable finding, not a
   non-event.

EnkiDW (the analytical warehouse of the seven EnkiDB types) is named as the
home of this procedure because it is where cross-time, cross-orbit
comparison of a particle's history actually happens — the same store, the
same tool, every time a position shift needs explaining.

---

## 6 · The Golden lifecycle loop

**GOLD → AGED → DECAY → POSITION → LOCATION → TIME → GOLD**

Seven stages, seven dimensions of Hepta Space — not a coincidence this
ecosystem treats as needing reconciliation; see `00_codex_government_by_
sevens.md` for why "7" recurs independently across this ecosystem rather
than being derived once. Read left to right:

1. **GOLD** — sealed, trusted, terminal essence (§1, §2).
2. **AGED** — a newer fact has appeared; this version is superseded, not
   wrong (§4).
3. **DECAY** — the gap has grown wide enough to distrust for current
   decisions; still not rewritten (§4).
4. **POSITION** — where Gravity Slop is read and, if needed, debugged (§3,
   §5) — the particle's rendered coordinates on its Membrane.
5. **LOCATION** — which store (EnkiDB/EnkiDW/EnkiMDB/EnkiDDB) currently
   holds the particle — itself just another position, at ecosystem scale.
6. **TIME** — when the particle is being read, which determines how AGED or
   DECAYed it currently reads as, relative to newer facts.
7. **GOLD** — the loop closes: essence is unchanged throughout, so the
   particle a Steward re-seals (or a newer Golden Record supersedes) is
   still recognizably the same authoritative claim it always was, wearing
   a new position/location/time.

**Clause 6.1 — The loop is a reading order, not a state machine.** No
engine transitions a particle mechanically around this loop. §4 (Aged/Decay)
is Steward-witnessed; §3/§5 (Position) is render/debug-time only; Location
and Time are simply *where* and *when* the particle is currently queried.
The loop is this tablet's teaching device for holding all seven ideas in
one frame — not a new sealed state machine competing with `GL-ONT-001`'s
`GOLDEN/FUZZY/DEAD`.

---

## 7 · Codex compliance & placement

- **A-1 zero new mathematics:** composes `GL-ONT-001`'s state trichotomy,
  `GL-VIZ-004`'s Aged/Decay mechanic, `GL-MEM-001`'s membrane physics, and
  the already-implemented Gravity Slop formula. New = the epistemic
  reading (§2, §4) and the named procedure (§5).
- **A-4 cited:** `GL-ONT-001` · `GL-VIZ-004` · `GL-MEM-001` ·
  `MANDATORY_VS_OPTIONAL_ATTRIBUTES.md` · `shala_layered_organism_v2.html`
  (rehearsal-only reference implementation, not production code).
- **PB:** reserved, not yet assigned — this tablet seals the concept only;
  no runtime code changes.

## 8 · Seal

```
Sealed by: ______________________  (DUB.SAR 𒁾, CSR-08)
Date:      ______________________
AkkadianSeal (Ed25519): ______________________
```
