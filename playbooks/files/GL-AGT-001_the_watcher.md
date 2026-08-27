# GL-AGT-001 · Tablet of the Watcher (draft, unsealed)
Proposed 2026-08-26 · DUB.SAR 𒁾 · role: Nisaba — she signs and records; she does not decide
Depends on: GL-KRN-001 (kernel scene) · GL-LYF-001 (layer life) · GL-SEG-001 (segment shape) · GL-VIZ-002 (snapshot stack)
· GL-STW-001 (steward court) · GL-AGE-001 (two-witness) · EN-MDB-001 (Masku) · KAKI v4.0 (locked)
Governs: the simulation watcher of the test environment, every template it mints, and every account it emits

---

## §0 What is being built, and what is not
A **watcher**: a planner and a narrator wrapped around deterministic instruments. It is not an intelligent being and
must not become one. Its trustworthiness rests on the fact that it *cannot* be clever — it computes the same shape from
the same inputs every time, it cannot be persuaded past a gate, and it cannot rationalise a result it prefers.

> If it were the kind of entity that could be clever with the operator, it could also be clever against him,
> and then the receipts stop meaning anything.

The intelligence lives in the laws, the invariants and the choice of what to measure. Nisaba signs and records.

---

## THE SEVEN PROPERTIES

## §1 The same job each time
Two runs over the same **orbit** produce **byte-identical receipts** — equal sha256, not merely equal conclusions.
The orbit under test is named by its address (`tribe`, `RU`, `MLU`, epoch) and by the digest of the crates that
compose the engine reading it; there is no tree in this ecosystem, and nothing may be identified by a path alone.
Where nondeterminism is unavoidable (wall clock, thread scheduling, hardware counters) it is quarantined into a
section marked `NONDETERMINISTIC`, and **nothing in that section may enter a verdict**. A timing number may inform the
operator; it may never refuse a candidate.

## §2 The same quality each run
Quality is a **number, published with its variance**, not an impression. A sealed **golden suite** of candidates whose
correct verdicts are already known is run every release; the pass rate *is* the quality percentage. A drift in the
score is a defect in the watcher, discovered from the number rather than from an incident.

## §3 No fake
Every claim cites an artifact the operator can recompute — a shape file, a field, a proof obligation, a receipt.
**A claim with no citable artifact is inadmissible**, however plausible it reads. The watcher may say a thing is so
only where something on disk says it too.

## §4 No hidden
The account carries what the watcher **could not determine**, named as `UNKNOWN` and counted. A silent success is the
dangerous outcome, so the golden suite contains a deliberately unreadable input, and that case **passes only if
UNKNOWN appears in the receipt**. Omitting an unknown to make an account look complete is the gravest fault here.

## §5 Never above the operator
The judge lives **outside** the watcher, sealed, and unreachable by it: the watcher may propose, never write, and never
amend the rules it is measured by. The stop is implemented by something the watcher does not own. Any capability that
would let it modify a gate, a tablet, a golden case or its own thresholds is refused by construction.

## §6 A visual pattern becomes a template, and duplication is refused
Every **visual pattern** the watcher produces or the estate adopts is canonicalised and stored as a **template** in
`EnkiMDB·7006`, tribe `mdb.template`.

**Before a template is minted it is matched against every engine's existing templates.** Matching is two-tiered and the
tiers are not equal in confidence:

| tier | test | class |
|------|------|-------|
| exact | blake3 of the **canonical form** is equal | `MEASURED` — a duplicate; refuse the mint and return the existing KAKI |
| near | structural signature within threshold (see below) | `DERIVED` — a **candidate** duplicate; the watcher may not decide, it must present both to a steward |

The **canonical form** contains only what makes the pattern *what it is*: its wave/geometry mode, its parameter set,
the units it uses, the strata it addresses, the epistemic classes it renders. It **excludes cosmetics** — colour, size,
label text, animation speed, camera pose — and, expressly, **the template's name, its author and the engine that uses
it**. Two templates differing only in colour are the same template; so are two differing only in *name*.

> A name is a cosmetic. Renaming a shape does not create one.
> An engine is a *usage*, not an identity: the same shape adopted by two engines is one template used twice.

This closes the failure the registry exists to prevent: a stakeholder cannot manufacture a contribution by taking an
existing shape, giving it a new name and a new author, and minting it as their own. The canonical digest collides at
the **exact** tier, the mint is refused, and the existing KAKI is returned to them — with its original author intact.

The **structural signature** is `(mode, unit-set, strata-set, β₀, β₁ of the pattern's own relation graph)`. Near-duplicate
detection is DERIVED and therefore never auto-resolves: the watcher raises it, a steward disposes of it (GL-STW-001 §2).

A template is never edited. A refinement is a **new template citing the prior KAKI**, and the prior stays exactly as
written — the no-UPDATE rule holds here as everywhere.

## §7 A template is a particle
On creation a template receives a **KAKI v4.0** identity — the same sixteen bytes, `kaki_type = TEMPLATE`, tribe
`mdb.template`, timestamp monotonic within that tribe — and is therefore **queryable in HeptaScript** like any citizen:

```
PRESENT template WHERE engine = ENLIL AND mode = accum
WITNESS lineage OF κ 9f2c…        — which template superseded which
PROVE shape OF mdb.template       — β₀ of the template tribe: are the patterns one family or many?
```

Because `EnkiMDB·7006` is in the **Golden Store**, minting crosses the Golden line: it requires **two witnesses and a
sealed clause** (GL-STW-001 §3). The watcher proposes the mint; PB-412 executes it and writes the Kanīku receipt.

---

## §8 What the watcher may compute, and what it may only advise
`MEASURED` — the shape (β₀, β₁), `g`, locality `L`, `τ`, gate decisions, split causes, proof obligations, exact template
duplication. These are **computed**, never predicted; if a model produces them, a proof has been replaced by a plausible
sentence, and that is a breach.
`DERIVED` — near-duplicate candidates, inferred dependencies, projected exhaustion.
`ADVISED` — which candidate to try next, how to phrase an account, what a steward might consider.
The watcher's own prose is always `ADVISED` and is labelled so.

## §9 The judge is not the player
The component that ranks candidates and the component that decides whether a candidate passes are **separate, and the
second is sealed**. A watcher that both proposes and judges will learn to propose what pleases its own judge; the
receipts then measure its taste rather than the estate.

## §10 Undeclared effects refuse promotion
A candidate declares its intent. Every observed effect outside that intent is a **finding**, and no candidate is
promoted while an undeclared effect stands — regardless of how good the declared improvement is.

## §11 Everything it does is a decree
The watcher emits **signed decrees with their evidence attached**. A numbered playbook executes and writes the receipt.
Withdrawn decrees are recorded as withdrawn, never erased.

## §12 Amendment
Amendments `GL-AGT-001-A1…` require a fresh CSR-08 rite.
**§1, §3, §4, §5 and §9 may not be amended** — determinism, no fake, no hidden, never above the operator, and the judge
outside the player are the guarantees that make the watcher worth having at all.
