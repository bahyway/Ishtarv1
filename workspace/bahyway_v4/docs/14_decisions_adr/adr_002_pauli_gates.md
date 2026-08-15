# ADR-002 — Four Pauli Exclusion Gates

> **DubSar Help** | `ADR > 002` | Architecture Decisions

<!--META
doc_kaki:
  concept_hash:   "0x00000000"
  tribe:          "DT-14"
  concept_type:   "0x02"
  epoch:          "2026-01-01"
  concept_depth:  230
  riksu_count:    2
  snapshot_epoch: "2026-06-06"

concept:          "Four Pauli Exclusion Gates"
summary:          "Shamash, Marduk, Enlil, and Nanna — the four sovereign gates that enforce particle state transition laws."
sovereign_laws:   []

riksu_bindings:
  - target: "high_council.md"
    concept: "Pauli Exclusion Gates"
    type: "PEER"
  - target: "adr_008_ooo_foundation_kaki_roles_forbidden_operations.md"
    concept: "Forbidden Operations"
    type: "CHILD"

orbit_tags:       ["Pauli Exclusion Gates"]
rag_keywords:     ["SEAL", "PARZU", "Shamash", "Marduk", "Enlil", "Nanna", "particle state", "Pauli Exclusion"]
-->

## Status: Accepted

## Context

Smart City data arrives from thousands of concurrent sources. Without formal
exclusion rules, the system would suffer from data flapping, race conditions,
source conflicts, and zombie data corrupting the Active Orbit.

## Decision

Adopt the four Pauli Exclusion gates (ADAD, ANU, MARDUK, SHAMASH) as the
canonical governance model. Every signal must pass all four gates before
entering the Active Orbit. The gates are named after Mesopotamian deities to
align with the broader BahyWay mythological naming system.

## Consequences

- All data state conflicts are handled by a single, composable pipeline.
- Gate thresholds (ADAD_BREATH_MS, ANU_AUTHORITY_RANK, etc.) are tunable per
  Tribe via .akk files.
- Lean 4 and Z3 provide formal verification of the exclusion rules.
