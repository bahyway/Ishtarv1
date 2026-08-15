# Shakkanakku PB Manual — Phase 2 Playbooks

Covers every playbook landed from `docs/phase2-incoming/` per
`docs/phase2-incoming/RENUMBERING_MAP.md`. All are **DRAFT / unsealed**
design work unless their own header says otherwise — running one is, per
CSR-08 doctrine, the Architect's act of confirming intent, not a
rubber-stamped deploy.

## How to run any of these safely

```bash
# Always dry-run first, from the repo root:
ansible-playbook playbooks/playbook_NNN_slug.yml --check

# Then, once satisfied:
ansible-playbook playbooks/playbook_NNN_slug.yml
```

The default inventory (`ansible/inventory.ini`, wired via `ansible.cfg`)
resolves automatically — no `-i` flag needed. All Phase 2 playbooks target
`hosts: uruk` (your Fedora Workstation 44 box), `hosts: localhost`, or the
pre-existing `enkidb-node-write`/`enkidb-node-read` VMs — **except** the
ones listed under "Not yet runnable" below, which name infrastructure that
doesn't exist in this ecosystem yet and will fail cleanly with Ansible's
"could not match supplied host pattern" until it's provisioned and added
to the inventory.

**2026-08-15 silent-error audit — fixed, not just flagged.** A full pass
found and fixed everything below before any of these playbooks are meant
to be run for real:
- **PB-533–541, 544–548** (9 files) had a stale `ansible.builtin.copy`
  task that would have written a duplicate law document to
  `$HOME/bahyway/docs/...` instead of the real, already-committed `docs/`
  copy. **Worse**, five of them (**544–548**, `GL-PHY-001`, `GL-PAT-001`,
  `GL-PAT-002`, `GL-DB-001`, `GL-DOC-001`) turned out to have **no
  standalone `docs/` file at all** — their law text existed only inside
  the playbook's embedded copy, missed in the original Step 4 landing.
  All 14 were extracted (the missing 5 committed for the first time),
  then converted to pointer-style playbooks like `playbook_301` — they
  now target `hosts: localhost` and just confirm the doc is in `docs/`.
- **PB-339, 549–554** (7 files) defined `bahyway_root:
  {{ ansible_env.HOME }}/BahyWay.Ecosystem` — a path that doesn't exist.
  Running them would have silently scaffolded new Rust crates in an
  orphaned directory tree, never wired into the real Cargo workspace and
  never caught by `cargo test --workspace`. Fixed to
  `{{ playbook_dir }}/../workspace/bahyway_v4`, the real workspace root.
- **PB-531** guessed `$HOME/BahyWay-Ecosystem` as its default `repo_root`
  — already guarded by an explicit fail-if-missing check (not silent),
  but fixed to default to `{{ playbook_dir }}/..` so it works without
  needing `-e repo_root=...` on every run.
- **PB-543** pointed `nl_doc`/`catalog_file` at `$HOME/bahyway/docs/...`
  paths that don't exist and don't match the real NL-001 doc's actual
  name or location. Fixed to the real `docs/NL-001-A1.md` and
  `docs/catalog/CAT-001-index.md` — this one now correctly amends files
  this session has been editing directly all along.

All fixes verified: every playbook YAML re-parses cleanly (274 files),
and the 5 newly-discovered docs are now in `docs/catalog/CAT-001-index.md`
and `docs/BAHYWAY_PHASE2_GLOSSARY.md`.

## Runnable now — target `uruk` or `localhost`

| PB | File | What it does |
|---|---|---|
| 160 | `playbook_160_tpl_001_section_e_RECOVERY.yml` | Recovery/continuation companion to the existing `playbook_160_tpl_001_section_e_corrected.yml`. |
| 301 | `playbook_301_flight_to_location_law_seal.yml` | Pointer confirming `GL-NAV-001` (Flight-to-Location) landed in `docs/`. |
| 321 | `playbook_321_kidinnu_engine.yml` | Builds `kidinnu-engine` (civil-protection minimax evacuation directive). **Blocked** on two things regardless of host: the `Kidinnu` name itself is still PROPOSED/unsealed (NL-001), and its own header defers implementation behind the PB-150..PB-160 testing gate. |
| 322 | `playbook_322_ontograph_scaffold.yml` | Verifies the OntoGraph crate (`crates/ontograph`, GL-ONT-001, Unified Pattern "Nebuchadnezzar") is landed and its 4 law tests pass. Real, already-verified crate — `cargo test -p ontograph` confirmed clean during landing, not just at playbook run time. |
| 339 | `playbook_339_parzu_case_particle.yml` | Parzu case-particle scaffolding. `bahyway_root` path fixed 2026-08-15. |
| 420, 421, 426–431, 436, 437 | field-core scaffold, tile pipeline, registry bridge, Šala tablet vault, VGCA/catenoid crates, Asalluhi/Hendursaga crates, invoice-datum service, finalization gate | Membrane/traffic engine crate scaffolding (batch 6). |
| 500, 501, 507–509, 517, 527–530 | Igigi Watch core, Bells service, Kittu alert wiring, Shakkanakku chronicle sink, watch UI deploy, Sila Grid crate, HeptaMapSpace renderer, uniqueness-reach lens, HeptaScript ext cluster, traffic arc gate | Sensor/watch/rendering services. |
| 531, 532 | Arsenal Inventory Survey, Deploy Šala v4 | Read-only Algebra-Arsenal scan (`repo_root` default fixed 2026-08-15); Šala tablet deployment (`hosts: localhost`). |
| 533–541, 544–548 | law-seal suite, now pointer-style (orbit-witness-isolation, Tupsimati wizard, Madanu court, pattern-minting template, ticket law, StoryEngine ontology, harmonization survey, homeostasis, living-shape drift, physics-service, foreign-pattern-quarantine, pattern-maturation-delivery, no-false-authority, single-glossary) | Renumbered from the colliding PB-185–200 block. Fixed 2026-08-15: converted to pointer-style (`hosts: localhost`), 5 missing docs (544–548) extracted and committed for the first time. |
| 542, 543 | Girsu Vulkan classroom; Girsu extension naming seal | Both `hosts: localhost`/`uruk` as designed. 543's `nl_doc`/`catalog_file` paths fixed 2026-08-15 to the real `docs/NL-001-A1.md` and `docs/catalog/CAT-001-index.md`. |
| 549–554 | GulaFederation suite (advisory API, synthetic Baghdad dataset, Godot mobile Hubble scaffold, offline OSM tile bundler, signed advisory verifier, medicine-batch audit) | Renumbered from the colliding PB-321–326 block. `hosts: localhost`. `bahyway_root` path fixed 2026-08-15. |
| 555 | Hala naming-correction seal (verifies the Uruinimgina→Hala rename landed correctly, `cargo check -p shakkanakku` clean) | `hosts: localhost`. Seals the naming decision only — the proposed cuneiform glyph (𒄩𒆷) stays DRAFT pending verification. |
| 556 | Deploy bahyway.com/beemdm.com/heptascript.com on `uruk` via nginx | `hosts: uruk`, needs `become: true`. HTTP-only by default; pass `-e request_tls=true` only once DNS + port 80/443 reachability are confirmed (see the playbook's own header for exactly what that requires). All three sites' Google Fonts CDN dependency removed 2026-08-15. |
| 557 | Production go-live: redeploy CQRS core from `master` | `hosts: localhost`. **The missing "next step" `scripts/otap-promote.sh` names but never built** — closes the gap between "accepted in git" and "actually serving." Refuses without `-e i_understand_this_is_production=true`; verifies local HEAD == `origin/master`; re-runs `cargo test --workspace`; redeploys via the real `playbook_212` (not reimplemented); witnesses the go-live in `docs/SHEDU/NARU_AUDIT_JOURNAL.md`. Does **not** chain `playbook_284`/`playbook_556` — run those explicitly if this release touches the dashboard or websites. |

## Runnable if the existing write/read-node VMs are up

Targets the **pre-existing**, already-defined `enkidb-node-write`
(192.168.122.101) or `enkidb-node-read` (192.168.122.107) hosts — not
blocked by missing inventory, just requires those VMs reachable over SSH
as they already needed to be for ~89 pre-Phase-2 playbooks.

`PB-432`–`435`, `502`, `503`, `505`, `506`, `510`–`516`, `518`–`526`
(dashboard census, conservation-delta audit, seven-gates enforcement,
blackbox-cycle wiring, Lamassu cadence daemon, seismograph drill,
Lahmu–Lahamu heartbeat, backpressure ladder, Lamassu-sweep/Enlil-
reconsecration/snapshot-partition/Nuzi-prune/chaos-drill rites, vineyard
extent/harvest, maxpressure scheduler, OSM province ingest, arterial
template mint, typed census, scenario engine, cell-transmission model,
MFD perimeter control, hotspot detection, parking assignment) — and
`PB-504` (`uruk_kish_weir`, targets `enkidb-node-read`).

## Not yet runnable — target infrastructure that doesn't exist yet

These name hosts that have no entry in `ansible/inventory.ini` and will
fail cleanly rather than silently misrun. Each needs the Architect to
provision the real machine/role and add it to the inventory first.

| Host alias | Playbooks | What's missing |
|---|---|---|
| `vault_librarian` | 314 | A dedicated vault-librarian VM (mentioned in the PB-310-320 suite's own README as part of its topology) — doesn't exist. |
| `vault_body` | 318 | The ZFS vault-body pool host — likely intended to be `uruk` itself (local ZFS pool) rather than a separate machine; needs Architect confirmation. |
| `nas_vault` | 330, 338 | A NAS device (7×5TB, per the suite's README) — not yet acquired/provisioned. |
| `host_forge` | 313, 315, 316, 317, 319, 320 | Ambiguous — likely also meant to be `uruk`, but landed under a distinct alias; needs Architect confirmation before merging with `uruk`. |
| `write_node` / `read_node` | 310–312 | Likely meant to be the existing `enkidb-node-write`/`enkidb-node-read`, but use a different alias spelling than what's actually in the inventory — needs confirmation before merging. |
| `najaf_base_station` | 422 | An RTK GPS base station (physical hardware) — not yet acquired. |
| `najaf_field_sbc` | 423, 424, 425 | A field single-board-computer image for GPR anomaly survey work — not yet acquired. |

## `kish` — defined but not connectable

**Verified in Step 7**: none of the 90 landed playbooks actually target
`hosts: kish` directly today (315/316 target `host_forge`; `kish` shows up
in their variables/prose, not their `hosts:` line) — so this note is
precautionary for when Kish-targeting playbooks are written, not a live
issue right now. `kish` resolves the *name* (added to
`ansible/inventory.ini` in Step 3) but cannot connect — no `ansible_host`
is set yet, since the second machine doesn't exist. This is intentional:
failing cleanly beats silently running nothing,
per the PB-210 lesson already learned once in this repo.

## Suite-level reference docs (not individually numbered playbooks)

`PB-360-374_naṣāru_BWVL_Playbook_Suite_DRAFT.md`,
`PB-380-389_Mašḫalu_Playbook_Suite_DRAFT.md`,
`PB-390-393_Sealed_Submission_Playbook_Suite_DRAFT.md`,
`PB-394-397_Earned_Assertion_Playbook_Suite_DRAFT.md`,
`PB-398-401_Gate_and_Bench_Membrane_Playbook_Suite_DRAFT.md` — these
describe *planned* playbook ranges; no individual `.yml` files exist yet
for these numbers. `PB_REGISTRY_427-514_DRAFT.md`,
`PB-310-320_SUITE_README.md`, `PB-310-320_SEAL_REGISTER.md`, and
`PB-549-554_GulaFederation_MANIFEST.md` are the suites' own planning/seal
records, landed alongside their playbooks for reference.
