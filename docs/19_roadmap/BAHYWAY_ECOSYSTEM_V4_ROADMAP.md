# BahyWay.Ecosystem v4.0 — Final Production Roadmap

> "الحالة القديمة لا تُمحى أبدًا" — The old state is never erased.

---

## Executive Architecture

BahyWay.Ecosystem v4.0 is a sovereign, append-only, Triple-O (Orbit-Oriented Ontology)
data platform built in pure Rust with zero external crate dependencies (§0.3 Rule).
Every particle carries an immutable KAKI identity.  All state changes are journalled;
nothing is ever deleted.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        BahyWay.Ecosystem v4.0                               │
│                                                                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │ HeptaScript  │  │  ConEngine   │  │  NARAMSIN    │  │  BeeMDM ETL  │  │
│  │   v2.0 W5H2  │  │  7 CSR Rules │  │  Stage 0+1   │  │  6 Stations  │  │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  │
│         │                 │                  │                  │           │
│  ┌──────▼─────────────────▼──────────────────▼──────────────────▼───────┐  │
│  │                    ŠUMU-UKIN Routing Layer                            │  │
│  │          tribe_id → session_registry → fan-out                       │  │
│  └──────────────────────────────┬────────────────────────────────────────┘  │
│                                 │                                           │
│  ┌──────┬───────┬───────┬───────▼──────┬───────┬───────┬───────────────┐  │
│  │7001  │ 7002  │ 7003  │ 7004         │ 7005  │ 7006  │ 7007          │  │
│  │EnkiDB│EnkiDW │EnkiSDB│EnkiODB       │EnkiQDB│EnkiMDB│EnkiDDB        │  │
│  └──────┴───────┴───────┴──────────────┴───────┴───────┴───────────────┘  │
│                                                                             │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │              Sovereign Index Layer (enkidb-indexes)                  │  │
│  │    NatiruIndex (orbital-bucket)  +  EavExactIndex (Xor8+BSearch)   │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Phase 0 — Foundation (COMPLETE ✓)

| Crate | Status | Description |
|-------|--------|-------------|
| `bahyway-crc` | ✓ CRC-16 done, CRC-32 added | KAKI integrity + ZIP/gzip validation |
| `enkidb-kaki` | ✓ | 16-byte universal particle identity |
| `enkidb-journal` | ✓ | Append-only EAV journal, 64-byte blocks |
| `enkidu-protocol` | ✓ | HEPT wire protocol, SPSC ring, frame codec |
| `heptascript` v2.0 | ✓ | W5H2 query language, AnchorStrategy, execute_stream |
| `enkidb-indexes` | ✓ | NatiruIndex + EavExactIndex |
| `naramsin-archive` | ✓ skeleton → ✓ implemented | Stage 0 ZIP/tar.gz/stubs |
| `naramsin-format` | ✓ | Stage 1 CSV/JSON/XML parsing |
| `enkidb-session-registry` | ✓ | Sync session registry, native TOML parser |
| `enkidb-con-engine` | ✓ | 7 CSR security rules, NĀRU WAL |

---

## Phase 1 — BeeMDM Testing (TOMORROW → Week 1)

### Test Corpus: 50 Compressed Files
- 7 archive formats: zip, nested.zip, CORRUPT.zip, tar.gz, tar.bz2, tar.xz, 7z
- Malicious files (zip-bombs, oversized ISIZE, path-traversal names): rejected by NARAMSIN
- Corrupt files: Truncated, BadSignature → `NRM_TRUNCATED` / `NRM_UNKNOWN_FORMAT`

### Particle Scale: 10 Million Particles

```
Target:  HeptaScript execute_stream < 1 second on all 7 EnkiDB types
Method:  ANCHOR SurrogateTime → NatiruIndex prune → EavExactIndex filter → stream
Safety:  ABORT_SCAN 5_000_000 (emergency cut-off at 50% scan)
```

### Test Matrix

| Test ID | Description | Accept Criteria |
|---------|-------------|-----------------|
| TST-001 | ZIP extract (STORE) | All files extracted, CRC-32 verified |
| TST-002 | ZIP extract (DEFLATE) | Inflate correct, CRC-32 match |
| TST-003 | Corrupt ZIP | `NRM_TRUNCATED` returned, no partial data |
| TST-004 | Nested ZIP | Max depth 4, bomb rejected at depth 5 |
| TST-005 | tar.gz extract | gzip unwrap → tar reader correct |
| TST-006 | tar.bz2 | `UnsupportedFormat` stub returned |
| TST-007 | tar.xz | `UnsupportedFormat` stub returned |
| TST-008 | 7z | `UnsupportedFormat` stub returned |
| TST-009 | Malicious path (`../../../etc/passwd`) | Name sanitised or rejected |
| TST-010 | HeptaScript 10M particles, NODE scan | < 1 second |
| TST-011 | HeptaScript 10M, ANCHOR SurrogateTime | < 500ms |
| TST-012 | EavExactIndex bloom negative filter | 0 false positives on absent attrs |
| TST-013 | CSR-01 Sargon passport gate | Invalid passport rejected |
| TST-014 | CSR-07 Tribe isolation | Cross-tribe Client blocked |
| TST-015 | NĀRU journal integrity | verify_all() returns true after 1000 ops |

---

## Phase 2 — Scale Testing (Week 2)

### Particle Scale: 100 Million Particles

```
HeptaScript target: < 1 second for range queries with NatiruIndex + EavExactIndex
Memory budget: NatiruIndex at 100M particles × 64 entries × 8 B = 51.2 GB (cold)
Strategy: BUCKET_ORBITALS tuning + cold-tier sharding by surrogate range
```

### Deliverables
- [ ] NatiruIndex sharding across orbital buckets (surrogate range partitioning)
- [ ] EavExactIndex multi-shard merge for large attr spaces
- [ ] ŠUMU-UKIN AllParallel fan-out wired to real TcpStream connections
- [ ] ConEngine pool tested under 100M write load
- [ ] PAZUZU simulation run: 7 threat tests (PAZUZU-01 through PAZUZU-07)

### PAZUZU Gap Remediation
| Gap | Fix |
|-----|-----|
| PAZUZU-01 Sargon | Wire AkkadiSafeEngine (replace StubCredentialStore) |
| PAZUZU-02 Role | Enforce TabletWriter+ for all write paths |
| PAZUZU-03 NĀRU | Implement NĀRU-SYNC replication agent |
| PAZUZU-04 Pool | Add `max_connections` cap to ConnectionPool |
| PAZUZU-05 Opcode | Add opcode whitelist to `send_frame()` |
| PAZUZU-06 TIAMAT | Wire Engine 5 TIAMAT to ConEngine events |
| PAZUZU-07 App Gate | Add ConEngine gate to UTNAPISHTIM app registration |

---

## Phase 3 — Billion-Particle Production (Week 3-4)

### Particle Scale: 1 Billion Particles

```
Target: HeptaScript < 1 second on ALL 7 EnkiDB types for any query pattern
Architecture: cold-tier bucket grouping, surrogate sharding, ANCHOR E7First
```

### Pass Criteria for AI Agent Build Authorization
- [ ] All 50 compressed files processed correctly
- [ ] HeptaScript < 1 sec at 1B particles for: NODE, ACROSS, TIER, FILTER_ORDER queries
- [ ] All 7 CSR rules enforced under PAZUZU simulation
- [ ] NĀRU WAL journal: no integrity failures at 1B operations
- [ ] Zero data loss in append-only mode (orbital audit passes)

---

## Phase 4 — BeeMDM ETL (Post Phase 3)

### Pre-KAKI Structural Pipeline

```
Raw Archive (NARAMSIN Stage 0)
    ↓
DataStructure Station  — schema detection, column profiling
    ↓
DataCompare Station    — diff against reference tribe schema
    ↓
DataCleansing Station  — PII vault, null imputation, type coercion
    ↓
KAKI assigned — particle enters the post-KAKI tier-transition pipeline below
```

### Post-KAKI Tier-Transition Pipeline (real, tested — `eridu-runtime::SchedulerLoop`)

```
EnkiSDB (staged, Pending)
    ↓ ValidationSweep (every 900 ticks)
    ├── pass ─────────────────────────────→ EnkiODB (Active) → EnkiDW (retire) / EnkiDB (Golden Store)
    └── fail (Quarantined) → BlackBox Station scan
                                 ├── malware_flag=true → Storage Sector (terminal jail)
                                 └── malware_flag=false → EnkiQDB (fuzzy, pending Data Steward review)
                                                                ↓ Data Steward resolves
                                                     ├── clean → requeued into EnkiSDB (Pending)
                                                     └── confirmed harmful → Storage Sector
```

This replaces the previous "BlackBox Station = orbital trust probe / anomaly
detection" description, which conflated it with the unrelated
`orbital-trust-probe` crate (causal attribution for orbital drift). The real
BlackBox Station (`blackbox-station`) built and tested 2026-07-01 is the
Error Handling Station described above.

### Orbital Degradation States per Particle
```
GOLDEN_GEM  → GREEN_BRONZE → YELLOW_CLAY → ORANGE_RUST
                                                      ↓
                              RED_CRACKED ← DARK_ASH ← (repair attempt)
                                  ↓
                              DEAD_SEALED  (immutable, never erased)
```

---

## Phase 5 — AI Agents (Post BeeMDM ETL)

> Authorization Gate: HeptaScript must sustain < 1 sec at 1B particles first.

### AI Agent Stack

| Agent | Role | Crates Required | Status |
|-------|------|-----------------|--------|
| NINSUN (Healer / Progressive Refiner) | Advisory pattern/semantic anomaly detection, Stage 2b | `ninsun-agent`, `ninsun-steward-bridge` | Real, tested (2026-07-01). `explain::offline_explain` is a stub pending real inference wiring |
| DataSteward Agent | Review flagged particles, approve/reject | `data-steward-station`, `ninsun-steward-bridge`, `enkidullm-core` | Queues real and tested; the human/agent decision step itself is not yet automated |
| DataCleansing Agent | Auto-impute, type-coerce, PII detect | `data-cleansing-station`, `enkidullm-model` |
| DataCompare Agent | Schema drift detection, lineage tracing | `compare-tribe-schema`, `enkidullm-oracle` |
| DataStructure Agent | Column profiling, type inference | `data-structure-station`, `enkidullm-ingest` |
| BIGRING Sentinel | Auto-entry topology gating | `bahyway-algebra`, `ea-agent-algebra` |

### BIGRING
- Mathematical topology layer — no Tribe_KAKI required, automatic entry
- Governed by `bahyway-algebra` Betti numbers (β₀, β₁, β₂)
- No DataSteward approval needed for BIGRING-classified particles

---

## Sovereign Crate Registry (v4.0)

### Engine Tier
| Crate | Port | Function |
|-------|------|----------|
| `enkidb-con-engine` | — | Connection Engine, 7 CSR rules |
| `naramsin-archive` | — | Stage 0 archive decompression |
| `naramsin-format` | — | Stage 1 format parsing |
| `heptascript` | — | W5H2 query language engine |

### Database Tier
Corrected 2026-07-01 against the architect's clarification of each type's true
role — see `docs/00_codex/BAHYWAY_ECOSYSTEM_V4_GLOSSARY.md` §EnkiDB Types for full
detail on each type and what's real vs. mislabeled vs. not yet built.

| Crate | Port | Type | True Role (summary) |
|-------|------|------|------|
| `enkidb-engine` / `enkidb-persist` / `enkidb-storage` | 7001 | EnkiDB | Golden Store, final destination. Journal-replay today; real WAL + data files planned |
| `enkidb-dw` | 7002 | EnkiDW | Data warehouse — full ETL + analytics; receives retired EnkiODB particles |
| `enkidb-sdb` | 7003 | EnkiSDB | Stage/landing DB; `ValidationSweep` promotes or quarantines |
| `enkidb-odb` | 7004 | EnkiODB | Operational DB; state changes are new inserts, not mutations |
| `enkidb-qdb` | 7005 | EnkiQDB | Quarantine jail for **fuzzy/unknown particles only** (corrected 2026-07-01 — see BlackBox Station below) |
| `enkidb-quantdb` (mislabeled) | 7006 | EnkiMDB | **Not yet built.** True role: Service/App Metadata DB for TemplateEngine. `enkidb-quantdb` is a real, separate financial tick/OHLC store, unrelated to EnkiMDB's true purpose |
| `enkidb-recovery` (mislabeled) | 7007 | EnkiDDB | **Not yet built.** True role: Internal/client documentation DB backing a MetaEngine AI agent. `enkidb-recovery` is real, separate crash-recovery logic, unrelated to EnkiDDB's true purpose |

### Post-KAKI Tier-Transition Stations (not numbered ports)
| Crate | Function |
|-------|----------|
| `blackbox-station` | Scans EnkiSDB's Quarantined particles; routes by `malware_flag` to Storage Sector (harmful) or EnkiQDB (fuzzy) |
| `storage-sector` | Hardware-isolated, terminal jail for BlackBox-confirmed-harmful particles |
| `data-steward-station` | `QuarantineReviewQueue` — reviews EnkiQDB's fuzzy backlog; resolves clean (→ requeue to EnkiSDB) or confirms harmful (→ Storage Sector) |
| `ninsun-steward-bridge` | `NinsunAdvisoryQueue` — Stage 2b advisory inbox for NINSUN's pattern/semantic anomaly proposals; Steward confirms/rejects |

### Index Tier
| Crate | Index Type |
|-------|-----------|
| `enkidb-indexes` | NatiruIndex (orbital-bucket) + EavExactIndex (Xor8+binary) |

### Protocol Tier
| Crate | Function |
|-------|----------|
| `enkidu-protocol` | HEPT wire protocol, SPSC ring, buffer pool, frame codec |
| `enkidb-session-registry` | Sync session registry, native TOML parser |

### Security Tier
| Crate | Function |
|-------|----------|
| `hepta-sec-firewall` | Packet-level firewall rules |
| `hepta-sec-policy` | SovereignRole policy enforcement |
| `hepta-sec-sentinel` | PAZUZU threat detection |
| `musaru-security` | Credential vault |

---

## §0.3 Sovereign Constraints — Never Negotiable

1. `#![forbid(unsafe_code)]` on every crate
2. No external crates: no serde, tokio, thiserror, blake3, tracing, flate2, rocksdb
3. KAKI is immutable — generated once, never changed, never refused by new releases
4. Append-only — no DELETE, no UPDATE that erases history
5. Every write journalled to NĀRU WAL before acknowledgement
6. All 7 CSR rules applied on every ConEngine operation
7. ŠUMU-UKIN routing required for any cross-tribe query

---

## Timeline Summary

| Phase | Duration | Gate |
|-------|----------|------|
| Phase 1 — 10M test | Day 1 (Tomorrow) | 50 files clean, <1 sec |
| Phase 2 — 100M test | Week 2 | PAZUZU simulation pass |
| Phase 3 — 1B test | Week 3-4 | All EnkiDB types <1 sec |
| Phase 4 — BeeMDM ETL | Month 2 | 6 stations operational |
| Phase 5 — AI Agents | Month 3+ | HeptaScript gate passed |
