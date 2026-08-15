# CrossTribe-KAKI

> **DubSar Help** | `κ_ct` | Identity

## Purpose

The CrossTribe-KAKI encodes the relationship between two particles belonging to
different Tribes. It is the algebraic "bridge" between Jordan Blocks.

## Mechanism

**Corrected 2026-07-07** — the previous revision of this document conflated two
separate things under one claim ("never persisted... materialised on PROBE
only"). They are not the same fact and do not share the same rule:

- **Identity is persisted, permanently.** A CrossTribe-Kaki's identity payload
  (which anchor particles/tribes it links) is minted once and stored — never
  deleted, never recomputed. This is a physical record in EnkiDB, exactly like
  any other particle's Identity-Kaki.
- **Effective state is never persisted.** Only the *derived health* of the
  link (Gold/Orange/Gray, per the IDU Probing Rule, ADR-008 Decision 6) is
  computed at query time and never stored. This is the part §8.3 actually
  governs.
- The "basis-transformation matrix P / P⁻¹" mechanism described in earlier
  drafts of this document was never implemented (`idu-prober::crosstribe`
  ships only the discrete anchor-state composition, `compose_n_anchors`) and
  is superseded by **ADR-011 (Amelu: Tribe-to-Tribe Connection Particles)**
  for any relationship dense enough to need geometric representation rather
  than a discrete edge.
- The ANU Gate governs which Tribe holds authority when CrossTribe particles
  conflict.

## Sovereign Constraints

§8.3: CrossTribe *effective state* is computed on PROBE, never stored. The
CrossTribe-Kaki's *identity* (which anchors/tribes it links) is a stored,
permanent record — see ADR-011 for the reorg-safe design (anchor/Tribe-Journal
pointers only, never tribe names or snapshots).

## See Also

- `02_identity/kaki_triad.md`
- `04_gates/anu_gate.md`
- `01_mathematics/tri_kaki_index.md`
- `14_decisions_adr/adr_011_amelu_tribe_connections.md`
