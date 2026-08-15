# Orbital Layer Theorems — PA-13 to PA-16

> **DubSar Help** | `Math > Orbital Theorems` | Mathematics

## PA-13 — Orbital Layer Decomposition

Quality distance: δ_T(p) = 1 − HPS(p) ∈ [0,1].

Logarithmic shell radii: r_k = 1 − (1 − ε)^k.

Shell membership is computable in O(1) from δ_T(p) alone — no neighbor
inspection, no global state.

## PA-14 — Particle Position in Orbital Space

```
radius(p)  = δ_T(p) · R_max
azimuth(p) = 2π · κ(p)[12] / 256      (KAKI byte 12: D6 timestamp high)
altitude(p) = (κ(p)[14]/256 − 0.5) · H_max  (KAKI byte 14: D7 integrity)
```

Position is a pure function of (κ(p), ι_T). Two particles with the same KAKI
in the same tribe always occupy the same orbital position.

## PA-15 — Tribe Birthing on the Rim

ι_{T_child} = lim Σ_T(Q_n) where Q_n ⊂ T_parent ∩ Σ_{N-1}.

Outer-rim particles with emergent coherence compose a new child tribe. Their
KAKIs do not change; their storytelling journals continue unbroken.

## PA-16 — Multi-Scale Particle Rendering

| Shell count | Render mode |
| :--- | :--- |
| < 10,000 | PointSprite — per-particle textured quad. |
| 10,000 – 999,999 | Instanced — GPU-side instanced draw calls. |
| ≥ 1,000,000 | Volumetric — compute-shader density field + ray march. |

In all three modes, every particle retains a queryable KAKI. Volumetric
rendering is a perceptual compression, not an information loss.

## See Also

- `09_observatory/shell_decomposition.md`
- `09_observatory/particle_position.md`
- `09_observatory/rendering_modes.md`
- `_diagrams/orbital_layers_diagram.md`
