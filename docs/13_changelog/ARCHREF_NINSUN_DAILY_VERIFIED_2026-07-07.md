# Verification of 5 Google-Drive Documents (Jun 26–27 batch) — 2026-07-07

Batch: `BahyWay_v4_Architecture_Reference_20260627.docx` (Vol. I),
`BahyWay_v4_ArchRef_Vol2_BeeMDM_TripleO_20260627.docx` (Vol. II),
`BahyWay_v4_DailyReport_20260626.docx`,
`BahyWay_v4_NINSUN_GlossaryAddition_20260626.docx` (uploaded twice, byte-identical).

This is 5 of the 28 documents you downloaded from Google Drive (Jun 26 →
today). The other 23 have not been checked yet — send them and I'll continue
the same way. Findings below use W5H2 per document: what it says, whether
it's already real in the repo, and what to do about it.

---

## 1. `BahyWay_v4_Architecture_Reference_20260627.docx` (Vol. I) — SEALED 2026-06-27

| | |
|---|---|
| **What** | Canonical reference: KAKI byte layout, 7 EnkiDB types (ports 7001–7007), ENLIL 6-index stack, HeptaShellIndex (882 zones), memory budget at 1B scale, 7 GeoLaws, BeeMDM É-DUBBA gate table, 4 AI agents, CSR-01–08, reserved tribe IDs. |
| **Who** | DUB.SAR Bahaa Fadam. |
| **When** | 2026-06-27, marked SEALED. |
| **Where** | Governs `enkidb-kaki`, `enkidb-indexes`, `geo-engine`, `con-engine`, `enkimdb`. |
| **Why** | Single-page architectural source of truth — exactly the kind of document EnkiDDB is meant to make queryable instead of re-uploaded. |
| **How** | Table-driven spec, no prose ambiguity. |
| **How much** | 15 numbered sections, ~765 lines extracted. |

**Compliant / matches verified repo state — safe to treat as current:**
- KAKI layout κ[6]/κ[7]/κ[8..11]/κ[12..13]/κ[14..15] — matches `enkidb-kaki` and every other canonical source this session. κ[8..11] stated explicitly **"reserved — never repurpose"**, a third independent source contradicting ADR-003's `seq_counter` reassignment (still an open Architect item, not resolved by count).
- 7 EnkiDB types, ports 7001–7007 — names (EnkiDB/EnkiDW/EnkiSDB/EnkiODB/EnkiQDB/EnkiMDB/EnkiDDB) match the repo's own `BAHYWAY_ECOSYSTEM_V4_GLOSSARY.md`/`ROADMAP.md`. Port numbers were not previously on record in repo docs — worth adding if not already in a service-map doc.
- CSR-08 "Architect Sovereignty" — same wording as the NINSUN glossary doc (item 4 below). **Added to `docs/00_codex/BAHYWAY_ECOSYSTEM_V4_GLOSSARY.md` CSR table this session** (was missing; table only had CSR-01–07).
- AI Agent Quartet table (TamuzAI/EaAgent/NINSUN/NuskuAgent) — TamuzAI and EaAgent confirmed LIVE in repo (`ea-agent-core`, `ea-agent-algebra`, `ea-agent-chat`, `ea-agent-oracle`, `enkidullm-chat`); NINSUN confirmed built (`ninsun-agent`, `ninsun-steward-bridge` — ahead of this doc's "To build" status, i.e. repo has progressed past this snapshot); NuskuAgent confirmed still absent (doc says "v4.3 future" — consistent, no crate exists).

**Needs an Architect ruling, not a repo error:**
- KAKI type table lists only 3 `kaki_type` values (0x01/0x02/0x03) — **no Pattern=0x04**. This is now the *third* "SEALED" canonical source (alongside the KAKI PDF and ADR-008) that omits NISABA's `Pattern=0x04`, which nonetheless exists and is used in `enkidb-kaki/src/types.rs`. Strengthens the case that Pattern=0x04 is a real, later, unratified addition — still open.

**New naming surfaced, not yet in repo, worth adding if you confirm it's current:**
- `AgentId` enum in `crates/agent-council/src/agent.rs` defines a **different** 3-member group — `{TamuzAI, Ninsun, Pazuzu}` — as "the three sovereign AI agents on the pattern governance council," for Phase-1/Phase-2 Pattern-KAKI evaluation specifically. This is a narrower, purpose-scoped council, not the ecosystem-wide 4-member "Agent Quartet" this document defines. Not necessarily a conflict (different council, different job), but the overlapping membership + similar naming ("agent council" vs "Agent Quartet") is worth a one-line disambiguating note somewhere so a future reader doesn't assume they're the same body. Flagged, not changed.

---

## 2. `BahyWay_v4_ArchRef_Vol2_BeeMDM_TripleO_20260627.docx` (Vol. II) — SEALED 2026-06-27

| | |
|---|---|
| **What** | BeeMDM ETL detail: É-DUBBA 7-gate table (S1–S7) with Akkadian/physical names and NĀṬIRU bitmap bits, 7 processing-chain stations × gate matrix, NERGAL AV engine spec, TIAMAT 5-engine + alert-level detail, KIBRATU 7-cause table, DubSar IDE architecture, full W5H2 clause reference, service map, end-to-end ZIP ingestion flow. |
| **Who** | DUB.SAR Bahaa Fadam. |
| **When** | 2026-06-27, same day as Vol. I, marked SEALED. |
| **Where** | Governs the whole ADAD/BeeMDM ingestion path. |
| **Why** | This is the station→EnkiDB-type mapping that a separate transcript (`Before_Run_BeeMDM_ETL_Tests_.md`) said was still an open blocker for writing the BeeMDM test — it already existed, in this document, the same day. |
| **How** | Same table-driven style as Vol. I. |
| **How much** | 13 sections, ~1109 lines extracted. |

**The one direct contradiction found — Vol. I vs Vol. II disagree with each other, same day, both "SEALED":**

Vol. I §14 names Gate **S7** as **`KIBRATU`**. Vol. II §5 (and §4.2, §5.1, §13.1 — five separate places, all internally consistent with each other) names Gate **S7** as **`NERGAL_GATE`**, and dedicates all of §7 to NERGAL as the sovereign AV engine that runs at S7. Vol. II's usage is corroborated independently: KIBRATU is documented everywhere else this session (con-engine CSR-06, the 7 KIBRATU causes, `crates/agent-council`) as the **cause-analysis engine**, not a gate name — so Vol. II's S7=NERGAL_GATE is very likely correct and Vol. I's S7=KIBRATU is the transcription error. Not changed in either source document (both are your Word files, outside my edit authority) — flagging for your ruling since both claim SEALED status on the same day.

**Compliant / confirms and sharpens existing repo knowledge:**
- É-DUBBA gate sequence S1–S7 (MASHSHARU→NĀMZITUM→PASHIRU→KISPU_GATE→UTNAPISHTIM→NISABA→NERGAL_GATE) — MASHSHARU and NĀMZITUM already referenced correctly in repo (`naramsin-integrity/src/lib.rs`, `BAHYWAY_ECOSYSTEM_V4_GLOSSARY.md`). **PASHIRU and UTNAPISHTIM appear nowhere in the current repo** — genuinely new names to add if you confirm this gate sequence is current.
- KIBRATU 7-cause table (STALE/PARALLEL_FAILURE/UNKNOWN_STATE/MISCLASSIFIED/HIDDEN_PATTERN/INTRUDER_CORRUPT/PUZRU) — matches exactly what's already on record from `enkidb-journal/src/event_cause.rs` and prior sessions. No drift.
- **TIAMAT alert ladder — this document resolves the 3-way naming conflict I had flagged in `BAHYWAY_ECOSYSTEM_MANUAL_V4.md`.** §8.1 gives a 6-level ladder `GREEN(0)/DILBAT(1)/KAKKAB(2)/ERRA(3)/NERGAL(4)/MAROON(5)`. The Manual's 5-level list (missing ERRA) and PAZUZU's 4-level list (missing NERGAL, using ERRA in its place) both turn out to be partial restatements of this fuller ladder, not genuine alternatives. **Corrected the flagged note in `BAHYWAY_ECOSYSTEM_MANUAL_V4.md` this session** to record this; the table itself was left as-is pending your sign-off on inserting ERRA. BC-ENV-001's separate `Stable/Watch/Serious/ERRA` wording is still unreconciled — different vocabulary entirely, not clearly the same ladder.
- Confirms NERGAL-as-AV-engine and NERGAL-as-alert-level is **intentional**, stated explicitly in §7: "NERGAL is also a TIAMAT alert level." Not a drafting accident — still recommend an explicit Architect sign-off since it reads as a collision to an implementer, but it's not an undiscovered bug.
- Service map (§12) status column: `con-engine` and `enkimdb` marked BUILT, `enkidb-query-server`/`enkidb-persist`/`heptascript`/`enkidb-enlil-index`/`hepta-shell-index`/`geo-engine`/`enkidb-ingest` marked LIVE, `nergal-av`/`tiamat`/`kibratu`/`ninsun-agent` marked PLANNED/PARTIAL — repo has since progressed past several of these (`ninsun-agent` now exists; treat this service map as a dated snapshot, not current status).
- KISPU HeadStore fix identified as the bottleneck for "HeptaScript query 10K: ~3min current → <1s after KISPU fix (fix in PB98)" — this is the **eighth** independent confirmation this session that the KISPU/`akk_decode` fix is real, known, and still outstanding.

---

## 3. `BahyWay_v4_DailyReport_20260626.docx`

| | |
|---|---|
| **What** | Session log for 2026-06-26: Playbooks 93–97 deployed (geo-engine, hepta-shell-index, PAZUZU simulation, enkimdb, ENLIL query server partial), infra state snapshot, root-cause note for the KISPU bug, next-day priority list. |
| **Who** | DUB.SAR Bahaa Fadam, on eriduous-vdi. |
| **When** | 2026-06-26. |
| **Where** | `enkidb-node-read`/`enkidb-node-write` (192.168.122.x), `journal.bin` (80,272 NAJAF_CEMETERY particles). |
| **Why** | Operational record — not a design document, needs no compliance check, only a consistency check against later claims. |
| **How** | Plain status report. |
| **How much** | 59 lines. |

**Compliant — a ninth independent confirmation of the KISPU root cause, and it's the most precise one yet:** *"bridge.rs confirmed: EAV triples ARE encoded with akkvalue codec (codec_encode). eav_triple_to_value() is the correct decode path — NOT akk_decode(&bytes, 0). Fix: replace akk_decode(&triple.value, 0) with enkidb_ingest::bridge::eav_triple_to_value(&triple). This is ONE line change in enkidb-query-server main.rs."* This is a real diagnostic entry, not a chat narrative — nothing to add or correct, it just reinforces the existing ledger. Nothing here to flag as non-compliant.

---

## 4. `BahyWay_v4_NINSUN_GlossaryAddition_20260626.docx`

| | |
|---|---|
| **What** | Formal definition of NINSUN as 4th Agent Quartet member, the Quartet's sealed roster, NINSUN's architectural boundaries (may/may-not list), CSR-08 text, NINSUN's own KAKI record, and the Prescription Report structure. |
| **Who** | DUB.SAR Bahaa Fadam. |
| **When** | 2026-06-26. |
| **Where** | Governs `ninsun-agent`, `ninsun-steward-bridge`. |
| **Why** | Defines the human/AI authority boundary (CSR-08) that everything else in the ecosystem must respect. |
| **How** | Definition + rule list + KAKI worked example. |
| **How much** | 74 lines. |

**Compliant and already substantially real in the repo — this is the strongest-verified document in the batch:**
- `ninsun-agent::analyze()` exists in the repo and does what this doc describes: takes `KakiSummary` batches, detects drift (`FUZZY_SUSTAINED_7DAY`, `DEAD_UNREVIEWED` — confirmed by reading the actual source and its passing tests), emits advisory `RefineProposal`s, never mutates a particle. `ninsun-steward-bridge::NinsunAdvisoryQueue` also exists with `confirm()`/`reject()` journaling to `EventCause::NinsunAdvisoryConfirmed`/`NinsunAdvisoryRejected` — matches this document's "diagnosis autonomous, execution DUB.SAR-only" model exactly, and matches the repo's own glossary entry for NINSUN nearly word-for-word (the glossary entry looks like it was written directly from this document or a shared source).
- CSR-08 text here is identical in substance to Vol. I's CSR-08 entry — two independent sealed sources agreeing verbatim is a strong basis for treating it as real. **Added to the repo's CSR table this session** (see item 1).
- NINSUN's own KAKI worked example (tribe_id=0xFF00, kaki_type=0x01 Identity, kaki_role=0x01 KISHIB, release.station=S7) is internally consistent with the canonical KAKI layout — no violation.

**Needs an Architect note, not a fix:**
- NINSUN's 6-state orbital lifecycle (`GOLDEN_GEM/GOLDEN_ALIVE/FUZZY_AGED/FUZZY_GRAY/FUZZY_DECAY/DEAD_EXPIRED`) describes *NINSUN's own health*, distinct from a regular particle's orbital state vocabulary used elsewhere (Birth/Active/Golden/Aged in NISABA's docs, GOLDEN_ALIVE/FUZZY_GRAY/DEAD_SEALED in the BeeMDM gate-failure table above). These appear to be compatible (same root vocabulary, NINSUN's is a superset used reflexively), but nobody has stated that explicitly — worth a one-line confirmation that "particle orbital states" and "NINSUN's self-health states" are the same enum, not two similarly-named but separate ones.

---

## Corrections made to the repo this session

1. `docs/00_codex/BAHYWAY_ECOSYSTEM_V4_GLOSSARY.md` — added CSR-08 to the CSR table (was silently missing; table said "7 rules" and stopped at CSR-07).
2. `workspace/bahyway_v4/docs/20_meta_engine/BAHYWAY_ECOSYSTEM_MANUAL_V4.md` — updated the TIAMAT naming-conflict note with Vol. II's 6-level ladder, which resolves two of the three previously-flagged variants as partial restatements rather than genuine alternatives. Table itself not rewritten, pending your sign-off on inserting `ERRA`.

## Open items for your ruling (additions to the existing list)

8. Vol. I vs Vol. II: is Gate S7 named `KIBRATU` or `NERGAL_GATE`? Both documents are dated 2026-06-27 and marked SEALED; they disagree. Evidence favors `NERGAL_GATE` (Vol. II is internally consistent across 5 mentions; KIBRATU is independently and consistently documented elsewhere as the cause-analysis engine, not a gate).
9. Should `BAHYWAY_ECOSYSTEM_MANUAL_V4.md`'s TIAMAT table be rewritten to the 6-level `GREEN/DILBAT/KAKKAB/ERRA/NERGAL/MAROON` ladder now that Vol. II supplies it? And separately: is BC-ENV-001's `Stable/Watch/Serious/ERRA` a deliberately distinct domain ladder, or should it be retired in favor of the canonical one?
10. `PASHIRU` and `UTNAPISHTIM` (É-DUBBA gates S3/S5) appear in Vol. II but nowhere else in the repo — confirm they're current before they're wired into code.
11. Is `crates/agent-council`'s 3-member `{TamuzAI, Ninsun, Pazuzu}` pattern-governance council the same body as, or deliberately distinct from, the 4-member ecosystem "Agent Quartet" `{TamuzAI, EaAgent, NINSUN, NuskuAgent}`? Same two names overlap (TamuzAI, Ninsun/NINSUN), different third/fourth members, different stated purpose.

---

**Status: 5 of 28 documents checked.** Send the remaining 23 and I'll continue the same way — verify against actual repo code where possible, cross-check against what's already been confirmed this session, and flag anything genuinely new, missing, or contradictory rather than assuming.
