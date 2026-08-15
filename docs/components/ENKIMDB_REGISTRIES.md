# EnkiMDB Internal Registries — Error Registry/Journal + Unified Attribute Type Registry

**Standalone component reference. Follows `docs/TRANSPARENCY_STANDARD.md`.
No claim below is asserted without a tag and a citation.**

## Status: ✅ VERIFIED — real code, real tests, wired end-to-end

Both registries below were designed and built in this session (2026-07-24),
directly on top of the existing `enkidb-kaki` / `enkidb-journal` /
`template-engine` / `enkimdb` machinery — no new mechanism was invented.
Every claim in this document cites a real file and was checked by running
`cargo test`.

## Why these live in EnkiMDB, not a client-writable table

Both registries answer the Architect's design requirement directly: they
must be **internal, read-only-to-clients data**, reachable only through
EnkiMDB's CQRS Read Node — never a table a client (or a client-facing
service) can insert into directly. That shape already exists for
artifact cataloging (`enkimdb::WriteNode::ingest_artifact` — 🔒 the CQRS
split: write path mints Kakis via a real `enkidb_journal::Journal` WAL,
read path is `enkimdb::readnode` / ADR-012 Data Files). The two registries
below reuse that exact write path, under two new EAV namespaces
(`error_type.*` / `error_occurrence.*` / `attr_type.*`) alongside the
existing `artifact.*` namespace — not a new database, not a new mechanism.

## 1. Error Registry + Journal

### The design (Architect's framing, 2026-07-24)

Each **ErrorType** is a Particle with an Identity-Kaki — it is born once
(low cardinality: "Lemniscate identity phrase needs 3+ consonants" is
one ErrorType, defined once). Each **ErrorOccurrence** is a Particle with
a fresh Event-Kaki every time that ErrorType actually fires (high
cardinality: who hit it, when, from which service) — targeting the
ErrorType's existing Identity-Kaki, never minting a new one. This is the
same "N events, one target" shape the Journal already supports for any
other entity.

One correction made during design, not yet raised back to the Architect
for confirmation: the Architect's phrasing suggested "the StoryEngine can
register" occurrences. Per StoryEngine's own doc comment
(`crates/story-engine`, read-only by design — "the Journal stores events,
StoryEngine answers questions"), StoryEngine does not write. The real
write path is `enkimdb::WriteNode` (via `AdadGate`-style minting);
StoryEngine's role stays what it already is — reading the Journal back to
answer "what happened," never producing entries.

### What's real

| Piece | Location | Tag |
|---|---|---|
| `ErrorSeverity` (Info/Warning/Error/Critical), `ErrorTypeSpec { code, summary, severity, defined_by }`, `ErrorOccurrenceSpec { source, detail }` | `crates/enkidb-journal/src/error_registry.rs` | ✅ VERIFIED — 4 tests, `cargo test -p enkidb-journal` |
| `EventCause::ErrorTypeRegistered = 0x70`, `ErrorOccurred = 0x71`, `ErrorTypeUnknown = 0x72` (reserved for a future occurrence-against-unknown-type guard, not yet used by any caller) | `crates/enkidb-journal/src/event_cause.rs` | ✅ VERIFIED — included in `round_trip_all_variants` test |
| `RegistryEmitter::emit_error_type` (mints Identity-Kaki, role=Parzu — "logic/template/axiom/rule"; emits `error_type.code`/`.summary`/`.severity`/`.defined_by`), `emit_error_occurrence` (mints nothing, targets the existing ErrorType Identity-Kaki; emits `error_occurrence.source`/`.detail`) | `crates/enkimdb/src/registry_emitter.rs` | ✅ VERIFIED — 2 of 3 tests on this file cover the Error Registry |
| `WriteNode::ingest_error_type(spec, epoch) -> IdentityKaki`, `WriteNode::log_error_occurrence(type_kaki, occurrence, epoch)` | `crates/enkimdb/src/writenode.rs:71-111` | ✅ VERIFIED — compiles + covered by `cargo test -p enkimdb` (16 passed) |

### Why role=Parzu for ErrorType, not Zikru

`enkidb_kaki::KakiRole` (`crates/enkidb-kaki`): `Kishib` = external
artifact, `Zikru` = record/entity, `Parzu` = logic/template/axiom/rule.
An ErrorType is a *rule* ("this condition is an error, at this
severity") — the same category as a Template or an AttrType, not a
record instance. An ErrorOccurrence's own Event-Kaki is minted with
`KakiRole::Zikru` (`writenode.rs:102`) — it *is* a record: one specific
firing.

### What is NOT yet built

- No caller anywhere in the workspace actually invokes
  `ingest_error_type` / `log_error_occurrence` outside of its own unit
  tests — ❌ NOT FOUND as a live integration. The kupru tools' existing
  `_explain_kupru_error()` (Godot GDScript, `sargon-passport-manager` /
  `gilgamesh-master-key`) still only shows errors to the user; it does
  not yet log them to EnkiMDB. Wiring that is future work, not claimed
  done here.
- `EventCause::ErrorTypeUnknown` is defined but nothing emits it yet —
  logging an occurrence against a `type_kaki` that was never registered
  is not currently guarded or detected.
- No de-duplication: registering the same `code` twice via
  `ingest_error_type` mints two distinct ErrorType Identity-Kakis (unlike
  `AttrTypeRegistry::register`, which rejects duplicate `attr_hash`).
  `WriteNode` has no in-memory registry of already-seen `code`s to check
  against — a real gap if this is wired into a live service later.

## 2. Unified Attribute Type Registry

### The design (Architect's framing, 2026-07-24)

One canonical physical representation per attribute, registered once,
shared across EnkiSDB / EnkiODB / EnkiDB / EnkiWD (and any other EnkiDB
Type) so two clients never silently disagree on precision — e.g. a price
stored as a 16,2 fixed-point value by one client and an 18,1 or a raw
`f32` by another, breaking exact comparison/summation once that data
flows through shared pipelines. Registered once per `attr_hash`
(low-cardinality, like `Template` itself), version-tagged so a later
widening (e.g. more decimal places) is an explicit new version, never a
silent reinterpretation of bytes written under the old one — because KAKI
/ EAV / Journal entries are immutable.

### What's real

| Piece | Location | Tag |
|---|---|---|
| `AttrType` enum (`FixedPointScaledInteger { scale }`, `Float32`, `Float64`, `Integer`, `Text`, `Blob`), `.expected_byte_len()` | `crates/template-engine/src/attr_type.rs:28-72` | ✅ VERIFIED |
| `AttrTypeSpec { attr_hash, name, attr_type, version }` | `crates/template-engine/src/attr_type.rs:76-93` | ✅ VERIFIED |
| `AttrTypeRegistry` (`register` rejects duplicate `attr_hash` — does not overwrite; `validate_value(attr_hash, bytes) -> Result<(), TypeViolation>`) | `crates/template-engine/src/attr_type.rs:109-161` | ✅ VERIFIED — 8 tests, `cargo test -p template-engine` |
| `TypeViolation::UnregisteredAttribute` / `WrongByteLength` | `crates/template-engine/src/attr_type.rs:97-106` | ✅ VERIFIED |
| `EventCause::AttrTypeRegistered = 0x73` | `crates/enkidb-journal/src/event_cause.rs` | ✅ VERIFIED |
| `RegistryEmitter::emit_attr_type` (mints Identity-Kaki, role=Parzu; emits `attr_type.name`/`.representation`/`.attr_hash`/`.version`) | `crates/enkimdb/src/registry_emitter.rs:57-70` | ✅ VERIFIED |
| `WriteNode::ingest_attr_type(spec, epoch) -> IdentityKaki` | `crates/enkimdb/src/writenode.rs:117-133` | ✅ VERIFIED |
| **Enforcement point**: `vgca_validation::beam::validate_with_types(template, eav, attr_types) -> ValidationResult` — a second, independent check layered onto the existing required-field gate `validate()` already performed. Only `WrongByteLength` fails `ValidationResult::is_valid()`; `UnregisteredAttribute` alone does not (an attribute with no canonical type yet is not necessarily an error — matches the doc comment on `TypeViolation::UnregisteredAttribute` itself). | `crates/vgca-validation/src/beam.rs` | ✅ VERIFIED — 4 new tests (`validate_with_types_*`), `cargo test -p vgca-validation` → 35 passed |

### A correction made mid-session, worth keeping visible

An earlier pass in this same session concluded `vgca-validation` was
"entirely about VGCA-Σ/Δ/Λ statistical/geometric outlier detection" and
therefore the wrong place for type enforcement — that was true of
`vgca.rs` specifically, but the crate also contains `beam.rs`, a
separate file doing exactly the required-field structural validation
this registry needed to extend. The correction: `beam::validate_with_types`
*is* the right enforcement point, in the crate originally (correctly)
identified, just the wrong file within it. This is recorded here rather
than silently fixed, per `TRANSPARENCY_STANDARD.md`'s discipline around
self-corrections.

### What is NOT yet built

- `beam::validate_with_types` is not yet called from any live ingest
  path — ❌ NOT FOUND as a wired integration. It exists, is tested in
  isolation, and is the designated call site; nothing in
  `enkiddb-write-server` / `enkimdb-write-server` / EnkiSDB / EnkiODB /
  EnkiWD write paths calls it yet.
- No `AttrTypeRegistry` is currently populated for any real production
  attribute (e.g. no `price_usd`/`latitude` constants exist outside
  `attr_type.rs`'s own tests) — the registry mechanism is real; the
  *catalog* of canonical types for this ecosystem's actual attributes
  has not been authored.
- Cross-EnkiDB-Type enforcement (EnkiSDB / EnkiODB / EnkiDB / EnkiWD each
  calling the *same* shared `AttrTypeRegistry` instance, sourced from
  EnkiMDB's read node) is the stated end goal but is architecturally
  unproven here — today's registry is an in-process `HashMap`, not yet
  loaded from or synced against EnkiMDB's materialized Read Node.

## Verification record

```
cargo test -p enkidb-journal -p template-engine   # 21 + 26 passed, 0 failed
cargo test -p enkimdb                             # 16 passed, 0 failed
cargo test -p vgca-validation                      # 35 passed, 0 failed
cargo build --workspace                            # clean
```

## Next steps (not started)

1. Wire `beam::validate_with_types` into an actual write-server ingest
   path (`enkimdb-write-server` is the natural first target, since both
   registries live in EnkiMDB).
2. Populate a real `AttrTypeRegistry` catalog for this ecosystem's
   highest-drift-risk attributes (money, geo-coordinates) as a startup
   step, not per-request construction.
3. Give `WriteNode` a duplicate-`code` guard for `ingest_error_type`,
   matching `AttrTypeRegistry::register`'s duplicate-`attr_hash`
   rejection.
4. Emit `EventCause::ErrorTypeUnknown` from `log_error_occurrence` when
   `type_kaki` doesn't resolve to a real prior `ErrorTypeRegistered`
   entry (requires `WriteNode` to look itself up in its own Journal, or
   accept an `&AttrTypeRegistry`-style side table).
5. Wire kupru tools' `_explain_kupru_error()` (GDScript) to actually log
   occurrences to EnkiMDB via a network call, not just display them
   locally.
