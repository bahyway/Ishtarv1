# Mathematical Axioms

> **DubSar Help** | `Codex > Axioms` | Non-Negotiables

## Axiom 1 — Particle Identity

For any particle p, there exists a unique KAKI κ(p) ∈ {0..255}^16 such that
κ(p) is computable in O(1) and is stable across all storage locations.

## Axiom 2 — Tribe Ideal

For any tribe T, there exists a Tribe Ideal vector ι_T ∈ [0,1]^7 that defines
the target state for all particles in T.

## Axiom 3 — Quality Distance

δ_T(p) = 1 − HPS(p), where HPS is the Hepta Priority Score. δ_T(p) = 0 means
perfect alignment; δ_T(p) = 1 means maximum divergence.

## Axiom 4 — Pauli Exclusion

No two particles may occupy the same (Identity-KAKI, Orbit-Level, QuantumState)
triplet simultaneously within a Tribe.

## See Also

- `01_mathematics/enlil_algebra.md`
- `04_gates/pauli_exclusion.md`
