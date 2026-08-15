# GL-GOV-002 (candidate) — THE LAW OF EARNED ASSERTION
## The instrument's authority comes entirely from refusing to claim more than it can prove
### BahyWay.Ecosystem v4.0 · KEYSTONE binding GL-GOV-001 · GL-HS3-001/002 · GL-FOR-001 · GL-MEM-001 · GL-VIZ-003 · CSR-08 · Status: DRAFT — pending CSR-08 by DUB.SAR 𒁾

---

## 0 · The Law

**No component of BahyWay.Ecosystem — no query, no engine, no membrane, no lens,
no visualization, no stakeholder — may assert as fact anything it has not proven.
Every claim carries the evidence by which it was earned, or it is not a claim but
a proposal. The instrument's entire authority derives from this refusal. The
refusals form a single set: if any one slips, all lose their meaning, because a
reader can no longer trust that any fact in the system was earned.**

---

## 1 · The set of refusals (each is the same refusal)

| Site | The refusal | The failure it forbids |
|---|---|---|
| **GOLDEN store** | GOLDEN ⟹ verified fact | a prediction labelled GOLDEN |
| **AsalluhiEngine** | the membrane trills, but the cause must be PROVEN before prescribing | a trilling membrane that auto-diagnoses |
| **Submission (GL-GOV-001)** | the stakeholder proposes; the ranked authority admits with a witness | a freehand edge admitted on assertion |
| **Uncertainty (ε, GL-HS3-002)** | ε is carried, never averaged, never decisive | a comforting average that hides the catastrophic gap; ε used to decide silently |
| **The lens (ε as detector)** | the lens detects the hole; it never names what fills it | an Unknown cluster that auto-names its concept |
| **FCA discovery** | a relation is *derived* (computed) or *asserted* (witnessed), never blurred | a hand-drawn edge wearing the authority of computation |

**Why they are one law:** each site refuses to let *the detection of a
question* silently become *the assertion of an answer*. The day any single site
slips, the reader loses the guarantee that distinguishes this instrument from an
oracle — and every other refusal becomes unverifiable, because trust is
indivisible.

---

## 2 · The seduction clause (why the law targets the obvious, not the ambiguous)

The law is hardest to hold not when the answer is unclear, but when it is
**obvious**. Staring at 47 Unknown particles clustered in one Hepta region, the
pull to declare "it is obviously X" is enormous, and honouring the law will feel
like pedantry. **This is exactly the case the law exists for.** Unwitnessed
assertions do not slip in through ambiguity — ambiguity makes everyone cautious.
They slip in through seduction, when the answer feels so evident that proving it
seems unnecessary. Therefore: **the law binds most strictly precisely when the
conclusion feels most obvious.** "Obvious" is a signal to demand the witness, not
to waive it.

---

## 3 · Enforcement in HeptaScript (the query & analysis language)

The law is enforced not by convention but by **the grammar**: HeptaScript is
built so that an unproven assertion has *no expressible form*. You cannot write
the forbidden move.

**3.1 · No verb asserts fact without a witness.** Every fact-producing clause
requires a `WITNESS` term or it does not parse:
```
EMIT  <particle> AS GOLDEN            → REJECTED at parse: GOLDEN requires WITNESS
EMIT  <particle> AS GOLDEN WITNESS <proof>   → admitted; the proof is stored with it
```
There is no form of `EMIT ... AS GOLDEN` without a witness. The seductive move is
not resisted at runtime; it is unwritable.

**3.2 · Two kinds of output verb, never interchangeable.**
- `EMIT` — produces a *fact*; requires a witness; result tagged `derived`/`GOLDEN`.
- `PROPOSE` — produces a *proposal*; requires no proof but is tagged `asserted`
  and routed to the ranked authority (GL-GOV-001); may **never** be read by a
  clause expecting a fact.
A `PROPOSE` result assigned where a fact is required is a **type error**, not a
silent coercion.

**3.3 · The four-outcome honesty contract is mandatory on every ASK.** No query
returns a bare result. Every `ASK` resolves to `FACT` / `WEAK` / `GHOST` / `NONE`,
grounded in ε (GL-HS3-002): low-ε clean closure → FACT; rising ε → WEAK; no
closure → GHOST; absent → NONE. A query cannot return a GHOST *as if* it were a
FACT — the outcome tag is part of the return type.

**3.4 · ε is carried, never averaged, never decisive (GL-HS3-002).**
- ε aggregates by **envelope/worst-case only**. `AVG(ε)` over a population is a
  **forbidden form** — it does not parse. `MAX(ε)`, `ENVELOPE(ε)`, and
  `WORST(ε) WITH location` are the only aggregators, and they always report the
  location of the worst gap.
- ε may **guard** (`PROVE ... WITH ε < τ`) and **triage** (`SORT BY ε DESC`) but
  may never **decide**: no clause admits a particle to a tribe *because* its ε is
  low. Placement is by FCA closure (derived), not by ε (measured).

**3.5 · The lens detects, never names.**
```
SCAN Unknown WHERE ε > θ GROUP BY closure-neighborhood
   → returns: UNKNOWN clusters {count, Hepta-centroid, spread, shared-unclosed-attrs}
```
`SCAN Unknown` can only *return a located, counted proposal*. There is **no**
clause that turns a `SCAN Unknown` result into a named concept. To name it, the
proposal must pass `PROPOSE concept ... WITNESS ...` through the ranked authority.
The detector and the namer are separated by the grammar, not by discipline.

---

## 4 · Enforcement in BWVL / naṣāru (the symbol-icon visual language)

The law is enforced **in what the renderer is able to draw**. A visual that
asserted more than was proven would be a lie the eye trusts; the grammar forbids
it.

**4.1 · Provenance is a visual channel, always on.** Every rendered particle
carries its status in its appearance, by rule (the symbol grammar):
- **derived / GOLDEN** — solid, settled, full-colour.
- **asserted (witnessed)** — marked distinctly (e.g. a witnessed-ring); the eye
  can always tell an admitted relation from a computed one.
- **Unknown / high-ε** — rendered as **unsettled**: dim, shimmering, held-back —
  never as a confident point. Uncertainty is *seen*, not hidden.
There is no render path that draws an asserted or Unknown particle as though it
were derived. The visual cannot launder provenance.

**4.2 · The Unknown is drawn as located mass, never as fog.** A fragmentation
cluster renders as a **counted, positioned, dim region** (N particles, a Hepta
centroid), not as a blur or an omission. The gap in knowledge is shown *as a
thing you can point at*, with its mass — honest and actionable — never smoothed
away.

**4.3 · The lens never labels.** naṣāru may **highlight** an Unknown cluster and
show its count and location; it may **not** print a concept-name over it. A name
appears only after the ranked authority admits it with a witness — at which point
the cluster resolves from Unknown-mass into a named, derived concept, and its
appearance changes to match its earned status.

**4.4 · Never-averaged, visually.** A tribe's health/uncertainty is rendered by
its **worst** member, not its mean — the membrane trills to the catastrophic
particle, not to the soothing average. The eye is shown the worst case and where
it lives.

---

## 5 · Honest scope of this law (what it guarantees, what it doesn't)

- **Guaranteed by grammar** (cannot be written): §3.1 GOLDEN-without-witness,
  §3.2 fact/proposal type-separation, §3.4 `AVG(ε)`, §3.5 lens-names-concept.
  These are *unexpressible*, not merely discouraged.
- **Guaranteed by render path** (cannot be drawn): §4.1 provenance channel,
  §4.3 lens-labels. The renderer has no code path for the forbidden image.
- **Convention, needing review** (expressible but wrong): a stakeholder can still
  *propose* a bad edge, or *interpret* an Unknown cluster in their own head. The
  law cannot stop a human from believing "obviously X" — it can only stop the
  system from *recording* that belief as fact without a witness. The last line of
  defence is the ranked authority (GL-GOV-001), not the grammar. This limit is
  stated, not hidden.

---

## 6 · Codex compliance & placement
- **A-1 zero new mathematics:** composes existing refusals (GOLDEN scope,
  AsalluhiEngine PROVE, GL-GOV-001 submission, ε discipline, FCA
  derived/asserted) into one keystone; the new content is the *unifying law* and
  its *grammatical enforcement*.
- **PB:** PB-394–397 (below the tablet index; wire the grammar rejections, the
  type-separation, the ε aggregators, and the render provenance channel).

## 7 · Open seals for CSR-08
The Law of Earned Assertion · the set-of-refusals table · the seduction clause
(bind strictest when obvious) · the HeptaScript grammatical enforcement
(§3.1–3.5) · the BWVL render enforcement (§4.1–4.4) · the honest scope statement
(§5) · PB-394–397.

*Recorded in the reign of Gudea 1.0. The instrument guards truth by making the
lie unwritable and the unproven undrawable — and by confessing, where a human's
belief cannot be reached by grammar, that the last witness must be a ranked
authority and not a feeling. The refusals are one wall; it stands or falls
together. Nothing sealed until DUB.SAR confirms under CSR-08.*
