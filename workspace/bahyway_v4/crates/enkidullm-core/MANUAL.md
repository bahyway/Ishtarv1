# 𒂗𒆠𒁺 enkidullm-core — Manual
**Version:** 4.0.2 | **Layer:** 9.5 — EnkiduLLM Foundation

---

## What It Solves

Every book ingested into EnkiduLLM must become a **sovereign particle** — an entity with an immutable identity, a mutable assessment shell, and a clean separation between what a book *is* and what the system *thinks* about it.

Without this crate, there is no stable identity for a book across ingest, embedding, and audit. A book would be just a blob of text with no KAKI nucleus, no provenance seal, and no Triple-O orbit structure to carry trust levels, reputation scores, or knowledge triples.

**`enkidullm-core` solves the identity and assessment storage problem for books.**

---

## How It Works (Mechanism)

### 1. Book Identity — `book_kaki.rs`

Every book is identified by a **uuid_hash** computed from its ISBN or file path using FNV-1a:

```
uuid_hash = FNV-1a( isbn_bytes )   OFFSET=2_166_136_261  PRIME=16_777_619
```

This hash is deterministic: the same ISBN always produces the same uuid_hash. It becomes the nucleus of a `BookKaki` particle.

**BookKaki structure:**
```
BookKaki {
    nucleus: Kaki          ← immutable 16-byte KAKI (§2.4 — no assessment here)
    orbit:   BookOrbit     ← all assessment lives here
}
```

The nucleus is minted by `KakiMinter` from `enkidb-kaki`. The orbit wraps five assessment shells.

**Book Domain Tribes** (`BookDomainTribe`) map subject areas to TribeId constants:
| Tribe | TribeId | Subject |
|---|---|---|
| COMPUTER_SCIENCE | 0x1001 | Algorithms, systems, networking |
| MATHEMATICS | 0x1002 | Pure and applied mathematics |
| PHYSICS | 0x1003 | Classical and quantum physics |
| BIOLOGY | 0x1004 | Life sciences |
| ECONOMICS | 0x1005 | Economics and finance |
| PHILOSOPHY | 0x1006 | Logic, ethics, epistemology |
| CROSS_DOMAIN | 0x1007 | Interdisciplinary works |
| LINGUISTIC | 0x10FF | Reserved — audit events, model KAKIs |

`BookDomainTribe::from_domain(s)` maps a lowercase domain string to the correct TribeId using prefix matching. Unknown domains fall back to CROSS_DOMAIN (0x1007).

---

### 2. EAV Orbit Shells — `orbit.rs`

Assessment is structured into **four shells** ordered by change frequency:

| Shell | Contents | Mutability |
|---|---|---|
| `CoreShell` | title, isbn, author, publication_year, archetype | Static after ingest |
| `ClassificationShell` | domain_tribe, sub_domains, language, edition | Slow change |
| `ReputationShell` | idu_state, trust_score, citation_count, peer_reviews | Medium change |
| `ContentShell` | knowledge_triples, concept_count, embedding_kaki_ref | Fast change |

**`IduState`** controls the trust level of a book:
```
Gold   → active, verified canonical source
Orange → suspect, unverified, or flagged by audit
Gray   → deprecated, withdrawn, or dead
```

State transitions:
- `combine(a, b)` → takes the *weaker* of two states (Gold + Orange = Orange)
- `degrade()` → steps down one level (Gold → Orange → Gray)
- `BookOrbit::update_reputation()` → promotes Orange → Gold when trust_score ≥ 0.85

**`BookArchetype`** classifies the structural role of a book:
`Textbook | Reference | Monograph | Proceedings | PractitionerGuide | Standard`

**`KnowledgeTriple`** is the atomic unit of extracted knowledge:
```
KnowledgeTriple { subject: String, predicate: String, object: String }
e.g.  ("ch1", "introduces", "CAP_Theorem")
```

---

### 3. Vertical EAV Store — `eav_store.rs`

The `VerticalEavStore` is a columnar attribute store for book orbit data. It stores `(entity_uuid, attribute, value, timestamp)` tuples in **per-attribute vertical columns** backed by a `BTreeMap` index.

**`OrbitValue`** is the attribute value type:
| Variant | Storage | Indexable |
|---|---|---|
| `Text(String)` | String | Yes |
| `Integer(i64)` | i64 | Yes |
| `Float(u32)` | bit-pattern of f32 | Yes |
| `Boolean` | bool | Yes |
| `KakiRef(u32)` | uuid_hash reference | Yes |
| `DoiRef(String)` | DOI string | Yes |
| `ConceptRef(String)` | concept identifier | Yes |
| `Embedding(Vec<u32>)` | f32 bit-patterns | **No** — excluded from BTree |
| `Absent` | sentinel | No |

Float uses `u32` bit-pattern representation to enable `Ord` without external crates.

**Key operations:**
- `insert(uuid, attribute, value, timestamp)` — inserts and updates BTree index
- `find_in_range(attribute, min, max)` — BTree range query → list of entity UUIDs
- `traverse_kaki_refs(start_uuid, max_depth)` — BFS graph walk through KakiRef chains (cycle-safe)
- `get_profile(uuid)` — scatter-gather: collects all attributes for one entity

---

## Dependency Map

```
enkidullm-core
    ├── enkidb-kaki      ← Kaki 16-byte layout, KakiMinter, KakiRole (Kishib/Zikru/Parzu)
    ├── bahyway-core     ← TribeId primitive
    └── bahyway-crc      ← CRC-16 for KAKI checksum verification
```

**Dependents** (crates that import enkidullm-core):
```
enkidullm-ingest  ← uses KnowledgeTriple, IduState, BookOrbit
zikru-embed       ← uses IduState, TribeId constants
enkidullm-audit   ← uses KnowledgeTriple, IduState for plagiarism signal
```

---

## Sovereign Constraints

| Rule | Location | Effect |
|---|---|---|
| **§2.4 Structural-Facts-Only** | `BookKaki::nucleus` | The KAKI nucleus stores uuid_hash + tribe only. No title, no author, no score. All assessment lives in `BookOrbit`. |
| **§8.3 CrossTribe State** | `ContentShell` | Plagiarism relationship between two books is NOT stored in any orbit shell. It is computed on demand by `enkidullm-audit`. |
| **No unsafe code** | `#![forbid(unsafe_code)]` | All crate code is safe Rust. No raw pointers, no transmutes. |
| **No third-party runtime deps** | `Cargo.toml` | Only internal bahyway/enkidb crates. No `serde`, no `tokio`, no `ordered-float`. |
| **Float as u32 bits** | `OrbitValue::Float` | `f32::to_bits()` / `f32::from_bits()` used for `Ord` — avoids the `NaN != NaN` ordering problem without external crates. |

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

### Test Cases — `book_kaki.rs` (7 tests)

| Test | Depth | What It Validates |
|---|---|---|
| `book_uuid_hash_stable` | L1 | Same ISBN bytes always produce the same uuid_hash (FNV-1a is deterministic) |
| `book_uuid_hash_distinct` | L1 | Different ISBN bytes produce different uuid_hashes (no trivial collision) |
| `domain_tribe_computer_science` | L1 | `from_domain("computer science")` returns COMPUTER_SCIENCE TribeId (0x1001) |
| `domain_tribe_cross_domain_fallback` | L1 | Unknown domain string falls back to CROSS_DOMAIN (0x1007), never panics |
| `linguistic_tribe_reserved` | L1 | LINGUISTIC (0x10FF) is never returned by `from_domain()` — reserved for audit |
| `book_kaki_nucleus_valid` | L2 | `BookKaki::new()` produces a KAKI whose `verify_checksum()` passes |
| `ingestion_event_is_event_kaki` | L2 | `BookKaki::ingestion_event()` returns a KAKI with `kaki_type = Event (0x02)` |

### Test Cases — `orbit.rs` (7 tests)

| Test | Depth | What It Validates |
|---|---|---|
| `knowledge_triple_construction` | L1 | `KnowledgeTriple::new(s, p, o)` stores all three fields correctly |
| `book_archetype_labels` | L1 | Each `BookArchetype` variant returns a non-empty `label()` string |
| `idu_state_combine` | L1 | `combine(Gold, Orange) = Orange`; `combine(Gold, Gold) = Gold` (takes weaker) |
| `idu_state_degrade` | L1 | Gold → Orange → Gray; Gray.degrade() stays Gray (no underflow) |
| `orbit_from_core_defaults` | L2 | `BookOrbit::from_core()` initialises all assessment fields to safe neutral defaults |
| `reputation_update_promotes_to_gold` | L2 | Orange book with trust_score ≥ 0.85 is promoted to Gold after `update_reputation()` |
| `reputation_update_stays_orange_below_threshold` | L2 | Orange book with trust_score < 0.85 stays Orange (threshold is a hard gate) |

### Test Cases — `eav_store.rs` (9 tests)

| Test | Depth | What It Validates |
|---|---|---|
| `float_roundtrip` | L1 | `OrbitValue::from_f32(v).as_f32()` recovers the original value exactly |
| `orbit_value_ordering` | L1 | `Ord` on `OrbitValue` satisfies: Text < Integer < Float < Boolean < KakiRef |
| `insert_and_find_text` | L2 | Inserted Text attribute is found by BTree lookup |
| `find_in_float_range` | L2 | `find_in_range()` returns only UUIDs whose float attribute falls in [min, max] |
| `embedding_not_indexed` | L2 | Inserting an Embedding value does NOT add it to the BTree index (too large) |
| `get_profile_scatter_gather` | L2 | `get_profile(uuid)` collects all attributes for one entity across all vertical lines |
| `version_history` | L2 | Two inserts with different timestamps both appear; `get_history()` returns both in order |
| `kaki_ref_traversal` | L2 | `traverse_kaki_refs(start, depth=3)` follows KakiRef chains and returns all reachable UUIDs |
| `no_cycle_in_traversal` | **L3** | Circular KakiRef chain (A→B→A) does not cause infinite loop — BFS visited set breaks cycle |

### Gaps & Future Test Targets

| Area | Missing Coverage | Suggested Test |
|---|---|---|
| Cross-crate | No test verifies BookKaki feeds correctly into VerticalEavStore | Integration test: ingest a book, store orbit attributes, query by trust range |
| `orbit.rs` | `ContentShell` knowledge_triples field is never queried in tests | Add test: insert triples, verify concept_count is accurate |
| `eav_store.rs` | `find_in_range` only tested for Float; Text range untested | Add test: range query on Text attribute (alphabetical ordering) |

---

*"The KAKI is born, the Orbit moves, the Journal remembers."*
