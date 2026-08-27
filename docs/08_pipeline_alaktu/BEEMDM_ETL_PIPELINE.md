# BeeMDM ETL Pipeline

**Standalone component reference. Follows `docs/08_pipeline_alaktu/TRANSPARENCY_STANDARD.md`.
Verified against real source (`bin/bee-watchdog`, `crates/bee-mdm-bus`,
`crates/enkidw`) and the real `docs/17_troubleshooting/TESTING_PLAYBOOK_PHASE1.md` on
2026-07-21.**

---

## What BeeMDM is

`bee-watchdog` (`bin/bee-watchdog`) is the FileWatchDog daemon that runs
the full ETL station chain per ZIP batch dropped into a shared landing
directory. `bee-mdm-bus` (`crates/bee-mdm-bus`) is the real-time status
bus (`EtlTierCounts`: SDB pending/promoted/quarantined, QDB/ODB/DW
totals, current tick, ticks until next sweep) that feeds `dubsar-theater`
and the DubSar visualizer panels. Separately, `enkidw` (`crates/enkidw`)
provides its own LandingZone→PersistedDb ETL path used by `enkidw-cli`
and (as of this session) `enkidw-write-server` — a parallel, simpler
ingestion path for EnkiDW specifically, not the same station chain
described below.

## Directory layout (real, from `bee-watchdog`'s own source)

```
shard/                              (LandingZone root — shared Vagrant folder)
├── *.zip                           ← DROP HERE
├── profiles/
│   ├── {batch}.template.akk        ← AKK template (compiled from .akk PARTICLE)
│   ├── {batch}.dqprofile           ← client SLA rules (nullable, thresholds, dim-tags)
│   └── {batch}.schema.ref          ← compare-tribe-schema reference (auto-saved on first arrival)
├── Processing/
│   └── {batch}_{ts}/               ← extracted entries + .schema (station work area)
└── Moved_To/
    └── {batch}_{ts}/               ← completed batches (all Golden Records persisted)
```

## The real per-ZIP processing order

1. `LandingZone::poll()` — detect new ZIPs.
2. **Musarû security gate** — pre-extraction byte scan (malware signatures).
3. **VGCA-Δ block analysis** — binary trajectory / ZIP-bomb detection; severity ≥4 → reject.
4. **ZIP extraction** — `ZipEngine` (DEFLATE + STORE).
5. `ProcessingZone::stage()` — write entries + `.schema` to `Processing/{batch}_{ts}/`.
6. `BatchSchema::infer()` — entity seed, mandatory/optional attributes, table name.
7. **compare-tribe-schema gate** — arriving schema vs SLA/DubSar SDM reference; `SchemaMismatch` → `DeadVerdict`, batch rejected.
8. Load or auto-generate `ClientDqProfile` from `shard/profiles/{batch}.dqprofile`.
9. **Per-record station chain** (Adad Gate is the sole KAKI issuer):
   - **a. adad-gate** — `ArrivalRecord` → `IdentityKaki` + `EventKaki` + EAV
   - **b. DataStructureStation** — map raw EAV against template
   - **c. data-cleansing** — DAMA-DMBOK DQ.COMPLETENESS / UNIQUENESS / VALIDITY
   - **d. VGCA beam** — required-field template gate on structured EAV
   - **e. client-dq-profile** — EAV + profile → `FuzzyDimensions`
   - **f. score-engine** — `FuzzyDimensions` → B11 → State + Quality + ColorRGB
   - **g. Routing by B11:**
     - Gem/Tribe (B11 ≥ 140) → `ParticleState::Golden` → PermanentStore + PersistedDb
     - Active (B11 100–139) → `ParticleState::Fuzzy` → PersistedDb + StewardStation
     - Dead (B11 < 100) → `ParticleState::Dead` → PersistedDb only
10. Batch-completion event — commit the Entity KAKI to the journal (file-level event).
11. `ProcessingZone::complete()` — move `Processing/{batch}_{ts}/` → `Moved_To/{batch}_{ts}/`.

Client SLA GUI, first step: drop a `.dqprofile` file in `shard/profiles/`
to configure per-attribute rules (nullable, fill_threshold, dim, weight).
Without one, a profile auto-generates from `BatchSchema`
(mandatory→non-nullable, optional→nullable).

## The real test procedure (10M-particle scale)

`docs/17_troubleshooting/TESTING_PLAYBOOK_PHASE1.md` is the authoritative, run-in-order
test manual for the 50-compressed-file / 10-million-particle BeeMDM ETL
test:

- **Corpus:** 7 archive formats (zip, nested.zip, CORRUPT.zip, tar.gz,
  tar.bz2/tar.xz/7z — stub, expected) plus malicious (zip-bomb,
  path-traversal) and corrupt fixtures.
- **Blocks A–F**, top-to-bottom:
  - **A** — NARAMSIN archive decompression (9 tests + 9 corpus checks)
  - **B** — CRC integrity (`bahyway-crc`, 7 tests)
  - **C** — Session registry parsing (6 tests)
  - **D** — ConEngine 7 CSR rules (6 tests — see `CONENGINE_CSR.md`)
  - **E** — HeptaScript at 10M particles (target <1s; `ABORT_SCAN` safety valve)
  - **F** — Full 50-file corpus batch run, gate: zero unexpected errors
- **Pass/fail gate:** every block PASS, the E-004 query <1000ms, F-001 zero unexpected errors.

## Entry criteria before that test may begin (still-open items, stated plainly)

1. ✅ Preflight/Phase A–B green — full-workspace `cargo test` clean.
2. ✅ CSR-08 implemented in code (PB-170) — closed 2026-08-27, see `CONENGINE_CSR.md`. (One honest scope limit remains there: the crate enforces the gate but doesn't itself provide the Architect-confirmation channel.)
3. ✅ GATE-1 (pipeline-step half) ruled and coded — `HeptaGate`.
4. ⏳ Test dataset staged on real hardware — cannot be checked from a sandbox.
5. ❌ Latency budget table accepted as pass/fail criteria — needs the Architect's confirmation.
6. ⏳ KAKI-minting-only-at-`enkidb-ingest::bridge` dry run on 100 records — operational step, runs on real hardware.

Two items are genuinely still open (2 and 5); two require hands-on action
on real hardware this environment cannot reach (4 and 6).

## Verify it yourself

```
cargo test -p bee-mdm-bus -p enkidw
cargo run --bin bee-watchdog -- --help
```
