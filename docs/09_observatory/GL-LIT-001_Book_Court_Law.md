# GL-LIT-001 — The Book Court Law
## Literature Patterns as Minted Templates with Provenance

**Ecosystem:** BahyWay.Ecosystem v4.0
**Domain:** GL (Global Law) — Literature / Pattern Provenance / Nasaru Instrument family
**Status:** SEALED — landed by PB-325 as `crates/bookcourt`, 5/5 tests passing (FUZZY-at-mint invariant, the gate guard, the annotation clause, cross-corpus unification, lattice rank). Note (§9): the PB-323 numbering collision the tablet flagged is a separate housekeeping item for CSR-08, unrelated to this crate's own PB-325 assignment, and remains open.
**Author:** DUB.SAR 𒁾
**Related tablets:** GL-ONT-001 (OntoGraph), GL-TPL-001 (Pattern Minting, two-witness), GL-DST-001 (Theater-as-Workbench: stage never truth), GL-STY-001 (StoryEngine Journal), NL-001 §6b.

---

## 1. Purpose

The Book Court is the Nasaru instrument by which published mathematics — books, papers, monographs — is read as a *structured corpus*, staged for comprehension, latticed for relations, and mined for patterns that may enrich ecosystem services. The court delivers the **full picture** of a work: which results consume which assumptions, which machinery is shared, what the whole work secretly stands on (the invariant intent at ⊤).

The court judges **structure**, never **correctness**. It is an instrument of comprehension and provenance, not a referee of theorems.

## 2. The Five Rites

**Rite I — Stage.** The object of the chapter is staged in the Membrane Court grammar: the mathematical object itself, live, camera-navigable (Bird Eye / Ground / ṬĀLUKU / Ṣabātu). Geometric Algebra objects (rotors, blades, meets, conformal points) are stageable by nature and form the founding corpus.

**Rite II — Lattice.** The work's results become an FCA formal context: objects = definitions, lemmas, theorems, algorithms; attributes = assumptions, prior results, and tools each consumes. The closure yields the derivation lattice — the flat instrument beside the stage, which never bends (GL-DST-001). Clicking a concept lights what it governs on the stage.

**Rite III — Pick.** The researcher brackets a result Ṣabātu-style. Its journal line reports its intent (assumptions consumed), its consumers, its lattice rank in its home work, and its cross-corpus matches (Clause 5).

**Rite IV — Mint with Provenance.** A picked pattern is minted under GL-TPL-001 as a template particle:
- Optional EAV carries: `lit.source` (work, edition), `lit.locus` (chapter, theorem number, page), `lit.intent` (assumption list), `lit.rank_home` (lattice rank in home work), `lit.notation` (notation family), `lit.witness_1` (the work's own statement), `lit.witness_2` (empty at mint — see Rite V).
- The template's state at mint is **FUZZY**. A literature pattern is never born GOLDEN.

**Rite V — Gate Before Service.** No minted literature pattern may be consumed by any ecosystem service, calculus, or engine until its second witness exists:
- **Full witness:** Lean4/Z3 proof of the specific identity relied upon, at Gate G4 (design-time only), OR
- **Working witness:** a pure-Rust implementation property-tested against independently known cases, sealed by numbered playbook.
Upon second witness, `lit.witness_2` is filled, the template may be promoted to GOLDEN, and only then may a service ORBIT it. "Approved" means approved by the ladder, never by the visualization.

## 3. The Two-Witness Clause (restated for literature)

The book's statement is one witness. Independent verification is the second. One witness mints; two witnesses serve. This is GL-TPL-001 applied to mathematics itself.

## 4. The Annotation Clause

Extraction from LaTeX/PDF follows the soil doctrine: the machine proposes annotations from parseable structure (theorem numbering, explicit references, citation keys); the human researcher confirms semantic ones (implicit assumptions). No single-witness annotation enters the formal context.

## 5. The Cross-Corpus Clause

All courted works share one growing formal context per domain. Concepts whose intents match across works under GL-TPL-001 two-witness matching (MinHash at scale) are surfaced as **unification candidates** — the same theorem in different notational costume. A confirmed unification is a Nebuchadnezzar-class pattern over mathematics and a candidate Apkallu registry entry.

## 6. The Structure-Only Clause (intellectual property)

The court stores **structure and pointers, never prose**: theorem identifiers, loci, assumption lists, dependency edges, and the researcher's own annotations. The text of the work is never copied into the golden store. The instrument sends the researcher *to* the book; it does not replace the book.

## 7. The Humility Clause

Every court verdict is structural: "consumed / not consumed," "shared intent," "orphan," "invariant intent." The court never emits "correct" or "incorrect." Where extraction confidence is partial, the verdict carries its ε.

## 8. Founding Corpus

Geometric Algebra: first rehearsal on one chapter of Dorst–Fontijne–Mann (*Geometric Algebra for Computer Science*), chosen for stageable objects (meet, join, projection) with direct enrichment paths into Nasaru. Second corpus candidate: Błaszczyszyn–Haenggi–Keeler–Mukherjee (the Sixth Court's home work), whose derivation lattice is already rehearsed.

## 9. Playbook

- **PB-325** — Book Court crate scaffold (`bookcourt`) inside the Nasaru workspace: literature formal context with provenance EAV fields, lattice reuse from the `ontograph` crate, FUZZY-at-mint rule, gate guard (`assert_served` fails without witness_2), law tests.

*(Numbering note for the Architect: PB-323 has been claimed twice in recent sessions — once for the LamassuEngine↔OntoGraph bridge in GL-ONT-001 §9, once for the Anu governor rename. One of the two must move to PB-326; the collision is flagged here for CSR-08 adjudication rather than silently resolved.)*

## 10. Seal

```
Sealed by: DUB.SAR 𒁾 (Bahaa Fadam), via explicit chat confirmation (CSR-08)
Date:      2026-08-27
AkkadianSeal (Ed25519): PENDING — no real signing infrastructure wired
                        yet (no Sargon/Gilgamesh passport ceremony run
                        against this tablet). The chat confirmation above
                        is the Architect's real CSR-08 act; the
                        cryptographic seal is separate, real follow-on
                        work.
```
