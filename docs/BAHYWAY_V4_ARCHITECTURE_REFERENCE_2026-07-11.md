# 𒁾𒆳 BahyWay.Ecosystem v4.0 — Architecture Design Reference

**EnkiDB Sovereign Database Family — Services — Indexes — Storage — Algebra Arsenal — Testing Manual**
**DUB.SAR 𒁾 Bahaa Fadam | 2026-07-11 | SEALED, superseding all prior architecture references on points of conflict noted below**

**الحالة القديمة لا تُمحى أبداً 𒁾𒊬**

---

## 0. What this document is and is not

This consolidates the 2026-06-27 Architecture Design Reference, the
2026-07-01 Glossary/Roadmap, the 2026-07-07 28-document Google Drive
verification marathon (`docs/RM-002_ADDENDUM...`, `docs/BATCH2..6...`,
`docs/CLOSING_SUMMARY_28_DOCUMENTS...`, `docs/ARCHREF_NINSUN_DAILY...`,
`docs/BAHYWAY_V4_MANIFESTO_FINAL_RUN_SEQUENCE...`), NL-001 (Release
Codename Law), and everything built and verified in this session
(2026-07-11) — Enbilulu Calculus, the Algebra Arsenal (`bahyway-field`,
`algebra-arsenal`, `graph-engine`, octonions, Riemannian geometry,
Markov chains, symmetric Jordan Normal Form), and four newly-landed
concept documents (GL-ADU-002, GL-MRD-002 + Rev.2, GL-DST-001).

**This document follows `docs/TRANSPARENCY_STANDARD.md` (sealed
2026-07-11).** Every claim below carries or implies one of that
standard's tags — ✅ VERIFIED, 🧩 PARTIAL, 📄 DOCUMENTED, ⚠ COLLISION,
❌ NOT FOUND, 🔒 LAW, ⏳ UNREACHABLE — even where this document uses
plain checkmarks/prose for readability in a table; treat "✓ real,
coded" as ✅ VERIFIED with the citation given in the same cell, and
"(to build/extend)" as ❌ NOT FOUND unless stated otherwise. Two errors
were found and corrected the same day this document and the standard
were written: the HEPT-protocol magic-bytes claim and the
`KibratuCause` variant list, both confined to
`docs/BAHYWAY_ECOSYSTEM_V4_GLOSSARY.md` (see that document's H and K
sections) — neither claim was ever present in this document, but both
had been silently wrong in "SEALED" documents for weeks before being
caught by direct grep. That is the actual argument for the standard:
not that this document was wrong, but that nothing before the standard
existed would have caught it if it had been.

Every fact below was checked against actual repository code this
session, not copied from a prior document's self-report — the same
discipline the 28-document review already established and that this
document continues. Where two prior "SEALED" sources disagree, both
readings are shown and the disagreement is preserved as an **open
item requiring your ruling**, not silently resolved by this document.
Nothing here erases the source documents (`docs/*.md`, all still
present) — this is a consolidation, not a replacement, per the
ecosystem's own law.

---

## 1. Sovereign Architectural Axioms

- Everything is a Particle — engines, crates, data records, documentation, agents.
- Every particle has a KAKI 16-byte identity — immutable, sovereign, self-describing.
- Zero async runtime — pure `std::net` + `std::thread` everywhere.
- Zero external crate dependencies — pure Rust only (no serde, tokio, thiserror, blake3, tracing, flate2, rocksdb — confirmed still true of every crate this session touched).
- `#![forbid(unsafe_code)]` — no unsafe blocks anywhere.
- One wall, one law, one debug point — ConEngine for connections, GeoEngine for geometry.
- الحالة القديمة لا تُمحى أبداً — the old state is never erased.
- والمعمار وحده يقرر متى ينمو الكائن — the Architect alone decides when the organism grows (CSR-08).
- All quality/state/colour assessments live in EAV ONLY — never in KAKI bytes.
- B11 = round(H(P) × 240). Plimpton 322. Never 255.

---

## 2. KAKI 16-Byte Particle Identity — Canonical Layout

**Correction made in this pass:** `docs/BAHYWAY_ECOSYSTEM_V4_GLOSSARY.md`'s
KAKI entry (`timestamp_ns: u64 [0..8]`, `tribe_id: u32 [8..12]`,
`surrogate: u16 [12..14]`) does **not** match the real code and is
superseded here. Checked directly against
`workspace/bahyway_v4/crates/enkidb-kaki/src/kaki.rs`'s own header
comment (cites `KAKI_v4.0.pdf §1.2`) and the 2026-06-27 Architecture
Reference, which agree with each other and with the code:

| Bytes | Name | Description |
|---|---|---|
| κ[0..4] | minted_id / uuid_hash | Numeric ID minted at creation; `uuid_hash()` is the firewall key |
| κ[4..6] | tribe_id | u16, PA-15 sovereignty |
| κ[6] | kaki_type | 0x01 Identity / 0x02 Event / 0x03 CrossTribe |
| κ[7] | kaki_role | 0x01 KISHIB / 0x02 ZIKRU / 0x03 PARZU |
| κ[8..12] | reserved | Zeroed; never repurpose |
| κ[12..14] | timestamp | Birth timestamp, u16 |
| κ[14..16] | checksum | CRC-16/CCITT (poly 0x1021, init 0xFFFF) over κ[0..14] |

**Immutability rules:** byte values never modified; never reassigned
to a different particle; only held via `Copy` or shared `&Kaki` (no
`&mut Kaki` exists anywhere); no assessment data (state/quality/color)
in these bytes.

**PERMANENTLY DEPRECATED — NEVER REFERENCE:** RED/GREEN/BLUE score
bytes (v3.5 concept, deleted). Confirmed zero matches anywhere in
`enkidb-kaki` or `idu-prober`.

**OPEN ITEM #1 (κ[8..12]):** three sources call this `reserved`
(the canonical PDF, this Architecture Reference, the code itself);
one (ADR-003) wants to reassign it to `seq_counter`. Not resolved —
needs your ruling before anything writes to those bytes.

**OPEN ITEM #2 (kaki_type):** `enkidb-kaki/src/types.rs` defines a
4th value, `KakiType::Pattern = 0x04` (NISABA GA-cluster-derived
KAKIs, minted deterministically). No canonical source (PDF, this
document's own predecessor, ADR-008) lists it. Real and in active use
in code; not yet ratified into the canonical byte-layout table. Needs
your ruling: promote it, or confirm it's an intentional out-of-band
extension.

### 2.1 KAKI Identity Categories

| Category | kaki_type | tribe_id | Examples |
|---|---|---|---|
| Internal File | 0x01 | 0xFF00+ | Engines, crates, scripts, playbooks |
| External File | 0x01 | 0x0001+ | CSV, Excel, PDF, ZIP batches |
| Record | 0x01 | 0x0001+ | Grave record, sensor reading |
| Event | 0x02 | any | Gate transition, status update |
| CrossTribe | 0x03 | any | Requires Gilgamesh Passport (CSR-05) |

### 2.2 CrossTribe-KAKI v4.0.1 compliance — VERIFIED

The v4.0.1 canonical spec's substantive change (RGB birth-state bytes
removed, all quality assessment moved to EAV) is already fully
compliant in code: `Kaki::mint()` writes κ[8..12] zeroed and never
touches them elsewhere; `idu_prober::crosstribe::compose_n_anchors()`
matches the IDU Probing Rule's effective-state table exactly (all
Golden → Gold, any Dead → Gray, mixed → Orange), 5 tests covering it
including the N-anchor hyperedge case. No corrective work needed.

---

## 3. EnkiDB Sovereign Database Family — 7 Types

All 7 share the same KAKI layout, ENLIL indexes, and HeptaScript
query language; they differ in purpose, lifecycle, and storage
strategy.

| Port | Name | True Role | Real Crate(s) | Status (2026-07-11) |
|---|---|---|---|---|
| 7001 | EnkiDB | Golden Store — final, permanent destination | `enkidb-engine`, `enkidb-persist`, `enkidb-storage` | Journal-replay-on-open today; real WAL + data-file rearchitecture still deferred |
| 7002 | EnkiDW | Data warehouse — full ETL + analytics, receives retired EnkiODB particles | `enkidb-dw`, `enkidw` | Real, tested |
| 7003 | EnkiSDB | Stage/landing DB — `ValidationSweep` every 900 ticks promotes or quarantines | `enkidb-sdb`, `enkisdb` | Real, tested |
| 7004 | EnkiODB | Operational DB — state changes are new inserts, never mutations | `enkidb-odb`, `enkiodb` | Real, tested |
| 7005 | EnkiQDB | Quarantine jail for **fuzzy/unknown particles only** (never confirmed-malicious) | `enkidb-qdb`, `enkiqdb` | Real, tested |
| 7006 | EnkiMDB | Service/App Metadata DB — BahyWay's own crates/playbooks/services as KAKI-sealed EAV particles | `enkimdb` | **Now real** — closed 2026-07-01→2026-07-11. As of the 2026-07-07 review this was still confirmed absent; `enkimdb` now exists with exactly this description. `enkidb-quantdb` (financial tick/OHLC store) remains real and separate, no longer squatting on this role. |
| 7007 | EnkiDDB | Internal/client documentation DB, backing a future MetaEngine AI agent | `enkiddb` | **Now real** — same timeline as EnkiMDB above. `enkidb-recovery` (crash recovery) remains real and separate, no longer squatting on this role. |

This is the single most consequential update in this pass: every
document dated up to 2026-07-07 (including the Manifesto's own
"Collection C — confirmed not built... EnkiMDB and EnkiDDB start
here") states EnkiMDB/EnkiDDB do not exist. They now do, confirmed by
direct inspection of `crates/enkimdb/Cargo.toml` and
`crates/enkiddb/Cargo.toml` today.

### 3.1 Post-KAKI Tier-Transition Pipeline (real, tested)

```
EnkiSDB (staged, Pending)
    v ValidationSweep (every 900 ticks)
    +-- pass ------------------------> EnkiODB (Active) -> EnkiDW (retire) / EnkiDB (Golden Store)
    +-- fail (Quarantined) -> BlackBox Station scan
                                 +-- malware_flag=true  -> Storage Sector (terminal jail, one-way)
                                 +-- malware_flag=false -> EnkiQDB (fuzzy, pending Data Steward review)
                                                                v Data Steward resolves
                                                     +-- clean -> requeued into EnkiSDB (Pending)
                                                     +-- confirmed harmful -> Storage Sector
```

`storage-sector` and `blackbox-station` are real, tested crates
(landed 2026-07-01). `Storage Sector` sealing is a one-way door — the
crate exposes no method to remove, read back, or requeue a sealed
particle.

### 3.2 Pre-KAKI Structural Pipeline (BeeMDM)

```
Raw Archive (NARAMSIN Stage 0)
    v
DataStructure Station  -- schema detection, column profiling
    v
DataCompare Station    -- diff against reference tribe schema
    v
DataCleansing Station  -- PII vault, null imputation, type coercion
    v
KAKI assigned -- particle enters the post-KAKI pipeline above
```

### 3.3 É-DUBBA Gate Sequence — OPEN, THREE INCOMPATIBLE TELLINGS

**This is the single most-tangled open item carried forward from the
28-document review, and it remains open in this pass — not resolved
here.**

- **Vol. I** (2026-06-27): S1 MASHSHARU → ... → S7 **KIBRATU** (commit).
- **Vol. II** (2026-06-27, same day, also marked SEALED): S1 MASHSHARU
  → S2 NĀMZITUM → S3 PASHIRU → S4 KISPU_GATE(TIAMAT 5) → S5
  UTNAPISHTIM → S6 NISABA(DATA_STEWARD) → S7 **NERGAL_GATE** (AV scan
  + commit). Internally consistent across 5 separate mentions.
- **GL-001** (living document, references components postdating both
  volumes): MASHSHARU → NĀMZITUM → TIAMAT Engine 5 → PAŠIRU → KISPU →
  UTNAPISHTIM (6 items, different order), with Nisaba as an add-on
  "Stage 7" unrelated to NERGAL or an AV scan. **CAT-001 corroborates
  this reading** (DataSteward at S7, UTNAPISHTIM at Stage 6),
  disagreeing with Vol. II.

Evidence weight favors Vol. II's *naming* (NERGAL_GATE, not KIBRATU —
KIBRATU is independently and consistently documented elsewhere,
including in real code `enkidb-journal::event_cause`, as the
cause-analysis engine, not a gate) but GL-001/CAT-001's *ordering*
(DataSteward/Nisaba at S7, UTNAPISHTIM at S6). No single source has
both right, and this document does not invent a fourth version. **You
need to settle this with one authoritative table before it is wired
into anything new** (it already affects `blackbox-station`,
`data-steward-station`, and any future BeeMDM station code).

Distinct from the above and **already resolved** (2026-07-07): the
*sector*-level GATE-1 question (G1 APSU→Storage, G2 ADAD→ETL, G3
SHEDU→Security, G4 MUMMU→Algebra, G5 ENKIDU→AI, G6 DUBSAR→Languages,
G7 ENLIL→Governance, sealed 2026-06-18) is a **different question**
from the É-DUBBA pipeline-step sequence above, despite sharing the
"GATE-1" name in some documents. The pipeline-*function* half of
GATE-1 (GLS-001's doc-vs-SVG conflict) was ruled by you as "merge,
don't choose one" and is real, tested code today:
`bahyway_core::HeptaGate` — 7 variants, each carrying both `sector()`
(unchanged) and a new `pipeline_function()` (merged), 6 tests, all
passing (confirmed again this session).

---

## 4. ENLIL 6-Index Sovereign Stack

Query execution order = sovereign sargability, most pruning first:

| Index | Name | Type | Complexity | Purpose |
|---|---|---|---|---|
| 0 | SurrogateMap | u32↔KAKI-16 bidirectional | O(1) | Golden Key — eliminates full KAKI scan |
| 1 | KISPU HeadStore | Columnar arrays by attr_hash | O(1) per attr | Attribute projection — no journal touch |
| 2 | NATIRU | RoaringBitmap per state+gate | O(1) bitmap AND | Partition pruning before any EAV |
| 3a | BTreeRange | BTreeMap epoch/quality/B11 | O(log n) | Range queries on linear dimensions |
| 3b | idx_eav | HashMap attr+val→surrogates | O(1) hash | Exact attribute=value match |
| 4 | HeptaShellIndex | E7 heptagonal zones | O(1) zone, O(126) cone | Orbital (r,θ,φ) spatial queries |

**The KISPU/index-wiring fix — CLOSED, 2026-07-07, reconfirmed today.**
This was the single most-corroborated real blocker in the entire
28-document review (independently found 9+ times across grep
sessions, business cases, and daily reports): `heptascript::execute()`
did a full journal replay per particle on every query, never calling
`enkidb-indexes` despite it existing as real, tested code. Fixed: a
new `heptascript::indexed` module builds a real `SurrogateMap` +
`EavExactIndex` snapshot once at startup (`build_indexes`) and
resolves exact-equality `WHERE` clauses through it (`execute_indexed`)
before touching any particle's full history, falling back to the
always-correct `execute()` only when a query genuinely can't be
pruned that way. `enkidb-query-server` serves every query through
`execute_indexed`. Verified again this session:
`cargo test -p heptascript` → **183 passed** (up from the 164
originally cited, then 170 after the fix, now further ahead),
`cargo test -p enkidb-indexes` → **54 passed**, exact match to every
prior citation.

### 4.1 HeptaShellIndex — Sovereign Orbital Index

Foundation: Maryna Viazovska E7 lattice (Fields Medal 2022).

| Property | Value |
|---|---|
| Total zones | 882 (7 shells × 126 zones/shell) |
| E7 kissing number | 126 |
| Polar bands | 7 |
| Azimuthal sectors | 18 |
| Zone insert/lookup | O(1) |
| Cone query | O(126) |
| Gap detection | O(1) — Θ_E7(n) − bucket.len() |

---

## 5. GeoEngine = `bahyway-algebra` + the Algebra Arsenal

**"GeoEngine" is the name at least four sealed concept documents
(HS-EXT-002, GL-MRD-002, GL-DST-001, TPL-001) use for the ecosystem's
single mathematical truth source — confirmed 2026-07-10 to be
`bahyway-algebra`, not a separate crate.** `bahyway-algebra`'s own
`lib.rs` states this explicitly. Every GeoLaw in §7 below is enforced
here or in a crate `bahyway-algebra` is the truth source for.

### 5.1 Foundational math — verified against real source this session, not asserted

The full Algebra Arsenal is now indexed by one crate,
`algebra-arsenal` (`crates/algebra-arsenal`), which re-exports the
real primitive from wherever it actually lives, each backed by a
passing test — the standing, one-command answer to any future doubt
is `cargo test -p algebra-arsenal -p bahyway-field`.

| Concept | Real home | Status |
|---|---|---|
| Field (abstract, ℝ) | `bahyway-field::RealField` | ✓ new this session — genuinely missing before, now real, 7 axiom tests |
| Zmod240 (B11 ring) | `bahyway-field::Zmod240` | ✓ new — corrected from "field": ℤ/240 is a ring (240 composite, 2 has no inverse mod 240), proven by test |
| Vector Space (ℝ⁷) | `kinetic-engine::Vec7D` | ✓ real Add/Sub/Mul<f64>, dot/magnitude/normalize |
| Inner Product Space, weighted | `hepta-score` (`health_score`/`weighted_distance`) | ✓ real — the actual H(P) = 1/(1+√Σwᵢ(Pᵢ−Tᵢ)²) formula |
| Simplex / Simplicial Complex | `najaf-engine::topology` | ✓ real — barycentric 7D membership, Gaussian-elimination ghost reconstruction |
| Eigenvalue / Eigenvector | `ea-agent-algebra::matrix::SovereignMatrix` | ✓ real — exact 2×2 including complex, power-iteration spectral radius |
| Jordan Normal Form | `ea-agent-algebra::jnf` (new) + `jordan.rs` (existing) | ✓ complete for symmetric matrices (Jacobi eigendecomposition, spectral theorem guarantees size-1 blocks); general defective-matrix JNF explicitly NOT built |
| Clifford Algebra Cl(7) | `bahyway-algebra::clifford` | ✓ real, full 128-blade Cl(7,0) — was wrongly marked "to extend" before this session's re-verification |
| Bivector | `Multivector::grade(2)` | ✓ real |
| Rotor | `bahyway-algebra::rotor` | ✓ real, 7 tests |
| Spinor | — | (partial) only the single-plane Rotor form exists; general multi-plane spinor does not |
| Octonions | `bahyway-algebra::octonion` (new) | ✓ new — built via Cayley-Dickson doubling (R→C→H→O), verified via norm multiplicativity and confirmed loss of associativity |
| Manifold / Geodesic / Covariant Derivative / Riemannian Curvature | `vgca-engine::riemannian` (new) | ✓ new — general metric-tensor machinery, verified against a known analytic case (sphere K=1/r²), not just self-consistency |
| Shannon Entropy | `vgca-engine` (`bfv`, `fsv`) | ✓ real — two implementations (byte + text), broader than previously stated |
| KL Divergence | `vgca-engine::kl_divergence` (new) | ✓ new — fixed ε=1e-10 smoothing per Sovereign Rule |
| Directed Graph / PageRank / Betweenness / SCC | `graph-engine` (new crate) | ✓ new — SCC≥3 wired to real `alert-engine` (AML ring detection Sovereign Rule) |
| Markov Chain / Steady-State / Mean First Passage Time | `ammas-engine::markov` (new) | ✓ new — verified against closed-form 2-state analytic results |
| Enbilulu Calculus (Φ_Enbi, TIAMAT bands, horizon, Têrtu, Milu) | `bahyway-algebra::enbilulu` | ✓ real, 14 tests, consumed (not duplicated) by `wpd-engine::junction` |

**Confirmed still not re-verified this pass** (carried unchanged from
`ALGEBRA_GLOSSARY.md`'s 2026-06-05 table): DomainCentroid, Orbits
Calculus, Particles Algebra, Tribe Algebra, Modular Form, Theta
Series, Eisenstein Series E₂.

Full detail and provenance citations:
`workspace/bahyway_v4/ALGEBRA_GLOSSARY.md` Part VI.

### 5.2 GeoLaws (7)

| GeoLaw | Name | What it enforces |
|---|---|---|
| GeoLaw-01 | E7 Lattice | E7 = ONLY orbital zone decomposition. 126 kissing neighbours. |
| GeoLaw-02 | Plimpton 322 | B11 = round(H(P) × 240). 240 is the ONLY valid divisor. Never 255. |
| GeoLaw-03 | Betti Numbers | β₀/β₁/β₂ computed ONLY in GeoEngine. |
| GeoLaw-04 | Gap(n) | Gap(n) = Θ_E7(n) − Θ_found(n). Positive=HIDDEN_PATTERN. Negative=INTRUDER_CORRUPT. |
| GeoLaw-05 | Jordan Normal Form | Orbit stability: eigenvalues inside unit disc = stable. `ea-agent-algebra::jordan::JordanAnalyzer`, real. |
| GeoLaw-06 | VGCA∆ Tribe Algebra | Governs tribe interactions via `VgcaRegistry`. |
| GeoLaw-07 | Pauli Exclusion | No two KAKIs at same (r,θ,φ,tribe_id). |

---

## 6. Connection Sovereignty Rules (CSR-01 through CSR-08)

| Rule | Name | Enforcement point | Status |
|---|---|---|---|
| CSR-01 | Sargon Gate | `ConEngine::boot()` + `query()` | ✓ real, coded |
| CSR-02 | Role Gate | `SessionRegistry::resolve()` | ✓ real, coded |
| CSR-03 | NĀRU Frame Journal | `PooledConnection::send_frame()` | ✓ real, coded |
| CSR-04 | Credential Check | `CredentialStore` trait | ✓ real, coded (`StubCredentialStore` — accepts any zeroed blob unconditionally; real `AkkadiSafeEngine` wiring still pending) |
| CSR-05 | Gilgamesh Gate | `ConEngine::query()` | ✓ real, coded |
| CSR-06 | KIBRATU Emission | connection error paths | ✓ real, coded (stub emission). The emission step is real; the specific `KibratuCause` 7-variant taxonomy some documents attach to it is not — see `docs/BAHYWAY_ECOSYSTEM_V4_GLOSSARY.md` §K, corrected 2026-07-11. |
| CSR-07 | Tribe Isolation | `PooledConnection::send_frame()` | ✓ real, coded |
| CSR-08 | Architect Sovereignty | **All agents** (cross-cutting, not a single ConEngine code path) | **Sealed as governance law, confirmed real and correctly cited by two independent 2026-06-26/27 sources word-for-word — but NOT YET coded.** `enkidb-con-engine/src/csr.rs` implements CSR-01 through CSR-07 only, as of this session. This is the real, still-open PB-170 gate (see §9). |

`cargo test -p enkidb-con-engine`: 6 tests passing (CSR-01/02/07,
NĀRU journal verify, role ordering, NĀRU entry serialize).

**PAZUZU threat-simulation cross-check (verified line-by-line against
real `con-engine` source in the 07-07 review, not asserted):**
PAZUZU-01 (StubCredentialStore accepts any zeroed blob) — confirmed.
PAZUZU-03 (NĀRU-SYNC not running) — confirmed, stated in code's own
header. PAZUZU-04 (no `max_connections` cap) — confirmed. PAZUZU-05
(no opcode whitelist in `send_frame()`) — confirmed. These are real,
open gaps to close before the security posture claimed anywhere in
this ecosystem can be trusted at the connection layer, not decorative
test names.

---

## 7. AI Agent Quartet

| Agent | Role | Crates | Status (2026-07-11) |
|---|---|---|---|
| TamuzAI | Code generation | `ea-agent-core`, `ea-agent-algebra`, `ea-agent-chat` | LIVE |
| EaAgent | Mathematical truth — GeoLaws, Plimpton 322 | `ea-agent-algebra`, `ea-agent-oracle` | LIVE |
| NINSUN | Healer / Progressive Refiner, advisory-only | `ninsun-agent`, `ninsun-steward-bridge` | Real, tested — ahead of the 2026-06-27 doc's "To build" status. Real, tested TF-IDF+cosine memory search now wired via `adapa-recall` (this session). ESARHADDON SMI urgency now wired into its steward-queue priority (this session). |
| NuskuAgent | Governance lamp-bearer, WAY v2.0 policy | — | Still absent, v4.3+ future, consistent with every prior check |

**Distinct, narrower body — not the same as the Quartet:**
`crates/agent-council::AgentId` defines `{TamuzAI, Ninsun, Pazuzu}` as
a 3-member Pattern-governance council for Pattern-KAKI evaluation
specifically. Same two names (TamuzAI, Ninsun) overlap with the
4-member ecosystem-wide Quartet by coincidence of purpose, not
identity — flagged, not merged, pending your one-line disambiguation.

---

## 8. Naming Law

### 8.1 Three-layer law (NL-001, sealed pending your AkkadianSeal)

| Layer | Name class | Spent examples |
|---|---|---|
| Components, engines, calculi | Gods & divine beings | Marduk, Enlil, Enki, Tiamat, Nisaba, Ninsun, Addu, Igigi |
| Structural/geographic | Cities | Eshnunna, Susa, Nuzi, Eridu(ous) |
| Release eras | Kings | Gudea (era 1), Zagesi (era 2) |

Format: `BahyWay.Ecosystem vX.Y "‹King› M.m"`. Founding designation:
**`BahyWay.Ecosystem v4.0 "Gudea 1.0"`**. Orthography: codenames are
single, unbroken, plain-Latin words — no hyphens, apostrophes, or
diacritics (Lugalzagesi not Lugal-Zagesi). Exclusion registry: Sargon,
Gilgamesh, Esarhaddon (all — security), UrNammu, NaramSin (both —
component engines) — every one of these five verified real and spent
in code this session (CSR-01 Sargon gate, CSR-05 Gilgamesh gate,
`SargonKdf`/`SargonPassport` in `kupru`, `urnammu-attestationd`,
`naramsin-*`). Reserved, unassigned: Anu, Alulim, Nuzi, Susa.

### 8.2 "-Way" suffix deprecation (sealed 2026-07-03)

> "All suffix 'WAY' in crate names are deprecated, to preserve 'WAY'
> as the Security Protocols Language that has the `.way` file type."

Crate/component names lose "-Way" (`AkkadiSafeWay`→`AkkadiSafeEngine`,
etc.). **WAY v2.0 itself — the sovereign security-policy language and
its `.way` files — is explicitly exempt and unchanged.** v3.5-era
source comments that self-label as v3.5 are historical record, kept
as-is. Confirmed applied in the repo; one stale script comment
(`bahywaylab.sh`) was corrected 2026-07-07.

### 8.3a Sovereign database names — Tigris, Euphrates, Enkidu (sealed 2026-07-12)

**EnkiDDB is Tigris. EnkiMDB is Euphrates.** The Architect's own framing:
"these 2 Enki Types Databases are the Soul of my BahyWay.Ecosystem in
each release — without them there is no Ecosystem." Layered on top of
the real type/crate identifiers, the same relationship "GeoEngine" has
to `bahyway-algebra` — `enkiddb`/`EnkiDDB` and `enkimdb`/`EnkiMDB` are
unchanged, real crate/type names; Tigris/Euphrates are the sovereign
names shown alongside them (DubSar Theater format: `EnkiDDB (Tigris
v4.0)`, `EnkiMDB (Euphrates v4.2)`).

Both are now backed by a real, tested versioning mechanism, not just a
label: `enkidb-readnode::generation` (new, shared by both database
types) formalizes the "each materialization needs a fresh path"
constraint (§4's KISPU note doesn't apply here, but the Data Files
writer's append-only behavior does — see `playbook_174`) into a
`(sovereign_name, version)` -> Data Files scheme.
`enkiddb::materialize_version`/`list_versions` and
`enkimdb::materialize_version`/`list_versions` let multiple versions of
the same database (e.g. Tigris v4.0 and v4.1, or two Euphrates
sessions v4.1 and v4.2) exist side by side, each independently
openable and queryable — the concrete mechanism behind "compare v4.0
with v4.1" in DubSar Theater's version picker. 4 new tests in
`enkidb-readnode::generation` plus 2 end-to-end comparison tests (one
per database type) prove two versions of the same sovereign name hold
genuinely different data, not the same path twice.

**"Enkidu" for EnkiduLLM/TamuzAI — proposed, not yet applied.** Checked
for collision first: `HeptaGate::Enkidu = 5` already exists (sector
"AI Agents", pipeline function "AI-Assisted Match/Dedupe & Steward
Review") — not a bad collision, a good alignment, since EnkiduLLM
already is the AI engine that gate represents. The Architect has not
yet chosen between a full literal crate rename (`enkidullm-core` ->
`enkidu-core`, etc. — touches 6 crates, 3 external dependents,
`dubsar-visualizer`/`orbital-trust-probe`/`zikru-embed`) and the same
sovereign-name-on-top pattern used for Tigris/Euphrates. Not applied
either way yet — recorded here so it isn't lost, not because a
decision was made.

### 8.3 Sealed concept documents landed 2026-07-11

| Doc ID | Title | File |
|---|---|---|
| GL-ADU-002 | Addu Cyclone Extension Law | `docs/marduk/addu/GL-ADU-002-cyclone-extension.md` |
| GL-MRD-002 | Nēberu Slicer — Orbit Section & 7D Wave Law | `docs/marduk/GL-MRD-002-neberu-slicer.md` |
| GL-MRD-002 Rev.2 | Analysis-to-Solution Law (DETECT→PROVE→PREDICT→PRESCRIBE) | same file, §7 |
| GL-DST-001 | Theater-as-Workbench Law | `docs/theater/GL-DST-001-theater-as-workbench.md` |

Landed via `playbooks/playbook_166` through `playbook_169`, renumbered
from their originating session's PB-161–164 (which collide with this
repo's real, different playbook_161–164) and corrected for host name
(`eriduous-vdi`, not `eriduous_vdi`) and doc paths (into this repo's
`docs/` tree, not `~/bahyway/docs/...` outside any checkout). Indexed
in `docs/catalog/CAT-001-index.md`.

**Open, still unnamed:** "Nēberu Slicer" (HS-EXT-004 nouns: SECTION,
PLANE, WAVEBAND, HARMONIC) awaits your confirmation, same as the four
concept documents' implementation status — concept-sealed, not built.

---

## 9. BeeMDM ETL — General & Specific Testing Manual

### 9.1 The concrete test procedure

`docs/TESTING_PLAYBOOK_PHASE1.md` is the real, existing, run-in-order
test manual for the **50 compressed files / 10 million particles**
BeeMDM ETL test — this is the authoritative answer to "General and
Specific Stations Tests," not something to design from scratch:

- **Corpus:** 7 archive formats — zip, nested.zip, CORRUPT.zip, tar.gz,
  tar.bz2 (stub, expected), tar.xz (stub, expected), 7z (stub,
  expected) — plus malicious (zip-bomb, path-traversal) and corrupt
  fixtures.
- **Blocks A–F**, run top-to-bottom, each with exact `cargo test`/
  `cargo run --example` commands and expected output:
  - **A** — NARAMSIN archive decompression (9 tests + 9 corpus checks)
  - **B** — CRC integrity (`bahyway-crc`, 7 tests)
  - **C** — Session registry parsing (6 tests)
  - **D** — ConEngine 7 CSR rules (6 tests)
  - **E** — HeptaScript at 10M particles (unit tests + a real timed
    query, target <1s; ABORT_SCAN safety valve)
  - **F** — Full 50-file corpus batch run, gate: zero unexpected errors
- **Pass/fail gate:** every block PASS, E-004 query <1000ms, F-001
  zero unexpected errors — checklist and a Notes section to fill in
  live, in the file itself.

### 9.2 Entry criteria before the test may begin (MAN-001's own six-item gate)

1. ✅ Preflight/Phase A–B green. Confirmed today: `cargo test --workspace`
   = **3,374 passed, 0 failed**, across all 144 crates + 9 binaries.
2. ❌ **CSR-08 implemented in code (PB-170).** Still open — see §6.
3. ✅ GATE-1 (pipeline-step half) ruled and coded — `HeptaGate`,
   confirmed 2026-07-07, reconfirmed today.
4. ⏳ Test dataset staged on `eriduous-vdi` — cannot be checked from
   this sandbox; no `journal.bin` or data directory exists in this git
   checkout. Run on `eriduous-vdi`.
5. ❌ Latency budget table accepted as pass/fail criteria — not
   located in this repository or any document processed. You need to
   supply or confirm this.
6. ⏳ KAKI-minting-only-at-`enkidb-ingest::bridge` dry run on 100
   records, observed directly — operational step, run on
   `eriduous-vdi`.

**Two of six are still genuinely open** (items 2 and 5); two require
your hands-on action on `eriduous-vdi` (items 4 and 6, this sandbox
cannot reach your local VMs — see §10). Items 1 and 3 are closed.

### 9.3 Playbook run order (all playbooks, current as of this document)

Target `eriduous-vdi` unless noted. Run in this order:

| # | File | Verifies |
|---|---|---|
| 1–8 | `playbook_153`–`playbook_160` | Fable Impact Gate, HS-EXT-002/003, Wizard contract/install, TPL-001 §D/E |
| 9 | `playbook_161_fable_crosstribe_wizard_reconciliation.yml` | Audit-trail record |
| 10 | `playbook_162_hepta_parallel_dispatch_and_ide_editor.yml` | Real TCP parallel dispatch, HeptaScript editor |
| 11 | `playbook_163_enbilulu_calculus_geoengine.yml` | Enbilulu Calculus |
| 12 | `playbook_164_algebra_arsenal_boundaries.yml` | `bahyway-field` + `algebra-arsenal` |
| 13 | `playbook_165_algebra_arsenal_gap_closure.yml` | Octonions, Riemannian geometry, graph-engine, Markov, symmetric JNF |
| 14 | `playbook_166_addu_cyclone_extension_seal.yml` | GL-ADU-002 |
| 15 | `playbook_167_neberu_slicer_concept_seal.yml` | GL-MRD-002 |
| 16 | `playbook_168_analysis_to_solution_law.yml` | GL-MRD-002 Rev.2 (depends on 167) |
| 17 | `playbook_169_theater_as_workbench_law.yml` | GL-DST-001 |
| 18–19 | `playbook_172`, `playbook_173` (`hosts: localhost`, self-contained git fetch) | heptascript index wiring, GATE-1 HeptaGate |

Each of 1–17: `scp` to `eriduous-vdi`, then
`ansible-playbook <file> -i "localhost," -c local -v`. 18–19 do their
own `git fetch`/`checkout`/`pull` first, safe to run standalone.

**None of these target `enkidb-node-read` or `enkidb-node-write`
directly** — this batch of work is crate/doc-level, not a CQRS-node
deployment. If you want the DubSar Visualizer pulling live 7D Hepta
Space data from those two nodes specifically, that needs a new
playbook once you tell me how those nodes currently receive the
query-server binary.

---

## 10. What this sandbox can and cannot check

This session runs in an isolated, ephemeral cloud container with no
network path to your local QEMU/KVM VMs (`eriduous-vdi`,
`dubsar-workstation`, `enkidb-node-read`, `enkidb-node-write`). Every
`cargo test`/`cargo build` figure in this document was run **in this
git checkout**, which holds all 144 crates (the code) but **zero
persisted data** — no `journal.bin`, no data directory. The
80,272-particle NAJAF_CEMETERY journal and every "particles exist"
claim in any document lives on your `eriduous-vdi` Forge tree, not
here. §9.2 items 4 and 6, and the actual 10M/50-file timed run, must
be run and observed by you there.

---

## 11. Confirmed absent — do not build against these without designing from scratch

Confirmed absent from this repository and from every one of the 28
Google-Drive documents' actual attachments, despite confident
"SEALED ✓"/"WRITTEN" narration in some of them: **SumerEngine, NUZI
(crate/module), AsakkuEngine, ZeroEngine, ShoWEngine, the full TIAMAT
engine set as a crate, NERGAL as an AV-engine crate, Merkle journal
verification, `AkkadiRulesEngine`/`AkkadiSafeEngine`/
`AkkadiCipherEngine` as actual crates** (names only, plus one now-fixed
stale script comment), **the PB-119–136 ESARHADDON-series compressed
playbooks** (the `esarhaddon` crate itself is real; that specific
18-playbook delivery claim is not), **`ashnan-kaki`** (blocks PB-118/140
if ever wanted). **SusaEngine "validated 9/9"** is the single
most-repeated unconfirmed claim across multiple documents — never once
backed by an actual playbook file, unlike every other "validated"
claim that did check out. Flagged most strongly for your own
independent verification before trusting it.

---

## 12. Open items requiring your ruling (consolidated, not re-numbered from source)

**Structural:**
1. κ[8..12] — reserved (3 sources) vs ADR-003's `seq_counter` (1 source).
2. `KakiType::Pattern = 0x04` — promote to canonical, or confirm intentionally out-of-band.
3. É-DUBBA gate sequence (§3.3) — three incompatible tellings, needs one authoritative table.
4. TIAMAT alert-band naming — `GREEN/DILBAT/KAKKAB/ERRA/NERGAL/MAROON` (Vol. II, fullest) vs GL-001's claim that ERRA *replaces* NERGAL at the alert layer (direct contradiction of Vol. II's own explicit "NERGAL is also a TIAMAT alert level").
5. `BC-ENV-001` claimed by two unrelated business cases (NANSHE/Diyala River vs. Enbilulu/WPDEngine) — needs a renumbering ruling.
6. Two unrelated crates both legitimately named "WPDEngine" (structural-defect/priority routing, real in this repo; acoustic leak-triangulation, real in an uploaded playbook) sharing one crate name.
7. FierWall Defender vs. `hepta-sec-firewall` — same engine or two layers?
8. `Shedu` (a sovereign journal name) vs. `SHEDU` (the security sector) — same collision pattern as NERGAL.
9. `AdadAI` and `TERTUM` (GL-001 terms) — real and current, or draft-only?
10. `crates/agent-council`'s 3-member council vs. the 4-member Agent Quartet (§7) — same body or deliberately distinct?

**Gating the BeeMDM test (§9.2):**
11. CSR-08 code implementation (PB-170) — governance law is sealed; the code shape is not decided.
12. Latency budget table — not located anywhere; needs you to supply or confirm one.

**Playbook numbering — now confirmed collision-prone across at least nine separate incidents** (PB-150 alone has four claimants: AsakkuEngine-deploy, WPD-Engine acoustic diagnostics, CSR-08 rule file, and — before renumbering — this session's own work; PB-151 two; PB-152 two; PB-117-vs-156–160; PB-87–90-vs-88–97; and PB-161–164 vs. this session's real work, now corrected in §8.3). **Recommendation, not yet a rule:** before assigning any new playbook number, grep `playbooks/` in this actual repository first — every collision found this session involved a number assigned without that check.

**Test counts:** treat every test-count figure in any document dated
before 2026-07-11 as superseded by this document's §9.2 item 1 figure
(3,374 passed, 0 failed) and by `ALGEBRA_GLOSSARY.md` Part VI's
per-concept citations — several older documents' specific numbers
(RM-001's PB-99–109 table, `kinetic-engine`'s "29 tests") were already
confirmed stale by the 2026-07-07 review, in both directions (some
undercounted, some overcounted).

---

*𒁾𒆳 BahyWay.Ecosystem v4.0 — Architecture Design Reference, 2026-07-11.
Every figure in this document was produced by running `cargo build`/
`cargo test`/`grep`/`find` against the actual repository during this
session, or by reading the cited source document in full — nothing
here is copied from a prior document's claim without independent
re-verification. Where a prior document's claim could not be
independently checked (anything requiring `eriduous-vdi`), it is
marked ⏳, not silently assumed.*
