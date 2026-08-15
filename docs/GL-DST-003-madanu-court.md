# GL-DST-003 — Madanu Court Law
**Status:** SEALED (concept). Implementation queued behind PB-160.
**Stage:** MADANU sub-tab of the MARDUK surface, DubSar Theater.
**Serves:** GL-DST-002 (Tupsimati wizard), GL-VIZ-002
(Orbit Witness & Isolation), the data steward workflow.

## Clause 1 — The Name Is Spent
MADANU — the divine judge who stands in Marduk's own house
at Esagila — names the steward's judgment chamber, per
NL-001 as ratified by the Architect. Ishtaran (arbiter of
Der) returns to the naming reserve. Should the decision
workflow harden into its own component (steward roles,
decree routing, verdict audit), it becomes MadanuEngine,
and this court is its Theater surface.

## Clause 2 — The Append-Only Decree Law (CQRS)
A decree NEVER updates and NEVER deletes. It APPENDS.
- The court composes a sovereign HeptaScript SYNC decree
  (WHO steward, WHAT verdict, WHERE home, WHEN decreed,
  WHY note, HOWMANY particles) and seals it with the
  Ed25519 AkkadianSeal.
- Execute is the CSR-08 moment: the decree travels over
  HEPT to the WRITE node, and the ENGINES append the new
  state to each particle through a KISPU four-way commit.
- A particle's state history is immutable forever. The
  CURRENT state is a projection: the latest appended
  entry, folded in O(1) by the ENLIL ring indexes.
- StoryEngine's chronicle IS the ledger, not a copy of
  it. Every decree becomes a STEWARD DECREE event —
  decree id, verdict, WHY note, seal — inscribed into
  every touched particle's story.

## Clause 3 — Verdicts and Re-Run Semantics
Four verdicts: PROMOTE (advance one station), REWORK
(return one station), KILL (the KISPU rite — the particle
enters the memorial rim; the plate is a memorial), HOLD
(annotated wait; dwell continues, reason inscribed).
A REWORK does not erase the first journey — it appends a
second journey. A particle returned to INGEST re-runs the
whole pipeline, and a particle that ran three times
carries all three runs in its chronicle. Batch scope is
whatever the steward has witnessed: a house, an orbit, a
state cohort, a pattern cohort, the long-dwellers.

## Clause 4 — The GOLDEN Terminus
GOLDEN = trusted truth, and its residence is EnkiDB ·
port 7004, the core trusted store. PROVE is the gate into
it: particles at the PROVE station still reside in
EnkiQDB awaiting their certificate. EnkiDW, EnkiMDB, and
EnkiDDB hold downstream SERVED PROJECTIONS of golden
truth, never a second original.

## Clause 5 — The Apsu Convergence
The pipeline's visual completion state is APSU, per the
Architect's BIGRING prototype: when every LIVING particle
of the witnessed orbits holds GOLDEN as its latest
appended state (the dead rim stands apart as memorial),
the ORBIT 3D lens dissolves the station shells and
renders the golden Deep — free particles in the golden
palette gathered about the central sphere. Orbits exist
because particles are still becoming; when all truth is
trusted, the stations are no longer needed. The first
water is the final state. This is a stage-only rendering:
nothing is computed, nothing changes — the Theater merely
shows that the work is done.

## Authority Note
Inherits all standing law: stage-never-truth (GL-DST-001);
fourteen doors, one rite (GL-DST-002); witness scopes
(GL-VIZ-002); NINSUN/Namtila advisory pattern; CSR-08
Architect Sovereignty; the 1-billion-particles-under-
1-second runtime law; NL-001 orthography.

— Inscribed for DUB.SAR 𒁾, BahyWay.Ecosystem v4.0.
