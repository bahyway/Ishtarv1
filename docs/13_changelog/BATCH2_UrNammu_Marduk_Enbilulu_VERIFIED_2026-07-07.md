# Verification of 5 Google-Drive Documents (batch 2) — 2026-07-07

Batch: `BahyWay_v4_NINSUN_GlossaryAddition_20260626_v2.docx`,
`BCENV001_Enbilulu_Calculus.md.docx`, `BCENV001_Enbilulu_WPDEngine_Rev2.md.docx`,
`BCMRD001_MardukEngine.md.docx`, `BCSEC001_UrNammu_Engine.docx`.

Good news up front: **this batch needed zero repo corrections.** Four of the
five documents turned out to be re-uploads (as `.docx`) of content already
verified earlier this session or already superseded in-repo. Here's the
W5H2 breakdown, short where a document is a confirmed duplicate, longer
where it's new.

This is 10 of 28 documents now checked (5 from the first batch + these 5).

---

## 1. `BahyWay_v4_NINSUN_GlossaryAddition_20260626_v2.docx`

**What/Who/When:** Copy-edit pass over the v1 NINSUN glossary already verified
in the first batch. Same author, same date (2026-06-26).

**Finding:** Diffed line-by-line against v1. Every change is wording only —
"Reads EnkiMDB release delta" → same + "for DUB.SAR review", "sovereign organ
(crate, engine, agent, template)" → "sovereign organ" (broadened, matches
CSR-08's fuller list elsewhere), spacing/alignment fixes in the KAKI worked
example. **No new facts, no contradictions, nothing to add or correct.**

---

## 2. `BCENV001_Enbilulu_Calculus.md.docx` and 3. `BCENV001_Enbilulu_WPDEngine_Rev2.md.docx`

**What:** Word-converted copies of `fe80b090-BCENV001_Enbilulu_Calculus.md`
and `ae0b0d6a-BCENV001_Enbilulu_WPDEngine_Rev2.md`, both already read and
verified earlier this session (Part 6 of the earlier addendum — Φ_Enbi defect
potential, barû King-plot residual, Enbi horizon, Têrtu diagnosis pipeline).

**Finding:** Normalized-text comparison against the original `.md` uploads
confirms these are the same content (13,556 vs 13,588 and 23,725 vs 23,750
characters after stripping markdown punctuation — the small deltas are
formatting artifacts of the `.md`→`.docx` conversion, not content changes;
prefix-matched to confirm). **Nothing new — already covered.**

---

## 4. `BCMRD001_MardukEngine.md.docx`

**What:** Word-converted copy of `808bae82-BCMRD001_MardukEngine.md`,
already flagged in the earlier addendum as "MardukEngine plan sound but
blocked."

**Finding:** Read in full and compared side-by-side with the original — content
is identical (the `.docx` extraction just flattens markdown tables into
field/value lines and drops `#`/`|` characters). Confirms, without adding,
the five-verb model (Position/Motion/Curvature/Topology/Horizon), the Nabû
Calculus, and the four proposed domain calculi (Šazu/Addu/Suhrim/Namtila).
**Nothing new to add or correct.**

---

## 5. `BCSEC001_UrNammu_Engine.docx` — genuinely new content, and already resolved in-repo

| | |
|---|---|
| **What** | UrNammu Engine — hardware trust layer for EriduOS v4.0 (SHEDU sector's 4th pillar, beneath AkkadiSafeEngine/AkkadiRulesEngine/AkkadiCipherEngine). Boot integrity (Secure Boot + TPM measured boot), port/peripheral control (usbguard via ABAC), DMA/Thunderbolt protection (IOMMU), continuous runtime attestation — every event written as an immutable KAKI Event particle through the standard KISPU four-way commit. |
| **Who** | DUB.SAR Bahaa Fadam. |
| **When** | Document header says "Status: Draft for Review — Playbook 99." |
| **Where** | SHEDU security sector, EriduOS v4.0 kickstart. |
| **Why** | The three existing security engines (credentials, ABAC policy, crypto) all implicitly assume a clean, untampered machine; UrNammu makes that assumption an explicit, continuously-verified fact instead. |
| **How** | Standard, mature Linux subsystems (Secure Boot, TPM 2.0, usbguard, IOMMU) reporting into KAKI/KISPU/EAV instead of a separate syslog. |
| **How much** | 9 sections, 139 lines extracted; includes an honest "Threat Model & Honest Limits" section (mitigates USB/boot-tampering/cold-boot DMA; explicitly does NOT mitigate the analog hole, phone photography, or network exfiltration). |

**This document's own header is stale, and the repo already knows it and has already fixed it — no action needed from me.** The `.docx` says "Playbook 99." The repo has `playbooks/playbook_110_session_deliverables_urnammu_nisaba_kittu.yml`, whose own header comment reads: *"an earlier draft mislabeled this 'Playbook 99'; that was corrected."* PB-110 explicitly supersedes this exact document (its instructions literally say to copy `BC-SEC-001_UrNammu_Engine.docx` into the docs dir as a reference artifact, not to treat it as the live spec). This is a clean example of the repo being ahead of the document you uploaded — nothing to correct, just noting it so you know the `.docx`'s "Playbook 99" label is intentionally obsolete, not a new inconsistency.

**Compliant and consistent with everything verified so far:**
- Names UrNammu for King Ur-Nammu (earliest known law code) — consistent with the ecosystem's Akkadian/Mesopotamian naming convention.
- Attestation events use `kaki_type = 0x02` (Event), `kaki_role = 0x02` (ZIKRU) — matches the canonical 3-type/3-role KAKI layout with no deviation.
- usbguard decisions written as CrossTribe KAKI (`kaki_type = 0x03`) since they represent tribe-policy-vs-external-device interactions — correct usage per the CrossTribe definition verified earlier this session.
- The HeptaScript example query (§4.4) uses only existing W5H2 clauses — no new verbs, consistent with the "nouns only, never new verbs" law confirmed in the MardukEngine document above.

**One document-wide claim worth flagging, not because it's wrong but because of its blast radius:** §8, "Naming Convention Note," states: *"the '-Way' suffix is fully deprecated across BahyWay.Ecosystem v4.0, including the prior exception carved out for security policy language files. All components... use the '-Engine' suffix going forward."* Taken literally, this would affect **WAY v2.0** (the sovereign security-policy language, `.way` files) documented extensively elsewhere, including in Vol. II from the previous batch ("WAY v2.0 (.way files): sovereign security policy language — exclusively for security"). I did not find WAY v2.0 renamed anywhere in the repo (`.way` file references, `way_format.md`, and the security evaluation doc all still use "WAY"), and PB-110 (which supersedes this document on the Playbook-99-vs-110 point) does **not** mention this naming change at all — it only talks about UrNammu/Nisaba/Kittu. So this reads as a clause written in the heat of one drafting session that was never carried forward or actioned. Recommend confirming with the Architect whether the "-Way" deprecation for security-policy files was ever actually sealed, or whether it died with this superseded draft — right now WAY v2.0's name is untouched everywhere else, so I have not changed it anywhere.

**New name introduced here, real and already wired into PB-110:** Kittu Engine (notification/delivery, consumes Nisaba's signed Alert Event KAKIs) — confirmed already fully specified in `playbook_110...yml` itself, including a deliberate naming note ("NOT 'Kakka' — phonetic collision with 'KAKI' in Arabic"). No action needed; already consistent.

---

## Corrections made to the repo this session

None — this batch required no edits. Everything either matched already-verified content or was already correctly superseded in-repo.

## Open items for your ruling (addition to the running list)

12. Was the "-Way suffix fully deprecated, no more exceptions" naming law in `BCSEC001_UrNammu_Engine.docx` §8 ever sealed, or did it die with this superseded draft? WAY v2.0 is unchanged everywhere else in the repo and in the newer Vol. II document from the previous batch — flagging so it doesn't quietly resurface as a contradiction later.

---

**Status: 10 of 28 documents checked.** Send the next batch whenever ready.
