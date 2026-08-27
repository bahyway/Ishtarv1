# GL-LBR-001-A1 — The Migration Chapter
## The Two Scribes Rite: Kīma Labīrīšu over Warehouse Migrations

**Ecosystem:** BahyWay.Ecosystem v4.0 — EnkiDW·7005 migration doctrine
**Amends:** GL-LBR-001 (Labīru) · consumes GL-ONT-001 (OntoGraph/FCA), GL-NSR-001-A2 (Inquest), GL-LIT-001 (Book Court annotation clause), Kidinnu Standard
**Status:** SEALED — landed by PB-339 as `crates/two-scribes`, 5/5 tests passing (L32-L36).
**Author:** DUB.SAR 𒁾

---

## 1. The Inversion

Migration validation fails when it verifies the **scribe's hand** (unit tests over thousands of new procedures, functions, triggers) instead of the **tablet produced**. This chapter seals the inversion: the legacy system is the labīru; equivalence is judged by confrontation of outputs on sealed inputs. Code is read only where the confrontation points.

## 2. The Four Rites

**Rite I — Seal the Input.** A confrontation is lawful only on a **frozen, Kidinnu-sealed input snapshot** (the input-labīru). "Both ran on the same data" must be a verifiable seal, never an assertion — unsealed comparisons are inadmissible.

**Rite II — The Two Scribes.** Legacy and new run on the sealed input. Outputs are hashed hierarchically (**Merkle**: row-group → partition → table → warehouse root) after **canonicalization**: key-ordered rows, declared float quantum ε, declared collation. One root comparison acquits thousands of procedures at once; a mismatch drills to the diverging row-groups in O(log) comparisons. **Per-stage deposits** (an intermediate labīru at each DAG stage) let bisection localize *where* drift enters the pipeline — the Rigmu onset rite applied to stages instead of time.

**Rite III — Pattern Inquest, never Procedure Inquest.** Every diverging procedure yields a **divergence signature** (stage, tables, column set, kind ∈ {MISSING_ROWS, EXTRA_ROWS, VALUE_DRIFT, TYPE_DRIFT}, magnitude class). **FCA closure over (procedure × signature)** collapses thousands of divergences into few concepts — one root cause per concept, one inquest per concept. The harvest lands in Optional EAV (`mig.signature`, `mig.concept_id`); Nebuchadnezzar minting over migration failures is expected and welcomed.

**Rite IV — Decree or Bug.** Every **intended** difference from legacy is pre-registered as a decreed evolution particle (W5H2, signed) *before* confrontation. The colophon covers divergence **only** with decreed causes: undecreed divergence is a bug by definition — no excavation of Confluence/Jira decides intent after the fact. The Book Court's Semantic Gate may mine the legacy document corpus into **candidate decree drafts** (machine proposes; the migration architect confirms — GL-LIT-001 annotation clause). The corollary that saves migrations from bug-compatibility: when legacy was wrong, the decree says so — *"diverge here; the labīru carries the defect"* — and fixing the past happens in daylight.

## 3. Colophons per procedure (and per release)

- **CONCORD** — output kīma labīrīšu on the sealed input (within declared ε).
- **LAWFUL EVOLUTION** — divergence fully covered by decreed evolution particles.
- **SILENT DRIFT** — uncovered divergence → migration RIGMU: the procedure's release is frozen; the pattern-inquest (Rite III) opens; no silent close. A KAKIv4.0 / go-live gate passes only when the warehouse root is CONCORD-or-Lawful across all stages.

## 4. Honesty Clauses

- **Declared ε or drift**: undeclared numeric tolerance is a breach; ε and collation are part of the seal.
- **Nondeterminism named**: order-sensitive or clock-sensitive procedures are registered as such, confronted on canonical forms or statistical witnesses (multiset hashes), never quietly skipped.
- **Coverage is measured**: the rite reports the fraction of warehouse mass under CONCORD/LAWFUL/DRIFT — a migration's true progress metric, replacing "tests written."

## 5. Playbook

- **PB-339** — Two Scribes kernel: canonicalizer (key order, ε-quantization), Merkle table with O(log) diff drill, per-stage bisection, divergence-signature extractor, signature clustering (FCA-ready), decree-coverage colophon. Law tests **L32–L36**.

## 6. Seal

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
