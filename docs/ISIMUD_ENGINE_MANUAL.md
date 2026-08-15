# BahyWay.Ecosystem IsimudEngine — Operations Manual

`playbooks/playbook_IsimudEngine.yml` is the Central PB: it runs every
playbook tracked in `docs/PLAYBOOK_EXECUTION_TRIAGE.md`, in the order the
triage doc itself mandates, and produces four reports — two for engineers,
two for stakeholders. It is not itself a numbered PB (it would try to invoke
itself if it were) and has no row of its own in the triage doc's tables.

**Engine name: IsimudEngine.** The Architect named this engine after Isimud,
Enki's two-faced messenger/vizier in the source mythology this ecosystem's
naming already draws from — the one entity here meant to be *called by*
something else, not run by hand like a numbered PB. It's the stable,
callable identity any external Dashboard should reach for when it wants to
stand up a full Test Environment containing all of BahyWay.Ecosystem v4.0
(or a later version). Filename **renamed 2026-07-27** to
`playbook_IsimudEngine.yml` (all internal/CI references updated in the
same pass — see §2 for the current invocation) so the file on disk
matches the identity every report it produces already carries:
`IsimudEngine` (`orchestrator_name` internally) so a Dashboard — or a
human reading the report — can see which engine actually
ran, not just which ecosystem version.

## 1. What it actually does

1. Reads `docs/PLAYBOOK_EXECUTION_TRIAGE.md` directly — parses its own
   markdown tables (number, filename, status, and the `## Phase N` heading
   each row sits under) as the run manifest. There is no second,
   separately-maintained list of "which playbooks exist" to drift out of
   sync with the doc the way `EnkiDDB_MANUAL.md`'s guidance once drifted
   behind `playbook_208` — the triage doc is the single source of truth for
   both a human reading it and this orchestrator running it.
2. Also parses each phase heading's own one-line `**Declaration:**` (added
   to the triage doc 2026-07-27, one per `## Phase N` heading) into a
   phase→declaration map, and surfaces it in the Executive Report's
   Phase-by-Phase table (§5) — so that report explains WHAT each phase is
   and WHY it's grouped that way, not just a pass/fail count, without
   anyone needing to open the triage doc itself.
3. Skips every row marked `SKIP` (superseded files, gap-diagnostics,
   Architect-dismissed playbooks like PB-144's two Grafana files).
4. Runs every remaining playbook, in doc order, via `ansible-playbook
   <file>.yml -i "localhost," -c local -e forge_root=<same forge_root>`.
   **Continues past individual failures** — one orchestrator run gives the
   full ecosystem picture, not just the first blocker.
5. Times, classifies, and scores every attempt (§4), then writes four files
   to `docs/SHEDU/` (§5) and a line to the NĀRU journal — each report now
   also carries the real Operator (the OS user who ran it) alongside
   version/release/host/datetime, and the IsimudEngine identity itself.

## 2. Usage

```bash
cd ~/Forge/EnkiDB/playbooks
ansible-playbook playbook_IsimudEngine.yml -i "localhost," -c local -v
```

All variables are optional (`-e NAME=VALUE`):

| Variable | Default | Effect |
|---|---|---|
| `run_profile` | `full` | `full` runs everything non-SKIP. `ci_fast` additionally excludes anything matching `heavy_pattern` (currently `full_corpus_ingestion\|scale_sweep` — real multi-hour playbooks like PB-208 and PB-221). Use `ci_fast` for frequent runs, `full` on a schedule or before a release. |
| `pb_filter_numbers` | *(none)* | Comma-separated PB numbers, e.g. `"249,250,251"`. Restricts the run to just these (still in doc order, SKIP rows among them still skipped). Use this to re-run one phase after fixing a failure, or to smoke-test the orchestrator itself without a multi-hour full run. Ignored if `pb_sequence` is set. |
| `pb_sequence` | *(none)* | The real "run these specific PBs on an existing environment" tool — see §10 below. Composite-aware: recognizes when a listed PB (like PB-259) already covers others in the list and skips the duplicates automatically. Takes over PLAN entirely when set (`run_profile`'s `ci_fast` exclusion and `pb_filter_numbers` are both ignored). |
| `heal_points_threshold` | `0` | The final `GATE` task fails the whole run (non-zero exit) if Heal_points ends up below this. Set it in CI to turn "ecosystem degraded" into a red build rather than a report nobody reads. |
| `stop_on_error` | `false` | `false` (default) is the original "continue past failures" behavior — one run gives the full ecosystem picture, unattended. `true` halts the RUN loop at the first failing PB instead of continuing: prints a `STOPPED at PB-<N> (<file>)` banner naming the exact error, the matched `known_errors` entry's cause+fix if one exists (see §11), and how to resume — then still writes all four reports, covering only what actually ran. Use this for the Architect's own actual workflow (fixing one real failure at a time), `false` for an unattended health sweep. |
| `forge_root` | `$HOME/Forge/EnkiDB` | Repo root. Forwarded explicitly to every sub-playbook invocation — see §7 for why that forwarding exists. |
| `secrets_vars_file` | *(none)* | Path (absolute, or relative to `forge_root`) to a YAML vars file forwarded as `-e @<file>` to every sub-playbook. Needed for unattended runs of secret-consuming playbooks (e.g. PB-153's `fable_api_key`). See §9. |
| `disk_free_warn_gb` | `15` | Free-space threshold (GB) on `forge_root`'s filesystem. Below this, the preflight check (§3) warns before the run continues. |
| `auto_disk_cleanup` | `false` | If `true` **and** free space is below `disk_free_warn_gb`, the preflight check runs `cargo clean` + `podman system prune -f` itself before continuing. Left off by default — see §3 for why this isn't automatic. |
| `launch_guis` | `false` | Four Phase 6 playbooks (PB-226, PB-227, PB-230, PB-234) background a real, persistent Godot GUI window (`nohup godot ... &`) rather than exit clean like every other playbook. With the default `false`, IsimudEngine excludes all four from the run — a full/ci_fast sweep now always ends at the report, never with an orphaned DubSar window on whatever host ran it. The report's "How to start the DubSar IDEs" section gives the exact commands to launch them afterward by hand. Set `-e launch_guis=true` to have the sweep launch them for real instead (not recommended for unattended/CI runs). Like `heavy_pattern`, this exclusion never applies when `pb_sequence` explicitly names one of the four by number. |

## 3. Preflight: disk space

Added 2026-07-27 after a real failure: a `podman build` inside PB-205
(the asset-server image) ran 28 real minutes on eriduous-vdi, then
failed with `no space left on device` writing to `/var/tmp` during its
`COPY . .` step. Root cause — no `.containerignore` existed for
`workspace/bahyway_v4` (the build context every Containerfile in this
repo uses), so that `COPY . .` transferred the entire directory
including `target/`, which was **21G** in that real, cargo-built
checkout. Fixed going forward
(`workspace/bahyway_v4/.containerignore` now excludes `target/`,
`.git/`, `.godot/`), but that fix doesn't reclaim space *already*
consumed by pre-existing `target/` artifacts or dangling Podman
image/build-cache layers left over from before it landed — and several
playbooks in this chain (PB-205, PB-208's corpus ingestion, PB-221's
scale sweep, every write/read-server image build) are exactly the kind
of build-heavy step that can transiently need tens of GB.

So the orchestrator now checks free space on `forge_root`'s filesystem
**before** touching the triage doc or running anything, via `df
--output=avail` (not `ansible_mounts` — simpler, and doesn't depend on
which mount entry Ansible's fact-gathering happens to consider the
"real" one for a bind-mounted or overlay path). Below `disk_free_warn_gb`
(default 15G), it prints a warning naming the exact safe cleanup
commands rather than guessing at the free space it needs and blocking
partway through a multi-hour run:

```bash
cd ~/Forge/EnkiDB/workspace/bahyway_v4 && cargo clean
podman system prune -f
```

Both are safe and reversible — `cargo build` regenerates `target/` on
its next invocation (slower once, not lost), and `podman` rebuilds any
pruned image/cache layer the next time something needs it. Neither is
run automatically by default; pass `-e auto_disk_cleanup=true` to have
the orchestrator run them itself when the check trips, or run them by
hand and re-invoke. `podman system prune -f`'s own `failed_when: false`
is deliberate too — a dangling image can be held by a real, previously-
interrupted "external" build container that `podman ps -a` doesn't even
list (reproduced live the same day), and that edge case shouldn't block
every other playbook behind one prune failure.

## 4. Heal_points and Trust State

**Heal_points is a weighted pass-rate, not a flat percentage.** Every
attempted (non-SKIPPED, non-profile-excluded) playbook gets a criticality
weight, classified automatically by filename pattern:

| Weight | Category | Pattern |
|---|---|---|
| 3 | Security / data-integrity | HeptaSec, security singles, `urnammu-attestationd`, `kittu-engine`, `musaru-security`, authorship gates, CQRS deploys, sovereign-db naming |
| 2 | Everything else | *(default)* |
| 1 | Docs / scaffolds / diagnostics | gap-diagnostics, roadmap/glossary/report docs, domain scaffolds, concept seals, law-amendment notes |

```
Heal_points = (Σ weight of PASSED) / (Σ weight of ATTEMPTED) × 100
```

This is a heuristic, not a hand-curated per-playbook table — deliberately,
for the same reason the manifest itself is parsed rather than
hand-maintained: it shouldn't need updating every time a new playbook
lands. If something is misclassified, adjust `critical_pattern` /
`low_pattern` at the top of the playbook.

**Trust State** translates the number into the same four-tier vocabulary
this ecosystem's own security crates already use for KAKI B11 trust scoring
(Golden / Active / Suspicious / Blocked) — not a literal reuse of that 0–255
byte scale, since Heal_points measures the whole ecosystem's health on 0–100
rather than one entity's trust, but the same spirit and the same words:

| Trust State | Heal_points | Meaning |
|---|---|---|
| **GOLDEN** | ≥ 85 | Fully operational. All attempted verifications, including every security/data-integrity playbook, passed. |
| **ACTIVE** | ≥ 60 | Stable, with gaps outside the critical path. Review failures before the next release. |
| **SUSPICIOUS** | ≥ 35 | Degraded — a critical (security/data-integrity) verification failed. Do not treat the build as production-ready. |
| **BLOCKED** | < 35 | Critical. Immediate remediation required. |

## 5. The four reports (all written to `docs/SHEDU/`)

| File | Audience | Contains |
|---|---|---|
| `ISIMUD_ENGINE_REPORT_<ts>.txt` | Engineers | Every playbook, raw filenames, full `stderr` tail on failure, exact durations, **plus** an "Errors captured during execution" section (below). |
| `ISIMUD_ENGINE_REPORT_<ts>.json` (+ `..._LATEST.json`) | CI / tooling | Same data, structured — `heal_points`, `results[]` with per-playbook `passed`/`rc`/`start`/`end`/`duration`/`error`/`error_lines`. Used by the GitHub Actions job summary. |
| `EXECUTIVE_REPORT_<ts>.md` | Stakeholders (git-trackable) | Same results, translated: phase names instead of filenames, the Trust State label instead of a raw score, findings without stack traces. Renders cleanly on GitHub. |
| `EXECUTIVE_REPORT_<ts>.html` (+ `..._LATEST.html`) | **CEO / external handoff** | Self-contained, styled, single file — no external assets, safe to email or open from disk. Print-to-PDF friendly (`@media print` rules included). This is the one to hand off. |

The executive HTML is deliberately not the same document as the technical
`.txt` — it never shows a raw filename, a stack trace, or the word
"ansible." It shows: prepared date, release, a Trust State verdict with a
one-line plain-English explanation, a phase-by-phase pass/fail table, a
findings list naming only the PB number and phase (full diagnostic detail
lives in the technical report, one click away for whoever needs it), and a
closing attestation sealed with the git commit hash — echoing this
ecosystem's own KAKI/checksum vocabulary for identity and integrity.

Both `_LATEST.json` and `_LATEST.html` are stable filenames the orchestrator
overwrites every run — bookmark or link `EXECUTIVE_REPORT_LATEST.html`
directly rather than a timestamped one if you want a single URL that's
always current.

### 5.1 "Errors captured during execution" — errors as they happened, not just the final rc

(2026-07-27, the Architect's own request.) The technical `.txt`/`.json`
reports already had a **final** verdict per PB — `rc`/`passed`/an 800-char
`stderr` tail, all reflecting only how the sub-playbook ended. That misses
anything that happened mid-run and then got scrolled past by later,
successful-looking output, or a task that failed on one host/step but the
overall play still exited 0. Every PB record now also carries `error_lines`
— every line in that PB's own **full** stdout+stderr (not a tail) matching
a real Ansible failure/error signature (`fatal: [...]`, `FAILED!`,
`ERROR!`, `unreachable=`/`failed=` with a nonzero count, or a
`"failed": true` JSON result), in the order they actually occurred,
regardless of whether the PB passed or failed overall. The `.txt` report
prints this as its own section, right before the final rc-based "Failures"
section — errors *before* the final verdict, then the final verdict itself,
so you never have to guess whether something failed silently along the way.
On a clean run this section just says `(none)`.

### 5.2 "How to start the DubSar IDEs" — the sweep ends at the report, on purpose

(2026-07-27, the Architect's own request.) The run itself never opens a
DubSar IDE window — see `launch_guis` in §2's variable table. Both the
`.txt` and `.md`/`.html` reports close with a short reference section
instead, naming the two things a stakeholder actually does next once the
report shows a healthy environment:

- **EnkiDB 7-Types Connector Wizard** (to work with the Databases) — launch
  DubSar Theater (PB-234), then press **Ctrl+W** inside it.
- **DubSar PDM IDE** (to work with Client documents and the SLA Layer
  configuration) — launch PB-230 directly; it's a separate app, not a tab
  inside DubSar Theater.

The `.txt` report also lists PB-227 (Sargon Passport Manager / Gilgamesh
Master Key Manager) for minting or inspecting a Passport first, and the
exact `ansible-playbook` command for all three. The JSON report carries the
same information machine-readably as `launch_guis`, `gui_excluded_count`,
and `gui_excluded` (the withheld PB numbers).

## 6. GitHub Actions / CI

Wired at `.github/workflows/isimud-engine.yml`, targeting a
**self-hosted runner registered from eriduous-vdi** — GitHub's own hosted
runners cannot reach the private VM network (`192.168.122.x`), TPM
hardware, or ZFS filesystems most of these playbooks check for real; a
hosted runner would just report BLOCKED across most phases, the way
PB-144 did before it was dismissed. See the workflow file's own header
comment for the one-time runner registration steps.

Triggers: `ci_fast` on every push/PR, `full` nightly at 03:00 UTC, and
on-demand via `workflow_dispatch` with a selectable profile and optional
`pb_filter_numbers`. The job publishes Heal_points to the GitHub Actions
step summary (via `jq` against `ISIMUD_ENGINE_LATEST.json`) and
uploads both technical and executive reports as build artifacts.

## 7. Two real bugs, fixed, worth knowing about

Both reproduced live while building and testing this orchestrator, not
theoretical:

- **Self-referencing `vars:`.** `run_profile: "{{ run_profile | default('full') }}"`
  looks like a normal "give me a default" pattern but is not — Ansible's
  `vars:` block resolves this by looking up `run_profile`, which is the
  variable currently being defined, causing infinite Jinja recursion the
  moment no `-e run_profile=...` override is passed. Extra-vars already
  take precedence over `vars:` automatically, so the self-reference was
  both wrong and unnecessary. Fixed by using plain defaults.
- **`forge_root` not forwarded to sub-playbooks.** Each sub-playbook has its
  own default `forge_root: "{{ ansible_env.HOME }}/Forge/EnkiDB"`. That's
  correct on eriduous-vdi's manual layout, but the orchestrator's own
  `command` invocation of each sub-playbook didn't pass `forge_root`
  through — so every sub-playbook resolved its own default independently,
  which broke the moment the checkout lived somewhere other than
  `$HOME/Forge/EnkiDB` (reproduced in the authoring sandbox; would have
  broken identically under a CI runner, which checks code out into its own
  workspace directory). Fixed by forwarding `forge_root` explicitly on
  every sub-playbook invocation.

## 8. Troubleshooting

- **"Triage doc missing"** — `forge_root` isn't resolving to the actual
  checkout. Pass `-e forge_root=<path>` explicitly.
- **A sub-playbook reports "workspace not synced" despite the orchestrator
  itself finding everything fine** — you're running a version of the
  orchestrator from before the §7 forwarding fix, or a local edit removed
  the `-e forge_root={{ forge_root }}` from the `RUN` task's command line.
- **Heal_points is lower than expected** — check the technical `.txt`
  report's "Failures" section for the real `stderr`; the executive report
  deliberately doesn't include it.
- **A playbook you expected to run didn't** — check whether it's marked
  `SKIP` in the triage doc (intentional), excluded by `ci_fast`'s
  `heavy_pattern` (check `run_profile`), or simply not yet added as a row
  to the triage doc at all (the orchestrator only knows about playbooks the
  doc tracks — see the triage doc's own "How to use this" rule 4).

## 9. Secrets

Some playbooks need a real secret at runtime — PB-153's `fable_api_key`
(an Anthropic API key, so the design-time Fable Impact Gate can call
`claude-fable-5`) is the first one.

**The Architect's real secret store for BahyWay.Ecosystem v4.0 is the
Sargon Passport Manager** (PB-227, `crates/kupru`'s real Argon2id +
Ed25519 + ChaCha20-Poly1305) — **not Ansible Vault.** Any older playbook
comment that says otherwise predates this and is superseded.

Sargon Passport Manager is a Godot GUI with no headless/CI-callable
interface today, so there's no automated bridge from the vault straight
into an orchestrator run. The workflow is: open it by hand
(`ansible-playbook playbook_227_build_and_launch_kupru_tools.yml`), copy
out the secret(s) the run needs, and drop them into a local file under
the gitignored `secrets/` directory:

```yaml
# secrets/orchestrator_secrets.yml — gitignored, never committed
fable_api_key: "sk-ant-..."
```

Then point the orchestrator at it — it forwards the whole file as
`-e @<file>` to every sub-playbook it runs, so any playbook needing a var
defined in there picks it up automatically:

```bash
ansible-playbook playbook_IsimudEngine.yml -i "localhost," -c local -v \
  -e secrets_vars_file=secrets/orchestrator_secrets.yml
```

If `secrets_vars_file` is set but doesn't resolve to a real file, the
orchestrator fails fast with a clear message rather than letting every
secret-consuming sub-playbook fail one at a time with "undefined
variable." If it's left unset (the default), nothing changes — most
playbooks need no secrets at all.

If secret-consuming playbooks keep growing in number, the next real step
is a small `kupru` CLI binary that exports one named secret to stdout, so
Ansible's `lookup('pipe', ...)` could pull straight from the vault
without a hand-copied intermediate file — not built yet, since only one
playbook has needed a secret so far.

## 10. `pb_sequence` — running a specific set of PBs on an existing environment

**The real situation this was built for:** PB-259 already deploys/ingests
the 7 EnkiDB Types in one command, but everything from PB-215 through the
rest of the triage doc still needed running on an environment PB-259 had
already touched — without re-running the parts PB-259 already covers
(212, 221, 222, 208, 216, 213). Doing that by hand means remembering
exactly which numbers those are and subtracting them from a range every
time. `pb_sequence` does that subtraction automatically.

### Syntax

Comma-separated tokens. Each token is either a bare number or a range:

```bash
-e pb_sequence="PB259,PB215..PB253"
```

- `"PB"`/`"pb"` prefix is optional on either side — `259`, `PB259`,
  `pb259` all mean the same thing.
- A range (`A..B`) always expands **ascending**, filtered to numbers that
  actually exist as real rows in `docs/PLAYBOOK_EXECUTION_TRIAGE.md` —
  gaps (e.g. 217-220, which were never used) are silently dropped from
  the range, not treated as errors.
- A bare number that doesn't exist in the triage doc at all (a real
  typo) is reported as an **unrecognized token** in every report, loudly
  — never silently dropped without a trace.

### Composite dedup — the actual point of this feature

For every PB number that appears anywhere in the resolved list, IsimudEngine
reads **that PB's own file** and greps its `import_playbook:` lines —
there is no hand-maintained "PB-259 covers these" map to go stale. If
PB-259 is anywhere in `pb_sequence`, everything it `import_playbook`s
(212, 221, 222, 208, 216, 213) is marked covered and skipped everywhere
else in the same run — regardless of whether it also showed up via a
bare number or inside a range. A composite whose OWN required sub-PBs
are already fully covered by an earlier composite in the list (e.g.
PB-215, which needs 212+208+213 — all three already covered once PB-259
has run) is itself skipped too, as fully redundant.

Given `pb_sequence="PB259,PB215..PB253"` against this repo's real triage
doc (verified live before shipping this feature):

```
Resolved run order (20): PB-259, PB-223, PB-224, PB-225, PB-226, PB-227,
  PB-228, PB-229, PB-230, PB-231, PB-232, PB-233, PB-234, PB-247, PB-248,
  PB-249, PB-250, PB-251, PB-252, PB-253

Skipped (composite dedup, 4): PB-215 [this composite's own required
  sub-playbooks are already covered by an earlier composite in this
  sequence], PB-216 [already covered by an earlier composite playbook
  in this sequence], PB-221 [already covered...], PB-222 [already
  covered...]
```

(PB-231/233 still get individually skipped afterward as usual, by the
existing doc-`SKIP`-status handling — composite dedup and doc-`SKIP`
status are two independent filters, both still apply.)

### What changes vs. the default run

- Execution follows the **resolved order** above, not doc order.
- `run_profile`'s `ci_fast` heavy-playbook exclusion does **not** apply —
  an explicit request by number is never profile-filtered away.
- `pb_filter_numbers` is ignored if `pb_sequence` is also set.
- Every report (`ISIMUD_ENGINE_REPORT_*.txt/.json`,
  `EXECUTIVE_REPORT_*.md/.html`) gains a **Requested Sequence** line
  showing the raw string plus counts, and the technical `.txt`/`.json`
  report additionally prints an **Output highlights** section — the last
  ~1500 characters of real stdout (FLUSH/QUERY/SEARCH counts, real
  numbers, whatever that playbook's own `debug` tasks printed) for every
  PB actually run, pass or fail. This section is deliberately **only**
  shown when `pb_sequence` is set — on a full 65-playbook sweep it would
  make the report unreadable, but for a deliberately small custom
  sequence it's exactly the "useful messages and tokens" the report
  should carry.

## 11. Environment manifest — declared preconditions, checked before the run

(2026-07-28, the Architect's own request.) Three real failures this session
were each individually diagnosed live, mid-run, costing 20–60 minutes each:
`scons` missing, a Vulkan/GPU renderer mismatch that froze the VDI, and a
stale process left running at 215% CPU for hours. Every one of them was
discoverable in seconds by a check run *before* the sweep started. Rather
than rediscovering the next one the same way, `playbooks/
isimud_environment_manifest.yml` declares hardware/software/network
preconditions plus a `known_errors` catalog — signature, real cause, real
fix, for every failure class this session actually hit and actually fixed.
IsimudEngine's PREFLIGHT-ENV section (right after the existing disk-space
check) loads this file and runs every declared check:

- CPU cores and RAM against `hardware.cpu_cores_min` / `ram_gb_min`.
- Every tool in `software.required_tools` present on PATH — a miss prints
  the matching `known_errors` entry's cause and fix when one exists (e.g.
  KE-001 for `scons`), or an honest "no known-error entry yet" when it
  doesn't, rather than inventing one.
- GPU renderer: `vulkaninfo --summary` checked for the manifest's
  `known_software_renderer_signature` (`llvmpipe`) — informational only,
  since PB-226 already forces `gl_compatibility` unconditionally regardless
  of what this reports.
- Podman daemon reachability (`podman info`).
- Every host in `network.expected_hosts` (currently just the core CQRS VM,
  `192.168.122.107`) — one TCP check against its first declared port.

Every one of these **warns, never hard-fails** the whole sweep — same
philosophy as the existing disk-space check: a human reading the report
decides whether to fix it now or accept a degraded run. The only hard
failure is the manifest file itself being missing while
`env_manifest_check=true` (default) — pass `-e env_manifest_check=false`
to skip this section deliberately (e.g. re-running `pb_sequence` on a
machine already confirmed good).

**Growing the catalog:** add a new `known_errors` entry only after a real
run actually hits a new failure class and it's actually fixed — never
speculatively. This is the same discipline `docs/TRANSPARENCY_STANDARD.md`
already enforces everywhere else in this repo: a short, 100%-verified list
beats a long, partly-invented one.

## 12. Sealing a successful run as a KAKI particle in EnkiDDB

(2026-07-28, the Architect's own request: "after successful run consider
it as a Particle, got KAKI v4.0 Identity, and save it in EnkiDDB.") This
needs no new tooling — `bin/enkiddb-cli`'s real, tested `ingest` command
already does exactly this for any directory of `.md` files, using
`enkidb_kaki::KakiMinter` to mint the real 16-byte KAKI key and
`WriteNode::ingest_directory_categorized_checked` to seal it into Tigris
(EnkiDDB). `EXECUTIVE_REPORT_<ts>.md` is already a git-trackable markdown
file written to `docs/SHEDU/` by every run — it only needs to actually be
committed first, since `enkiddb-cli`'s authorship gate
(`check_authorship`) requires a real git-committed author on the team
allowlist (`bahaa.fadam@gmail.com` is seeded in by default).

```bash
# 1. Let IsimudEngine finish and inspect Heal_points / Trust State.
# 2. Commit the run's reports (an ordinary git commit, by design --
#    sealing a report into EnkiDDB is deliberately a separate, explicit
#    step from the automated sweep itself, never a silent side effect of
#    an unattended run).
cd ~/Forge/EnkiDB
git add docs/SHEDU/EXECUTIVE_REPORT_*.md
git commit -m "IsimudEngine run: Heal_points <N>, Trust State <STATE>"

# 3. Seal it -- mints a real KAKI v4.0 key and writes it into EnkiDDB.
cd workspace/bahyway_v4
cargo run --release --bin enkiddb-cli -- ingest ../../docs/SHEDU
```

`--dry-run` reports what would be ingested (SCAN + CATEGORIZE only)
without journaling anything, if you want to confirm the authorship check
passes before actually sealing. Once ingested, the run's own Executive
Report is queryable through EnkiDDB like any other sealed document —
`ORBIT`/`WITNESS` against it the same as any other particle in the
Documentation Database.

## 13. Shakkanakku — the interactive governor (PB-263, `crates/shakkanakku`)

IsimudEngine's own model is "run everything, report at the end" — real,
useful for unattended CI, but the Architect's own complaint about it was
concrete: a long unattended run that hits a red error partway through
still shows only the final status, with the actual failure buried in a
wall of terminal text with no easy way to hand it over for diagnosis.
**Shakkanakku** (`playbook_263_deploy_shakkanakku_governor.yml`, crate at
`crates/shakkanakku`) is the answer — an eframe/egui GUI (or headless CLI,
`--no-default-features`) that runs the same corpus one playbook at a time,
via real `ansible-playbook ... ANSIBLE_STDOUT_CALLBACK=json` subprocesses,
and **halts** the instant a MAJOR failure occurs — not at the end of the
run — showing exactly which playbook, which task, and the real error text,
pending an explicit Architect decision (Skip & continue, or generate a
matched `[[remedy]]` fix playbook and retry). Warnings proceed
automatically, by the same "caution is law, unmatched = MAJOR" default
this engine's own STOP banner already uses.

**Same source of truth, not a second one.** Shakkanakku's decree
(`shakkanakku.toml`, repo root) sets `triage_doc =
"docs/PLAYBOOK_EXECUTION_TRIAGE.md"` — at load time it parses this file's
own tables (`crates/shakkanakku::config::playbooks_from_triage`, logic
equivalent to §"PARSE" above: track `## ` phase headings, split
`| # | File | Status | Note |` rows on `|`, drop `SKIP`, keep document
order and duplicate PB numbers) and uses that as its playbook list,
instead of a hand-copied static list that could silently drift from what
IsimudEngine itself runs. This was a real, explicitly-flagged risk before
PB-263 — "two competing sources of truth" — closed by having Shakkanakku
read the doc directly rather than wrapping `playbook_IsimudEngine.yml` as
a single subprocess (which was considered and rejected: IsimudEngine
invokes each sub-playbook via `ansible.builtin.command`, so its own
top-level JSON callback would only show IsimudEngine's own tasks, not
each sub-playbook's internal task/host/msg detail — exactly the
per-playbook granularity Shakkanakku's halt-and-diagnose UI needs).

**Chronicle, not just a report.** Every run also appends an immutable
JSONL event log (`docs/SHEDU/shakkanakku_chronicle/`, gitignored like
every other `docs/SHEDU/*` run-output) — each line already shaped to
become an Event particle (`kaki_type 0x02`) if ingested into EnkiODB via
the same `enkiddb-cli` pipeline §12 describes, though that ingestion step
is not automated by Shakkanakku itself, same "sealing is a separate,
explicit act" principle as §12.

**Known limits, honestly scoped:**
- The seal key (`secrets/shakkanakku_seal.key`, gitignored) is a bare
  Ed25519 key file on disk, owner-only permissions (`0600`) but otherwise
  unencrypted — a floor, not the destination. Real secret custody in this
  ecosystem is the Sargon Passport Manager vault (`crates/kupru`'s
  Argon2id + Ed25519 + ChaCha20-Poly1305, see §9's Secrets section) —
  Shakkanakku has no scriptable path into that vault yet. Treat this key
  exactly like `akkadian_seal.key`: never commit it, rotate immediately if
  it's ever exposed.
- Abort is responsive at three points — between playbooks, while halted
  awaiting an Architect decision, and (via a poll-and-kill loop around the
  spawned `ansible-playbook` child, not a blocking wait) during a
  playbook's actual execution. It cannot interrupt a single Ansible task
  that itself hangs indefinitely inside one module call — the same class
  of limit IsimudEngine's own `stop_on_error` has at the Ansible-loop
  level (see §2's variable table and `playbooks/isimud_tasks/run_one_pb.yml`).
- `simulate = true` is the checked-in decree's default — an Architect
  flips it to `false` deliberately before Shakkanakku touches real
  infrastructure. It never defaults to live execution.

**A third face: the browser.** The native egui window (`shakkanakku`)
assumes an OS window manager that draws decorations, honors `Maximized`
against the visible viewport, and gives the app a resize border — none of
which held on eriduous-vdi's bare, GPU-less Wayland compositor (no window
chrome at all, `Maximized` filled the compositor's *virtual* desktop rather
than the visible viewer pane, and Mesa's Zink Vulkan-on-GL emulation
produced frame-ghosting under `eframe`'s `wgpu` backend). Rather than keep
chasing native-window behavior on an environment that can't reliably supply
it, `shakkanakku-web` (same `--bin`, `web` feature) puts the identical
engine — `config`/`runner`/`remedy`/`report`/`chronicle`, byte-for-byte
unchanged, now `pub` in a real `crates/shakkanakku` lib target both faces
link against — behind a small hand-rolled HTTP/1.1 server
(`src/bin/shakkanakku_web.rs`, `std::net` only, no async runtime: one
Architect, one process, one `Mutex<ServerState>`, so that's the right
amount of machinery). The frontend is plain HTML/CSS/vanilla JS
(`src/bin/web_assets/`) polling `/api/state` every 700ms and posting small
JSON commands — no WebGL/canvas, so it renders correctly even on a host
whose GPU driver stack is this broken, and CSS Grid (`fr` units, a
mobile-width breakpoint) gives genuine proportional reflow on resize, which
is the one thing the native face could never be made to do reliably here.
Run `shakkanakku-web` from the repo root (same relative-path rule as the
native face) and open `https://<host>:8763/` — port via
`SHAKKANAKKU_WEB_PORT`. `eframe`'s own native GPU/decoration fragility on
bare compositors is real infrastructure history here (see git log on
`crates/shakkanakku/src/{app,main}.rs`), not a one-off; the web face is the
recommended default on eriduous-vdi and similar VDI environments.

**Real auth, not a label — `crates/shakkanakku/src/web_auth.rs` +
`web_tls.rs`.** A network-reachable governor for real infrastructure with
nothing in front of it is not acceptable, so the web face never speaks
plaintext HTTP and never serves a mutating endpoint to an unauthenticated
caller:

- **TLS** (`web_tls.rs`): `rustls` with the `ring` crypto provider
  (deliberately not `aws-lc-rs`, which needs cmake — the same class of
  native-toolchain fragility PB-90's GCC 16 fight already cost real time
  here). A self-signed X.509 certificate (`rcgen`) is minted on first run
  to `secrets/shakkanakku_web_{cert,key}.pem` (gitignored, `0600`, same
  pattern as `report.rs`'s AkkadianSeal key) and reused after that — the
  browser will warn once on an unrecognized self-signed cert; trusting it
  explicitly is the correct, standard thing to do for an internal-only
  tool, not something to paper over with a fake CA.
- **Authentication** (`web_auth.rs`): login is a real Sargon-format
  passport vault (`salt(32) || ChaCha20-Poly1305-sealed JSON`, root phrase
  `"sargon-vault-root"`, AAD `"BahyWay.Ecosystem.v4.VaultBlob"`) — the
  exact byte format and constants `kupru-gdext`'s
  `encrypt_vault_blob`/`decrypt_vault_blob` already use, so a
  `sargon_vault.dat` either the Sargon Passport Manager or the Gilgamesh
  Master Key Godot tool wrote opens here unmodified. `/api/login` runs the
  real Argon2id derive → AEAD decrypt → `SargonPassport::verify_seal()`
  chain against every entry in the vault and signs in as the **strongest**
  one that actually verifies — an Ed25519 signature check, so hand-editing
  `istar.privilege_level` in a copied passport invalidates its seal (kupru
  proves this in `passport.rs`'s own test suite; `web_auth.rs` re-proves it
  survives the extra vault round-trip). Server-side sessions (kupru itself
  has no session concept — this is that layer) are a random 256-bit ID,
  HMAC-signed (`kupru::fast_hasher().keyed_digest`) with a key generated
  fresh in memory each process start, verified with
  `subtle::ConstantTimeEq` on every request, capped at 8h or the
  passport's own expiry, whichever is sooner.
- **Privilege gating**: viewing the run and exporting the sealed report
  needs any valid passport (`privilege_level` ≥ 1, Sargon-minted
  "gardener"); starting, aborting, skipping, retrying, reconfiguring, or
  generating a fix playbook needs `privilege_level == 7` — an
  Architect passport, which in this ecosystem only the Gilgamesh Master
  Key ceremony mints. This is CSR-08's "awaiting the Architect" made
  literal rather than a UI label: the halt banner already said an
  Architect must ratify a MAJOR; now the server enforces which passport
  is cryptographically allowed to click that button.
- **A real login defender**: `LoginDefender` tracks failed attempts per
  source IP and locks that IP out for 15 minutes after 5 failures inside a
  5-minute window (`429 Too Many Requests`, `Retry-After` header) —
  independent of Argon2id's own deliberate multi-second cost per attempt,
  which throttles brute force before the explicit counter even matters.
  Nothing else in this workspace provides this for a login endpoint —
  `musaru-security` and the `hepta-sec-*` crates are content-scanning and
  in-process trust-scoring tools (confirmed by reading their source before
  building this: no IP-based rate limiting, no packet-level firewall, no
  OS-level enforcement anywhere in that stack), not a substitute for it.
- **Network posture**: binds `127.0.0.1` by default regardless of auth —
  defense in depth, and the only posture that needs zero configuration to
  be correct. `SHAKKANAKKU_WEB_BIND` opts into a LAN/`0.0.0.0` address
  explicitly, and the server prints a loud warning when it does.
