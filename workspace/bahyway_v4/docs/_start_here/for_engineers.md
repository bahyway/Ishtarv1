# Start Here — Engineers

> **DubSar Help** | `Help > Engineers` | Onboarding

## KAKI — The Identity Primitive You Will Use Every Day

**KAKI (Knowledge–Akkadian–Keyword–Identity)** — a 16-byte sovereign seal.
Every particle you create, every event you append, every query you write
is grounded in a KAKI PK. Understanding the byte layout is not optional.

```
κ[0..3]   uuid_hash    FNV-1a of particle content at birth
κ[4..5]   tribe_id     tribe scope
κ[6]      kaki_type    0x01 Identity · 0x02 Event · 0x03 CrossTribe
κ[7]      kaki_role    0x01 KISHIB · 0x02 ZIKRU · 0x03 PARZU
κ[8..11]  seq_counter  deterministic creation ordinal within tribe-epoch
κ[12..13] timestamp    sovereign epoch at birth
κ[14..15] checksum     CRC-16/CCITT over κ[0..13]
```

Entity resolution is **deterministic** — no ML, no probabilistic matching.
Canonical declaration: `docs/00_codex/glossary.md` | Byte layout spec: `ADR-003`.

---

## Reading Path

1. `02_identity/kaki_triad.md` — the three KAKI types.
2. `01_mathematics/tri_kaki_index.md` — O(1) indexing strategy.
3. `05_storage/enkidb.md` — storage crate interface.
4. `07_file_formats/akk_format.md` — the .akk law file grammar.
5. `11_tooling/nabu_cli.md` — the CLI reference.
6. `08_pipeline_alaktu/submission.md` — the CI/CD pipeline.

## Purpose

Engineers use this tree to locate API contracts, file format grammars,
and crate dependency maps before writing code.

## Sovereign Constraints

Pure Rust only. No third-party databases. No ORM.
