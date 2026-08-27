# GL-BND-001 · Tablet of the Band (draft, unsealed)
Proposed 2026-08-26 · DUB.SAR 𒁾
Depends on: GL-AGT-001 §3 (no fake) · §4 (no hidden) · §5 (never above the operator) · §6 (duplication refused)
· §9 (the judge is not the player) · §11 (everything is a decree) · GL-VIZ-002 §8 (the render grammar)
Governs: the Watcher_Scanner, the scoring of a stakeholder's templates against real data, and the band it proposes

---

## §0 Why a band at all
A stakeholder who mints templates deserves to know whether their work **holds against real orbits**, and the estate
deserves a way to tell a contribution from a claim. A band is a **summary of evidence**, never a reward for effort and
never a title. It is proposed by the scanner, signed by a steward, and it can be **taken back**.

## §1 A template is scored against real orbits, never against taste
Five measures, each computed by an instrument and each `MEASURED`:

| measure | weight | what it asks |
|---------|--------|--------------|
| `FIDELITY` | 0.30 | applied to a real orbit, can the counts be recovered from what it renders? A pattern that loses the measure it claims to show is worthless however handsome |
| `DISCRIMINATION` | 0.25 | shown two orbits that genuinely differ — healthy and decaying — does it look different? A pattern that renders every estate the same reveals nothing |
| `COVERAGE` | 0.15 | on what fraction of real orbits are its units and strata defined? |
| `HONESTY` | 0.20 | does it render confidence in the fill and draw the unknown (GL-VIZ-002 §8, §9)? |
| `ORIGINALITY` | 0.10 | distance from the nearest existing template |

`USE` — how often stewards held it, and whether it led to a decree — is **reported and carries weight zero**. It is
the most tempting measure and the easiest to farm; the scanner publishes it as context, never as credit.

## §2 A duplicate scores zero
A template whose canonical digest already exists scores **0.00**, whatever its other measures. Renaming a shape is not
work (GL-AGT-001 §6), and no arrangement of the remaining measures may lift it.

## §3 Quantity is not merit
A stakeholder's band is computed from the **median** of their templates' scores, never the sum, the maximum or the
count. Minting fifty templates to have one score well **lowers** the median and lowers the band. Each refused
duplicate subtracts `0.05` from the median before the band is read.

> A portfolio is judged by its middle, not by its best. This is deliberate: the estate is not improved by volume,
> and a stakeholder cannot approve themselves by producing more.

## §4 The ladder
| band | median | additional requirement |
|------|--------|------------------------|
| `WHITE` 白帯 | ≥ 0.00 | a first template exists |
| `YELLOW` 黄帯 | ≥ 0.45 | — |
| `ORANGE` 橙帯 | ≥ 0.55 | — |
| `GREEN` 緑帯 | ≥ 0.65 | no exact duplicate in the portfolio |
| `BLUE` 青帯 | ≥ 0.72 | at least three templates scored |
| `BROWN` 茶帯 | ≥ 0.80 | at least one template survives a second epoch unchanged |
| `BLACK` 黒帯 | ≥ 0.88 | **at least one template adopted by an engine other than the author's own**, and no refusal in the portfolio |

The black band's extra condition is the only honest test of value: **someone else chose to use it.** A stakeholder
cannot reach it by their own hand, and no volume of self-declared excellence substitutes.

## §5 A band is worn, not owned
Every band is re-computed each epoch against current data. A template that fails on orbits it did not meet before
lowers the median and **may lower the band**. Demotion is recorded with its reason, in the same words as a promotion.
No band is permanent, and none is retroactive: a band earned is a statement about the estate as it stood that epoch.

## §6 The scanner proposes; a steward signs
The scanner emits a **band decree** with every measure and its citation attached. A steward signs it. The scanner may
not award, may not amend a weight, and may not add or remove a measure — those live outside it and are sealed
(GL-AGT-001 §5, §9).

## §7 The scores are published whole
Every band shows the five measures that produced it, the `USE` figure it did not count, the refusals subtracted, and
**what the stakeholder must do to rise** — named as a specific measure, never as encouragement. A band with no visible
arithmetic is a title, and titles are what this tablet exists to prevent.

## §9 The dan ranks
Above the black band the ladder continues, and each step asks the same thing in a larger form: **someone else chose
to use the work, again, and it held.**

| rank | median | distinct engines that adopted it | templates surviving |
|------|--------|----------------------------------|---------------------|
| `BLACK` 黒帯 初段 | ≥ 0.88 | 1 | 1 through 2 epochs |
| `BLACK+1` 弐段 | ≥ 0.88 | 2 | 2 through 3 epochs |
| `BLACK+n` | ≥ 0.88 | n+1 | n+1 through n+2 epochs |
| `BLACK+11` 十二段 · **Master of Masters** | ≥ 0.88 | 12 | 12 through 13 epochs |

The **engines must be distinct**, and an engine the stakeholder owns or maintains does not count. This is deliberate:
a circle of colleagues cannot raise one another, because the same engine adopting twice adds nothing. Master of
Masters is not designed to be reachable in a year; it is designed to mean something when it is.

## §10 The reward is a discount, and it follows the band
A band lowers the **daily licence fee** by a fixed percentage read from a **sealed price table**:

| band | discount |
|------|----------|
| WHITE | 0% |
| YELLOW | 1% · ORANGE 2% · GREEN 3% · BLUE 5% · BROWN 8% |
| BLACK 初段 | 12% |
| each dan above | +1% |
| BLACK+11 | 23% |

**Ceiling 25%.** The discount is a property of the band **currently held**, never a sum of past awards: a stakeholder
cannot accumulate reductions by minting more, only by holding a higher band. This is §3 carried into the ledger.

## §11 The discount touches the licence, never the floor
It reduces the **licence component only**. Metered cost — compute, storage, egress — is never discounted, and the
total may never fall below the **marginal cost floor** declared in the price table. A reward that made the estate
pay for its own use would not be a reward; it would be a leak.

## §12 A demotion is never a surprise charge
Bands are re-computed every epoch (§5). When a band falls:
- the lower discount applies **from the next billing period**, never retroactively;
- **nothing is ever back-billed** — a discount already granted stands;
- the stakeholder is told **which measure fell and by how much**, in the same words as a promotion (§7).

A reward that could become a debt would make every stakeholder afraid to publish, and fear is the opposite of what
this tablet is for.

## §13 The scanner does not price
The price table is **sealed and read-only** to the scanner, exactly as the ladder and the weights are (§6,
GL-AGT-001 §5, §9). The scanner proposes a band; the table maps band to discount; a steward signs; billing applies it.
No component that earns a discount may also set one.

## §14 The reward is published with its arithmetic
Every discount shows the band, the measures that produced it, the adoptions that justified the dan, the ceiling, and
the floor. A reduction whose reason cannot be read is a favour, and favours are what §0 exists to prevent.

## §8 Amendment
Amendments `GL-BND-001-A1…` require a fresh CSR-08 rite.
**§2, §3, §4's black-band condition, §9's distinct-engine rule, §11's floor and §12's no-back-billing may not be
amended** — a duplicate scoring zero, merit measured at the median, value proven by another's adoption, engines that
must be distinct, a floor the estate cannot be discounted below, and a reward that can never become a debt.
