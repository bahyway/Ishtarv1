# AnuGovernor PB Manual — Phase 2 Playbooks

Covers every playbook landed from `docs/phase2-incoming/` per
`docs/phase2-incoming/RENUMBERING_MAP.md`. All are **DRAFT / unsealed**
design work unless their own header says otherwise — running one is, per
CSR-08 doctrine, the Architect's act of confirming intent, not a
rubber-stamped deploy.

## PB-609 — the automated run-manifest (2026-08-23, rewritten same day)

Now that the Uruk 3-VM CQRS+vault topology is confirmed live,
**`playbooks/playbook_609_run_manifest.yml`** walks the 109-playbook
backlog this manual describes below (the freshly-repaired PB-310-320
suite, the "Runnable now" table, and the "Runnable if write/read VMs
up" table). Per the Architect's explicit instruction, it runs
**unattended** — one confirmation for the whole walk, then `--check`
and LIVE automatically for every item, no per-item prompt. A failure
is journaled and the walk continues by default
(`-e stop_on_error=true` to halt on the first one instead). CSR-08
still governs the two playbooks inside it that require a human act
regardless (`playbook_313`'s typed seal, `playbook_557`'s production
confirmation flag) — those correctly refuse either way.

The backlog (`playbooks/data/run_manifest_609_backlog.yml`) is sorted
in **strict ascending PB-number order**, not the three tables'
original presentation order — that's what caused the walk to visibly
jump (PB-570 → PB-430 → PB-502) on its first real run. Checked for
real dependencies (preflight/chain-gate references between backlog
items, not guessed): only two exist, `playbook_437` requires
`PB-427..436` journaled first and `playbook_530` requires
`PB-515..529` — both cite ranges below their own number by
construction, so ascending order satisfies both automatically. The
other ~106 items are independent. A full renumbering (renaming every
file) was deliberately not done — near-zero functional benefit for
items with no real dependency on each other, against real risk to the
many cross-references already fixed this session.

The first real run (77/109 green, 33 failed) surfaced and fixed a full
round of genuine bugs: a `vars_files` loading gap across 8 of the
PB-310-320 files, two relative-path bugs, a missing-`chdir` bug in 13
crate-scaffold playbooks (silent in 9 of them — nothing downstream
checked, so they "succeeded" while scaffolding into the wrong
directory), a `--check`-mode simulation gap in 6 more, two Jinja2
self-reference infinite recursions, a wrong package name, and —
the single biggest finding — **all 25 "Runnable if VMs up" items were
false-positive successes**: `hosts: enkidb-node-write`/`-read` are
retired literal hostnames that no longer resolve, so every one of
those 25 playbooks silently matched zero hosts and ran nothing, the
exact PB-210 anti-pattern this repo already fought once. Retargeted to
the real `uruk-node-write`/`-read`. Full detail in the commit history;
re-run to see the real result now.

Excluded on purpose: the already-green Gula chain (`playbook_602`–`608`),
`playbook_318` and the Najaf hardware suite (still blocked on
unacquired physical hardware), and the six undrafted suites
(`PB-360-374`, `380-409`) that have no `.yml` files yet. Run with:
```bash
ansible-playbook playbooks/playbook_609_run_manifest.yml
```

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
  name or location. Fixed to the real `docs/02_identity/NL-001-A1.md` and
  `docs/99_index/CAT-001-index.md` — this one now correctly amends files
  this session has been editing directly all along.

All fixes verified: every playbook YAML re-parses cleanly (274 files),
and the 5 newly-discovered docs are now in `docs/99_index/CAT-001-index.md`
and `docs/99_index/BAHYWAY_PHASE2_GLOSSARY.md`.

## Runnable now — target `uruk` or `localhost`

| PB | File | What it does |
|---|---|---|
| 160 | `playbook_160_tpl_001_section_e_RECOVERY.yml` | Recovery/continuation companion to the existing `playbook_160_tpl_001_section_e_corrected.yml`. |
| 301 | `playbook_301_flight_to_location_law_seal.yml` | Pointer confirming `GL-NAV-001` (Flight-to-Location) landed in `docs/`. |
| 321 | `playbook_321_kidinnu_engine.yml` | Builds `kidinnu-engine` (civil-protection minimax evacuation directive). **Blocked** on two things regardless of host: the `Kidinnu` name itself is still PROPOSED/unsealed (NL-001), and its own header defers implementation behind the PB-150..PB-160 testing gate. |
| 322 | `playbook_322_ontograph_scaffold.yml` | Verifies the OntoGraph crate (`crates/ontograph`, GL-ONT-001, Unified Pattern "Nebuchadnezzar") is landed and its 4 law tests pass. Real, already-verified crate — `cargo test -p ontograph` confirmed clean during landing, not just at playbook run time. |
| 339 | `playbook_339_parzu_case_particle.yml` | Parzu case-particle scaffolding. `bahyway_root` path fixed 2026-08-15. |
| 420, 421, 426–431, 436, 437 | field-core scaffold, tile pipeline, registry bridge, Šala tablet vault, VGCA/catenoid crates, Asalluhi/Hendursaga crates, invoice-datum service, finalization gate | Membrane/traffic engine crate scaffolding (batch 6). |
| 500, 501, 507–509, 517, 527–530 | Igigi Watch core, Bells service, Kittu alert wiring, AnuGovernor chronicle sink, watch UI deploy, Sila Grid crate, HeptaMapSpace renderer, uniqueness-reach lens, HeptaScript ext cluster, traffic arc gate | Sensor/watch/rendering services. |
| 531, 532 | Arsenal Inventory Survey, Deploy Šala v4 | Read-only Algebra-Arsenal scan (`repo_root` default fixed 2026-08-15); Šala tablet deployment (`hosts: localhost`). |
| 533–541, 544–548 | law-seal suite, now pointer-style (orbit-witness-isolation, Tupsimati wizard, Madanu court, pattern-minting template, ticket law, StoryEngine ontology, harmonization survey, homeostasis, living-shape drift, physics-service, foreign-pattern-quarantine, pattern-maturation-delivery, no-false-authority, single-glossary) | Renumbered from the colliding PB-185–200 block. Fixed 2026-08-15: converted to pointer-style (`hosts: localhost`), 5 missing docs (544–548) extracted and committed for the first time. |
| 542, 543 | Girsu Vulkan classroom; Girsu extension naming seal | Both `hosts: localhost`/`uruk` as designed. 543's `nl_doc`/`catalog_file` paths fixed 2026-08-15 to the real `docs/02_identity/NL-001-A1.md` and `docs/99_index/CAT-001-index.md`. |
| 549–554 | GulaFederation suite (advisory API, synthetic Baghdad dataset, Godot mobile Hubble scaffold, offline OSM tile bundler, signed advisory verifier, medicine-batch audit) | Renumbered from the colliding PB-321–326 block. `hosts: localhost`. `bahyway_root` path fixed 2026-08-15. |
| 555 | Hala naming-correction seal (verifies the Uruinimgina→Hala rename landed correctly, `cargo check -p anu-governor` clean) | `hosts: localhost`. Seals the naming decision only — the proposed cuneiform glyph (𒄩𒆷) stays DRAFT pending verification. |
| 556 | Deploy bahyway.com/beemdm.com/heptascript.com on `uruk` via nginx | `hosts: uruk`, needs `become: true`. HTTP-only by default; pass `-e request_tls=true` only once DNS + port 80/443 reachability are confirmed (see the playbook's own header for exactly what that requires). All three sites' Google Fonts CDN dependency removed 2026-08-15. |
| 557 | Production go-live: redeploy CQRS core from `master` | `hosts: localhost`. **The missing "next step" `scripts/otap-promote.sh` names but never built** — closes the gap between "accepted in git" and "actually serving." Refuses without `-e i_understand_this_is_production=true`; verifies local HEAD == `origin/master`; re-runs `cargo test --workspace`; redeploys via the real `playbook_212` (not reimplemented); witnesses the go-live in `docs/16_runbooks/NARU_AUDIT_JOURNAL.md`. Does **not** chain `playbook_284`/`playbook_556` — run those explicitly if this release touches the dashboard or websites. |
| 558 | Land EriduScaffold (OTA ground scaffolder + STATUS.md ledger + honest-dogfooding build-DAG scheduler) | `hosts: uruk`, `become: false`. Installs `bahyway.sh`/`eridu.sh` to `~/BahyWay`, executable, not auto-run (first run is the Architect's own act). `-e compile_build_dag=true` also copies and `rustc -O`-compiles `eridu_build_dag.rs` (verified clean-compiling and correct this session). Landed from a parallel Claude session's "Interactive orbit selection" thread; full source + 14 Šala prototypes + evaluation README at `shala-prototypes/batch9_eridu_scaffold/`. Required freeing the name "Eridu" from "EriduOS" first — see that batch's README for the rename (commit `9024000`). Deliberately does not provision any database/container or wire the Stage-1 EnkiSDB-emit hook — out of EriduScaffold's own declared scope. |
| 559 | Register the Šala Hub (`shala_hub_index.html`) as the library's front door — rehearsal AND learning tool | `hosts: localhost`, `become: false`. Pointer/verification-style: PROVEs the already-landed Hub (`shala-prototypes/batch2_pdm_orbit_selection/`) is a 7-act cinematic teaching argument (grep-checks its "usual way"/APSU/Observatory-gate/"one cause, one ticket"/keyboard-nav content), PROVEs both "Enter the Observatory" gate links resolve to real sibling files (MARDUK datamine, PDM modeler), and PROVEs `INDEX.md` + `SHALA-DESIGN-CHARTER.md` §1a document the correction. Deliberately does **not** relocate the file — its relative `href`s would break (the `file://` path lesson learned during EriduScaffold's landing, this session). Realizes the BahyWayAcademy insight (`shala-prototypes/batch9_eridu_scaffold/04_BahyWayAcademy.md`) without loosening Way-of-Work rule 5. |
| 560 | Seal the Nimrud Stack — the four-member (WITNESS/JUDGE/MOVE/STAGE) water-leak judgment-and-action arc (GL-STK-001) | `hosts: localhost`, `become: false`. Renumbered from a parallel Claude session's PB-353. PROVEs the six law clauses (members/verbs, born-particles-only supply, Adapa+OntoGraph naming, frozen constants, no-self-minted-negatives, PCA-never-localizes) and the Šala court's unborn refusal records (Nebuchadnezzar, Enkido, the trained classifier). Cross-references `crates/wpd-engine` — a real, tested crate this repo already has that the source session didn't know about. Full detail and renumbering table at `shala-prototypes/batch10_nimrud_stack/README.md`. |
| 561 | Seal the Nimrud Observatory — twelve W5H2 verdict indicators, each epistemically stamped (GL-NIM-001) | `hosts: localhost`, `become: false`. Renumbered from PB-354. PROVEs the four-class Epistemic Stamp Law (MEASURED/DERIVED/ESTIMATED/ADVISED, never rendered as a higher class than earned), all twelve indicators, the FAVAD frozen-constants clause, and the Šala observatory's REHEARSAL banner plus its DIRECT CRACK OBSERVATION refusal record. |
| 562 | Seal Two Witnesses of Place — the derived-depth {7,3} hepta-address for interring a leak verdict (GL-NIM-002) | `hosts: localhost`, `become: false`. Renumbered from PB-355. PROVEs both witnesses of place (UTM truth + hepta-address, never in the locked KAKI bytes), the derived address-depth formula (d\* from σ_d, rehearsal figure 11 shells), deterministic cell ownership (EEEngine explicitly not summoned), and the Šala descent court's false-precision lock. **Cites GL-MAP-001 and PB-347, neither of which is landed in this repo** — see the batch README. |
| 563 | Seal the Nimrud Notebook — the read-only, four-view HeptaScript interrogation console (GL-NTB-001) | `hosts: localhost`, `become: false`. Renumbered from PB-356. PROVEs the HeptaScript-only language law (SQL refused at parse, not errored), stage-never-truth with EMIT disabled in rehearsal, the hepta-address×epoch join key, epistemic-class preservation across views, and the Šala notebook's four view cells plus its one SQL refusal cell. |
| 564 | Land the TID/FCA/EAV Unified Pattern batch — bacteriology worked example, nine prototype iterations | `hosts: localhost`, `become: false`. Pure rehearsal/research landing (no GL law tablet — the source thread proposed none). PROVEs the full 16-file manifest and spot-checks the documented arc: a live ε-threshold Vietoris-Rips demo, breadcrumb hierarchical zoom, a buggy Three.js/OrbitControls lattice, and the final hand-rolled 3D deep-zoom engine that dropped the OrbitControls dependency entirely. Also lands an enhanced navigable evolution of the already-landed `sala-realm-map.html`. Full detail at `shala-prototypes/batch11_tid_fca_eav_bacteriology/README.md`. |
| 565 | Seal the Book Court Law — literature read as structured corpus, two-witness mint-then-serve, structure-only (GL-LIT-001) | `hosts: localhost`, `become: false`. Renumbered from a parallel Claude session's PB-325. PROVEs the five rites (Stage/Lattice/Pick/Mint with Provenance/Gate Before Service), the two-witness clause applied to mathematics, and the structure-only (no prose stored) clause. Also lands two corpus registrations: a Dorst-Fontijne-Mann Geometric Algebra rehearsal and a Spieksma/Spreij stochastic-calculus shelf with explicit bindings into GL-NSR-001 and GL-ALG-003. Full detail at `shala-prototypes/batch12_nasaru_saturday_courts/README.md`. |
| 566 | Seal the Abūbu Calculus — membrane rupture: compliance, critical density, horizon, EnkiQDB quarantine (GL-ALG-003) | `hosts: localhost`, `become: false`. Renumbered from PB-327. PROVEs the tablet's own six law tests L1-L6, the compliance bounds (κ_min=0 Rigid Decree, κ_max=1 Yield Normalization), and the default ε_Q=0.05 quarantine threshold. Hosted under NinurtaEngine's truth tier per GL-PHY-001; the Šala court is theater-tier only (GL-DST-001). |
| 567 | Seal the IshumEngine Founding Tablet — offline cemetery navigation: skeleton, filtration, constellation, pack, route (GL-ISM-001) | `hosts: localhost`, `become: false`. Renumbered from PB-329. PROVEs the tablet's own four law tests L1-L4 and all five organs, including the refusal-buffers-as-walls clause. Consumes GL-NJF-001's grave layer and never contradicts it. |
| 568 | Seal the Nasaru Alert Law — MARKASU-01, the mooring model, Nasaru's first alert on motion (GL-NSR-001) | `hosts: localhost`, `become: false`. Renumbered from PB-330. PROVEs the tablet's own four law tests L1-L4, the Ornstein-Uhlenbeck mooring model, and the two-witness (autocorrelation + variance) PROVE-form — one witness alone journals FUZZY, never rings the bell. |
| 569 | Seal the Temennu Baseline & Rigmu Escalation — per-tribe founding, LEVEL/TREND witnesses, GOLDEN-tribe great cry (GL-NSR-001-A1) | `hosts: localhost`, `become: false`. Renumbered from PB-331. Amends PB-568/GL-NSR-001. PROVEs the tablet's own four law tests L5-L8, the Temennu immutability clause (automatic re-baselining forbidden), and the Rigmu freeze-and-explain obligation (Gate G4 halt, no silent close). |
| 570 | Seal the Rigmu Inquest Doctrine — mandatory W5H2 investigation on every great cry, five HeptaScript operations (GL-NSR-001-A2) | `hosts: localhost`, `become: false`. Renumbered from PB-332. Amends PB-569/GL-NSR-001-A1. PROVEs the seven-field W5H2 explanation particle (missing any field refuses the mint) and the five sovereign operations (ORBIT/PROVE/WITNESS/EMIT/SYNC) — no SQL exists here and none may be smuggled in. |
| 571 | Seal the Najaf Grave Court — sanctity capacity, Qibla-erosion lawful vacancy, the dignity refusal clause (GL-NJF-001) | `hosts: localhost`, `become: false`. Freshly numbered — no source PB file was ever delivered for this tablet (GL-NJF-001 §7 cites "PB-328" in prose only). PROVEs the sanctity-capacity countdown, the Qibla-erosion construction with orientation fixed as a constant, and the GOLDEN/FUZZY/DEAD refusal ladder (an unverifiable old grave becomes a protective buffer, never an erasure). |
| 572 | Land the remaining Saturday rehearsal suite + provenance record — Nasaru Leak Court v1/v2, Sixth Court, Membrane Court v3, OntoGraph conversation, week-transcript | `hosts: localhost`, `become: false`. Pure rehearsal/research landing (no GL law tablet). PROVEs the six-file manifest and flags **GL-ALG-002** ("𝒟_Θ deficit"), cited by both Nasaru Leak Court builds but confirmed absent repo-wide — the playbook fails loudly if GL-ALG-002 is later landed without this note being updated. Also preserves the design conversation behind the already-landed GL-ONT-001/PB-322 and the source session's own full 9,661-line week-provenance transcript. Full detail at `shala-prototypes/batch12_nasaru_saturday_courts/README.md`. |
| 573 | Seal the Shape & Pattern Court — the Bell Veto: approve/release physically disabled while MARKASU or RIGMU rings, regardless of steward signature (GL-DST-004) | `hosts: localhost`, `become: false`. Renumbered from a parallel Claude session's PB-334. PROVEs the tablet's own four law tests L13-L16, the Bell Veto clause (physically disabled, not warned), the Loss-Explanation Clause, and the no-silent-override path (Madanu decree with CSR-08 seal only). PROVEs the Šala court renders the nabalkutu register, both named harms (Curve Loss, Census Loss), and the decree row. Full detail at `shala-prototypes/batch13_sunday_shape_labiru_courts/README.md`. |
| 574 | Seal the Ṣalmu Registry — eight closed simplicial standard shapes, BWVL glyphs, the Apsu Vacancy Principle (GL-SHP-001) | `hosts: localhost`, `become: false`. Renumbered from PB-335. PROVEs the tablet's own four law tests L17-L20, all eight registry entries (RĪQU/EPERU/ŠIBIRTU/ḪARRĀNU/KIPPATU/MAŠTABBA/USKARU/KIRṢU), and the Apsu Vacancy Principle (a filled orbit center is a fabricated-interior finding, never a style). PROVEs the Šala gallery renders all eight forms. |
| 575 | Land the Story Membrane Court — 3D particle picker, StoryEngine journal, membrane scar-profile churn diagnosis | `hosts: localhost`, `become: false`. Pure landing record — no dedicated GL law tablet (serves the already-landed GL-STY-001/GL-MDM-001/GL-DST-004). Renumbered from PB-336. PROVEs the Ṣabātu bounce-select signal, the StoryEngine journal read, the scar-profile membrane, and the PING-PONG-names-the-missing-GL-MDM-001-decree verdict (confirmed a live cross-reference, not dangling — GL-MDM-001 is already landed at `docs/06_governance_parzu/`). Source YAML materializes a Rust crate via Ansible copy tasks onto `dubsar_workstation`; this playbook does not replicate that pattern — see the batch README's "Materialization note". |
| 576 | Seal the BWVL Verb Law — WITNESS/REHEARSE/PROPOSE, and the missing fourth verb APPLY that BWVL may never own (GL-VSL-001) | `hosts: localhost`, `become: false`. Renumbered from PB-337. PROVEs the tablet's own four law tests L24-L27, the no-APPLY law, the SEND-TO-BENCH termination, and the Shape Horizon T_shape (completing the T\*/T_θ countdown grammar). PROVEs the Šala theater renders all three verb families, the SIMULATION watermark, and the MEND repair ghost. |
| 577 | Seal the Labīru Doctrine — the mortality of GOLDEN, the write-once Origin Deposit, the Kīma Labīrīšu confrontation rite (GL-LBR-001) | `hosts: localhost`, `become: false`. Renumbered from PB-338 (this repo's own existing PB-338 is an unrelated concept — `sibittu_jail`). PROVEs the tablet's own four law tests L28-L31, the mortality-of-GOLDEN doctrine (world-drift + record-drift), the write-once Origin Deposit into NUZI, the CONCORD/LAWFUL EVOLUTION/SILENT DRIFT-as-RIGMU router, and the Honesty Clause against eternal-truth claims. |
| 578 | Seal the Migration Chapter — the Two Scribes Rite: kīma labīrīšu over warehouse migration validation via Merkle confrontation, never unit-testing the new code (GL-LBR-001-A1) | `hosts: localhost`, `become: false`. Renumbered from PB-339 (this repo's own existing PB-339 is an unrelated concept — `parzu_case_particle`). Amends PB-577/GL-LBR-001. PROVEs the tablet's named law tests L32-L36, the four rites (seal the input / Two Scribes Merkle drill / pattern inquest via FCA closure / decree-or-bug), and the three-way per-procedure colophon. |
| 579 | Seal the Miṣru Rite — jugglers are born on contested rule boundaries; collapse is conditionally inevitable absent a harmonization decree (GL-LBR-001-A2) | `hosts: localhost`, `become: false`. Renumbered from PB-340. Amends PB-578/GL-LBR-001-A1. PROVEs the tablet's own four law tests L37-L40, the Miṣru Theorem's three sealed claims (birth, no-decay, conditional inevitability), and the lawful-stakeholder-claim clause rejecting unconditional doom. PROVEs the Šala court renders the contested band, the T_shape ensemble, and the decree lever. |

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
| `vault_body` | 318 | The physical 7×5TB RAID-Z2 NAS host `zpool create ... raidz2 {{ nas_disks }}` runs against — same not-yet-acquired hardware as `nas_vault` below, and NOT the same thing as `uruk-node-vault` (that's a lightweight KVM guest with no disk bays to pass through; merging this alias into it would make PB-318 a live footgun the day someone runs it with `-e confirm_disks=true`). Kept deliberately unmerged, fails cleanly like `kish` below. |
| `nas_vault` | 330, 338 | Same physical NAS device (7×5TB) as `vault_body` above — not yet acquired/provisioned. |
| `najaf_base_station` | 422 | An RTK GPS base station (physical hardware) — not yet acquired. |
| `najaf_field_sbc` | 423, 424, 425 | A field single-board-computer image for GPR anomaly survey work — not yet acquired. |

**Resolved 2026-08-23** (the Uruk 3-VM CQRS+vault topology — `uruk-node-write`
.111 / `uruk-node-read` .112 / `uruk-node-vault` .113 — went live via
`playbook_265 -e create_vault_node=true`, confirmed reachable): the four
alias rows previously listed here as ambiguous are merged and no longer
"not yet runnable" —
- `vault_librarian` (314, backup/restore drills — a software role) →
  `uruk-node-vault`.
- `host_forge` (313, 315, 316, 317, 319, 320 — control-plane/libvirt work:
  VM disk attach, promotion ceremony, inventory validation, game day,
  host storage layout) → `uruk` (the bare-metal control node itself,
  confirmed by `playbook_320`'s own `virsh`/`lvcreate` commands and its
  vars already resolving `vm_write_name`/`vm_read_name`/`vm_vault_name`
  to the real `uruk-node-*` names).
- `write_node` (310, 311) → `uruk-node-write`; `read_node` (312) →
  `uruk-node-read`.

**Fixed 2026-08-23 (same pass, following through on the problem found
during this merge):** several of these six playbooks were written
against infrastructure assumptions that didn't match what actually
exists today. All now corrected:
- Stale pre-2026-08-21 IPs fixed to the real, live node addresses:
  `playbook_313`'s SSH to the old write-node IP `192.168.122.101` → `.111`;
  `playbook_315`'s health-check loop `.101`/`.107` → `.111`/`.112`;
  `playbook_317`'s health-check curls → `.111`/`.112`.
- The phantom `enki-write` hostname (never a real libvirt domain) fixed
  to the real domain name `uruk-node-write` everywhere it was used as
  such: `playbook_313`'s and `playbook_317`'s `virsh destroy`/`domstate`
  calls. (`playbook_313`'s `/etc/hosts` repoint task, which uses
  `enki-write` as a deliberate floating service alias re-pointed on
  promotion — a different, legitimate use — is left untouched; its
  target IP `192.168.122.120` is still an unverified placeholder, same
  status as before.)
- `playbook_316`/`317`'s sibling-playbook invocations fixed from
  nonexistent filenames (`pb313_promotion_ceremony.yml`,
  `pb320_vm_disk_provisioning.yml`, `pb312_read_node_rebuild.yml`,
  `pb314_backup_muster.yml`) to the real ones
  (`playbooks/playbook_313_promotion_ceremony.yml`, etc.), and dropped
  the nonexistent `-i inventories/uruk` flag in `playbook_317` (the
  default inventory in `ansible.cfg` already covers `uruk`/the node
  VMs). `playbook_316`'s `-i inventories/kish` is kept as-is — see next
  point.
- The missing `inventories/uruk/group_vars/all.yml` and
  `inventories/kish/group_vars/all.yml` (`playbook_315`'s and
  `playbook_316`'s actual root cause — these files were part of the
  original PB-310-320 design in
  `docs/phase2-incoming/batch3_streaming_pu_ctg_pb310_320/`, but were
  never copied into `inventories/` at landing time) are now landed for
  real at the repo root, with `ship_target_host` corrected from the
  placeholder `vault-lib` to the real, live `uruk-node-vault`. All other
  values are unverified placeholders exactly as originally delivered
  (`nas_disks`, `host_pv` — both still marked `# VERIFY!` in the files
  themselves) — the Architect still needs to confirm those against real
  hardware before, e.g., `playbook_319`'s `vgcreate` runs for real.
  Deliberately **not** created: `inventories/uruk/hosts.yml` /
  `inventories/kish/hosts.yml` — the original design's copy of these
  (in the same `phase2-incoming` source) would have reintroduced a
  second, competing inventory with its own stale IPs, exactly the
  dual-source-of-truth problem `ansible/inventory.ini`'s own header
  already documents fighting once (the eriduous-vdi/uruk history). The
  single `ansible/inventory.ini` remains the only host-listing source of
  truth; `kish`'s missing hosts file is intentional — it's what makes
  `playbook_316`'s Stage 6 fail cleanly (zero hosts matched) until a
  second real machine exists, per the PB-210 lesson.

**Still open, not fixed — a design question, not a bug:**
`playbook_317`'s T1 task invokes `playbook_313` as a subprocess via
`ansible.builtin.command`, but `playbook_313` has its own `vars_prompt`
for the CSR-08 `architect_seal`. A command-module child process isn't
guaranteed a TTY, so this can hang or fail waiting on input that never
arrives if `playbook_317` itself is ever run non-interactively.
Automating a game-day drill around an interactive human seal is a real
design decision (skip the seal in game-day mode? pass it via
`-e`, weakening CSR-08's premise that it's always freshly typed?) that
needs the Architect's own call, not a silent patch — flagged in
`playbook_317`'s own file as a `NOTE`, not resolved.

## `kish` — defined but not connectable

**Verified in Step 7**: none of the 90 landed playbooks actually target
`hosts: kish` directly today (315/316 target `uruk` as of the 2026-08-23
alias merge above; `kish` shows up in their variables/prose, not their
`hosts:` line) — so this note is
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
`PB-398-401_Gate_and_Bench_Membrane_Playbook_Suite_DRAFT.md`,
`PB-402-409_Nasaru_Sensing_Stochastic_Geometry_Suite_DRAFT.md` (naṣāru's
fourth phase, GL-SEN-001, 2026-08-15) — these describe *planned* playbook
ranges; no individual `.yml` files exist yet for these numbers. `PB_REGISTRY_427-514_DRAFT.md`,
`PB-310-320_SUITE_README.md`, `PB-310-320_SEAL_REGISTER.md`, and
`PB-549-554_GulaFederation_MANIFEST.md` are the suites' own planning/seal
records, landed alongside their playbooks for reference.
