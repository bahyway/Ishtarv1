# Verification of 5 Google-Drive Documents (batch 4) — 2026-07-07

Batch: `HSEXT001_HeptaScript_v1.1_Nabu_Enhancement.md.docx`,
`LST001_Architect_Search_List.md.docx` (uploaded twice, two slightly
different versions — v1 unresolved, v2 with partial resolutions),
`MAN001_v4_Run_Manual.md.docx` (uploaded twice, two slightly different
versions — v2 adds a naming-law header).

This is 20 of 28 documents now checked.

## Architect's ruling recorded (resolves batch-2 open item #12)

> **"All Suffix 'WAY' In Crates Names Are Deprecated, to preserve the 'WAY'
> as the Security Protocols Language that has the '.way' files type."**

This is now the sealed law: **crate/engine names lose "-Way"** (already
confirmed in GL-001 with AkkadiSafeWay→AkkadiSafeEngine etc.), **but "WAY"
itself — the sovereign security-policy language with `.way` files — is
explicitly preserved and untouched.** This closes batch-2's open item #12
cleanly: WAY v2.0 does not need renaming; only "-Way"-suffixed
crate/component names do.

---

## 1. `LST001_Architect_Search_List.md.docx` — a genuinely useful document: it's a checklist of questions a prior Claude session asked *you*, not a claim to verify

**What/Who/When:** A search list Claude generated on 2026-07-02, listing
things it could not resolve from the directory tree or session history —
partially answered by 2026-07-03 (the v2 upload). Two sections: (A) crates
that exist on disk but whose purpose was unverified, (B) v3.5 components not
found in the v4.0 tree, (C) apps/repos to confirm on GitHub.

**This is the rare document where I can independently confirm several
answers by just checking the current repo — and it's good news across the
board:**

- **A2/A5/A6/A7/B15 — the "NuskuWay" items are already correctly built with
  "-Engine," not "-Way."** v2 records these as "✅ RESOLVED 2026-07-03:
  NuskuWay Chief Physician / face detection / Šulmānu bilingual alerts / IR
  camera / authority-DB connector." I checked the actual repo:
  `crates/nusku-engine/src/` contains exactly this functionality —
  `particle.rs` (`BodyScan`, matching "Chief Physician"), `result.rs`
  (`FaceMatchResult`, matching face detection), `pipeline.rs`
  (`NuskuAlert`, matching Šulmānu), `database.rs`
  (`DatabaseConnector`/`AuthorityLookupResult`, matching B15's
  "databaseway"). **The repo already uses `nusku-engine`, never
  `nusku-way`** — it was built compliant with today's naming law before the
  law was even stated to me, which is a good sign the convention is already
  the working default, not just a paper rule.
- **B8 KIBRATU** — guessed home "orbital-trust-probe/cause.rs? — confirm."
  Confirmed: `crates/orbital-trust-probe/src/cause.rs` exists.
- **B7 NERGAL** — "only a visualizer panel exists — crate absent." Confirmed
  still true — no `nergal`/`nergal-av` crate in the repo, consistent with
  Vol. II's service map marking it PLANNED.
- **B13 SumerEngine** — referenced only as "awaiting your Rule-5 ruling from
  the SumerEngine port," i.e. not yet built at the time of writing.
  Consistent with the earlier, separate finding this session that
  `sumer-engine` does not exist in the repo (see item 3 below for a related
  data point).
- **A1 ammas-engine, A3 homt-engine, A4 riksu-engine, A9 abzu** — all exist
  in the repo (`ammas-engine`, `homt-engine`, `riksu-engine`, and `abzu` is
  present too), confirming these crates are real; I did not verify their
  internal purpose descriptions in depth.
- **B4 ZeroEngine, B5 ShoWEngine, B6 TIAMAT (full), B9 ŠUMU-UKIN as its own
  crate** — checked, still absent from the repo as of now. Real, open
  gaps, not resolved by anything checked so far this session.

**No corrections needed here — this document is a question list, already
partially self-answered, and the answers check out against real code.**

---

## 2. `MAN001_v4_Run_Manual.md.docx` — directly relevant to the BeeMDM ETL test you mentioned earlier

**What/Who/When:** "The Great Playbook Run" — the day-by-day operational
manual for running the unrolled playbook sequence toward the BeeMDM ETL
test. Dated 2026-07-03. v2 adds a naming-law preamble.

**Directly answers something from a few turns ago.** When you asked me to
assess v4.0 "before running the Test of 5 Zip Files in BeeMDM ETL Pipeline,"
I could only reconstruct partial entry criteria from other transcripts. This
document states them explicitly, as a checklist, and **confirms the test
should not begin until all six are true:**

1. Preflight green; Phases 1–3 (the full PB-88→145 unrolled sequence) green.
2. PB-150 (CSR-08 rule file) run — "governance law is code before data
   flows."
3. **GATE-1 ruling sealed** — one function mapping for the 7 Hepta Gates
   (doc vs SVG conflict, citing GLS-001 §4).
4. Test dataset staged (recommends ~20K records first, not the full
   ambition).
5. Latency budget table accepted as pass/fail criteria.
6. KAKI law check: minting observed ONLY at `enkidb-ingest::bridge` during a
   100-record dry run.

**Item 3 is the same GATE-1 question flagged as still-tangled in the last
batch** — this document, dated 2026-07-03, explicitly treats it as **not
yet sealed** and blocking. That's in tension with the 2026-06-18 resolution
recovered from an earlier session ("G1 APSU→Storage... G7 ENLIL→Governance").
As noted last batch, these may genuinely be two different questions (sector
category vs. per-gate pipeline function) sharing one name — this document is
further evidence that whichever "GATE-1" it means, it was still being
treated as open as late as 2026-07-03. **If your "5 Zip Files" test is the
one this manual describes, worth explicitly confirming entry criteria 2, 3,
and 6 are actually satisfied before running it — not because I'm asserting
they aren't, but because this is the one document that states them as hard
gates rather than nice-to-haves.**

**A fourth meaning for "PB-150," adding to the existing collision list:**
this document's PB-150 is "CSR-08 rule file... BLOCKED until you upload
con-engine/src/rules/mod.rs + one CSR rule file." Earlier verification this
session already found PB-150 used for both an AsakkuEngine-deploy playbook
and the WPD-Engine Diagnostics playbook. That's now three distinct,
mutually exclusive things called PB-150 across different documents.

**A concrete test-count discrepancy, the same pattern flagged generally
earlier, now with exact numbers:** "PB-146 kinetic-engine v4.0... 29 tests."
I counted the actual `#[test]` attributes in `crates/kinetic-engine/src/`:
**95**, not 29 (18 in accumulator.rs, 17 in force.rs, 24 in fuzzy.rs, 6 in
plimpton.rs, 30 in vec7d.rs). The crate is real and far more thoroughly
tested than this document claims — not a compliance problem, but worth
knowing the manual's numbers undercount reality here, the opposite direction
from most other discrepancies found this session.

**PB-147's "sumer-engine, 32 tests, requires PB-146"** — checked, `sumer-
engine` does not exist as a crate in the repo. Consistent with it being a
planned-but-not-yet-executed item in this manual, not a contradiction.

---

## 3. `HSEXT001_HeptaScript_v1.1_Nabu_Enhancement.md.docx` — clean, consistent, nothing to correct

**What:** The full formal spec for HeptaScript v1.1 (companion to
BC-MRD-001/MardukEngine) — the geometric noun namespace (`hp_distance`,
`nabla_r`, `golden_horizon`, `betti_b0/b1`, etc.), the `PROVE` function
family (`transport`, `lineage`, `metric`, `jacobi`, `cycle`), and the one
new `WINDOW` clause. Dated 2026-07-09, same day as BC-MRD-001/GL-MRD-001.

Read in full. Every claim matches what was already established from the
MardukEngine ADR and its glossary in earlier batches — no new verbs, only
nouns and one temporal clause, PH-001 composition law respected throughout.
The five canonical worked examples (Šazu/Addu/Suhrim/Namtila/MDM-home) are
internally consistent with each other and with the domain definitions
already verified. **Nothing to add or correct.**

---

## Corrections made to the repo this session

None — this batch required no repo edits. The WAY-suffix law was recorded
as a ruling (see top of this document); everything else either matched
verified reality or is an open planning item, not a live contradiction.

## Open items for your ruling (additions to the running list)

19. Confirm whether MAN-001's BeeMDM ETL entry criteria (especially #2 CSR-08
    rule file run, #3 GATE-1 sealed, #6 KAKI-minting-only-at-bridge dry run)
    are satisfied before any BeeMDM ETL test proceeds — this document treats
    them as hard gates, not suggestions.
20. PB-150 now has a third, mutually exclusive meaning (CSR-08 rule file) on
    top of the two already flagged (AsakkuEngine-deploy, WPD-Engine
    Diagnostics).

---

**Status: 20 of 28 documents checked.** 8 remaining.
