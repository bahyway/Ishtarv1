# BahyWay.Ecosystem v4.0 — Playbook Deployment Reference (Reconciled v2)
## SCP Transfer + ansible-playbook Launch Commands
Generated: 2026-07-26

This reference lists playbooks that actually exist in the repo as
reconciled *_reconciled.yml files. It does not list PB-119-136 as
playbook files (they never existed as files), but see the RM-001
addendum v2 above for what content in that range is real anyway.

**Everything below this point (Infrastructure Overview / Sync
convention / PB-111-145 list) describes the state before PB-212's real
2-VM CQRS split and the later git-pull-based workflow (both landed
2026-07-26, same day as this file, but after it) — it's kept for
historical record, not as current instructions. For the playbooks that
actually run against the real `enkidb-node-write`/`enkidb-node-read`
split today, use the section immediately below instead.

---

## PB-272 to PB-275 — NinurtaEngine (Purussûm Calculus) + Girsu `.akknb` fold-in

Added 2026-08-01. All four are `hosts: localhost, connection: local` —
**none of them need `-i ansible/inventory.ini`**, they don't touch
`enkidb-node-write`/`enkidb-node-read` at all. Run from `playbooks/`
exactly like the localhost-only playbooks elsewhere in this doc:

```bash
cd ~/Forge/EnkiDB/playbooks
ansible-playbook <playbook_file>.yml -i "localhost," -c local -v
```

**Run order: PB-273 before PB-272** — PB-272 verifies
`crates/ninurta-engine` exists and fails loudly if it doesn't. PB-274 and
PB-275 are independent of both and of each other.

| PB | File | What it does |
|---|---|---|
| 272 | `playbook_272_ninurta_naming_and_adr_corrections.yml` | Verifies `ninurta-engine` and the 3 corrected ADRs exist, runs `cargo test -p ninurta-engine` (asserts ≥23 passed, 0 failed) and `cargo test -p naming-registry` (asserts 0 failed) — confirms the NinurtaEngine/Purussûm `naming-registry` seal and the GeoEngine→`bahyway-algebra`/ShamashEngine→NinurtaEngine corrections hold. |
| 273 | *(scaffolded directly as real Rust this session, no playbook file — `crates/ninurta-engine` itself is the deliverable)* | Real CSD/bifurcation-detection math: detrending, restoring-rate λ (OLS regression), lag-1 autocorrelation, Fourier-surrogate significance testing (direct DFT, zero external deps), the composed Purussûm Calculus verdict, GOLDEN/FUZZY/DEAD λ trichotomy. 23/23 tests. |
| 274 | `playbook_274_heptascript_window_detrend_gap.yml` | Documents-only, builds nothing: verifies `WITNESS`/`SYNC` are real HeptaScript tokens and `WINDOW`/`DETREND`/`OBSERVE` and `HS-EXT-001` are **not** real (an uploaded design tablet's citation didn't hold up), writes a gap report, stops short of any parser change pending Architect grammar sign-off (CSR-08). |
| 275 | `playbook_275_girsu_akknb_fold_in.yml` | Verifies `.vscode/extensions/bahyway-akkadian` (the real, already-installed Girsu extension — NOT a second "akkadian-aol" extension) now declares the `.akknb` notebook type, uses the real `crates/aaol/src/token.rs` keyword vocabulary (`particle`/`tribe`/`rule`/`equation`/`guard`/...) instead of a stale placeholder set, and uses `#` line comments (matching the real AAOL lexer, which has no block-comment support) instead of `//`. Re-runs the existing `install-extensions.sh` symlink installer — no new Ansible/vsce/VSIX pipeline was introduced.

See `docs/14_decisions_adr/adr_017_three_layer_pdm_paradigm.md`,
`adr_018_topological_engine_division.md`, and
`adr_019_girsu_csd_notebook.md` for the full design/correction record
these four playbooks implement and verify.

---

## PB-276 to PB-278 — Eshnunna/Susa/Nuzi naming seal + Uruinimgina headless ingestion

Added 2026-08-01. All three are `hosts: localhost, connection: local`, same
invocation convention as PB-272 to PB-275 above. PB-276 and PB-277 are
independent of each other; PB-278 is independent of both but assumes the
workspace builds (any earlier PB already proves that).

```bash
cd ~/Forge/EnkiDB/playbooks
ansible-playbook <playbook_file>.yml -i "localhost," -c local -v
```

| PB | File | What it does |
|---|---|---|
| 276 | `playbook_276_eshnunna_columnar_engine_seal.yml` | Verifies `crates/eshnunna-engine` and ADR-021 exist, runs `cargo test -p eshnunna-engine` (asserts ≥5 passed, 0 failed) and `cargo test -p naming-registry` (0 failed). EshnunnaEngine is a real columnar data-file engine (surrogate u32 -> fixed byte offset -> mmap'd column value via `enkidb-storage`'s `AppendWriter`/`MmapReader`) fixing the diagnosed journal-walk retrieval bottleneck behind HeptaScript's 5s/100-particle -> hours/10,000-particle degradation. Not yet wired into the live read path — that's tracked separately, not claimed done here. |
| 277 | `playbook_277_susa_nuzi_sippar_naming_split.yml` | Verifies `Susa`/`Nuzi`/`SipparStore` all appear in `naming-registry`, runs `cargo test -p susa-engine` (asserts ≥9 passed) and `cargo test -p naming-registry` (0 failed). Susa is a *registration* of the already-real, already-tested `crates/susa-engine` — closes the open "does client-document ingestion need a new engine name" question (no, SusaEngine already is it). Nuzi (retrieval/query side) and SipparStore (ingest/write side) are sealed as a CQRS pair, matching the Write=Journal/Read=Datafiles shape used elsewhere in the 7 Types EnkiDB — resolves the Nuzi-vs-SipparStore naming conflict without discarding either name. |
| 278 | `playbook_278_uruinimgina_fedora_w44_setup.yml` | Builds `uruinimgina-cli` (new headless face for `docpulse`, no `gui` feature required), idempotently creates the external docs repo on branch `devVM` with a `main` branch present (the two hard git requirements `docpulse` enforces), creates the archive/manifest/EnkiDDB-root directories OUTSIDE that repo, and renders `uruinimgina.toml`. Deliberately does **not** run the actual Reform & Pulse itself — minting and git push are attended Architect actions (CSR-08); the final task prints the exact command to run by hand in a real terminal, where halts prompt for retry/skip/abort on stdin. Override any path with `-e uruinimgina_docs_repo=...` etc. — see `docs/20_meta_engine/URUINIMGINA_EXTERNAL_DOCS.md` for what each field means. |

See `docs/14_decisions_adr/adr_021_eshnunna_susa_nuzi_naming_seal.md` for
the full naming record PB-276/277 verify, and
`docs/20_meta_engine/URUINIMGINA_EXTERNAL_DOCS.md` for the external-repo
ingestion mechanics PB-278 sets up.

---

## PB-279 and PB-280 — host resource-utilization check + git lifecycle

Added 2026-08-01, so a full bare-metal bring-up never needs a manual git
command or a manual `btop` glance to know whether the host has headroom.

| PB | File | What it does |
|---|---|---|
| 279 | `playbook_279_host_resource_utilization_check.yml` | Gathers real host facts (Ansible `setup` + raw `free`/`df`/`uptime`/`ip -s link`/`swapon`), computes available RAM (true `MemAvailable`, not just `MemFree`), swap-used %, disk headroom per real mount, and load-per-vCPU. Warns (does not fail) past configurable thresholds (`min_available_ram_gb=8`, `max_swap_used_pct=50`, `min_available_disk_gb=50`, `max_load_per_core=1.0`), writes a timestamped snapshot to `docs/testing/PB279_RESOURCE_UTILIZATION_<date>.txt`. Run before AND after a heavy bring-up (e.g. PB-259) to compare. |
| 280 | `playbook_280_git_repo_lifecycle.yml` | Four modes via `-e git_action=<mode>`: `clone` (SSH clone of `bahyway/EnkiDB`, checks SSH access first, no-ops if already present), `sync` (default — fetch, refuses to pull over a dirty tree, tags a local-only `pre-sync-<timestamp>` restore point before every fast-forward pull, reports rather than guesses on real divergence), `push` (current branch only, refuses master/main directly, requires `-e confirm_push=true` as a second explicit flag), `restore` (`git reset --hard <restore_tag>`, requires the tag to exist and `-e confirm_restore=true` if the tree is dirty). Never force-pushes, never discards work without a named restore point existing first. |

Both are `hosts: localhost, connection: local` — same invocation
convention as the rest of this section.

---

## PB-281 — Uruinimgina git-history recovery & retry

Added 2026-08-01, after a real incident: `git filter-repo --strip-blobs-
bigger-than 90M` was run against a local clone to fix a GitHub GH001
oversized-file push rejection, which rewrote every local commit hash and
diverged the local clone from `origin` ("fetch first" on push, "refusing to
merge unrelated histories" on pull). See `playbook_281_...yml`'s own header
for the full story, including the two real traps hit along the way: two
directories sharing the basename `bahyway_v4` (one the shared `bahyway/
EnkiDB` clone, one the Architect's separate personal multi-machine
"DailyWorks" repo) being mistaken for each other, and a local, untracked
config file (`uruinimgina.toml`) that a plain re-clone would silently drop.

| PB | File | What it does |
|---|---|---|
| 281 | `playbook_281_uruinimgina_git_recovery_and_retry.yml` | Three modes via `-e git_action=<mode>`: `diagnose` (default, read-only — ahead/behind vs. `origin` per local branch, flags branches with BOTH nonzero as a possible rewritten-history signature; scans `git_branch`'s history for any blob over `blob_limit_mb` without touching it), `reclone` (backs up `repo_dir` to a timestamped, never-auto-deleted sibling directory, fresh-clones the same `origin` URL, restores any file matching `preserve_globs` — default `uruinimgina.toml` — from the backup into the new clone; requires `-e confirm_reclone=true`), `retry_pulse` (builds and runs `uruinimgina-cli` against `uruinimgina_toml`; since Ansible gives the process no TTY, any HALT auto-Aborts per the CLI's own documented EOF behavior — this mode is for a pulse expected to go cleanly, not for babysitting a halt loop). **Never force-pushes, anywhere, ever** — recovery is always "discard the corrupted local clone and re-clone from origin," which is lossless specifically because a rejected push never actually reached GitHub. |

Also `hosts: localhost, connection: local`.

---

## The Production gate — `target_env`, retrofitted onto PB-259 and PB-280

Added 2026-08-01: `playbooks/tasks/require_gilgamesh_for_production.yml`, a
shared, reusable task file (`import_tasks`) that any playbook affecting real
infrastructure now runs first. Rule: Development/Test/Acceptance run
freely (any stakeholder passport, or none); Production requires a verified
`privilege_level=7` passport — informally "Gilgamesh Passport" (minted via
the Gilgamesh Master Key ceremony) vs. "Sargon Passport" (privilege 1-6,
Sargon Passport Manager) — the same two labels DubSar IDE's own login
dropdown already uses (`godot/dubsar-theater/scripts/enki_engines.gd:61`).
Both decode to the same `SargonPassport` Rust struct; only the label and
`privilege_level` differ. No new crypto was written — the gate shells out
to the already-real `bin/kupru-vault-cli` (`cargo build --release -p
kupru-vault-cli` once first), which wraps `kupru-vault::open_vault_and_
authenticate`, the same function Shakkanakku's own `vault_check_enabled`
corpus gate (`runner.rs`) already uses — this just gives that same check to
playbooks run directly via `ansible-playbook`, which `runner.rs`'s gate
never covers (it only runs inside Shakkanakku's own corpus loop).

Fails closed on every path: an unset or unrecognized `target_env` stops
the play before anything real happens; a Production target without
`KUPRU_VAULT_PASSPHRASE` exported, without the vault CLI built, or without
a privilege_level=7 passport in the vault is refused as `BlockedByAuthority`.

**Wired into:**
- `playbook_259_full_7types_enkidb_bootstrap.yml` — a prepended play runs
  the gate before any of its six imported playbooks. `-e target_env=...`
  is now required on every invocation.
- `playbook_280_git_repo_lifecycle.yml` — only `git_action=push` is
  gated (the one mode that reaches shared infrastructure); `clone`/
  `sync`/`restore` stay ungated (local-only or read-only).

**Extended 2026-08-01 to cover the rest of the currently-active infra
corpus** (the Architect's explicit "do ALL open issues" instruction):

- `playbook_263_deploy_shakkanakku_governor.yml` — the standard
  `import_tasks` gate added (installs the governor system-wide).
- `playbook_265_shakkanakku_type1_infra_cqrs_nodes.yml` — **not** the
  standard gate; this playbook already had its own, more sophisticated
  environment-aware vault mechanism (`cqrs_environment: production/dev/
  test/acc`, real per-environment VM names/IPs, its own `vault_check_
  enabled`/`vault_min_privilege` — "Safety mechanism 3", pre-existing).
  Adapting that existing mechanism, not duplicating a second one:
  `cqrs_environment=production` now *always* forces the vault check on
  at `privilege_level=7`, non-bypassable regardless of what `-e
  vault_check_enabled`/`vault_min_privilege` are passed; `dev`/`test`/
  `acc` are untouched, still opt-in, still off by default.

**Deliberately still not gated, on reconsideration, not by oversight:**
- `playbook_268_bahyway_host_privilege_groups.yml` — creates 5 fixed OS
  groups, idempotent, no coherent "which environment" concept applies
  (a Linux group isn't Dev/Test/Acceptance/Production-scoped).
- `playbook_278_uruinimgina_fedora_w44_setup.yml` — only *renders config
  and builds a CLI*; the actual risky action (running `uruinimgina-cli`
  to mint/push) is already a deliberate, attended, un-automatable step
  by design (see that playbook's own header) — gating the setup step
  itself would be gating the wrong point in the flow.
- Everything else in the ~90-playbook historical/reconciled corpus
  (PB-90 through PB-118 and similar) — dev-history, not part of the
  live bring-up path this gate protects.

---

## The 3-tab web dashboard — Resource Check / Shakkanakku / Uruinimgina

Added 2026-08-01, resolving the tension between "fully automated OTAP
bring-up" and "Sargon/Gilgamesh/DubSar IDE are desktop GUI apps that
can't usefully be launched unattended": rather than forcing those GUI
tools into headless automation, `shakkanakku-web` (already real — TLS,
Sargon-Passport auth, previously a single-tab corpus-runner dashboard)
now has three tabs behind the same login:

1. **Resource Check** — `crates/shakkanakku/src/resource_check.rs`
   (new), a native Rust equivalent of PB-279's checks (available RAM via
   real `/proc/meminfo` `MemAvailable`, swap %, disk headroom via `df`,
   load/vCPU) — no `ansible-playbook` shell-out from a live web request.
   `GET /api/resource_check` (AnyPassport) runs it fresh and returns the
   result.
2. **Shakkanakku** — the existing corpus runner, unchanged, now just one
   of three tabs.
3. **Uruinimgina** — new: `docpulse::spawn_docpulse` (previously GUI-only
   and CLI-only) is now a third driver of the same engine, via
   `POST /api/docpulse/{start,abort,retry,skip}` and
   `GET /api/docpulse/state` (AnyPassport for the state poll, Architect
   for the mutating actions — same tier split as Tab 2's `/api/run`).

**The gate is real, not cosmetic**: `ServerState::resource_check_permits_
start()` requires the last check to be both green and less than 5
minutes old (`RESOURCE_CHECK_MAX_AGE_SECS`) before `/api/run` (Tab 2) or
`/api/docpulse/start` (Tab 3) will start anything — enforced server-side
on every request, not just reflected in the frontend's disabled tab
buttons (`app.js`'s `gateOpen()`/`renderGate()` are UX only).

Build/run exactly as before: `cargo build --release -p shakkanakku --bin
shakkanakku-web --no-default-features --features web`, then
`shakkanakku-web` from the repo root (or via `playbook_263...yml -e
shakkanakku_features=web`). No new binary, no new port, no new auth
mechanism — the same tool, three tabs instead of one.

---

## First-Time 7-Types EnkiDB Bring-Up — run order + why it isn't one file

Added 2026-07-27, so the Architect never has to open an individual
playbook's header to find this out.

**TL;DR — run exactly one command, for the first time and every time you
need a full rebuild:**

```bash
cd ~/Forge/EnkiDB/playbooks
ansible-playbook playbook_259_full_7types_enkidb_bootstrap.yml -i ../ansible/inventory.ini
```

That single command already IS "configure the 7 Types correctly, then
create and ingest them all in one go" — it runs the six pieces below
itself, in the order below, inside that one invocation. **You do not run
PB-212/221/222/208/216/213 by hand — PB-259 does.** What follows explains
what happens inside it and why those six remain separate *files* on disk
instead of one file with 1000+ lines.

| Step | PB | File | What it establishes |
|---|---|---|---|
| 1 | 212 | `playbook_212_deploy_cqrs_2node_split.yml` | Builds images + creates/runs the EnkiDDB + EnkiMDB Write/Read containers on the real 2-VM split. Infrastructure only — no document data yet. |
| 2 | 221 | `playbook_221_enkidb_core_deploy_and_scale_sweep.yml` | Deploys EnkiDB (core)'s own Write/Read containers, then runs a **synthetic** benchmark seed (`SEED:<n>`, 1M-1B rows by default) to prove HeptaScript query performance at scale — see the corrected note just below. Independent of step 1 — own containers, own ports. |
| 3 | 222 | `playbook_222_enkisdb_odb_qdb_dw_deploy.yml` | Deploys EnkiSDB/EnkiODB/EnkiQDB/EnkiDW's six containers, then FLUSH/sync/verify against whatever's already in each type's `landing/` zone — legitimately **zero rows** unless something was manually dropped there first. |
| 4 | 208 | `playbook_208_full_corpus_ingestion_runbook.yml` | Ingests the real repo corpus into EnkiDDB (must run AFTER step 1 — this is what gives EnkiDDB actual document content), then flushes/syncs/verifies itself. |
| 5 | 216 | `playbook_216_populate_enkimdb_catalog.yml` | Scans crates + playbooks into EnkiMDB (must run AFTER step 1, same reasoning as step 4 — this is what gives EnkiMDB actual content), then flushes/syncs/verifies itself. |
| 6 | 213 | `playbook_213_cross_host_flush_sync_verify.yml` | One final combined FLUSH + sync + QUERY/SEARCH pass for EnkiDDB **and** EnkiMDB only — confirms both, in one call, now that steps 4/5 gave them real data. Does not touch EnkiDB core or the BeeMDM-chain types. |

**Correction (2026-07-27) — EnkiDB does not ingest data at creation time,
and this table's step 2 originally implied it did.** Only two of the 7
types have an automated path that scans real, already-committed content
and pushes it in: **EnkiDDB** (step 4, real documents) and **EnkiMDB**
(step 5, real crates/playbooks). The other five — EnkiDB (core, Golden
Store) and the whole BeeMDM chain (EnkiSDB → EnkiODB/EnkiQDB →
EnkiDW) — receive real Golden Particles *only* through `bee-watchdog`
polling a landing directory (ZIP drop → Musarû scan → station chain →
B11 score routing → `ParticleState::Golden` → `PermanentStore`; see
`docs/components/BEEMDM_ETL_PIPELINE.md`). No playbook in this repo
automates that whole chain into EnkiDB. Step 2's `SEED:<n>` is a
synthetic scale-proof tool sent straight to the write server's TCP
port — useful for proving query performance, not a substitute for real
ingestion, and not something a first-time bring-up should treat as
"EnkiDB now has real data." Pass `-e '{"enkidb_seed_sizes": []}'` to
PB-259 to deploy EnkiDB core's containers without injecting any
synthetic rows. Step 3 legitimately does the same "deploy only, zero
content" thing for the four BeeMDM-chain types, by design — that is not
a gap PB-259 (or PB-222) is meant to close.

**Why this is six files, not one:**

1. **Deploy and data-ingest are not the same kind of operation, and don't
   share a re-run rule.** Step 1 (deploy) changes *infrastructure*
   (images, containers) — safe and cheap to re-run any time code
   changes; nothing about it walks the document corpus. Steps 4/5
   (ingest) walk the *entire real repo* and mint brand-new KAKI-sealed
   entities every time they run — re-running PB-208 against an
   already-ingested Write Node does not update existing documents in
   place, it **duplicates** them (same for PB-216 — its own manual
   documents "no deduplication across re-scans" as a deliberate,
   explicit-action design, not an oversight). Folding ingest into the
   same file as deploy would mean either re-ingesting (and duplicating)
   on every trivial redeploy, or building "did the corpus actually
   change" detection logic that these six small, single-purpose files
   don't need individually.
2. **Each piece has its own standalone reuse case the Architect (or
   automation) actually needs.** "Just redeploy the Read container after
   a code fix" is step 1 alone. "Just re-ingest new docs added under
   `docs/`" is step 4 alone. "Just re-verify EnkiDDB+EnkiMDB are in
   sync" is step 6 alone. Merging them away would remove those smaller,
   faster, more targeted operations — PB-259 doesn't replace any of
   them, it *sequences* them correctly for the one specific case of
   "build everything from nothing."
3. **Steps 2/3 (PB-221/222) are already as consolidated as they can
   honestly be** — each is one file doing its own type's deploy + seed +
   sync + verify, because no other legitimate reuse case for splitting
   EnkiDB core's or EnkiSDB/ODB/QDB/DW's pieces apart has been found.
   They're the existing proof that "one file per real unit of work" is
   already the target shape — EnkiDDB/EnkiMDB's four pieces (212/208/216/213)
   stay separate specifically because they're each reused independently
   elsewhere, unlike 221/222's internals.

**On "why does it destroy/recreate/re-ingest/flush the same database
multiple times" — the honest breakdown of what's actually destructive and
what isn't:**

- **Container recreate ≠ data destroy.** The named Podman volume holding
  the real entity/EAV data (`enkiddb-write-data`, etc.) is a separate
  mount from the container process; stopping/removing/recreating the
  container never touches it. **Fixed 2026-07-27 (PB-212.4):** PB-212 no
  longer recreates unconditionally on every run either — it now compares
  the running container's image ID to the freshly-built one and only
  recreates on a real difference, so an unchanged redeploy is a true
  no-op. (PB-221/222 still use the older "skip if it already exists"
  pattern and have their own, opposite, currently-unfixed issue — see
  their own KNOWN ISSUE header notes and `docs/PLAYBOOK_EXECUTION_TRIAGE.md`.)
- **FLUSH is not "erase," it's "persist."** It moves in-memory writes to
  disk — the opposite of destructive. The one real risk is *ordering*:
  flushing/syncing before anything has been (re-)ingested legitimately
  reports 0 entities and (correctly) refuses to sync that emptiness over
  real existing data. That was PB-215's actual bug, fixed 2026-07-27 by
  reordering ingest before sync — see that file's own PB-215.1 note.
- **Nothing here re-ingests already-ingested data "again" as part of a
  normal PB-259 run.** Step 4/5 run exactly once per PB-259 invocation.
  Re-running PB-259 itself a second time WOULD re-ingest (and, per point
  1 above, duplicate) — that's expected: PB-259 is the *first-time full
  build* tool, not a routine day-to-day command. Day 2 onward, reach for
  the individual step you actually need (e.g. PB-208 alone after adding
  new docs) instead of re-running the whole chain.

**What comes after PB-259 — nothing is required; here's what's optional
and what's an open gap, so the answer isn't left implicit:**

- **Nothing else is mandatory.** PB-259 is the terminal step for
  everything that's automatable today. There is no PB-260-or-similar
  that must follow it.
- **Optional, if you want the CEO report:** run
  `playbook_IsimudEngine.yml` (see below) — it's a separate,
  bigger tool that also runs the other ~64 tracked playbooks, so only
  reach for it when you actually want that full-ecosystem report, not as
  a routine follow-up to PB-259.
- **Optional, day-2 onward (not right after PB-259):** if new documents
  land under `docs/`, re-run PB-208 alone (not the whole chain) to
  ingest just those. Same for PB-216 alone after new crates/playbooks.
- **Open gap, not a "run PB-X" answer:** if you want EnkiDB core or the
  BeeMDM-chain types (EnkiSDB/EnkiODB/EnkiQDB/EnkiDW) to hold real
  Golden Particles instead of nothing (or, for EnkiDB core, synthetic
  benchmark rows), there is currently no playbook for it — `bee-watchdog`
  itself is never deployed as a live daemon by any playbook in this repo
  (confirmed by grep: no Containerfile, no `hosts:`-targeted deploy task
  exists for it), so even manually dropping a ZIP into a `landing/`
  volume today has nothing running to pick it up and process it. Real
  content for those 5 types is a real, currently-unbuilt piece of work,
  not something PB-259 or any existing playbook was meant to close.

**CEO / audit-trail reporting:** `docs/PLAYBOOK_EXECUTION_TRIAGE.md` is
the manifest `playbook_IsimudEngine.yml` parses to decide what to
run; PB-259 is already a row in it (added 2026-07-27). Running
`ansible-playbook playbook_IsimudEngine.yml -i "localhost," -c local -v`
(see `docs/ISIMUD_ENGINE_MANUAL.md`) will already include PB-259 in
its output, and now records, for every run: **Operator** (the real OS
user who ran it, `ansible_env.USER`), DateTime, Host, Crate Version, and
Release (`git describe`) — in the technical `ISIMUD_ENGINE_REPORT_
<ts>.json/.txt`, the CEO-facing `EXECUTIVE_REPORT_<ts>.md/.html`, and a
one-line ledger entry per run appended to `docs/SHEDU/NARU_AUDIT_JOURNAL.md`
(fixed 2026-07-27 — Operator was missing from all of these before).
PB-259 itself does not yet write its own separate input/output log; the
IsimudEngine's existing reporting is what covers it today.

---

## Playbooks that require `-i ansible/inventory.ini` (real 2-VM CQRS split)

Added 2026-07-27 after the same mistake was made twice: running one of
these without `-i ansible/inventory.ini` doesn't fail loudly. Ansible
silently falls back to running every `delegate_to:` step as if it were
local, so a FLUSH/health-check/deploy step that should hit the real
write or read VM instead tries `127.0.0.1` on eriduous-vdi itself —
`ConnectionRefusedError` or "container not running," neither of which
looks like a missing flag. Every playbook below either sets
`delegate_to:` against `enkidb-node-write`/`enkidb-node-read` (or a
`{{ enkiddb_write_host }}`/`{{ enkiddb_read_host }}` var resolving to
them), or targets the `enkidb_write`/`enkidb_read` inventory groups
directly via `hosts:` — confirmed by grepping the actual task files,
not by memory.

Run every one of these **from `playbooks/`**, exactly like this
(substitute the real filename):

```bash
cd ~/Forge/EnkiDB/playbooks
ansible-playbook <playbook_file>.yml -i ../ansible/inventory.ini
```

| PB | File | Why it needs it |
|---|---|---|
| 203 | `playbook_203_enkiddb_enkimdb_health_check_and_backup.yml` | `delegate_to` per container/volume, split across both real hosts |
| 208 | `playbook_208_full_corpus_ingestion_runbook.yml` | imports `tasks/enkiddb_cross_host_sync.yml` (FLUSH/sync/verify `delegate_to` the write host) |
| 213 | `playbook_213_cross_host_flush_sync_verify.yml` | imports the same cross-host sync task file as PB-208 |
| 214 | `playbook_214_diagnose_read_node_not_ready.yml` | `delegate_to: enkidb-node-read` |
| 215 | `playbook_215_full_environment_bootstrap.yml` | `import_playbook`s PB-212 + PB-208 + PB-213 in sequence (fixed 2026-07-27 — was PB-212+213+208, sync-before-ingest, see PB-259's header) — inherits all three's requirement (see PB-212's own footnote below) |
| 216 | `playbook_216_populate_enkimdb_catalog.yml` | `delegate_to: enkidb-node-write` |
| 221 | `playbook_221_enkidb_core_deploy_and_scale_sweep.yml` | plays target `hosts: enkidb_write` / `hosts: enkidb_read` directly, plus `delegate_to` in the sweep task |
| 222 | `playbook_222_enkisdb_odb_qdb_dw_deploy.yml` | same pattern as PB-221 |
| 223 | `playbook_223_cqrs_node_disk_diagnostics.yml` | `hosts: enkidb_write, enkidb_read` |
| 224 | `playbook_224_cqrs_node_reconfigure.yml` | `hosts: enkidb_write, enkidb_read` |
| 258 | `playbook_258_enkiddb_topic_graph_report.yml` | `delegate_to: "{{ enkiddb_read_host }}"` |
| 259 | `playbook_259_full_7types_enkidb_bootstrap.yml` | `import_playbook`s PB-212 + PB-221 + PB-222 + PB-208 + PB-216 + PB-213 in sequence — inherits all six's requirement. **Run PB-259 alone; it runs all six itself, in order, inside this one invocation — do not run any of them by hand first.** |

**Not on this list on purpose:** PB-211
(`playbook_211_flush_sync_and_verify_enkiddb.yml`) imports the
same-host `tasks/enkiddb_flush_sync_verify.yml` — genuinely no
`delegate_to` in it, because it predates the 2-VM split and was written
for Write+Read as two containers on one host. It doesn't need the
inventory flag, but running it against the real split also won't do
what you want; PB-213 is its real cross-host successor. Confirmed by
grepping the task file directly, not assumed from the pattern.

**Also not in the main table, for a different reason:** PB-212
(`playbook_212_deploy_cqrs_2node_split.yml`) genuinely does target
`hosts: enkidb_write` / `hosts: enkidb_read` directly and does need
`-i ansible/inventory.ini` — that part of the original entry was
correct. It's out of the routine table because it's a **one-time
deployment step**, not something run repeatedly like PB-208/213/258:
it stands up the 2-VM CQRS split itself (containers, volumes,
`podman-restart.service`), and that split is already live on real
infra as of 2026-07-26 (confirmed `[x]` in the triage doc). Re-running
it isn't part of normal day-to-day workflow — but it's still the
correct playbook to reach for if the split ever needs to be rebuilt
from scratch (e.g. after a host reboot where containers don't come
back on their own, which has happened for real on this fleet before).
If you do run it again, it still needs the same flag:

```bash
ansible-playbook playbook_212_deploy_cqrs_2node_split.yml -i ../ansible/inventory.ini
```

If a playbook you're about to run isn't in this table, it either runs
entirely on `localhost` (most playbooks in this repo — no flag needed)
or is one this list hasn't caught up to yet; grep the file itself for
`delegate_to:` or a `hosts:` line naming `enkidb_write`/`enkidb_read`
before assuming.

---

## Infrastructure Overview

| Node | IP | Role | Run playbooks FROM |
|------|----|------|--------------------|
| Fedora laptop (Host) | — | GitHub remote, download point | — |
| eriduous-vdi | 192.168.122.214 | Dev control plane, ansible runner | FROM eriduous-vdi (scp'd from laptop) |
| enkidb-node-write | 192.168.122.101 | CQRS write path | FROM eriduous-vdi |
| enkidb-node-read  | 192.168.122.107 | CQRS read path  | FROM eriduous-vdi |
| dubsar-workstation | 192.168.122.121 | Grafana+Prometheus ONLY | FROM eriduous-vdi |

## Sync convention

eriduous-vdi has no GitHub access. Workflow is always:
  1. On the laptop: git pull origin claude/iphone-playbooks-crosstribe-eval-nbc5sw
  2. scp -r workspace/bahyway_v4 bahaa@eriduos-vdi:~/Forge/EnkiDB/workspace/bahyway_v4
  3. scp playbooks/*_reconciled.yml bahaa@eriduos-vdi:~/playbooks/
  4. ssh bahaa@eriduos-vdi, then: ansible-playbook <name>_reconciled.yml -i "localhost," -c local -v

## Playbooks confirmed to exist — PB-111 to PB-145

111, 112, 113, 114, 115, 116, 117, 118 (blocked), 137, 138, 139, 140,
141, 142, 143, 144 (configs ready, deploy blocked), 145.

## Special Notes

**PB-98 MUST run before PB-111** — the KISPU fix is the prerequisite
for the performance gate.

**PB-144** generates real Grafana/Prometheus configs locally
regardless of hardware; remote deploy only runs if dubsar-workstation
is reachable on SSH + the target ports.

**PB-140** requires both EnkiDB nodes running and the ASHNAN test
corpus present.

---

## `launchers/` — run one specific tool from Shala2, not the whole corpus

Added 2026-08-02, after the Architect asked how to launch DubSar Theater /
Sargon Passport Manager / Gilgamesh Master Key from Shala instead of typing
`ansible-playbook` by hand each time. Shala2 (Shakkanakku's own web tab)
already runs any config you point it at — it doesn't have to be the big,
whole-corpus `shakkanakku.toml`. A launcher config is just a minimal
`shakkanakku.toml`-shaped file whose `playbooks = [...]` list has exactly
one entry, so Load + Run fires only that one playbook, with whatever
`[[parameters]]` it needs pre-filled.

Requires PB-284's own `chdir` fix (2026-08-02) to be live first — before
that fix, `shakkanakku-web`'s working directory depended on wherever
`ansible-playbook` was invoked from, so a relative config path like
`launchers/launch_dubsar_theater.toml` could resolve to the wrong place.

| File | Runs | Notes |
|---|---|---|
| `launchers/launch_dubsar_theater.toml` | PB-226 | Login screen directly (run mode), no editor chrome. |
| `launchers/launch_sargon_passport_manager.toml` | PB-285 `-e tool=sargon` | Gardener-tier passport creation/management. |
| `launchers/launch_gilgamesh_master_key.toml` | PB-285 `-e tool=gilgamesh` | Architect Key ceremony. |

**Usage from Shala:** sign in with an Architect passport (needed for
Shala2's Run button regardless), go to Shala2, type the launcher's path
(e.g. `launchers/launch_dubsar_theater.toml`) into the **config** field,
click **Load**, then **Run**. No terminal needed.

Add more of these the same way for any single playbook worth a one-click
launch (e.g. a future EnkiDB-monitoring launcher) — just don't set
`triage_doc`, or `Config::load` will overwrite the one-entry `playbooks`
list with the entire triage doc's corpus.

---
*BahyWay.Ecosystem v4.0 — Sovereign, Pure Rust, Offline*
