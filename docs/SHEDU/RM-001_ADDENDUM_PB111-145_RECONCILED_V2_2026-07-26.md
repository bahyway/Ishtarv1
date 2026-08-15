# RM-001 Addendum v2 (Reconciled) — BahyWay.Ecosystem v4.0 Roadmap Update
## Playbook Series PB-111 through PB-145 — corrected against real merged work
Generated: 2026-07-26 | Node: eriduous-vdi

This supersedes both the original unexecuted PB-145 draft (which
falsely claimed PB-119-136 "WRITTEN") and this playbook's own v1
reconciliation (which correctly said the playbook FILES don't
exist, but didn't know that most of the underlying CONTENT had
already been built directly and merged via PR #17/#18 in a
separate session). Full workspace test result at generation time:
  test result: ok. 65 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

---

## What PB-119-136 was supposed to build, and what's actually real

| Intended scope | Status | Evidence |
|---|---|---|
| ESARHADDON KAKI schema (SeismicEvent/SurvivorSignal/StructuralState) | **REAL** | `crates/esarhaddon/src/kaki.rs` |
| ESARHADDON TIAMAT SMI formula | **REAL, tested** | `crates/esarhaddon/src/smi.rs`, 2 tests |
| ESARHADDON Survivor Signal Aggregator | **REAL** | `crates/esarhaddon/src/survivor.rs` |
| ESARHADDON HeptaMap E7 rescue grid | **REAL** | `crates/esarhaddon/src/grid.rs` |
| ESARHADDON atmospheric/gas threat model | **REAL** | `crates/esarhaddon/src/atmosphere.rs` (CH4/CO/H2S/PM2.5 lethality) |
| ESARHADDON DubSar Theater scenes | **NOT BUILT** | no Godot scene code anywhere |
| ESARHADDON field ruggedisation (mesh radio) | **NOT BUILT** | nothing found |
| ASHNAN TIAMAT CMI/IRI/LMI formulas | **REAL, tested** | `crates/ashnan/src/{cmi,iri,lmi}.rs` |
| ASHNAN provenance / confidence tiers (TIER 0-3) | **REAL** | `crates/ashnan/src/provenance.rs` |
| ASHNAN external ingestion adapters (FAO/WOAH) | **NOT BUILT** | tier framework exists, no actual adapter code |
| ASHNAN DubSar Theater scenes | **NOT BUILT** | nothing found |
| ASHNAN regional extension (Jordan/Syria/Fertile Crescent) | **REAL** | PB-141 reconciled this session (config only) |
| NARAMSIN regression pass (7-archive corpus) | **NOT BUILT as own suite** | individual crates (archive/format/integrity/audit/bridge) exist and pass tests; no dedicated regression harness |
| INANA series entirely (KAKI schema, FETCH wiring, tribe crystallisation, advertiser reports) | **DOES NOT EXIST** | no crate, no doc, not mentioned in any session — pure fabrication in the original PB-145 draft |
| NAMTAR / EREŠKIGAL scaffolds | **REAL, tested** | `namtar-kaki`/`ereskigal-kaki`, 4 tests each (reconciled this session) |
| NINSUN agent + Steward advisory bridge | **REAL, tested** | `ninsun-agent`, `ninsun-steward-bridge` |
| ENKI-TERRA ingest wiring (nanshe/esarhaddon/ashnan-ingest) | **REAL, tested** | redesigned against the actual `naramsin-bridge::process()` signature, 9 tests |
| BlackBox Station / Storage Sector quarantine split | **REAL, tested** | `blackbox-station`, `storage-sector`, wired into `SchedulerLoop::run_sweep()` |
| `enkidb-query-server` + BIGRING orbital bridge | **REAL** | rebuilt from scratch, `ACROSS BIGRING` clause wired to `bahyway-algebra::orbital` |
| bee-watchdog restart-duplicate-ingestion bug | **FOUND AND FIXED** | real production bug, `archive()` step added, confirmed at `bin/bee-watchdog/src/main.rs:929` |
| `enkidb-quantdb` (EnkiMDB slot) workspace registration | **FOUND ORPHANED, FIXED** | had 32 real tests, was never in `cargo build --workspace` scope until fixed |

All of the above marked REAL/tested was independently re-verified
against this repo — not taken on the source documents' word —
via `cargo test --workspace`: 288 test suites, 0 failures.

---

## Playbook Status Table — PB-111 to PB-145

| PB  | File | Module / Layer | Status |
|-----|------|---------------|--------|
| 111 | enkidb_cqrs_node_verify | EnkiDB — CQRS health gate | RECONCILED |
| 112 | kispu_headstore_benchmark | KISPU — sub-1s performance gate | RECONCILED |
| 113 | naramsin_engine_formalise | NARAMSIN — archive layer + NRM_* codes | RECONCILED |
| 114 | naramsin_format_layer | NARAMSIN — CSV/JSON/XML format layer | RECONCILED |
| 115 | naramsin_mashsharu_bridge | NARAMSIN — MASHSHARU bridge + NĀRU | RECONCILED |
| 116 | enki_terra_suite_wire | ENKI-TERRA — NARAMSIN ingestion wiring | RECONCILED |
| 117 | ashnan_kaki_schema | ASHNAN — 4 particle types + ColourID B11 | RECONCILED |
| 118 | ashnan_sensor_pilot_setup | ASHNAN — Diyala pilot + test corpus | BLOCKED (needs corpus + SSH) |
| 119-136 | ESARHADDON / ASHNAN TIAMAT / NARAMSIN regression / INANA | **PARTIALLY REAL** — see table above; playbook files never existed, but ESARHADDON/ASHNAN core + ingest/quarantine work is real; INANA is pure fabrication |
| 137 | urnammu_attestationd_impl | UrNammu — TPM2 boot attestation daemon | IMPLEMENTED (11 tests) |
| 138 | kittu_engine_v1 | Kittu — ShoWEngine + email delivery | IMPLEMENTED (4 tests) |
| 139 | ninsun_refiner_wire | NINSUN — progressive refiner | REAL (`ninsun-agent`, separate earlier session) |
| 140 | edubba_seal_integration | É-DUBBA — 6-stage integration test | RECONCILED |
| 141 | ashnan_regional_extension | ASHNAN — Jordan/Syria/Fertile Crescent | RECONCILED |
| 142 | namtar_domain_scaffold | NAMTAR — sacred burial | IMPLEMENTED (namtar-kaki, 4 tests) |
| 143 | ereskigal_domain_scaffold | EREŠKIGAL — informal settlement | IMPLEMENTED (ereskigal-kaki, 4 tests) |
| 144 | grafana_monitoring_setup | Grafana+Prometheus on dubsar-workstation | CONFIGS READY, deploy BLOCKED (no live hardware) |
| 145 | rm001_roadmap_update | This document (v2) | RECONCILED |

---

## Genuine remaining gaps (confirmed absent, not just unverified)

1. **INANA series** — entirely fabricated in the original PB-145 draft. No crate, no doc, no design. Would need to be designed from scratch if wanted.
2. **NARAMSIN regression pass** — no dedicated 7-archive-corpus regression suite exists as its own artifact.
3. **DubSar Theater scenes** for ESARHADDON and ASHNAN — Godot visualization work, not started.
4. **Field ruggedisation** — mesh radio transport, battery-backup field node — not started.
5. **Real EnkiDDB (Documentation DB) and EnkiMDB (Metadata DB)** in the Architect's actual intended sense — `enkidb-recovery` and `enkidb-quantdb` are legitimate crates but were mislabeled onto those two port slots by a prior session; the real Documentation/Metadata databases (to back MetaEngine and TemplateEngine respectively) don't exist yet. Explicitly deferred by the Architect to a future session.
6. **Nergal AV backend** — design-only: a manual spec (`BAHYWAY_ECOSYSTEM_MANUAL_V4.md` §16) and a client-side Three.js prototype exist; no scanning backend, no DubSar IDE freeze/jam hooks, no website firewall.
7. **SLA Layer resolution** — naming conflict between `sla-engine` (app go-live governance) and `client-dq-profile` (the one actually wired into bee-watchdog, tagged "BeeMDM §SLA") never resolved.
8. **EnkiDB storage engine rearchitecture** — `PersistedDb` still does full journal-replay on open, `MmapReader` is not actually memory-mapped (eager `fs::read()`). This is the real blocker for 1B+ particle scale, and it's exactly the scope of ADR-012 (Data Files Law) written earlier this session — architecture is documented, implementation has not started.

---

*End of RM-001 Addendum v2 (Reconciled) — PB-111 to PB-145*
