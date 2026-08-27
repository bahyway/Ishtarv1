# GL-LBR-001 — The Labīru Doctrine
## Truth Has a Timestamp: The Mortality of GOLDEN and the Kīma Labīrīšu Rite

**Ecosystem:** BahyWay.Ecosystem v4.0 — EnkiDB·7004 / EnkiDW·7005 / NUZI archive
**Consumes:** GL-NSR-001 + A1/A2 (Markasu · Temennu · Rigmu), GL-STY-001 (NĀRU), GL-SHP-001 (Ṣalmu), Point-in-Time Totality, Kidinnu Standard (Ed25519)
**Status:** SEALED — landed by PB-338 as `crates/labiru`, 4/4 tests passing (L28-L31).
**Author:** DUB.SAR 𒁾

**Names.** *Labīru* — old, original; the ancient tablet from which copies descend. *Kīma labīrīšu (šaṭir)* — "(written) according to its original," the scribal colophon certifying a copy confronted against its source. Both unspent on the ledger; no register could fit better.

---

## 1. The Mortality of GOLDEN (the doctrine)

**GOLDEN is not eternal truth. GOLDEN is truth witnessed at a time, held under watch.** Conclusive evidence is expected — not feared — that golden particles age and degrade, through two channels this tablet names:

- **World-drift:** the record stands still while the world moves; the golden record becomes a true statement about a world that no longer exists.
- **Record-drift:** the record itself moves while claiming stillness — migrations, re-serializations, encoding changes, storage rot, silent or unauthorized writes. Today's "golden" is *not byte-identical* to the truth established at initial ingestion, and any system that cannot measure that distance is asserting eternity it cannot inspect.

Systems without sealed origins and total journals cannot distinguish **the truth** from **what survived**. BahyWay makes the opposite promise — not that truth is eternal, but that **the distance from origin is always measurable, and every step of the path is explainable or the bells ring.**

## 2. The Origin Deposit (§labīru)

At initial ingestion, before any correction can touch it, every particle's origin form is sealed into **NUZI** (inward/archive, immutable by charter):

- content witness: the ingestion-time byte hash (bound to the KAKI Integrity dimension);
- context witness: tribe_id, Mandatory-EAV spine snapshot, the tribe's Ṣalmu and betti_signature at mint;
- authority witness: Kidinnu Ed25519 seal + ingestion timestamp + source lineage.

The Origin Deposit is **never updated** — not by steward, not by migration, not by decree. A decree may deposit a *new* labīru beside the old (a re-founding), but the old deposit is unerasable: the archive does not forget so that the present may be judged.

## 3. The Kīma Labīrīšu Rite (§confrontation)

Periodically — and always before a KAKIv4.0 release gate — the living particle is **confronted against its Origin Deposit**:

1. Recompute the living content hash and current context witnesses.
2. Compute the **Divergence D(t)**, decomposed along Hepta axes: Integrity (hash distance), Quality (EAV diff count), Temporal (staleness vs world witnesses), Shape (Ṣalmu/betti drift of the tribe). D(t) is journaled — **the aging curve of truth itself** — and Markasu may moor it: a Temennu on D with the slackening watch, so accelerating decay rings before it is visible.
3. Route the verdict:
   - **CONCORD** — *kīma labīrīšu*: the living record is according to its original (D within ε). The colophon is stamped.
   - **LAWFUL EVOLUTION** — the record diverged, and NĀRU accounts for **every step**: the divergence explains itself from the journal (corrections, decreed migrations, Gate-passed promotions). Truth aged in daylight. Stamped with its explanation chain.
   - **SILENT DRIFT** — the record diverged and the journal has **gaps**: some distance from origin has no witnessed cause. This is the collapse your note names — and it is a **RIGMU by definition** (GL-NSR-001-A1 §3): freeze, inquest (GL-NSR-001-A2, with the labīru itself as the WHEN/WHAT anchor — onset bisection runs against NUZI deposits), no silent close.

## 4. The Honesty Clause (against "eternal truth")

No BahyWay surface, document, or ŠĀRU report may describe GOLDEN as permanent, absolute, or eternal. The lawful vocabulary is: *"GOLDEN — witnessed at t₀, confronted at t, divergence D(t), status {CONCORD | LAWFUL EVOLUTION | under RIGMU}."* Claiming eternity is a named breach and accrues τ under TIAMAT, because an unmeasurable truth claim is a transparency deficit.

## 5. Playbook

- **PB-338** — Labīru kernel: origin-deposit writer (write-once enforced), confrontation rite (hash + context diff), Hepta-decomposed divergence, journal-coverage checker (does NĀRU explain the whole distance?), verdict router. Law tests: **L28** the deposit is write-once (second write refused, decree deposits beside, never over); **L29** fully journaled divergence → LAWFUL EVOLUTION; **L30** one journal gap → SILENT DRIFT → Rigmu opened; **L31** D = 0 exactly iff content and context are witness-identical (concord is exact, never approximate).

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
