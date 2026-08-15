# Orbital Layers Diagram Reference

> **DubSar Help** | `Observatory > Orbital Layers` | Diagrams

## What This Diagram Shows

The orbital layer diagram visualises Theorems PA-13 to PA-16:

- **Shells** (PA-13): logarithmically spaced rings, δ_T(p) = 1 − HPS(p).
- **Particle position** (PA-14): radius = δ · R_max, azimuth from KAKI byte 12,
  altitude from KAKI byte 14.
- **Tribe birthing** (PA-15): the outer rim spawning a child orbital system.
- **Rendering modes** (PA-16): PointSprite / Instanced / Volumetric by count.

## Asset Location

Place rendered `.svg` or `.png` files in this directory with the naming pattern:
`pa-{theorem_number}_{short_name}.{ext}` (e.g. `pa-14_particle_position.svg`).

## See Also

- `09_observatory/shell_decomposition.md`
- `09_observatory/particle_position.md`
