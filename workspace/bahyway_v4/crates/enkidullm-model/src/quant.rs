//! quant.rs — Pure Rust dequantization kernels for GGUF quantized tensors.
//!
//! Implements Q4_K_M (primary), Q4_0, Q8_0, F16, F32 dequantization.
//! All arithmetic is pure Rust — no SIMD intrinsics, no unsafe code.
//! Performance target: sufficient for 7B model inference on GTX 1650 Max-Q CPU path.
#![forbid(unsafe_code)]

use crate::gguf::GgufError;

/// Quantization type enum matching GGML constants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quantization {
    F32 = 0,
    F16 = 1,
    Q4_0 = 2,
    Q4_1 = 3,
    Q8_0 = 8,
    Q4KS = 12,
    Q4KM = 13,
    Q6K = 14,
}

impl Quantization {
    pub fn from_u32(v: u32) -> Option<Self> {
        match v {
            0 => Some(Self::F32),
            1 => Some(Self::F16),
            2 => Some(Self::Q4_0),
            3 => Some(Self::Q4_1),
            8 => Some(Self::Q8_0),
            12 => Some(Self::Q4KS),
            13 => Some(Self::Q4KM),
            14 => Some(Self::Q6K),
            _ => None,
        }
    }
    /// Block size in number of elements.
    pub fn block_elements(self) -> usize {
        match self {
            Self::Q4KS | Self::Q4KM | Self::Q6K => 256,
            _ => 32,
        }
    }
}

/// Dequantize a Q4_K_M tensor block to f32 values.
/// Q4_K_M super-block (256 elements, 144 bytes):
///   2 bytes: f16 scale_max
///   2 bytes: f16 scale_min  
///   12 bytes: 6-bit quantized sub-block scales (8 sub-blocks × 6 bits × 2)
///   128 bytes: 4-bit quantized values (256 × 4 bits)
pub fn dequantize_q4_k_m(data: &[u8], num_elem: usize) -> Result<Vec<f32>, GgufError> {
    const BLOCK_SIZE: usize = 256;
    const BLOCK_BYTES: usize = 144;
    const SUB_BLOCKS: usize = 8;
    const SUB_BLOCK_SIZE: usize = 32;

    let num_blocks = num_elem / BLOCK_SIZE;
    if data.len() < num_blocks * BLOCK_BYTES {
        return Err(GgufError::AlignmentError(format!(
            "Q4_K_M: expected {} bytes, got {}",
            num_blocks * BLOCK_BYTES,
            data.len()
        )));
    }

    let mut out = Vec::with_capacity(num_elem);

    for block_idx in 0..num_blocks {
        let base = block_idx * BLOCK_BYTES;
        let block = &data[base..base + BLOCK_BYTES];

        // Read super-block scale and min (f16 → f32)
        let d = f16_to_f32(u16::from_le_bytes([block[0], block[1]]));
        let dmin = f16_to_f32(u16::from_le_bytes([block[2], block[3]]));

        // Read 8 sub-block scales (6-bit each, packed in 12 bytes)
        let scales = read_6bit_scales(&block[4..16]);

        // Read 128 bytes of 4-bit values (256 elements)
        let quants = &block[16..144];

        for sub in 0..SUB_BLOCKS {
            let sc = scales[sub * 2] as f32;
            let mn = scales[sub * 2 + 1] as f32;
            let scale = d * sc;
            let min_val = dmin * mn;

            let q_base = sub * 16; // 32 elements × 4 bits = 16 bytes
            for i in 0..SUB_BLOCK_SIZE {
                let byte_idx = q_base + i / 2;
                let nibble = if i % 2 == 0 {
                    quants[byte_idx] & 0x0F
                } else {
                    (quants[byte_idx] >> 4) & 0x0F
                };
                out.push(scale * nibble as f32 - min_val);
            }
        }
    }

    Ok(out)
}

/// Dequantize Q4_0 blocks to f32.
/// Block (32 elements, 18 bytes): 2 bytes f16 scale + 16 bytes 4-bit values.
pub fn dequantize_q4_0(data: &[u8], num_elem: usize) -> Result<Vec<f32>, GgufError> {
    const BLOCK_SIZE: usize = 32;
    const BLOCK_BYTES: usize = 18;

    let num_blocks = num_elem / BLOCK_SIZE;
    if data.len() < num_blocks * BLOCK_BYTES {
        return Err(GgufError::AlignmentError(format!(
            "Q4_0: expected {} bytes, got {}",
            num_blocks * BLOCK_BYTES,
            data.len()
        )));
    }

    let mut out = Vec::with_capacity(num_elem);

    for block_idx in 0..num_blocks {
        let base = block_idx * BLOCK_BYTES;
        let d = f16_to_f32(u16::from_le_bytes([data[base], data[base + 1]]));
        let quants = &data[base + 2..base + 18];

        for i in 0..BLOCK_SIZE {
            let byte_idx = i / 2;
            let nibble = if i % 2 == 0 {
                quants[byte_idx] & 0x0F
            } else {
                (quants[byte_idx] >> 4) & 0x0F
            };
            // Q4_0: values are 0..15, center at 8 → subtract 8
            let x = (nibble as i32 - 8) as f32;
            out.push(d * x);
        }
    }

    Ok(out)
}

/// Dequantize Q8_0 blocks to f32.
/// Block (32 elements, 34 bytes): 2 bytes f16 scale + 32 bytes int8 values.
pub fn dequantize_q8_0(data: &[u8], num_elem: usize) -> Result<Vec<f32>, GgufError> {
    const BLOCK_SIZE: usize = 32;
    const BLOCK_BYTES: usize = 34;

    let num_blocks = num_elem / BLOCK_SIZE;
    if data.len() < num_blocks * BLOCK_BYTES {
        return Err(GgufError::AlignmentError(format!(
            "Q8_0: expected {} bytes, got {}",
            num_blocks * BLOCK_BYTES,
            data.len()
        )));
    }

    let mut out = Vec::with_capacity(num_elem);

    for block_idx in 0..num_blocks {
        let base = block_idx * BLOCK_BYTES;
        let d = f16_to_f32(u16::from_le_bytes([data[base], data[base + 1]]));
        let quants = &data[base + 2..base + 34];

        for &q in quants {
            out.push(d * (q as i8) as f32);
        }
    }

    Ok(out)
}

/// Convert F16 (IEEE 754 half precision) to F32.
/// Pure Rust — no intrinsics.
pub fn f16_to_f32(half: u16) -> f32 {
    let sign = ((half >> 15) as u32) << 31;
    let exp = ((half >> 10) & 0x1F) as u32;
    let mantissa = (half & 0x3FF) as u32;

    if exp == 0 {
        if mantissa == 0 {
            return f32::from_bits(sign); // ±0
        }
        // Subnormal — normalize
        let mut m = mantissa;
        let mut e = 127 - 14;
        while m & 0x400 == 0 {
            m <<= 1;
            e -= 1;
        }
        m &= 0x3FF;
        return f32::from_bits(sign | (e << 23) | (m << 13));
    }

    if exp == 31 {
        // Inf or NaN
        return f32::from_bits(sign | 0x7F800000 | (mantissa << 13));
    }

    // Normal number
    let e = exp + 127 - 15;
    f32::from_bits(sign | (e << 23) | (mantissa << 13))
}

/// Read 8 pairs of 6-bit sub-block scales from 12 bytes.
/// Returns 16 u8 values (8 scale + 8 min scale).
fn read_6bit_scales(data: &[u8]) -> [u8; 16] {
    let mut out = [0u8; 16];
    // 12 bytes encode 16 values of 6 bits each (96 bits total)
    for i in 0..8 {
        let bit_offset = i * 12;
        let byte_lo = bit_offset / 8;
        let bit_lo = bit_offset % 8;

        // Scale (6 bits)
        let raw_s = if bit_lo <= 2 {
            (data[byte_lo] >> bit_lo) & 0x3F
        } else {
            let lo = (data[byte_lo] >> bit_lo) as u16;
            let hi = (data[byte_lo + 1] as u16) << (8 - bit_lo);
            ((lo | hi) & 0x3F) as u8
        };
        out[i * 2] = raw_s;

        // Min scale (next 6 bits)
        let bit_offset2 = bit_offset + 6;
        let byte_lo2 = bit_offset2 / 8;
        let bit_lo2 = bit_offset2 % 8;
        let raw_m = if bit_lo2 <= 2 {
            (data[byte_lo2] >> bit_lo2) & 0x3F
        } else if byte_lo2 + 1 < data.len() {
            let lo = (data[byte_lo2] >> bit_lo2) as u16;
            let hi = (data[byte_lo2 + 1] as u16) << (8 - bit_lo2);
            ((lo | hi) & 0x3F) as u8
        } else {
            (data[byte_lo2] >> bit_lo2) & 0x3F
        };
        out[i * 2 + 1] = raw_m;
    }
    out
}

/// Matrix-vector multiply: y = A × x
/// A is (rows × cols) in row-major order, x is (cols,), y is (rows,).
pub fn matmul_f32(a: &[f32], x: &[f32], rows: usize, cols: usize) -> Vec<f32> {
    assert_eq!(a.len(), rows * cols, "matrix size mismatch");
    assert_eq!(x.len(), cols, "vector size mismatch");
    let mut y = vec![0.0f32; rows];
    for (r, y_r) in y.iter_mut().enumerate() {
        let row_start = r * cols;
        let mut acc = 0.0f32;
        for c in 0..cols {
            acc += a[row_start + c] * x[c];
        }
        *y_r = acc;
    }
    y
}

/// Softmax in-place over a slice.
pub fn softmax_inplace(x: &mut [f32]) {
    let max = x.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let mut sum = 0.0f32;
    for v in x.iter_mut() {
        *v = (*v - max).exp();
        sum += *v;
    }
    if sum > 0.0 {
        for v in x.iter_mut() {
            *v /= sum;
        }
    }
}

/// RMS normalization: x / rms(x) * weight
pub fn rms_norm(x: &[f32], weight: &[f32], eps: f32) -> Vec<f32> {
    assert_eq!(x.len(), weight.len());
    let rms = (x.iter().map(|v| v * v).sum::<f32>() / x.len() as f32 + eps).sqrt();
    x.iter()
        .zip(weight.iter())
        .map(|(v, w)| v / rms * w)
        .collect()
}

/// SiLU activation: x * sigmoid(x)
pub fn silu(x: f32) -> f32 {
    x / (1.0 + (-x).exp())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn f16_to_f32_zero() {
        assert_eq!(f16_to_f32(0x0000), 0.0f32);
    }

    #[test]
    fn f16_to_f32_one() {
        // 1.0 in f16 = 0x3C00
        let v = f16_to_f32(0x3C00);
        assert!((v - 1.0).abs() < 1e-6, "expected 1.0, got {v}");
    }

    #[test]
    fn f16_to_f32_minus_one() {
        // -1.0 in f16 = 0xBC00
        let v = f16_to_f32(0xBC00);
        assert!((v + 1.0).abs() < 1e-6, "expected -1.0, got {v}");
    }

    #[test]
    fn f16_to_f32_infinity() {
        assert!(f16_to_f32(0x7C00).is_infinite());
    }

    #[test]
    fn matmul_identity() {
        // 2×2 identity matrix × [1, 2] = [1, 2]
        let a = vec![1.0f32, 0.0, 0.0, 1.0];
        let x = vec![1.0f32, 2.0];
        let y = matmul_f32(&a, &x, 2, 2);
        assert!((y[0] - 1.0).abs() < 1e-6);
        assert!((y[1] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn softmax_sums_to_one() {
        let mut x = vec![1.0f32, 2.0, 3.0];
        softmax_inplace(&mut x);
        let sum: f32 = x.iter().sum();
        assert!((sum - 1.0).abs() < 1e-6, "softmax sum = {sum}");
    }

    #[test]
    fn rms_norm_unit_vector() {
        let x = vec![1.0f32, 0.0, 0.0];
        let w = vec![1.0f32, 1.0, 1.0];
        let y = rms_norm(&x, &w, 1e-6);
        // rms([1,0,0]) = sqrt(1/3) ≈ 0.577
        assert!(y[0] > 1.0, "first element should be > 1");
    }

    #[test]
    fn silu_at_zero() {
        // silu(0) = 0 * sigmoid(0) = 0 * 0.5 = 0
        assert!((silu(0.0)).abs() < 1e-6);
    }

    #[test]
    fn dequantize_q8_0_basic() {
        // Construct a minimal Q8_0 block: scale=1.0 (f16=0x3C00), values 0..31
        let mut data = Vec::new();
        data.extend_from_slice(&0x3C00u16.to_le_bytes()); // scale = 1.0
        for i in 0i8..32 {
            data.push(i as u8);
        }
        let result = dequantize_q8_0(&data, 32).unwrap();
        assert_eq!(result.len(), 32);
        assert!((result[0] - 0.0).abs() < 1e-5);
        assert!((result[1] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn quantization_block_elements() {
        assert_eq!(Quantization::Q4KM.block_elements(), 256);
        assert_eq!(Quantization::Q4_0.block_elements(), 32);
        assert_eq!(Quantization::Q8_0.block_elements(), 32);
    }
}
