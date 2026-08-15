# Phase 2 Incoming — Raw Preservation Staging Area

**Status: STAGED, UNRECONCILED. Nothing in this tree has been integrated,
renumbered, or verified to run. Do not treat any playbook or doc ID here
as authoritative until it has passed the reconciliation pass (see the
7-step integration plan tracked in this session) and landed in its real
home (`docs/`, `playbooks/`, or a prototypes/reference directory).**

## Why this directory exists

Between 2026-08-08 and 2026-08-14 the Architect (Bahaa Fadam, DUB.SAR 𒁾)
worked across seven separate Claude conversations — while recovering from
eye surgery — designing Phase 2 of BahyWay.Ecosystem v4.0: CompareEngine's
Jaccard-based comparison design, the Šala prototype workbench (~90 HTML
rehearsal tabs), a large family of sealed-concept law/GL documents, and
well over 100 numbered playbooks across several independent numbering
ranges (PB-301, PB-310–322, PB-330–401, PB-420–437, PB-500–530).

This directory is step 1 of the integration plan: every file from all
seven upload batches is copied here **byte-for-byte, unmodified**, so the
Architect's work is safely in git history before any reconciliation,
renumbering, or editing happens. If anything goes wrong in later steps,
the originals are always recoverable from here (or from git history).

## Batches

| Directory | Source thread topic | Files |
|---|---|---|
| `batch1_compareengine_jaccard_flight/` | Jaccard index for CompareEngine; BIGRING 3D + Hubble + StoryEngine viz; Flight-to-Location (Nabû/NaviEngine/Hubble); PB-301 | 11 |
| `batch2_pdm_orbit_selection/` | Interactive orbit selection in particle viz; DubSar PDM (Pattern Data Modeler, native C++/GLSL Vulkan prototype); PB-160 recovery; **PB-185–200 law-seal suite** | 59 |
| `batch3_streaming_pu_ctg_pb310_320/` | Streaming data design; PU (Particles Unit) + CTG metrology; Qishtu/Zibānītu/Barūtu law tablets; **PB-310–320 Continuity/Storage/Two-Streams suite** (Uruk/Kish topology) | 48 |
| `batch4_unified_algebra_kidinnu/` | Fire-disaster civil protection chain (Δr Observatory → Contested Sky → Fadam Verdict → Fire Gravity → HeptaMap Refuge → Kidinnu Standard); Zibānītu A2 calculus; **PB-321 (Kidinnu Engine)** | 8 |
| `batch5_unified_algebra_theorem_sasu/` | Unified Algebra Theorem (53-member Algebra Arsenal), living orbits, Sasu Orbit Workbench; **PB-321 (Arsenal Inventory Survey) — collides with batch 4's PB-321**; PB-322 (deploy Šala v4) | 6 |
| `batch6_membrane_traffic_pb420_530/` | Membrane-flux/traffic/medical-sensor engines (Igigi Watch, Parzu Tremor Watch, Sila Grid, Asalluhi, Hendursaga); Algebra Manifesto/Manual; **PB-420–437 and PB-500–530 suites** (56 playbooks); a third `GL-NAV-001` claim | 94 |
| `batch7_silo_visualization_realm/` | Silo malfunction prediction → Visualization Realm map (Bench Membrane, Sealed Submission, Earned Assertion, Compression Gate); medical-sector expansion (Ninisina Engine, Living Anatomy); naṣāru/BWVL visual-language law family (GL-VIZ-000–006); **PB-330/338/339 + PB-360–401 suites**; nested `GulaFederation_PB-321_326.zip` — **a third claim on PB-321** | 77 |

Full per-batch file listing: see git history of this commit, or run
`find docs/phase2-incoming -type f | sort`.

## Known conflicts (resolved in step 2/3 of the integration plan, not here)

- **PB-321** claimed three times: batch 4 (Kidinnu Engine), batch 5
  (Arsenal Inventory Survey), batch 7 (GulaFederation range 321–326).
- **GL-NAV-001** claimed three times: batch 1 (flight-to-location),
  batch 2 (knowledge-graph-navigation), batch 6 (Hendursaga Charter Annex A).
- **PB-185–200** (batch 2) collide with unrelated, already-committed
  playbooks of the same numbers already in `playbooks/`.
- Host aliases are inconsistent across batches and do not match the real
  `ansible/inventory.ini` (`eriduous-vdi` / `enkidb-node-write` /
  `enkidb-node-read`): batch 1 uses `eriduous_vdi` (wrong separator),
  batch 3 introduces `uruk`/`kish`, batch 4/5 introduce `dubsar_workstation`.
  Batch 6's `PB-504_uruk_kish_weir.yml` and `PB-505_lahmu_lahamu_heartbeat.yml`
  corroborate batch 3's Uruk/Kish naming as intentional, not accidental.

None of the above is fixed in this directory. Step 3 of the plan applies
the Architect-approved resolutions; this directory stays as the untouched
original record.
