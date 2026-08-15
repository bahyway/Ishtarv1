# RM-002 Addendum — Verified Against Actual Repository State
Generated: 2026-07-07
Appends to: RM-002 (`BahyWay.Ecosystem v4.0: Final Consolidated Roadmap`) and its
source planning documents — `KAKI_v4.0.1_canonical.pdf`, `GL-MRD-001`, `BC-MRD-001`,
`HS-EXT-001`. None of these four documents existed in this repository before this
session; all four were authored in a separate ("iPhone") session and retrieved from
Google Drive for this evaluation. This addendum applies the same discipline as
`RM-001_ADDENDUM_PB111-145_VERIFIED_2026-07-01.md`: verify claims against
`cargo build` / `cargo test` / `find` on the actual repository, not against a prior
session's self-report.

## Why this addendum exists

RM-002 (dated 2026-07-09, authored ahead of that date) claims a long ledger of
playbooks through PB-152 are "WRITTEN" or "SEALED ✓," including several — PB-117
(ASHNAN KAKI schema), PB-119–136 (the ESARHADDON series) — that the repository's own
RM-001 addendum had already verified, six days earlier, **do not exist**. RM-002 does
not incorporate that correction; it reasserts the original unverified claims. This
addendum re-verifies the full ledger against the repository as it stands today, and
separately verifies the CrossTribe-KAKI structural change described in
`KAKI_v4.0.1_canonical.pdf`.

## Executive summary (added at close of the 2026-07-07 verification marathon)

This addendum grew across a full session of source material supplied directly
(business cases, ADRs, Ansible playbooks with embedded Rust source, Godot
scripts, HTML simulations, test corpora) — not just Drive documents. Closing
summary, for a reader who won't read all seven parts:

**Verified real, by actually building and running code, not just reading
claims:**
- PB-152 (ENLIL Tribe HotIndex) — 5/5 tests, correct at 1B-particle scale
  (Part 6)
- PB-150 WPD-Engine "Diagnostics" (acoustic leak triangulation) — 11/11
  tests, matches RM-002's claim exactly (Part 7)
- PAZUZU-01→05 threat-simulation claims — verified line-by-line against the
  real `con-engine` source; genuinely real gaps, not decorative (Part 5)
- `wpd_orbit_simulation.html` — a JS port of the same PB-150 Rust logic
  (identical Deming fit, CRLB error budget, material table, pseudo-noise
  function); consistent, no new discrepancy
- NARAMSIN spec — matches the real `naramsin-*` crates already in this repo
  (Part 7)
- CrossTribe-KAKI's v4.0.1 structural change — already compliant in
  `enkidb-kaki`/`idu-prober` (Part 1)

**Confirmed still fabricated or unverifiable, despite RM-002 marking them
"SEALED ✓":** SumerEngine, NUZI, AsakkuEngine (source code never found or
supplied anywhere, despite two AsakkuEngine deployment playbooks existing —
Part 2, Part 7), the PB-119–136 Esarhaddon compressed series (Part 2).

**Corrected in the repository itself this session** (see commit history on
this branch): `crosstribe_kaki.md`'s false "never persisted" claim,
`enkidb-kaki/src/kaki.rs`'s stale type-count comment, ADR-011's own
mis-citation of a Forbidden Operation number, `BAHYWAY_ECOSYSTEM_MANUAL_V4.md`'s
undocumented TIAMAT/NERGAL naming collision.

**Left flagged, not resolved — each needs the Architect's ruling, not a
guess:**
1. κ[8..11] byte assignment — ADR-003 (`seq_counter`) contradicts ADR-008 and
   the canonical PDF (`reserved`), same-day-dated documents disagreeing (Part 1, 3)
2. Two separate, non-identical Forbidden-Operations lists (ADR-008's 17 vs.
   the canonical PDF's own) — never reconciled (Part 1, 3)
3. Three different TIAMAT alert-band namings in circulation (Part 5)
4. `BC-ENV-001` claimed by two unrelated business cases (NANSHE/Diyala vs.
   Enbilulu/WPDEngine) (Part 6)
5. **Five separate playbook-number collisions**, all involving RM-001/RM-002's
   compressed PB-111–145 ledger or playbooks supplied directly: Ashnan
   (PB-117 vs. 156–160), PAZUZU (87–90 vs. RM-002's 88–97), Esarhaddon (its
   own 119–155 vs. RM-002's 119–136/137–145), and now PB-150 and PB-151 each
   claimed twice by unrelated components (Parts 2, 4, 5, 6, 7). This is no
   longer an occasional glitch — every detailed per-module roadmap checked
   this session has collided with RM-002's compressed ledger.
6. Two unrelated "WPDEngine" implementations sharing one crate name — the
   real defect/priority/sector crate in this repo, and the acoustic-solver
   crate this session verified from PB-150 (Part 7)
7. Ashnan's `kaki_role = PARZU` for a plain sensor-reading particle type
   (Part 4)

**The one finding that isn't a documentation problem:** the KISPU HeadStore
— the columnar index HeptaScript needs to avoid O(n) journal replay — does
not exist in code. This was independently confirmed **six separate times**
across this session's source material (a live VDI grep session, RM-002's own
BLK-1 gate, and Phase-0 prerequisites independently stated in the Ashnan,
Esarhaddon, and Diyala River business cases, plus PB-152's own design notes).
Every other finding here is about which document to trust; this one is about
an actual missing piece of the runtime that several real modules are already
blocked on.

## Part 1 — CrossTribe-KAKI structure change (KAKI v4.0.1 canonical): VERIFIED COMPLIANT

`KAKI_v4.0.1_canonical.pdf` specifies one substantive change from prior drafts:
removal of deprecated RED/GREEN/BLUE birth-state bytes from the KAKI byte layout.
Bytes κ[8..11] become reserved/zeroed; all quality, colour, and freshness assessment
moves exclusively into EAV attributes.

Checked against `workspace/bahyway_v4/crates/enkidb-kaki/src/kaki.rs` and
`crates/idu-prober/src/crosstribe.rs`:

- No RGB/red_score/green_score/blue_score bytes exist anywhere in `enkidb-kaki` or
  `idu-prober` (grep-verified, zero matches).
- `Kaki::mint()` writes κ[8..12] as reserved/zeroed and never touches them elsewhere;
  `byte_layout()` test asserts `&b[8..12] == [0u8; 4]`.
- The IDU Probing Rule's effective-state composition table (§8 of the canonical doc)
  matches `idu_prober::crosstribe::compose_n_anchors()` exactly: all-Golden → Gold,
  any Dead → Gray, mixed Golden/Fuzzy → Orange. Five tests cover this, including the
  N-anchor hyperedge case; all assertions are consistent with the spec.

**Conclusion: the CrossTribe-KAKI structure in this repository already conforms to
the v4.0.1 canonical spec.** No corrective playbook is needed for this change.

One drift worth a decision, not a fix: `enkidb-kaki/src/types.rs` defines a fourth
`KakiType::Pattern = 0x04` (for NISABA GA-cluster-derived KAKIs, minted
deterministically rather than randomly) that the canonical PDF does not mention at
all — the PDF's byte layout table and Forbidden Operations list both describe exactly
three `kaki_type` values. The doc comment at the top of `kaki.rs` line 6 still says
"0x01 Identity / 0x02 Event / 0x03 CrossTribe" only, which is now stale relative to
`types.rs`. Recommend either promoting Pattern-Kaki to the canonical spec in the next
KAKI revision, or confirming it's intentionally out-of-band and updating the stale
comment to say so.

## Part 2 — RM-002 sealed-era ledger, re-verified against repo state

| PB / item | RM-002 claim | Verified against repo |
|---|---|---|
| PB-88–97 | WRITTEN (Phase A run pending) | Not independently re-verified this session; unchanged from RM-001 |
| PB-98 BLK-1 | Query-server rewritten to `enkidb-indexes` — gate closed | **Confirmed.** `bin/enkidb-query-server/src/main.rs` documents the abandonment of `enkidb_enlil_index`/`hepta_shell_index`/`geo_engine` and now depends on `heptascript` (which indexes via `enkidb-indexes` internally). Matches merged PR #15. |
| PB-99–109 | WRITTEN (run pending) | Not independently re-verified this session |
| PB-110 | RUN ✓ | Confirmed — `playbooks/playbook_110_session_deliverables_urnammu_nisaba_kittu.yml` exists |
| PB-111–116, 118 | WRITTEN | Consistent with RM-001 addendum's findings (mix of superseded/redesigned/already-implemented/blocked) — unchanged |
| **PB-117 (ashnan-kaki)** | WRITTEN | **False, again.** No `ashnan-kaki` crate exists anywhere in `workspace/bahyway_v4/crates`. This is the exact gap RM-001's addendum flagged on 2026-07-01; RM-002 reasserts "WRITTEN" without correction. Blocks PB-118/140 as before. |
| **PB-119–136 (ESARHADDON series)** | WRITTEN ("compressed delivery") | **Still not found.** No trace of an 18-playbook ESARHADDON/ASHNAN-TIAMAT/NARAMSIN-regression/INANA delivery in the repository. Same conclusion as RM-001 addendum: this range does not exist. |
| PB-137–145 | WRITTEN | Consistent with RM-001 addendum (ninsun-agent, namtar-kaki, ereskigal-kaki, ashnan regional configs — real; UrNammu attestation impl, Kittu Engine v1 — not attempted; grafana/prometheus, É-DUBBA — blocked on infra) |
| PB-146 KineticEngine | SEALED ✓, 29/29 tests | **Crate is real** (`crates/kinetic-engine`), but currently has **95** `#[test]` functions across its source files, not 29. Either the count is stale or refers to a subset; the crate itself is not fabricated. |
| **PB-147 SumerEngine** | SEALED ✓, 32/32 tests | **Does not exist.** No `sumer-engine` crate, binary, or directory anywhere in the repository. |
| PB-148 NaviEngine | SEALED ✓ | **Crate is real** (`crates/navi-engine`), substantially tested across 15 source files |
| PB-149 NajafEngine | SEALED ✓ | **Crate is real** (`crates/najaf-engine`), substantially tested across 9 source files |
| PB-150 WPDEngine | SEALED ✓, 11/11 tests | **Crate is real** (`crates/wpd-engine`), but currently has **45** `#[test]` functions, not 11 |
| **PB-151 NUZI** | SEALED ✓ | **Does not exist.** No `nuzi` crate, module, or file anywhere in the repository |
| PB-152 ENLIL Tribe HotIndex | SEALED ✓ | An `Enlil` algebra module exists (`crates/bahyway-algebra/src/enlil.rs` — Jordan Normal Form tribe algebra + the four High Council gates), but nothing in it matches the specific claim of a "branchless autovectorized batch kernel, 2.41B particles/s" hot index. Unverified as described. |
| **AsakkuEngine** | SEALED ✓ | **Does not exist.** No trace anywhere in the repository. |
| CAT-001 catalog | "128 on-disk crates, 13 collections" | Repository currently has **131 crates** + **9 binaries** under `workspace/bahyway_v4`. Close to the claimed figure but not exact; not independently worth reconciling further given the more serious ledger errors above. |

**Pattern:** every crate RM-002 claims that this session could independently find
(KineticEngine, NaviEngine, NajafEngine, WPDEngine, the PB-98 gate) is real, with the
specific test-count figures being the only inaccuracy. Every crate/playbook claim this
session could **not** find (ashnan-kaki, the PB-119–136 series, SumerEngine, NUZI,
AsakkuEngine) is a repeat or extension of exactly the fabrication pattern RM-001's
addendum already documented once. RM-002 should not be treated as authoritative for
those five items without independent re-verification at build time.

## Part 3 — MardukEngine playbook plan (GL-MRD-001, PB-MRD-01→17): NOT STARTED, PLAN IS SOUND

No `marduk` crate exists anywhere in the repository (only a pre-existing
`docs/04_gates/marduk_gate.md`, unrelated to the new analytics engine). This is
genuinely new, unstarted design work — not a repeat of the WRITTEN/SEALED fabrication
pattern above, since GL-MRD-001 never claims it's already built.

The plan itself (PB-MRD-01 crate scaffold → 02 metric/template machinery → 03 Nabû
covariant-derivative core → 04–06/08 derived verbs → 07 HeptaScript v1.1 wiring → 09
diagnosis instantiation → 10–13 domain packages → 14–16 delivery/theater → 17 test
gate) is coherently dependency-ordered and ends in a concrete, falsifiable seal
criterion (zero phantom deviation alerts across all four domains under a template
re-seal). No structural objection to adopting it as written.

RM-002 §1 states its own governing law: no queued extension set (including
PB-MRD-01–17) receives real PB numbers until Phase A (with the PB-98 BLK-1 gate),
Phase B, `TESTING_PLAYBOOK_PHASE1` Blocks A–F, and a BeeMDM **50-zip** ETL test are
all complete.

- PB-98 BLK-1: **confirmed closed** (Part 2, above).
- `TESTING_PLAYBOOK_PHASE1` Blocks A–F: no recorded evidence in this repository that
  the full block sequence (50-file corpus, 10M particles, sub-1-second HeptaScript
  query) was executed end-to-end and recorded, as opposed to individual crate unit
  tests passing.
- BeeMDM 50-zip test: PRs #17 and #18 document real end-to-end runs against **one**
  zip file each (including the restart-duplication bug fix in #18), not the full
  50-zip corpus the gate specifies.

**Conclusion: the stated gate for unlocking PB-MRD-01–17 is not yet closed.** This
doesn't mean the MardukEngine plan is wrong — it means starting PB-MRD-01 now would be
a deliberate decision to relax RM-002's own gate, not a case of the gate already being
satisfied. That decision belongs to the Architect, not to a session assuming the gate
was met.

## What's still genuinely open

1. **PB-117 (`ashnan-kaki`)** — unchanged from RM-001 addendum. Real design work
   (particle classes, ColourID B11 test, CMI formula), never built, blocks PB-118/140.
2. **PB-119–136** — unchanged from RM-001 addendum. An 18-playbook gap with no prior
   draft to recover; would need to be designed and built from scratch if still wanted.
3. **SumerEngine, NUZI, AsakkuEngine** — newly discovered in this session. RM-002
   marks all three "SEALED ✓" with specific test counts; none exist. If these are
   wanted, they need to be scoped and built as new work, not treated as complete.
4. **The BeeMDM 50-zip gate and full `TESTING_PLAYBOOK_PHASE1` run** — the actual
   precondition RM-002 itself sets for unlocking PB-MRD-01–17 (and PB-ENB-01–16).
   Either run it for real, or make an explicit, recorded decision to proceed without it.
5. ~~**KakiType::Pattern (0x04) vs. the canonical spec**~~ — **fixed 2026-07-07**:
   `kaki.rs`'s stale 3-type comment now notes the 4th (Pattern) type and flags it as
   needing an Architect ruling on canonical status, rather than silently
   contradicting `types.rs`.

## Part 4 — Ashnan (BC-AGRI-001), verified against the source business case

`BC-AGRI-001` (the actual ASHNAN design document, dated June 2026 — predates
RM-001) was supplied directly and read in full. Two findings correct this
addendum's own earlier confidence and RM-002's ledger:

1. **A genuine playbook-numbering conflict, not yet resolved.** RM-001 and
   RM-002 both call the Ashnan KAKI schema **PB-117** ("WRITTEN" per RM-002;
   "not actually produced" per RM-001's own correction — both agree on the
   number, disagree on status). But BC-AGRI-001's own 8-phase roadmap assigns
   the identical deliverable to **Playbooks 156–160**, as part of a
   156–197 range covering the full Ashnan rollout (sensor pilot, CMI/IRI/LMI
   formulas, international ingestion, DubSar Theater scenes, regional
   extension). Both numbering schemes cannot be correct for the same
   deliverable. BC-AGRI-001 is the more detailed, earlier-dated source
   document, but that is not this addendum's call to make — the Architect
   should rule which numbering is authoritative before either is built
   against.
2. **The KISPU HeadStore blocker is now corroborated a third time,
   independently.** BC-AGRI-001's own Phase 0 — written before RM-001,
   RM-002, or the VDI grep session existed — already names "KISPU HeadStore
   fix (shared prerequisite with NANSHE/ESARHADDON)" pinned to **Playbook 98,
   immediate**. Combined with the VDI grep (zero HeadStore implementation
   found) and RM-002's own BLK-1 gate at PB-98, three independent sources now
   agree this is real, unresolved, and blocks more than one module.
3. **ESARHADDON's actual definition, previously unverified, is now
   confirmed**: earthquake rescue — structural collapse prediction and
   survivor signal fusion, ~31,000 PU/sec sensor mesh, sub-second response —
   matching the existing `crates/esarhaddon` (`survivor.rs`, `grid.rs`,
   `smi.rs`). This addendum previously could only confirm the crate exists,
   not what it does; BC-AGRI-001's suite table settles it.
4. **Open, not resolved here:** BC-AGRI-001 §4.1 assigns "Livestock Health
   Event" `kaki_role = 0x03 PARZU`, but PARZU is defined elsewhere (ADR-008
   Decision 3) as logic/template/rule, not a data event — a herd sensor
   reading reads more like ZIKRU (record). Flagged for the Architect's
   ruling, not silently corrected, since it may be an intentional modeling
   choice this addendum doesn't have context for.
5. A genuine, matched 90-day test corpus (`ashnan_*_90day.csv` × 4, four
   archive formats, plus nested/corrupt/7z edge cases) was supplied and
   verified to align with `naramsin-archive`'s supported formats and the
   existing `TESTING_PLAYBOOK_PHASE1.md` test pattern (recursion-depth,
   truncation, unsupported-format rejection). Real fixture material, not
   placeholder data.

## Part 5 — PAZUZU threat simulation, verified against the con-engine source

A complete `con-engine` crate (a separate, more developed design than the
repo's current `enkidb-con-engine` — see the ConEngine finding logged
separately in this session) and a matching PAZUZU-01→07 Godot threat
simulation (`PazuzuAgent.gd`, `OrbitalDegradation.gd`,
`PAZUZU_SIMULATION.md`) were supplied together. PAZUZU's premise is that each
numbered test targets one real, specific gap in that `con-engine` design —
this was independently checked line-by-line against the actual uploaded
source, not taken on faith:

| Test | Claimed gap | Verified against `con-engine` source |
|---|---|---|
| PAZUZU-01 | `StubCredentialStore` accepts a zeroed 32-byte blob unconditionally | **Confirmed.** `csr04_credentials.rs`'s `StubCredentialStore::get()` returns `Some(SealedCredential { blob: vec![0u8; 32] })` for any `session_id`, no validation |
| PAZUZU-02 | No `GilgameshPassport` issued to any session — CrossTribe fan-out unguarded | **Partially confirmed, correctly scoped.** `csr05_gilgamesh.rs`'s check logic is correct (rejects a Client without a passport) — this is an honestly-labeled *deployment* gap (no real passport issued yet), not a code defect |
| PAZUZU-03 | NĀRU-SYNC agent not running — WAL file unprotected between write and sync | **Confirmed** — `naru/mod.rs`'s own header comment states Phase 2 (NĀRU-SYNC) is a separate, not-yet-built process |
| PAZUZU-04 | No `max_connections` limit in `ConnectionPool` | **Confirmed** — `pool.rs`'s `connect_all()` has no capacity bound anywhere |
| PAZUZU-05 | No opcode whitelist in `send_frame()` | **Confirmed** — `PooledConnection::send_frame()` validates only the length-prefix framing, never the opcode |
| PAZUZU-06 | TIAMAT Engine 5 not wired to ConEngine session events | Not verifiable from the supplied files — TIAMAT wiring code wasn't part of this upload |
| PAZUZU-07 | UTNAPISHTIM sovereign application factory has no ConEngine gate at registration | Not verifiable — UTNAPISHTIM hasn't been supplied in this session |

Five of seven claims were directly, independently confirmed against real code
rather than asserted. This is a genuinely well-grounded security test design,
not flavor text over placeholder gaps — worth noting since it's the opposite
finding from most of this addendum's other entries.

**Two new inconsistencies found while verifying, corrected/flagged
separately:**

1. **A three-way TIAMAT alert-band naming conflict**, now flagged directly in
   `BAHYWAY_ECOSYSTEM_MANUAL_V4.md` §"TIAMAT → Data Tier → Alert Mapping":
   the repo's own canonical manual still lists `GREEN/DILBAT/KAKKAB/NERGAL/MAROON`,
   but `NERGAL` collides with the separately-named Nergal anti-virus engine
   (GLS-001). `BC-ENV-001` noticed this and renamed its own ladder entirely
   (`Stable/Watch/Serious/ERRA`); `PAZUZU_SIMULATION.md` noticed it too and
   swapped `NERGAL`→`ERRA` in place (`DILBAT/KAKKAB/ERRA/MAROON`). Neither
   later document cites or corrects the manual, so three different band
   namings are now in circulation. Not resolved here — needs one Architect
   ruling on the canonical ladder.
2. **A playbook-numbering collision with RM-002's own ledger.**
   `PAZUZU_SIMULATION.md` §8 proposes "Playbook 87: Deploy PAZUZU simulation
   scene... 88: Seed EnkiDB with test tribe particles... 89: Deploy NĀRU-SYNC
   agent stub... 90: Run PAZUZU-01→07." But RM-002 §2 already assigns
   **PB-88–97** to "Foundation gap closure" (v4.0 workspace migration, kaki-types,
   zakāru journal, NATIRU index, HeptaScript v1, ENLIL stack, TIAMAT Engine 4,
   marked WRITTEN) — entirely different content, same numbers. This is the
   third distinct playbook-numbering collision found this session (after
   Ashnan's PB-117 vs. Playbooks 156–160, and this one), reinforcing that
   numbering conflicts across parallel design sessions are systemic here, not
   isolated incidents. Needs the Architect to arbitrate before either
   proposal is built against.

## Part 6 — Esarhaddon and Diyala River (BC-QUAKE-001, BC-ENV-001), and an
## independent verification of PB-152's own claimed benchmark

### A genuine document-ID collision, not just a numbering mismatch

`BahyWay_BCENV001_Diyala_River.docx` — the NANSHE river-contamination business
case (sewage toxicity, H₂S dispersion, BMI mortality index) — is titled
**BUSINESS CASE BC-ENV-001**, dated June 2026. This is a completely different
document from the Enbilulu/WPDEngine **BC-ENV-001** already in this addendum's
Part 1–3 discussion (water pipeline deformation, dated 2026-07-07). Two
unrelated business cases claim the identical document ID. Corroborating
evidence this isn't a one-off typo: `BahyWay_Roadmap_Update_ESARHADDON.docx`
independently refers to *"the same KISPU HeadStore fix already pending for
NANSHE/BC-ENV-001"* — confirming NANSHE/Diyala is the ID's original holder,
and Enbilulu's later reuse of the same ID is the collision. Needs an Architect
ruling on which keeps `BC-ENV-001` and which gets renumbered — not resolved
here.

### A fourth playbook-numbering conflict, and now a clear pattern

Esarhaddon's own detailed roadmap (both `BahyWay_BCQUAKE001_ESARHADDON.docx`
and the roadmap-update doc agree) assigns: Phase 0 → PB-98 (shared); Phase 1
(KAKI schema) → **Playbooks 119–122**; Phase 2 (Survivor Signal Aggregator) →
**123–127**; Phase 3 (TIAMAT SMI + KIBRATU cascade) → **128–133**; Phase 4
(HeptaMap rescue grid) → **134–138**; Phase 5 (collapse forecast + gas
bridge) → **139–143**; Phase 6 (DubSar Theater scenes) → **144–150**; Phase 7
(field ruggedisation) → **151–155**. Compare RM-002's own ledger: **PB-119–136
= "ESARHADDON module series (compressed... 18 playbooks)"** and **PB-137–145 =
"Platform completion"** (UrNammu, Kittu, NINSUN, Ashnan regional extension,
NAMTAR, EREŠKIGAL, Grafana). Esarhaddon's own detailed plan needs 37 playbook
slots (119–155) just for itself — RM-002's compressed 18-slot estimate for the
same range, and its entirely different PB-137–145 content, both collide with
it. Combined with Ashnan's PB-117-vs-156–160 conflict and PAZUZU's
PB-87–90-vs-PB-88–97 conflict, this is now **four** independent
playbook-numbering conflicts found this session, all involving the same
RM-001/RM-002 compressed PB-111–145 ledger. The pattern is now unambiguous:
that compressed ledger does not match any of the detailed per-module roadmaps
it claims to summarize. Recommend treating RM-001/RM-002's PB-111–145 range as
unreliable for planning until reconciled against the actual per-module
documents, rather than continuing to discover collisions one module at a time.

### PB-152 (ENLIL Tribe HotIndex) — independently built and run, not just read

Unlike every other "SEALED ✓" crate this addendum has checked, PB-152 shipped
as a complete, self-contained Ansible playbook with the full Rust source
embedded (`Cargo.toml`, `hot_table.rs`, `batch.rs`, `bench_sweep.rs`, a 5-test
suite). This addendum extracted that source verbatim, built it, and ran it —
not just confirmed the crate exists.

- **`cargo test --release`: 5/5 passed**, matching the playbook's own gate
  (`failed_when: 'test result: ok. 5 passed' not in ...`). Correctness claims
  (exact invalid count, exact first-offender index) are real, not asserted.
- **Throughput: reproduced in spirit, not in the exact number.** RM-002
  states "2.41 B particles/s single-core." On this session's container (4
  cores, no stated AVX-512), single-thread measured **0.98 B particles/s** —
  the 1B-under-1s law does *not* hold single-threaded here, only at 4 threads
  (3.41 B particles/s). This doesn't contradict the original claim: the
  playbook's own header honestly scopes it to "single Xeon core @2.8 GHz
  (AVX-512)," and different hardware giving a different number for a
  branchless SIMD-shaped kernel is expected, not a red flag. But **RM-002's
  one-line summary drops that hardware qualifier**, presenting a
  hardware-specific measurement as if it were a universal guarantee — the
  same "compressed summary loses the caveat" failure mode found elsewhere in
  RM-002, just on an otherwise-genuine, correctly-built result rather than a
  fabricated one.

This is the most positive finding of this addendum: PB-152 is real, correct,
tested, and does what it claims *on its own stated hardware* — a useful
contrast to SumerEngine/NUZI/AsakkuEngine, which don't exist at all.

## Part 7 — PB-150/151 collide twice; the WPDEngine "11/11" mystery is now solved

Two more Ansible playbooks were supplied, and they resolve one long-standing
loose end from Part 2 of this addendum while opening two new direct
collisions.

### PB-150 claimed by two entirely unrelated components

- `PB150asakkuenginedeploy.yml` — "PB-150: AsakkuEngine Deployment," a
  systemd-service rollout of a pre-built AsakkuEngine binary to client
  Java/.NET hosts.
- `PB150_wpd_engine_v4.yml` — "PB-150 — WPD-Engine v4.0: Sovereign Water
  Pipeline Diagnostics Engine," a full acoustic leak-triangulation crate
  (Deming regression, GCC-PHAT correlation, dispersion compensation,
  joint-indexed snapping, Cramér–Rao error budget, 11-test suite).

Two structurally unrelated components, same playbook number, in files
supplied together. Not a compressed-vs-detailed mismatch like the earlier
findings — a direct, unambiguous collision.

### PB-151 collides too

`PB151asakkuenginebuilddeploy.yml` ("AsakkuEngine Build & Deploy From
Source," superseding PB-150's pre-built-binary step) collides with RM-002's
own ledger entry **"PB-151 | NUZI Tribe Genealogy Registry ... | SEALED ✓."**
Same number, two unrelated components, again.

### The WPDEngine "11/11 vs 45" discrepancy from Part 2 is now explained, not just noted

Part 2 of this addendum flagged that RM-002's "PB-150 WPDEngine... 11/11
tests" didn't match the real `crates/wpd-engine` in this repository, which
has **45** tests — and left it as an unexplained number mismatch. It is not
a mismatch of the same thing; it is **two different engines sharing one
name**. This addendum extracted `PB150_wpd_engine_v4.yml`'s complete embedded
Rust source (`material.rs`, `stage1_kingplot.rs`, `calibration.rs`,
`solver/{gcc_phat,dispersion,joint_index}.rs`, `crlb.rs`, `verdict.rs`,
`event.rs`) and built and ran it independently: **`cargo test` → 11/11
passed**, exact match to RM-002's claim.

But this crate's actual file structure (Deming/GCC-PHAT/CRLB acoustic
triangulation) shares almost nothing with the real `crates/wpd-engine`
already in this repository (`defect.rs`, `priority.rs`, `sector.rs`,
`segment.rs`, `spectral.rs`, `nav.rs`, `domain.rs`, `kaki.rs`) — a
structural-defect/priority-routing design, not an acoustic solver. Two
completely different implementations, both legitimately named "WPDEngine,"
both called "wpd-engine" as a crate. RM-002's "11/11" line refers to *this*
uploaded design (verified real and correct), not to the crate actually
sitting in `workspace/bahyway_v4/crates/wpd-engine` (which has its own real
45 tests, per Part 2). Neither is fake — they're simply two different things
with the same name, and RM-002 never disambiguates which one it means.

### NARAMSIN spec matches the real repo crates cleanly

`EN-NARAMSIN-001` (supplied as two identical `.docx` copies) specifies
exactly the five crates already present in this repository —
`naramsin-archive`, `naramsin-format`, `naramsin-integrity`, `naramsin-audit`,
`naramsin-bridge` — and its integrity-code taxonomy (`NRM_CLEAN`,
`NRM_TRUNCATED`, `NRM_MALFORMED`, etc.) explains exactly why the earlier
`ashnan_test_CORRUPT.zip` fixture (Part 4) was deliberately truncated: it's
the designed trigger for `NRM_TRUNCATED`. Good consistency — no correction
needed here, only confirmation.

---
*Verified against actual repository state — `cargo test` output, `find`/`grep` over
`workspace/bahyway_v4`, and direct reading of `KAKI_v4.0.1_canonical.pdf`,
`GL-MRD-001`, `BC-MRD-001`, `RM-002` as retrieved from Google Drive — not against
either document's self-report. Where they disagreed, this document defers to what the
repository actually contains.*
