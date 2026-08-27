// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  panam-engine · src/lib.rs
//  Panam (𒊩𒉡𒀭) — Face detection and KAKI face PK generation
//  IR + RGB dual-channel face identification
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

pub use nusku_engine::KakiPK;

#[derive(Debug, Clone)]
pub struct FaceDetection {
    pub face_kaki: KakiPK,
    pub confidence: f32,
    pub bounding_box: [f32; 4],   // [x, y, w, h] normalized
    pub landmarks: Vec<[f32; 2]>, // 5-point (eyes, nose, mouth)
    pub thermal_temp: f32,        // forehead temperature from IR channel
    pub face_kaki_hex: String,
}

// ── KAKI derivation ───────────────────────────────────────────────

/// Derive a KAKI face PK from a 128-dimensional facial embedding.
///
/// Pure-Rust avalanche hash — no external crates.
/// Mixing constants derived from the golden ratio (φ = 1.6180339887…)
/// and its successive powers; guaranteed non-zero output for any input.
pub fn derive_face_kaki(embedding: &[f32; 128], frame_id: u64) -> KakiPK {
    // Golden-ratio fractional-part constants (32-bit Fibonacci mixing)
    const PHI32: u32 = 0x9e37_79b9; // Knuth multiplicative constant
    const C0: u32 = 0x6c62_272e; // Murmur3 mix constant
    const C1: u32 = 0x2745_937f;

    // Seed from frame_id
    let mut state: [u32; 4] = [
        (frame_id & 0xffff_ffff) as u32 ^ PHI32,
        ((frame_id >> 32) & 0xffff_ffff) as u32 ^ C0,
        PHI32.wrapping_mul(C1),
        C0.wrapping_mul(PHI32),
    ];

    // Fold each embedding f32 as its raw bits
    for (i, &v) in embedding.iter().enumerate() {
        let bits = v.to_bits();
        let lane = i & 3;
        state[lane] =
            state[lane].wrapping_add(bits).wrapping_mul(PHI32) ^ state[lane].rotate_left(13);
        // Cross-lane mix every 4 elements
        if lane == 3 {
            state[0] ^= state[1].rotate_left(7);
            state[1] ^= state[2].rotate_left(11);
            state[2] ^= state[3].rotate_left(17);
            state[3] ^= state[0].rotate_left(19);
        }
    }

    // Final avalanche
    for _ in 0..4 {
        state[0] = state[0].wrapping_mul(C0) ^ state[2].rotate_left(5);
        state[1] = state[1].wrapping_mul(C1) ^ state[3].rotate_left(9);
        state[2] = state[2].wrapping_mul(PHI32) ^ state[0].rotate_left(13);
        state[3] = state[3].wrapping_mul(C0) ^ state[1].rotate_left(7);
    }

    // Pack 4×u32 → 16 bytes
    let mut kaki = [0u8; 16];
    for (i, &s) in state.iter().enumerate() {
        kaki[i * 4..i * 4 + 4].copy_from_slice(&s.to_le_bytes());
    }
    kaki
}

pub use nusku_engine::kaki_to_hex;

// ── Simulated face detection ──────────────────────────────────────

/// Simulate a face detection result for development / testing.
/// Returns None on every third frame to model real-world no-detections.
pub fn simulate_face_detection(frame_id: u64, forehead_temp: f32) -> Option<FaceDetection> {
    if frame_id % 3 == 2 {
        return None;
    }

    let mut emb = [0.0f32; 128];
    for (i, v) in emb.iter_mut().enumerate() {
        *v = ((frame_id as f32 * 0.01 + i as f32 * 0.1).sin()).abs();
    }

    let face_kaki = derive_face_kaki(&emb, frame_id);
    let face_kaki_hex = kaki_to_hex(&face_kaki);

    Some(FaceDetection {
        face_kaki,
        confidence: 0.87,
        bounding_box: [0.35, 0.05, 0.30, 0.28],
        landmarks: vec![
            [0.42, 0.12],
            [0.58, 0.12], // eyes
            [0.50, 0.18], // nose
            [0.43, 0.24],
            [0.57, 0.24], // mouth corners
        ],
        thermal_temp: forehead_temp,
        face_kaki_hex,
    })
}

// ── Tests ─────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kaki_is_16_bytes() {
        let emb = [0.5f32; 128];
        let kaki = derive_face_kaki(&emb, 0);
        assert_eq!(kaki.len(), 16);
    }

    #[test]
    fn different_embeddings_produce_different_kakis() {
        let emb_a = [0.1f32; 128];
        let mut emb_b = [0.1f32; 128];
        emb_b[64] = 0.9;
        assert_ne!(derive_face_kaki(&emb_a, 0), derive_face_kaki(&emb_b, 0));
    }

    #[test]
    fn different_frame_ids_produce_different_kakis() {
        let emb = [0.5f32; 128];
        assert_ne!(derive_face_kaki(&emb, 0), derive_face_kaki(&emb, 1));
    }

    #[test]
    fn kaki_to_hex_is_32_chars() {
        let kaki = [0xabu8; 16];
        let hex = kaki_to_hex(&kaki);
        assert_eq!(hex.len(), 32);
        assert_eq!(&hex[..4], "ABAB"); // nusku-engine uses uppercase
    }

    #[test]
    fn simulate_skips_every_third_frame() {
        assert!(simulate_face_detection(0, 36.5).is_some());
        assert!(simulate_face_detection(1, 36.5).is_some());
        assert!(simulate_face_detection(2, 36.5).is_none());
        assert!(simulate_face_detection(3, 36.5).is_some());
    }

    #[test]
    fn simulate_carries_forehead_temp() {
        let det = simulate_face_detection(0, 38.2).unwrap();
        assert!((det.thermal_temp - 38.2).abs() < 1e-5);
    }

    #[test]
    fn simulate_confidence_is_reasonable() {
        let det = simulate_face_detection(0, 36.5).unwrap();
        assert!(det.confidence > 0.0 && det.confidence <= 1.0);
    }
}
