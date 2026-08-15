# Multi-Scale Rendering Modes — PA-16

> **DubSar Help** | `Observatory > Rendering` | Observatory

## Purpose

PA-16 defines three rendering modes that switch automatically based on the
particle count in a shell (|Σ_k|). In all three modes, every particle retains
its KAKI — no information is lost.

## The Three Modes

### PointSprite (|Σ_k| < 10,000)
- Each particle is a textured quad with KAKI-derived colour.
- Click-to-storytelling: direct KAKI lookup.
- Used for: small tribes, inner gold-quality shells, zoomed-in views.

### Instanced (10,000 ≤ |Σ_k| < 1,000,000)
- GPU-side instanced rendering; tens of thousands per draw call.
- Click-to-storytelling: picking buffer maps screen pixel → KAKI.
- Used for: mid-density shells.

### Volumetric (|Σ_k| ≥ 1,000,000)
- Density field accumulated by compute shader; ray-marched for display.
- Click-to-storytelling: clicking a region triggers a zoom-in that switches
  to Instanced or PointSprite mode. 2–3 clicks to reach any individual KAKI.
- Used for: high-density outer shells where individual particles are visually
  indistinguishable.

## The Storytelling Preservation Theorem

> In all three rendering modes, every particle retains a queryable KAKI.
> Volumetric rendering is a compression for visual perception, not for the data.

## See Also

- `09_observatory/orbital_visualization.md`
- `09_observatory/hubble_zoom.md`
