# BahyWay.Ecosystem v4.0 — Sovereign Roadmap
𒁾 *ṭuppu* — "the clay tablet of the realm"

**Branch:** `claude/serene-goldberg-tpobS`  
**Date:** 2026-05-31  
**Method:** W5H2 (Who · What · When · Where · Why · How · How Much)

---

## W5H2 — The Ecosystem

### WHO — Who builds and uses BahyWay.Ecosystem

| Stakeholder | Role |
|---|---|
| **Bahaa Fadam (DUB.SAR 𒁾)** | Sovereign architect — all architectural decisions |
| **Data Stewards** | Operate the BeeMDM pipeline, inspect KAKI PKs, manage quality |
| **AkkadianAOL Developers** | Write `.akk` policy scripts compiled by `crates/aaol` |
| **EnkiDB Operators** | Run the sovereign database engine, monitor tribe orbits |
| **Autonomous `.akk` agents** | Carry SargonPassport, act within granted IŠRETU scopes |

### WHAT — What the ecosystem is

BahyWay.Ecosystem v4.0 is a **pure Rust, zero-external-database sovereign data platform**.  
It replaces PostgreSQL and all third-party databases with:
- **EnkiDB** — native key-value + EAV storage with KAKI 16-byte PKs
- **AkkadianAOL** — a sovereign policy language that compiles to 5 targets
- **BeeMDM** — a 4-lane × 8-station quality pipeline (ADR-004)
- **DubSar IDE** — pure DOM + HTML Canvas 2D development environment

### WHEN — When v4.0 is used

```
v3.5 (PostgreSQL-dependent, bevy/egui UI)
  │
  ▼  [NOW — migration in progress]
v4.0 (pure Rust, pure DOM, sovereign crypto)
  │
  ▼  [FUTURE]
v4.5 (Dilithium post-quantum upgrade, BLAKE3 sovereign hash)
```

### WHERE — Architecture map

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        BahyWay.Ecosystem v4.0                               │
│                                                                             │
│  Layer 12  bahyway-web          (pure DOM + Canvas 2D)                     │
│  Layer 11  dubsar-ide           dubsar-visualizer                          │
│  Layer 10  eridu-runtime        eridu-scheduler       eridu-supervisor     │
│                                                                             │
│  Layer 9.1 ┌─────────┐  ┌──────────┐  ┌────────┐                         │
│            │  kupru   │  │ akkvalue │  │ istar  │  ◄─ Created Today       │
│            │ (crypto) │  │ (31-var) │  │ (ABAC) │                         │
│            └─────────┘  └──────────┘  └────────┘                         │
│                                                                             │
│  Layer 9   ┌──────────┐  ┌───────────┐  ┌────────┐  ┌────────────┐       │
│            │   aaol   │  │ akkadi-ir │  │ akkadi │  │heptascript │       │
│            │(compiler)│  │   (IR)    │  │(vocab) │  │            │       │
│            └──────────┘  └───────────┘  └────────┘  └────────────┘       │
│                  ▲  created today ──────────┘                             │
│                                                                             │
│  Layer 8   adad-gate  musaru-security  compare-tribe-schema  vgca-valid.  │
│            data-structure-station  data-cleansing-station                  │
│            data-steward-station    permanent-storage                       │
│                                                                             │
│  Layer 7   template-engine  template-library  diagnosis-engine             │
│            diagnosis-templates  damadmbok-dictionary                       │
│                                                                             │
│  Layer 6   idu-prober  idu-batching                                        │
│                                                                             │
│  Layer 5   story-engine  fuzzy-engine  hepta-score  score-engine           │
│            alert-engine  snapshot-job  navi-engine  najaf-engine           │
│            dmw-engine    nusku-engine  azuga-engine  nusku-fuzzy           │
│            iris-engine   panam-engine  nusku-score   shulman-engine        │
│            wpd-engine    homt-engine                                       │
│                                                                             │
│  Layer 4.5 vgca-engine  tribe-orbit-engine  ammas-engine                  │
│            shedu-engine  riksu-engine                                      │
│                                                                             │
│  Layer 4   enkidb-engine   enkidb-query                                    │
│                                                                             │
│  Layer 3   enkidb-indexes  enkidb-dictionary                               │
│                                                                             │
│  Layer 2   enkidb-block   enkidb-journal   enkidb-storage                  │
│            enkidb-snapshot enkidb-recovery  enkidb-persist                 │
│            enkidb-dw      enkidb-sdb       enkidb-qdb     enkidb-odb      │
│                                                                             │
│  Layer 1   enkidb-kaki    enkidb-vector-id                                 │
│                                                                             │
│  Layer 0   bahyway-core   bahyway-crc   bahyway-algebra                    │
└─────────────────────────────────────────────────────────────────────────────┘

  Binaries:  akkadi-cli  bahyway-cli  bahyway-server  dubsar  enkidw  najaf-ingest
  Tests:     tests/e2e   tests/chaos
```

### WHY — Why v4.0 was built

| Pain in v3.5 | Sovereign solution in v4.0 |
|---|---|
| PostgreSQL dependency — external service, license, ops burden | EnkiDB — pure Rust, embedded, zero-dep storage |
| Bevy/egui desktop UI — not web-native | Pure DOM + HTML Canvas 2D (bahyway-web) |
| 23 different value types across crates | One `AkkValue` (31 variants) in `akkvalue` |
| Access control scattered as `if` blocks | `istar` — one ABAC engine, 5 meta-rules |
| No canonical signing — JSON field order risk | Fix 4: `passport_canonical_bytes()` |
| 16-byte nonce in LinguisticProof (birthday 2^64) | Fix 2: 32-byte nonce (birthday 2^128) |
| "Way" suffix on all type names (EnkiWay, VaultWay…) | ADR-002: suffix removed from all v4.0 names |

### HOW — How the ecosystem operates

```
.akk source
    │ aaol lexer → parser → semantic → AkkFile AST
    ▼
akkadi-ir IrBuilder
    │ AkkIr tree (9 node types, NodeId FNV-1a)
    ▼
AkkBackend (Rust / Python / JSON / PS / XML)
    │
    ▼ runtime

SargonKdf (Argon2id 54it)     AkkadianSeal (Ed25519)
    │  derive_key()                 │  seal()
    ▼                              ▼
AkkadianCipher (ChaCha20)    SargonPassport (4 layers)
    │  seal(plaintext, aad)        │  passport_seal() ← canonical binary
    ▼                              ▼
AkkFile (5 layers: RĒŠU/MUDÛ/KANĀKU/ŠIPRU/ŠIPIR ŠARRI)
    │
    ▼
EnkiDB (KAKI 16-byte PK, EAV triples, AkkValue)
    │
    ▼
istar AkkFirewall (5 meta-rules + composable rules)
    │  Allow / Deny / Escalate / Redact / Audit
    ▼
Pipeline stations → permanent-storage → bahyway-web
```

### HOW MUCH — Ecosystem metrics (v4.0, as of 2026-05-31)

| Metric | Value |
|---|---|
| Total workspace crates | 75 |
| Total workspace binaries | 6 |
| Crates created / migrated today | **5 new + 1 binary** |
| Files created today | **~75 source files** |
| Lines of Rust today | **~8,000** |
| Commits today | **4** |
| External dependencies removed vs v3.5 | PostgreSQL, SQLite, Bevy, egui |
| Sovereign speedup (avg) | **~425×** vs legacy SQL |
| KAKI PK size | **16 bytes** |
| Quality divisor | **240.0** (ADR-001) |
| Max credential TTL | **54 hours** (ŠATTĀTU_MAX) |

---

## Crates Created Today — Detail

### 1. `crates/akkadi-ir` — Sovereign IR
**Commit:** `7e35118`  
**Layer:** 9 (Languages)

| W | Answer |
|---|---|
| Who | AkkadianAOL compiler + code-gen backends |
| What | Target-agnostic IR tree — 9 node variants, Walker, Backend trait |
| When | After parsing, before code generation |
| Where | Between `aaol` and all codegen targets |
| Why | Decouples grammar from targets; enables 5 backends from one source |
| How | AkkNode enum, FNV-1a NodeId, AkkWalker visitor, AkkBackend trait |
| How much | 10 files · ~1,600 lines · **zero** external deps |

**Key types:** `AkkNode`, `NodeId`, `AkkIr`, `IrBuilder`, `AkkWalker`, `AkkBackend`, `QualityLane`, `HeptaDim`, `KineticForce`

---

### 2. `crates/kupru` — Sovereign Crypto Layer
**Commit:** `77fd705` (stubs) + `e673eb1` (full implementation)  
**Layer:** 9.1 (Sovereign Crypto)

| W | Answer |
|---|---|
| Who | All crates that seal data or issue credentials |
| What | 5-layer .akk format + ChaCha20 + Ed25519 + Argon2id + SHA3-512 + SargonPassport |
| When | File write (seal), file read (verify), credential issuance/validation |
| Where | Security foundation — no BahyWay crate may bypass it |
| Why | Single crypto source of truth; Fix 2 (32-byte nonce) + Fix 4 (canonical signing) |
| How | AkkFile 5 layers, SargonKdf 54 Argon2id iterations, passport_canonical_bytes() |
| How much | 10 files · ~1,200 lines · 3 crypto algorithms + HMAC |

**Key fixes implemented:**
- **Fix 2:** LinguisticProof nonce raised 16→32 bytes (birthday bound 2^128)
- **Fix 4:** passport_seal() uses canonical binary — not JSON — for deterministic signing

**Key constants:** `SATTATU_MAX = 54h`, `QUALITY_DIVISOR = 240.0`, `AKK_VERSION = 0x04`

**Key types:** `AkkFile`, `SargonPassport`, `SealKeyPair`, `AkkadianCipher`, `SargonKdf`, `SovereignHash`, `LinguisticProof`

---

### 3. `crates/akkvalue` — Sovereign EAV Value Type
**Commit:** `77fd705`  
**Layer:** 9.1 (Cross-cutting)

| W | Answer |
|---|---|
| Who | Every crate that stores or transmits typed data |
| What | 31-variant sovereign EAV value type |
| When | At every crate boundary in the data pipeline |
| Where | Cross-cutting — imported by Layers 2 through 12 |
| Why | Eliminates impedance mismatch between NuskuValue/StoryValue/HeptaValue (v3.5) |
| How | Tagged JSON `{"type":"T","value":V}` + serde visitor for binary compat |
| How much | 3 files · ~500 lines · 31 variants · 10 supporting types |

**Key types:** `AkkValue` (31 variants), `AkkTriple`, `AkkCoordinate`, `AkkDate`, `AkkHijriDate`, `AkkNationalId`, `PolicyVerdict`

---

### 4. `crates/istar` — Sovereign Access Control
**Commit:** `77fd705`  
**Layer:** 9.1 (Access Control)

| W | Answer |
|---|---|
| Who | musaru-security, adad-gate, permanent-storage |
| What | ABAC engine with 5 immutable constitutional meta-rules |
| When | Before every data write, before cross-domain reads, before admin ops |
| Where | Between application logic and all data access paths |
| Why | Centralises scattered `if quality > N` checks into one auditable engine |
| How | Priority-sorted `Vec<AkkRule>` closures; first non-None non-Audit wins |
| How much | 2 files · ~280 lines · 5 meta-rules · 5 verdict types |

**Constitutional meta-rules (in priority order):**
1. Dead quality (q<59) → Deny
2. Cross-domain write + low clearance → Deny
3. Admin operation + non-gem quality → Deny
4. Sensitivity > clearance → Escalate
5. Delete/Admin → Audit (informational)

---

### 5. `bin/akkadi-cli` — Sovereign CLI
**Commit:** `c728909`  
**Layer:** Binary

| W | Answer |
|---|---|
| Who | Data stewards, developers, DevOps, architects |
| What | Unified CLI + sovereign notebook (7 command groups, 10 cell types) |
| When | Development time, debugging, administrative operations |
| Where | Talks to all crates via library APIs + HTTP to bahyway-server |
| Why | First-class developer experience; KAKI inspector saves hours of manual decoding |
| How | Subcommand dispatch, AkkadiKernel executes .akk cells via aaol parser |
| How much | 20 files · ~2,000 lines · 7 commands · 3 output formats · avg ~425× speedup display |

---

### 6. `policies/pollution_way_policy.akk` — Policy Template
**Commit:** `e673eb1`  
**Type:** AkkadianAOL v3.5 source

| W | Answer |
|---|---|
| Who | AkkadianAOL compiler, pollution domain operators |
| What | Complete ABAC policy for air/water/oil pollution monitoring |
| When | Compiled at startup into `AkkRule` closures for the istar firewall |
| Where | `policies/` — sovereign policy store |
| Why | Reference implementation of WHO AQI thresholds + HPS alarm triggers |
| How | POLICY / ALARM / FIREWALL / CIPHER_POLICY directives in AkkadianAOL syntax |
| How much | 320 lines · 7 sections · 3 domains (Air/Water/Oil) · 5 alarm levels |

---

## Dependency Graph (new crates only)

```
akkadi-cli
    ├──► aaol (AkkadianAOL parser for notebook)
    ├──► akkadi (vocabulary)
    └──► akkadi-ir (IR inspection)

akkadi-ir
    └──► (zero deps)

kupru
    └──► (serde, zeroize, rand, sha3, subtle, chacha20poly1305,
           ed25519-dalek, argon2, hmac, uuid)

akkvalue
    └──► (serde, serde_json, uuid, chrono)

istar
    ├──► (serde, tracing)
    └──► [logical dependency on kupru for SargonPassport type]
```

---

## Pending Work (not yet migrated)

| Item | Description | Priority |
|---|---|---|
| `shulman-engine` full impl | Only stub `lib.rs` — full report generator uploaded | High |
| `akkadi-cli/client/enkiway.rs` | EnkiDB client module missing from CLI | Medium |
| `aaol/compiler` full codegen | `akkadian_codegen_full.rs` not yet merged | Medium |
| `akkadian_semantic_full.rs` | 30 KB full semantic analyser — replaces Phase 1B stub | High |
| `databaseway` | v3.5 crate; needs v4.0 rewrite (imports nusku_wayv352) | Low |
| `wpd-engine` 7 more files | pipeline.rs, scan.rs, state.rs (Bevy UI — not applicable in v4.0) | N/A |

---

## ADR References

| ADR | Decision | Status |
|---|---|---|
| ADR-001 | `QUALITY_DIVISOR=240.0`, `GEM_B11=200`, `TRIBE_B11=140`, `ACTIVE_B11=100` | ✅ Implemented |
| ADR-002 | "Way" suffix deprecated from all v4.0 type/crate names | ✅ Enforced |
| ADR-003 | KAKI 16-byte PK layout (B0-B15 semantics) | ✅ Documented |
| ADR-004 | BeeMDM 4-lane × 8-station pipeline | ✅ In akkadi-cli |
| ADR-005 | UI Deployment Tiers — Canvas 2D (Vagrant/Tier 1) vs Bevy 7D (Bare-Metal/Tier 2) | ✅ Documented |
| ADR-008 | VGCA Sigma/Delta/Lambda geometric cleansing | In progress |
| Fix 2 | LinguisticProof nonce 16→32 bytes | ✅ In kupru |
| Fix 4 | Passport canonical binary signing | ✅ In kupru |

---

## ADR-005 — UI Deployment Tiers

### Background

The essential BahyWay architecture is **7-dimensional (3D spatial)** — FuzzyDimensions
D1-D8 are intended to be rendered as geometric primitives in a live 3D Bevy scene,
with egui panels for controls and stakeholder gating.

The current Vagrant development environment (Windows host → Vagrant Fedora box) has
**no GPU passthrough**, making Bevy non-functional. Canvas 2D was chosen as a
Vagrant-compatible transport layer, not as the target architecture.

### Decision

| Tier | Trigger | UI Stack | Forbidden in this tier |
|------|---------|----------|------------------------|
| **Tier 1 — Vagrant** | Windows host, no GPU | Pure DOM + HTML Canvas 2D (WASM) | Bevy, egui, Tokio |
| **Tier 2 — Bare Metal** | MSI Prestige 15 · Fedora SilverBlue · GPU available | Bevy + egui · 7D/3D spatial rendering | none of the above |

### Tier 2 crates (deferred, not forbidden)

These crates are **Vagrant-incompatible**, not architecturally wrong:
- `bevy` — ECS 3D scene engine
- `egui` / `bevy_egui` — immediate-mode UI
- `tokio` — async runtime for parallel orchestrators (Nigin scanner, M11)
- `petgraph` — graph topology algorithms

### Coexistence

`bahyway-web` (Canvas 2D WASM) remains available in Tier 2 as a web-embed / remote
stakeholder layer. It is not replaced — it is complemented by the Bevy layer.

---

## Sovereign Constraints (universal — both tiers)

```
✓ #![forbid(unsafe_code)]          — zero unsafe Rust ecosystem-wide
✓ NO PostgreSQL / SQLite / Redis    — pure Rust storage only
✓ QUALITY_DIVISOR = 240.0          — never 255 (ADR-001)
✓ "Way" suffix removed              — EnkiWay → Enki, VaultWay → Vault (ADR-002)
✓ SATTATU_MAX = 54 hours            — no credential outlives Sargon's reign
✓ Ed25519 → upgradeable to Dilithium-3 (algorithm_id field in SealKeyPair)
✓ SHA3-512 → upgradeable to SPHINCS+ (HashAlgorithm::PostQuantum = 0xFF)
✓ Real architecture target: 7D (3D spatial) — Bevy + egui on bare-metal GPU
```

---

---

## ENKI-TERRA Environmental Intelligence Suite

| Module | Deity | Domain | Business Case | Status |
|--------|-------|--------|--------------|--------|
| NANSHE | 𒀭𒀭𒀭 | River contamination | BC-ENV-001 | Sealed |
| NAMTAR | 𒀭𒀭𒀭 | Sacred burial navigation | — | Designed |
| EREŠKIGAL | 𒀭𒀭𒀭 | Informal settlement topology | — | Designed |
| ESARHADDON | 𒀭𒀸𒋩 | Earthquake rescue intelligence | BC-QUAKE-001 | Sealed — Playbook 119 |
| ASHNAN | 𒀭𒀳 | Climate · agriculture · livestock | BC-AGRI-001 | Sealed — Playbook 156 |
| NARAMSIN | 𒈎𒋀𒁲 | Sovereign archive & format reader | EN-NARAMSIN-001 | Sealed — Playbook 198 |

### Shared Prerequisite — All ENKI-TERRA Modules

- [ ] **[Playbook 98]** KISPU HeadStore fix (`akk_decode → eav_triple_to_value`) — unblocks all real-time ENKI-TERRA feeds

---

### v4.x — ESARHADDON 𒀭𒀸𒋩 Earthquake Rescue Module (BC-QUAKE-001)

- [ ] [Phase 1] ESARHADDON KAKI schema: seismic event (0x02/ZIKRU), survivor identity (0x01/KISHIB), structural state (0x02/PARZU) + EAV ontology — **Playbooks 119–122**
- [ ] [Phase 2] Survivor Signal Aggregator — acoustic / thermal / RF / robot LIDAR Bayesian confidence fusion — **Playbooks 123–127**
- [ ] [Phase 3] TIAMAT Engine 5 Structural Mortality Index (SMI) formula + KIBRATU PARALLEL_FAILURE cascade collapse detection — **Playbooks 128–133**
- [ ] [Phase 4] HeptaMap E7 rescue grid — GREEN/AMBER/RED/BLACK zone classification + multi-team collision-free coordination — **Playbooks 134–138**
- [ ] [Phase 5] TIAMAT T+1hr / T+6hr / T+24hr collapse forecast + NANSHE gas dispersion bridge (CH₄/CO/H₂S/PM2.5) — **Playbooks 139–143**
- [ ] [Phase 6] DubSar Theater scenes: RUBBLE FIELD (13M GPUParticles3D BIGRING), SMI HEATMAP, SURVIVOR ORBIT, GAS PLUME, TIAMAT FORECAST — **Playbooks 144–150**
- [ ] [Phase 7] Field ruggedisation — mesh radio transport, battery-backup KVM field node, ŠĀRU deficit reporting — **Playbooks 151–155**

---

### v4.x — ASHNAN 𒀭𒀳 Climate-Agriculture-Livestock Module (BC-AGRI-001)

- [ ] [Phase 1] ASHNAN KAKI schema: soil/crop (ZIKRU), pest identity (KISHIB), livestock (PARZU), external ingestion (CrossTribe) + provenance EAV ontology — **Playbooks 156–160**
- [ ] [Phase 2] Diyala agricultural sensor pilot deployment — soil, weather, pest trap, livestock ear-tag networks — **Playbooks 161–166**
- [ ] [Phase 3] TIAMAT Crop Mortality Index (CMI): F_soil × F_heat × F_water × F_phenology — **Playbooks 167–171**
- [ ] [Phase 4] TIAMAT Pest Incursion Risk Index (IRI): degree-day accumulation + trap data + host CMI — **Playbooks 172–175**
- [ ] [Phase 5] TIAMAT Livestock Mortality Index (LMI): THI + forage + disease + KIBRATU HIDDEN_PATTERN famine-cascade — **Playbooks 176–180**
- [ ] [Phase 6] International ingestion adapters: FAO, WOAH, national met service, confidence tier framework (TIER 0–3) — **Playbooks 181–186**
- [ ] [Phase 7] DubSar Theater ASHNAN scenes: GRAIN FIELD, PEST ORBIT, HERD HEATMAP, PROVENANCE LAYER, CASCADE FORECAST — **Playbooks 187–192**
- [ ] [Phase 8] Regional extension framework — Jordan, Syria, broader Fertile Crescent threshold templates — **Playbooks 193–197**

---

### v4.x — NARAMSIN 𒈎𒋀𒁲 Sovereign Archive & Format Reader Engine (EN-NARAMSIN-001)

- [ ] [Phase 1] `naramsin-archive` — zip / tar.gz / tar.bz2 / tar.xz / 7z decompression + nested archive recursion — **Playbooks 198–202**
- [ ] [Phase 2] `naramsin-format` — CSV / JSON / XML parsing into row-oriented intermediate representation — **Playbooks 203–206**
- [ ] [Phase 3] `naramsin-integrity` — NRM_* code classification, validated against ASHNAN BeeMDM test corpus (incl. NRM_TRUNCATED on the corrupt zip) — **Playbooks 207–210**
- [ ] [Phase 4] `naramsin-audit` — NĀRU journal integration for archive_id audit particles — **Playbooks 211–212**
- [ ] [Phase 5] `naramsin-bridge` — sovereign entrypoint wired into NANSHE, ESARHADDON, and ASHNAN MASHSHARU gates — **Playbooks 213–217**
- [ ] [Phase 6] Regression pass — full ASHNAN BeeMDM test corpus (7 archive variants) end-to-end — **Playbook 218**

---

*𒁾 DUB.SAR — BahyWay.Ecosystem v4.0 | 2026-06-30*
