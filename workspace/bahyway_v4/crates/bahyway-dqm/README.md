# 𒀭𒂗𒆠 bahyway-dqm — Sovereign Data Quality Management
**Version:** 4.0.0 | **Layer:** 8 — Data Quality | **Status:** Production

---

## What It Solves

Every data pipeline produces scores — but most scores are black boxes. A record
is either "valid" or it is not, with no explanation of which dimension failed,
why, or by how much. The result:

- *"Why was this record rejected?"* — no dimension-level answer
- Silent quality decay that only surfaces as wrong reports weeks later
- Completeness checked in one crate, validity in another, uniqueness nowhere
- No SLA enforcement — no way to know if the data meets enterprise standards

**`bahyway-dqm` solves all six quality dimensions at once** — as a
sovereign, deterministic Data Quality Management engine implementing the full
DAMA-DMBOK quality framework in pure Rust.

| Quality Problem | Sovereign Answer |
|---|---|
| No completeness baseline | Field-presence counting — every field tracked against total declared |
| No validity enforcement | Deterministic Rule Engine + Z-score Welford outlier detection |
| No lineage integrity | FNV-1a Merkle Tree — every record's ancestry verifiable |
| No consistency checks | Cross-field conflict marker 0xFF detection |
| No deduplication awareness | Levenshtein + Jaro-Winkler + Soundex — three algorithms, one score |
| No freshness guarantee | Epoch-based freshness window with linear decay |

---

## Architecture Position

```
EXTERNAL RECORDS                    DQM REPORT
────────────────                    ──────────
ERP records     ─┐              ┌─▶ composite_score (mean of 6)
CRM contacts    ─┤              ├─▶ B11 = score × 240
HR data         ─┤  BAHYWAY-DQM ├─▶ sla_compliant flag
Legacy payloads ─┤  (this crate)├─▶ per-dimension scores
Ingested batch  ─┤              ├─▶ DqmBatchReport
Partner records ─┘              └─▶ compliance_rate()

        ↕ DqmSla enforced at assessment time
        ↕ MerkleTree per record for lineage proof
        ↕ RuleEngine — deterministic, never probabilistic
        ↕ RunningStats — Welford online Z-score
```

---

## Quick Start

```rust
use bahyway_dqm::{DqmEngine, DqmSla};

// 1. Build the engine with an SLA preset
let engine = DqmEngine::new(DqmSla::enterprise());

// 2. Assess a record — Vec of (field_name, value_bytes) pairs
let record = vec![
    ("customer_id".to_string(), b"CUS-001".to_vec()),
    ("full_name".to_string(),   b"Ahmed Al-Rashid".to_vec()),
    ("email".to_string(),       b"ahmed@example.com".to_vec()),
    ("amount".to_string(),      b"1500".to_vec()),
];

let report = engine.assess_record(&record, /*epoch=*/1000);

// 3. Inspect the composite score
println!("B11: {}", report.b11);                          // e.g. 216
println!("SLA compliant: {}", report.sla_compliant);       // true / false
println!("Composite score: {:.3}", report.composite_score);// e.g. 0.900

// 4. Inspect per-dimension scores
println!("Completeness:  {:.3}", report.completeness);
println!("Validity:      {:.3}", report.validity);
println!("Accuracy:      {:.3}", report.accuracy);
println!("Consistency:   {:.3}", report.consistency);
println!("Uniqueness:    {:.3}", report.uniqueness);
println!("Timeliness:    {:.3}", report.timeliness);

// 5. Batch reporting — aggregate over many records
use bahyway_dqm::DqmBatchReport;
let mut batch = DqmBatchReport::new();
for record in records {
    batch.add(engine.assess_record(&record, epoch));
}
println!("Compliance rate: {:.1}%", batch.compliance_rate() * 100.0);
```

---

## Six Quality Dimensions

| Dimension | Algorithm | B11 Impact | DAMA-DMBOK Reference |
|---|---|---|---|
| **Completeness** | Field-presence counting — non-empty fields / total declared fields | Full weight (1/6) | DAMA-DMBOK2 §13.3.1 — *"The degree to which data values are present in the dataset"* |
| **Validity** | Deterministic Rule Engine (5 rule types) + Welford Z-score outlier detection | Full weight (1/6) | DAMA-DMBOK2 §13.3.2 — *"The degree to which data conforms to defined business rules"* |
| **Accuracy** | FNV-1a binary Merkle Tree lineage integrity — inclusion proofs per record | Full weight (1/6) | DAMA-DMBOK2 §13.3.3 — *"The degree to which data correctly describes the real-world object"* |
| **Consistency** | Cross-field conflict marker 0xFF detection — any 0xFF byte signals anomaly | Full weight (1/6) | DAMA-DMBOK2 §13.3.4 — *"The absence of difference when comparing versions of the same data"* |
| **Uniqueness** | Levenshtein Wagner-Fischer similarity + Jaro-Winkler + Soundex American NARA | Full weight (1/6) | DAMA-DMBOK2 §13.3.5 — *"No entity is recorded more than once"* |
| **Timeliness** | Epoch-based freshness window — linear decay from `current_epoch − max_age` | Full weight (1/6) | DAMA-DMBOK2 §13.3.6 — *"Data is available at the time required"* |

---

## B11 Quality Lane Reference

| Lane | B11 Range | Composite Score | Meaning |
|---|---|---|---|
| GEM | ≥ 200 | ≥ 0.833 | Master-data quality — sovereign record |
| TRIBE | ≥ 140 | ≥ 0.583 | Enterprise minimum for production use |
| ACTIVE | ≥ 100 | ≥ 0.417 | Active but below enterprise standard |
| FUZZY | ≥ 60 | ≥ 0.250 | Partial — human review recommended |
| DEAD | < 60 | < 0.250 | Reject — does not meet minimum threshold |

**ADR-001 invariant:** `B11 = (composite_score × 240.0).round() as u8` — **always 240, never 255.**

---

## Module Map

| Module | Purpose |
|---|---|
| `engine` | `DqmEngine` — orchestrates all 6 dimensions per record; `DqmReport` + `DqmBatchReport` |
| `sla` | `DqmSla` — 3 presets (enterprise, master_data, exploratory); per-dimension thresholds |
| `merkle` | `MerkleTree` — FNV-1a binary Merkle tree; `inclusion_proof()`, `verify_proof()` |
| `rules` | `RuleEngine` — 5 deterministic rule types: not_empty, contains_char, numeric_range, min_length, digits_only |
| `stats` | `RunningStats` — Welford online algorithm for mean + variance + Z-score outlier detection |
| `similarity` | `levenshtein_similarity`, `jaro_winkler`, `sounds_like` — three independent string similarity algorithms |

---

## SLA Presets

| Preset | Completeness | Validity | Accuracy | Consistency | Uniqueness | Timeliness |
|---|---|---|---|---|---|---|
| `DqmSla::enterprise()` | 0.98 | 0.95 | 0.90 | 0.95 | 0.99 | 0.90 |
| `DqmSla::master_data()` | 1.00 | 0.99 | 0.99 | 0.99 | 1.00 | 0.95 |
| `DqmSla::exploratory()` | 0.80 | 0.75 | 0.70 | 0.75 | 0.90 | 0.70 |

`DqmSla::custom([c, v, a, co, u, t])` allows arbitrary per-dimension thresholds.

---

## Sovereign Constraints

```
✓ #![forbid(unsafe_code)]       — zero unsafe Rust
✓ No external dependencies       — only bahyway-core; zero third-party crates
✓ QUALITY_DIVISOR = 240.0        — ADR-001, NEVER 255
✓ FNV-1a not SHA-256             — deterministic, sovereign, no crypto dependency
✓ Welford online algorithm        — numerically stable, single-pass, no heap alloc
✓ Wagner-Fischer Levenshtein      — O(min(m,n)) space, exact edit distance
✓ American NARA Soundex          — official phonetic standard, no locale dependency
✓ Rule Engine is deterministic   — same input, same result, every time, any machine
```

---

## Dependency Map

```
bahyway-dqm
    └── bahyway-core    (BahywayError, TribeId, ParticleState)
```

`bahyway-dqm` has **zero third-party dependencies**. It is the lightest
quality-assessment crate in the ecosystem — no crates.io imports, no build
scripts, no proc-macros.

---

## Test Coverage: 76 tests · 1 doc-test · 0 failures · 0 warnings

| Module | Tests | Coverage |
|---|---|---|
| `engine` | 18 | DqmEngine construction, all 6 dimensions, composite score, B11 encoding, SLA compliance, batch reporting, compliance_rate |
| `sla` | 8 | All 3 presets, custom thresholds, per-dimension access, compliance check logic |
| `merkle` | 12 | Tree construction, root hash, inclusion proofs, verify proof, single-leaf, multi-leaf, empty tree |
| `rules` | 14 | All 5 rule types, rule pass/fail, multi-rule evaluation, RuleEngine construction |
| `stats` | 10 | Welford mean, variance, Z-score, outlier detection threshold, single value, running updates |
| `similarity` | 14 | Levenshtein exact, Jaro-Winkler score, Soundex same/different codes, identical strings, empty strings |
| `lib.rs` | 1 | Doc-test: full `DqmEngine::assess_record()` pipeline |
| **Total** | **76 + 1** | **Zero failures** |

---

*𒁾 DUB.SAR — BahyWay.Ecosystem v4.0 | Sovereign Data Quality Management*
