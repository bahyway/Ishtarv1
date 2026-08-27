# PB-672 Backlog Triage — 2026-08-26

Continuing the PB-609/PB-672 unattended manifest walks from a fresh
`claude/ecosystem-delivery-rd1ksb` session (a brand-new container cloned
from `origin/master`, not the Architect's real `uruk`/`girsu` bare-metal
box). Started from the 6 outstanding failures the prior session's
checkpoint left in `naru/run_manifest_672_log.jsonl` (PB-670/671/675/
677/678/679) and ended up finding something more structural. Recorded
here in the same spirit as `docs/16_runbooks/PLAYBOOK_EXECUTION_TRIAGE.md`
— real root causes, not just status flips.

## Real, fixed: two missing OS packages

Neither `ansible` nor `ansible-playbook` was even installed in this
session's container (`pip3 install --user ansible cffi` was needed just
to run anything — recorded here because the next fresh session will hit
the same gap). Two further **OS-level** binaries the Phase-3 corpus
shells out to were also missing:

- **`lvm2`** (`vgs`/`lvs`/`pvs`) — `playbook_671_dfg_storage_threat.yml`
  reads the volume manager as its first MEASURED step. Installed
  (`apt-get install -y lvm2`); re-ran the playbook standalone
  (`ansible-playbook playbooks/playbook_671_dfg_storage_threat.yml`,
  not through the walker) and it now passes clean end to end
  (`ok=11 changed=3 failed=0`) — a real, independently-verified fix, not
  dependent on any other PB's side effects.
- **`openssh-client`** (`ssh`) — every playbook that `delegate_to`s
  `uruk-node-write`/`-read`/`-vault` (PB-310/311/312/314/320/558, and
  others) needs a real `ssh` binary on the controller even when the
  target resolves back to `localhost`. Installed
  (`apt-get install -y openssh-client`).

Same category as `requirements-ansible-python.txt` (added 2026-08-26
for `requests`/`docker`) — a fresh session/host needs `lvm2` and
`openssh-client` present before the Phase-3 corpus can run for real.

## Real, fixed: PB-677/678/679 were never bugs — reclassified

`playbook_677_template_registry.yml`, `playbook_678_pdm_shape_admission.yml`,
and `playbook_679_watcher_scanner.yml` each carry an explicit assert gate
with a `fail_msg` naming the exact `-e` flag an operator must pass
(`tpl=`, `cand=`, `who=`) — by design, refusing to mint/admit/harvest
anything without a named candidate. Running them bare, as the unattended
walker does for the rest of the backlog, was always going to fail every
single time; that's not a scaffolding defect, it's the safety gate
working. Removed from `playbooks/data/run_manifest_672_backlog.yml`
(same treatment the file already gives the ~21 html-only Šala tabs) —
see the comment left in place there and in `playbook_672`'s own header
for how to invoke each one for real.

## Not fixed, and not a code bug: the dnf/systemd/SSH-fleet failures

Re-running the remaining PB-670/675 by hand (after the two package
fixes above) still fails — both shell out to `bahyway-enkidb` /
`bahyway-lamassu`, and neither exists anywhere in this repo as a
buildable binary (see next section). Separately, spot-checking a sample
of the ~20 other PB-609-backlog entries already logged as failed
(PB-160, 310-320, 421, 532, 542, 556-558) found every one of them
failing for a reason specific to *this container*, not the playboots:

- `ansible.builtin.dnf` tasks (PB-421/542/556) fail because this
  container's real package manager is `apt` (Ubuntu) — the real target
  host (`uruk`) is Fedora, per PB-253's own "Fedora44 I/O tuning" and
  this doc's own `cd ~/Forge/EnkiDB/playbooks` convention. Correct for
  the real box, wrong only here.
- `ansible.builtin.systemd` tasks (PB-319) fail because this container
  has no systemd as PID 1. Same story — real on `uruk`, absent in a
  throwaway container.
- Hardcoded `{{ ansible_env.HOME }}/Forge/EnkiDB/...` and
  `{{ ansible_env.HOME }}/BahyWay` paths (PB-160, PB-558) are the real
  target host's actual checkout convention (matches
  `PLAYBOOK_EXECUTION_TRIAGE.md`'s own run instructions verbatim) — this
  repo happens to be cloned at `/home/user/EnkiDB` in *this* session
  only.
- PB-313/532/557 fail on their own deliberate refusal gates (no prior
  seal yet / missing source artifact / no
  `-e i_understand_this_is_production=true`) — correct behavior, not bugs.

**None of these should be "fixed"** by loosening the playbooks to
tolerate Ubuntu/apt/no-systemd/wrong-paths — that would degrade
correctness on the real target hardware to satisfy a sandbox that was
never meant to be the runtime host. Recorded here so the next session
doesn't spend time chasing these as regressions.

## The one real, portable, still-open gap: `bahyway-enkidb` / `bahyway-lamassu` do not exist

This is the actual finding worth acting on. `bahyway-enkidb` is invoked
by **20** playbook files and `bahyway-lamassu` by **6** (26 total,
`grep -rl` count) as an already-installed CLI — but a full audit of
every `[[bin]]` target in `workspace/bahyway_v4/crates/*/Cargo.toml`
turns up no crate that builds either binary. The real engine code exists
(`enkidb-engine`, `enkidb-kaki`, `enkidb-journal`, etc. and
`lamassu-engine`/`lamassu-cadenced`) — there is simply no CLI crate
wrapping it under either name, on any host, real or sandboxed. This is
independent of dnf-vs-apt, systemd, or checkout path: it's why PB-670
and PB-675 fail and would fail identically on `uruk` today.

**Recommended next step for a future session:** scaffold a thin
`bahyway-enkidb-cli` binary crate (and `bahyway-lamassu-cli`) in
`workspace/bahyway_v4/crates/`, wrapping the existing engine crates, that
implements the subcommands the 26 playbooks actually invoke (`present`,
`orbit`, `--tribe`/`--port`/`--json`, `shape --scope ... --emit ...`,
etc. — the real call signatures are all recoverable by grepping the 26
callers). Not attempted in this session — it's a real design decision
(argument surface, output format) that deserves its own pass rather than
being rushed as a side effect of a triage walk.

## A more structural finding: cross-session resume is unsafe as designed

Re-running `playbook_672_run_manifest_phase3.yml`'s full walk in this
fresh container (to pick up the PB-671 fix) surfaced something bigger
than any single PB: **PB-618/619/620** (the Kaniku Seal chain — sealed
for real in the prior session, per `naru/run_manifest_672_log.jsonl`'s
own `live_rc: 0` entries and the `Fix two real bugs blocking the CSR-08
seal chain (PB-618/619/620/654)` commit) **failed all over again here**,
along with everything downstream that depends on them being sealed
(PB-621/623/624/625/626/628/631-636/646-649/654/655/657/663).

Root cause: `run_manifest_one.yml`'s resume support only checks whether
`naru/run_manifest_672_log.jsonl` (git-committed) already has a
`live_rc: 0` line for a file, and skips re-running it if so. But the
actual filesystem side effects those playbooks produce — everything
under `{{ ansible_env.HOME }}/bahyway/...` — are **not git-tracked** and
live only in whatever container ran them. Each Claude Code on the web
session gets a brand-new, freshly-cloned container (see this repo's own
"Environment configuration" — "the repository was cloned fresh when the
container started"), so the committed log's claim of success travels
across sessions but the state it was true *of* does not. The walker then
correctly skips re-doing PB-618-620's work (trusting the log), and the
first real seal-chain check downstream correctly fails, because in this
container that work never actually happened.

This isn't a bug in any individual playbook — every one of the "new"
failures above is that playbook correctly detecting missing prerequisite
state. It's a gap in the walker's own resume design: **`live_rc: 0` in
the journal means "this succeeded in some past container," not "this
container's filesystem is in the state this succeeded from."** A full
walk only means what the log says it means when run start-to-finish in
one continuous environment (the real `uruk` box); resuming a partial
walk across separate ephemeral sandbox sessions will produce a long tail
of false failures exactly like this triage did, without fixing anything.

**Not fixed here** — the two honest options are (a) make `~/bahyway`
(or an equivalent state directory) persist across sessions somehow, or
(b) treat any full walk started in a fresh container as needing to run
truly start-to-finish, ignoring the resume-skip for chain-gated ranges.
Left for the Architect to decide; this session reverted the log file
back to its prior committed state (`git checkout --
naru/run_manifest_672_log.jsonl`) rather than commit the 26 false-failure
lines this walk produced, so the historical record stays trustworthy.

## What actually changed in this commit

- `playbooks/data/run_manifest_672_backlog.yml` — PB-677/678/679 removed
  (operator-tool reclassification, see above).
- `playbooks/playbook_672_run_manifest_phase3.yml` — header note updated
  to match.
- This file.

`lvm2`/`openssh-client` installs and the standalone PB-671 re-run were
real and verified but are container-local (not something to commit);
recorded here so the next session's `ansible-galaxy`/`pip` bootstrap step
knows to add them.
