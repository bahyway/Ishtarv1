# Orbital Visualization — Overview

> **DubSar Help** | `Observatory` | Observatory

## Purpose

The orbital visualization renders every particle in a Tribe as a point in 3D
space, where position is a pure function of the particle's KAKI and the Tribe
Ideal (§PA-14). Shell membership is determined by quality distance (§PA-13).

## Architecture

- **Shell decomposition** (PA-13): logarithmic rings, inner = gold, outer = dead.
- **Particle position** (PA-14): (radius, azimuth, altitude) from KAKI bytes.
- **Tribe birthing** (PA-15): outer-rim child tribes visible as sub-ring systems.
- **Rendering modes** (PA-16): PointSprite / Instanced / Volumetric.
- **Click-to-storytelling**: every particle retains its KAKI in all render modes;
  click → KAKI → full storytelling journal in StoryWay.

## See Also

- `09_observatory/shell_decomposition.md`
- `09_observatory/particle_position.md`
- `09_observatory/tribe_birthing.md`
- `09_observatory/rendering_modes.md`
- `09_observatory/hubble_zoom.md`
- `_diagrams/orbital_layers_diagram.md`
