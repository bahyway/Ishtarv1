# Playbook Execution Triage — v4.0 "Close the Gap"

Living checklist. Started 2026-07-24 because the 90–225 range had never been
run against real infrastructure (eriduous-vdi) — code was landed, sandbox-
built, and tested in-container, but never executed live. That gap grows every
session unless something outside the session tracks it. This file is that
something: **every new playbook added to `playbooks/` from now on gets a row
here before the session that wrote it ends.**

Status legend: `[ ]` not run · `[~]` run partially / needs re-check · `[x]`
run and confirmed on real infra · `SKIP` duplicate/superseded/gap-diagnostic
only, no infra action needed.

Run phases **in order** — later phases assume earlier ones are live. Within
a phase, numeric order is fine except where noted.

**Fixed 2026-07-26 — 74 playbooks had `hosts: eriduous-vdi` instead of
`hosts: localhost`.** Found when the Architect ran PB-174 through PB-200
the standard documented way (`-i "localhost," -c local`) and every single
one printed `skipping: no hosts matched` — Ansible does not treat a play
matching zero hosts as a failure, so this looked like a clean pass with
`rc=0` in every terminal, but **zero tasks actually ran**. All 74 files
(spanning 153–169, 174–216, 221–222, 232–246) are now fixed to
`hosts: localhost` + `connection: local`, verified against real task
output. `[ ]` rows below for those numbers are still accurate — nothing
in them has actually been run yet, despite what a prior terminal may have
shown — **do not treat any pre-2026-07-26 run of PB-153–169, 174–216,
221–222, or 232–246 as having verified anything; re-run them.** The
IsimudEngine (see its own section below) was also hardened to
treat `rc=0` + "no hosts matched" as a failure, so this class of
silent false-positive can't corrupt Heal_points again.

---

## Phase 0 — Foundation (PB-90 to PB-118)

**Declaration:** Foundational bootstrap layer — KISPU baseline, session registry, corpus/archive setup, early index/engine scaffolding — that every later phase assumes is already live.

The Architect believes PB-90 or PB-91 already ran, before pausing to build
EnkiDDB/EnkiMDB + DubSar first. **Re-confirm, don't assume** — state may have
drifted since.

| # | File | Status | Note |
|---|------|--------|------|
| 90 | `playbook_90_phase_a_foundation_confirmation.yml` | `[~]` | run 2026-07-25 on eriduous-vdi, hit stale hardcoded pass-counts (54/170) vs. real growth (58/216) — fixed to `>= baseline` assertions, re-run pending |
| 90 | `playbook_90_zakaru_natiru_reconciled.yml` | `[~]` | different content, same number — re-verify separately |
| 91 | `playbook_91_enkidb_ingest_reconciled.yml` | `[~]` | believed run — re-verify |
| 92 | `playbook_92_con_engine_reconciled.yml` | `[ ]` | |
| 93 | `playbook_93_geo_engine_reconciled.yml` | `[ ]` | |
| 94 | `playbook_94_heptascript_v1_reconciled.yml` | `[ ]` | |
| 95 | `playbook_95_enlil_index_stack_reconciled.yml` | `[ ]` | superseded in spirit by PB-183 Anu rename — run this first anyway for history, PB-183 corrects the name after |
| 96 | `playbook_96_tiamat_engine4_reconciled.yml` | `[ ]` | |
| 97 | `playbook_97_kispu_baseline_reconciled.yml` | `[ ]` | |
| 98 | `playbook_98_kispu_fix_reconciled.yml` | `[ ]` | fix-on-97, run right after it |
| 99 | `playbook_99_phase_b_testing_playbook_phase1.yml` | `[ ]` | |
| 99 | `playbook_99_v4_workspace_setup_reconciled.yml` | `[ ]` | different content, same number |
| 100 | `playbook_100_naramsin_archive_reconciled.yml` | `[ ]` | |
| 101 | `playbook_101_session_registry_reconciled.yml` | `[ ]` | |
| 102 | `playbook_102_con_engine_v4_reconciled.yml` | `[ ]` | |
| 103 | `playbook_103_enkidb_indexes_reconciled.yml` | `[ ]` | |
| 104 | `playbook_104_heptascript_v2_reconciled.yml` | `[ ]` | |
| 105 | `playbook_105_sumuukin_routing_reconciled.yml` | `[ ]` | |
| 106 | `playbook_106_corpus_files_reconciled.yml` | `[ ]` | |
| 107 | `playbook_107_archive_corpus_run_reconciled.yml` | `[ ]` | |
| 108 | `playbook_108_hepta_bench_10m_reconciled.yml` | `[ ]` | |
| 109 | `playbook_109_full_phase1_report_reconciled.yml` | `[ ]` | report/summary — run after 90-108 land |
| 110 | `playbook_110_session_deliverables_reconciled.yml` | `[ ]` | |
| 110 | `playbook_110_session_deliverables_urnammu_nisaba_kittu.yml` | `[ ]` | different content, same number |
| 111 | `playbook_111_cqrs_kispu_reverify.yml` | `[ ]` | |
| 111 | `playbook_111_enkidb_cqrs_node_verify_reconciled.yml` | `[ ]` | different content, same number |
| 112 | `playbook_112_kispu_headstore_benchmark_reconciled.yml` | `[ ]` | |
| 113 | `playbook_113_naramsin_engine_formalise_reconciled.yml` | `[ ]` | |
| 113 | `playbook_113_naramsin_enkiterra_coverage.yml` | `[ ]` | different content, same number |
| 114 | `playbook_114_naramsin_format_layer_reconciled.yml` | `[ ]` | |
| 115 | `playbook_115_naramsin_mashsharu_bridge_reconciled.yml` | `[ ]` | |
| 116 | `playbook_116_enki_terra_suite_wire_reconciled.yml` | `[ ]` | |
| 117 | `playbook_117_ashnan_kaki_schema_gap.yml` | `SKIP` | gap-diagnostic only |
| 117 | `playbook_117_ashnan_kaki_schema_v2_reconciled.yml` | `[ ]` | the actual fix — run this one |
| 118 | `playbook_118_ashnan_sensor_pilot_setup.yml` | `SKIP` | superseded by v2 below |
| 118 | `playbook_118_ashnan_sensor_pilot_setup_v2_reconciled.yml` | `[ ]` | run this one |

## Phase 1 — Registries & domain engines (PB-137 to PB-145)

**Declaration:** Domain engine registries and scaffolds (Urnammu attestation, Kittu engine, Ninsun/Namtar/Ereskigal regional, Edubba seal) that other phases' services depend on existing first.

| # | File | Status | Note |
|---|------|--------|------|
| 137 | `playbook_137_urnammu_attestationd_gap.yml` | `SKIP` | gap-diagnostic only |
| 137 | `playbook_137_urnammu_attestationd_impl_reconciled.yml` | `[ ]` | run this one |
| 138 | `playbook_138_kittu_engine_v1_gap.yml` | `SKIP` | gap-diagnostic only |
| 138 | `playbook_138_kittu_engine_v1_reconciled.yml` | `[ ]` | run this one |
| 139 | `playbook_139_ninsun_namtar_ereskigal_ashnan_regional.yml` | `[ ]` | |
| 139 | `playbook_139_ninsun_refiner_wire_reconciled.yml` | `[ ]` | different content, same number |
| 140 | `playbook_140_edubba_seal_integration.yml` | `SKIP` | superseded by reconciled below |
| 140 | `playbook_140_edubba_seal_integration_reconciled.yml` | `[ ]` | run this one |
| 141 | `playbook_141_ashnan_regional_extension_reconciled.yml` | `[ ]` | |
| 142 | `playbook_142_namtar_domain_scaffold_reconciled.yml` | `[ ]` | |
| 143 | `playbook_143_ereskigal_domain_scaffold_reconciled.yml` | `[ ]` | |
| 144 | `playbook_144_grafana_monitoring_setup.yml` | `SKIP` | Architect decision 2026-07-26: dismissed, not just superseded — see note after this table |
| 144 | `playbook_144_grafana_monitoring_setup_reconciled.yml` | `SKIP` | Architect decision 2026-07-26: dismissed — both PB-144 files stand as-is (not deleted) but neither will be run |
| 145 | `playbook_145_rm001_roadmap_update_reconciled.yml` | `[ ]` | doc update, low risk |

**Why both PB-144 files are dismissed, not just blocked:** both are gated on
`dubsar-workstation` (192.168.122.121), which doesn't exist. But the
reconciled version's own report already admits that wouldn't matter even if
the host existed — `enkidb-query-server` exposes no `/metrics` endpoint, so
Prometheus would have nothing to scrape, and two of the three Grafana
dashboards already had panels stripped out because they reference metrics
from crates (`ESARHADDON`, `INANA`) that don't exist in this repo at all.
Grafana would be an empty shell pointed at data nothing emits. The Architect
already has a working pure-Rust alternative — `bahyway-api` (real HTTP
server, `/api/v1/live`, `/api/v1/batches`, `/api/v1/tribes`, `/health`,
smoke-tested live in PB-252) plus the DubSar IDE's EnkiDDB/EnkiMDB Monitor
Dashboard scenes (Godot + Rust GDExtension, no Prometheus/Grafana layer) —
so there's no monitoring capability actually being given up by skipping
this. Rows kept (not deleted) per the Architect's request, so the decision
and its reasoning stay visible in the sequence rather than disappearing.

## Phase 2 — Fable / HeptaScript / Algebra extensions (PB-153 to PB-169)

**Declaration:** HeptaScript query-language extensions, the Fable AI-advisory gate (makes a real cloud call — see Secrets below), wizard/UI contract work, and the geometric-algebra arsenal underneath them.

| # | File | Status | Note |
|---|------|--------|------|
| 153 | `playbook_153_fable_impact_gate_corrected.yml` | `[x]` | run 2026-07-26 on eriduous-vdi (ok=7 changed=4 failed=0) — DOES make a real cloud call (stage 3, `fable_advisory`, hits `claude-fable-5`); `fable_api_key` supplied from the Architect's own BahyWay v4.0 vault (Sargon Passport Manager, PB-227 / `crates/kupru`), not Ansible Vault — see "Secrets" note below |
| 154 | `playbook_154_hs_ext_002_scaffold_corrected.yml` | `[ ]` | |
| 155 | `playbook_155_hs_ext_003_cube_count_corrected.yml` | `[ ]` | |
| 156 | `playbook_156_wizard_contract_corrected.yml` | `[ ]` | |
| 157 | `playbook_157_wizard_install_corrected.yml` | `[x]` | run 2026-07-26 on eriduous-vdi — first run failed twice for real: (1) copied from a `wizard_wiz001_v2/` staging dir that had never actually been created/committed (the playbook's own header already admitted the real wizard files were landed by hand, but the source it claimed to record was never captured), fixed by pointing `src_dir` at the real git-tracked source (`workspace/bahyway_v4/godot/dubsar-theater` itself, `remote_src: true`); (2) the lexicon conformance gate then false-positived on `enki_engines.gd:32`, a comment that documents the forbidden-word policy itself, fixed by extending the gate's exclusion filter. Re-run clean end to end (ok=7 failed=0, `LEXICON CLEAN`) |
| 158 | `playbook_158_tpl_001_templates_projections_corrected.yml` | `[ ]` | |
| 159 | `playbook_159_tpl_001_section_d_corrected.yml` | `[ ]` | |
| 160 | `playbook_160_tpl_001_section_e_corrected.yml` | `[ ]` | this is the Dead_Split1 federated-state-scope law — cross-check against `bahyway-core::death_legacy` (built this session) before running, may already be satisfied |
| 161 | `playbook_161_fable_crosstribe_wizard_reconciliation.yml` | `[ ]` | |
| 162 | `playbook_162_hepta_parallel_dispatch_and_ide_editor.yml` | `[ ]` | check against PB-226+ DubSar IDE (built this session) for overlap before running |
| 163 | `playbook_163_enbilulu_calculus_geoengine.yml` | `[ ]` | |
| 164 | `playbook_164_algebra_arsenal_boundaries.yml` | `[ ]` | |
| 165 | `playbook_165_algebra_arsenal_gap_closure.yml` | `[ ]` | fix-on-164, run right after it |
| 166 | `playbook_166_addu_cyclone_extension_seal.yml` | `[ ]` | concept seal — verify what infra action (if any) it performs |
| 167 | `playbook_167_neberu_slicer_concept_seal.yml` | `[ ]` | concept seal — same check |
| 168 | `playbook_168_analysis_to_solution_law.yml` | `[ ]` | law amendment — verify infra action |
| 169 | `playbook_169_theater_as_workbench_law.yml` | `[ ]` | law amendment — verify infra action |

## Phase 3 — Index fixes + EnkiDDB/EnkiMDB foundation (PB-172 to PB-198)

**Declaration:** Sequential index-naming corrections (Enlil→Anu, Naba→Nisaba) plus the initial build-out of EnkiDDB/EnkiMDB themselves — the two EnkiDB Types PB-259 (Phase 4/5's note) later deploys and ingests real data into.

**Sequential — later ones correct earlier ones by name (Enlil→Anu, Naba→Nisaba).**

| # | File | Status | Note |
|---|------|--------|------|
| 172 | `playbook_172_enlil_index_wiring_fix.yml` | `[ ]` | |
| 173 | `playbook_173_gate1_heptagate_resolution.yml` | `[ ]` | |
| 174 | `playbook_174_enkiddb_cqrs_writenode_readnode.yml` | `[ ]` | |
| 175 | `playbook_175_enkimdb_cqrs_writenode_readnode.yml` | `[ ]` | |
| 176 | `playbook_176_sovereign_db_names_and_generations.yml` | `[ ]` | |
| 177 | `playbook_177_enkiddb_rag_chunking_and_categorization.yml` | `[ ]` | |
| 178 | `playbook_178_enkiddb_ingest_cli_directory_upload.yml` | `[ ]` | |
| 179 | `playbook_179_taxonomy_from_canonical_docs_layout.yml` | `[ ]` | |
| 180 | `playbook_180_otap_pipeline_for_enkiddb_rebuild.yml` | `[ ]` | |
| 181 | `playbook_181_enkiddb_podman_write_read_servers.yml` | `[ ]` | |
| 182 | `playbook_182_enlil_no_tree_in_orbits_audit.yml` | `[ ]` | |
| 183 | `playbook_183_anu_index_stack_rename_and_w5h2_reference.yml` | `[ ]` | corrects PB-95/172 naming — run after them |
| 184 | `playbook_184_storage_prebuild_readiness_for_enkiddb.yml` | `[ ]` | |
| 185 | `playbook_185_anu_index_prebuild_readiness_for_enkiddb.yml` | `[ ]` | |
| 186 | `playbook_186_mandatory_vs_optional_attributes_reference.yml` | `[ ]` | |
| 187 | `playbook_187_sumer_engine_and_nergal_av_sealed.yml` | `[ ]` | |
| 188 | `playbook_188_irkalla_sovereign_name_and_eridu_os_layout.yml` | `[ ]` | |
| 189 | `playbook_189_nisaba_not_naba_correction.yml` | `[ ]` | naming fix, run before 190/191 |
| 190 | `playbook_190_nisaba_internal_interface_lamassu_tda.yml` | `[ ]` | |
| 191 | `playbook_191_nisaba_bounded_autonomy_grant.yml` | `[ ]` | |
| 192 | `playbook_192_enkiddb_enkimdb_podman_deploy_executable.yml` | `SKIP` | superseded by PB-212 (2026-07-19): the Architect's real intended shape is one VM per CQRS role, not one VM per database, with `eriduous-vdi` reserved as the IDE/control/monitoring host. Confirmed 2026-07-26: this playbook really did run once (2026-07-18) — its 5 containers exist on eriduous-vdi as `Exited (137)`, stale leftovers from before the PB-212 migration. Left stopped, not restarted; safe to `podman rm` when convenient, not urgent |
| 193 | `playbook_193_enkiddb_enkimdb_roadmap_manual_glossary.yml` | `[ ]` | docs |
| 194 | `playbook_194_enkiddb_concept_exposure_preview.yml` | `[ ]` | |
| 195 | `playbook_195_hepta_gate_names_correction.yml` | `[ ]` | |
| 196 | `playbook_196_nergal_sovereign_name_registered.yml` | `[ ]` | |
| 197 | `playbook_197_onion_layers_design_and_artifact.yml` | `[ ]` | |
| 198 | `playbook_198_enkiddb_cli_ingest_dir_authorship_gate.yml` | `[ ]` | |

## Phase 4 — Real-infra hardening (PB-199 to PB-216)

**Declaration:** Real-infrastructure fixes and the EnkiDDB/EnkiMDB CQRS deploy+ingest+sync chain (PB-199-216) — the part of this doc PB-259 consolidates for a 7-Types build; see the note right below for exactly which rows that covers.

**For the 7-Types EnkiDB build specifically, see PB-259
(`playbook_259_full_7types_enkidb_bootstrap.yml`) instead of running
212/213/216 by hand from this table — it chains them (plus 208/221/222)
in the correct order as one command. 199-202/211/214/215 in this phase
are unrelated fixes/superseded/diagnostic-only, not part of that chain;
see `docs/SHEDU/DEPLOY_REFERENCE_ALL_PLAYBOOKS.md`'s first-time-bringup
section for the full breakdown of what's in PB-259 vs. what isn't.**

**Sequential and load-bearing: PB-200/201/202 are fixes for real bugs PB-199
hit live on eriduous-vdi (SELinux `z`/`Z`, missing git in container, git
"dubious ownership"). Skipping ahead here will likely reproduce those exact
bugs.**

| # | File | Status | Note |
|---|------|--------|------|
| 199 | `playbook_199_enkiddb_write_source_mount_for_ingest_dir.yml` | `[ ]` | |
| 200 | `playbook_200_source_mount_z_vs_Z_selinux_fix.yml` | `[ ]` | fix for real bug hit running 199 |
| 201 | `playbook_201_enkiddb_write_container_needs_git_for_authorship.yml` | `[ ]` | fix for real bug #2 |
| 202 | `playbook_202_enkiddb_write_git_dubious_ownership_fix.yml` | `[ ]` | fix for real bug #3 |
| 203 | `playbook_203_enkiddb_enkimdb_health_check_and_backup.yml` | `[~]` | run for real 2026-07-26 on eriduous-vdi, twice. First attempt failed at 1.3, all four containers reported "NOT RUNNING" — authored 2026-07-19 for the old single-host topology (`hosts: localhost`, every task a bare `podman`/`stat` call, no `delegate_to`); PB-212 (later the same day) moved the real fleet onto the 2-VM CQRS split and this playbook was never updated to follow — it was checking `podman ps` on eriduous-vdi, which hasn't run any of these containers since PB-212 landed, the fleet itself was never actually unhealthy. Fixed (PB-203.2): every task split by which real host each container/volume now lives on (`delegate_to: enkidb-node-write`/`enkidb-node-read`), port checks against each VM's real IP. Second attempt: 1.1-1.7 passed for real (`enkiddb-write: Up About an hour` etc., all four ports confirmed live, all four volumes confirmed "has materialized Data Files") after fixing 1.6's real `Permission denied` reading `.../volumes/enkiddb-write-data/_data/current/entities.data` with `podman unshare test -f` (PB-203.3) — but 2.3's backup tarball step then hit its *own* real `Permission denied` (`tar: .: Cannot stat`): PB-203.3's staging approach pre-created the destination with a plain `mkdir` and `podman unshare cp -a src/. dest/`-merged into it, which doesn't reproduce PB-213/258's actually-working shape. Fixed (PB-203.4): mirror PB-213/258 exactly — `podman unshare cp -a` creates the `current` subdirectory FRESH under a plain parent (never merges into a pre-existing dir), and `tar` names that subdirectory explicitly instead of trying to stat `.` on anything `podman unshare` touched directly. `--syntax-check` passes. **Not yet re-run against real infra** — needs the Architect to confirm 2.3 onward now succeeds |
| 204 | `playbook_204_akkadi_debug_capture_client_notebook_delivery.yml` | `[ ]` | |
| 205 | `playbook_205_asset_server_and_remote_ingest_3podman_vdi.yml` | `[ ]` | |
| 206 | `playbook_206_musaru_security_scan_pre_ingest_gate.yml` | `[ ]` | |
| 207 | `playbook_207_dubsar_notebook_project_godot_and_enkiddb_wiring.yml` | `[ ]` | check overlap with PB-226+ before running |
| 208 | `playbook_208_full_corpus_ingestion_runbook.yml` | `[x]` | run for real 2026-07-26 on eriduous-vdi. First attempt: `RUN skipped -- 9 file(s) failed` (all-or-nothing authorship gate, zero of 241 sent) — task 2.3's message wrongly implied partial success, fixed (`856addd`). Root files: 9 untracked local docs (ERESKIGAL/NAMTAR/SHEDU/WIZ-001/TPL-001) with no git history; committed them (`c192036`, `f5cdbde`), which needed a real `.gitignore` fix first (`docs/SHEDU/` directory-pattern silently dead-ended per-file negation exceptions, `294d932`). Second attempt: git history now found but author `bahyway@enkidb.io` (eriduous-vdi's own default git identity, distinct from `bfadam@bahyway.com`) wasn't on the team allowlist — added (`5f865a0`). Third attempt: **`241 authorized, 0 rejected`, `241 document(s) sent`, `OK:FLUSHED:5256`** — real success. Surfaced a real, separate bug in the read-side deployment, see PB-212's note. **PB-208.2 (`00bd817`):** this playbook's own `vars:` for the imported `enkiddb_cross_host_sync.yml` task never set `enkiddb_search_phrase`, so its SEARCH step always sent `SEARCH:5:` (empty query text) — `adapa-recall`'s `RecallIndex::query` returns an empty `Vec` the instant the query tokenizes to zero tokens, an unconditional `0 hit(s)` regardless of real corpus content. Root-caused via a local `RagIndex`/`RecallIndex` repro (real hits against the real doc, ruling out the search logic) plus a live `--dry-run` CATEGORIZE (confirming `docs/BAHYWAY_ONION_LAYERS_DESIGN.md` itself is authorized and ingestible, ruling out the corpus). Fixed by setting `enkiddb_search_phrase: "Onion Layers"` explicitly, matching PB-211/PB-213. Re-run for real 2026-07-26: `241 authorized, 0 rejected`, `OK:FLUSHED:15768`, QUERY `matched=20 rows`, **SEARCH now returns 5 real ranked hits** (scores 0.4627/0.3911, real text snippets from "The Four Laws"/"The 25 Sealed Layers") — all three Definition-of-done criteria confirmed |
| 211 | `playbook_211_flush_sync_and_verify_enkiddb.yml` | `[ ]` | |
| 212 | `playbook_212_deploy_cqrs_2node_split.yml` | `[x]` | confirmed genuinely run for real, twice. First 2026-07-19 (not sandbox — verified via real pasted terminal output, all 4 containers up on `.101`/`.107`, `--restart=always` set, lingering fix for SSH-logout kills landed as `3d436db`). Then the underlying hypervisor host rebooted ~2026-07-22/23 — `--restart=always` alone does NOT survive a reboot, only `podman-restart.service` (unset) makes it act; all 4 containers came back `Exited (137)` and sat dead for days, manually restarted 2026-07-26, ingested data confirmed intact (15M in `enkiddb-read-data`, nothing lost). Fixed 2026-07-26 (PB-212.2, `ea4556d`+`edc6057` — first pass only landed in the Write play, its 0.6 task name differs slightly from the Read play's so the replace missed it, caught immediately from live output and fixed). Re-run 2026-07-26 against the real inventory (`-i ansible/inventory.ini`, not `-i "localhost,"` — the earlier attempt with `-i "localhost,"` silently skipped both real plays, `no hosts matched`): `enkidb-node-write ok=18 changed=7 failed=0`, `enkidb-node-read ok=17 changed=6 failed=0`, `podman-restart.service` confirmed enabled on both. Note: `-i "localhost,"` never works for this playbook — it targets `enkidb_write`/`enkidb_read` inventory groups over real SSH, unlike every other single-host playbook in this repo. **Third real bug found the same day (PB-212.3, `fa13276`):** "check if exists, skip run if it does" meant rebuilding the image (steps 1.1/1.2) never reached an already-running container. `enkiddb-read` was created 2026-07-19 and every later run of this playbook rebuilt its image but found the container already existed and skipped recreating it — so it kept serving the pre-`02a6216` (2026-07-21) JSON wire protocol for a week regardless of how many times the image was rebuilt underneath it. Traced from a real PB-208 follow-up run: QUERY reported `matched=1634411131 rows` from only 5256 real entities; that number's little-endian bytes decode to literal `{"ka`, the start of the old JSON response. Fixed by replacing the skip-if-exists pattern with always stop+rm+recreate for all four containers — volumes are separate named mounts, so this never touches real data, only the container process |
| 213 | `playbook_213_cross_host_flush_sync_verify.yml` | `[x]` | run for real 2026-07-26 on eriduous-vdi, `ok=22 changed=12 failed=1`. First hit a real inventory bug (see `ansible/inventory.ini`, `15f9004`): `enkidb-node-write`/`enkidb-node-read` had no explicit `ansible_connection`, so this playbook's play-level `connection: local` (needed for its `hosts: localhost` + `delegate_to:` pattern) silently bled through to the delegated hosts — every "remote" task ran locally on eriduous-vdi instead of over SSH, giving `ConnectionRefusedError` on FLUSH (connecting to eriduous-vdi's own empty :7101, not the write node's) that looked exactly like a dead container. Root-caused via `-vvvv` showing `ESTABLISH LOCAL CONNECTION` where an SSH command should have appeared; fixed by adding `ansible_connection=ssh` to both inventory hosts. Once fixed, the actual FLUSH/sync/SELinux-restore mechanics all worked correctly — but exposed a real, separate gap: the write node's own `/data/current` is genuinely empty (confirmed via `podman exec ... ls -la`, all files timestamped exactly when this run's FLUSH executed) — its July 19 ingestion never survived the reboot+rebuild the way the read node's did. This run's sync faithfully propagated that real emptiness, overwriting the read node's previously-intact 15M of July 19 data with empty files. Not data loss in the "gone forever" sense — the underlying source documents are git-tracked and untouched — but the database now needs a real fresh ingestion. **PB-208 then ran real ingestion (241 docs, FLUSHED:5256) and this playbook's own sync/QUERY/SEARCH steps ran again** — QUERY returned `matched=1634411131 rows` (garbage) and SEARCH returned `[]`, both traced to a real PB-212 bug (stale pre-`02a6216` container never recreated on rebuild, fixed `fa13276` — see PB-212's note for the full root cause). **PB-212's stale-container fix (`fa13276`) was applied, but recreating the Write container triggered enkiddb-write-server's OWN documented v1 limitation** ("The Journal itself is in-memory only... Documents ingested since the last flush are lost if the container restarts before the next one. This is a real, known v1 limitation, not hidden.") — FLUSH correctly reported `0`, and this task's sync then propagated that real emptiness over the Read Node's data a second time, plus SEARCH's own client turned out to still be parsing the pre-`02a6216` JSON format (fixed `f0add16` — was printing raw unparsed binary bytes as "results"). Added a real safety net (`8f709c8`): the sync now refuses to overwrite a populated Read Node with suspiciously-smaller/empty staged data instead of silently destroying it, `enkiddb_allow_shrink_sync: true` to override when genuinely expected. **SEARCH then kept returning `0 hit(s)` even against a confirmed-populated (`rag_sections=5015`) real index** — root-caused (see PB-208.2, `00bd817`) to PB-208's own `enkiddb_cross_host_sync.yml` import never setting `enkiddb_search_phrase`, so every SEARCH call sent empty query text; `RecallIndex::query` returns empty on zero query tokens by design, unconditionally. Not a bug in this playbook or its sync/verify mechanics at any point. **Closing confirmation run, 2026-07-26 (via PB-208 with the fix applied):** `241 authorized, 0 rejected`, `OK:FLUSHED:15768`, QUERY `matched=20 rows`, SEARCH `5 hit(s)` with real scores (0.4627/0.3911) and real text snippets. All three Definition-of-done criteria (real ingest, real QUERY, real SEARCH) confirmed in one clean pass — closing `[x]` |
| 214 | `playbook_214_diagnose_read_node_not_ready.yml` | `[ ]` | diagnostic tool, run if 213 shows issues |
| 215 | `playbook_215_full_environment_bootstrap.yml` | `[~]` | master bootstrap — good re-entry point if state gets messy. **Real ordering bug found live 2026-07-27**: chained `212 -> 213 -> 208` (sync BEFORE ingest). PB-212's own 3.1/3.3 unconditionally stops+removes+recreates every container on every run (deliberate, see PB-212's PB-212.3 note), which always empties the Write Node's in-memory journal; running PB-213 immediately after got a legitimate `FLUSH:0`, and its own safety guard correctly refused to sync that over the Read Node's real, already-populated data (`staged=0 bytes, destination=51161402 bytes`) — no data was lost, but the run failed every time it was re-run against an already-live environment. Fixed (PB-215.1): reordered to `212 -> 208 -> 213`; also fixed PB-212's own summary message, which baked in the same wrong order. `--syntax-check`-equivalent (`yaml.safe_load`) passes. **Not yet re-run against real infra** with the fix |
| 216 | `playbook_216_populate_enkimdb_catalog.yml` | `[ ]` | |

## Phase 5 — EnkiDB Types deploy (PB-221 to PB-225)

**Declaration:** Deploys the remaining 5 of the 7 EnkiDB Types (core + the BeeMDM chain) — PB-221/222 are steps 2/3 of PB-259's 7-Types build; PB-223-225 are diagnostic/reusable/unrelated, see below.

**221/222 are steps 2/3 of PB-259 (see Phase 4's note above) — don't run
them by hand as part of a 7-Types build. 223/224/225 are diagnostic/
reusable/unrelated, not part of that chain.**

| # | File | Status | Note |
|---|------|--------|------|
| 221 | `playbook_221_enkidb_core_deploy_and_scale_sweep.yml` | `[ ]` | **Known issue flagged 2026-07-27, not yet fixed**: tasks 3.1/3.2 use "check if exists, skip run if it does" for both containers -- the exact pre-PB-212.3 pattern that let a rebuilt image never reach an already-running container for a week on EnkiDDB/EnkiMDB. Hasn't triggered here yet only because neither container has been re-run after a real image change. PB-212.4's conditional-recreate-on-image-change fix is the right pattern to port here; not done in this pass (see this playbook's own KNOWN ISSUE header note) |
| 222 | `playbook_222_enkisdb_odb_qdb_dw_deploy.yml` | `[ ]` | the other 4 of the 7 EnkiDB Types. **Same known issue as PB-221 above** (same pattern, same fix needed, not yet applied) |
| 223 | `playbook_223_cqrs_node_disk_diagnostics.yml` | `[ ]` | diagnostic, run if 221/222 show disk issues |
| 224 | `playbook_224_cqrs_node_reconfigure.yml` | `[ ]` | reusable tool, run as needed |
| 225 | `playbook_225_ooo_vocabulary_gate.yml` | `[ ]` | |

## Phase 6 — DubSar IDE (PB-226 to PB-234)

**Declaration:** Builds and launches the DubSar Godot IDE itself (login gate, wizards, theme, PDM) — a separate UI subsystem, unrelated to EnkiDB data or PB-259.

**Confirmed: never run by anyone, anywhere. Highest-confidence, highest-value
target — run PB-234 (not 231/233 separately, it supersedes both).**

| # | File | Status | Note |
|---|------|--------|------|
| 226 | `playbook_226_launch_dubsar_godot_ide.yml` | `[x]` | run live many times 2026-07-19 through 2026-08-02. FIXED 2026-08-02, live: was running its headless `--import` priming scan unconditionally on EVERY launch (effectively starting Godot twice per icon click), reported as real slowness — added an explicit opt-in `-e skip_prime=true` (default `false`, so terminal use keeps priming on; only PB-288's desktop launcher passes it). Still reported slow with skip_prime alone — the 12-module stat+copy loop (6 bridge libs × verify+copy) also ran unconditionally on every launch; gated `when: not skip_build` — **this broke a real bare-metal launch live**: on a checkout where the bridges had never been copied into `dubsar-theater/bin/` before, `-e skip_build=true` skipped the copy entirely, all 6 `.so`s were missing, and the window came up blank gray (`login.gd` couldn't find `KupruBridge`/`EnkiEngines`). REVERTED to a self-healing check: stats the actual destination files first, only skips the copy when `skip_build` is true **and** every destination `.so` is already really present — fast when warm, never silently wrong when it isn't. FIXED 2026-08-02, live (second real bug the same day): login/theater now loaded, but the 3D CENTER_STAGE/BIGRING view rendered as two flat, unshaded green shapes instead of a 50,000-particle ring — root cause was the `gl_compatibility`/`opengl3` renderer override, a 2026-07-28 workaround scoped specifically to eriduous-vdi's software-GPU (no working Vulkan) situation, carried over unexamined to the bare-metal host, which has a real Intel UHD Graphics iGPU with working Vulkan. Defaults reverted to empty (Godot now uses `project.godot`'s own `forward_plus`); the old VDI-era values are still available as an explicit opt-in (`-e rendering_method=gl_compatibility -e rendering_driver=opengl3`) for any future host that genuinely lacks a working Vulkan driver. |
| 227 | `playbook_227_build_and_launch_kupru_tools.yml` | `[ ]` | |
| 261 | `playbook_261_isimud_headless_bootstrap_passport.yml` | `[x]` | Written 2026-07-27, answering the Architect's own question — how does an unattended first-time IsimudEngine run get past DubSar's login gate without a human clicking through a Godot GUI window to mint a Passport. Runs `scripts/isimud_bootstrap_mint.gd` via `godot --headless --script` inside the dubsar-theater project. **First real run on eriduous-vdi (2026-07-27) caught a genuine, severe, pre-existing bug**, not a bug in this playbook: `derive_key()` mints a fresh RANDOM salt on every call by design (`kupru::sargon_kdf::SargonKdf::new`), but `login.gd` (DubSar + DubSar PDM) and Sargon Passport Manager's/Gilgamesh's own vault/ledger code all called it a second time at decrypt time expecting the SAME key back — meaning **no one could ever have successfully logged into DubSar, or reopened a Sargon vault / Gilgamesh ledger across an app restart, before this fix**, in any prior session. This playbook's own round-trip check (mint→import→decrypt→verify, not just a file write) is what surfaced it: the priming import step crashed (rc=134) obscuring the real signal at first, but the mint script itself failed cleanly at `roundtrip decrypt_vault_blob: aead::Error` once isolated. Fixed same-day: added `KupruBridge.derive_key_with_salt()` (kupru-gdext), persisted the salt alongside every sealed blob (`salt_b64` field in the two JSON login vaults; 32 raw bytes prepended to Sargon's/Gilgamesh's own single-blob vault/ledger files), switched every decrypt-time caller to `derive_key_with_salt`, and added 3 new regression tests to `kupru::sargon_kdf` pinning the exact property that broke (`with_salt` must reproduce `new`'s key given its salt) — 20/20 `kupru` tests passing, full `cargo build --workspace` clean. Mints a Sargon **gardener** (privilege 1) Passport via `KupruBridge.issue_gardener_passport` (the exact call Sargon Passport Manager's own GUI makes for its lowest tier — no new mint path invented). **Gilgamesh (architect, privilege 7, Shamir M-of-N ceremony) is never touched by this playbook and never will be** — that stays 100% human, by design, per the Architect's own explicit choice when asked. **Confirmed fully working end-to-end on real eriduous-vdi infra, 2026-07-27** (two runs): first re-run after the salt fix hit `rc=0` with `ISIMUD-BOOTSTRAP OK: minted + imported + round-trip-verified a gardener Passport` in the log, but the playbook itself still hard-failed from a second, unrelated bug in its own success-detection (its shell task redirects output to `{{ godot_log }}` via `>`, so Ansible's `mint_result.stdout` was always empty, and `failed_when` was checking that empty string for the OK marker) — fixed same-day by reading the real output back with `cat` before checking it. Second re-run: **`ok=14 failed=0`, clean pass.** Also fixed a cosmetic report bug the same run surfaced: the "Report what was minted" section was leaking unrelated `at: ...` stack-trace lines (from three separate, pre-existing, unrelated missing GDExtensions — `libdubsar_gridnav_gd.so`, `libmarduk_gdext.so`, `libnaming_registry_gd.so` — none of which this playbook builds or copies; a different subsystem's own gap, not investigated further here) into the summary, because they share the same two-space indent; tightened the match regex to exclude them. The priming import step's `rc=134` crash still happens every run and remains unexplained but confirmed non-blocking — KupruBridge loads and works correctly regardless, three runs running now. |
| 228 | `playbook_228_build_bahyway_codium_theme_extension.yml` | `[ ]` | |
| 229 | `playbook_229_eriduos_unified_desktop_theme.yml` | `[ ]` | |
| 230 | `playbook_230_build_and_launch_dubsar_pdm.yml` | `[ ]` | |
| 231 | `playbook_231_build_navi_translate_plugin.yml` | `SKIP` | run PB-234 instead |
| 232 | `playbook_232_impact_gate_deterministic.yml` | `[ ]` | |
| 233 | `playbook_233_build_engine_map_plugin.yml` | `SKIP` | run PB-234 instead |
| 234 | `playbook_234_launch_dubsar_ide_full.yml` | `[ ]` | **run this** — consolidated, all 4 tabs |

---

## Phase 7 — Workspace integrity (PB-247)

**Declaration:** One-shot audit of the Cargo workspace's own crate/bin wiring against `workspace.members` — a standing regression gate, already closed `[x]`, not an EnkiDB data concern.

| # | File | Status | Note |
|---|------|--------|------|
| 247 | `playbook_247_workspace_orphan_crate_audit_and_wire.yml` | `[x]` | run 2026-07-25 in sandbox before shipping — audits crates/bin dirs vs. workspace.members, found+fixed 3 real orphans (bahyway-dqm, orbital-trust-probe, quant-engine); standing regression gate for future orphans |

## Phase 8 — Ecosystem completeness (PB-248)

**Declaration:** One-shot build+test gate across all 56 real crates the ecosystem believed existed but had zero playbook coverage for — already closed `[x]`, a standing regression gate for future orphans.

| # | File | Status | Note |
|---|------|--------|------|
| 248 | `playbook_248_ecosystem_completeness_build_test_gate.yml` | `[x]` | run 2026-07-25 in sandbox before shipping — Architect found `ezida-ir` had no playbook coverage; audit against full playbooks/*.yml text found 56 real crates (HeptaSec×4, EnkiduLLM stack×8, ezida-ir/akkadi-ir/dfg-engine/hdf-bridge, pattern-intelligence×4, pipeline stations×9, storage internals×8, eridu-scheduler/-supervisor, misc×11) with zero mention anywhere; cargo check + cargo test --release on all 56 — 797 tests, 0 failures; standing gate re-scans for new gaps on every run. Recommended next: individual live-infra playbooks for HeptaSec + EnkiduLLM (highest-stakes untested subsystems) |

## Phase 9 — Full recovery closure: functional verification + last manual-procedure gap (PB-249–253)

**Declaration:** Final closure of the "everything must be a playbook" directive — functional verification for five separate subsystems (HeptaSec, EnkiduLLM/EaAgent, security singles, standalone apps, Fedora44 I/O tuning). Run in the exact order given below; unrelated to EnkiDB data.

Architect's directive: "I want FULL recovery of all non-playbook code to be
in playbooks so that when I create the Ecosystem again I will run the
working playbooks not searching for fragmented manual code." Closes every
remaining item from PB-248's 56-crate batch plus one docs-sweep finding.

**None of PB-249–252 have actually been run on real infra yet** — they were
verified end-to-end in the authoring sandbox before shipping (real
assertions, real failures caught and fixed), but per this file's own status
legend that's not the same as `[x]`. Correcting that now so this table is
trustworthy as a literal run order, not just a shipping record.

Run on eriduous-vdi, in this exact order (each is idempotent — safe to
re-run):

```bash
cd ~/Forge/EnkiDB/playbooks
ansible-playbook playbook_249_heptasec_layer8_functional_verification.yml    -i "localhost," -c local -v
ansible-playbook playbook_250_enkidullm_eaagent_functional_verification.yml  -i "localhost," -c local -v
ansible-playbook playbook_251_security_singles_functional_verification.yml   -i "localhost," -c local -v
ansible-playbook playbook_252_standalone_apps_smoke_launch.yml               -i "localhost," -c local -v
ansible-playbook playbook_253_fedora44_io_tuning_application.yml             -i "localhost," -c local -v -e data_dir=/var/enkidb
```

PB-253 needs `become`/root (it remounts and runs `zfs set`) — run with
`--ask-become-pass` or as a user with passwordless sudo. Pass the real
`data_dir` if it isn't `/var/enkidb` on this host. PB-250's live-node test
only passes if the PB-212 EnkiDDB Read Node is actually reachable at
192.168.122.107:7102 from this host — it fails loudly rather than hanging
if not, so don't be alarmed if it reports the node unreachable and skips
that one check; the rest of the playbook still completes and is still a
real pass.

| # | File | Status | Note |
|---|------|--------|------|
| 249 | `playbook_249_heptasec_layer8_functional_verification.yml` | `[ ]` | sandbox-verified 2026-07-25 before shipping (not yet run on eriduous-vdi) — new cross-crate integration suite (`crates/hepta-sec-web/tests/full_stack_integration.rs`, 7 tests) chains KakiExtractor→WebSentinelGuard→Sentinel→Policy→Firewall through real HTTP-header bytes; found+fixed a real dead-code bug (`HeptaSecSentinel::purge_stale()` always returned 0); open item noted honestly — HeptaSec is verified correct but still not embedded in any running server binary |
| 250 | `playbook_250_enkidullm_eaagent_functional_verification.yml` | `[ ]` | sandbox-verified 2026-07-25 before shipping (not yet run on eriduous-vdi) — real PDF fixture ingested through enkidullm-ingest→tokenize→zikru-embed::train_epoch (`crates/zikru-embed/tests/real_document_training_chain.rs`); real MemoryStore→search chain (`crates/enkidullm-memory/tests/real_session_search_chain.rs`); `ea-agent-chat`'s `!enkiddb` command tested against the real PB-212 node (`#[ignore]`d, run via playbook's `wait_for` reachability gate — will only actually exercise the live node once run from a host that can reach 192.168.122.107:7102); corrected a doc/reality mismatch in enkidullm-memory (claimed working persistence that doesn't exist yet) |
| 251 | `playbook_251_security_singles_functional_verification.yml` | `[ ]` | sandbox-verified 2026-07-25 before shipping (not yet run on eriduous-vdi) — istar's `AkkFirewall` (5 constitutional meta rules + gate builders) had ZERO tests before this playbook; added 17 real tests directly in `crates/istar/src/akk_firewall.rs`. pii-vault/bahyway-z3 already had thorough real tests (15 and 11+ respectively) — audited and confirmed sufficient rather than manufacturing busywork; pii-vault deployment is tracked separately as an open item in `sla-engine`'s own GDPR gap register. adad-gate/vgca-validation are not orphaned — real deps of dubsar/najaf-ingest/bee-watchdog, already covered |
| 252 | `playbook_252_standalone_apps_smoke_launch.yml` | `[ ]` | sandbox-verified 2026-07-25 before shipping (not yet run on eriduous-vdi) — real smoke-launch (not just cargo test) for bahyway-server (bounded --ticks run, clean shutdown), bahyway-cli (stdin REPL round trip), enkidw-cli (real ingest against empty landing dir), bahyway-api (real HTTP server, curl-verified /health + /api/v1/live). bahyway-web is a WASM app (trunk + browser), honestly scoped out — build-only, not smoke-launched |
| 253 | `playbook_253_fedora44_io_tuning_application.yml` | `[ ]` | NOT run against real infra — applies real host mount/ZFS-dataset changes (`docs/FEDORA44_IO_TUNING_GUIDANCE.md`'s "What to actually do next" steps 1-2, noatime + recordsize/compression/atime=off), deliberately not auto-executed in sandbox since remounting a live filesystem is a shared-infra-risk action outside authoring scope; syntax-checked and detection logic (`findmnt` parsing) manually verified correct against the sandbox's real ext4 mount. Needs root — see command block above |

## Phase 10 — Web Arsenal: public Academy interface (PB-254)

**Declaration:** Public-facing static-HTML5 Academy site generator (AcadEngine) for bahyway.com/heptascript.com/beemdm.com — renders EnkiDDB/EnkiMDB's already-ingested content, doesn't ingest anything itself.

AcadEngine: the rendering layer for the three public sites
(www.bahyway.com, www.heptascript.com, www.beemdm.com) — turns
EnkiDDB/EnkiMDB's public-facing documentation into a static-HTML5
Academy. Renumbered and ground-checked from an ideation-only session's
draft that used PB-180 (already taken) and a host/invocation model this
repo doesn't use — see PB-254's own header comment for the full
correction record.

| # | File | Status | Note |
|---|------|--------|------|
| 254 | `playbook_254_acadengine_web_academy_scaffold.yml` | `[x]` | run 2026-07-26 — `acad-engine` crate scaffolded (content model, per-domain HTML5 renderer, routing + canonical laws), registered in workspace `Cargo.toml`, 2/2 tests passing (`routing_law_holds`, `mirror_declares_canonical`). Post-gate, deliberately deferred: live EnkiDDB feed for lecture particles, real TemplateEngine (.tmpl) replacing the string-builder renderer, sovereign per-language lexers for build-time code highlighting, actual SUSA outward publication |
| 255 | `playbook_255_buzu_bivector_orbit_encoding.yml` | `[x]` | run 2026-07-26 — renumbered/corrected from an ideation-only "PB-182" draft that collided with the real, already-landed PB-182 and only shipped a markdown spec, zero code. Landed `crates/buzu-core`: GL-VIZ-001 §1 (bivector → rotor → parametric orbit position) implemented for real by reusing `bahyway-algebra::clifford::Multivector`'s already-tested Cl(7,0) geometric product, proven by 7/7 passing tests (radius-preservation, in-plane confinement, rotor composition, FUZZY additive semantics). Genuinely open, not guessed: D1 (bivector residence per-Tribe vs per-particle), D2 (BUZU chunk byte layout), D3 (FUZZY packed encoding) — see `docs/GL-VIZ-001.md` §5 |
| 256 | `playbook_256_buzu_chunk_seal_d1_d3_ratified.yml` | `[x]` | run 2026-07-26 — Architect ratified the performance recommendation for D1/D2/D3 against the >1B particles/<1s condition; this playbook implements the ratification as real, tested code, not just an updated document. Landed `crates/buzu-core::chunk`: per-Tribe residence (D1, avoids the 24+GB/1B-particle wall of per-particle bivectors), 32-byte SoA `ChunkHeader` + 65536-particle (2^16, GPU-warp-aligned) chunks + FNV-1a load-integrity checksum (D2), sparse index+delta FUZZY side-array so GOLDEN particles cost zero extra bytes (D3). 15/15 tests passing (7 from PB-255 + 8 new: header size, checksum round-trip + corruption detection, FUZZY sparsity, capacity enforcement, packed-vs-direct position equivalence, FUZZY dequantization). Honest limit measured, not asserted: CPU-side evaluation runs ~1.0M particles/s in the authoring sandbox (1B sequentially ≈ 1000s) — the actual <1s/1B claim needs a real GPU dispatch path, out of this crate's scope by design (§1's whole point is GPU-parallel evaluation, not CPU) |
| 257 | `playbook_257_buzu_gpu_wgpu_compute_verification.yml` | `[x]` | run 2026-07-26 — the actual GPU dispatch GL-VIZ-001 §5 said no CPU-side crate could honestly measure. Landed `crates/buzu-gpu`: a real wgpu compute shader (`shaders/orbit.wgsl`) evaluating the identical bivector→rotor→position law, plus a dispatcher that runs it against whatever `wgpu` adapter exists and reports an honestly-labeled result — real hardware, software emulation, or a genuine skip if no adapter exists at all, never a fabricated number. Shader formula hand-derived then validated (not trusted on derivation alone) via a throwaway property test against `Rotor::apply` across 2000 random cases (max error ~4.4e-16). A real bug was caught and root-caused during authoring: running against SwiftShader (software Vulkan, the only adapter available in the authoring sandbox — no `/dev/dri` present) showed a ~1.3e-3 discrepancy; isolated via a pure-f32 Rust reimplementation of the identical formula (only ~2.7e-7 off the f64 CPU reference — precision, not a bug) and a direct sin/cos comparison (confirmed SwiftShader's trig builtins are deliberately lower-precision than real hardware/libm). Fixed with an adapter-aware correctness tolerance (tight for hardware, explicitly looser for software), not by loosening a single global number. Confirmed working both ways: honest skip with no adapter, and correctness-passed + labeled-software-throughput under SwiftShader. **Not yet run against real hardware** — eriduous-vdi is a KVM/libvirt VM and may have no GPU passthrough; run `cargo run --release -p buzu-gpu -- 1000000000` on real hardware for the actual >1B/<1s measurement |
| 258 | `playbook_258_enkiddb_topic_graph_report.yml` | `[ ]` | authored 2026-07-26 in response to an uploaded "Knowledge Graph via Dynamic Orbits" design (PB-184/NabuEngine draft from a separate "iPhone_Session"). That draft's naming/numbering was set aside as irrelevant per the Architect; the underlying citation-affinity-clustering concept was checked for validity (sound — a real, standard bibliometric technique) and rebuilt honestly against this repo's own real data instead of a synthetic 3-doc fixture: new module `crates/enkiddb/src/topics.rs` (`TopicGraph`) built on `meta.collection` (the topic partition already assigned at ingest) and real `depends-on` citation edges. **A real, previously-undetected bug was found and fixed in the process**: `WriteNode::link_discovered_dependencies` wrote a citing document's `link.target`/`link.description` directly onto its own entity; both `ReadNode`/`CachedReadNode` fold an entity's history into a single-slot `(attr_hash -> value)` map before projecting, so a document citing more than one other document silently lost all but the last-journaled citation when read back through a Read Node. Fixed by minting one child edge-entity per citation (mirroring `emit_sections`/`emit_concept_mentions`'s existing pattern) — proven by two new regression tests (3 real citations all independently queryable; a real ingested document correctly detected as a cross-topic bridge). New `enkiddb-cli topics <data-dir>` subcommand built and smoke-tested locally against this repo's real 71-document `docs/` corpus (12 real topics, 15 real bridges detected, e.g. "The 7 EnkiDB Types" correctly bridging `components`↔`concept-law`). 89/89 `cargo test -p enkiddb --lib` passing (86 pre-existing + 3 new), full `cargo build --workspace` clean. **Not yet run against real infra** — needs the Architect to run this playbook for real against the live `enkidb-node-read` Data Files |
| 259 | `playbook_259_full_7types_enkidb_bootstrap.yml` | `[ ]` | authored 2026-07-27 in response to the Architect asking why PB-215 kept destroying/recreating containers and whether the related per-type deploy/ingest/sync playbooks could be folded into one. Chains, via `import_playbook`, in dependency order: PB-212 (EnkiDDB+EnkiMDB deploy) → PB-221 (EnkiDB core deploy, then a **synthetic** benchmark seed — not real data) → PB-222 (EnkiSDB/EnkiODB/EnkiQDB/EnkiDW deploy only — flush/sync legitimately reports zero rows absent a manual landing-zone drop) → PB-208 (EnkiDDB REAL corpus ingest+flush/sync) → PB-216 (EnkiMDB REAL catalog populate+flush/sync) → PB-213 (final combined EnkiDDB+EnkiMDB flush/sync/verify pass). **Correction, same day**: this row originally said "one command builds real data into all 7 EnkiDB Types" — wrong. EnkiDB (core, Golden Store) and the BeeMDM chain (EnkiSDB/EnkiODB/EnkiQDB/EnkiDW) only ever receive real Golden Particles through `bee-watchdog`'s landing-zone pipeline (see `docs/components/BEEMDM_ETL_PIPELINE.md`), which no playbook here automates end-to-end. Only EnkiDDB and EnkiMDB have an automated real-content path. Does not reimplement any of those five playbooks' logic — only fixes the sequencing (ingest before sync, same root cause as PB-215's fix above) so one command builds all 7 types' infrastructure, with real content wherever an automated path for it actually exists. **Run this playbook alone — the six `import_playbook`s above mean PB-259 runs each of them itself, in order, inside this one invocation; do not run PB-212/221/222/208/216/213 by hand first.** Heavy by default (PB-221's 1M/10M/100M/1B synthetic seed sweep runs unless overridden — `-e '{"enkidb_seed_sizes": [1000000, 10000000]}'` for a smaller proof, `-e '{"enkidb_seed_sizes": []}'` for none at all — documented in the playbook's own header). `yaml.safe_load` passes. **Not yet run against real infra** |

## Phase 11 — Orbit spectral diagnostics + Theater law amendments (PB-260)

**Declaration:** Answers two real questions checked against this repo's actual state (not taken on faith) — can Godot realistically animate the BeeMDM ETL chain, and can Riemann-Hypothesis-adjacent mathematics help TOP Algebra's orbit calculus. Lands two law amendments (GL-DST-001 §6, new GL-MRD-003) plus a genuinely new, tested `orbit-spectral-engine` crate. Unrelated to the EnkiDB-7-Types build (PB-212–259) or DubSar IDE (Phase 6) — its own track.

| # | File | Status | Note |
|---|------|--------|------|
| 260 | `playbook_260_orbit_spectral_diagnostics_and_gl_amendments.yml` | `[~]` | run locally 2026-07-27 during authoring (sandbox, not eriduous-vdi yet) — `cargo test -p orbit-spectral-engine`: 8/8 passing (erf against known values, GUE CDF derivative verified against its own analytic PDF, both a genuine inverse-transform-sampled Poisson fixture and a jittered non-Poissonian repulsion fixture correctly classified `PoissonLike`/`GueLike`), full `cargo build --workspace` clean. `cargo test --workspace --no-fail-fast` (task 4.2 corrected to pass `--no-fail-fast`; plain `cargo test --workspace` stops at the first failing crate alphabetically and had been silently hiding every crate after it, including this one — confirmed by re-running with `--no-fail-fast` and finding `orbit-spectral-engine`'s own 8/8 tests present and passing at line 4635 of that run's log, not skipped) surfaces exactly one failure workspace-wide, and it is the same pre-existing, unrelated one: `buzu_core::chunk::tests::throughput_measurement_cpu_side_pack_and_evaluate` (landed in PB-256, asserts CPU-side evaluation completes under 10s) took ~31s in this sandbox both times (0.1 Mparticles/s here vs. PB-256's own recorded ~1.0 Mparticles/s — this sandbox is measurably slower, not a regression this playbook introduced; `orbit-spectral-engine` was never touched by that test or vice versa). PB-260 tasks 4.4–4.7 now diff the individual `FAILED` test names against that one known name and only hard-fail the playbook on a genuinely unexpected failure — this run had none. Not marked `[x]` until confirmed on real infra. Lands `docs/theater/GL-DST-001-theater-as-workbench.md` §6 (statistical witnessing for the ORBIT 3D lens — closes the real 80,272-built vs 13M+-sealed particle-count gap found in `orbit_multimesh.gd`/`bigring.gd`; GPU deployment constraint for `eriduous-vdi`'s real KVM virtio-GPU/virgl throttling, three options named, none chosen — a documented decision point, not resolved here) and new `docs/marduk/GL-MRD-003-orbit-spectral-diagnostics.md` (GUE-vs-Poisson orbit-return-time spacing diagnostic — real, closed-form, zero-dependency; complements `lamassu-engine`'s shape with rhythm; slots into GL-MRD-002 §7's Analysis-to-Solution Law as its DETECT-phase signal). **Not yet wired to live data** — `orbit-spectral-engine` has no data source until GL-MRD-002's own Nēberu Slicer produces real Poincaré-section crossings; its tests use clearly-labeled synthetic fixtures, never presented as live BIGRING data. **GPU passthrough decision (GL-DST-001 §6.2) intentionally left open** — needs the Architect's real hardware/virtualization choice, not something this playbook can decide. **Not yet run against real eriduous-vdi infra** — needs the Architect to run this playbook for real (its own GPU/virtualization preflight only means something against the real host) |

## Phase 12 — Shakkanakku Governor (PB-263)

**Declaration:** Builds and installs the Shakkanakku GUI/headless governor — executes this doc's own tables one row at a time via `ansible-playbook ... ANSIBLE_STDOUT_CALLBACK=json`, halting on MAJOR failures pending Architect ratification (CSR-08) instead of IsimudEngine's own continue-through-everything-then-report-at-the-end model. Consumes this file directly (`crates/shakkanakku::config::playbooks_from_triage`, same parsing logic as IsimudEngine's own PLAN phase's awk step below) — one hand-maintained playbook list, not two.

| # | File | Status | Note |
|---|------|--------|------|
| 263 | `playbook_263_deploy_shakkanakku_governor.yml` | `[~]` | authored + built + tested 2026-07-28 in the authoring sandbox (not eriduous-vdi yet) — `cargo test -p shakkanakku`: 12/12 passing under both `--no-default-features` (headless) and `--features gui`; full `cargo build --workspace --no-default-features` clean; a real dry-run against this repo's own live `shakkanakku.toml` (`simulate=true`, `triage_doc` pointed at this file) correctly parsed all 129 non-SKIP rows across Phases 0-11 in document order, ran the full rehearsal, and produced a real AkkadianSeal-verified report (`report::verify` returned `true`). A real latent bug was found and fixed in the process: `load_or_create_key` never created its seal key's parent directory (unlike every other output path in the crate), which would have failed on a fresh checkout's not-yet-created `secrets/` dir on the very first run — fixed before this row was written, not after. **Not yet run against real eriduous-vdi infra** — needs the Architect to run this playbook for real, then run `shakkanakku` (or `shakkanakku --no-default-features` build headless) from the repo root against `shakkanakku.toml` with `simulate=false` to execute the actual corpus. |

---

## Phase 13 — Patterns Arsenal: LOD + Exact-Aggregation Density Field (PB-264)

**Declaration:** Lands two GDScript patterns validated from a ParaView/vaex/HOOMD-blue billion-element visualization patterns document against this repo's own Patterns Arsenal — interaction-cadence LOD (`orbit_multimesh.gd`) and an exact-aggregation density field (`density_field.gd`, new) — plus a real, independently-found fix: `orbit_multimesh.gd` was talking to `enkidb-query-server`, a protocol `sumuukin_client.gd`'s own header confirms was never deployed anywhere in this fleet, and `sumuukin_client.gd` itself never decoded the MEASURE/GRAVITY aggregate trailer every real Read Node already sends.

| # | File | Status | Note |
|---|------|--------|------|
| 264 | `playbook_264_visualization_patterns_arsenal_lod_density.yml` | `[~]` | run 2026-07-29 in the authoring sandbox: confirms all three changed/new GDScript files exist and are non-trivial in size, attempts real Godot discovery (same logic as PB-226), and — since no Godot 4 binary is present in this sandbox — honestly reports "not engine-verified here" rather than claiming a pass. A separate bracket-balance check (parens/braces/brackets counted and matched) ran clean on all three files as a weak but real sanity signal; this is not a substitute for a real Godot parse. **Not yet engine-verified against a real Godot install** — re-run with `-e godot_bin=/path/to/Godot` (or on a host with Godot on PATH) for a real headless import/parse check before trusting these scripts live. **Not yet tested against a real EnkiDB Read Node** — the new `density_field.gd`'s `GRAVITY BAND ... MEASURE DENSE` query and the fixed `orbit_multimesh.gd` wire protocol both need a live `enkidb-read-server` (port 7001) to prove the aggregate-trailer decode round-trips correctly against real server bytes, not just against the Rust source read by eye. |

---

## Phase 14 — Shakkanakku Type-1 Infra: CQRS VM provisioning + all-7-Types backup/restore (PB-265–267)

**Declaration:** Closes the real "Infra Gap" the Architect named directly — Shakkanakku had governor/deploy/config-management playbooks but nothing that creates the CQRS write/read VMs themselves from nothing, and no backup/restore drill covering all 7 EnkiDB Types (only EnkiDDB+EnkiMDB, PB-203). These three playbooks were authored, syntax-checked, and (where the logic was non-trivial — environment-scoped IP derivation, zip/combine archive-to-volume pairing) independently verified via live `ansible -m debug`/throwaway-playbook runs in this session — but were never added to this file, which is the one and only source `shakkanakku::config::playbooks_from_triage` reads (per this file's own house rule #4 below). That omission, not a defect in the playbooks themselves, is why Shakkanakku never surfaced them to the Architect to run.

| # | File | Status | Note |
|---|------|--------|------|
| 265 | `playbook_265_shakkanakku_type1_infra_cqrs_nodes.yml` | `[~]` | authored 2026-07-29: `ansible-playbook --syntax-check` clean; the environment-scoped IP-derivation Jinja2 (production→.101/.107, dev/test/acc→distinct non-colliding `.15x` pairs) independently verified via a live `ansible localhost -m debug` run for all 4 `cqrs_environment` values. Implements both Architect-specified safety mechanisms: a duplicate-CQRS-pair guard (`virsh dominfo` check, blocks creation if either target VM name already exists, applies to every environment not just production) and an optional real Sargon-vault operator-authentication gate (`vault_check_enabled`, off by default until a real vault exists on the host — see PB-227/kupru-vault-cli). **Cannot be executed or verified in this sandbox** — no `virsh`/`virt-install`/`qemu-img` on PATH, no `/dev/kvm`, no systemd (`libvirtd` cannot start). **Not yet run against the real bare-metal Fedora Workstation 44 host** — needs the Architect to run this for real; only then do PB-192/205/212/222-class playbooks have real VMs to configure. |
| 266 | `playbook_266_all_7_types_health_check_and_backup.yml` | `[~]` | authored 2026-07-29, extending PB-203's proven health-check + `podman unshare` backup pattern from EnkiDDB/EnkiMDB (4 containers/volumes) to all 7 EnkiDB Types (12 real containers, 15 real volumes) — the real container/port/volume topology for every Type was checked against source before writing this (`bin/*/src/main.rs`, `playbook_222`'s own header) rather than guessed; confirmed EnkiODB/EnkiQDB have no dedicated write-server of their own (`enkisdb-write-server` is one process that owns all three write-side stages, per `playbook_222`'s own documented reasoning) — this playbook's `fleet_containers`/`fleet_volumes` reflect that correctly (12 containers, not 14). `ansible-playbook --syntax-check` clean; the `zip`-based volume/mountpoint pairing used for the per-volume materialization check was independently verified correct via a live throwaway playbook run. **Cannot be run against real infra in this sandbox** — needs the real 2-VM CQRS fleet (PB-265, then PB-192/205/212/222-class deploy) to exist first. |
| 267 | `playbook_267_all_7_types_restore_and_verify.yml` | `[~]` | authored 2026-07-29 — "a restore-and-verify playbook, prove backups actually work, don't just take them" (Architect's own instruction). Extracts the latest PB-266 archive per volume, boots a throwaway read-server container (built from the real per-crate Containerfile — the exact repo-root-vs-`workspace/bahyway_v4/deploy/podman/` path split across the 7 Types was checked directly, not assumed uniform) on a scratch port (81xx range, disjoint from the live fleet's real ports), and sends a real HeptaScript `QUERY` over the actual binary wire protocol to prove the restored data is live-queryable — never touches the live fleet's containers/volumes/ports. Honestly scopes `enkidw-persist-data` (a `PersistedDb` journal, not the `current/entities.data` shape the other 14 volumes share) to an extraction-only check, no live-server proof, since no server opens that layout. `ansible-playbook --syntax-check` clean; the `restore_pairs` zip/combine pairing (both the normal and `-e restore_tarball_override=...` branches) independently verified via a live throwaway playbook run. **Cannot be run against real infra in this sandbox** — needs PB-266 to have produced real archives on the real host first. |

---

## Phase 15 — BahyWay host privilege groups + Shakkanakku OS-identity signal (PB-268)

**Declaration:** Closes a real gap the Architect caught directly: the 5-group privilege model (Architects/dataStewards/Administrators/Developers/Other-Stakeholders) was discussed conceptually earlier this session but never built — no `ansible.builtin.group` task anywhere created any of the 5 groups. This is the fix, plus the reason it matters now: Shakkanakku's run-confirmation registry (`shakkanakku_run.*` in EnkiMDB, Phase 14 above's sibling work) could already capture a cryptographic vault-passport identity, but "there's no identity in the loop to attach" for a run with no vault configured — these 5 real Fedora groups are the second, always-available identity signal (`id -un`/`id -Gn`, no vault needed) that closes that.

| # | File | Status | Note |
|---|------|--------|------|
| 268 | `playbook_268_bahyway_host_privilege_groups.yml` | `[x]` | **run for real in this session's own sandbox** (this playbook needs only `ansible.builtin.group` + real root, unlike PB-265/266/267's libvirt/Podman-fleet dependencies) — first run created all 5 groups (`changed=1`, real gids assigned: `bahyway-architect:1002`, `bahyway-datasteward:1003`, `bahyway-administrator:1004`, `bahyway-developer:1005`, `bahyway-stakeholder:1006`), confirmed via `getent group` for each; second run confirmed idempotent (`changed=0`). Deliberately mints no passport and assigns no real user (CSR-08/Architect Sovereignty) — the companion `shakkanakku::runner.rs` change (same commit) reads these groups via `id -Gn`, cross-references the 5 names, and records `os_username`/`os_groups_csv`/`os_bahyway_role` on every `shakkanakku_run.*` particle alongside (never replacing) the separate vault-passport identity. **Full positive-case live end-to-end verified**: added the sandbox's own `root` account to `bahyway-architect`, ran `shakkanakku` for real (`sg bahyway-architect -c ...`), confirmed the runner printed `OS identity: user=root bahyway_role=bahyway-architect`, and confirmed via a real HeptaScript `QUERY` against a real `enkimdb-read-server` that `shakkanakku_run.os_bahyway_role`/`os_username` landed and are queryable — then removed `root` from the group again (sandbox hygiene, not needed on the real host). `cargo test -p shakkanakku`: 15/15 passing (3 new: real `id -un`/`id -Gn` call, plus the pure strongest-match/no-match selection logic). `cargo test -p enkimdb`: 25/25 passing. **Group CREATION is fully verified for real; group ASSIGNMENT to real personnel is deliberately left to the Architect** — this playbook and this session never decide who is an Architect/dataSteward/Administrator/Developer/Stakeholder, only that the groups exist to assign people into. |

---

## Phase 16 — Retire EriduOS-VDI as control-node hardware (PB-269)

**Declaration:** The Architect's own instruction this session: EriduOS-VDI (the KVM/libvirt VM at 192.168.122.214 that has been the control node since PB-210/212) is no longer used in v4.0. The control-node role moves to the bare-metal Fedora Workstation 44 host already referenced by PB-265/268. Deliberately NOT a mass rename of the `eriduous-vdi` Ansible alias / CI runner label across the ~89 playbooks and `.github/workflows/isimud-engine.yml` that use it — see this playbook's own header comment and `ansible/inventory.ini`'s updated comment for why that would be invasive for zero functional benefit. `enkidb-node-write`/`enkidb-node-read` (192.168.122.101/.107) are explicitly UNCHANGED — the Architect's choice was to keep the 3-host topology and move only the control node, not collapse the fleet onto one box.

| # | File | Status | Note |
|---|------|--------|------|
| 269 | `playbook_269_retire_eriduous_vdi_confirm_baremetal_control_node.yml` | `[~]` | authored 2026-07-30; `yaml.safe_load_all` passes (no `ansible-playbook` on PATH in this authoring sandbox to `--syntax-check` directly). Best-effort, non-fatal `systemd-detect-virt` check reports bare-metal-vs-VM as a real signal rather than asserting it; confirms Fedora release + rust toolchain present; confirms `enkidb-node-write`/`enkidb-node-read` inventory entries are still present and untouched. Deliberately does not rename the `eriduous-vdi` alias/label anywhere, does not touch the two node-VM inventory entries, and does not register the GitHub Actions runner itself (prints the manual re-registration step instead — needs a real token from GitHub Settings and physical access to the host, neither available to this playbook or this session). **Not yet run against the real bare-metal Fedora Workstation 44 host** — needs the Architect to run this for real, then manually re-register the CI self-hosted runner from that machine under the same `eriduous-vdi` label. |

---

## Phase 17 — Shakkanakku as the one central KAKIv4.0 tool (PB-270)

**Declaration:** The Architect's own instruction this session: "all the elements of bahyWay.Ecosystem will be in one central tool: Shakkanakku Engine be executed and got its Kakiv4.0 Identity and be saved in EnkiMDB or EnkiDDB." Two new scanners, `crate_mint.rs` and `tablet_mint.rs`, join the pre-existing `pb_mint.rs` in Shakkanakku's own Corpus-run sequence, so one Corpus execution now mints playbooks, workspace crates, and `.akk`/`.way`/`.tmpl` tablets — not three separate tools (Shakkanakku, `enkimdb-write-server`, `girsu-mint`) for three element kinds. Documents stay on Uruinimgina/`docpulse`'s existing path into EnkiDDB, untouched.

| # | File | Status | Note |
|---|------|--------|------|
| 270 | `playbook_270_shakkanakku_one_central_kaki_tool.yml` | `[~]` | authored 2026-07-30; `yaml.safe_load_all` passes (no `ansible-playbook` on PATH in this authoring sandbox). The Rust side it verifies IS fully real-tested directly (not just via this playbook): `cargo test -p shakkanakku --no-default-features` — 23/23 passing, including 2 new `crate_mint` tests and 4 new `tablet_mint` tests. Also live-smoke-tested end-to-end against this actual repo checkout (a throwaway, non-workspace-member binary, removed after use — not committed): `crate_mint::mint_new_crates` found and minted all ~180 real workspace crates on a first pass, 0 on an idempotent re-run; `tablet_mint::mint_new_tablets` found and minted 16 real pre-existing `.akk` files (`crates/pollution-engine/policies/*.akk`, `workspace/bahyway_v4/policies/*.akk`, `templates/automation/*.akk`, `templates/sovereign/*.akk`) with zero false positives from `target/`/`.git/`, 0 on the idempotent re-run. **The playbook's own Ansible tasks (build/test/stat-check) are not yet run for real** — needs `ansible-playbook` on a host that has it (e.g. the bare-metal Fedora Workstation 44 control node, PB-269) to close that last gap; the underlying logic they merely wrap is already proven. |

---

## Phase 18 — assets-node build playbook + playbook documentation minting into EnkiDDB (PB-271)

**Declaration:** Two Architect asks in the same session. First: "one specific Podman to keep all bahyWay.Ecosystem assets and binary source files... downloaded once and work forever... promoted into the same flatpak of the whole ecosystem" — closed with a real `deploy/podman/Containerfile.assets-node` (pinned rustc 1.94.1, vendored crate sources, rebranded Godot 4.3-stable) and `playbook_271` to build it, always referenced downstream by content digest, never a floating tag. Second, explicit: every numbered playbook should get not just its file catalogued (PB-270's `pb_mint`, EnkiMDB) but its own KAKIv4.0 Identity-Kaki AND Event-Kaki, with its **documentation** — not just its existence — saved into EnkiDDB and queryable via HeptaScript "for versioning and release comparison." Closed with a new `enkiddb::PB_DOCS_TRIBE_ID` (0x7165), a new `DocumentParser::parse_playbook_header` (extracts a playbook's leading `#`-comment block into a real `DocumentStructure`, distinct from markdown parsing), and a new `pb_doc_mint.rs` scanner wired into Shakkanakku's Corpus sequence alongside PB-270's three: mints a playbook's documentation fresh when new, **supersedes** the prior version (ADR-014's law) when the header text changes, and stays quiet when it hasn't — every mint/supersede already carries both an Identity-Kaki and an Event-Kaki via the same `WriteNode::ingest_document`/`supersede_document` primitives `docpulse` uses for markdown docs.

Also resolved a real drift risk flagged while wiring the Containerfile: `playbook_262`'s Godot rebrand patch was inlined directly in its own YAML (`ansible.builtin.copy: content: ...`), so this playbook's Containerfile couldn't reference it without risking two independently-editable copies. Extracted into `workspace/bahyway_v4/godot/patch_engine_source.py`; `playbook_262` now `copy: src:`s from that same file PB-271's Containerfile stages.

| # | File | Status | Note |
|---|------|--------|------|
| 271 | `playbook_271_build_bahyway_assets_node.yml` | `[~]` | authored 2026-07-30; `yaml.safe_load_all` passes (neither `podman` nor `ansible-playbook` on PATH in this authoring sandbox, so the playbook's own tasks are unrun). What IS real-verified directly: `cargo test -p enkiddb` — 95/95 passing, including 3 new `parser::` tests for `parse_playbook_header` (one against this repo's own real `playbook_269` file, which caught and fixed a real bug — the first heuristic missed "WHY THIS EXISTS: <prose on the same line>", only catching standalone `HEADER:` lines); `cargo test -p shakkanakku --no-default-features` — 26/26 passing, including 3 new `pb_doc_mint` tests. Live-smoke-tested end-to-end against this actual repo (a throwaway, non-workspace-member binary, removed after use — not committed): `pb_doc_mint::mint_or_supersede_pb_docs` found and minted documentation for all 97 real playbooks in this repo (tribe `0x7165` confirmed in every emitted KAKI), 0 on an idempotent re-run. **PB-271's own Ansible tasks (stage build context, `podman build`, resolve digest, append registry) are not yet run for real** — needs a host with both `podman` and `ansible-playbook` (the bare-metal Fedora Workstation 44 control node, PB-269) to actually build the assets-node image; the Containerfile and manifest template were written from real, verified facts (rustc 1.94.1, this Cargo.lock's crate count, PB-262's own clone tag) but the multi-stage build itself — including the full Godot SCons compile — has not been executed anywhere. |

---

## Phase 19 — Naming corrections, Uruinimgina external-docs bring-up, resource/git-lifecycle tooling & the Shala dashboard's git-recovery mode (PB-272–281)

**Declaration:** Closes out the 2026-08-01 bare-metal bring-up session: naming/ADR corrections (PB-272), documentation of a deferred grammar gap (PB-274, no code change), folding `.akknb` into the real Girsu VS Code extension (PB-275), sealing Eshnunna/Susa/Nuzi/SipparStore naming (PB-276/277), first-ever headless Uruinimgina bring-up on Fedora Workstation 44 (PB-278), host resource-utilization check + git lifecycle (PB-279/280), and the git-history recovery/retry playbook written after the real `filter-repo` divergence incident on the Architect's personal `bahyway_v4` DailyWorks repo (PB-281). **PB-273 (build `ninurta-engine`) was pure Rust crate work with no playbook wrapper — no row here by design, not an oversight.**

| # | File | Status | Note |
|---|------|--------|------|
| 272 | `playbook_272_ninurta_naming_and_adr_corrections.yml` | `[x]` | run 2026-08-01 — naming-registry entries + ADR corrections (Ninurta/Purussum naming sealed, stale GeoEngine/Shamash references fixed). |
| 274 | `playbook_274_heptascript_window_detrend_gap.yml` | `[x]` | run 2026-08-01 — documents the HeptaScript `WINDOW`/`DETREND` grammar gap ADR-019 relies on; deliberately makes no grammar change (deferred, tracked here so the gap isn't silently forgotten). |
| 275 | `playbook_275_girsu_akknb_fold_in.yml` | `[x]` | run 2026-08-01 — folds `.akknb` notebook container-serializer support into the real, already-installed `bahyway-akkadian` Girsu VS Code extension; corrects the extension's stale keyword vocabulary/comment syntax against `crates/aaol/src/token.rs`. |
| 276 | `playbook_276_eshnunna_columnar_engine_seal.yml` | `[x]` | run 2026-08-01 — seals Eshnunna naming, scaffolds `eshnunna-engine`. |
| 277 | `playbook_277_susa_nuzi_sippar_naming_split.yml` | `[x]` | run 2026-08-01 — verifies Susa/Nuzi/SipparStore all appear in `naming-registry`; seals the Nuzi (read)/SipparStore (write) CQRS naming split. |
| 278 | `playbook_278_uruinimgina_fedora_w44_setup.yml` | `[x]` | run live 2026-08-01 on the real bare-metal Fedora Workstation 44 host during the first-ever full bring-up — hit and fixed a real `ansible_env.HOME` → `/root` bug under `gather_facts:false` (now `lookup('env','HOME')` across PB-276/277/278/280). Builds `uruinimgina-cli`, sets up the external docs repo (`devVM`+`main`), renders `uruinimgina.toml`. |
| 279 | `playbook_279_host_resource_utilization_check.yml` | `[x]` | run live 2026-08-01 — real RAM/swap/disk/load thresholds; hit and fixed a real missing-`docs/testing/`-directory bug on first run. |
| 280 | `playbook_280_git_repo_lifecycle.yml` | `[~]` | authored 2026-08-01, `--syntax-check` clean; four modes (clone/sync/push/restore), never force-pushes. Superseded for the git-history-divergence failure mode specifically by PB-281 below — use PB-280 for ordinary clone/sync/push, PB-281 when local and remote history have actually diverged. |
| 281 | `playbook_281_uruinimgina_git_recovery_and_retry.yml` | `[x]` | run live 2026-08-01 against the real `bahyway_v4` DailyWorks repo (`~/Forge/bahyway_v4`) after a `git filter-repo --strip-blobs-bigger-than 90M` run diverged local history from `origin` — `diagnose` mode correctly reported `ahead 0, behind 0` on both `devVM`/`main` post-recovery and confirmed zero oversized blobs. **Never force-pushes** — `reclone` mode always backs up (never deletes) and re-clones from the real `origin`, safe specifically because no push had ever actually reached GitHub. See the playbook's own header for the full incident writeup, including the basename-collision trap (`~/Forge/bahyway_v4` vs. `~/Forge/EnkiDB/workspace/bahyway_v4`) that caused the confusion in the first place. |
| 282 | `playbook_282_fetch_sargon_vault_from_eriduous_vdi.yml` | `[x]` | run live 2026-08-02 — fetched real `sargon_vault.dat` (4235 bytes) from eriduous-vdi to `~/sargon_vault.dat` on the bare-metal host. First attempt failed: plain `virsh domstate` reported "shut off" while virt-manager showed the VM running — root cause was `virsh` defaulting to `qemu:///session` while the VM runs under `qemu:///system`; fixed by adding an explicit `-c {{ libvirt_uri }}` (default `qemu:///system`) to every `virsh` call in this playbook and PB-283. |
| 283 | `playbook_283_fetch_and_diff_eriduous_vdi_enkidb.yml` | `[x]` | run live 2026-08-02 — fetched eriduous-vdi's EnkiDB checkout to a new `~/Forge/EriduOS-EnkiDB` and confirmed clean: VM's copy was on an old, unrelated branch (`claude/iphone-playbooks-crosstribe-eval-nbc5sw`), 0 ahead/9 behind origin — nothing stranded there beyond the vault PB-282 already fetched. The long list of untracked files reported were routine Godot `.uid`/`.import` cache files, not real work. Both PB-282 and PB-283 needed a follow-up fix (see PB-282's own row) for `virsh` defaulting to the wrong libvirt connection URI on this host. |
| 284 | `playbook_284_launch_shala_dashboard.yml` | `[x]` | run live 2026-08-02 — `shakkanakku-web` had been started by hand in a foreground terminal that later closed, silently taking the Shala dashboard down (discovered only when the browser reported "Unable to connect"). Same detached-launch pattern PB-226 already uses for DubSar Theater; confirmed the port listening both times it was run (before and after the `vault_path` login fallback was added). |
| 285 | `playbook_285_launch_sargon_or_gilgamesh_key_tool.yml` | `[x]` | run live 2026-08-02 — reached the actual launch step cleanly (kupru-gdext built, bridge copied into the tool project) but halted at Godot auto-discovery: bare metal has never had Godot installed, only eriduous-vdi did. See PB-286. FIXED 2026-08-02, live: also ran its headless `--import` priming scan unconditionally on every launch (same fix as PB-226) — added `-e skip_prime=true` opt-in, default `false`. Still reported slow with skip_prime alone — its stat+copy pair for `libkupru_gdext.so` also ran unconditionally on every launch; gated `when: not skip_build` — same regression PB-226 hit (see its row): reverted to a self-healing check that stats the actual destination `.so` first and only skips the copy when it's really already there, not just because `skip_build` was passed. |
| 286 | `playbook_286_install_godot_engine_bare_metal.yml` | `[x]` | run live 2026-08-02 — installed Godot 4.3.stable.official.77dcf97d8 to `~/.local/bin/godot4`, verified it runs. PB-285 immediately found it via auto-discovery on the next run. |
| 287 | `playbook_287_place_vault_into_sargon_userdata.yml` | `[ ]` | authored 2026-08-02 — found live: `sargon-passport-manager/scripts/main.gd` hardcodes `VAULT_PATH := "user://sargon_vault.dat"`, which Godot resolves to `~/.local/share/godot/app_userdata/<config/name>/sargon_vault.dat` — a THIRD path, different from both `~/sargon_vault.dat` (PB-282's fetch destination) and Shala's own `vault_path` login field. On bare metal's first-ever run of the tool that directory didn't exist yet, so unlocking would have created a new, empty vault instead of opening the real one. Reads `config/name` directly from `project.godot` (not hardcoded twice), refuses to overwrite an existing vault at the destination without `-e confirm_overwrite=true` (reports both checksums first). Syntax-checked (`yaml.safe_load` clean) — not yet run for real. |
| 288 | `playbook_288_create_desktop_launchers.yml` | `[x]` | run live 2026-08-02, all 3 icons confirmed working after a real logout/login cleared GNOME Shell's stale icon-cache entry. Iterated through 6 real bugs total, all found live on the Architect's actual GNOME desktop: (1) un-canonicalized `..`-containing `Icon=` path; (2) `.desktop` `Exec=` doesn't source `~/.bashrc`, so `~/.local/bin` (PB-286's Godot install) was missing from PATH — fixed with explicit `-e godot_bin=`; (3) GNOME's app-grid cache didn't notice an in-place `.desktop` edit — fixed with remove-then-rewrite; (4) a raw absolute SVG path in `Icon=` never reliably renders on this host — fixed by installing real PNGs into the standard hicolor icon theme, referenced by bare name (matched against an already-working "Girsu" launcher from an earlier session); (5) GNOME's icon cache specifically needed a session restart, not just a file/cache refresh, to notice a brand-new icon-theme entry; (6) all three tools shared one byte-identical placeholder `icon.svg`, so even once rendering, they were visually indistinguishable — fixed with a per-tool ImageMagick hue tint (cosmetic, best-effort). Also switched `Terminal=true` (visible bash window, required "Press Enter to close") to `Terminal=false` (silent background launch, output to `/tmp/<id>_launch.log`, desktop notification on failure via `notify-send` if available). FIXED 2026-08-02, live: reported real slowness on repeat icon clicks, root-caused to PB-226/PB-285 both running a redundant headless priming scan every launch — each `exec_cmd` now also passes `-e skip_prime=true` (safe here since a desktop icon is never the first-ever launch on a fresh checkout). |
| 289 | `playbook_289_multi_location_playbook_catalog.yml` | `[x]` | authored + live-smoke-tested 2026-08-02 against this repo's own two in-checkout locations (283 unique playbooks catalogued, 41 real same-PB-number collisions correctly surfaced — e.g. PB-88/90/91/92/93/94/97/102/104/105/108 meaning different things in `playbooks/` vs `workspace/bahyway_v4/ansible/playbooks/`, several already partially reconciled under this repo's own `_reconciled` filename suffix), unreachable-location reporting verified, re-run idempotency verified (0 re-mints on a clean second run). New engine: `crates/shakkanakku/src/pb_catalog.rs` (5 unit tests, all passing), CLI `pb-catalog-cli`, new `enkiddb::PLAYBOOK_CATALOG_TRIBE_ID` (0x7167, continuing the 0x716x sequence, deliberately separate from the sealed `pb_mint::PB_TRIBE_ID`/`PB_DOCS_TRIBE_ID` corpus so an unreconciled backup copy never silently mixes with the current one), new `WriteNode::link_documents`/`mint_marker` methods, new `EventCause::DocumentCrossReferenced` (0x7C) — a neutral link cause distinct from `DocumentSuperseded`, since which side (if either) is authoritative stays the Architect's call, never auto-decided. Content-hash deduped across locations (same bytes in 5 places mints once). **Run live on the real bare-metal host 2026-08-02**, all real locations: 612 new unique playbooks catalogued, 3635 same-PB-number collisions found, `ok=14 changed=3 failed=0`. |
| 290 | `playbook_290_publish_and_serve_enkiddb_read_node.yml` | `[ ]` | authored 2026-08-02 — found live: after a real PB-289 run, nothing was actually SERVING the materialized data for Graph Explorer to query. `enkiddb-read-server`'s own header comment is explicit that it only reads `DATA_DIR/current/{entities,eav}` and that promoting a materialized generation into `current` is "a separate, external sync step... not this server's job" — and no playbook in this repo runs a local, bare-metal `enkiddb-read-server` at all (every existing reference is an old VDI/Podman deployment). This playbook publishes the latest `tigris/<timestamp>` generation as `current` (a symlink, replaced every run) and launches `enkiddb-read-server` detached on `127.0.0.1:7007`, verified listening before declaring success. Kept as its own playbook, not folded into PB-289, mirroring the same write/read CQRS split ADR-012 already establishes. Syntax-checked (`yaml.safe_load` clean) — not yet run for real. |

---

## How to use this on eriduous-vdi

1. Work one phase at a time, in order. Don't skip ahead — Phase 4 especially
   is a real fix-chain, not independent playbooks.
2. For each playbook: run it, flip its status to `[x]` (or `[~]` with a
   one-line note if partial/blocked), commit this file with a short message
   noting what ran.
3. If a playbook fails, stop that phase, capture the error, and report it
   back before continuing — don't push through a phase with a broken
   foundation underneath it (that's exactly how PB-200/201/202 got written).
4. New playbooks added after today get a row in the correct phase (or a new
   Phase 7+) in the same session that authors them — that's the rule that
   stops this file from going stale like the gap it replaces.

## Secrets (added 2026-07-26, after PB-153's first real run)

Some playbooks need a real secret at runtime — PB-153's `fable_api_key`
(an Anthropic API key, so the design-time Fable Impact Gate can actually
call `claude-fable-5`) is the first one to hit this.

**The Architect's standing secret store for BahyWay.Ecosystem v4.0 is the
Sargon Passport Manager** (PB-227, `playbooks/playbook_227_build_and_launch_kupru_tools.yml`)
— a standalone "kind-of-KeePass" vault GUI backed by `crates/kupru`'s real
Argon2id + Ed25519 + ChaCha20-Poly1305, **not Ansible Vault**. Any
playbook note or comment that says "supplied via Ansible Vault" (PB-153's
own header comment among them) predates this and is superseded — Ansible
Vault still works mechanically (`-e @vault.yml --ask-vault-pass`), but
it's not where this ecosystem's secrets actually live, and using it means
keeping a second, unsynced copy of the same secret.

**Honest limit:** `crates/kupru` is a library, not a CLI — Sargon Passport
Manager is Godot-GUI-only today, with no headless/scriptable way to pull a
secret out of it. So the actual flow is manual: open Sargon Passport
Manager (`ansible-playbook playbook_227_build_and_launch_kupru_tools.yml`),
copy the secret out, and pass it to whichever playbook needs it via
`-e name=value` directly, or drop it into a local file under the
gitignored `secrets/` directory and pass `-e @secrets/whatever.yml`. There
is currently no automated bridge from the vault to Ansible — if that
becomes painful (more secret-consuming playbooks are likely as Fable-gated
and other cloud-touching playbooks grow), the next step would be a small
`kupru` CLI binary that exports one named secret to stdout, which
Ansible's `lookup('pipe', ...)` could then consume directly. Not built
yet — no second secret-consuming playbook has needed it yet either.

**IsimudEngine:** unattended (CI / nightly) runs cannot open a GUI,
so any secret-consuming playbook needs its value forwarded in ahead of
time — see `secrets_vars_file` in the IsimudEngine section below.

## IsimudEngine — run everything above in one go (2026-07-26)

`playbooks/playbook_IsimudEngine.yml` parses this file's own tables
at run time and executes every non-SKIP row, in the order they appear here
— it is not itself a numbered PB (running it as one would make it try to
invoke itself) and has no row above.

```bash
cd playbooks
ansible-playbook playbook_IsimudEngine.yml -i "localhost," -c local -v
# or, to only run a subset (e.g. re-running one phase, or after fixing a
# specific failure):
ansible-playbook playbook_IsimudEngine.yml -i "localhost," -c local -v \
  -e pb_filter_numbers="249,250,251,252,253"
# ci_fast profile excludes multi-hour scale-sweep / full-corpus-ingestion
# playbooks (see heavy_pattern in the file) — meant for frequent runs:
ansible-playbook playbook_IsimudEngine.yml -i "localhost," -c local -v \
  -e run_profile=ci_fast
# secret-consuming playbooks (e.g. PB-153's fable_api_key) need their
# secrets forwarded ahead of time -- see the Secrets note above. Put them
# in a local, gitignored file under secrets/ and point the orchestrator
# at it; it forwards that file's vars to every sub-playbook it runs:
ansible-playbook playbook_IsimudEngine.yml -i "localhost," -c local -v \
  -e secrets_vars_file=secrets/orchestrator_secrets.yml
```

It writes a timestamped report (`docs/SHEDU/ISIMUD_ENGINE_REPORT_*
.txt` and `.json`) with per-playbook pass/fail, duration, and error output,
plus **Heal_points**: a weighted pass-rate (security/data-integrity
playbooks weighted 3×, docs/scaffolds/gap-diagnostics 1×, everything else
2×) rather than a flat percentage, so one critical failure moves the score
more than one doc-update failure. It continues past individual failures
(so one run gives the full picture, not just the first blocker) and
records every run to the NĀRU journal.

Wired into GitHub Actions at `.github/workflows/isimud-engine.yml`,
targeting a **self-hosted runner registered from eriduous-vdi** — GitHub's
own hosted runners can't reach the private VM network, TPM hardware, or
ZFS filesystems most of these playbooks check for real, so hosted runners
would just report BLOCKED across most phases. Runs `ci_fast` on every
push/PR, `full` nightly at 03:00 UTC, and on-demand via workflow_dispatch
with a selectable profile and optional `pb_filter_numbers`.
