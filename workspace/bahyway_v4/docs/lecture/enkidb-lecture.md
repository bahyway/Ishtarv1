---
marp: true
theme: default
class: invert
paginate: true
size: 16:9
style: |
  :root {
    --gold: #d4a843;
    --gem:  #00e5a0;
    --tribe: #4db8ff;
    --fuzzy: #ff8c42;
    --bg:   #0a0e1a;
  }
  section {
    background: #0a0e1a;
    color: #e8e8e8;
    font-family: 'Segoe UI', Arial, sans-serif;
    font-size: 22px;
    padding: 40px 60px;
  }
  h1 { color: #d4a843; font-size: 1.9em; margin-bottom: 0.2em; }
  h2 { color: #d4a843; font-size: 1.4em; border-bottom: 1px solid rgba(212,168,67,0.3); padding-bottom: 0.3em; }
  h3 { color: #00e5a0; font-size: 1.1em; }
  strong { color: #d4a843; }
  em { color: #00e5a0; font-style: normal; }
  code { background: rgba(212,168,67,0.12); color: #00e5a0; padding: 0.1em 0.4em; border-radius: 3px; font-family: 'JetBrains Mono', monospace; }
  pre { background: rgba(0,0,0,0.5); border: 1px solid rgba(212,168,67,0.2); border-radius: 6px; padding: 0.8em 1.2em; }
  pre code { background: none; padding: 0; }
  blockquote { border-left: 4px solid #d4a843; background: rgba(212,168,67,0.08); padding: 0.5em 1em; margin: 0.5em 0; color: #e8d5a3; }
  table { font-size: 0.85em; width: 100%; }
  th { background: rgba(212,168,67,0.15); color: #d4a843; }
  tr:nth-child(even) { background: rgba(255,255,255,0.04); }
  .ar { direction: rtl; text-align: right; font-size: 0.9em; color: #e8d5a3; margin-top: 0.3em; display: block; }
  section.title-slide { text-align: center; }
  section.section-break { background: linear-gradient(135deg, #0a0e1a 0%, #1e3a5f 100%); text-align: center; }
  section.section-break h1 { font-size: 2.5em; padding-top: 1em; }
---

<!-- _class: title-slide -->

# 𒀭𒂗𒆤 EnkiDB

## Orbit-Oriented Ontology & the BahyWay Triple-O Philosophy

<span class="ar">إنكي-قاعدة البيانات — فلسفة الكون المداري السيادي</span>

---

**Bahaa Fadam** — BahyWay Sovereign Ecosystem
Iraqi Technology University, Baghdad · 2026

*v4.0.2 · 79 Crates · Pure Rust · 1,804 Tests · Zero External Databases*

---

## Agenda — 90 Minutes

<span class="ar">جدول الأعمال</span>

| # | Section | Duration |
|---|---------|----------|
| I | **The Problem** — Why all databases fail | 15 min |
| II | **Triple-O Philosophy** — 3 Axioms · KAKI · H(P) | 20 min |
| III | **The Engine** — 12 Layers · BeeMDM Pipeline | 20 min |
| IV | **Live Demo** — WASM Particle Visualizer | 15 min |
| V | **Real Applications** — Najaf · WPD · Nusku | 10 min |
| — | **Q&A** — Sovereignty Manifesto | 10 min |

---

<!-- _class: section-break -->

# Part I — The Problem

<span class="ar" style="font-size:1.4em">الجزء الأول — المشكلة</span>

---

## Every Enterprise Has This Problem

<span class="ar">كل مؤسسة تعاني من هذه المشكلة</span>

**The Spaghetti Reality:**
- 12 databases, each with its own schema for "customer"
- 3 ETL pipelines that disagree on what "name" means
- Nobody knows which record is the truth
- `DELETE` destroys audit history · `UPDATE` erases causality
- Soft-delete hacks · audit log afterthoughts

> "A datum without an orbit is data without a home."
> <span class="ar">بيانات بلا مدار هي بيانات بلا وطن</span>

---

## Why Relational Databases Fail at Scale

<span class="ar">لماذا تفشل قواعد البيانات العلائقية على النطاق الواسع</span>

| Problem | Root Cause |
|---------|-----------|
| **Identity Confusion** | Row ID ≠ Entity. UUID collision at billion scale. |
| **Quality is Implicit** | "Valid" = syntactically OK. No semantic orbit. |
| **State Overwrites History** | UPDATE destroys causality. Triggers are side-effects. |
| **No Trust Model** | Assumed (you control it). Never computed. |
| **Arabic Hostility** | Collation hacks. No semantic understanding. |

> Enki's answer (4,000 years ago):
> *"The tablet is sealed. The scribe adds a new tablet. The old tablet is eternal."*
> <span class="ar">اللوح مختوم. الكاتب يضيف لوحاً جديداً. اللوح القديم أبدي.</span>

---

<!-- _class: section-break -->

# Part II — Triple-O Philosophy

<span class="ar" style="font-size:1.4em">الجزء الثاني — فلسفة الكون المداري المثلث</span>

*3 Axioms that replace all of relational theory*

---

## Axiom 1 — Every Datum is a Particle

<span class="ar">البديهية الأولى — كل بيانة هي جسيم</span>

A datum is **not** a row, document, or record.

A datum is a **physical object** with:

- **Position** in 7-dimensional quality space
- **Velocity** — rate of state change
- **Density** — neighbour count (SPH model)
- **Orbital energy** — quality score H(P)

> Physics metaphor: data obeys the same laws as matter.
> Every datum has a place it *belongs*, and we can *measure* how far it has drifted.

---

## Axiom 2 — Identity ≠ State

<span class="ar">البديهية الثانية — الهوية ليست الحالة</span>

```
KAKI Nucleus (κ) — 16 bytes — IMMUTABLE — born once, sealed forever
┌──────────┬──────────┬────────┬────────┬─────────┬─────────┬────────┐
│uuid_hash │ tribe_id │  type  │  role  │   seq   │  epoch  │ crc16  │
│  4 bytes │  2 bytes │ 1 byte │ 1 byte │ 4 bytes │ 2 bytes │ 2 bytes│
└──────────┴──────────┴────────┴────────┴─────────┴─────────┴────────┘
```

**EAV Orbit** — mutable · event-driven · append-only · time-stamped
← All state lives here. *Never* in the nucleus.

> *The nucleus never changes. The orbit evolves.*
> <span class="ar">الجوهر لا يتغير. المدار يتطور.</span>

---

## Axiom 3 — Quality is Orbital Distance

<span class="ar">البديهية الثالثة — الجودة هي المسافة المدارية</span>

```
H(P) = 1 / (1 + √ Σ wᵢ(Pᵢ − Tᵢ)²)

B11 = round(H(P) × 240)    →    [0 .. 240]
```

**7 Ibn Wahshiyya Dimensions:**

| D | Dimension | Weight |   | Lane | B11 |
|---|-----------|--------|---|------|-----|
| D1 | Accuracy | 0.30 | | **GEM** | ≥ 200 |
| D2 | Completeness | 0.20 | | **TRIBE** | 140–199 |
| D3 | Consistency | 0.15 | | **ACTIVE** | 100–139 |
| D4 | Validity | 0.15 | | **FUZZY** | 60–99 |
| D5 | Uniqueness | 0.10 | | **DEAD** | < 60 |
| D6 | Timeliness | 0.05 | | | |
| D7 | Integrity | 0.05 | | *Why 240?* Plimpton 322 | |

---

## The Three Orbital Rings

<span class="ar">الحلقات المدارية الثلاث</span>

```
                    ╔══════════════════════╗
                    ║   ●  GEM  ●          ║  Inner Ring (r ≈ 1.0)
                    ║  ● TRIBE ●  ACTIVE ● ║  Mid Ring   (r ≈ 2.2)
                    ║ ●  FUZZY   ● DEAD  ● ║  Outer Ring (r ≈ 3.5)
                    ╚══════════════════════╝
```

| Ring | Lanes | Radius | Rule |
|------|-------|--------|------|
| **Inner** | GEM | ≈ 1.0 | Calibrates centroid · target 35.4% |
| **Mid** | TRIBE + ACTIVE | ≈ 2.2 | Core tribal members |
| **Outer** | FUZZY + DEAD | ≈ 3.5 | Rule 7: ≥11 neighbours → DeadArchive |

**TAU_R7 = 11** · **RESONANCE_RADIUS = 0.15** · **GEM_RATE_TARGET = 35.4%**

---

## KAKI — Three Types, Three Roles

<span class="ar">كاكي — ثلاثة أنواع وثلاثة أدوار</span>

| Type | Code | Role | Purpose |
|------|------|------|---------|
| **IdentityKaki** | `0x01` | KISHIB `0x01` | Permanent sovereign particle. Born once. |
| **EventKaki** | `0x02` | ZIKRU `0x02` | Mutable state carrier. Each change = new seal. |
| **CrossTribeKaki** | `0x03` | PARZU `0x03` | Inter-tribe connector. Topology bridge. |

**Forbidden in κ nucleus:**
- ❌ Quality scores (B11)
- ❌ Lane assignments (GEM/TRIBE/…)
- ❌ VGCA vectors
- ❌ Any state that can change

> 4.3 billion KAKIs per tribe per epoch (seq_counter = 32-bit)

---

## The 7 Tribal Laws

<span class="ar">القوانين القبلية السبعة</span>

Priority ascending: `P(L3) < P(L7) < P(L1) < P(L2) < P(L5) < P(L4) < P(L6)`

| Law | Name | Rule |
|-----|------|------|
| L3 | **Scope Law** | Domain boundary enforcement |
| L7 | **Overflow Law** | Density ≥ 11 → DeadArchive (Rule 7) |
| L1 | **Identity Law** | KAKI nucleus locked forever |
| L2 | **Quality Law** | Promotes particles improving H(P) |
| L5 | **Transition Law** | Stabilises FUZZY, prevents oscillation |
| L4 | **Orbit Law** | Governs GEM ascent via centroid |
| L6 | **Sovereignty Law** | SUPREME — GEM particles protected |

---

<!-- _class: section-break -->

# Part III — The Engine

<span class="ar" style="font-size:1.4em">الجزء الثالث — المحرك</span>

*79 crates · 12 layers · Pure Rust · Zero external databases*

---

## 12-Layer Architecture

<span class="ar">البنية المعمارية ذات الـ ١٢ طبقة</span>

```
L12  bahyway-web           Pure DOM + Canvas 2D · WASM target
L11  dubsar-ide · visualizer
L10  eridu-runtime · scheduler · supervisor
L9.5 enkidullm-core · ingest · zikru-embed · audit    ← Sovereign LLM
L9.1 kupru · akkvalue · istar                          ← Crypto / ACL
L9   aaol · heptascript · akkadi · akkadi-ir           ← Languages
L8   bahyway-fabric · bahyway-dqm  ◄── Enterprise Data Fabric
     adad-gate → cleansing → permanent-storage
L7   template-engine · damadmbok-dictionary
L6   idu-prober · idu-batching
L5   story · fuzzy · score · hepta-score · najaf · wpd (18 engines)
L4.5 vgca-engine · tribe-orbit-engine · ammas-engine   ← Physics
L4   enkidb-engine · enkidb-query
L3   enkidb-indexes (7 types) · enkidb-dictionary
L2   block · journal · storage · snapshot · quantdb
L1   enkidb-kaki · enkidb-vector-id
L0   bahyway-core · bahyway-crc · bahyway-algebra
```

---

## The Eternal Constants

<span class="ar">الثوابت الأبدية — لا يمكن تغييرها أبداً</span>

| Constant | Value | Origin |
|----------|-------|--------|
| `QUALITY_DIVISOR` | **240** | Plimpton 322 (Babylonian base-60) |
| `PARTICLES_PER_TRIBE` | **7** | Ibn Wahshiyya 7 quality dimensions |
| `TAU_R7` | **11** | Rule 7 overflow trigger |
| `RESONANCE_RADIUS` | **0.15** | SPH cubic kernel window |
| `GEM_RATE_TARGET` | **35.4%** | Sovereign ideal distribution |
| `DELTA_FRAG` | **0.35** | Fragmentation threshold |
| `UNKNOWN_FLOOR` | **0.10** | Minimum SourceTrust floor |

> These constants are sealed in **ADR-001**. Changing them invalidates the entire mathematical foundation.

---

## BeeMDM — Sovereign ETL Pipeline

<span class="ar">بي-إم-دي-إم — خط أنابيب البيانات السيادي</span>

```
S0  adad-gate              VaultGate — sole ingestion entry point · KAKI minter
 ↓
S1  musaru-security        Authentication · Authorization · Threat detection
 ↓
S2  vgca-validation        VGCA 7D FSV geometry + 6D BFV delta validation
 ↓
S3  data-structure-station  Structural conformance
 ↓
S4  data-cleansing-station  VGCA transformation + DQM 6-dimension scoring
 ↓
S5  bahyway-dqm            DAMA-DMBOK: Completeness·Validity·Accuracy·
                           Consistency·Uniqueness·Timeliness
 ↓
S6  idu-prober             Cross-tribe identity resolution (Shamash IDU)
 ↓
S7  data-steward-station   Governance + approval workflow
 ↓
S8  permanent-storage      Golden Records · Immutable · CRC-16 sealed ✓
```

---

## StoryEngine — Time-Travel Queries

<span class="ar">محرك القصة — استعلامات السفر عبر الزمن</span>

**CQRS Architecture:**
- Write side: Append-only Journal (Event-KAKIs)
- Read side: Event projection (state never stored, always projected)
- No `UPDATE`. No `DELETE`. **Ever.**

```rust
// Time-travel: state at any historical epoch
story_engine.project_at(&particle, epoch_2024_01_01)
// → finds snapshot ≤ epoch
// → applies delta Event-KAKIs since snapshot
// → returns complete state at that point in history
```

> Every state change adds a new Event-KAKI.
> The old state is still there. Always.
> <span class="ar">كل تغيير يضيف ختماً جديداً. الحالة القديمة لا تُمحى أبداً.</span>

---

## orbital-trust-probe — Self-Regulating Quality

<span class="ar">مسبار الثقة المداري — الجودة ذاتية التنظيم</span>

**Problem:** At billion-particle scale, a particle may drift from its orbit even when fuzzy rules haven't changed. Is this a data quality problem or a trust problem?

**4-Step Causal Attribution:**
1. **FuzzyRules fingerprint** — did the rules change?
2. **StoryEngine EAV delta** — legitimate state evolution?
3. **ScoreEngine field analysis** — neighbour density shift?
4. **Unexplained residual** → trust penalty applied

**Closed Feedback Loop:**
```
OrbitalDeviation → D9 penalty → penalised D6 (SourceTrust)
→ lower B11 → organic ring correction → self-terminating cascade ✓
```

---

<!-- _class: section-break -->

# Part IV — Live Demo

<span class="ar" style="font-size:1.4em">الجزء الرابع — العرض الحي</span>

*Rust → WebAssembly → Browser · bahyway.com*

---

## BeeMDM Live Demo

<span class="ar">العرض الحي لخط أنابيب BeeMDM</span>

**What you will see:**

**Panel 1 — Orbital Rings**
100 particles orbiting in real time. Color = quality lane.
Watch particles migrate as quality scores change.

**Panel 2 — BeeMDM Pipeline**
Live ETL processing. Each station lights up as data flows.
DQM 6-dimension heatmap per record.

**Panel 3 — Story Engine**
Time-travel slider. Drag to any epoch → see particle state.
CQRS projection computed live in the browser.

> **Tech:** Rust compiled to WASM via `wasm-bindgen` · Canvas 2D via `web-sys`
> Zero server calls · Runs entirely in your browser
> <span class="ar">يعمل بالكامل في متصفحك — لا خادم مطلوب</span>

---

<!-- _class: section-break -->

# Part V — Real Applications

<span class="ar" style="font-size:1.4em">الجزء الخامس — التطبيقات الحقيقية</span>

---

## NajafEngine — Cemetery Navigation

<span class="ar">محرك النجف — نظام ملاحة المقابر الكبرى</span>

**The Problem:**
Wadi Al-Salam, Najaf — the world's largest cemetery.
5–6 million graves. Families arrive and cannot find ancestors.
No unified registry. No spatial index.

**The EnkiDB Solution:**
- Each grave = a sovereign **KAKI particle** (IdentityKaki type 0x01)
- Spatial position encoded in H3 hexagonal index (geo_precision dimension D5)
- Family relations via **CrossTribeKaki** bridges
- Historical records via **StoryEngine** time-travel
- **orbital-trust-probe** detects data drift across archival records

```
crate: najaf-engine
binary: bin/najaf-ingest  ← sovereign ingestion pipeline
```

---

## WPD, DMW & Nusku

<span class="ar">أنابيب المياه وبغداد ونوسكو</span>

**WPDEngine — Baghdad Water Pipeline**
Each pipe segment = KAKI particle · Pressure readings = Event-KAKIs
`orbital-trust-probe` detects anomalous pressure before leaks form

**DMWEngine — SQL Advisor**
Translates legacy SQL queries → KAKI-based retrieval
Bridges old relational systems to sovereign data architecture

**NuskuEngine — Thermal Screening**
*(Named after the Sumerian god of fire and light)*
IR camera data → particle orbit analysis
Temperature anomalies = orbital deviations → alert-engine
50 ms thermal pipeline budget (sovereign SLA)

**AMMASEngine — Kinetic Particle System**
*(Named after the Akkadian war god)*
SPH density + orbital velocity · Models large-scale data migration

---

## EnkiDB vs. Traditional Databases

<span class="ar">إنكي-قاعدة البيانات مقابل قواعد البيانات التقليدية</span>

| Capability | PostgreSQL / MySQL | **EnkiDB** |
|-----------|-------------------|------------|
| Identity | AUTO_INCREMENT | KAKI 16-byte sovereign seal |
| Data Quality | Constraints (syntax) | H(P) equation — 7D orbital score |
| Deletion | `DELETE` (destroys history) | ❌ Forbidden — INSERT supersedes |
| Update | `UPDATE` (erases causality) | New Event-KAKI appended |
| Time Travel | Audit log add-on | Native CQRS `project_at(epoch)` |
| Trust | Assumed | Computed — orbital deviation probe |
| Dependencies | External runtime | Pure Rust — zero external deps |
| Arabic | Collation hacks | VGCA-Σ 7D native Arabic geometry |

---

## BahyWay.Ecosystem — By the Numbers

<span class="ar">النظام الإيكولوجي بالأرقام</span>

| Metric | Value |
|--------|-------|
| Sovereign Crates | **79** |
| Passing Tests | **1,804** |
| Lines of Rust | **~74,022** |
| Architecture Layers | **12** |
| External Databases | **0** |
| `unsafe {}` blocks (physics) | **0** |
| Bytes per KAKI | **16** |
| KAKIs per tribe per epoch | **4,294,967,294** |
| Forbidden Operations | **17** |

---

## The Sovereignty Manifesto

<span class="ar">البيان السيادي</span>

🏛 **Your data is sovereign territory.** It obeys your laws, not Amazon's.

🔑 **Every datum deserves an immutable identity.** KAKI is its permanent passport.

🌌 **Quality is not a checkbox.** It is a mathematical distance in orbital space.

📜 **History is sacred.** No operation may erase what was.

🦀 **Memory safety is sovereignty.** Rust enforces what laws cannot.

🌍 **Iraqi mathematics built this.** Plimpton 322 · Ibn Wahshiyya · Enki · Gilgamesh.

> <span class="ar">بياناتك أرض سيادية. هويتها أبدية. جودتها رياضية. تاريخها مقدس.</span>

---

<!-- _class: title-slide -->

# 𒀭𒂗𒆤

## Questions?

<span class="ar" style="font-size:1.5em">أسئلة؟</span>

---

**github.com/bahyway/enkidb** · **bahyway.com** · **heptascript.com**

*Bahaa Fadam · bahaa.fadam@gmail.com*
*BahyWay Sovereign Ecosystem*

EnkiDB v4.0.2 · 79 Crates · Pure Rust · 1,804 Tests · Zero External Databases
