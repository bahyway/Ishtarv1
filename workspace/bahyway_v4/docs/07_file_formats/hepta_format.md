# .hepta File Format

> **DubSar Help** | `.hepta` | File Formats

## Purpose

The .hepta file stores the 7D Hepta vector snapshot for a particle or tribe.
It is the persisted form of the Heptagon quality score.

## Structure

7 × f32 values in little-endian binary, one per dimension:

| Byte offset | Dimension |
| :--- | :--- |
| 0–3 | D1 |
| 4–7 | D2 |
| 8–11 | D3 |
| 12–15 | D4 |
| 16–19 | D5 |
| 20–23 | D6 (timestamp-derived) |
| 24–27 | D7 (integrity checksum-derived) |

## See Also

- `01_mathematics/particles_algebra.md`
- `09_observatory/particle_position.md`
