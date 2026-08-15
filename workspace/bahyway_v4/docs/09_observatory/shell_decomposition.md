# Shell Decomposition — PA-13

> **DubSar Help** | `Observatory > Shells` | Observatory

## Purpose

PA-13 defines how every particle in a Tribe is assigned to exactly one orbital
shell based on its quality distance from the Tribe Ideal.

## Formal Definition

```
δ_T(p) = 1 − HPS(p)          ∈ [0, 1]
r_k    = 1 − (1 − ε)^k       (logarithmic shell radii)
Σ_k    = {p ∈ T : r_k ≤ δ_T(p) < r_{k+1}}
```

- δ_T(p) = 0 → perfect alignment with Tribe Ideal (Golden Record).
- δ_T(p) = 1 → maximum divergence (Dead Particle).
- Shell membership is O(1) from δ_T(p) — no iteration over other particles.

## DubSar IDE Hover

When you hover over a shell ring in the Observatory, the tooltip shows:
- Shell index k
- Shell radius range [r_k, r_{k+1})
- Particle count |Σ_k|
- Current render mode for this shell

## See Also

- `09_observatory/particle_position.md`
- `01_mathematics/orbital_theorems.md`
