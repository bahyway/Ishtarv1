# Mon20260817 Delivery — Landed and Verified — 2026-08-21

Reconciled and landed the five Mon20260817 delivery packages that were
staged byte-for-byte at `docs/mon20260817-incoming/` earlier this session:
Nasaru Visualization Style Default, Nergal AV, Sala BadTuri, Shala Najaf
Navigator, and Triple-O OntoGraph. The staging directory is kept as the
untouched original record, matching the `docs/phase2-incoming/` convention.

## Renumbered on landing (ID collisions with already-sealed tablets)

Three of the five packages' law tablets collided with IDs already sealed
elsewhere in this repo. Each was renumbered to the next free number in its
series and every cross-reference (including in the companion Triple-O
Manual, Triple-O Glossary, and BahyWay.Ecosystem Manual) was updated to
match:

| Delivered as | Collides with | Landed as |
|---|---|---|
| `GL-ONT-002` — The Non-Substitution Law | `GL-ONT-002` — Phase 0 Recognizer Law (`docs/01_mathematics/GL-ONT-002_Phase0_Recognizer_Law_DRAFT.md`) | `docs/01_mathematics/GL-ONT-003-non-substitution-law.md` |
| `GL-NAV-001` — The Ninĝišzida Engine | `GL-NAV-001`/`GL-NAV-002` already claimed (flight-to-location, Hendursaga AnnexA, knowledge-graph-navigation) | `docs/09_observatory/GL-NAV-003-ningiszida-engine.md` |
| `GL-VIZ-001` — Nasaru Default Visualizing Style | `GL-VIZ-001` — Bivector Orbit Encoding / BUZU chunk (`docs/07_file_formats/GL-VIZ-001.md`); `GL-VIZ-000`, `002`–`008` also already claimed | `docs/09_observatory/GL-VIZ-009-nasaru-default-style.md` |

`GL-NRG-001` and `GL-NRG-001-A2` (Nergal Transport Calculus, BabTurdiEngine)
had no collision and landed under their original IDs at
`docs/18_security/GL-NRG-001-nergal-transport-calculus.md` and
`docs/18_security/GL-NRG-001-A2-babturdi-engine.md`. The older draft
revision of `GL-ONT-001` bundled in the Triple-O OntoGraph package was
**not** landed as a duplicate — it predates and is a strict subset of the
already-sealed `docs/01_mathematics/GL-ONT-001_OntoGraph_Unified_Pattern_Law.md`
(missing the PB-322 verified-landing details); it remains available for the
historical record at `docs/mon20260817-incoming/tripleo_ontograph/`.

The Triple-O Manual, Triple-O Glossary, and BahyWay.Ecosystem Manual landed
at `docs/00_codex/`.

## Three new crates, landed and verified

PB-344, PB-348, and PB-350 (all previously free playbook numbers — the
340–419 range was entirely open) scaffold three real Rust crates, now
committed as flat workspace members of `workspace/bahyway_v4` alongside
`ontograph`:

- **`crates/nergal-transport`** (GL-NRG-001) — Monte Carlo transport on a
  sector graph; k_eff criticality, shielding, importance scanning.
  `cargo test -p nergal-transport`: 4/4 passing.
- **`crates/babturdi-engine`** (GL-NRG-001-A2) — semantic cost-attribution
  over the refused-record ledger. `cargo test -p babturdi-engine`: 4/4
  passing.
- **`crates/ningiszida`** (GL-NAV-003) — offline BFS navigation over a
  heptagonal graveyard field. `cargo test -p ningiszida`: 4/4 passing.

Following the playbook_322 precedent, `playbook_344`, `playbook_348`, and
`playbook_350` are landed as **verify** playbooks (crate presence,
workspace membership, `cargo test`), not scaffold-from-embedded-YAML
playbooks — the crates are real, committed, tested source now, so
re-materializing them on every run would silently overwrite hand-edits.

### Three real bugs found and fixed during landing (none of this code had ever been run before)

1. **`babturdi-engine::cohort_median`** took the *upper* of the two middle
   values for an even-sized cohort. With exactly two clients this collapses
   the median to the costlier client's own cost, so that client could never
   exceed "the median" and law test L63 always passed it as
   `LowCostEarned` regardless of how costly it actually was. Fixed to
   average the two middle values (the standard even-count median).
2. **`nergal-transport` law test L46** asserted that 15/20 seeded outbreaks
   invade a k_eff = 1.2 medium. The true branching-process survival
   probability for that offspring distribution is exactly 1 − q = 2/3
   (q solves q = 0.1 + 0.6q + 0.3q²), and the first 20 SplitMix64 seeds
   happen to sample low (7/20) — ordinary variance at n = 20, not a defect
   in the transport code. Fixed by widening to 200 trials (empirically
   66.6% invasion rate, matching theory exactly) and asserting ≥50%, which
   holds with real margin under the true rate.
3. **`ningiszida` law test L70b** asked `navigate()` to route between two
   corner grave cells, (0,0) and (4,4), that have no road-adjacent neighbour
   at all in the 5×5 test snapshot — they are graph-disconnected from the
   road network by construction, so `navigate` correctly returned `None`
   and the test's own `is_some()` assertion could never pass. Fixed by
   routing between (1,1) and (3,3), graves that do each border the road
   cross, which is what the test actually intended to check.

## Šala prototypes landed

`shala-prototypes/batch9_mon20260817_delivery/` (3 HTML rehearsal courts,
renumbered law-ID references patched to match) plus `nasaru_style.js` at
the `shala-prototypes/` root as a new shared interaction-grammar core,
alongside the existing `shala_charter.css`. `shala-prototypes/INDEX.md`
updated (122 → 125 files, 7 → 8 batches).
