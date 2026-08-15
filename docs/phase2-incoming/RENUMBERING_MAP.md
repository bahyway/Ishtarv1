# Phase 2 Renumbering Map — Step 3 (CONFIRMED by the Architect, 2026-08-14)

Supersedes the "proposals" in `CONFLICT_MAP.md` — everything below is the
authoritative mapping Steps 4-6 will apply when landing files into their
real homes (`docs/`, `playbooks/`, prototype reference area). The raw
batches in `docs/phase2-incoming/batchN_.../` are left untouched; this map
is what tells later steps what to rename each file to on the way out.

## GL-doc IDs

| Source | Original name | Final ID | Notes |
|---|---|---|---|
| batch 1 (embedded in `pb-301`'s `copy:` task) | GL-NAV-001 — Flight-to-Location | **GL-NAV-001** | Extracted to its own `docs/GL-NAV-001-flight-to-location.md` at Step 4 (matches the PB-197+ convention: docs are committed files, not Ansible-copied to a remote HOME path). |
| batch 6 | `GL-NAV-001_Hendursaga_Charter_AnnexA_DRAFT.md` | **GL-NAV-001, Annex A** | Filed as the literal Annex A of the law above — same domain (Wādī al-Salām field architecture = the NajafEngine instance of Flight-to-Location). |
| batch 2 | `GL-NAV-001-knowledge-graph-navigation.md` | **GL-NAV-002** | Distinct topic (NabuEngine docs-as-orbits). Its "PB-184 recovery" provenance claim doesn't match the real `playbook_184` in this repo — flagged to the Architect in Step 2, unresolved, carried forward as a note on the landed doc rather than blocking the renumber. |
| batch 7 | `GL-VIZ-001_Morphological_Discovery_DRAFT.md` | **GL-VIZ-007** | Real `docs/GL-VIZ-001.md` (BUZU, sealed+proven in Rust) is untouchable. |
| batch 7 | `GL-VIZ-002_Shape_Verdict_DRAFT.md` | **GL-VIZ-008** | Extends the doc above; renumbered together to preserve the 007→008 "extends" adjacency. |
| batch 2 | `GL-VIZ-002-orbit-witness-isolation.md` | **GL-VIZ-002** (unchanged) | No collision with the real repo; keeps its number. |

All other GL-doc IDs across all 7 batches (GOV-001/002/003, HS3-001/002,
KAKI-002, MEM-001, STD-002, TOOL-001, FOR-001, TKT-001, TPL-001/002,
STY-001, ORG-001, MDM-001, DST-002/003, DDB-001/004, and the DDB-002/003 +
MED-001 Annex pairs) land unchanged.

## Playbook numbers

| Source | Original | Final | Notes |
|---|---|---|---|
| batch 4 | `pb-321-kidinnu-engine.yml` | **PB-321** (unchanged) | Strongest sequential claim (directly follows the PB-310-320 suite). Host fixed `dubsar_workstation` → `uruk`. |
| batch 5 | `PB-321-arsenal-inventory-survey.yml` | **PB-531** | |
| batch 5 | `PB-322-deploy-shala-v4.yml` | **PB-532** | Kept paired after 531 to preserve batch 5's own internal 321→322 sequence. |
| batch 2 | `pb-185` … `pb-200` (16 files) | **PB-533 … PB-548** | Same internal order preserved. Collided with unrelated, already-merged `playbook_185`-`playbook_200`. |
| batch 7 (GulaFederation) | `PB-321` … `PB-326` (6 files) | **PB-549 … PB-554** | Internally consecutive suite shifted as a block. |
| batch 1 | `pb-301-flight-to-location-law-seal.yml` | **PB-301** (unchanged) | Host fixed `eriduous_vdi` (wrong separator) → `uruk`. |
| batch 3 | `pb310` … `pb320` (11 files) | **PB-310 … PB-320** (unchanged) | Already uses its own `uruk`/`kish` inventories natively — merge with real `ansible/inventory.ini` at Step 5, don't renumber. |
| batch 2 | `PB-160-tpl-001-section-e-RECOVERY.yml` | **PB-160-RECOVERY** (companion, not a renumber) | Genuine continuation of the real `playbook_160_tpl_001_section_e_corrected.yml`; lands as a follow-up playbook, not a replacement. |
| batch 6 | `PB-420` … `PB-437`, `PB-500` … `PB-530` (49 files) | unchanged | No collisions found. |
| batch 7 | `PB-330`, `PB-338`, `PB-339` | unchanged | No collisions found. |
| batch 7 | `PB-360`–`374`/`380`–`389`/`390`–`393`/`394`–`397`/`398`–`401` (suite-description docs only) | unchanged | No individual files exist yet at these numbers — nothing to renumber. |

## Host aliases (repo change, applied now)

`ansible/inventory.ini` updated: added `uruk` (`ansible_connection=local`)
as the canonical alias for the bare-metal Fedora Workstation 44 box, per
the Architect's confirmation that `eriduous-vdi`/VDI is fully retired.
`eriduous-vdi` is kept, unchanged, alongside it — pointing at the same
local connection — so the ~89 pre-Phase-2 playbooks and the CI workflow's
self-hosted-runner label (`.github/workflows/isimud-engine.yml`,
`runs-on: [self-hosted, eriduous-vdi]`) keep working without a repo-wide
rename. That full rename is a separate, larger decision (it also requires
re-registering the physical GitHub Actions runner under a new label on
GitHub's side, which this repo cannot do) and is left for the Architect to
carry out explicitly if/when wanted, not applied here as a side effect.

`kish` is reserved but **not yet defined** with real connection details —
no second machine exists yet. Playbooks targeting `hosts: kish` (the
PB-310-320 suite's promotion/two-stream playbooks, `PB-504_uruk_kish_weir`)
will fail cleanly (Ansible "could not match supplied host pattern") rather
than silently misrun, until the Architect adds a real `ansible_host` for it.

All Phase 2 playbooks landing from Step 5 onward target `hosts: uruk`.
