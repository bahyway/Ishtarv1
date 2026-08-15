# GL-VIZ-001 — Bivector Orbit Encoding and the BUZU Chunk

Status: §1 (the encoding law) SEALED and PROVEN — landed by
PB-255 as `crates/buzu-core`, 7/7 tests passing on real
geometric algebra, not just described. §2 (the BUZU chunk
format) and §3 (FUZZY packed encoding) are now ALSO SEALED —
landed by PB-256 as `crates/buzu-core::chunk`, 15/15 tests
passing total (7 from §1 + 8 new). D1-D3 (§5) are RATIFIED,
not defaulted — see §5 for the designs chosen and why.

## 1. The encoding law (SEALED, PROVEN)

Rendering the billion does not move positions; it moves laws
of motion. Each orbit is encoded by its bivector B (the
orbital plane element of geometric algebra); the GPU forms
the rotor R = exp(-B*theta/2) and carries particles around
their orbits parametrically. Per-frame CPU->GPU traffic
reduces to theta (and view uniforms) — not a stream of
positions.

    We do not draw a billion positions;
    we draw orbits, and particles are phases upon them.

This is Triple-O rendered natively: the orbit is the
first-class object of the ontology and of the renderer
alike. Puhu reading: the bivector is the pattern (shared by
every particle on the orbit); the phase is the occupant.

**Implementation:** `crates/buzu-core` (workspace member,
depends only on `bahyway-algebra`'s already-tested Cl(7,0)
`Multivector` geometric product — no third-party crate, no
hand-derived formula independent of that tested substrate).
Public surface: `Bivector::from_plane`, `Bivector::exp` (the
law itself), `Rotor::apply` (the sandwich product),
`orbit_position` (GOLDEN — pure parametric evaluation),
`orbit_position_perturbed` (FUZZY — additive semantics, §3).

**Proof, not assertion** — 7 tests, all passing:
- `normal_matches_coordinate_planes` — the plane-normal
  (Hodge dual) convention is verified against the three
  coordinate planes before being trusted elsewhere.
- `full_turn_returns_to_start` — theta = 2*pi is the identity.
- `quarter_turn_in_xy_plane` — a concrete, checkable rotation.
- `rotor_preserves_length` — the core rendering promise: a
  particle stays at orbit radius from its nucleus at every
  phase, sampled at 16 points around the circle.
- `rotor_composition_matches_direct_angle` — composing two
  rotors via the geometric product equals one rotor at the
  summed angle (the group structure a GPU incremental-update
  path would eventually rely on).
- `full_orbit_traces_a_circle_in_plane` — sampled at 32
  phases, every point sits at exactly the orbit radius from
  the nucleus AND lies in the declared plane (checked against
  the plane's own normal, not assumed) — the end-to-end §1
  claim, proven.
- `fuzzy_perturbation_is_additive` — §3's semantics, proven.

## 2. The BUZU chunk (artifact, NL-001-A1) — SEALED (PB-256)

BUZU is the upload-once binary unit of orbit data, implemented
in `crates/buzu-core::chunk`:
- **`ChunkHeader`**: a fixed 32-byte, `#[repr(C)]` record —
  `nucleus:[f32;3]` (12B) + `bivector:[f32;3]` (12B) +
  `count:u32` (4B) + `checksum:u32` (4B). Verified by
  `header_is_32_bytes` to have zero padding.
- **Capacity**: `CHUNK_CAPACITY = 65536` (2^16) particles per
  chunk — aligned to real GPU warp/wavefront widths (32/64) so
  a chunk dispatches with no partial-warp waste. 1B particles
  → ~15,259 chunks, each independently LOD/cull-able.
  `seal()` panics (not silently truncates) if handed more.
- **Checksum**: FNV-1a, 32-bit, deliberately non-cryptographic
  — a load-integrity check on an immutable chunk, not a
  security boundary (this ecosystem's real crypto, kupru's
  Argon2id/ChaCha20-Poly1305, stays reserved for actual
  secrets). `BuzuChunk::from_bytes` returns `None` on a failed
  checksum rather than trusting corrupt data — proven by
  `checksum_roundtrips_and_detects_corruption`.
- Chunks are immutable once sealed, and are simultaneously LOD
  and culling units: distant chunks render as aggregates; near
  chunks render raw; visibility is decided per-chunk, never
  per-particle.
- Chunk emission is an EnkiDB EMIT target so one sealed dataset
  feeds both the sovereign renderer (raw wgpu, DubSar Theater)
  and outward faces (WASM / geospatial tier) without
  re-derivation.

## 3. State trichotomy interaction — SEALED (PB-256)

GOLDEN particles: pure parametric evaluation via
`orbit_position` (zero delta, zero extra bytes) — proven in §1
and reconfirmed by `packed_position_matches_orbit_position_directly`.
FUZZY particles: deform off the shared orbit via a **sparse
index+delta side-array**, `fuzzy: Vec<(u32, [i8;3])>` — present
ONLY for particles that deviate. A perturbation is quantized to
3 signed bytes (not 3xf32 = 12 bytes) since a deformation term
is a small correction, not a full-precision position; the
physical size of one quantization step (`perturb_scale`) is
supplied by the caller at evaluation time. `fuzzy_is_sparse_golden_particles_add_nothing`
proves 1000 GOLDEN particles alongside 3 FUZZY ones produce a
fuzzy list of exactly 3 entries. DEAD particles: fixed points;
render from the aggregate tier only (still out of scope).

Consequence: a chunk's true bandwidth cost scales with its
FUZZY ratio — the health of the data is literally the cost
of drawing it. This is now a property of the byte format, not
just a stated intention.

## 4. Honest residual

Bivector encoding removes the TRANSFER bottleneck, not the
RASTERIZATION one. Chunk-level LOD/aggregation (§2) remains a
load-bearing member of this law, not an optimization, and is
not yet built.

## 5. Architect decisions — RATIFIED 2026-07-26 (PB-256)

The Architect confirmed the recommended designs; each is now
implemented and tested, not merely proposed:

- **D1 — RATIFIED: per-Tribe.** The math is correct either way
  (per-particle bivectors were never wrong), but the byte
  budget settles it decisively: per-particle bivector storage
  is ~24+ bytes/particle — at 1B particles that is 24+ GB,
  larger than the 12 GB positional-transfer wall this whole
  design exists to eliminate, and it does not fit in any real
  GPU's VRAM. Per-Tribe amortizes the bivector+nucleus to
  near-zero and additionally reuses the radius ALREADY derived
  per-particle from KAKI bytes in `bahyway_algebra::orbital`
  (`orbital_position`'s delta-derived radius) — so a GOLDEN
  particle costs zero new bytes beyond the phase it needs
  anyway. Implemented as `ChunkHeader`'s shared nucleus+bivector
  in `crates/buzu-core::chunk`.
- **D2 — RATIFIED: SoA, 32-byte header, 65536-particle chunks,
  FNV-1a checksum.** Structure-of-Arrays for GPU memory
  coalescing at scale; chunk size aligned to real warp/wavefront
  widths so dispatch never wastes a partial warp; a
  non-cryptographic checksum because this is a load-integrity
  check, not a secret. See §2 for the implemented layout.
- **D3 — RATIFIED: sparse index+delta side-array.** Honors the
  principle §3 already stated before this ratification — "a
  chunk's true bandwidth cost scales with its FUZZY ratio" only
  holds if GOLDEN (the common case) pays zero bytes for FUZZY's
  exception. A dense always-present perturbation field would
  have contradicted that principle. See §3 for the implemented
  format.

**Honest limit, unchanged by ratification:** `crates/buzu-core`'s
own test suite measures real CPU-side pack+evaluate throughput
(`throughput_measurement_cpu_side_pack_and_evaluate`) — on one
CPU core in the authoring sandbox, ~1.0M particles/s for
evaluation, meaning 1B particles sequentially would take ~1000s,
not <1s. This is expected and does not contradict the law: §1's
whole point is that evaluation belongs on the GPU (one thread
per particle, fully parallel), not the CPU. The <1s claim for
1B particles is a GPU-dispatch/rasterization measurement this
crate does not and cannot make — that remains real, separate,
honestly out-of-scope follow-up work (a wgpu compute/vertex
shader consuming this exact chunk format).

## 6. Naming record

BUZU: Architect-coined, Akkadian phonetic style, no
etymology claimed (Truth Before Beauty). PUZUR considered and
returned to the artifact name pool (puzru, "secret/shelter").
ERISHUM considered and reserved to the ERA pool instead (king
of Assur; per NL-001, kings name eras) — see
`docs/02_identity/NL-001-A1.md` for the running artifact-name registry
this playbook starts.
