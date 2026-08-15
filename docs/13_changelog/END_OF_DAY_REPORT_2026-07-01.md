# End-of-Day Report — 2026-07-01

## What shipped today (PR #17, merged to `master` at `9494c26`)

### 1. BeeMDM tier-transition pipeline corrected
Your architecture clarification — EnkiQDB is a jail for **fuzzy/unknown**
particles only, confirmed-harmful particles must go to a separate
hardware-isolated Storage Sector, and the Data Steward needs a real
loop-back path — is now real code, not just a design decision:

- **`enkidb-journal`**: 4 new `EventCause` variants (`0x60-0x63`):
  `BlackBoxRoutedHarmful`, `BlackBoxRoutedFuzzy`, `StorageSectorMove`,
  `StewardResolvedRequeue`.
- **`storage-sector`** (new crate): the hardware-isolated, terminal jail.
  Sealing is a one-way door — no method exists to remove or requeue.
- **`blackbox-station`** (new crate): scans every `Quarantined` particle
  in EnkiSDB and splits them by `malware_flag` — harmful → Storage Sector,
  fuzzy → EnkiQDB.
- **`data-steward-station`**: new `QuarantineReviewQueue` — pulls the fuzzy
  backlog from EnkiQDB, resolves cases clean (requeue into EnkiSDB) or
  confirmed-harmful (seal into Storage Sector). EnkiQDB stays append-only;
  original records are never touched.
- **`eridu-runtime::SchedulerLoop`**: `run_sweep()` now routes quarantined
  particles through BlackBox Station instead of dumping everything into
  EnkiQDB unconditionally — this was the actual bug your architecture
  review caught.
- **`tests/e2e`**: `five_tier_pipeline` extended with 2 new integration
  tests proving the corrected flow end-to-end (malware → Storage Sector;
  fuzzy → EnkiQDB → Steward review → resolve-clean-and-requeue or
  confirm-harmful-and-seal); 3 existing tests that asserted the old
  (incorrect) "malware lands in EnkiQDB" behavior were fixed to match
  reality.

### 2. NINSUN as the Data Steward's AI assistant
Built from the architecture doc you provided ("NINSUN surfaces, the Data
Steward decides, KISPU commits"):

- **`ninsun-steward-bridge`** (new crate): `NinsunAdvisoryQueue` — takes a
  batch of `ninsun-agent::RefineProposal`s, sorts by descending confidence
  so the Steward triages the highest-likelihood anomalies first.
  `confirm()`/`reject()` are the only way a case leaves `Pending`; both
  journal a real entry (2 more new `EventCause` variants, `0x64-0x65`:
  `NinsunAdvisoryConfirmed`, `NinsunAdvisoryRejected`) against the real
  particle the proposal targets.
- Deliberately its own crate, not folded into `data-steward-station` —
  keeps the AI-adjacent `ninsun-agent` dependency out of the deterministic
  pipeline crate. This bridge is the only place that touches both.
- Scope note: this is a Stage 2b concern (pattern/semantic anomaly review,
  conceptually between NĀMZITUM and TIAMAT) — a different pipeline point
  from the BlackBox/Storage Sector/EnkiQDB work above (structural/
  sovereignty failure handling). Not wired into `SchedulerLoop`.

### 3. Documentation corrected
`docs/00_codex/BAHYWAY_ECOSYSTEM_V4_GLOSSARY.md` and
`docs/19_roadmap/BAHYWAY_ECOSYSTEM_V4_ROADMAP.md` were carrying wrong information from
an earlier design pass — both updated today:
- EnkiDB Types table: EnkiQDB's role corrected (fuzzy-only, not "quantum/
  probabilistic"); EnkiMDB and EnkiDDB explicitly flagged as **not yet
  built**, with the crates currently squatting on those ports
  (`enkidb-quantdb`, `enkidb-recovery`) called out as real-but-mislabeled,
  unrelated systems.
- BlackBox Station's definition corrected — it was described as "orbital
  trust probe / anomaly detection," which conflated it with the unrelated
  `orbital-trust-probe` crate. Now describes the real, tested scan-and-route
  logic.
- New glossary/roadmap entries: Storage Sector, NINSUN /
  `ninsun-steward-bridge`, the `EventCause` catalogue, and the real
  post-KAKI tier-transition pipeline diagram.

## Verification
`cargo build --workspace` and `cargo test --workspace` both pass clean, no
failures, across the entire repository (not just the new crates) — run
after every commit today, not just claimed.

## What's explicitly deferred (your call, not forgotten)
- **EnkiDW, EnkiMDB, EnkiDDB real implementations** — you named these "a
  long story, needs a new session." EnkiMDB's true role (Service/App
  Metadata for TemplateEngine) and EnkiDDB's true role (internal/client
  documentation DB for a MetaEngine AI agent) are now correctly documented,
  but neither has real code yet.
- **EnkiDB's real WAL + data-file storage engine** (replacing full-journal-
  replay-on-open and the current fake-mmap `MmapReader`) — flagged as
  foundational to the "Golden Store, >1B particles" goal, tied to the same
  deferred set.
- **StoryEngine cross-database extension** (tracking a particle's full
  journey across all 7 stores, not just single-journal current-state
  projection) — mentioned by you, not explicitly in today's build scope.

## Tomorrow
Your call on sequencing — both are ready whenever you are:
1. Download the merged `master` zip, copy to `eriduos-vdi`, run
   `cargo build --workspace` / `cargo test --workspace` there for
   independent verification (the pattern we've used all session), then
   report back what you see.
2. Continue with EnkiDW / EnkiMDB / EnkiDDB design and real implementation
   in a fresh session, now that their true roles are locked in the
   glossary and roadmap.

Nothing today touched those three types' code — only their documented
role, so there is no partial/inconsistent state waiting for you there.
