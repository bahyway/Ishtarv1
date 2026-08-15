# The KAKI Triad

> **DubSar Help** | `Identity > KAKI Triad` | Identity

## Purpose

The KAKI Triad is the three-way classification of all KAKI types in BahyWay.
Each type maps to a different indexing strategy and a different algebraic role.

## The Three Types

| Type | Algebraic Role | Index Strategy |
| :--- | :--- | :--- |
| **Identity-KAKI** | Nucleus (Eigenvalue λ) | O(1) spectral hash pointer |
| **Events-KAKI** | Orbit tail (Jordan chain entry) | O(1) LRU tail append |
| **CrossTribe-KAKI** | Basis transformation (P matrix) | O(1) matrix multiply (PROBE-only) |

## Sovereign Constraints

§2.4: No assessments in KAKI nucleus.
§8.3: CrossTribe state computed on PROBE, never stored.

## See Also

- `02_identity/identity_kaki.md`
- `02_identity/events_kaki.md`
- `02_identity/crosstribe_kaki.md`
- `01_mathematics/tri_kaki_index.md`
