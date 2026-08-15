# Particle Position in Orbital Space — PA-14

> **DubSar Help** | `Observatory > Particle Position` | Observatory

## Purpose

PA-14 defines the 3D position of every particle as a pure function of its KAKI
and the Tribe Ideal. No layout algorithm, no force-directed placement.

## Formula

```
radius(p)   = δ_T(p) · R_max
azimuth(p)  = 2π · κ(p)[12] / 256      ← KAKI byte 12 (D6 timestamp high)
altitude(p) = (κ(p)[14] / 256 − 0.5) · H_max  ← KAKI byte 14 (D7 integrity)
```

## Properties

- **Deterministic**: same KAKI + same Tribe Ideal → same position, always.
- **Sovereign**: orbital position follows from identity, not from circumstance.
- **Temporal locality**: particles created at similar times cluster at similar
  azimuths (KAKI byte 12 is derived from the D6 timestamp).
- **Integrity stratification**: particles with similar checksums sit at similar
  altitudes (KAKI byte 14 is derived from the D7 integrity checksum).

## See Also

- `09_observatory/shell_decomposition.md`
- `09_observatory/rendering_modes.md`
- `01_mathematics/orbital_theorems.md`
