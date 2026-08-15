# akkvalue — Sovereign EAV Value Type
𒀊𒆪 *akkû* — "sovereign quality / the precious thing itself"

> **Layer 9.1 · Cross-cutting** | 31 variants | Tagged JSON · Binary compatible

---

## W5H2 Manual

### WHO — 𒀭 Who Uses This Crate

| Persona | Role |
|---|---|
| **EnkiDB storage layer** (`enkidb-engine`, `enkidb-persist`) | Stores and retrieves typed EAV triples |
| **Pipeline stations** (adad-gate → permanent-storage) | Pass typed values between processing steps |
| **Story Engine** (`story-engine`) | Carries typed narrative payloads |
| **Hepta Score Engine** (`hepta-score`) | Emits `AkkValue::QualityScore(b11)` per dimension |
| **Akkadi CLI** (`bin/akkadi-cli`) | Displays and inspects typed values in tables / JSON |
| **AkkadianAOL compiler** (`aaol`) | Evaluates typed expressions during semantic analysis |
| **istar firewall** (`istar`) | Reads `AkkValue::PolicyVerdict` from rule evaluation |

---

### WHAT — 𒁾 What This Tablet Contains

`akkvalue` defines **`AkkValue`** — the single canonical typed value for the entire BahyWay.Ecosystem.  
Every EAV (Entity–Attribute–Value) triple in EnkiDB carries one `AkkValue`.  
Replacing 31 `match` arms scattered across the codebase with one shared enum.

#### The 31 Variants

**Scalar (6)**
```
Null | Bool(bool) | Int(i64) | Float(f64) | Text(String) | Bytes(Vec<u8>)
```

**Identity (3)**
```
Uuid(uuid::Uuid) | NationalId(AkkNationalId) | Phone(AkkPhone)
```

**Temporal (4)**
```
Timestamp(i64) | Date(AkkDate) | HijriDate(AkkHijriDate) | Duration(u64)
```

**Geographic (2)**
```
Coordinate(AkkCoordinate) | CountryCode(String)   // ISO 3166-1 alpha-2
```

**Domain / KAKI (3)**
```
DomainByte(u8) | QualityScore(u8)   // 0–240, QUALITY_DIVISOR=240.0
              | KakiPk([u8; 16])
```

**Linguistic (3)**
```
AkkadianRoot(String) | NameVector(AkkNameVector) | LangCode(String)  // BCP-47
```

**ML / Analytics (4)**
```
Embedding(Vec<f32>) | Probability(f64) | Label(String) | Confidence(f64)
```

**Pipeline (2)**
```
PipelineStatus(PipelineStatus) | StepIndex(u32)
```

**Security / Crypto (3)**
```
PolicyVerdict(PolicyVerdict) | CipherAlgorithm(CipherAlgorithm) | SealSignature(Vec<u8>)
```

**Structural (1)**
```
List(Vec<AkkValue>)   // recursive — nested lists supported
```

#### Supporting types (`types.rs`)

| Type | Description |
|---|---|
| `AkkCoordinate` | WGS-84 lat/lon/alt |
| `AkkDate` | Gregorian ISO 8601 |
| `AkkHijriDate` | Islamic calendar — KAKI B12 domain |
| `AkkNationalId` | ISO country code + national number |
| `AkkPhone` | ITU country code + subscriber number |
| `AkkNameVector` | Ordered name parts (Given/Family/Father/Grandfather/Tribe/Honorific/Kunyah/Laqab) |
| `PolicyVerdict` | Allow / Deny / Escalate / Redact / Audit |
| `CipherAlgorithm` | ChaCha20-Poly1305 / AES-256-GCM / Ed25519 / Dilithium-3 |
| `PipelineStatus` | Pending / Running / Completed / Failed(String) / Cancelled |

---

### WHEN — 𒌓 When Is This Invoked

`AkkValue` is active **at every boundary** in the data pipeline:

```
Sensor / Source
    │  AkkValue::Float(15.3)   PM2.5 reading
    ▼
adad-gate (KAKI validation)
    │  AkkValue::KakiPk([u8;16])
    ▼
data-cleansing-station
    │  AkkValue::QualityScore(185)
    ▼
story-engine
    │  AkkValue::Text("PM2.5 exceeds WHO limit")
    ▼
EnkiDB (EAV store)
    │  AkkTriple { entity: "sensor:7", attribute: "pm2_5", value: AkkValue::Float(15.3) }
    ▼
Akkadi CLI display
    │  Table / JSON / Cuneiform
    ▼
permanent-storage
    │  AkkFile payload = serde_json::to_vec(&triple)?
```

---

### WHERE — 𒆳 Architectural Position

```
akkvalue is a cross-cutting concern — it has NO layer position of its own.
It is imported by crates at ALL layers above Layer 0.

Layer 9.1 (where it lives physically):
    kupru  ◄──  akkvalue  ──►  istar
               (shared types)
                    ▲
    ┌───────────────┼───────────────┐
    │               │               │
Layer 2         Layer 5         Layer 8
enkidb-persist  story-engine    permanent-storage
```

The rule: **any crate that stores, transmits, or displays a typed value imports `akkvalue`**.

---

### WHY — 𒀊 Why This Exists

**The problem it solves:**  
In v3.5, each engine had its own value type (`NuskuValue`, `StoryValue`, `HeptaValue`).  
Converting between them at crate boundaries created constant impedance mismatch and deserialization errors.

**The sovereign solution:**  
One canonical type. Every crate speaks `AkkValue`. No conversion needed at boundaries.

**Why 31 variants?**  
The variants cover the complete **BahyWay domain ontology**:
- Scalars for raw sensor data
- Identity types for KAKI-compliant stakeholder data (Iraqi National ID, phone)
- Hijri date for KAKI B12 Islamic calendar compliance
- ML types (Embedding, Probability, Confidence) for sovereign AI pipelines
- Security types (SealSignature, CipherAlgorithm) so crypto metadata travels with data

**Why tagged JSON?**  
`{"type":"QualityScore","value":185}` is human-readable, debuggable in the DubSar IDE,  
and survives schema evolution — adding a new variant is non-breaking for existing consumers.

**Why `visit_bytes` + `visit_seq` deserialiser?**  
The serde visitor handles both binary encodings (CBOR-style, length-prefixed arrays)  
and JSON text — the same `AkkValue` struct works in storage (binary) and API (JSON) contexts.

---

### HOW — 𒅗 How It Works

#### Creating values

```rust
use akkvalue::{AkkValue, AkkDate, AkkTriple};

let reading   = AkkValue::Float(15.3);
let date      = AkkValue::Date(AkkDate::new(2026, 5, 31));
let quality   = AkkValue::QualityScore(185);    // TribeMember lane

// EAV triple
let triple = AkkTriple::new("sensor:baghdad-7", "pm2_5", reading);
let json   = serde_json::to_string(&triple)?;
// → {"entity":"sensor:baghdad-7","attribute":"pm2_5","value":{"type":"Float","value":15.3}}
```

#### Pattern matching

```rust
match value {
    AkkValue::QualityScore(q) if q >= 200 => println!("Gem ✓"),
    AkkValue::QualityScore(q) if q >= 140 => println!("TribeMember"),
    AkkValue::QualityScore(q) if q <  59  => println!("Dead — blocked"),
    AkkValue::Null                         => println!("∅"),
    other                                  => println!("{other}"),  // Display impl
}
```

#### Display format

| Variant | Display |
|---|---|
| `Null` | `∅` |
| `Float(15.3)` | `15.3` |
| `KakiPk([…])` | `kaki:b0b1b2…` (hex) |
| `HijriDate(1447,1,1)` | `1447H-01-01` |
| `Embedding([…32…])` | `emb[32]` |
| `List([…5…])` | `list[5]` |

---

### HOW MUCH — 𒀸 Sovereign Metrics

| Metric | Value |
|---|---|
| Source files | 3 |
| Lines of Rust | ~500 |
| Value variants | **31** |
| Supporting types | 10 |
| Serde format | Tagged JSON + binary visitor |
| External dependencies | serde, serde_json, uuid, chrono |
| `QUALITY_DIVISOR` | **240.0** (ADR-001 — never 255) |

---

## Sovereign Constraints

- `QualityScore` is clamped to `min(value, 240)` on construction (ADR-001)
- `KakiPk` must be exactly 16 bytes — enforced at deserialisation
- `#![forbid(unsafe_code)]`
- `List(Vec<AkkValue>)` is recursive — depth must be bounded by callers
- `Probability` and `Confidence` are `f64` in range `0.0..=1.0` — callers enforce range

---

## Files

```
crates/akkvalue/
├── Cargo.toml          (deps: serde, serde_json, uuid, chrono)
└── src/
    ├── lib.rs           — crate root, AkkTriple, all re-exports
    ├── value.rs         — AkkValue enum (31 variants), Serialize/Deserialize, Display
    └── types.rs         — AkkCoordinate, AkkDate, AkkHijriDate, AkkNationalId,
                           AkkPhone, AkkNameVector, AkkNamePartLabel,
                           PolicyVerdict, CipherAlgorithm, PipelineStatus
```
