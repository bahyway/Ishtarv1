# Testing the Full BeeMDM ETL Processing Stations Chain at 1 Billion Rows (Synthetic NajafEngine Data)

## Scope, stated honestly up front

This guide exercises the ONE real, wired station chain confirmed this
session by direct code reading: `bin/bee-watchdog`'s own `main()` —
`LandingZone::poll` → Musarû security gate → VGCA-Δ block analysis → ZIP
extraction → `ProcessingZone::stage` → `BatchSchema::infer` → compare-
tribe-schema gate → per-record `adad-gate` (sole KAKI issuer) →
`DataStructureStation` → `data-cleansing` → VGCA beam → `client-dq-profile`
→ `score-engine` → B11 routing (Golden→`PermanentStore`+`PersistedDb`;
Fuzzy→`PersistedDb`+`StewardStation`; Dead→`PersistedDb` only).

`eridu-runtime::SchedulerLoop`'s separate SDB→ODB/QDB tier-transition
system is NOT exercised by this guide — it's a real but currently
disconnected second pipeline (fed by whoever calls `SdbStore::stage()`,
not by `bee-watchdog`), out of scope for "the BeeMDM ETL Processing
Stations Chain" as that phrase is used in this repo's own docs
(`docs/08_pipeline_alaktu/BEEMDM_ETL_PIPELINE.md`'s subject).

No `docs/17_troubleshooting/TESTING_PLAYBOOK_PHASE1.md` exists in this checkout (checked
directly — the 50-file/10M-particle test manual referenced in an external
document is not part of this repo). This guide is a fresh, from-scratch
1B-row procedure built directly from the two real binaries this session
already has: `bin/najaf-gen` (synthetic data generator) and
`bin/bee-watchdog` (the real ETL chain).

---

## 0. Honest cost estimate before you start

`najaf-gen`'s own printed estimate: ~150 bytes/row. At 1,000,000,000 rows
that's **~140 GB of raw CSV on disk**, before ZIP wrapping — and
`najaf-gen --zip` uses STORE-only (uncompressed) ZIP (`enkidw::zip_engine::
build_store_zip`, confirmed by reading that function), so `--zip` does
**not** shrink this. Budget ~140 GB free on whatever filesystem
`--out-dir` points at, and separately, real disk for `bee-watchdog`'s own
`PersistedDb` (`--data-dir`) and `PermanentStore` output, which will grow
by roughly the same order of magnitude as records are processed and
land. Run `df -h` on the real target filesystem yourself first —
`najaf-gen` does not check available space before writing.

This is a real, multi-hour run at this scale on typical hardware — plan
for it, don't expect it to finish in minutes.

---

## 1. Generate 1 billion synthetic NajafEngine records

```bash
cargo build --release -p najaf-gen -p bee-watchdog

# 1B rows, 2M rows/file -> 500 real STORE-zip files, each independently
# a valid ZIP bee-watchdog's LandingZone can pick up.
./target/release/najaf-gen \
  --total-rows 1000000000 \
  --rows-per-file 2000000 \
  --zip \
  --seed 1400 \
  --progress-every 50000000 \
  --out-dir /data/najaf_1b_shard
```

- `--seed 1400` is a deliberate choice, not arbitrary — it's the Hijri
  epoch najaf-engine's own `GraveParticle` doc comment uses as its worked
  example (1400 = 1979 CE); any seed reproduces byte-identical output on
  repeat runs, useful for a controlled re-test.
- Watch the progress line every 50M rows; at real disk-write throughput
  this is the honest way to estimate total wall-clock time before
  committing to the full 1B run unattended.
- Output: `/data/najaf_1b_shard/najaf_00000.zip` … `najaf_00499.zip`
  (500 files of 2,000,000 rows each = 1,000,000,000 rows total).

**Optional — test the pipeline shape at smaller scale first.** Given the
real, multi-hour cost above, run the exact same command with
`--total-rows 10000000` (10M, matching this repo's own Phase 1 gate in
`docs/19_roadmap/BAHYWAY_ECOSYSTEM_V4_ROADMAP.md`) into a separate `--out-dir`
first, confirm the full procedure below works end to end, THEN commit to
the 1B run. This is not a required step, but is the honest, low-risk way
to catch a configuration mistake before it costs hours instead of minutes.

---

## 2. Start the real BeeMDM ETL chain against that shard

```bash
mkdir -p /data/najaf_1b_state

./target/release/bee-watchdog \
  --shard /data/najaf_1b_shard \
  --data-dir /data/najaf_1b_state \
  --tribe-id 0x0001 \
  --interval-ms 500
```

What this actually runs, per file, in order (confirmed by direct code
reading of `bin/bee-watchdog/src/main.rs`, not assumed):

1. `LandingZone::poll` finds the next `.zip` in `--shard`.
2. Musarû security gate scans the ZIP itself (`musaru_security::zip_scan`)
   before anything is extracted — a malicious/corrupt archive is rejected
   here, never opened further.
3. VGCA-Δ block-feature analysis (`vgca_delta`) on the raw archive.
4. ZIP extraction → `ProcessingZone::stage`.
5. `BatchSchema::infer` on the extracted CSV's header row, then
   `compare_versions` against the prior schema version for this tribe
   (schema-drift gate — a changed header shape between batches is a real,
   detected event, not silently accepted).
6. Per record: `adad_gate::AdadGate` — the SOLE KAKI issuer in this whole
   chain (no other station mints identity) — then `DataStructureStation`,
   `data_cleansing_station::cleanse`, a VGCA beam validation pass,
   `client_dq_profile` dimension scoring, and `score_engine::score` →
   B11 tier.
7. Routing by B11 outcome: Golden → `PermanentStore` + `PersistedDb`;
   Fuzzy → `PersistedDb` + `StewardStation` (queued for human review);
   Dead → `PersistedDb` only, never promoted further.

The B11 distribution `najaf-gen` itself generates (from its own
`gen_row`, hand-verified against its actual roll thresholds) is
deliberately mixed: roughly 39% of rows land at or above the Golden
threshold (140), roughly 51% land in the Fuzzy band (60–139), and
roughly 10% fall below the Dead threshold (60) — so a real run at any
scale exercises all three routing paths, not just the happy path.

---

## 3. Monitor a long-running 1B-row pass without needing DubSar/the web UI

`bee-watchdog` writes 3 real JSON files into `--data-dir` after every
processed batch (confirmed in `main.rs`, these are the same files
`bahyway-api`/the web UI read, but you don't need either running):

```bash
watch -n 5 'cat /data/najaf_1b_state/live_export.json'
```

`live_export.json` fields: `timestamp`, `last_batch`, `total`, `golden`,
`fuzzy`, `dead`, `batches` — a running tally, updated after each of the
500 batches. `tribes_summary.json` gives the same numbers keyed by
`tribe_id` (useful once you run more than one tribe concurrently);
`batches_export.json` gives the full per-batch history if you need to
audit which specific batch produced an anomaly.

If Nisaba's Architect-only digest (`nisaba::orchestrator::
NisabaOrchestrator::observe_etl_tier_counts`, built this session) is
wired to poll `bee-mdm-bus::EtlTierCounts` during the run, a real 1B-row
pass is exactly the kind of sustained load its `sdb_pending_backlog_threshold`
(default 10,000) and malware-hit checks are meant to catch — check
`architect_digest()` periodically for `HighPriority` findings rather than
only reading the raw JSON exports.

---

## 4. Pass criteria (adapted from this repo's own Phase 3 gate)

`docs/19_roadmap/BAHYWAY_ECOSYSTEM_V4_ROADMAP.md`'s Phase 3 defines the 1B-particle
production gate in terms of HeptaScript query latency across all 7 EnkiDB
types. For this specific test (the BeeMDM station chain itself, not
HeptaScript querying afterward), the honest pass criteria are:

- [ ] All 500 ZIP files are consumed by `LandingZone` (none left
      unprocessed in `--shard` after the run — check the "Moved_To"
      directory `main.rs` logs at startup, where processed archives land).
- [ ] `live_export.json`'s final `total` equals exactly 1,000,000,000.
- [ ] `golden + fuzzy + dead == total` at every checkpoint (no record
      silently dropped between stations).
- [ ] No Musarû/VGCA-Δ rejection on any of the 500 files (this is
      synthetic, trusted-source data — a rejection here would indicate a
      bug in `najaf-gen`'s ZIP construction, not a real security event).
- [ ] Zero schema-drift halts (`compare_versions`) across all 500
      batches — `najaf-gen` emits one fixed header shape for the whole
      run, so a drift halt would indicate a real bug, not expected data
      variation.
- [ ] `StewardStation`'s queue after the run contains every Fuzzy-tier
      record and only Fuzzy-tier records (spot-check a sample against
      `batches_export.json`'s recorded fuzzy count).
- [ ] Once this passes, HeptaScript query-latency testing against the
      resulting `PermanentStore`/EnkiDB Golden Records content is the
      separate, next real test (Phase 3's own `< 1 second at 1B
      particles` gate) — not part of this guide's scope.

---

## 5. What this guide deliberately does not attempt

- It does not exercise `eridu-runtime::SchedulerLoop`'s SDB/ODB/QDB tier
  system — no code path in this repo currently feeds `bee-watchdog`'s
  output into `SdbStore::stage()`, so there is nothing real to test there
  yet (a genuine, separately-flagged architectural gap, not something
  this guide can honestly claim to cover).
- It does not run the Data Steward's 3 real decision paths
  (`resolve_requeue_to_sdb`/`resolve_promote_to_odb`/`confirm_quarantine`)
  against the Fuzzy-tier backlog this generates — that's a real, separate
  manual (or agent-driven, per CSR-08 advisory-never-authority) review
  step, deliberately left to the Architect/Data Steward rather than
  scripted here.
