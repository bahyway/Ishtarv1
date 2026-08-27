//! The BUZU chunk (GL-VIZ-001 §2/§5, D1-D3 SEALED 2026-07-26).
//!
//! Three designs, chosen and measured, not merely proposed:
//!
//! D1 (residence) -- PER-TRIBE. One [`crate::Bivector`] + nucleus is
//! shared by every particle on the orbit (`ChunkHeader`); a particle
//! costs only its phase (`theta: f32`, 4 bytes) here. Its RADIUS is
//! deliberately NOT stored in this chunk -- it already comes from the
//! existing KAKI-derived scheme in `bahyway_algebra::orbital`
//! (`orbital_position`'s delta-derived radius), which is already
//! GPU-resident for other rendering purposes. Duplicating it here
//! would be exactly the kind of dense, always-present field D3 below
//! argues against. Callers supply `radius` at evaluation time.
//!
//! D2 (byte layout) -- Structure-of-Arrays, GPU-dispatch-aligned.
//! `ChunkHeader` is a fixed 32-byte, `#[repr(C)]` record (verified by
//! `header_is_32_bytes`): nucleus (12B) + bivector (12B) + count (4B)
//! + checksum (4B). [`CHUNK_CAPACITY`] = 65536 = 2^16 particles/chunk,
//!   aligned to real GPU workgroup sizes (32/64-wide warps/wavefronts)
//!   so a chunk dispatches with no partial-warp waste; 1B particles ->
//!   ~15,259 chunks, each independently LOD/cull-able. The checksum is
//!   FNV-1a (32-bit, non-cryptographic) -- a load-integrity check, not a
//!   security boundary; this ecosystem's real crypto (kupru's
//!   Argon2id/ChaCha20-Poly1305) is reserved for actual secrets.
//!
//! D3 (FUZZY encoding) -- sparse index+delta side-array
//! (`fuzzy: Vec<(u32, [i8;3])>`), present ONLY for particles that
//! deviate from their shared orbit. A GOLDEN particle (the common
//! case, per the trichotomy's own name) costs zero fuzzy bytes.
//! Perturbation is quantized to signed bytes (i8, ~1/128 of a unit
//! reference-frame step) -- 3 bytes, not 12 (3xf32) -- since a
//! deformation term is inherently a small correction, not a
//! full-precision position.

use crate::{orbit_position, orbit_position_perturbed, Bivector};

/// 2^16 -- GPU-dispatch-aligned chunk capacity (D2).
pub const CHUNK_CAPACITY: usize = 65536;

/// The shared, per-orbit header (D1): nucleus + bivector, amortized
/// across every particle in the chunk. `#[repr(C)]` so its layout is
/// exactly what a GPU-side struct would mirror.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChunkHeader {
    pub nucleus: [f32; 3],
    pub bivector: [f32; 3], // b01, b02, b12
    pub count: u32,
    pub checksum: u32,
}

const HEADER_BYTES: usize = 32;

impl ChunkHeader {
    fn to_bytes(self) -> [u8; HEADER_BYTES] {
        let mut b = [0u8; HEADER_BYTES];
        b[0..4].copy_from_slice(&self.nucleus[0].to_le_bytes());
        b[4..8].copy_from_slice(&self.nucleus[1].to_le_bytes());
        b[8..12].copy_from_slice(&self.nucleus[2].to_le_bytes());
        b[12..16].copy_from_slice(&self.bivector[0].to_le_bytes());
        b[16..20].copy_from_slice(&self.bivector[1].to_le_bytes());
        b[20..24].copy_from_slice(&self.bivector[2].to_le_bytes());
        b[24..28].copy_from_slice(&self.count.to_le_bytes());
        b[28..32].copy_from_slice(&self.checksum.to_le_bytes());
        b
    }

    fn from_bytes(b: &[u8]) -> Option<Self> {
        if b.len() < HEADER_BYTES {
            return None;
        }
        let f = |r: std::ops::Range<usize>| f32::from_le_bytes(b[r].try_into().unwrap());
        let u = |r: std::ops::Range<usize>| u32::from_le_bytes(b[r].try_into().unwrap());
        Some(ChunkHeader {
            nucleus: [f(0..4), f(4..8), f(8..12)],
            bivector: [f(12..16), f(16..20), f(20..24)],
            count: u(24..28),
            checksum: u(28..32),
        })
    }
}

/// FNV-1a, 32-bit -- deliberately non-cryptographic (D2): this is a
/// load-integrity check on an immutable chunk, not a secret.
fn fnv1a32(bytes: &[u8]) -> u32 {
    let mut hash: u32 = 0x811c9dc5;
    for &b in bytes {
        hash ^= b as u32;
        hash = hash.wrapping_mul(0x0100_0193);
    }
    hash
}

/// A sealed BUZU chunk: one shared orbit (header) plus up to
/// [`CHUNK_CAPACITY`] particle phases (SoA `theta`), plus a sparse
/// FUZZY side-array for the minority of particles that deform off the
/// shared orbit (D3).
#[derive(Debug, Clone, PartialEq)]
pub struct BuzuChunk {
    pub header: ChunkHeader,
    pub theta: Vec<f32>,
    /// (particle index within this chunk, quantized perturbation).
    pub fuzzy: Vec<(u32, [i8; 3])>,
}

fn checksum_of(
    nucleus: [f32; 3],
    bivector: [f32; 3],
    count: u32,
    theta: &[f32],
    fuzzy: &[(u32, [i8; 3])],
) -> u32 {
    let header0 = ChunkHeader {
        nucleus,
        bivector,
        count,
        checksum: 0,
    };
    let mut buf = Vec::with_capacity(HEADER_BYTES + theta.len() * 4 + fuzzy.len() * 7);
    buf.extend_from_slice(&header0.to_bytes());
    for t in theta {
        buf.extend_from_slice(&t.to_le_bytes());
    }
    for (idx, delta) in fuzzy {
        buf.extend_from_slice(&idx.to_le_bytes());
        buf.push(delta[0] as u8);
        buf.push(delta[1] as u8);
        buf.push(delta[2] as u8);
    }
    fnv1a32(&buf)
}

impl BuzuChunk {
    /// Seal a chunk: immutable from this point (D2). Panics if
    /// `theta.len() > CHUNK_CAPACITY` -- a caller error, not a
    /// runtime condition to silently truncate.
    pub fn seal(
        nucleus: [f32; 3],
        bivector: [f32; 3],
        theta: Vec<f32>,
        fuzzy: Vec<(u32, [i8; 3])>,
    ) -> Self {
        assert!(
            theta.len() <= CHUNK_CAPACITY,
            "a BUZU chunk holds at most {CHUNK_CAPACITY} particles, got {}",
            theta.len()
        );
        let count = theta.len() as u32;
        let checksum = checksum_of(nucleus, bivector, count, &theta, &fuzzy);
        BuzuChunk {
            header: ChunkHeader {
                nucleus,
                bivector,
                count,
                checksum,
            },
            theta,
            fuzzy,
        }
    }

    /// Recompute and compare the checksum -- the load-integrity check.
    pub fn verify(&self) -> bool {
        let expected = checksum_of(
            self.header.nucleus,
            self.header.bivector,
            self.header.count,
            &self.theta,
            &self.fuzzy,
        );
        expected == self.header.checksum
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf =
            Vec::with_capacity(HEADER_BYTES + self.theta.len() * 4 + self.fuzzy.len() * 7 + 4);
        buf.extend_from_slice(&self.header.to_bytes());
        for t in &self.theta {
            buf.extend_from_slice(&t.to_le_bytes());
        }
        buf.extend_from_slice(&(self.fuzzy.len() as u32).to_le_bytes());
        for (idx, delta) in &self.fuzzy {
            buf.extend_from_slice(&idx.to_le_bytes());
            buf.push(delta[0] as u8);
            buf.push(delta[1] as u8);
            buf.push(delta[2] as u8);
        }
        buf
    }

    /// Parse and verify in one step -- `None` on truncation OR a
    /// failed checksum, so a caller can never silently trust a
    /// corrupt chunk.
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let header = ChunkHeader::from_bytes(bytes)?;
        let mut off = HEADER_BYTES;
        let count = header.count as usize;
        let mut theta = Vec::with_capacity(count);
        for _ in 0..count {
            let t = f32::from_le_bytes(bytes.get(off..off + 4)?.try_into().ok()?);
            theta.push(t);
            off += 4;
        }
        let fuzzy_len = u32::from_le_bytes(bytes.get(off..off + 4)?.try_into().ok()?) as usize;
        off += 4;
        let mut fuzzy = Vec::with_capacity(fuzzy_len);
        for _ in 0..fuzzy_len {
            let idx = u32::from_le_bytes(bytes.get(off..off + 4)?.try_into().ok()?);
            let d = bytes.get(off + 4..off + 7)?;
            fuzzy.push((idx, [d[0] as i8, d[1] as i8, d[2] as i8]));
            off += 7;
        }
        let chunk = BuzuChunk {
            header,
            theta,
            fuzzy,
        };
        if chunk.verify() {
            Some(chunk)
        } else {
            None
        }
    }

    /// GOLDEN or FUZZY position for particle `i` (0-indexed within
    /// this chunk). `radius` is supplied by the caller (D1: it
    /// already lives in the existing KAKI-derived pipeline, not
    /// duplicated in this chunk). Perturbation, if `i` is in the
    /// sparse `fuzzy` list, is dequantized from i8 by `perturb_scale`
    /// (the physical size, in world units, of one quantization step).
    pub fn position(&self, i: usize, radius: f64, perturb_scale: f64) -> [f64; 3] {
        let plane = Bivector {
            b01: self.header.bivector[0] as f64,
            b02: self.header.bivector[1] as f64,
            b12: self.header.bivector[2] as f64,
        };
        let nucleus = [
            self.header.nucleus[0] as f64,
            self.header.nucleus[1] as f64,
            self.header.nucleus[2] as f64,
        ];
        let refdir = plane.canonical_reference();
        let reference = [refdir[0] * radius, refdir[1] * radius, refdir[2] * radius];
        let theta = self.theta[i] as f64;

        match self.fuzzy.iter().find(|(idx, _)| *idx as usize == i) {
            None => orbit_position(nucleus, reference, &plane, theta),
            Some((_, delta)) => {
                let perturbation = [
                    delta[0] as f64 * perturb_scale,
                    delta[1] as f64 * perturb_scale,
                    delta[2] as f64 * perturb_scale,
                ];
                orbit_position_perturbed(nucleus, reference, &plane, theta, perturbation)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_is_32_bytes() {
        assert_eq!(
            std::mem::size_of::<ChunkHeader>(),
            32,
            "repr(C) in-memory size must match the wire format"
        );
        assert_eq!(HEADER_BYTES, 32);
        let h = ChunkHeader {
            nucleus: [1.0, 2.0, 3.0],
            bivector: [0.1, 0.2, 0.3],
            count: 7,
            checksum: 42,
        };
        assert_eq!(h.to_bytes().len(), 32);
    }

    #[test]
    fn checksum_roundtrips_and_detects_corruption() {
        let chunk = BuzuChunk::seal(
            [1.0, 2.0, 3.0],
            [0.0, 0.0, 1.0],
            vec![0.0, 0.5, 1.0, 1.5],
            vec![(2, [3, -4, 5])],
        );
        assert!(chunk.verify());

        let bytes = chunk.to_bytes();
        let restored =
            BuzuChunk::from_bytes(&bytes).expect("a freshly sealed chunk must parse and verify");
        assert_eq!(restored, chunk);

        let mut corrupted = bytes.clone();
        corrupted[40] ^= 0xFF; // flip a byte inside the theta array
        assert!(
            BuzuChunk::from_bytes(&corrupted).is_none(),
            "a corrupted chunk must fail verification, not silently load"
        );
    }

    #[test]
    fn fuzzy_is_sparse_golden_particles_add_nothing() {
        // 1000 GOLDEN particles, only 3 FUZZY -- the fuzzy list must
        // reflect exactly the minority, proving D3's sparsity claim.
        let theta: Vec<f32> = (0..1000).map(|i| i as f32 * 0.001).collect();
        let fuzzy = vec![(10u32, [1i8, 0, 0]), (500, [0, -2, 1]), (999, [3, 3, 3])];
        let chunk = BuzuChunk::seal([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], theta, fuzzy);
        assert_eq!(
            chunk.fuzzy.len(),
            3,
            "fuzzy side-array must hold only the deviating particles, not all 1000"
        );
        assert!(chunk.verify());
    }

    #[test]
    #[should_panic]
    fn seal_rejects_over_capacity() {
        let theta = vec![0.0f32; CHUNK_CAPACITY + 1];
        let _ = BuzuChunk::seal([0.0; 3], [1.0, 0.0, 0.0], theta, vec![]);
    }

    #[test]
    fn packed_position_matches_orbit_position_directly() {
        let bivector = [1.0f32, 0.2, -0.3];
        let nucleus = [5.0f32, -1.0, 2.0];
        let radius = 3.0f64;
        let thetas: Vec<f32> = (0..64)
            .map(|i| i as f32 * std::f32::consts::TAU / 64.0)
            .collect();
        let chunk = BuzuChunk::seal(nucleus, bivector, thetas.clone(), vec![]);

        let plane = Bivector {
            b01: bivector[0] as f64,
            b02: bivector[1] as f64,
            b12: bivector[2] as f64,
        };
        let n = [nucleus[0] as f64, nucleus[1] as f64, nucleus[2] as f64];
        let refdir = plane.canonical_reference();
        let reference = [refdir[0] * radius, refdir[1] * radius, refdir[2] * radius];

        for (i, theta) in thetas.iter().enumerate() {
            let expected = orbit_position(n, reference, &plane, *theta as f64);
            let got = chunk.position(i, radius, 0.01);
            for k in 0..3 {
                assert!(
                    (expected[k] - got[k]).abs() < 1e-5,
                    "chunk-packed evaluation must match direct orbit_position"
                );
            }
        }
    }

    #[test]
    fn fuzzy_particle_position_includes_dequantized_perturbation() {
        let bivector = [1.0f32, 0.0, 0.0];
        let nucleus = [0.0f32; 3];
        let radius = 1.0f64;
        let perturb_scale = 0.02f64;
        let theta = vec![0.3f32];
        let chunk = BuzuChunk::seal(nucleus, bivector, theta, vec![(0, [10, -20, 5])]);

        let plane = Bivector {
            b01: 1.0,
            b02: 0.0,
            b12: 0.0,
        };
        let refdir = plane.canonical_reference();
        let reference = [refdir[0] * radius, refdir[1] * radius, refdir[2] * radius];
        let golden = orbit_position([0.0; 3], reference, &plane, 0.3);
        let expected_delta = [
            10.0 * perturb_scale,
            -20.0 * perturb_scale,
            5.0 * perturb_scale,
        ];

        let got = chunk.position(0, radius, perturb_scale);
        for k in 0..3 {
            assert!((got[k] - (golden[k] + expected_delta[k])).abs() < 1e-6);
        }
    }

    /// Not a hard performance assertion (unit tests must not be flaky
    /// across machines) -- but a REAL, printed measurement of
    /// CPU-side pack + evaluate throughput, honestly scoped: this
    /// measures the geometric-algebra evaluation path this crate
    /// controls, not GPU dispatch/rasterization (§4's honest residual
    /// still applies -- that half of the billion-particle claim needs
    /// a real GPU to measure and is out of this crate's scope).
    #[test]
    fn throughput_measurement_cpu_side_pack_and_evaluate() {
        const N: usize = 2_000_000;
        let bivector = [0.4f32, 1.0, -0.2];
        let nucleus = [1.0f32, 2.0, 3.0];
        let radius = 7.0f64;

        let start_pack = std::time::Instant::now();
        let mut chunks = Vec::new();
        let mut theta_buf = Vec::with_capacity(CHUNK_CAPACITY);
        for i in 0..N {
            theta_buf.push((i as f32) * 0.0001);
            if theta_buf.len() == CHUNK_CAPACITY {
                chunks.push(BuzuChunk::seal(
                    nucleus,
                    bivector,
                    std::mem::take(&mut theta_buf),
                    vec![],
                ));
            }
        }
        if !theta_buf.is_empty() {
            chunks.push(BuzuChunk::seal(nucleus, bivector, theta_buf, vec![]));
        }
        let pack_elapsed = start_pack.elapsed();

        let start_eval = std::time::Instant::now();
        let mut sink = 0.0f64;
        for chunk in &chunks {
            for i in 0..chunk.theta.len() {
                let p = chunk.position(i, radius, 0.01);
                sink += p[0];
            }
        }
        let eval_elapsed = start_eval.elapsed();

        eprintln!(
            "BUZU CPU-side throughput ({N} particles, {} chunks): pack {:?} ({:.1} Mparticles/s), evaluate {:?} ({:.1} Mparticles/s) [sink={sink:.3}, GPU dispatch/rasterization NOT measured here]",
            chunks.len(),
            pack_elapsed,
            N as f64 / pack_elapsed.as_secs_f64() / 1e6,
            eval_elapsed,
            N as f64 / eval_elapsed.as_secs_f64() / 1e6,
        );

        // Generous, non-flaky bound: this is a correctness+sanity gate,
        // not the real 1B/<1s claim (which needs the GPU path). `cargo
        // test` always builds unoptimized (debug_assertions on), where
        // position()'s per-call rotor/trig math (canonical_reference()
        // and the orbit_position* call, both recomputed per index) runs
        // far slower than in --release -- observed ~32s for 2M calls in
        // debug vs. a sub-second release run. The bound below is scaled
        // for that so the gate still fires on a genuine regression
        // without requiring --release to pass under `cargo test --workspace`.
        let bound_secs = if cfg!(debug_assertions) { 60.0 } else { 10.0 };
        assert!(pack_elapsed.as_secs_f64() < bound_secs && eval_elapsed.as_secs_f64() < bound_secs);
    }
}
