# 𒂗𒆠𒁺 zikru-embed — Manual
**Version:** 4.0.2 | **Layer:** 9.5 — EnkiduLLM Embedding Engine

---

## What It Solves

Text chunks from `enkidullm-ingest` are sequences of tokens. Before the system can compare books, detect plagiarism, or measure conceptual similarity, those tokens must become **dense vectors** — fixed-size numerical representations that encode semantic meaning.

Standard embedding models require PyTorch, ONNX Runtime, or large vocabulary files. None of that is permitted in the BahyWay v4 sovereign stack.

**`zikru-embed` solves the embedding problem in pure Rust:**
- No neural network frameworks
- No BLAS / LAPACK
- No floating-point sort dependencies
- The model itself is a sovereign KAKI particle

The output of `zikru-embed` is a `SectorEmbedding` — a 7-sector vector that encodes which **Hepta sectors** (θ₀–θ₆) a text chunk inhabits. This is the fingerprint used by `enkidullm-audit` to detect plagiarism.

---

## How It Works (Mechanism)

The embedding pipeline has four stages:

```
[TokenUnit stream]
    ↓  model_kaki.rs    — look up token embeddings from QuantizedMatrix
    ↓  attention.rs     — apply TribalFieldAttention (inverse-square field)
    ↓  pooling.rs       — pool attended vectors into 7 Hepta sectors
[SectorEmbedding: unified Vec<f32> + 7 sector means]
```

---

### Stage 1 — Quantized Embedding Lookup (`matrix.rs` + `model_kaki.rs`)

**`QuantizedMatrix`** stores weights as int8 with per-row affine quantization:

```
float_val = (int8_val - zero_point) * scale

Per-row quantization:
  scale      = range / 255        (or |val| / 127 for constant rows)
  zero_point = round(-min/scale - 128)  clamped to [-128, 127]
  q          = round(w/scale + zero)    clamped to [-128, 127]
```

**Constant-row special case:** When all weights in a row are identical (range = 0), the standard formula degenerates. The fix: use `|val| / 127` as scale with zero_point = 0, so dequantize(quantize(c)) = c exactly.

`matvec(vec)` — matrix-vector multiply, dequantizing on the fly:
```
out[r] = Σ_c ( (data[r*cols+c] - zero[r]) * scale[r] * vec[c] )
```
Processes 4 columns per loop iteration (cache locality). Handles remainder columns individually.

**`ZikruEmbedModel`** wraps two `QuantizedMatrix` instances:

| Matrix | Shape | Purpose |
|---|---|---|
| `embedding_weights` | `[vocab_size, embedding_dim]` | Token embedding lookup |
| `sector_weights` | `[7, embedding_dim]` | Classify embedding into 7 sector logits |

**The model IS a KAKI** (tribe=0x10FF LINGUISTIC, role=Parzu). Its nucleus is minted by `KakiMinter`. The model's identity is as sovereign as the data it processes.

**Vocabulary table:** `HashMap<u32, usize>` maps token uuid_hash → embedding row index. This avoids the collision artifacts of hash-modulo indexing. Unknown tokens get the next available row (with wraparound at vocab_size).

**Initialization:** Weights are seeded by FNV-1a pseudo-random values:
```
fnv_pseudo_normal(index, seed) = (fnv1a([index bytes, seed]) / 2^31) - 1.0
```
Produces values in [-1, 1]. Scaled by 0.02 for stable initial training.

---

### Stage 2 — TribalFieldAttention (`attention.rs`)

This is **not** standard Q/K/V transformer attention. It models token interactions as a **physical field** — each token is a particle with position (its embedding), field strength, tribe affinity, and sector.

**`TokenParticle`:**
```
position:       Vec<f32>    ← the token embedding vector
field_strength: f32         ← how strongly this token radiates influence
tribe_affinity: f32         ← cross-tribe damping factor
sector:         u8          ← which Hepta sector (θ₀–θ₆) this token belongs to
```

**Field interaction formula (inverse-square):**

```
weight(i, j) = field_strength[j] / (distance(i,j)² + ε)
             × sector_compatibility(i, j)
             × tribe_affinity[j]

sector_compatibility:
    same sector   → × 1.5   (amplify intra-sector coherence)
    diff sector   → × 0.5   (dampen cross-sector noise)
```

Distance is **Euclidean** between embedding vectors. ε = 1e-6 prevents division by zero for identical positions.

**Output = residual connection:**
```
output[i] = normalize(Σ_j weight(i,j) × position[j])   ← attended
output[i] += position[i]                                ← + original (residual)
```

This preserves the original signal while adding field-weighted neighborhood context.

---

### Stage 3 — HeptaSector Pooling (`pooling.rs`)

The 7 Hepta sectors correspond to the seven token classes from `enkidullm-ingest`:

| Sector | Token Class | Semantic Role |
|---|---|---|
| θ₀ | Word | General lexical content |
| θ₁ | ProperNoun | Named entities, concept labels |
| θ₂ | Operator | Symbolic / mathematical relations |
| θ₃ | Number | Quantitative data |
| θ₄ | Terminal | Boundary / punctuation structure |
| θ₅ | Delimiter | Syntactic grouping |
| θ₆ | Quotation | Cited / attributed content |

**`pool_sectors(particles, attended, fusion_weights) → SectorEmbedding`:**

1. Route each token to its sector based on `particle.sector % 7`
2. Compute mean of all attended vectors within each sector → `sectors[7]`
3. Apply fusion weights to produce the unified embedding:
   ```
   unified[d] = Σ_s (fusion_weights[s] × sectors[s][d])
   ```

**Default fusion weights:**
```
[0.10, 0.15, 0.15, 0.10, 0.10, 0.10, 0.30]
  θ₀    θ₁    θ₂    θ₃    θ₄    θ₅    θ₆
```

θ₆ (Quotation) carries the highest weight (0.30) because cited/attributed content is the **strongest plagiarism fingerprint** — a plagiarist who copies cited passages but strips citations is exposed by θ₆ similarity.

**`abstract_similarity(other)`** compares only the θ₆ sector between two `SectorEmbedding` instances — this is the deepest conceptual fingerprint used by the audit pipeline.

---

### Stage 4 — Contrastive Training (`trainer.rs`)

**`ZikruTrainer`** uses triplet contrastive loss to push similar books together and dissimilar books apart.

**Training sample:**
```
TrainingSample {
    anchor_hashes:   Vec<u32>   ← token hashes of anchor document
    positive_hashes: Vec<u32>   ← token hashes of related document (same tribe)
    negative_hashes: Vec<u32>   ← token hashes of unrelated document
    anchor_tribe:    u16        ← tribe of the anchor
    negative_tribe:  u16        ← tribe of the negative (may differ)
}
```

**Contrastive loss (triplet with margin):**
```
loss = max(0,  dist(anchor, positive) - dist(anchor, negative) + margin)
```

Where distance = Euclidean distance between `unified` embedding vectors.

**`ZikruMomentum`** — per-tribe gradient accumulation:
- Base momentum set at construction (e.g., 0.9)
- Cross-domain tribe (0x1007) gets `0.89 × base_momentum`
  — cross-domain texts require slower adaptation to prevent overfitting

**Gradient application:**
1. Dequantize row from `QuantizedMatrix`
2. Apply: `new_weight = old_weight - lr × gradient`
3. Re-quantize back to int8

This keeps the model quantized throughout training — no full-precision weight copy.

---

## Dependency Map

```
zikru-embed
    ├── enkidullm-core   ← IduState, TribeId constants (0x1001–0x10FF)
    ├── enkidb-kaki      ← Kaki, KakiMinter, KakiRole::Parzu (model IS a KAKI)
    └── bahyway-core     ← TribeId primitive
```

**Dependents:**
```
enkidullm-audit  ← calls embed_chunk() to get SectorEmbedding for plagiarism signal
(app layer)      ← calls train_epoch() to train the model on book pairs
```

---

## Sovereign Constraints

| Rule | Location | Effect |
|---|---|---|
| **No BLAS / no external linear algebra** | `matrix.rs` | `matvec` is hand-written with 4-column cache chunking. No `ndarray`, no `nalgebra`. |
| **No third-party runtime deps** | `Cargo.toml` | Zero external crates beyond internal bahyway/enkidb stack. |
| **Model is a KAKI (role=Parzu)** | `model_kaki.rs` | The embedding model has a sovereign identity. `nucleus.verify_checksum()` passes. Model provenance is traceable. |
| **Vocab table: HashMap, not modulo** | `model_kaki.rs` | `HashMap<u32, usize>` avoids collision artifacts that modulo indexing introduces for similar hash values. |
| **θ₆ highest fusion weight** | `pooling.rs` | Quotation sector is weighted 0.30 (vs 0.10–0.15 for others) because cited content is the primary plagiarism signal. |
| **Constant-row quantization fix** | `matrix.rs` | When a weight row is constant (range=0), use `|val|/127` as scale to preserve the constant under roundtrip. |
| **No unsafe code** | `#![forbid(unsafe_code)]` | All weight access is through safe slice indexing. |

---

---

## Test Coverage

### Depth Levels

| Level | Name | What It Proves |
|---|---|---|
| **L1** | Unit | Single function in isolation — one input, one expected output |
| **L2** | Component | Module behavior through sequential or stateful interaction |
| **L3** | Invariant | A sovereign rule that must never be violated |
| **L4** | End-to-End | Full pipeline across all modules in the crate |

### Test Cases — `matrix.rs` (5 tests)

| Test | Depth | What It Validates |
|---|---|---|
| `dimensions_preserved` | L1 | `from_f32(4, 8, ...)` → `rows=4, cols=8, data.len()=32, scales.len()=4` |
| `matvec_identity` | L1 | Approximate 2×2 identity matrix × [3.0, 5.0] → output ≈ [3.0, 5.0] within quantization error |
| `get_row_roundtrip` | L1 | Quantize row [0.5, -0.5] then dequantize → values within 0.01 of originals |
| `quantization_error_low` | L2 | 1×256 matrix of evenly spaced floats [0, 1] → mean absolute quant error < 0.005 |
| `matvec_larger_matrix` | **L3** | 8×16 all-ones matrix × 16-element all-ones vector → each output ≈ 16.0 (within 0.5) — proves the constant-row special case: `scale = |val|/127` instead of `range/255`, which would collapse to ~0 |

**Critical invariant tested:**
- `matvec_larger_matrix` is the regression test for constant-row quantization. If the fix is removed, all outputs fall to ~1e-8 instead of 16.0. This test must always be at L3.

### Test Cases — `attention.rs` (7 tests)

| Test | Depth | What It Validates |
|---|---|---|
| `cosine_distance_identical_vectors` | L1 | `cosine_distance(v, v)` = 0.0 |
| `cosine_distance_orthogonal_vectors` | L1 | `cosine_distance([1,0], [0,1])` = 1.0 |
| `euclidean_distance_known` | L1 | `euclidean_distance([0,0], [3,4])` = 5.0 (3-4-5 triangle) |
| `attend_empty_no_panic` | L1 | `tribal_field_attend(&[])` returns empty Vec without panic |
| `attend_single_particle_unchanged` | L2 | Single-particle attention: residual connection preserves original embedding (no other particles to interact with) |
| `attend_two_identical_sector_particles` | L2 | Two particles in the same sector: sector_compat = 1.5 applies; output magnitude is larger than single-particle case |
| `field_strength_affects_output` | L2 | High `field_strength` particle produces larger influence on neighbours than low `field_strength` particle at same distance |

### Test Cases — `pooling.rs` (7 tests)

| Test | Depth | What It Validates |
|---|---|---|
| `pool_empty_sequence_returns_zero_embedding` | L1 | Empty particle list → all-zeros `unified` and sector means |
| `cosine_similarity_self_is_one` | L1 | `SectorEmbedding::cosine_similarity(self, self)` = 1.0 |
| `sector_sector_wraps_to_valid_index` | L1 | `particle.sector % 7` always routes to a valid sector [0,6] |
| `sector_counts_correct` | L2 | 3 particles with sectors [0, 0, 1] → `sector_counts[0]=2, sector_counts[1]=1`, all others 0 |
| `unified_embedding_non_zero_when_tokens_present` | L2 | Non-empty particle list produces a `unified` vector with at least one non-zero element |
| `fusion_weights_affect_unified` | L2 | Changing `fusion_weights` to boost θ₆ (Quotation) changes `unified` output measurably |
| `abstract_sector_similarity_identical` | **L3** | `embedding.abstract_similarity(embedding)` = 1.0 — self-similarity of the θ₆ sector is always maximal, proving the plagiarism fingerprint is stable |

### Test Cases — `model_kaki.rs` (7 tests)

| Test | Depth | What It Validates |
|---|---|---|
| `pseudo_normal_bounded` | L1 | `fnv_pseudo_normal(i, 42)` ∈ [-1.0, 1.0] for 1000 random indices |
| `field_strength_clamped` | L1 | `field_strength(100)` clamps to index 7 and returns a positive value |
| `embed_token_returns_correct_dim` | L2 | `embed_token(hash)` returns Vec of length `embedding_dim` |
| `vocab_index_stable` | L2 | Same uuid_hash returns the same row index on two consecutive calls |
| `vocab_index_distinct_for_different_hashes` | L2 | Two different uuid_hashes → two different row indices (no trivial collision for small vocab) |
| `classify_sector_in_range` | **L3** | `classify_sector(embedding)` ∈ [0, 6] — validates that `sector_weights` has shape [7, dim] so `matvec(embedding)` returns 7 logits and argmax is in range. If shape were [dim, 7], `vec.len() ≠ cols` and this panics. |
| `model_init_nucleus_valid` | **L3** | `ZikruEmbedModel::init(...)` nucleus passes `verify_checksum()` — the model IS a sovereign KAKI (Parzu, tribe 0x10FF) |

**Critical invariant tested:**
- `classify_sector_in_range` is the regression test for the `sector_weights` shape bug. If `from_f32(dim, 7, ...)` is used instead of `from_f32(7, dim, ...)`, this test panics at `assert_eq!(vec.len(), self.cols)` in `matvec`. Must remain L3.

### Test Cases — `trainer.rs` (6 tests)

| Test | Depth | What It Validates |
|---|---|---|
| `embed_chunk_empty_no_panic` | L1 | `embed_chunk(model, &[])` returns a zero SectorEmbedding without panic |
| `zikru_momentum_cross_domain_lower` | L2 | `tribe_momentum(0x1007)` < `tribe_momentum(0x1001)` — cross-domain tribe gets 0.89× base momentum |
| `embed_chunk_returns_correct_dim` | L2 | `embed_chunk(model, hashes)` returns `unified.len() == embedding_dim` |
| `compute_loss_non_negative` | L2 | `compute_contrastive_loss()` returns loss ≥ 0.0 for any sample |
| `epoch_counter_increments` | L2 | `model.orbit.training_epochs` starts at 0, increments to 1 after first epoch, 2 after second |
| `train_epoch_reduces_loss_over_repeated_samples` | **L4** | Two consecutive `train_epoch()` calls both return finite `mean_loss` — validates the full training loop: embed → loss → gradient → dequantize → update → re-quantize |

**Total: 32 tests** — L1: 9 · L2: 16 · L3: 4 · L4: 3

### Gaps & Future Test Targets

| Area | Missing Coverage | Suggested Test |
|---|---|---|
| `trainer.rs` | Gradient actually updates weights (loss direction) not verified | Add test: train 10 epochs on identical anchor/positive, assert mean_loss decreases |
| `attention.rs` | Tribe affinity damping (`tribe_affinity < 1.0`) not directly tested | Add test: particle with tribe_affinity=0.1 produces smaller influence than particle with 1.0 |
| `pooling.rs` | θ₆ fusion weight dominance not proven quantitatively | Add test: text with all Quotation tokens → unified embedding closer to θ₆ sector than others |
| `model_kaki.rs` | Model serialization (dequantize → update → re-quantize roundtrip) untested in isolation | Add test: quantize, update one row, re-quantize, verify changed row differs from original |

---

*"The embedding is not a compression. It is a sovereign fingerprint — traceable, comparable, immutable in meaning."*
