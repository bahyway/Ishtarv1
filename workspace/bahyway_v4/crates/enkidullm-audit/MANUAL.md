# 𒂗𒆠𒁺 enkidullm-audit — Manual
**Version:** 4.0.2 | **Layer:** 9.5 — EnkiduLLM Plagiarism Probe & Evidence Journal

---

## What It Solves

Plagiarism is not similarity. Two books can share concepts because they draw from the same field. The offense occurs when:

> **A book uses another book's concepts without acknowledging the source.**

The gap between *what was taken* and *what was credited* is the plagiarism signal. Measuring that gap requires two independent analyses: how much of the source's knowledge graph appears in the suspect, and how much of the source's authorship is cited.

Without a sovereign audit pipeline, findings would be opinions — asserted, mutable, and legally fragile. With `enkidullm-audit`, findings become **immutable evidence tablets**: cryptographically sealed, append-only, independently verifiable.

**`enkidullm-audit` solves plagiarism detection as a sovereign forensic protocol.**

---

## How It Works (Mechanism)

The audit pipeline has four stages:

```
[suspect book triples]  +  [source book triples]  +  [suspect's citation list]
    ↓  concept_graph.rs   — measure knowledge overlap (Jaccard)
    ↓  citation.rs        — measure citation coverage ratio
    ↓  probe.rs           — combine into plagiarism signal, degrade IDU
    ↓  journal.rs         — seal finding as immutable Event-KAKI
[AuditJournalEntry: CRC-16 sealed, append-only evidence]
```

---

### Stage 1 — Concept Graph Overlap (`concept_graph.rs`)

Books are represented as **KnowledgeTriples** from `enkidullm-core`:
```
("ch1", "introduces", "CAP_Theorem")
("ch2", "depends_on", "Paxos")
```

Concepts are extracted from the **subjects and objects** of these triples (not predicates). They are normalized before comparison:
```
normalize_concept(s) = lowercase(s).keep(alphanumeric + '_')

"CAP_Theorem" → "cap_theorem"
"ACID vs BASE" → "acid_vs_base"
```

**Jaccard similarity** between concept sets A and B:
```
jaccard(A, B) = |A ∩ B| / |A ∪ B|

Range: [0.0, 1.0]
  0.0 = completely disjoint books (no shared concepts)
  1.0 = identical concept sets
```

**`concept_graph_overlap(source, suspect)`** extracts concepts from both books and returns their Jaccard similarity. This is the **knowledge overlap score**.

**`missing_concepts(source, suspect, cited_hashes, source_uuid)`** returns concepts that:
- Appear in the source book
- Appear in the suspect book (overlap)
- The suspect book does NOT cite the source

These are the concepts with **erasure** — taken but not credited.

---

### Stage 2 — Citation Coverage Analysis (`citation.rs`)

The citation graph measures whether the suspect book acknowledges the source.

**`analyze_citation_coverage(source_uuid, shared_concepts, cited_hashes, _) → CitationAnalysis`:**

```
cited = 1  if source_uuid ∈ cited_hashes
       0  otherwise

coverage_ratio = cited / |shared_concepts|

is_suspicious = coverage_ratio < SUSPICIOUS_THRESHOLD (0.15)
```

This is deliberately simple: **did the suspect cite the source at all?** One citation among N shared concepts gives ratio `1/N`. With 20+ shared concepts and no citation, ratio = 0.0 — maximally suspicious.

**`SUSPICIOUS_THRESHOLD = 0.15`** — a book sharing 7+ concepts with a source and citing it zero times is flagged as suspicious.

**`citation_depth(source_hash, suspect_hashes, citation_map)`** — BFS through a citation edge graph to find the shortest citation path from the suspect to the source. Returns `None` if source is unreachable (direct plagiarism with no indirect acknowledgment).

---

### Stage 3 — Plagiarism Probe (`probe.rs`)

The probe **combines** overlap and citation into a single signal and decides whether to flag.

**Plagiarism signal:**
```
signal = max(0,  knowledge_overlap - citation_coverage)

High overlap + low citation = high signal = strong evidence of plagiarism
High overlap + high citation = low signal = legitimate scholarly use
Low overlap + any citation = low signal = unrelated or independent work
```

**Flagging rule:**
```
is_flagged = (knowledge_overlap ≥ OVERLAP_SUSPICIOUS)  AND  citation.is_suspicious

OVERLAP_SUSPICIOUS = 0.75   ← concept overlap threshold to enter suspicion zone
OVERLAP_HIGH       = 0.90   ← concept overlap threshold for high-confidence finding
```

**IDU state degradation:** If `is_flagged`, the suspect book's `IduState` is degraded:
```
Gold   → Orange    (flagged: verified book now under suspicion)
Orange → Gray      (flagged: already suspect, now deprecated)
Gray   → Gray      (no further degradation)
```

**Detection mode strings** (recorded in the journal):
| Condition | Detection Mode |
|---|---|
| overlap ≥ 0.90, citation < 0.15, missing concepts present | `HIGH_OVERLAP_WITH_CONCEPT_ERASURE_θ₆ overlap=X citation_gap=Y` |
| overlap ≥ 0.90, citation < 0.15, no missing concepts | `HIGH_OVERLAP_CITATION_CHOPPING overlap=X` |
| overlap ≥ 0.75 (but < 0.90) | `MODERATE_OVERLAP_LOW_CITATION overlap=X coverage=Y` |
| overlap < 0.75 | `BELOW_THRESHOLD overlap=X` |

**`batch_probe(pairs)`** runs all `(suspect, source)` pairs and returns a `Vec<ProbeResult>`. Each pair is independent — probes do not share state.

---

### Stage 4 — Immutable Audit Journal (`journal.rs`)

A flagged `ProbeResult` becomes a permanent **sovereign evidence tablet** — an `AuditJournalEntry`.

**Entry construction (`from_probe`):**

1. **Deterministic uuid_hash** via FNV-1a over `(suspect_uuid, source_uuid, timestamp_secs)`:
   ```
   evidence_hash = FNV-1a(suspect LE bytes || source LE bytes || timestamp LE bytes)
   ```
   Same finding at the same time always produces the same hash — **no fabricated entries**.

2. **Event KAKI** — minted by `KakiMinter` with tribe=LINGUISTIC (0x10FF), role=Zikru:
   ```
   event_kaki = KakiMinter::new(0x10FF).mint_event(evidence_hash, KakiRole::Zikru)
   ```
   The evidence tablet IS a KAKI. Its checksum can be verified independently.

3. **CRC-16 evidence seal** over the payload:
   ```
   payload = suspect_uuid || source_uuid || overlap_bits || citation_gap_bits
           || timestamp || detection_mode_bytes
   evidence_checksum = crc16(payload)
   ```
   Note: `citation_gap` (stored value) is used in the payload — NOT `citation_coverage`. This prevents floating-point roundtrip errors: `1.0 - (1.0 - x) ≠ x` in f32.

**`verify()`** recomputes the CRC-16 from the stored fields. Any tampering with `overlap_score`, `citation_gap`, `detection_mode`, or `timestamp_secs` breaks the seal.

**`AuditJournal`** — append-only, deduplicated:
- `append(entry)` — rejects duplicates by `evidence_checksum` (same finding, same timestamp → same checksum → rejected)
- `by_idu_state(state)` — filter entries by the resulting IDU state
- `to_markdown()` — renders a human-readable Sovereign Audit Tablet:

```markdown
# 𒁾 Sovereign Audit Tablet
**Suspect:** `0x0000bbbb`  **Source:** `0x0000aaaa`
**Overlap:** 1.00  **Citation Gap:** 1.00  **State:** ORANGE
**Mode:** HIGH_OVERLAP_WITH_CONCEPT_ERASURE_θ₆ overlap=1.00 citation_gap=1.00
**Missing Concepts:** cap_theorem, acid_vs_base, paxos
**Evidence Seal:** `0x3a7f`  **Timestamp:** 9999999
```

---

### Top-Level Pipeline (`lib.rs`)

```rust
audit_and_journal(
    pairs:          &[(ProbeInput, ProbeInput)],
    journal:        &mut AuditJournal,
    timestamp_secs: u64,
) -> usize   // number of entries written
```

Runs `batch_probe` on all pairs, writes only flagged results to the journal, returns count. Non-flagged results produce no journal entries — the system does not record what it does not find.

---

## Dependency Map

```
enkidullm-audit
    ├── enkidullm-core   ← KnowledgeTriple (concept extraction), IduState (degradation)
    ├── enkidb-kaki      ← Kaki, KakiMinter, KakiRole::Zikru (Event-KAKI minting)
    ├── bahyway-crc      ← crc16() for evidence seal computation and verification
    └── bahyway-core     ← TribeId (AUDIT_TRIBE = 0x10FF)
```

**Dependents:**
```
(application layer)  ← calls audit_and_journal() after ingest + embedding
```

Note: `enkidullm-audit` does NOT depend on `zikru-embed`. Plagiarism detection at the concept-graph level operates on knowledge triples, not embedding vectors. The `zikru-embed` θ₆ similarity is a separate, complementary signal available to higher layers.

---

## Sovereign Constraints

| Rule | Location | Effect |
|---|---|---|
| **§8.3 CrossTribe state never stored** | `probe.rs`, `journal.rs` | `ProbeResult.is_flagged` is a transient boolean — it is never persisted. Only `AuditJournalEntry` (an Event-KAKI) is stored. The plagiarism *relationship* is recomputable on demand; the *evidence of computation* is what is sealed. |
| **§2.4 No assessment in KAKI nucleus** | `journal.rs` | The `event_kaki` nucleus stores only uuid_hash + tribe. Overlap scores, citation gaps, and detection modes live in `AuditJournalEntry` fields (orbit-equivalent). |
| **Append-only, deduplicated journal** | `AuditJournal::append()` | Entries cannot be removed or overwritten. Duplicate findings (same checksum) are rejected silently. |
| **CRC-16 evidence seal** | `journal.rs` | Any post-hoc modification of entry fields breaks `verify()`. The seal uses `citation_gap` directly (not `coverage`) to prevent f32 roundtrip drift. |
| **Plagiarism = similarity + erasure** | `probe.rs` | Detection requires BOTH `knowledge_overlap ≥ 0.75` AND `citation.is_suspicious`. Similarity alone is not accusation. |
| **No unsafe code** | `#![forbid(unsafe_code)]` | All hash computation and CRC operations use safe arithmetic only. |
| **Deterministic evidence hash** | `journal.rs` | `evidence_hash(suspect, source, timestamp)` is FNV-1a — same inputs always yield the same uuid_hash. Findings are reproducible, not random. |

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

### Test Cases — `citation.rs` (10 tests)

| Test | Depth | What It Validates |
|---|---|---|
| `coverage_zero_when_not_cited` | L1 | 5 shared concepts, cited_hashes empty → ratio=0.0, is_suspicious=true |
| `coverage_full_when_cited_one_concept` | L1 | 1 shared concept, source cited once → ratio=1.0, is_suspicious=false |
| `not_suspicious_one_concept_one_citation` | L1 | 1 concept, 1 citation → ratio=1.0 ≥ 0.15, not suspicious |
| `no_shared_concepts_not_suspicious` | L1 | 0 shared concepts → ratio=1.0, is_suspicious=false (empty overlap is not accusation) |
| `citation_depth_direct_hit` | L1 | Source IS one of the suspect hashes → depth=0 |
| `citation_depth_one_hop` | L1 | Suspect cites B, B IS source → depth=1 |
| `suspicious_below_threshold` | L2 | 20 shared concepts, source cited once → ratio=0.05 < 0.15 → is_suspicious=true |
| `citation_depth_two_hops` | L2 | Suspect → intermediate → source → depth=2 |
| `citation_depth_unreachable` | L2 | No path from suspect to source → returns None |
| `citation_depth_no_cycle` | **L3** | Cyclic citation graph (A→B→A) with no path to source → returns None without infinite loop — BFS visited set prevents cycling |

### Test Cases — `concept_graph.rs` (7 tests)

| Test | Depth | What It Validates |
|---|---|---|
| `normalize_concept_lowercases` | L1 | "CAP_Theorem" → "cap_theorem" (lowercase + strip non-alphanumeric except `_`) |
| `jaccard_identical_sets` | L1 | `jaccard({A,B}, {A,B})` = 1.0 |
| `jaccard_disjoint_sets` | L1 | `jaccard({A,B}, {C,D})` = 0.0 |
| `concept_overlap_same_triples` | L2 | Identical triple lists → `concept_graph_overlap` = 1.0 |
| `concept_overlap_renamed_concepts` | L2 | Source has 4 concepts, suspect renames 1 → overlap in (0.5, 1.0) range |
| `missing_concepts_when_not_cited` | L2 | Shared concepts exist, source NOT cited → missing list is non-empty |
| `no_missing_when_cited` | **L3** | Shared concepts exist, source IS cited → missing list is empty — proves citation clears the erasure signal. This is the sovereign rule: acknowledgment negates the accusation. |

### Test Cases — `probe.rs` (6 tests)

| Test | Depth | What It Validates |
|---|---|---|
| `detection_mode_is_populated` | L1 | `probe_plagiarism()` always returns a non-empty `detection_mode` string |
| `probe_identical_books_high_overlap` | L2 | Identical triples, source NOT cited → `is_flagged=true`, `degraded_idu_state=Orange` |
| `probe_disjoint_books_not_flagged` | L2 | Completely different triples → `is_flagged=false`, overlap ≈ 0.0 |
| `probe_renamed_concepts_detected` | L2 | Source with 4 concepts, suspect renames 1 → `knowledge_overlap > 0.5` (partial detection) |
| `plagiarism_signal_gap_correct` | L2 | Identical books, no citation → `plagiarism_signal > 0.0` (overlap − coverage > 0) |
| `probe_cited_book_not_flagged` | **L3** | Identical triples, source IS cited → `is_flagged=false`, `degraded_idu_state=Gold` — the most important invariant: **identical content + proper citation = NOT plagiarism**. Proves the system does not accuse legitimate scholarship. |

**Critical invariant tested:**
- `probe_cited_book_not_flagged` is the legal safety valve. A system that flags properly-cited similarity would destroy legitimate scholarship. This test must never be removed and must remain L3.

### Test Cases — `journal.rs` (6 tests)

| Test | Depth | What It Validates |
|---|---|---|
| `markdown_contains_required_fields` | L2 | `to_markdown()` output contains "ORANGE", concept names, and "Sovereign Audit Tablet" |
| `journal_filter_by_idu_state` | L2 | `by_idu_state(Orange)` returns only Orange entries; `by_idu_state(Gray)` returns only Gray |
| `entry_from_probe_has_valid_kaki` | L2 | `AuditJournalEntry::from_probe()` produces an `event_kaki` that passes `verify_checksum()` |
| `different_timestamps_produce_different_entries` | **L3** | Same ProbeResult at t=1000 vs t=2000 → different `event_kaki.uuid_hash()` — deterministic uniqueness: time is part of the identity |
| `journal_append_and_dedup` | **L3** | Same ProbeResult at same timestamp appended twice → second `append()` returns false, journal.len()=1 — proves append-only deduplication by CRC-16 checksum |
| `entry_verify_integrity` | **L3** | `AuditJournalEntry::from_probe()` then `entry.verify()` → true — proves CRC-16 payload is computed consistently using `citation_gap` (not `citation_coverage`), avoiding f32 roundtrip drift where `1.0 - (1.0 - x) ≠ x` |

**Critical invariant tested:**
- `entry_verify_integrity` is the regression test for the f32 roundtrip bug. If `build_payload` used `citation_coverage` (rather than the already-computed `citation_gap`), the reconstructed CRC-16 in `verify()` would differ, and this test would fail. Must remain L3.

### Test Cases — `lib.rs` (2 tests)

| Test | Depth | What It Validates |
|---|---|---|
| `audit_clean_books_no_journal_entries` | **L4** | Two books with completely different concepts → `audit_and_journal()` writes 0 entries — proves no false positives: the pipeline produces no output for non-overlapping books |
| `full_audit_pipeline` | **L4** | Suspect copies source exactly with no citation → 1 entry written to journal, `entry.verify()=true`, `entry.idu_state=Orange` — validates the complete chain: concept extraction → Jaccard → citation gap → flagging → KAKI minting → CRC-16 seal |

**Total: 31 tests** — L1: 7 · L2: 10 · L3: 9 · L4: 5

**Note:** `enkidullm-audit` has the highest L3 density (9 out of 31 tests) of all four crates. This reflects the forensic nature of the crate: most of its critical logic involves invariants with legal implications.

### Gaps & Future Test Targets

| Area | Missing Coverage | Suggested Test |
|---|---|---|
| `probe.rs` | Orange → Gray degradation path not tested | Add test: suspect already at Orange, flagged again → degraded_idu_state=Gray |
| `probe.rs` | `batch_probe` parallelism not tested for independence | Add test: two independent pairs in batch → results are independent (no state leak between pairs) |
| `journal.rs` | CRC-16 tampering detection not tested | Add test: mutate `overlap_score` in an entry, then call `verify()` → false |
| `lib.rs` | Mixed-flag batch (some clean, some plagiarised) | Add test: 3 pairs, 2 clean, 1 flagged → `audit_and_journal` writes exactly 1 entry |
| Cross-crate | No test verifies that `enkidullm-audit` results correctly feed IDU degradation back into `enkidullm-core` BookOrbit | Integration test: audit, degrade, store updated IduState in VerticalEavStore, query |

---

*"The system does not accuse. It reveals the gap between what is known and what is acknowledged."*
*— Triple-O Provenance Axiom*
