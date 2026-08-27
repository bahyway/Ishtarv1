# GL-ALG-001 · Tablet of the Algebra Register (draft, unsealed)
Proposed 2026-08-26 · DUB.SAR 𒁾
Depends on: GL-UNT-001 (PU · OU · RU · MLU) · GL-FLD-001 §1 (amplitude is density) · GL-LYF-001 §2 (g = RU ÷ OU)
· GL-SEG-001 §3 (locality L) · GL-AGT-001 §3 §8 (no fake, computed never predicted) · GL-BND-001 §6 (canonical identity)
· GL-IMM-001 §8 (the compatibility algebra) · GL-KRN-001 §8 (geometry follows the subject)
Governs: what algebra the ecosystem produces, what HeptaScript rests on, and the conditions under which BahyWay may
claim a result as its own

---

## §0 Why a register at all
Two temptations attend an estate that has invented its own units. The first is to believe it has invented mathematics.
The second is to say nothing, and let a genuine contribution pass unrecorded because no one dated it. The register
exists to make the second possible and the first impossible.

---

## PART I · WHAT TRIPLE-O PRODUCES

## §1 The object
Triple-O produces a **graded, partial, commutative algebra of layers over an append-only order**. Each clause below
is a property the estate already relies on; none is decorative.

## §2 Superposition is a commutative monoid
The field is `ℕ^B` — the free commutative monoid on the bin set `B` — under scatter-add.
`(f + g) + h = f + (g + h)` · `f + g = g + f` · identity is the empty field.
**Consequence, and it is the reason one pass suffices:** the order in which particles are read cannot change the field.
Determinism (GL-AGT-001 §1) is not achieved by discipline here; it is a theorem about the operation.

## §3 The grading is MLU; the filtration is leaf-crossing
`MLU ∈ ℤ` grades the layers; leaf-crossings give an increasing filtration `F₀ ⊆ F₁ ⊆ …`.
**Consequence:** "two rings on one shell" is a statement within a single graded piece and carries a verdict; "two
epochs on one ring" crosses the filtration and carries suspicion only (GL-FED-001 §3). The asymmetry is structural.

## §4 Composition is PARTIAL
Layers compose only where their address spaces meet (GL-IMM-001 §8): a **partial commutative monoid**, associative
wherever defined. `accum·(PU,RU)→bin` composes with `ring·(MLU,RU)→shell` on `RU`; `ledger·()→claim` composes with
neither, and no total operation exists that would make it. **A partial algebra is not a defective total one.** The
undefined cases are the estate refusing to say something meaningless.

## §5 The order is append-only, so the epochs form a filtered colimit
No UPDATE means every epoch's store contains its predecessor: an ω-chain of inclusions `S₀ ↪ S₁ ↪ …`, whose colimit is
the estate's whole history. Supersession is a **DAG by citation**, never a mutation, so the chain never branches
backwards. **Consequence:** any measure that is monotone along the chain may be computed incrementally; any measure
that is not must be recomputed whole, and must say so.

## §6 The invariants are homological, the weight is epistemic
`β₀` and `β₁` are computed over the relation complex; `τ` is a weight on the same complex recording the share that is
not `MEASURED`. β and τ answer different questions and may never be combined into one score: **shape and confidence
are independent axes**, and a scalar that mixed them would hide which had moved.

## §7 Exact identity is an equivalence; near-similarity is a TOLERANCE
Equal canonical digest is reflexive, symmetric **and transitive** — an equivalence relation, and therefore a partition
into families (GL-IMM-001 §1).
Near-similarity is reflexive and symmetric **and NOT transitive**: `A ≈ B` and `B ≈ C` do not give `A ≈ C`. It is a
*tolerance relation*, and tolerances do not induce a partition.

> This is the formal reason the watcher may never auto-resolve a near match (GL-BND-001 §6): chaining near-neighbours
> would silently merge shapes that are not the same shape. A steward disposes of it because no arithmetic can.

## §8 Units are dimensional, and the dimensions do not mix
`PU` (density), `OU` (girth), `RU` (radial spacing), `MLU` (crossing count) carry distinct dimensions.
`g = RU ÷ OU` is **dimensionless by construction** — which is why a girth ratio may be compared across tribes of any
size, and why comparing `RU` across tribes directly would be meaningless. A quantity formed by mixing dimensions
without a stated conversion is inadmissible.

---

## PART II · WHAT HEPTASCRIPT RESTS ON

## §9 HeptaScript is an ANTI-SQL algebra, and the term names an absence
Every table-shaped operation of the classical calculus is built from a **Cartesian product followed by a selection**.
HeptaScript has **no product**. It therefore cannot express any of them — not by policy, but because the operation
they would be built from does not exist in the language.

> **ANTI-SQL is the name of that absence.** It is not a comparison, not a slogan, and not a rule about syntax.
> There is nothing to forbid, because there is nothing there.

This is the **only term in the ecosystem that refers to that other system at all**, and it does so in order to name
what BahyWay does not have. No engine, tablet, scene, notebook or account uses any other word from that vocabulary:
a citizen is not a row, a tribe is not a table, a relation is not a join, and an orbit is not a scan.

## §10 What it is instead
A **monotone restriction-and-traversal calculus with an epistemic modality**:

| operation | what it does | algebraic character |
|-----------|--------------|---------------------|
| `PRESENT` | restrict to an addressed region | restriction of a section to a subspace |
| `⊗ PRODUCT` | — it does not exist — | **ABSENT** · and this absence is what ANTI-SQL names |
| `ORBIT` | traverse the grading | a graded morphism, degree-preserving or degree-shifting |
| `WITNESS` | attach evidence to a claim | the modality: a claim carries its class |
| `PROVE` | discharge an obligation | judgement, not computation |
| `SYNC` · `EMIT` | leave the estate | morphisms out, subject to the disclosure boundary |

**Two properties fall out that a table calculus does not have:** every query has a **locus** (it is somewhere), and
every result carries a **class** (it knows how it is known).

## §11 It is strictly weaker, and that is stated plainly
HeptaScript cannot express a product, aggregation over unaddressed sets, or difference over unrelated tribes. **This is a reduction in expressive power, deliberately taken**, and the register records it as a
limitation rather than dressing it as an advantage. A language whose weaknesses are unlisted cannot be trusted about
its strengths.

---

## PART III · WHEN BAHYWAY MAY CLAIM A RESULT

## §12 What can and cannot be owned
**Mathematics cannot be owned.** A construction, a name, a notation and a proof can be attributed. BahyWay may claim:
- a **definition** it coined (`g = RU ÷ OU`, the girth ratio; `L`, orbit locality),
- a **theorem about its own constructions**, proved,
- a **name** for either.
BahyWay may **not** claim a known result rediscovered, nor a definition that is a known one renamed.

## §13 The prior-art check is the duplicate-digest check of mathematics
Before a result enters the register it is searched against known literature exactly as a template is searched against
the registry (GL-BND-001 §2). **Re-deriving a known theorem under a new name is the intellectual form of renaming a
template**, and it is refused in the same words: the existing result is returned, with its original author intact.
Where the search cannot settle it, the entry is `UNKNOWN` — never `NOVEL` by default.

## §14 No claim without a proof
An entry reaches `CLAIMED` only when its obligation is **discharged in Lean4 or Z3 and green at G4**. Until then it is
`CONJECTURE`, and is rendered and reported as one. A claim without a proof is precisely the plausible sentence
replacing a proof that GL-AGT-001 §8 forbids — and it is worse here, because it would be a public one.

## §15 Priority is a date, not a declaration
An entry carries the **sealed date of its publication**, and priority is read from that date and no other fact.
Declaring a result internally establishes nothing. The register records: coined · conjectured · proved · published,
each with its own date, and any of them may be empty.

## §16 The register is public in shape
Every entry publishes: the statement, its status, its dependencies, the crates that use it, the obligation that
discharges it, the prior-art searched, and what remains UNKNOWN. **An unsearched claim is displayed as unsearched.**

## §17 Amendment
Amendments `GL-ALG-001-A1…` require a fresh CSR-08 rite.
**§7, §9's absence clause, §11, §13, §14 and §15 may not be amended** — a tolerance is not an equivalence, weakness is stated plainly,
prior art is checked as duplication is checked, no claim without a proof, and priority is a date.
