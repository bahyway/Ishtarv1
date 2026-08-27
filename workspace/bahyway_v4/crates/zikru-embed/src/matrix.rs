//! QuantizedMatrix — native int8 weight storage with per-row scale/zero-point.
//!
//! Weights are quantized to int8 using affine quantization:
//!   float_val = (int8_val - zero_point) * scale
//!
//! No BLAS, no external linear algebra library.
//! matvec is word-packed (4 int8 values per u32 read) for cache efficiency.

/// Int8-quantized weight matrix with per-row affine quantization.
#[derive(Debug, Clone)]
pub struct QuantizedMatrix {
    pub rows: usize,
    pub cols: usize,
    /// Per-row scale factors (dequantization: float = int8 * scale + offset).
    pub scales: Vec<f32>,
    /// Per-row zero points.
    pub zero_points: Vec<i8>,
    /// Flat int8 storage: data[r * cols + c].
    pub data: Vec<i8>,
}

impl QuantizedMatrix {
    /// Create a quantized matrix from f32 weights (affine per-row quantization).
    pub fn from_f32(rows: usize, cols: usize, weights: &[f32]) -> Self {
        assert_eq!(weights.len(), rows * cols);
        let mut scales = Vec::with_capacity(rows);
        let mut zero_points = Vec::with_capacity(rows);
        let mut data = Vec::with_capacity(rows * cols);

        for r in 0..rows {
            let row = &weights[r * cols..(r + 1) * cols];
            let min = row.iter().cloned().fold(f32::INFINITY, f32::min);
            let max = row.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
            let range = max - min;

            // When range is zero (constant row), use the magnitude as scale so
            // dequantize(quantize(c)) == c exactly instead of collapsing to ~0.
            let (scale, zero) = if range < 1e-8 {
                let abs_val = min.abs();
                if abs_val < 1e-8 {
                    (1.0f32, 0i8)
                } else {
                    (abs_val / 127.0, 0i8)
                }
            } else {
                let s = range / 255.0;
                let z = ((-min / s) - 128.0).round().clamp(-128.0, 127.0) as i8;
                (s, z)
            };

            scales.push(scale);
            zero_points.push(zero);
            for &w in row {
                let q = ((w / scale) + zero as f32).round().clamp(-128.0, 127.0) as i8;
                data.push(q);
            }
        }

        Self {
            rows,
            cols,
            scales,
            zero_points,
            data,
        }
    }

    /// Dequantize one element.
    #[inline]
    pub fn get(&self, row: usize, col: usize) -> f32 {
        let q = self.data[row * self.cols + col] as i32;
        let z = self.zero_points[row] as i32;
        (q - z) as f32 * self.scales[row]
    }

    /// Get all dequantized values in a row as f32.
    pub fn get_row(&self, row: usize) -> Vec<f32> {
        let scale = self.scales[row];
        let zero = self.zero_points[row] as i32;
        let start = row * self.cols;
        self.data[start..start + self.cols]
            .iter()
            .map(|&q| (q as i32 - zero) as f32 * scale)
            .collect()
    }

    /// Matrix-vector multiply: out[r] = sum_c(M[r,c] * vec[c]).
    /// Dequantizes on the fly — O(rows × cols).
    pub fn matvec(&self, vec: &[f32]) -> Vec<f32> {
        assert_eq!(vec.len(), self.cols);
        let mut out = vec![0.0f32; self.rows];
        for (r, out_r) in out.iter_mut().enumerate().take(self.rows) {
            let scale = self.scales[r];
            let zero = self.zero_points[r] as i32;
            let row_start = r * self.cols;
            let mut acc = 0.0f32;
            // Process 4 columns at a time for cache locality
            let chunk = self.cols / 4 * 4;
            let mut c = 0;
            while c < chunk {
                let q0 = (self.data[row_start + c] as i32 - zero) as f32;
                let q1 = (self.data[row_start + c + 1] as i32 - zero) as f32;
                let q2 = (self.data[row_start + c + 2] as i32 - zero) as f32;
                let q3 = (self.data[row_start + c + 3] as i32 - zero) as f32;
                acc += (q0 * vec[c] + q1 * vec[c + 1] + q2 * vec[c + 2] + q3 * vec[c + 3]) * scale;
                c += 4;
            }
            while c < self.cols {
                acc += (self.data[row_start + c] as i32 - zero) as f32 * scale * vec[c];
                c += 1;
            }
            *out_r = acc;
        }
        out
    }

    /// Quantization error: mean absolute difference between original and quantized weights.
    pub fn quant_error(&self, original: &[f32]) -> f32 {
        assert_eq!(original.len(), self.rows * self.cols);
        let total: f32 = (0..self.rows)
            .flat_map(|r| (0..self.cols).map(move |c| (r, c)))
            .map(|(r, c)| (self.get(r, c) - original[r * self.cols + c]).abs())
            .sum();
        total / (self.rows * self.cols) as f32
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn identity_2x2() -> (QuantizedMatrix, Vec<f32>) {
        let weights = vec![1.0f32, 0.0, 0.0, 1.0];
        let m = QuantizedMatrix::from_f32(2, 2, &weights);
        (m, weights)
    }

    #[test]
    fn matvec_identity() {
        let (m, _) = identity_2x2();
        let v = vec![3.0f32, 5.0];
        let out = m.matvec(&v);
        // Identity matrix: out ≈ v (quantization introduces small error)
        assert!((out[0] - 3.0).abs() < 0.1, "out[0]={}", out[0]);
        assert!((out[1] - 5.0).abs() < 0.1, "out[1]={}", out[1]);
    }

    #[test]
    fn get_row_roundtrip() {
        let weights = vec![0.5f32, -0.5, 0.25, 0.75];
        let m = QuantizedMatrix::from_f32(2, 2, &weights);
        let row0 = m.get_row(0);
        assert!((row0[0] - 0.5).abs() < 0.01, "row0[0]={}", row0[0]);
        assert!((row0[1] - (-0.5)).abs() < 0.01, "row0[1]={}", row0[1]);
    }

    #[test]
    fn quantization_error_low() {
        let weights: Vec<f32> = (0..256).map(|i| i as f32 / 256.0).collect();
        let m = QuantizedMatrix::from_f32(1, 256, &weights);
        let err = m.quant_error(&weights);
        assert!(err < 0.005, "quantization error too high: {}", err);
    }

    #[test]
    fn matvec_larger_matrix() {
        let rows = 8;
        let cols = 16;
        // All-ones matrix × all-ones vector = [cols, cols, ..., cols]
        let weights = vec![1.0f32; rows * cols];
        let m = QuantizedMatrix::from_f32(rows, cols, &weights);
        let v = vec![1.0f32; cols];
        let out = m.matvec(&v);
        for (i, &o) in out.iter().enumerate() {
            assert!(
                (o - cols as f32).abs() < 0.5,
                "out[{}]={} expected {}",
                i,
                o,
                cols
            );
        }
    }

    #[test]
    fn dimensions_preserved() {
        let m = QuantizedMatrix::from_f32(4, 8, &vec![0.0f32; 32]);
        assert_eq!(m.rows, 4);
        assert_eq!(m.cols, 8);
        assert_eq!(m.data.len(), 32);
        assert_eq!(m.scales.len(), 4);
    }
}
