# BahyWay.Ecosystem v4.0 — All PBs Roadmap

> "الحالة القديمة لا تُمحى أبدًا" — The old state is never erased.

---

## Honest scope of this document

There is no static "master PB roadmap" file anywhere in the `bahyway/EnkiDB`
repository, and there never has been one — checked directly: `playbooks/`
is not a tracked directory on any branch of this repo, and
`docs/16_runbooks/PLAYBOOK_EXECUTION_TRIAGE.md` (the triage doc `crates/enkimdb::pb`
and `crates/anu-governor::runner` are written to consume, per their own
module docs) does not exist in this checkout either. That is by design,
not an oversight: the real, numbered `playbook_<N>_*.yml` corpus Shakkanakku
runs against lives only where it is actually executed — the Architect's own
control-node hardware (see PB-269 below) — never committed to the shared
GitHub repository.

What this document **is**: a real roadmap reconstructed from the only PB
evidence that *does* live in this repo — the 15 commits in this
repository's own history (across all branches) whose message starts with
a real `PB-<number>` tag — plus the forward milestones from
`docs/19_roadmap/BAHYWAY_ECOSYSTEM_V4_ROADMAP.md`, the one static roadmap document
that does exist here.

What this document **is not**: a complete enumeration of every PB the
Architect has ever run. Many PBs referenced by number inside these same
commit bodies (PB-210, PB-212, PB-213, PB-265) never got their own
`PB-<n>: ...` commit subject in this repo and are not separately listed
below — they are real (cited by the commits that *do* have their own
entries), just not independently commit-tagged here.

---

## Part 1 — PBs with a real commit in this repository (chronological)

| PB | Date | Commit | Summary |
|----|------|--------|---------|
| PB-203.2 | 2026-07-26 | `6f669ae` | Fix health check/backup for the real 2-VM CQRS split — PB-203 (2026-07-19) was written for the old single-host topology; PB-212 later moved the fleet to `enkidb-node-write`/`enkidb-node-read` and PB-203 never followed, so health checks reported all containers "NOT RUNNING" against the wrong host. |
| PB-203.3 | 2026-07-26 | `d1f6dc4` | Fix rootless-Podman permission wall in health check/backup — rootless Podman remaps container UIDs into a subuid namespace; plain `stat`/`tar` against the volume mountpoint hit real `Permission denied`. Same wall PB-213/258 had already solved for `cp`/`rm`/`mv`. |
| PB-203.4 | 2026-07-26 | `4300b15` | Fix backup staging shape to match PB-213/258 exactly — PB-203.3's fix passed 1.1–1.7 for real, then hit its own `Permission denied` during the staged-copy `tar`; root cause was not reproducing PB-213/258's actually-working `podman unshare cp -a` shape. |
| PB-258 | 2026-07-26 | `8ae20e4` | Real citation-affinity topic graph over EnkiDDB, honest rebuild — evaluated an uploaded "Knowledge Graph via Dynamic Orbits" design against this repo's real state; kept the sound technique (citation-affinity clustering), rejected the draft's synthetic fixture and overclaimed scope. |
| PB-226 | 2026-07-28 | `82c1399` | Rebuild+copy all 6 dubsar-theater GDExtension bridges, not just kupru — only `kupru-gdext` was ever rebuilt on every prior PB-226 run; the other five (`marduk-gdext`, `naming-registry-gd`, `navi-translate-gdext`, `dubsar-gridnav-gd`, `enkimdb-registry-gd`) were stale until PB-262's from-source Godot swap made that visible. |
| PB-226 | 2026-07-28 | `98f0bd5` | Kill any stale DubSar instance before launching a new one — every prior run left the previous background Godot process running and stacked a new one on top; on `eriduous-vdi` (no real GPU, software rendering) that compounded into visible resource pressure. |
| PB-226 | 2026-07-28 | `ded1994` | Force `gl_compatibility`/opengl3 renderer — `eriduous-vdi` has no real GPU (`vulkaninfo` showed `VK_ERROR_INCOMPATIBLE_DRIVER`, `/dev/dri` has no render node), confirmed live. |
| PB-262 | 2026-07-28 | `f48df66` | Add glslang `<cstdint>` fix, correct idempotency bugs — real from-source Godot 4.3 build on `eriduous-vdi` (GCC 16.1.1) failed on a missing transitively-included header that GCC 13 (sandbox) tolerated. |
| PB-262 | 2026-07-28 | `4f3710b` | Add thorvg `<cstdint>` fix (patch 5, same GCC-16 bug class as glslang) — a second independent vendored-library missing-include bug, same root cause class as the glslang fix. |
| PB-262 | 2026-07-28 | `3dc93f4` | Detect dnf vs apt for the missing-tool install hint — preflight error message hardcoded `apt-get`, wrong on `eriduous-vdi` (Fedora/dnf). |
| PB-262 | 2026-07-28 | `e732e5e` | Disable `use_static_cpp` to fix link failure on Fedora — Godot's default static libstdc++ link needs Fedora's separate `libstdc++-static` package, not installed on `eriduous-vdi`. |
| PB-268 | 2026-07-29 | `0cf491b` | Create the Architect's 5 host privilege groups (Architects / DataStewards / Administrators / Developers / Other-Stakeholders); wire real OS identity into Shakkanakku's run-confirmation registry — closes a gap where a Shakkanakku run with no vault configured had no real identity in the confirmation loop. |
| PB-269 | 2026-07-30 | `750e9ef` | **Retire EriduOS-VDI as control-node hardware for v4.0** — the KVM/libvirt VM at `192.168.122.214` (control node since PB-210/212) is no longer used; the control-node role moves to the bare-metal **Fedora Workstation 44** host (PB-265/268 already reference it). `enkidb-node-write`/`enkidb-node-read` (`192.168.122.101`/`.107`) are explicitly unchanged — the 3-host topology is kept, only the control node moves off virtualized hardware. |
| PB-270 | 2026-07-30 | `4a5ea3b` | Make Shakkanakku the one central KAKI v4.0 tool — every real Corpus run now mints playbooks (pre-existing), workspace crates (new `crate_mint.rs`, via `enkimdb::scan_crates` + `ingest_artifact`), and `.akk`/`.way`/`.tmpl` tablets (new `tablet_mint.rs`) into EnkiMDB — one tool instead of three. |
| PB-271 | 2026-07-30 | `a53e736` | Build the assets-node Podman image (pinned Rust 1.94.1 toolchain, vendored crate sources); mint playbook documentation into EnkiDDB. |

**Bare-metal transition, confirmed from real commits above:** PB-269 is
the actual, executed cut-over the Architect described this session — the
control node moved from the virtualized `EriduOS-VDI` box to the
Architect's own **Fedora Workstation 44 (W44)** bare-metal host, while the
write/read EnkiDB node pair stays exactly where it was. Every PB filed
after PB-269 in this table (PB-270, PB-271) runs in that new bare-metal
reality.

---

## Part 2 — Shala4 Gate Orbits & Playbook Dependency Discovery (non-PB application-feature commits)

Not a `PB-<n>`-tagged operational run like Part 1 above — this is direct
application-feature development against the Shala dashboard and EnkiDDB
engine itself, real and chronologically dated (2026-08-02) but outside the
"playbook run" convention Part 1 documents. Listed separately, honestly,
rather than folded into Part 1 or omitted.

| Commit | Summary |
|--------|---------|
| `ef348b0` | **Phase 0-3**: fix a real EnkiDDB generation-completeness gap (`pb_catalog.rs` was silently dropping a previously-catalogued playbook's own content on any rerun after its first); add `WriteNode::tag_gate` + `gate_review.rs` suggest-then-approve scanner; add `/api/gates/*` Shala endpoints; ship the Shala4 "Gate Orbits" tab — playbooks browsable first as the 7 real `bahyway_core::hepta_gate::HeptaGate` sectors (Apsu/Adad/Shedu/Mummu/Enkidu/Dubsar/Enlil) instead of one flat ~900-node graph, each rendered as its own Three.js particle-orbit scene. |
| `b720868` | **Phase 4**: add `WriteNode::tag_domain` + `domain_review.rs`; 7×7 domain taxonomy (7 real domains per gate, 49 total); `/api/gates/<gate>/domains/*` endpoints; station/cube-stack visual with a real StoryEngine detail panel (every attribute shown is live EnkiDDB data, nothing simulated, unlike the uploaded ETL Transparency Dashboard prototype this was adapted from). |
| `f2f0550` | Make Gate Orbits *viewing* Public (no passport needed), matching Box Resources' access model — the Architect-only Review/Approve flows stay gated. |
| `838fe45`, `dc65245` | Add free-text playbook search (title/path, then extended to full body text) with results plotted as a 3D cube grid alongside the existing text list — HeptaScript's `WHERE` clause has no substring operator, so this reads directly from `pb_catalog`'s own registry in Rust rather than a HeptaScript query. |
| `11452c2` | **Playbook dependency discovery**: `pb_dependency_review.rs` scans every catalogued playbook's own text for a real whole-word mention of another catalogued playbook's identity (file stem, or `PB-<n>`/`PB<n>`/`playbook_<n>` forms) — same suggest-then-approve discipline as gate/domain classification. Approved pairs mint a real `depends-on` edge via `WriteNode::mint_link_edge` (the existing CrossTribe-Kaki link primitive). Shala's StoryEngine panel renders a playbook's `depends_on`/`depended_on_by` lists and a 3D orbit view (root as Tribe at center, dependencies as an orange ring, dependents as a purple ring, each clickable to recurse into its own StoryEngine view) — the real, un-simulated answer to "discover the root as Tribe and its related Particles in multiple layers as Orbits, till the last Particle." |

Also on 2026-08-02, outside this table: `363e0e1` fixed Graph Explorer's
root node to fill the DubSar Theater viewport instead of a fixed small box
(the maximize bug that motivated browsing playbooks by gate in the first
place, rather than as one flat graph).

### Planned next step — PB Compare State (designed 2026-08-02, not yet built)

Comparing 2+ variants of "the same" playbook (e.g. across branches or prior
runs) must not collapse to a binary "matches current repo code = keep,
otherwise = ignore" — a variant not currently in the repo may carry
legitimate code not yet merged, or be a deliberately superseded version
worth keeping for history, not silently discarding either way. Designed
(not yet implemented) as a new single-scalar `meta.compare_state` particle
per playbook variant, one of 8 `Shala_<State>` values (see the Glossary
entry for the full list and meanings), assigned by the same suggest-then-
approve discipline as gate/domain/dependency tagging — a scanner proposes
a state by comparing a variant's text/hash against the code currently
checked into this repo, an Architect approves or overrides it, nothing is
auto-decided. A transition from `Shala_Golden` to `Shala_Deprecated` also
calls the existing `WriteNode::supersede_document` (ADR-014's law, already
live in `pb_doc_mint.rs`) so the prior Golden's full identity chain stays
queryable, never erased.

---

## Part 3 — Forward roadmap (from `docs/19_roadmap/BAHYWAY_ECOSYSTEM_V4_ROADMAP.md`)

The static roadmap already in this repo defines the scale gates every
future PB run is measured against. Reproduced here (see that document for
the full detail, including the per-phase test matrix and the §0.3
sovereign constraints):

| Phase | Scale / Gate | Status as of this document |
|-------|--------------|------------------------------|
| Phase 0 — Foundation | Core crates (bahyway-crc, enkidb-kaki, enkidb-journal, enkidu-protocol, heptascript v2.0, enkidb-indexes, naramsin-archive/format, enkidb-session-registry, enkidb-con-engine) | Complete |
| Phase 1 — BeeMDM Testing | 50 compressed files, 10 million particles, HeptaScript `execute_stream` < 1 second on all 7 EnkiDB types | Superseded in practice by the real `bin/bee-watchdog` + `eridu-runtime::SchedulerLoop` chain built this session — this roadmap phase predates that architecture and should be read as historical intent, not current test target |
| Phase 2 — Scale Testing | 100 million particles, NatiruIndex sharding, PAZUZU simulation (7 threat tests) | Not yet run against the current architecture |
| Phase 3 — Billion-Particle Production | 1 billion particles, HeptaScript < 1 second on all 7 EnkiDB types for any query pattern | This is the gate `bin/najaf-gen` (built this session) and the step-by-step test guide in `docs/testing/BEEMDM_1B_NAJAF_TEST_GUIDE.md` (built alongside this document) target directly |
| Phase 4 — BeeMDM ETL | 6-station pipeline operational | The real chain is `bin/bee-watchdog`'s station sequence (LandingZone → Musarû → VGCA-Δ → ZIP extraction → ProcessingZone → BatchSchema → per-record adad-gate → DataStructureStation → data-cleansing → VGCA beam → client-dq-profile → score-engine → B11 routing), already built and tested, distinct in detail from this older roadmap's 6-station sketch |
| Phase 5 — AI Agents | Post-BeeMDM-ETL, gated on the Phase 3 HeptaScript pass | Not yet started |

---

## Part 4 — What is genuinely missing from this repo

For a complete "All PBs Roadmap" to exist as a single committed document,
one of the following would need to happen, and neither has yet:

1. The Architect exports (or points Shakkanakku's `--triage-doc` flag at)
   a real `docs/16_runbooks/PLAYBOOK_EXECUTION_TRIAGE.md` from the Fedora W44 host,
   and it gets committed here (or ingested into EnkiDDB directly per
   Part 4 below, without ever touching the shared GitHub repo — see
   `docs/20_meta_engine/URUINIMGINA_EXTERNAL_DOCS.md` for exactly how).
2. `enkimdb::pb::scan_pbs` is run live against the real `playbooks/`
   directory on that host and its output is captured as a document.

Until then, Part 1 above — real commits, not fabricated PB numbers — is
the accurate, honestly-scoped answer to "what PBs are documented in this
repository."
