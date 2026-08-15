# BahyWay v4 — Caller-Callee Reference
## Service / Library Dependency Hierarchy

> Branch: `claude/serene-goldberg-tpobS` on `bahyway/EnkiDB`
> Last updated: 2026-06-02
>
> **How to use this document**
> When you see an error message from crate X, trace its callees downward to find
> which lower-layer library produced it.  When you need to know who calls crate X,
> scan upward to find which binaries / callers depend on it.

---

## 1. Layer Map (top → bottom = caller → callee)

```
Layer 12  ┌──────────────────────────────────────────────────────────────────┐
          │  BINARIES                                                         │
          │  bee-watchdog  bahyway-api  bahyway-web  bahyway-server           │
          │  bahyway-cli   akkadi-cli   najaf-ingest  dubsar  enkidw          │
          └──────────────────────────────────────────────────────────────────┘
                 │                          │
Layer 11  ┌─────▼────────────────────────────────────────────────────────────┐
          │  UI / IDE                                                         │
          │  dubsar-ide        dubsar-visualizer   eridu-supervisor           │
          └──────────────────────────────────────────────────────────────────┘
                 │
Layer 10  ┌─────▼────────────────────────────────────────────────────────────┐
          │  RUNTIME / OS                                                     │
          │  eridu-runtime     eridu-scheduler                                │
          └──────────────────────────────────────────────────────────────────┘
                 │
Layer 9.5 ┌─────▼────────────────────────────────────────────────────────────┐
          │  LLM / EMBED                                                      │
          │  enkidullm-core    enkidullm-ingest                               │
          │  zikru-embed       enkidullm-audit                                │
          └──────────────────────────────────────────────────────────────────┘
                 │
Layer 9.1 ┌─────▼────────────────────────────────────────────────────────────┐
          │  SOVEREIGN PRIMITIVES  (leaf nodes — zero workspace deps)         │
          │  kupru (crypto)    akkvalue (31-var EAV)    istar (ABAC)          │
          └──────────────────────────────────────────────────────────────────┘

Layer 9   ┌──────────────────────────────────────────────────────────────────┐
          │  LANGUAGES                                                        │
          │  aaol   akkadi   akkadi-ir   heptascript   hepta                  │
          │  dfg-engine   hdf-bridge   ezida-ir                               │
          └──────────────────────────────────────────────────────────────────┘
                 │
Layer 8   ┌─────▼────────────────────────────────────────────────────────────┐
          │  PIPELINE / STATIONS                                              │
          │  musaru-security       adad-gate           compare-tribe-schema   │
          │  data-structure-station  data-cleansing-station                   │
          │  data-steward-station  permanent-storage   client-dq-profile      │
          │  vgca-validation                                                  │
          └──────────────────────────────────────────────────────────────────┘
                 │
Layer 7   ┌─────▼────────────────────────────────────────────────────────────┐
          │  TEMPLATES / GOVERNANCE                                           │
          │  template-engine   template-library   damadmbok-dictionary        │
          │  diagnosis-engine  diagnosis-templates                            │
          └──────────────────────────────────────────────────────────────────┘
                 │
Layer 6   ┌─────▼────────────────────────────────────────────────────────────┐
          │  IDU / BATCHING                                                   │
          │  idu-prober    idu-batching                                       │
          └──────────────────────────────────────────────────────────────────┘
                 │
Layer 5   ┌─────▼────────────────────────────────────────────────────────────┐
          │  ENGINES (scoring, story, alert, nav, nusku cluster)              │
          │  story-engine   score-engine   fuzzy-engine   alert-engine        │
          │  navi-engine    najaf-engine   hepta-score    snapshot-job        │
          │  dmw-engine     kinetic-engine pollution-engine                   │
          │  nusku-engine → azuga / iris / panam / wpd / shulman / nusku-score│
          └──────────────────────────────────────────────────────────────────┘
                 │
Layer 4.5 ┌─────▼────────────────────────────────────────────────────────────┐
          │  PHYSICS / INTELLIGENCE                                           │
          │  vgca-engine   tribe-orbit-engine   ammas-engine                  │
          │  shedu-engine  riksu-engine  vault-engine                         │
          └──────────────────────────────────────────────────────────────────┘
                 │
Layer 4   ┌─────▼────────────────────────────────────────────────────────────┐
          │  ENKIDB ENGINE + QUERY                                            │
          │  enkidb-engine    enkidb-query    enkidb-raft    enkidb-ingest     │
          └──────────────────────────────────────────────────────────────────┘
                 │
Layer 3   ┌─────▼────────────────────────────────────────────────────────────┐
          │  INDEXES + DICTIONARY                                             │
          │  enkidb-indexes   enkidb-dictionary                               │
          └──────────────────────────────────────────────────────────────────┘
                 │
Layer 2   ┌─────▼────────────────────────────────────────────────────────────┐
          │  STORAGE SUBSTRATE                                                │
          │  enkidb-block    enkidb-journal    enkidb-storage                 │
          │  enkidb-snapshot enkidb-recovery   enkidb-persist                 │
          │  enkidb-dw       enkidb-sdb        enkidb-qdb     enkidb-odb      │
          └──────────────────────────────────────────────────────────────────┘
                 │
Layer 1   ┌─────▼────────────────────────────────────────────────────────────┐
          │  KAKI / IDENTITY                                                  │
          │  enkidb-kaki    enkidb-vector-id    enkidb-particles              │
          └──────────────────────────────────────────────────────────────────┘
                 │
Layer 0   ┌─────▼────────────────────────────────────────────────────────────┐
          │  FOUNDATION  (leaf nodes — zero workspace deps)                   │
          │  bahyway-core    bahyway-crc    bahyway-algebra                   │
          └──────────────────────────────────────────────────────────────────┘
```

---

## 2. Full Caller → Callee Table

Each row: `CALLER` → callees it directly imports.

### Binaries

| Caller | Direct Callees (workspace crates only) |
|--------|----------------------------------------|
| **bee-watchdog** | alert-engine · bahyway-core · client-dq-profile · data-steward-station · data-structure-station · enkidb-dw · enkidb-engine · enkidb-journal · enkidb-kaki · enkidb-persist · enkidb-storage · musaru-security · permanent-storage · score-engine · story-engine · template-engine |
| **bahyway-api** | bahyway-core |
| **bahyway-web** | bahyway-core · enkidb-journal · enkidb-kaki · enkidb-odb · enkidb-qdb · enkidb-sdb · eridu-runtime |
| **bahyway-server** | bahyway-core · enkidb-engine · eridu-runtime · eridu-scheduler · eridu-supervisor · snapshot-job |
| **bahyway-cli** | aaol · bahyway-core · enkidb-engine · enkidb-query · heptascript |
| **akkadi-cli** | aaol · akkadi |
| **najaf-ingest** | adad-gate · bahyway-core · enkidb-engine · enkidb-kaki · enkidb-persist · enkidb-storage · musaru-security · permanent-storage · story-engine · template-library · vgca-validation |
| **dubsar** | aaol · adad-gate · bahyway-core · damadmbok-dictionary · dubsar-ide · dubsar-visualizer · enkidb-engine · enkidb-kaki · enkidb-persist · enkidb-query · enkidb-storage · heptascript · musaru-security · permanent-storage · score-engine · story-engine · template-engine · template-library · vgca-validation |
| **enkidw** | bahyway-core · enkidb-dw · enkidb-persist · enkidb-storage · story-engine |

---

### Layer 11 — UI / IDE

| Caller | Direct Callees |
|--------|----------------|
| dubsar-ide | aaol · bahyway-core · heptascript · template-engine |
| dubsar-visualizer | bahyway-core · enkidb-journal · enkidb-kaki · enkidb-snapshot · najaf-engine · score-engine · story-engine · wpd-engine |
| eridu-supervisor | bahyway-core · eridu-runtime · eridu-scheduler |

---

### Layer 10 — Runtime

| Caller | Direct Callees |
|--------|----------------|
| eridu-runtime | bahyway-core · enkidb-engine · enkidb-journal · enkidb-kaki · enkidb-odb · enkidb-qdb · enkidb-sdb · eridu-scheduler |
| eridu-scheduler | bahyway-core |

---

### Layer 9.5 — EnkiduLLM

| Caller | Direct Callees |
|--------|----------------|
| enkidullm-core | bahyway-core · bahyway-crc · enkidb-kaki |
| enkidullm-ingest | bahyway-crc · enkidullm-core |
| zikru-embed | bahyway-core · enkidb-kaki · enkidullm-core |
| enkidullm-audit | bahyway-core · bahyway-crc · enkidb-kaki · enkidullm-core |

---

### Layer 9 — Languages

| Caller | Direct Callees |
|--------|----------------|
| aaol | bahyway-core |
| heptascript | akkvalue · bahyway-core · bahyway-crc · enkidb-journal · enkidb-kaki · navi-engine |
| hdf-bridge | dfg-engine · hepta |
| hepta | *(none)* |
| akkadi | *(none)* |
| akkadi-ir | *(none)* |
| dfg-engine | *(none)* |
| ezida-ir | *(none)* |

---

### Layer 9.1 — Sovereign Primitives (leaf nodes)

| Crate | Callees | Notes |
|-------|---------|-------|
| kupru | *(none)* | Crypto: ChaCha20 / Ed25519 / Argon2id / SHA3-512 |
| akkvalue | *(none)* | 31-variant EAV value type |
| istar | *(none)* | ABAC firewall, 5 meta-rules |

---

### Layer 8 — Pipeline / Stations

| Caller | Direct Callees |
|--------|----------------|
| musaru-security | bahyway-core · enkidb-journal · enkidb-kaki |
| adad-gate | bahyway-core · enkidb-engine · enkidb-journal · enkidb-kaki |
| compare-tribe-schema | bahyway-core · template-engine |
| vgca-validation | bahyway-core · enkidb-journal · enkidb-kaki · template-engine |
| data-structure-station | bahyway-core · enkidb-journal · template-engine |
| data-cleansing-station | bahyway-core · damadmbok-dictionary · enkidb-journal · template-engine |
| data-steward-station | alert-engine · bahyway-core · diagnosis-templates |
| permanent-storage | bahyway-core · enkidb-engine · enkidb-journal · enkidb-kaki · story-engine |
| client-dq-profile | bahyway-core · enkidb-journal · fuzzy-engine · score-engine |

---

### Layer 7 — Templates / Governance

| Caller | Direct Callees |
|--------|----------------|
| template-engine | bahyway-core · enkidb-kaki |
| template-library | bahyway-core · template-engine |
| damadmbok-dictionary | bahyway-core · template-engine |
| diagnosis-templates | alert-engine · bahyway-core · template-engine |
| diagnosis-engine | alert-engine · bahyway-core · diagnosis-templates · enkidb-journal · enkidb-kaki · story-engine |

---

### Layer 6 — IDU / Batching

| Caller | Direct Callees |
|--------|----------------|
| idu-prober | bahyway-core · enkidb-journal · enkidb-kaki · enkidb-snapshot · story-engine |
| idu-batching | bahyway-core · enkidb-journal · enkidb-kaki · enkidb-snapshot · idu-prober · story-engine |

---

### Layer 5 — Engines

| Caller | Direct Callees |
|--------|----------------|
| story-engine | bahyway-core · enkidb-indexes · enkidb-journal · enkidb-kaki · enkidb-snapshot |
| fuzzy-engine | bahyway-core |
| score-engine | bahyway-core · enkidb-journal · enkidb-kaki · fuzzy-engine · story-engine |
| alert-engine | bahyway-core · enkidb-indexes · enkidb-kaki · score-engine |
| snapshot-job | bahyway-core · enkidb-journal · enkidb-kaki · enkidb-snapshot · enkidb-vector-id · story-engine |
| navi-engine | bahyway-core · enkidb-kaki |
| najaf-engine | bahyway-core · enkidb-kaki · navi-engine |
| dmw-engine | bahyway-core |
| nusku-engine | *(none)* |
| azuga-engine | nusku-engine |
| iris-engine | nusku-engine |
| panam-engine | nusku-engine |
| wpd-engine | nusku-engine |
| nusku-score | nusku-engine |
| shulman-engine | nusku-engine · nusku-score |
| hepta-score | *(none)* |
| kinetic-engine | *(none)* |
| pollution-engine | *(none)* |

---

### Layer 4 — EnkiDB Engine + Query

| Caller | Direct Callees |
|--------|----------------|
| enkidb-engine | bahyway-core · enkidb-indexes · enkidb-journal · enkidb-kaki · enkidb-recovery · enkidb-snapshot · story-engine |
| enkidb-query | akkvalue · bahyway-core · bahyway-crc · enkidb-engine · enkidb-journal · enkidb-kaki · heptascript |
| enkidb-ingest | akkvalue · bahyway-core · bahyway-crc · enkidb-engine · enkidb-journal · enkidb-kaki · enkidb-particles |
| enkidb-raft | *(none)* |

---

### Layer 4.5 — Physics / Intelligence

| Caller | Direct Callees |
|--------|----------------|
| tribe-orbit-engine | bahyway-algebra |
| ammas-engine | bahyway-algebra |
| riksu-engine | shedu-engine |
| vgca-engine | *(none)* |
| shedu-engine | *(none)* |
| vault-engine | *(none)* |

---

### Layer 3 — Indexes + Dictionary

| Caller | Direct Callees |
|--------|----------------|
| enkidb-indexes | bahyway-core · enkidb-kaki · enkidb-snapshot · enkidb-vector-id |
| enkidb-dictionary | bahyway-core · bahyway-crc |

---

### Layer 2 — Storage Substrate

| Caller | Direct Callees |
|--------|----------------|
| enkidb-block | bahyway-core · bahyway-crc · enkidb-kaki |
| enkidb-storage | bahyway-core · bahyway-crc · enkidb-block · enkidb-kaki |
| enkidb-journal | bahyway-core · bahyway-crc · enkidb-kaki · enkidb-storage |
| enkidb-snapshot | bahyway-core · enkidb-journal · enkidb-kaki · enkidb-vector-id |
| enkidb-recovery | bahyway-core · enkidb-journal · enkidb-kaki · enkidb-snapshot |
| enkidb-persist | bahyway-core · bahyway-crc · enkidb-engine · enkidb-journal · enkidb-kaki · enkidb-storage |
| enkidb-dw | aaol · bahyway-core · bahyway-crc · compare-tribe-schema · enkidb-engine · enkidb-journal · enkidb-kaki · enkidb-persist · enkidb-storage · musaru-security · story-engine |
| enkidb-sdb | bahyway-core · enkidb-dw · enkidb-journal · enkidb-kaki · musaru-security |
| enkidb-qdb | bahyway-core · enkidb-journal · enkidb-kaki · enkidb-sdb |
| enkidb-odb | bahyway-core · enkidb-journal · enkidb-kaki · enkidb-sdb |

---

### Layer 1 — KAKI / Identity

| Caller | Direct Callees |
|--------|----------------|
| enkidb-kaki | bahyway-core · bahyway-crc |
| enkidb-vector-id | bahyway-core |
| enkidb-particles | akkvalue · bahyway-core · enkidb-kaki |

---

### Layer 0 — Foundation (leaf nodes, zero workspace deps)

| Crate | Role |
|-------|------|
| bahyway-core | ParticleState · TribeId · QUALITY_DIVISOR · SATTATU_MAX · shared types |
| bahyway-crc | CRC-32 / CRC-64 checksum — no external crates |
| bahyway-algebra | Linear algebra for physics engines — no external crates |

---

## 3. Critical Processing Chain — bee-watchdog ETL

This is the chain you traverse when an error appears in bee-watchdog's output.

```
bee-watchdog (main)
│
├── enkidb-dw::LandingZone::poll()
│       └── bahyway-core   (TribeId, types)
│
├── musaru-security::zip_scan()            ← SECURITY GATE
│       ├── bahyway-core
│       └── enkidb-journal · enkidb-kaki
│
├── enkidb-dw::zip_engine::extract_ok()   ← ZIP extraction
│       └── bahyway-core · bahyway-crc · enkidb-storage
│
├── enkidb-dw::ProcessingZone::stage()    ← write to Processing/
│       └── enkidb-storage
│
├── enkidb-dw::BatchSchema::infer()       ← schema detection
│       └── enkidb-kaki
│
├── client-dq-profile::parse_profile()   ← load .dqprofile
│   client-dq-profile::ClientDqProfile::default_from_schema()
│       ├── bahyway-core
│       ├── fuzzy-engine                  ← dimension weights
│       └── score-engine                  ← B11 thresholds
│
├── template-engine::Template::default_template()
│       └── bahyway-core · enkidb-kaki
│
├── enkidb-dw::kaki_generator::generate() ← per record
│       └── enkidb-kaki  (mint IdentityKaki + EventKaki + EAV)
│
├── data-structure-station::structure()   ← EAV normalization
│       └── bahyway-core · template-engine
│
├── client-dq-profile::compute_dims()    ← FuzzyDimensions D1-D8
│       └── score-engine::FreshnessDecay
│
├── score-engine::score()                 ← B11 + ColorRGB + tier
│   score-engine::tier_to_state()
│       ├── bahyway-core   (ParticleState)
│       ├── fuzzy-engine   (dimension math)
│       └── enkidb-journal · enkidb-kaki  (EAV triple output)
│
├── ─ Golden path (B11 ≥ 140) ──────────────────────────────────
│   permanent-storage::PermanentStore::commit()
│       └── enkidb-engine · enkidb-journal · enkidb-kaki · story-engine
│   enkidb-persist::PersistedDb::commit()
│       └── enkidb-engine · enkidb-journal · enkidb-kaki · enkidb-storage
│
├── ─ Fuzzy path (B11 100-139) ─────────────────────────────────
│   alert-engine::Alert::new()
│       └── score-engine · enkidb-indexes · enkidb-kaki · bahyway-core
│   data-steward-station::StewardStation::receive()
│       └── alert-engine · diagnosis-templates
│   enkidb-persist::PersistedDb::commit()
│
├── ─ Dead path (B11 < 100) ────────────────────────────────────
│   (log only → eprintln!, same persist path as Fuzzy)
│
├── story-engine::encode_state()           ← batch-level journal entry
│       └── enkidb-indexes · enkidb-journal · enkidb-kaki · enkidb-snapshot
│
├── enkidb-engine::EnkiDb::project()       ← StoryEngine gate verify
│       └── enkidb-indexes · enkidb-journal · enkidb-kaki · enkidb-recovery
│
├── enkidb-dw::ProcessingZone::complete()  ← move → Moved_To/
│       └── enkidb-storage
│
├── write_live_export()      → {data_dir}/live_export.json
├── write_batches_export()   → {data_dir}/batches_export.json
└── write_tribes_summary()   → {data_dir}/tribes_summary.json
```

---

## 4. Error Message → Crate Lookup

When you see `[ERR!]` or `[WARN]` in bee-watchdog output, the prefix tells you
which station produced it:

| Log prefix / error keyword | Source crate | Look in |
|---------------------------|--------------|---------|
| `SECURITY BLOCK — Musarû` | musaru-security | `crates/musaru-security/src/` |
| `no extractable entries` | enkidb-dw zip_engine | `crates/enkidb-dw/src/zip_engine.rs` |
| `stage error` / `move to Moved_To failed` | enkidb-dw ProcessingZone | `crates/enkidb-dw/src/processing_zone.rs` |
| `schema →` / BatchSchema | enkidb-dw BatchSchema | `crates/enkidb-dw/src/batch_schema.rs` |
| `failed to parse .dqprofile` | client-dq-profile parse | `crates/client-dq-profile/src/parse.rs` |
| `[DEAD] particle` + B11 value | score-engine tier | `crates/score-engine/src/lib.rs` |
| `PersistedDb open failed` | enkidb-persist | `crates/enkidb-persist/src/lib.rs` |
| `LandingZone open failed` | enkidb-dw LandingZone | `crates/enkidb-dw/src/landing_zone.rs` |
| `ProcessingZone init failed` | enkidb-dw | `crates/enkidb-dw/src/processing_zone.rs` |
| `StoryEngine gate` | story-engine + enkidb-engine | `crates/story-engine/` + `crates/enkidb-engine/` |
| `live_export.json write failed` | bee-watchdog (fs::write) | `bin/bee-watchdog/src/main.rs` → `write_live_export()` |
| `batches_export.json write failed` | bee-watchdog (fs::write) | `bin/bee-watchdog/src/main.rs` → `write_batches_export()` |
| `tribes_summary.json write failed` | bee-watchdog (fs::write) | `bin/bee-watchdog/src/main.rs` → `write_tribes_summary()` |
| `API unreachable` (browser console) | bahyway-web fetch | `crates/bahyway-web/src/lib.rs` → `fetch_*()` |
| `bind … failed` | bahyway-api main | `bin/bahyway-api/src/main.rs` → `main()` |

---

## 5. Reverse Lookup — Who Calls This Crate?

Given a crate you're debugging, find every caller that imports it:

| Callee | Direct callers (all workspace crates) |
|--------|---------------------------------------|
| **bahyway-core** | almost every crate — universal root |
| **bahyway-crc** | enkidb-kaki · enkidb-block · enkidb-storage · enkidb-journal · enkidb-persist · enkidb-dw · bahyway-algebra · enkidb-dictionary · enkidb-query · enkidb-ingest · heptascript · enkidullm-core · enkidullm-ingest · enkidullm-audit |
| **enkidb-kaki** | enkidb-block · enkidb-storage · enkidb-journal · enkidb-snapshot · enkidb-recovery · enkidb-persist · enkidb-dw · enkidb-sdb · enkidb-qdb · enkidb-odb · enkidb-indexes · enkidb-engine · enkidb-query · enkidb-ingest · enkidb-particles · story-engine · score-engine · alert-engine · snapshot-job · navi-engine · najaf-engine · heptascript · template-engine · musaru-security · adad-gate · vgca-validation · permanent-storage · homt-engine · idu-prober · idu-batching · diagnosis-engine · enkidullm-core · zikru-embed · enkidullm-audit · eridu-runtime · dubsar-visualizer · **bee-watchdog** · **bahyway-web** · **najaf-ingest** · **dubsar** · **enkidw** |
| **enkidb-journal** | enkidb-storage → enkidb-journal · enkidb-snapshot · enkidb-recovery · enkidb-persist · enkidb-dw · enkidb-sdb · enkidb-qdb · enkidb-odb · enkidb-engine · enkidb-query · enkidb-ingest · story-engine · score-engine · snapshot-job · heptascript · musaru-security · adad-gate · vgca-validation · data-structure-station · data-cleansing-station · permanent-storage · client-dq-profile · diagnosis-engine · idu-prober · idu-batching · eridu-runtime · dubsar-visualizer · **bee-watchdog** · **bahyway-web** |
| **enkidb-storage** | enkidb-journal · enkidb-persist · enkidb-dw · enkidb-block · **bee-watchdog** · **najaf-ingest** · **dubsar** · **enkidw** |
| **enkidb-engine** | enkidb-persist · enkidb-dw · enkidb-query · enkidb-ingest · adad-gate · permanent-storage · eridu-runtime · **bee-watchdog** · **bahyway-server** · **najaf-ingest** · **dubsar** · **bahyway-cli** |
| **enkidb-persist** | enkidb-dw · **bee-watchdog** · **najaf-ingest** · **dubsar** · **enkidw** |
| **enkidb-dw** | enkidb-sdb · **bee-watchdog** · **enkidw** |
| **story-engine** | enkidb-engine · enkidb-dw · score-engine · permanent-storage · snapshot-job · diagnosis-engine · idu-prober · idu-batching · dubsar-visualizer · **bee-watchdog** · **najaf-ingest** · **dubsar** · **enkidw** |
| **score-engine** | client-dq-profile · alert-engine · dubsar-visualizer · **bee-watchdog** · **dubsar** |
| **fuzzy-engine** | score-engine · client-dq-profile |
| **alert-engine** | data-steward-station · diagnosis-templates · diagnosis-engine · **bee-watchdog** |
| **musaru-security** | enkidb-dw · enkidb-sdb · **bee-watchdog** · **najaf-ingest** · **dubsar** |
| **template-engine** | template-library · damadmbok-dictionary · compare-tribe-schema · vgca-validation · data-structure-station · data-cleansing-station · diagnosis-templates · dubsar-ide · **bee-watchdog** · **dubsar** |
| **client-dq-profile** | **bee-watchdog** |
| **permanent-storage** | **bee-watchdog** · **najaf-ingest** · **dubsar** |
| **data-structure-station** | **bee-watchdog** |
| **data-steward-station** | **bee-watchdog** |
| **enkidb-snapshot** | enkidb-recovery · enkidb-indexes · enkidb-engine · story-engine · snapshot-job · idu-prober · idu-batching · dubsar-visualizer |
| **enkidb-indexes** | enkidb-engine · alert-engine · story-engine |
| **navi-engine** | najaf-engine · heptascript |
| **nusku-engine** | azuga-engine · iris-engine · panam-engine · wpd-engine · nusku-score · shulman-engine |

---

## 6. Zero-Dependency Leaf Nodes

These crates have **no workspace dependencies** — they never produce a
"missing crate" link error and are always safe to check in isolation:

```
bahyway-core      bahyway-crc       bahyway-algebra
enkidb-raft       vgca-engine       shedu-engine      vault-engine
kinetic-engine    pollution-engine  hepta-score        nusku-engine
hepta             akkadi            akkadi-ir          dfg-engine
ezida-ir          kupru             akkvalue           istar
```

---

*𒁾 DUB.SAR — BahyWay v4 Call Graph | 2026-06-02*
