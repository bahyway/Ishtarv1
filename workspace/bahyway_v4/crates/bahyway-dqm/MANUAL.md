# 𒀭𒂗𒆠 bahyway-dqm — Sovereign Manual
**Version:** 4.0.0 | **Layer:** 8 — Data Quality Management
**W5H2 Transparency Framework** | Author: Bahaa Fadam — BahyWay Sovereign Ecosystem

---

## W5H2 — Crate Overview

| Symbol | Question | Answer |
|--------|----------|--------|
| **Who** | Who builds it? Who uses it? | Built by Bahaa Fadam. Used by `bahyway-fabric`, `bahyway-server`, `data-cleansing-station`, and any crate that must assess record quality across the full DAMA-DMBOK six-dimension framework. |
| **What** | What does it do? | Provides a sovereign, deterministic Data Quality Management engine that assesses any record across six DAMA-DMBOK quality dimensions (Completeness, Validity, Accuracy, Consistency, Uniqueness, Timeliness), computes a composite score, encodes it as B11 (×240, ADR-001), and evaluates SLA compliance. |
| **When** | When is it invoked? | At quality-gate time: after a record has been extracted and structured (by `bahyway-fabric` or `data-structure-station`), before the record is accepted into the sovereign pipeline or committed to permanent storage. Also invoked in batch mode over large record sets to compute compliance rates. |
| **Where** | Where does it live? | `crates/bahyway-dqm/` — Layer 8 (Data Quality), sitting alongside `bahyway-fabric`, `adad-gate`, and `vgca-validation`. |
| **Why** | Why does it exist? | Enterprise data pipelines validate data piecemeal — completeness checked in one place, validity in another, uniqueness nowhere. `bahyway-dqm` is the sovereign answer: one engine, six dimensions, one score, one SLA verdict — implemented in pure Rust with no external dependencies. |
| **How** | How does it work? | `DqmEngine::assess_record(record, epoch)` evaluates all six dimensions in sequence. Completeness counts non-empty fields. Validity applies the `RuleEngine` and `RunningStats` Z-score outlier detection. Accuracy verifies lineage integrity via a `MerkleTree` of FNV-1a hashes. Consistency scans for cross-field 0xFF conflict markers. Uniqueness averages Levenshtein, Jaro-Winkler, and Soundex similarity scores. Timeliness computes a linear freshness decay from the current epoch. The composite score is the arithmetic mean of all six; B11 = (score × 240).round(). |
| **How Much** | How much does it deliver? | **76 unit tests** + **1 doc-test** · **0 failures** · **0 warnings** · 6 modules · 3 string-similarity algorithms · 5 deterministic rule types · 1 Welford online statistics engine · ~850 lines of sovereign Rust |

---

## What It Solves — The Six Quality Gaps

Enterprise data quality is not one problem — it is six independent problems that
must all be solved at once. This crate eliminates each one structurally.

### Gap 1: No Completeness Baseline
**Solution: Field-presence counting**

Every field in the record is tested for emptiness. A field is complete when its
value byte-slice is non-empty. The completeness score is `non_empty / total`.
This is the simplest dimension but the most commonly neglected: most pipelines
assume fields are present without verifying.

```
record = [("name", b"Ahmed"), ("email", b""), ("id", b"001")]
completeness = 2 / 3 = 0.667
```

### Gap 2: No Validity Enforcement
**Solution: Rule Engine + Welford Z-score**

Validity has two components working together:
1. **Rule Engine** — deterministic rule evaluation: `not_empty`, `contains_char`,
   `numeric_range`, `min_length`, `digits_only`. Rules are composable and always
   produce the same result for the same input.
2. **RunningStats** — Welford online algorithm for outlier detection. After N
   observations, any value more than 2.5 standard deviations from the running mean
   is flagged as a validity anomaly. This catches statistical outliers without
   requiring a full dataset scan.

The validity score is the pass-rate of all rules applied, modulated by the
outlier flag.

### Gap 3: No Lineage Integrity
**Solution: FNV-1a Merkle Tree**

Every record's fields are hashed with FNV-1a and assembled into a binary Merkle
tree. The root hash is a deterministic fingerprint of the record's complete content.
`MerkleTree::inclusion_proof(index)` produces a sibling-path proof that can be
independently verified by any party with the root hash. This makes the accuracy
dimension tamper-evident without any external crypto library.

### Gap 4: No Consistency Checks
**Solution: Cross-field 0xFF conflict marker detection**

The byte value `0xFF` is used as the sovereign conflict sentinel. Any field whose
value bytes contain `0xFF` is flagged as a consistency conflict — indicating
that this field was marked as conflicting by an upstream station. The consistency
score is `(fields without 0xFF) / total_fields`. This protocol is deterministic,
byte-level, and requires no schema knowledge.

### Gap 5: No Deduplication Awareness
**Solution: Three-algorithm string similarity**

Uniqueness is computed as the mean of three independent string similarity metrics:

1. **Levenshtein (Wagner-Fischer)** — exact edit-distance similarity, O(m×n)
   time, O(min(m,n)) space. Catches near-duplicates with different spellings.
2. **Jaro-Winkler** — prefix-weighted transposition similarity. Particularly
   effective for personal names where first characters are more likely to be correct.
3. **Soundex (American NARA standard)** — phonetic equivalence. Catches
   homophones and pronunciation variants (e.g., "Smith" ≡ "Smythe").

The uniqueness score is `1.0 - mean(lev, jw, soundex)` for the primary key field
against a reference value. Higher uniqueness score = lower similarity = fewer
near-duplicates.

### Gap 6: No Freshness Guarantee
**Solution: Epoch-based linear freshness decay**

Each record carries an `epoch` timestamp. The timeliness score is:

```
if current_epoch - record_epoch >= max_age_epochs:
    timeliness = 0.0
else:
    timeliness = 1.0 - (age / max_age_epochs)
```

This produces a linear decay from 1.0 (brand new) to 0.0 (at or beyond the
freshness window). The default max_age is 1000 epochs.

---

## Module Reference

### `engine.rs` — DqmEngine, DqmReport, DqmBatchReport

The central orchestrator. One engine per SLA configuration.

**Key types:**

| Type | Purpose |
|---|---|
| `DqmEngine` | Holds a `DqmSla`; orchestrates all 6 dimensions per `assess_record()` call |
| `DqmReport` | Single-record result: 6 per-dimension scores + composite + B11 + sla_compliant |
| `DqmBatchReport` | Accumulates multiple `DqmReport`s; `compliance_rate()` = compliant / total |

**Core API:**

```rust
// Construct
let engine = DqmEngine::new(DqmSla::enterprise());

// Assess one record
let record: Vec<(String, Vec<u8>)> = vec![
    ("invoice_id".to_string(), b"INV-2026-001".to_vec()),
    ("amount".to_string(),     b"99500".to_vec()),
    ("status".to_string(),     b"approved".to_vec()),
];
let report: DqmReport = engine.assess_record(&record, epoch);

// DqmReport fields
report.completeness      // f64 in [0.0, 1.0]
report.validity          // f64 in [0.0, 1.0]
report.accuracy          // f64 in [0.0, 1.0]
report.consistency       // f64 in [0.0, 1.0]
report.uniqueness        // f64 in [0.0, 1.0]
report.timeliness        // f64 in [0.0, 1.0]
report.composite_score   // arithmetic mean of all 6
report.b11               // (composite_score × 240.0).round() as u8
report.sla_compliant     // true iff every dimension ≥ its SLA threshold

// Batch reporting
let mut batch = DqmBatchReport::new();
for (rec, ep) in records_with_epochs {
    batch.add(engine.assess_record(&rec, ep));
}
let rate: f64 = batch.compliance_rate(); // compliant_count / total_count
```

**B11 encoding invariant (ADR-001):**

```
B11 = (composite_score × 240.0).round() as u8

score = 1.000  →  B11 = 240  (perfect sovereign record)
score = 0.833  →  B11 = 200  (GEM threshold)
score = 0.583  →  B11 = 140  (TRIBE threshold — enterprise minimum)
score = 0.417  →  B11 = 100  (ACTIVE threshold)
score = 0.250  →  B11 =  60  (FUZZY threshold)
score < 0.250  →  B11 <  60  (DEAD — reject)
```

NEVER use 255 as the divisor. ALWAYS use 240. This is ADR-001 and it is
enforced ecosystem-wide.

---

### `sla.rs` — DqmSla

Per-dimension SLA thresholds.

**Presets:**

```rust
DqmSla::enterprise()   // [0.98, 0.95, 0.90, 0.95, 0.99, 0.90]
DqmSla::master_data()  // [1.00, 0.99, 0.99, 0.99, 1.00, 0.95]
DqmSla::exploratory()  // [0.80, 0.75, 0.70, 0.75, 0.90, 0.70]
DqmSla::custom([c, v, a, co, u, t]) // arbitrary thresholds
```

**SLA comparison table:**

| Dimension | enterprise | master_data | exploratory | Notes |
|---|---|---|---|---|
| Completeness | 0.98 | 1.00 | 0.80 | master_data demands 100% completeness |
| Validity | 0.95 | 0.99 | 0.75 | enterprise allows 5% rule failures |
| Accuracy | 0.90 | 0.99 | 0.70 | most lenient in enterprise |
| Consistency | 0.95 | 0.99 | 0.75 | 0xFF conflict tolerance |
| Uniqueness | 0.99 | 1.00 | 0.90 | near-zero duplicates for both prod presets |
| Timeliness | 0.90 | 0.95 | 0.70 | real-time data freshness |

**Compliance rule:** `sla_compliant = true` iff ALL six dimensions meet their
respective threshold. A single dimension below threshold fails the entire record.

---

### `merkle.rs` — MerkleTree

FNV-1a binary Merkle tree for lineage integrity.

**Algorithm — FNV-1a:**

```
FNV_OFFSET_BASIS = 0xcbf29ce484222325u64
FNV_PRIME        = 0x100000001b3u64

fnv1a(data):
    hash = FNV_OFFSET_BASIS
    for byte in data:
        hash ^= byte as u64
        hash = hash.wrapping_mul(FNV_PRIME)
    return hash
```

FNV-1a was chosen over SHA-256 for three sovereign reasons:
1. **No external dependency** — implemented in 6 lines of Rust
2. **Deterministic** — same bytes → same hash on any platform, any architecture
3. **Non-cryptographic** — not trying to be a security primitive; trying to be a
   content fingerprint for lineage verification

**Tree construction:**

```rust
let leaves = vec![b"field1_value".to_vec(), b"field2_value".to_vec()];
let tree = MerkleTree::new(leaves);
let root = tree.root(); // FNV-1a hash of the full record

// Inclusion proof — sibling path from leaf to root
let proof = tree.inclusion_proof(0); // proof for leaf index 0
let verified = MerkleTree::verify_proof(&proof, root, 0, b"field1_value");
```

**Internal structure:**

```
       root = h(h01 || h23)
      /                    \
   h01 = h(h0 || h1)     h23 = h(h2 || h3)
   /         \            /         \
h0=fnv(L0) h1=fnv(L1) h2=fnv(L2) h3=fnv(L3)
```

Parent nodes are computed as `fnv1a(left_bytes || right_bytes)`. Odd-numbered
levels duplicate the last node to maintain a binary tree invariant.

---

### `rules.rs` — RuleEngine

Deterministic rule evaluation. Every rule is a pure function: same input → same
output, guaranteed.

**Five rule types:**

| Rule | Signature | Passes when |
|---|---|---|
| `rule_not_empty` | `(value: &[u8]) -> bool` | value is non-empty |
| `rule_contains_char` | `(value: &[u8], ch: u8) -> bool` | value contains byte `ch` |
| `rule_numeric_range` | `(value: &[u8], min: f64, max: f64) -> bool` | value parses as f64 within [min, max] |
| `rule_min_length` | `(value: &[u8], min: usize) -> bool` | value.len() ≥ min |
| `rule_digits_only` | `(value: &[u8]) -> bool` | all bytes are ASCII digits (0x30–0x39) |

**RuleEngine construction and evaluation:**

```rust
use bahyway_dqm::rules::{RuleEngine, Rule};

let mut engine = RuleEngine::new();

// Register rules for a field
engine.add_rule(Rule::NotEmpty { field: "invoice_id" });
engine.add_rule(Rule::MinLength { field: "description", min: 5 });
engine.add_rule(Rule::NumericRange { field: "amount", min: 0.0, max: 1_000_000.0 });
engine.add_rule(Rule::ContainsChar { field: "email", ch: b'@' });
engine.add_rule(Rule::DigitsOnly { field: "postal_code" });

// Evaluate — returns (passed, total) rule counts
let record = vec![("amount".to_string(), b"750".to_vec())];
let (passed, total) = engine.evaluate(&record);
let validity_score = passed as f64 / total as f64;
```

**Sovereign invariant:** Rules are stateless closures. The `RuleEngine` holds
no mutable state between evaluations. Calling `evaluate()` twice on the same
record always returns the same result.

---

### `stats.rs` — RunningStats

Welford online algorithm for incremental mean, variance, and Z-score.

**Algorithm — Welford (1962):**

```
On each new value x:
    n     += 1
    delta  = x - mean
    mean  += delta / n
    delta2 = x - mean
    M2    += delta × delta2

variance = M2 / (n - 1)   [sample variance]
stddev   = sqrt(variance)
z_score  = |x - mean| / stddev
```

Welford's algorithm is numerically stable — it avoids the catastrophic
cancellation that afflicts the naive `E[x²] - E[x]²` formula. It computes
mean and variance in a single pass with O(1) space.

**API:**

```rust
use bahyway_dqm::stats::RunningStats;

let mut stats = RunningStats::new();
for value in data_stream {
    stats.update(value);
}

let mean     = stats.mean();
let variance = stats.variance();
let stddev   = stats.stddev();

// Z-score outlier detection (threshold = 2.5 standard deviations)
let z = stats.z_score(new_value);
let is_outlier = z > 2.5;
```

**Integration with Validity dimension:**

`DqmEngine` maintains a `RunningStats` instance per numeric field. When a new
record arrives, the engine updates the stats and checks whether any numeric field
value is an outlier (Z > 2.5). Outlier fields penalise the validity score.

---

### `similarity.rs` — String Similarity Algorithms

Three independent algorithms covering edit distance, transposition, and phonetics.

#### Levenshtein Similarity (Wagner-Fischer)

Edit-distance-based similarity for detecting near-duplicate spellings.

**Algorithm:**

```
levenshtein_distance(s, t):
    dp[i][j] = min edit operations to transform s[0..i] into t[0..j]
    transitions: insertion (cost 1), deletion (cost 1), substitution (cost 1 if s[i]≠t[j])

levenshtein_similarity(s, t):
    d = levenshtein_distance(s, t)
    return 1.0 - (d / max(len(s), len(t)))
```

**Space optimisation:** Wagner-Fischer with two-row rolling array → O(min(m,n)) space.

**Examples:**
```
("Ahmed", "Ahmad")   → distance=1 → similarity=0.800
("Ali",   "Ali")     → distance=0 → similarity=1.000
("Smith", "Jones")   → distance=5 → similarity=0.000
```

#### Jaro-Winkler

Prefix-weighted transposition similarity. Optimal for person names where the
first few characters are most likely to be correct.

**Algorithm:**

```
jaro(s1, s2):
    match_window = max(len(s1), len(s2)) / 2 - 1
    matches = chars within match_window that appear in both strings (in order)
    transpositions = half the count of matched chars in different order
    jaro = (matches/|s1| + matches/|s2| + (matches-transpositions)/matches) / 3

jaro_winkler(s1, s2):
    prefix = length of common prefix (capped at 4)
    jaro_winkler = jaro + prefix × 0.1 × (1 - jaro)
```

**Examples:**
```
("Ahmed", "Ahmad")      → jaro=0.893 → jw=0.915
("Mohammed", "Muhammed")→ jaro=0.944 → jw=0.966
("Ali", "Ali")          → jaro=1.000 → jw=1.000
```

#### Soundex (American NARA Standard)

Phonetic encoding — groups words that sound similar to the same 4-character code.

**Algorithm (American National Archives and Records Administration):**

```
soundex(s):
    1. Retain the first letter of s
    2. Remove all occurrences of H and W
    3. Map remaining letters to codes:
       B,F,P,V     → 1
       C,G,J,K,Q,S,X,Z → 2
       D,T         → 3
       L           → 4
       M,N         → 5
       R           → 6
       A,E,I,O,U,Y → 0 (removed)
    4. Remove adjacent duplicate codes
    5. Pad to 4 characters with zeros; truncate to 4

sounds_like(s1, s2):
    return soundex(s1) == soundex(s2)
```

**Examples:**
```
"Smith"  → S530    "Smythe" → S530  → sounds_like = true
"Robert" → R163    "Rupert" → R163  → sounds_like = true
"Ahmed"  → A530    "Ahmad"  → A530  → sounds_like = true
"Ali"    → A400    "Aly"    → A400  → sounds_like = true
"Smith"  → S530    "Jones"  → J520  → sounds_like = false
```

#### Uniqueness Score Computation

```rust
use bahyway_dqm::similarity::{levenshtein_similarity, jaro_winkler, sounds_like};

let s1 = "Ahmed Al-Rashid";
let s2 = "Ahmed Al-Rasheed";

let lev  = levenshtein_similarity(s1, s2);  // 0.875
let jw   = jaro_winkler(s1, s2);            // 0.944
let sl   = if sounds_like(s1, s2) { 1.0f64 } else { 0.0f64 }; // 1.0

// Uniqueness = 1 − mean similarity (high similarity → low uniqueness score)
let mean_sim = (lev + jw + sl) / 3.0;
let uniqueness = 1.0 - mean_sim;
// If uniqueness < SLA threshold → near-duplicate detected
```

---

## Integration Patterns

### Pattern 1: Single record assessment in the pipeline

```rust
use bahyway_dqm::{DqmEngine, DqmSla};

let engine = DqmEngine::new(DqmSla::enterprise());

// Called from bahyway-fabric Stage::Validate
fn quality_gate(record: &[(String, Vec<u8>)], epoch: u32) -> Result<u8, String> {
    let report = engine.assess_record(record, epoch);
    if report.sla_compliant {
        Ok(report.b11)
    } else {
        Err(format!(
            "SLA violation — B11={} composite={:.3} (completeness={:.3}, validity={:.3})",
            report.b11, report.composite_score, report.completeness, report.validity
        ))
    }
}
```

### Pattern 2: Batch compliance reporting

```rust
use bahyway_dqm::{DqmEngine, DqmBatchReport, DqmSla};

let engine = DqmEngine::new(DqmSla::enterprise());
let mut batch = DqmBatchReport::new();

for (record, epoch) in incoming_records {
    batch.add(engine.assess_record(&record, epoch));
}

let rate = batch.compliance_rate();
if rate < 0.95 {
    eprintln!("ALERT: batch compliance {:.1}% below 95% threshold", rate * 100.0);
}
println!("Batch: {}/{} records SLA-compliant", batch.compliant_count(), batch.total_count());
```

### Pattern 3: External uniqueness scoring against a known reference

```rust
use bahyway_dqm::similarity::{levenshtein_similarity, jaro_winkler, sounds_like};

// Deduplication probe — compare incoming record's primary key against known golden records
fn is_near_duplicate(incoming: &str, golden_records: &[String]) -> Option<String> {
    for golden in golden_records {
        let lev = levenshtein_similarity(incoming, golden);
        let jw  = jaro_winkler(incoming, golden);
        let sl  = if sounds_like(incoming, golden) { 1.0 } else { 0.0 };
        let similarity = (lev + jw + sl) / 3.0;
        if similarity > 0.92 {
            return Some(golden.clone()); // near-duplicate detected
        }
    }
    None
}
```

### Pattern 4: Merkle lineage proof for audit

```rust
use bahyway_dqm::merkle::MerkleTree;

// At ingestion: build tree and store root
let leaves: Vec<Vec<u8>> = record.iter()
    .map(|(_, v)| v.clone())
    .collect();
let tree = MerkleTree::new(leaves);
let root = tree.root();
// Store root alongside record in Journal

// Later: verify record was not tampered
let proof = tree.inclusion_proof(field_index);
let verified = MerkleTree::verify_proof(&proof, root, field_index, &field_value);
assert!(verified, "Lineage integrity violation: record was modified after ingestion");
```

### Pattern 5: Running stats for anomaly detection across a stream

```rust
use bahyway_dqm::stats::RunningStats;

let mut amount_stats = RunningStats::new();

for record in live_stream {
    if let Some(amount_bytes) = record.get("amount") {
        if let Ok(s) = std::str::from_utf8(amount_bytes) {
            if let Ok(v) = s.parse::<f64>() {
                amount_stats.update(v);
                let z = amount_stats.z_score(v);
                if z > 2.5 {
                    eprintln!("VALIDITY ALERT: amount={} is {:.1}σ from mean {:.2}", v, z, amount_stats.mean());
                }
            }
        }
    }
}
```

---

## Test Coverage

### Depth Levels

| Level | Name | What It Proves |
|---|---|---|
| **L1** | Unit | Single function — one input, one expected output |
| **L2** | Component | Module behavior through sequential or stateful interaction |
| **L3** | Invariant | A sovereign rule that must never be violated |
| **L4** | End-to-End | Full assess_record() pipeline across all 6 dimensions |

### Test Cases — `engine` (18 tests)

| Test | Depth | What It Validates |
|---|---|---|
| `assess_record_completeness_all_present` | L1 | All non-empty fields → completeness = 1.0 |
| `assess_record_completeness_partial` | L1 | 2/3 non-empty → completeness ≈ 0.667 |
| `assess_record_completeness_empty` | L1 | All empty → completeness = 0.0 |
| `assess_record_validity_all_pass` | L1 | All valid → validity = 1.0 |
| `assess_record_validity_outlier_penalised` | L2 | Z-score > 2.5 → validity < 1.0 |
| `assess_record_accuracy_merkle_consistent` | L1 | Same record twice → same root hash |
| `assess_record_consistency_no_conflict` | L1 | No 0xFF bytes → consistency = 1.0 |
| `assess_record_consistency_conflict_marker` | L1 | 0xFF in field → consistency < 1.0 |
| `assess_record_uniqueness_identical` | L1 | Identical key values → uniqueness = 0.0 |
| `assess_record_uniqueness_dissimilar` | L1 | Completely different keys → uniqueness = 1.0 |
| `assess_record_timeliness_fresh` | L1 | epoch=0, current=0 → timeliness = 1.0 |
| `assess_record_timeliness_decayed` | L1 | age = max_age → timeliness = 0.0 |
| `composite_score_is_mean_of_six` | **L3** | composite = (c+v+a+co+u+t)/6.0 always |
| `b11_uses_divisor_240` | **L3** | B11 = (score×240).round() — never 255 |
| `sla_compliant_all_pass` | L2 | All dimensions above thresholds → sla_compliant = true |
| `sla_compliant_one_fails` | **L3** | One dimension below threshold → sla_compliant = false |
| `batch_report_compliance_rate` | L2 | 3 compliant + 1 non-compliant → rate = 0.75 |
| `full_pipeline_doctest` | **L4** | Complete assess_record() + batch_report pattern |

### Test Cases — `sla` (8 tests)

| Test | Depth | What It Validates |
|---|---|---|
| `enterprise_preset_values` | L1 | `[0.98, 0.95, 0.90, 0.95, 0.99, 0.90]` exact |
| `master_data_preset_values` | L1 | `[1.00, 0.99, 0.99, 0.99, 1.00, 0.95]` exact |
| `exploratory_preset_values` | L1 | `[0.80, 0.75, 0.70, 0.75, 0.90, 0.70]` exact |
| `custom_sla_round_trips` | L1 | Custom array preserved unchanged |
| `compliance_check_all_pass` | L2 | All 6 scores above threshold → Ok(()) |
| `compliance_check_completeness_fails` | L2 | completeness below threshold → Err |
| `compliance_check_uniqueness_fails` | L2 | uniqueness below threshold → Err |
| `sla_per_dimension_access` | L1 | Index accessors return correct preset values |

### Test Cases — `merkle` (12 tests)

| Test | Depth | What It Validates |
|---|---|---|
| `empty_tree_root_is_zero` | L1 | No leaves → root = 0 |
| `single_leaf_root_is_leaf_hash` | L1 | One leaf → root = fnv1a(leaf) |
| `two_leaves_root_is_parent_hash` | L1 | root = fnv1a(h0 \|\| h1) |
| `four_leaves_correct_root` | L2 | Full binary tree root is reproducible |
| `root_changes_when_leaf_changes` | **L3** | Mutating any leaf changes root — tamper detection |
| `root_is_deterministic` | **L3** | Same leaves → same root, always |
| `inclusion_proof_length` | L1 | Proof length = ceil(log2(n)) |
| `inclusion_proof_verify_passes` | L2 | Correct value → verify = true |
| `inclusion_proof_verify_fails_wrong_value` | **L3** | Wrong value → verify = false — integrity enforced |
| `inclusion_proof_verify_fails_wrong_index` | **L3** | Wrong index → verify = false |
| `odd_leaf_count_handled` | L2 | 3 leaves → last leaf duplicated, tree is valid |
| `large_tree_verify` | L2 | 16 leaves — all inclusion proofs verify correctly |

### Test Cases — `rules` (14 tests)

| Test | Depth | What It Validates |
|---|---|---|
| `rule_not_empty_passes_nonempty` | L1 | b"value" → true |
| `rule_not_empty_fails_empty` | L1 | b"" → false |
| `rule_contains_char_found` | L1 | b'@' in email → true |
| `rule_contains_char_not_found` | L1 | b'@' not in plain text → false |
| `rule_numeric_range_in_bounds` | L1 | "500" in [0, 1000] → true |
| `rule_numeric_range_at_bounds` | L1 | "0" and "1000" → both true |
| `rule_numeric_range_out_of_bounds` | L1 | "1001" in [0, 1000] → false |
| `rule_numeric_range_non_numeric` | L1 | "abc" → false (not parseable) |
| `rule_min_length_meets` | L1 | len=5 min=5 → true |
| `rule_min_length_fails` | L1 | len=4 min=5 → false |
| `rule_digits_only_all_digits` | L1 | b"12345" → true |
| `rule_digits_only_mixed` | L1 | b"123a5" → false |
| `engine_evaluates_multiple_rules` | L2 | 3 rules, 2 pass, 1 fail → (2, 3) |
| `engine_evaluates_empty_ruleset` | L2 | 0 rules → (0, 0) — no division by zero |

### Test Cases — `stats` (10 tests)

| Test | Depth | What It Validates |
|---|---|---|
| `single_value_mean_is_value` | L1 | update(5.0); mean() == 5.0 |
| `two_values_mean_is_average` | L1 | update(4.0); update(6.0); mean() == 5.0 |
| `variance_zero_for_equal_values` | L1 | All same → variance = 0.0 |
| `variance_known_dataset` | L2 | [2,4,4,4,5,5,7,9] → variance = 4.0 (Welford) |
| `stddev_is_sqrt_variance` | L2 | stddev = sqrt(variance) |
| `z_score_at_mean_is_zero` | L1 | z_score(mean) == 0.0 |
| `z_score_at_stddev_is_one` | L1 | z_score(mean + stddev) == 1.0 |
| `outlier_detected_above_threshold` | **L3** | z > 2.5 → is_outlier flag |
| `non_outlier_below_threshold` | **L3** | z < 2.5 → not flagged |
| `running_update_100_values` | L2 | 100 values → mean converges to expected |

### Test Cases — `similarity` (14 tests)

| Test | Depth | What It Validates |
|---|---|---|
| `levenshtein_identical_strings` | L1 | similarity = 1.0 |
| `levenshtein_single_edit` | L1 | "kitten"→"sitten": similarity = 5/6 |
| `levenshtein_completely_different` | L1 | similarity ≈ 0.0 |
| `levenshtein_empty_strings` | L1 | both empty → 1.0; one empty → 0.0 |
| `jaro_winkler_identical` | L1 | score = 1.0 |
| `jaro_winkler_known_pair` | L1 | "MARTHA"/"MARHTA" → score ≈ 0.961 |
| `jaro_winkler_prefix_boost` | L2 | Common prefix increases score vs jaro alone |
| `jaro_winkler_empty` | L1 | empty strings → 0.0 |
| `soundex_identical_words` | L1 | "Robert"/"Robert" → same code |
| `soundex_homophones` | L1 | "Smith"/"Smythe" → same code S530 |
| `soundex_different_words` | L1 | "Smith"/"Jones" → different codes |
| `sounds_like_true` | L1 | "Robert"/"Rupert" → true |
| `sounds_like_false` | L1 | "Smith"/"Jones" → false |
| `sounds_like_arabic_names` | L2 | "Ahmed"/"Ahmad" → same Soundex code |

### Doc-Test — `lib.rs` (1 test)

| Test | What It Validates |
|---|---|
| Full `DqmEngine::assess_record()` | Complete API surface: construct engine, assess a 4-field record, read B11, check SLA compliance, build batch report, call compliance_rate() |

---

## Sovereign Constraints

| Rule | Location | Enforcement |
|---|---|---|
| **No third-party runtime deps** | `Cargo.toml` | Only `bahyway-core`; no serde, no rand, no regex, no unicode-segmentation |
| **No unsafe code** | Implicit | Pure safe Rust throughout — no pointer arithmetic, no transmutes |
| **QUALITY_DIVISOR = 240.0** | `engine.rs` | `b11 = (composite_score * 240.0).round() as u8` — asserted in test suite |
| **FNV-1a not SHA-256** | `merkle.rs` | Sovereign non-cryptographic hash — no ring, no sha2 dependency |
| **Welford not naive variance** | `stats.rs` | Numerically stable online algorithm — no catastrophic cancellation |
| **Wagner-Fischer not naive Levenshtein** | `similarity.rs` | O(min(m,n)) space — no O(m×n) allocation |
| **American NARA Soundex** | `similarity.rs` | Official phonetic standard — reproducible on any locale |
| **Rule Engine is pure** | `rules.rs` | No global state, no side effects, no randomness — deterministic always |

---

## Future Extensions (Roadmap)

| Extension | Layer Integration | Priority |
|---|---|---|
| Persist `DqmBatchReport` to Journal as `EventKaki` sequence after each pipeline run | `enkidb-journal` | High |
| HeptaScript `.hepta` query over DQM reports — "show all records with completeness < 0.90" | `heptascript` | High |
| Per-field rule registration via `.akk` policy files | `aaol` | Medium |
| Streaming Z-score alerts → `alert-engine` integration | `alert-engine` | Medium |
| Dubsar visualizer — live quality dimension bar chart | `dubsar-visualizer` | Medium |
| BLAKE3 as an optional sovereign hash backend (post-quantum ADR) | `kupru` | Low |

---

*"Quality is not a gate at the end. It is a sovereign dimension at every step."*

*𒁾 DUB.SAR — BahyWay.Ecosystem v4.0 | 2026-06-04*
