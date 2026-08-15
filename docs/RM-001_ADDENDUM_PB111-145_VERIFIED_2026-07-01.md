# RM-001 Addendum — PB-111 through PB-145, Verified Against Actual Repository State
Generated: 2026-07-01
Appends to: RM-001 (`docs/BAHYWAY_ECOSYSTEM_V4_ROADMAP.md`)

## Why this addendum exists

A prior addendum draft (PB-145, found in the playbooks folder but never actually
executed or committed) claimed the full PB-111→PB-145 series — 35 playbooks
including PB-119 through PB-136 (ESARHADDON, ASHNAN TIAMAT, NARAMSIN
regression, and the entire INANA series) — was "WRITTEN" and "SEALED." That
claim does not hold up: none of the PB-119 through PB-136 playbook files
exist anywhere in the actual `~/bahyway-ansible/playbooks/` folder, and
several of the crates that later playbooks depend on (`ashnan-kaki`,
`enkidb-enlil-index`, `hepta-shell-index`, `geo-engine`) don't exist in the
repository either. This addendum replaces that draft with a status verified
against what's actually in the repo, not what a summary claimed.

## Verified status — PB-111 to PB-145

| PB | File | Status (verified against repo) |
|---|---|---|
| 111 | enkidb_cqrs_node_verify | Superseded — checks a process (`enkidb-query-server`) that was abandoned mid-migration; rebuilt in a later session against the current index stack |
| 112 | kispu_headstore_benchmark | Superseded — same dead lineage as 111 |
| 113 | naramsin_engine_formalise | Not needed — `naramsin-core` crate does not exist; `naramsin-archive`/`format`/`bridge`/`audit`/`integrity` already cover this without it |
| 114 | naramsin_format_layer | Already implemented — `naramsin-format` exists, tested |
| 115 | naramsin_mashsharu_bridge | Already implemented — `naramsin-bridge` exists (Phase 1 scaffold: format detection + audit trail wired, full decompression pending) |
| 116 | enki_terra_suite_wire | **Redesigned and implemented this session** — original draft assumed `FileProvenance`/`NaruJournal`/`SchemaRegistry` types that were never implemented in `naramsin-bridge`; rewrote `nanshe-ingest`/`esarhaddon-ingest`/`ashnan-ingest` against the real `process()` signature |
| 117 | ashnan_kaki_schema | **Not actually produced** — `ashnan-kaki` crate does not exist despite being marked "WRITTEN" upstream |
| 118 | ashnan_sensor_pilot_setup | **Blocked** — needs a real generated test corpus and SSH access to `enkidb-node-write` (192.168.122.101); also depends on the missing `ashnan-kaki` (117) |
| 119–136 | ESARHADDON / ASHNAN TIAMAT / NARAMSIN regression / INANA series (18 playbooks) | **Do not exist** — claimed "WRITTEN" upstream, never actually produced. This is the numbering gap originally discovered; it was never lost, it was never made. |
| 137 | urnammu_attestationd_impl | Not yet attempted — depends on PB-110's `urnammu-attestationd` stub, which was scaffolded but not implemented |
| 138 | kittu_engine_v1 | Not yet attempted |
| 139 | ninsun_refiner_wire | **Implemented this session** — `ninsun-agent` crate, 7 tests passing |
| 140 | edubba_seal_integration | **Blocked** — needs both live EnkiDB nodes, the missing test corpus, and the missing `ashnan-kaki` |
| 141 | ashnan_regional_extension | **Implemented this session** — regional config JSON (Jordan Valley, Syria Euphrates, Fertile Crescent) + query templates |
| 142 | namtar_domain_scaffold | **Implemented this session** — `namtar-kaki` crate, 4 tests passing |
| 143 | ereskigal_domain_scaffold | **Implemented this session** — `ereskigal-kaki` crate, 4 tests passing |
| 144 | grafana_monitoring_setup | **Blocked** — needs a real `dubsar-workstation` (192.168.122.121) with Prometheus/Grafana already installed and sudo SSH access; nothing to verify without that hardware |
| 145 | rm001_roadmap_update | This document — supersedes the unexecuted draft |

## What actually shipped this session (verified, tested)

- `enkidb-query-server` — rebuilt from scratch against the current index/query stack (the old version was abandoned mid-migration, never deployed, imports crates that no longer exist). Verified end-to-end on both this sandbox and the real VDI: real TCP round-trip, real query parse/execute, real JSON response.
- BIGRING bridge — connects HeptaScript's `ACROSS BIGRING` clause to `bahyway-algebra::orbital`'s real position math. Both pieces existed independently, tested, unwired; now connected. Verified live on the VDI (`[BIGRING]` tag confirmed in server log on a real query).
- `ninsun-agent`, `namtar-kaki`, `ereskigal-kaki` — three real crates, 15 tests total, all passing.
- `nanshe-ingest`, `esarhaddon-ingest`, `ashnan-ingest` — redesigned against the real `naramsin-bridge` API, 9 tests total, all passing. Real CLI smoke test against an actual zip file confirmed end-to-end (`NRM_CLEAN`, correct journal line).
- Two real bugs found and fixed: a non-exhaustive `ArchiveError` match in `naramsin-integrity`, and a HeptaScript keyword/EAV-attribute-name collision (`E[state]` failed to parse because `state` was reserved as the STATE clause keyword).

## What's still genuinely open

1. **PB-117 (`ashnan-kaki`)** — the actual particle schema crate for ASHNAN was never built. This blocks 118 and 140. Worth scoping as real, deliberate design work (particle classes, ColourID B11 test, CMI formula) rather than another scaffold stub.
2. **PB-118, PB-140, PB-144** — all three require infrastructure this environment has no access to (a generated test corpus, live EnkiDB read/write nodes, a monitoring node with Prometheus/Grafana pre-installed). Configs can be prepared, but execution and verification are yours to run.
3. **PB-119–136** — an 18-playbook gap that was never real. If ESARHADDON's SeismicEvent/Survivor/Structural schema, ASHNAN's CMI/IRI/LMI fuzzy formulas, or INANA's tribe-crystallisation logic are still wanted, they need to be designed and built from scratch — there is no prior draft to recover, despite what the roadmap claimed.
4. **PB-137/138** (UrNammu attestation daemon impl, Kittu Engine v1) — not yet attempted this session.

---
*Verified against actual repository state, not against a prior session's self-report. Where the two disagreed, this document defers to what `cargo build`/`cargo test`/`find` actually show.*
