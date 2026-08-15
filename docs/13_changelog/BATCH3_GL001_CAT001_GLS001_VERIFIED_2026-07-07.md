# Verification of 5 Google-Drive Documents (batch 3) — 2026-07-07

Batch: `CAT001_BahyWay_v4_Sovereign_Service_Catalog.md.docx` (2026-07-02),
`GL001_Glossary_v4.docx` (undated, "Living Document"),
`GLMRD001_MardukEngine_Glossary_Playbooks.md.docx` (uploaded twice,
byte-identical, 2026-07-09), `GLS001_v4_New_Concepts_Glossary.md.docx`
(2026-07-03).

This batch produced two corrections **to my own conclusions from the last
two batches** — new evidence changed my read on two open items. That's
worth flagging plainly rather than burying: I was wrong about the "-Way"
suffix, and my TIAMAT resolution needs revising. Details below.

This is 15 of 28 documents now checked.

---

## 1. `GL001_Glossary_v4.docx` — GL-001, "Living Document" — the most consequential document in this batch

**What/Who/When/Where/Why/How:** The canonical cross-sector terminology
reference, explicitly maintained as a living document ("updated as new
components are sealed"). No date printed, but it already references UrNammu
Engine and Kittu Engine — both of which postdate the 2026-06-27 Architecture
Reference volumes — so it is chronologically **later** than Vol. I/Vol. II
from the first batch, and should be weighted accordingly on points where
they disagree.

### Correction #1 — I was wrong last batch: the "-Way" suffix deprecation is real and sealed, not a dead draft clause

Last batch I flagged `BCSEC001_UrNammu_Engine.docx`'s claim that "-Way" is
fully deprecated (including the prior security-language exception) as
*"reads like a clause written in the heat of one drafting session that was
never carried forward."* **GL-001 proves that wrong.** §5 states it as
current law in identical language, and — critically — backs it with actual
renames:

| Current (v4.0) | Former | 
|---|---|
| AkkadiSafeEngine | AkkadiSafeWay |
| AkkadiRulesEngine | AkkadiRulesWay |
| AkkadiCipherEngine | AkkadiCipherWay |

§10 repeats it as a formally retired term. This is two independent documents
(the UrNammu ADR and the living glossary) agreeing verbatim — strong enough
to treat as sealed.

**Checked against the actual repo — the rename is partially applied, not
fully propagated:**
- `workspace/bahyway_v4/docs/19_roadmap/BAHYWAY_ECOSYSTEM_V4_ROADMAP.md` already uses
  `AkkadiSafeEngine` — good, current.
- `workspace/scripts/tools/bahywaylab.sh` line 556 still said
  `AkkadiCipherWay` in a section-header comment, with no v3.5 qualifier.
  **Corrected this session** to `AkkadiCipherEngine`, with a note citing
  GL-001.
- Two `.akk` policy files (`pollution_way_policy.akk`,
  `import_schema_policy.akk`) also say `AkkadiRulesWay`/`AkkadiSafeWay` —
  **left unchanged**, because those comments are explicitly self-labeled
  `v3.5` ("Compiled by AkkadianAOL v3.5 → AkkadiRulesWay v3.5 Rust
  enforcement"). GL-001 itself says v3.5 terms are kept, not silently
  dropped, for historical accuracy — these are correctly-scoped legacy
  references, not current-law violations.

### Correction #2 — the TIAMAT alert-band note I wrote two batches ago needs reversing, not just extending

I previously concluded (using Vol. II) that the ladder is 6 levels with
`NERGAL` and `ERRA` both present, and that `NERGAL`'s dual use (AV engine +
alert level) was intentional. **GL-001 says the opposite**, verbatim: *"ERRA
𒀭𒂗𒆳 replaces the incorrectly-named NERGAL alert level (NERGAL is the BahyWay
AV engine, not an alert)."* That is a direct call that `NERGAL` at the alert
layer was a **mistake**, not a deliberate dual-purpose name — the reverse of
what Vol. II states about itself. Since GL-001 is the later, living
document, this is very likely the standing rule, which means
**PAZUZU_SIMULATION.md's ladder (`GREEN/DILBAT/KAKKAB/ERRA/MAROON`) was
right all along**, and it's `BAHYWAY_ECOSYSTEM_MANUAL_V4.md`'s table (still
showing `NERGAL`) that should change, not the other way around. Updated the
flagged note in the Manual this session to record both passes honestly
(kept the superseded first reading rather than deleting it, consistent with
this ecosystem's own "old state is never erased" law) and recommend the
`NERGAL`→`ERRA` table edit pending your direct confirmation. Table itself
left unedited — this is inferred from document recency, not a dated seal,
so I didn't want to silently flip it a second time without your sign-off.

### A third, still-different account of the É-DUBBA gate sequence

§9 gives: *"MASHSHARU → NĀMZITUM → TIAMAT Engine 5 → PAŠIRU → KISPU →
UTNAPISHTIM. Stage 7 (Nisaba/ṬUPŠARRU) triggers on top of this sequence."*
Compare:
- **Vol. I** (batch 1): S1 MASHSHARU...S7 **KIBRATU** (commit).
- **Vol. II** (batch 1): S1 MASHSHARU→S2 NĀMZITUM→S3 PASHIRU→S4
  KISPU_GATE(TIAMAT 5)→S5 UTNAPISHTIM→S6 NISABA(DATA_STEWARD)→S7
  **NERGAL_GATE** (AV scan + commit).
- **GL-001** (this batch): MASHSHARU→NĀMZITUM→**TIAMAT Engine 5**→PAŠIRU→
  KISPU→UTNAPISHTIM (6 items, different order — TIAMAT/PAŠIRU/KISPU
  sequenced differently than Vol. II), with Nisaba/art-generation as an
  add-on "Stage 7" that has nothing to do with NERGAL or an AV scan.

Three "sealed"/"canonical" documents, three different tellings of gate
count, order, and what Stage 7 even means. **`CAT-001` (item 2 below)
independently corroborates GL-001's version**, not Vol. II's — see below.
This is now the single most-tangled open item across all three batches, more
so than the TIAMAT naming. Recommend the Architect settle this with one
authoritative table rather than piecemeal.

**Other new terms from GL-001, checked against the repo:**
- **Riksu** — "cross-tribe relationship journal, third sovereign journal
  (alongside zakāru and Shedu)." Confirmed real: `crates/riksu-engine`
  exists with `topology.rs`, `snapshot.rs`, `arc.rs`. No conflict.
- **AdadAI** ("ingestion prediction agent") and **TERTUM** ("sovereign
  geometric data modelling language") — neither appears anywhere else in
  the repo or in any document processed this session. New, unverified,
  flagging rather than assuming they're current.
- **"Shedu" as a journal name** (one of the three sovereign journals) sits
  right next to **SHEDU as the security sector name** (Forge2) in the very
  same glossary, with no disambiguating note between them — worth a
  one-line clarification since it's the same kind of collision already
  flagged for NERGAL.

---

## 2. `CAT001_BahyWay_v4_Sovereign_Service_Catalog.md.docx` — dated 2026-07-02

**What:** A from-evidence service catalog — explicitly built from `tree_.txt`
(287 dirs, 776 files) plus the RM-001 roadmap, with an honest "state"
classification per crate (FULL / FULL+T / FULL+M / COMPACT / SHELL) based on
observable evidence (module count, tests/ present, MANUAL.md present) rather
than claims. 906 lines — the largest document in this batch; I did not read
it in full given its size and catalog nature, but sampled it for the
specific claims cross-referenced above.

**Corroborates GL-001's gate-sequence reading, not Vol. II's:** says
*"DataSteward (S7) is the human-confirmed authority NINSUN advises"* and
references *"É-DUBBA Stage-6 UTNAPISHTIM pass"* — both consistent with
GL-001's ordering (DataSteward/Nisaba at 7, UTNAPISHTIM at 6) and
inconsistent with Vol. II's (NISABA/DATA_STEWARD at S6, UTNAPISHTIM at S5,
NERGAL_GATE at S7). Two documents now agree with each other against one —
weight of evidence favors GL-001/CAT-001's version of the sequence, but this
is not the same as a dated seal, so still flagging for ruling rather than
rewriting anything.

**Its own honesty-seal methodology is worth calling out as good practice,
not a finding to correct:** it explicitly states a directory tree "proves
existence and module surface, not code completeness or test coverage" —
exactly the discipline I've been applying by extracting and compiling actual
playbook source rather than trusting "SEALED ✓" claims. This document is
self-aware about the same failure mode RM-002 fell into.

---

## 3. `GLS001_v4_New_Concepts_Glossary.md.docx` — dated 2026-07-03

**What:** A glossary of concepts extracted from two very large source
documents (`00/01_GA-GA_EnkiDW_EnkiDBTypes_.md`, 11,208 + 2,544 lines),
tagged with its own status flags: 🆕 NEW, ⬆ ENHANCES existing law, ⚔
CONFLICTS with sealed law (needs ruling), 🧪 EVALUATED-ONLY. This is the
most self-aware document processed so far — it flags its own open
contradictions instead of asserting SEALED status, which is exactly the
discipline this whole verification effort has been trying to impose from
outside.

**Its self-declared ⚔ conflicts, cross-checked against what's already known:**
- **⚔ GATE-1** — flags a doc-vs-SVG gate-function mismatch, but describes it
  with a **third** vocabulary that matches neither Vol. II's É-DUBBA stage
  functions nor the already-independently-confirmed 2026-06-18 seal
  ("G1 APSU→Storage, G2 ADAD→ETL, G3 SHEDU→Security, G4 MUMMU→Algebra,
  G5 ENKIDU→AI, G6 DUBSAR→Languages, G7 ENLIL→Governance" — recovered from
  session history and confirmed by the Architect earlier this project,
  matching GL-001's own "Forge2 7-sector architecture" entry). GLS-001's
  "Doc" column (Security/Structure/Compare/BlackBox/Steward/Cleanse/Quality)
  and "SVG" column (Identity/Validation/Enrichment/Harmonize/Dedupe/
  Scoring/Governance) describe detailed ETL-pipeline step responsibilities,
  not the broad sector categories the 2026-06-18 seal settled. **These are
  likely two different questions wearing the same "GATE-1" label** — the
  sector-level mapping (sealed) and the pipeline-step-level mapping
  (apparently still open, and now with a third naming scheme on top of
  Vol. II's and Vol. I's). Not resolved here; flagging the conflation
  itself as something the Architect should be aware of before ruling on
  either.
- **⚔ Z3-DEP** — matches exactly what was already resolved in an earlier
  session (Z3 at MUMMU/GeoEngine, design-time-only, downloaded once,
  Godot-model) — this document predates that resolution and can be treated
  as already answered.
- **⚔ C-1 "Quality in Nucleus"**, **⚔ C-2 tribe_id byte-width in WGSL
  kernels**, **⚔ C-3 tribe_id byte-width in HOMT OS v1.5** — all describe
  the same class of error already confirmed and corrected elsewhere this
  session: quality/color must be EAV-only, never in KAKI bytes; tribe_id is
  u16 at κ[4..5]. These read as pre-existing drift in older sketches
  (WGSL/GPU kernels, HOMT OS doc) that this glossary correctly caught — no
  new information, but confirms the drift exists in code/shader sketches
  I have not personally inspected. Worth a targeted grep of any `.wgsl`
  files in the repo if/when GPU work starts, not urgent now.
- **⚔ FierWall Defender vs hepta-sec-firewall** — asks whether these are the
  same engine or two layers. Not resolved in this document or anywhere
  else checked so far. New open item.
- **7-database pipeline reframing (⬆):** SDB→ODB→QDB→DB→DW→MDB→DDB as an
  *ordered* pipeline (not just parallel organs) — consistent with, and adds
  precision to, the 7 EnkiDB types already confirmed in Vol. I. No conflict.

---

## 4/5. `GLMRD001_MardukEngine_Glossary_Playbooks.md.docx` (uploaded twice, identical) — dated 2026-07-09

**What:** Companion glossary + playbook-planning sheet to `BC-MRD-001`
(MardukEngine), already fully verified in the first batch. Same date as the
ADR (2026-07-09), fully consistent with it — Nabû Calculus, the five verbs,
the four domain calculi (Šazu/Addu/Suhrim/Namtila), status "SEALED
(planning) — PB numbers assigned at unlock." Sampled the first 60 lines and
found no deviation from the already-verified ADR content. **Nothing new to
add or correct** — this is the glossary half of a document pair whose ADR
half was already checked.

---

## Corrections made to the repo this session

1. `workspace/scripts/tools/bahywaylab.sh` — renamed `AkkadiCipherWay` →
   `AkkadiCipherEngine` in a section comment, citing GL-001.
2. `workspace/bahyway_v4/docs/20_meta_engine/BAHYWAY_ECOSYSTEM_MANUAL_V4.md`
   — revised the TIAMAT flagged note to record GL-001's reversal of the
   NERGAL/ERRA reading (kept the superseded first pass in the text rather
   than deleting it).

## Open items for your ruling (additions to the running list)

13. Confirm: does `BAHYWAY_ECOSYSTEM_MANUAL_V4.md`'s TIAMAT table's `NERGAL`
    row become `ERRA`, per GL-001's explicit "ERRA replaces the
    incorrectly-named NERGAL alert level"? (Recommended, not yet applied.)
14. The É-DUBBA gate sequence now has three different tellings (Vol. I,
    Vol. II, GL-001/CAT-001) disagreeing on count, order, and Stage 7's
    meaning. Needs one authoritative table.
15. Is GLS-001's "GATE-1 conflict" (pipeline-step functions) the same
    question as the already-sealed 2026-06-18 GATE-1 ruling (sector
    categories), or a separate, still-open question wearing the same name?
16. FierWall Defender vs. `hepta-sec-firewall` — same engine, or two layers?
17. `AdadAI` and `TERTUM` (GL-001) — real and current, or draft-only terms
    that never got built out?
18. `Shedu` (a sovereign journal name) vs. `SHEDU` (the security sector) —
    same collision pattern as NERGAL; worth a disambiguating note.

---

**Status: 15 of 28 documents checked.** Send the next batch whenever ready.
