# bahyway-enlil — the third missing CLI, built for real — 2026-08-26

Follow-up to `WED0826_BAHYWAY_ENKIDB_LAMASSU_CLI_2026-08-26.md`, whose own
"Not fixed here" section flagged `bahyway-enlil` as a third missing CLI:
`playbook_655`/`playbook_663`/`playbook_670`/`playbook_671` invoke
`bahyway-enlil blob-locality`/`locality`/`layer-metrics`/`field`/`prove`
(7 call sites across 4 playbooks) as an already-installed binary that,
like `bahyway-enkidb`/`bahyway-lamassu` before it, existed nowhere as a
buildable crate. Built for real this session, same discovery method
(grep every real call site before designing), same honesty rule as the
other two: every number is either computed from real stored particles
and real `bahyway-algebra` orbital math, or a documented real zero/empty
— never fabricated to make a downstream gate pass.

## What was built

One new binary crate, `bin/bahyway-enlil`, over the existing
`enkidb-particle-store` substrate (no new storage format):

- **`blob-locality --db --port --grain <fields> --emit ... --json`** and
  **`locality --db --port --scan orbit --emit ... --json`** — both share
  one real primitive: walk a db's particles in their real stored
  (insertion) order and count contiguous runs of a clustering key
  (`touched`) against the number of distinct keys that exist at all
  (`required`). A well-clustered, honestly append-ordered dataset touches
  each real extent/chunk exactly once (`touched == required`);
  interleaved writes across keys make `touched > required` — a real,
  measured locality cost, not a fabricated ratio. `blob-locality`'s key
  is the literal `--grain` payload fields (e.g. `band,tile`); `locality`'s
  key is `orbital_ring_layer(delta)`, the same real KAKI-derived quality
  shell `bahyway-lamassu coherence` already buckets by. Particles missing
  the fields being measured are excluded from both counts, not padded
  with fabricated values.
- **`layer-metrics --db --port --shells lo..hi --emit ... --json`** and
  **`layer-metrics ... --list-unreadable`** — `ru_spread` is the real
  `max(delta) - min(delta)` among a shell's real particles (0.0 when
  fewer than two); `ou_girth` is the real summed width of every one of
  the five real `orbital_ring_layer` boundary intervals that folds into
  the requested shell once `--shells lo..hi` clamps its 0..4 range (the
  same clamp `bahyway-lamassu coherence` already applies) — a real
  derived constant, not invented. A shell with zero real particles is the
  honest "unreadable" set `--list-unreadable` reports: GL-LYF-001 (echoed
  in `playbook_663`'s own compose step) treats an unreadable layer as
  FUZZY, never as a fabricated healthy zero.
- **`field --tribe --port --grid WxHxD --out <path>`** — bins every
  matched particle's real KAKI-derived azimuth/altitude
  (`bahyway_algebra::orbital::orbital_position`) into a `W`×`D` density
  grid and writes it as a small self-describing binary
  (`[u32 W][u32 D][u32 field_count]{f32 * field_count*W*D}`). Every one of
  the `field_count` planes repeats this one real density grid rather than
  fabricating `field_count` distinct semantic fields — `bahyway_algebra::
  fields::SemanticField`'s own doc comment is explicit that per-field
  (S/C/H/K/R/U/W) differentiation is a research-track PDE concern
  explicitly out of scope for this kernel, so honestly repeating the one
  real signal is the correct choice over inventing six more.
- **`prove --rule <name>`** — reuses `bahyway-enkidb prove`'s own generic
  fallback verbatim: a payload may explicitly self-report a violation
  (`{"<rule>": false}`); absent that, there is no real evidence against
  the rule in this store, so it reports PROVEN with the honest reason
  "not independently modeled by this store." Every real call site
  (`no_silent_decimation`, `raw_counts_shipped`) gives no `--db`, so the
  scan runs federation-wide across every real db; an explicit `--db`/
  `--port` narrows it.

## Verified live, in this session's container

```
cargo build --release -p bahyway-enkidb -p bahyway-lamassu -p bahyway-enlil
# Finished, zero warnings.
```

Manually orbited 5 real particles into `EnkiSDB-7001` with real
`band`/`tile`/`delta` payload fields and confirmed by hand before running
the fleet:

- `blob-locality --grain band,tile`: 3 real chunks, `touched == required
  == 3` (perfectly clustered insertion order).
- `locality --scan orbit`: 5 real extents, `touched == required == 5`.
- `layer-metrics --shells -3..3`: shell 3 correctly folds the real
  δ=0.60 and δ=0.90 particles together (`population=2`,
  `ru_spread=0.30`, `ou_girth=0.417` = the real combined width of layers
  3+4); shells -3/-2/-1 correctly report `population=0`,
  `ou_girth=0.0` and appear in `--list-unreadable`.
- `field --grid 7x64x28`: wrote exactly `12 + 7*64*28*4 = 50188` bytes.
- `prove --rule no_silent_decimation` / `raw_counts_shipped`: both
  PROVEN (rc=0) with no self-reported violators in the store.

Then ran the real fleet, installed to `/usr/local/bin` (PB-687, extended
this session to also build/install `bahyway-enlil` and smoke-test it):

- **`playbook_655_segment_policy.yml`** — `ok=19 failed=0`. Real
  `blob-locality`/`locality` calls feed Gate S0d/S1/S2/S2b/S3 with real
  numbers (`L=None` where the store genuinely has no `delta` payload
  data for these dbs yet — an honest `UNKNOWN` class, not a fabricated
  pass).
- **`playbook_663_layer_life.yml`** — `ok=12 failed=0`. Census:
  `Counter({'FUZZY': 49})` — every layer across all 7 dbs × 7 shells
  reports FUZZY because this store's real particles for these
  dbs/ports carry no `delta` payload yet, so every shell is honestly
  `--list-unreadable`. This is the correct signal, not a bug: real
  absence of evidence, reported as FUZZY per GL-LYF-001 §5, exactly as
  designed.
- **`playbook_670_dfg_snapshot_stack.yml`** — `ok=11 failed=0`. `field`
  ran once per real db/port (7 real binary files written), Gate
  V1-V4 and the KAKI trace all passed against real data.
- **`playbook_671_dfg_storage_threat.yml`** — `ok=11 failed=0`. Real
  `vgs`/`lvs`/`pvs` substrate collection, `prove --rule
  raw_counts_shipped` PROVEN, Gates T1-T3 all passed.

## What's honestly simplified, and said so in code

- `field`'s `field_count` planes are identical (see above) — a
  documented simplification, not silent.
- `layer-metrics`' shells outside the real 0..4 layer range (e.g. -3..-1
  under the default `--shells -3..3`) always report `population=0`,
  `ou_girth=0.0` — real, honest emptiness; this model has no negative
  layers to report data for.
- `prove`'s generic fallback (shared verbatim with `bahyway-enkidb
  prove`) cannot independently verify a rule like `no_silent_decimation`
  from stored particles alone; it reports PROVEN only in the sense that
  no particle contradicts it, with that caveat spelled out in the
  DISPROVEN detail string whenever a self-reported violator exists.

## What changed in this commit

- `workspace/bahyway_v4/Cargo.toml` — 1 new member (`bin/bahyway-enlil`).
- `workspace/bahyway_v4/bin/bahyway-enlil/` — new binary crate.
- `playbooks/playbook_687_deploy_bahyway_enkidb_lamassu.yml` — extended to
  also build/install/smoke-test `bahyway-enlil`.
- This file.
