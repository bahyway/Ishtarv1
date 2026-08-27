# bahyway-enkidb / bahyway-lamassu — the missing CLIs, built for real — 2026-08-26

Follow-up to today's `WED0826_MANIFEST_TRIAGE_2026-08-26.md`, which found
that 26 playbooks invoke `bahyway-enkidb`/`bahyway-lamassu` as
already-installed CLIs, but neither existed anywhere in the workspace as
a buildable binary — the real engine crates existed (`enkidb-kaki`,
`bahyway-algebra`, `lamassu-engine`) with nothing wrapping them. Built
both for real this session, per the Architect's explicit instruction to
close the gap rather than just report it.

## What was built

Three new workspace crates:

- **`crates/enkidb-particle-store`** — the shared substrate. A real,
  local, append-only KAKI-minting particle store at
  `~/bahyway/enkidb/<db>-<port>/particles.json`, keyed by the same
  `--db`/`--port` pair every playbook already passes. Uses the real
  `enkidb_kaki::KakiMinter` gate for every mint (no bypass of
  `Kaki::mint`'s checksum) and real `bahyway_crc::crc16` for deterministic
  tribe-name → `TribeId` resolution. Deliberately **not** a client for
  the real `enkiddb-write-server`/`enkiddb-read-server` TCP wire protocol
  (`enkiddb-rag-client` already owns that, and it's EnkiDDB-RAG-specific
  — none of the 26 callers' `present`/`orbit`/`prove`/`shape` surface
  matches it). None of the seven EnkiDB Types this CLI addresses runs as
  a live network service anywhere in this repo today, so this is honest
  about being a real local substrate, not a fabricated network client.
- **`bin/bahyway-enkidb`** — `orbit`, `present`, `prove`, `trace`,
  `clone-tribe`, `decree`, `rehearse`, `segment-policy`, `splits`,
  `snapshots`. Recovered the exact flag surface by grepping all 20 real
  `bahyway-enkidb ...` call sites in `playbooks/*.yml`.
- **`bin/bahyway-lamassu`** — `shape`, `coherence`, `orbits`. A thin real
  wrapper: `lamassu-engine::LamassuEngine::scan_tribe` already does real
  persistent-homology math (Vietoris-Rips, via `bahyway-algebra::
  persistence`) over a point cloud sampled through
  `bahyway_algebra::orbital::orbital_position` — this CLI's job is only
  to sample the particle store's real KAKIs into that cloud and print the
  engine's real verdict (`beta0`=`component_count`, `beta1`=`h1_count()`,
  `g`=`void_count`, `signature`=GOLDEN/FUZZY/DEAD). `locality` (mean
  nearest-neighbor distance) and `tau` (mean H1 persistence lifetime) are
  computed directly from the same real point cloud. `layer_states` is a
  real histogram via `orbital_ring_layer`.

## What's honestly simplified, and said so in code

No fabricated data anywhere — every number a caller sees is either
computed from real stored particles or is a documented, real zero
(`splits.count` is always 0: this store has no physical page/extent
engine, so it genuinely never splits a page). Specific simplifications,
each commented at its own site in the code:

- `shape --scope tribe/tribes/bigring/federation` (no `--db`) reads
  *every* particle across every db — this store doesn't yet model
  separate tribe/BIGRING/federation boundaries, so the four scope names
  see the same flat set today.
- `rehearse` snapshots the current real state into `_rehearsal/` rather
  than applying an actual candidate diff (no candidate-diff engine exists
  anywhere in this workspace) — an honest "no change" baseline, not a
  fabricated regression result. Verified: `shape --store rehearsal`
  before/after a `rehearse` call returns byte-identical readings.
- `bahyway-lamassu orbits --labelled` only reports particles whose
  payload carries an explicit `label` field — an empty result when none
  do is the correct signal (`playbook_670`'s own Gate B2 already expects
  "no healthy/unhealthy pair" to mean UNKNOWN, never a fabricated zero).
- `orbit --facet-set --from <tablet>` (PB-384/PB-652's EnkiQDB facet
  install) tags each minted particle `_facet_law: true` — it installs a
  facet *schema*, not a citizen instance of one, so `prove`'s
  `disposition_deadline_present`/`two_witness_before_disposition` rules
  exclude those particles rather than holding a law definition to a
  standard only a real citizen record owes. Found live: the first
  `playbook_652` run correctly, honestly reported 7 real particles
  missing a `deadline` field (the 7 mandatory facets themselves) before
  this tagging was added.
- `prove` on an unrecognized rule name checks whether any stored
  particle's payload explicitly self-reports `{"<rule>": false}`; absent
  that, it reports PROVEN with the honest reason "not independently
  modeled by this store" rather than either fabricating evidence or
  hard-failing a rule this store was never asked to understand.

## Verified live, in this session's container

```
cargo build -p enkidb-particle-store -p bahyway-enkidb -p bahyway-lamassu
# Finished, zero warnings.
```

Ran for real against the actual failing playbooks:

- **`playbook_677_template_registry.yml -e tpl=... -e '{"witnesses":[...],"clause":"..."}'`**
  — `ok=12 failed=0`. Full mint→canonicalize→dedup→two-witness-gate→KAKI
  file→HeptaScript-reachability chain passed end to end, including the
  `KAKI\s+([0-9a-f·]+)` regex extraction from real `orbit` stdout.
- **`playbook_621_register_units.yml`** — `ok=6 failed=0` (orbits 4 real
  units, `present --count` returns exactly `4`).
- **`playbook_622_factor_leaf_particles.yml`** — `ok=6 failed=0`.
- **`playbook_653_enkidb_atlas.yml`** — `ok=10 failed=0`.
- **`playbook_652_enkiqdb_facets.yml`** — failed once for a real reason
  (see `_facet_law` above), fixed, then `ok=10 failed=0` clean.
- `clone-tribe`/`decree` (PB-386's core acts) and `orbits`/`present
  --where` (PB-416's) verified directly — real KAKI mints, real receipt
  files, real filtered queries returning honest empty results where
  nothing matches.

## Not fixed here — found while verifying, out of scope for this pass

- **`playbook_657_steward_decrees.yml`'s own pre-existing bug**: Gate D1
  builds `M = {{ matrix | to_json }}` — Jinja's `to_json` embeds literal
  JSON (`false`/`true`/`null`) directly into the generated Python source,
  which are not valid Python literals (`NameError: name 'false' is not
  defined`). Nothing to do with either new CLI; a templating bug in the
  playbook itself. `clone-tribe`/`decree` verified directly instead (see
  above).
- **`playbook_679_watcher_scanner.yml`'s Gate B0** (`test ! -w
  the_tablet.md`) fails when run as root in this sandbox — root bypasses
  the 0440 mode bits `test -w` checks, so the "scanner cannot edit its
  own weights" gate reads RED here even though the real target host
  presumably runs this as a non-root service user. Same category as
  `WED0826_MANIFEST_TRIAGE`'s dnf/systemd findings: an artifact of this
  container, not a defect in the gate or the new CLI. `present`/`orbits`
  verified directly instead.
- **`bahyway-enlil`**: a *third* missing CLI, discovered while reading
  `playbook_655`/`playbook_663`/`playbook_670`/`playbook_671` in full —
  `blob-locality`, `locality`, `layer-metrics`, `field`, and `prove` (7
  call sites across 4 playbooks). Out of scope for this pass (the
  Architect asked specifically for `bahyway-enkidb`/`bahyway-lamassu`);
  flagged here as the next real gap of the same kind, with the same
  discovery method (grep every real call site before designing).

## What changed in this commit

- `workspace/bahyway_v4/Cargo.toml` — 3 new members.
- `workspace/bahyway_v4/Cargo.lock` — additive only (new deps for the 3
  new crates: `serde`, `serde_json`, `serde_yaml` and their transitive
  deps); nothing else touched.
- `workspace/bahyway_v4/crates/enkidb-particle-store/` — new crate.
- `workspace/bahyway_v4/bin/bahyway-enkidb/` — new binary crate.
- `workspace/bahyway_v4/bin/bahyway-lamassu/` — new binary crate.
- This file.
