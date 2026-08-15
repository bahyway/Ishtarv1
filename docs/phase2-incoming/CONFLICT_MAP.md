# Phase 2 Conflict Map — Step 2

**Status: PROPOSALS ONLY. Nothing below is applied yet.** Step 3 applies
whichever resolutions the Architect confirms. This document was produced by
systematically diffing every PB-number and GL-doc-ID across all 7 incoming
batches against each other and against the real, already-committed
`playbooks/` and `docs/` trees.

## A. Real collisions requiring renumbering

### A1. `GL-VIZ-001` — batch 7 collides with an already-SEALED, already-IMPLEMENTED real-repo law
- **`docs/GL-VIZ-001.md`** (real repo, unchanged) — *Bivector Orbit Encoding
  and the BUZU Chunk*. SEALED and PROVEN, landed as working Rust
  (`crates/buzu-core`, 15/15 tests passing). **Cannot be touched or
  renumbered — this is load-bearing, tested code.**
- batch 7's `GL-VIZ-001_Morphological_Discovery_DRAFT.md` ("candidate,"
  DRAFT) is an unrelated topic (shape-as-reading) that happens to reuse the
  number. **Proposal: renumber to `GL-VIZ-007`.**
- batch 7's `GL-VIZ-002_Shape_Verdict_DRAFT.md` explicitly "extends
  GL-VIZ-001" (i.e., extends the doc being renumbered above) — its own
  number also collides (see A2). **Proposal: renumber together, to
  `GL-VIZ-008`**, so the 007→008 adjacency preserves the "extends" relationship.
- `GL-VIZ-003` through `GL-VIZ-006` (batch 7) — no collisions found, stay as-is.

### A2. `GL-VIZ-002` — batch 2 vs. batch 7
- batch 2's `GL-VIZ-002-orbit-witness-isolation.md` — status **SEALED
  (concept)** in its own thread, feeds on GL-VIZ-001 (the real BUZU law),
  escalates to GL-MRD-002. Topically consistent with the real repo's actual
  GL-VIZ-001. **Proposal: batch 2 keeps `GL-VIZ-002`.**
- batch 7's Shape Verdict doc renumbers to `GL-VIZ-008` per A1 above.

### A3. `GL-NAV-001` — three claims, one with a provenance question worth flagging directly to you
- batch 1's Flight-to-Location (Nabû/NaviEngine/Hubble) — embedded inside
  `pb-301`'s Ansible `copy:` task, not a standalone file. Structurally the
  most complete concept-seal (three guarded boundaries, domain-neutral,
  explicit NajafEngine instance). **Proposal: this becomes the base
  `GL-NAV-001`, extracted out of the playbook into its own `docs/` file
  (matching the PB-197+ convention — see A5).**
- batch 6's `GL-NAV-001_Hendursaga_Charter_AnnexA_DRAFT.md` — self-labels
  as "Annex A," DRAFT, unsealed. Its subject (Wādī al-Salām field
  architecture) is the *same domain instance* (NajafEngine) as batch 1's
  law, not batch 2's. **Proposal: this becomes the literal Annex A of the
  base law above, no renumbering needed — just correct filing.**
- batch 2's `GL-NAV-001-knowledge-graph-navigation.md` — a genuinely
  different topic (NabuEngine documentation-as-orbits). **Flag for you
  directly**: this file calls itself a "RECOVERY COPY... assembled
  2026-08-05 from the PB-184 session record," claiming to reconstruct
  sealed playbook text from 2026-07-26. But the real repo's actual
  `playbook_184` (`playbook_184_storage_prebuild_readiness_for_enkiddb.yml`)
  is about something else entirely — storage prebuild readiness, not
  NabuEngine orbits. Either that recovery is from a session/number that
  never actually landed in this repo, or there's a genuine mix-up in the
  other session's memory. I'm not able to resolve which from the files
  alone. **Proposal: renumber to `GL-NAV-002` regardless (it's a distinct,
  real topic either way), but please confirm the PB-184 provenance claim
  isn't pointing at content you expected to already exist somewhere.**

### A4. `PB-321` / `PB-322` — three-way and two-way collisions
| Number | Claimant | Proposal |
|---|---|---|
| PB-321 | batch 4 — Kidinnu Engine | **Keeps 321** — strongest claim: its own suite README states it explicitly as "next in sequence after the storage suite" (PB-310–320). |
| PB-321 | batch 5 — Arsenal Inventory Survey | Renumber to **PB-531** |
| PB-321 | batch 7 (GulaFederation) — Advisory API | Renumber to **PB-549** (see block below) |
| PB-322 | batch 5 — Deploy Šala v4 | Renumber to **PB-532** (stays paired after 531, preserving batch 5's own 321→322 sequence) |
| PB-322 | batch 7 (GulaFederation) — Synthetic Baghdad Dataset | Renumber to **PB-550** |

### A5. `PB-185`–`PB-200` (batch 2, 16 files) vs. real, already-committed playbooks
Every one of these numbers is already used in `playbooks/` for unrelated,
already-merged work (Anu index, Nisaba, Nergal naming, onion layers,
EnkiDB ingest CLI, SELinux mount fix). **Proposal: renumber the whole
block, in its existing internal order, to `PB-533`–`PB-548`.**

### A6. GulaFederation suite (batch 7, PB-321–326, 6 files)
Internally consecutive; two of its six numbers collide (A4). **Proposal:
shift the whole block together to keep it consecutive: `PB-549`–`PB-554`**
(gula-federation-advisory-api → 549, synthetic-baghdad-federation-dataset →
550, godot-mobile-hubble-scaffold → 551, offline-osm-tile-bundler → 552,
signed-advisory-verifier → 553, medicine-batch-audit → 554).

## B. Confirmed as NOT collisions (checked, false alarms ruled out)
- `GL-MED-001` (Medical Sector Charter) + `GL-MED-001_AnnexA_NinisinaEngine` —
  genuine annex relationship, both stay as-is.
- `GL-DDB-002` (EnkiDDB Corpus Law) + `GL-DDB-002_AnnexB_Babu_Intake_Law` —
  genuine annex, both stay as-is.
- `GL-DDB-003` (PreKAKI Schema Lifecycle) + `GL-DDB-003_AnnexA_SchemaFirst` —
  genuine annex, both stay as-is.
- No other GL-doc ID (GOV, HS3, KAKI, MEM, STD, TOOL, FOR, TKT, TPL, STY,
  ORG, MDM, DST, DDB-001/004) collides with the real repo or across batches.
- No PB number in batches 1, 3, 4 (except 321), 6, or the GulaFederation
  block (except 321/322) collides with the real repo's max of 290.

## C. Not a conflict, but worth noting
- `PB-360`–`374`, `380`–`389`, `390`–`393`, `394`–`397`, `398`–`401` (batch 7)
  exist only as *suite-description* Markdown documents naming a planned
  range — no individual numbered `.yml` files were delivered for these
  ranges. Nothing to renumber; these are design placeholders for later work.
- Batch 6's `PB-427`–`514` range has a real gap: only 427–437 and 500–514
  have actual files; 438–499 were never written (confirmed by unzipping
  both nested archives — no additional files hidden inside). Not a
  collision, just an incomplete range inherited as-is.

## D. Still open — needs your decision, not mine
**Host aliases.** Batch 1 uses `eriduous_vdi` (wrong separator vs. the real
`eriduous-vdi`), batches 4/5 use `dubsar_workstation`, batch 3 and batch 6
independently use `uruk`/`kish` for a two-host topology that doesn't exist
in the real `ansible/inventory.ini` yet. My working guess (Uruk = your
current box, Kish = a second machine) was corroborated across two
independent batches, but I can't confirm it without you. Please tell me:
is `uruk`/`kish` the name you want the real inventory to adopt (replacing
or supplementing `eriduous-vdi`), or should all of these just be corrected
to the existing `eriduous-vdi` alias?
