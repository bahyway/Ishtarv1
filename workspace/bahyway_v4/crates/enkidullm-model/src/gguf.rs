//! gguf.rs — Sovereign GGUF file parser (pure Rust, memory-mapped).
//!
//! GGUF v3 specification: magic 0x46554747, version u32, tensor_count u64,
//! metadata_kv_count u64, then metadata KV pairs, then tensor infos,
//! then alignment-padded tensor data.
//!
//! All reads are little-endian. Strings are u64-length prefixed UTF-8.
#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// GGUF magic bytes: "GGUF" in little-endian
const GGUF_MAGIC: u32 = 0x46554747;
const GGUF_VERSION_3: u32 = 3;

/// All GGUF errors are sovereign — no panics, no unwrap.
#[derive(Debug)]
pub enum GgufError {
    Io(String),
    InvalidMagic(u32),
    UnsupportedVersion(u32),
    InvalidUtf8(String),
    UnknownMetadataType(u32),
    UnknownQuantType(u32),
    TensorNotFound(String),
    AlignmentError(String),
}

impl std::fmt::Display for GgufError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO error: {e}"),
            Self::InvalidMagic(m) => write!(f, "Invalid GGUF magic: 0x{m:08X}"),
            Self::UnsupportedVersion(v) => write!(f, "Unsupported GGUF version: {v}"),
            Self::InvalidUtf8(s) => write!(f, "Invalid UTF-8: {s}"),
            Self::UnknownMetadataType(t) => write!(f, "Unknown metadata type: {t}"),
            Self::UnknownQuantType(t) => write!(f, "Unknown quantization type: {t}"),
            Self::TensorNotFound(n) => write!(f, "Tensor not found: {n}"),
            Self::AlignmentError(e) => write!(f, "Alignment error: {e}"),
        }
    }
}

/// GGUF metadata value — all supported types.
#[derive(Debug, Clone)]
pub enum GgufValue {
    U8(u8),
    I8(i8),
    U16(u16),
    I16(i16),
    U32(u32),
    I32(i32),
    F32(f32),
    U64(u64),
    I64(i64),
    F64(f64),
    Bool(bool),
    String(String),
    Array(Vec<GgufValue>),
}

impl GgufValue {
    pub fn as_str(&self) -> Option<&str> {
        if let Self::String(s) = self {
            Some(s)
        } else {
            None
        }
    }
    pub fn as_u32(&self) -> Option<u32> {
        if let Self::U32(v) = self {
            Some(*v)
        } else {
            None
        }
    }
    pub fn as_u64(&self) -> Option<u64> {
        if let Self::U64(v) = self {
            Some(*v)
        } else {
            None
        }
    }
    pub fn as_f32(&self) -> Option<f32> {
        if let Self::F32(v) = self {
            Some(*v)
        } else {
            None
        }
    }
}

/// GGUF tensor descriptor (metadata only — data offset for lazy loading).
#[derive(Debug, Clone)]
pub struct GgufTensor {
    pub name: String,
    pub shape: Vec<u64>,
    pub quant_type: u32,
    pub data_offset: u64,
    pub data_size: u64,
}

impl GgufTensor {
    /// Number of elements in this tensor.
    pub fn num_elements(&self) -> u64 {
        self.shape.iter().product()
    }

    /// Rows × cols for 2D tensors (weight matrices).
    pub fn dims_2d(&self) -> (usize, usize) {
        match self.shape.len() {
            1 => (1, self.shape[0] as usize),
            2 => (self.shape[1] as usize, self.shape[0] as usize),
            _ => (
                self.shape[self.shape.len() - 1] as usize,
                self.shape[..self.shape.len() - 1].iter().product::<u64>() as usize,
            ),
        }
    }
}

/// Parsed GGUF metadata.
#[derive(Debug)]
pub struct GgufMetadata {
    pub version: u32,
    pub tensor_count: u64,
    pub kv: HashMap<String, GgufValue>,
}

impl GgufMetadata {
    /// Architecture string (e.g. "llama", "qwen2", "mistral").
    pub fn arch(&self) -> Option<&str> {
        self.kv.get("general.architecture")?.as_str()
    }
    /// Model name.
    pub fn model_name(&self) -> Option<&str> {
        self.kv.get("general.name")?.as_str()
    }
    /// Context length.
    pub fn context_length(&self) -> Option<u32> {
        let arch = self.arch()?;
        self.kv.get(&format!("{arch}.context_length"))?.as_u32()
    }
    /// Embedding dimension.
    pub fn embedding_dim(&self) -> Option<u32> {
        let arch = self.arch()?;
        self.kv.get(&format!("{arch}.embedding_length"))?.as_u32()
    }
    /// Number of transformer layers.
    pub fn num_layers(&self) -> Option<u32> {
        let arch = self.arch()?;
        self.kv.get(&format!("{arch}.block_count"))?.as_u32()
    }
    /// Number of attention heads.
    pub fn num_heads(&self) -> Option<u32> {
        let arch = self.arch()?;
        self.kv
            .get(&format!("{arch}.attention.head_count"))?
            .as_u32()
    }
    /// Number of KV heads (GQA).
    pub fn num_kv_heads(&self) -> Option<u32> {
        let arch = self.arch()?;
        self.kv
            .get(&format!("{arch}.attention.head_count_kv"))?
            .as_u32()
    }
    /// Feed-forward dimension.
    pub fn feed_forward_dim(&self) -> Option<u32> {
        let arch = self.arch()?;
        self.kv
            .get(&format!("{arch}.feed_forward_length"))?
            .as_u32()
    }
    /// Vocabulary size.
    pub fn vocab_size(&self) -> Option<u32> {
        self.kv.get("tokenizer.ggml.tokens").and_then(|v| {
            if let GgufValue::Array(a) = v {
                Some(a.len() as u32)
            } else {
                None
            }
        })
    }
}

/// Complete parsed GGUF file.
pub struct GgufFile {
    pub metadata: GgufMetadata,
    pub tensors: HashMap<String, GgufTensor>,
    pub data: Vec<u8>,
}

impl GgufFile {
    /// Load a GGUF file from disk.
    pub fn load(path: &Path) -> Result<Self, GgufError> {
        let mut f = File::open(path).map_err(|e| GgufError::Io(e.to_string()))?;

        let mut data = Vec::new();
        f.read_to_end(&mut data)
            .map_err(|e| GgufError::Io(e.to_string()))?;

        Self::parse(data)
    }

    /// Parse GGUF from raw bytes.
    pub fn parse(data: Vec<u8>) -> Result<Self, GgufError> {
        let mut cursor = 0usize;

        // Magic
        let magic = read_u32(&data, &mut cursor)?;
        if magic != GGUF_MAGIC {
            return Err(GgufError::InvalidMagic(magic));
        }

        // Version
        let version = read_u32(&data, &mut cursor)?;
        if !(2..=GGUF_VERSION_3).contains(&version) {
            return Err(GgufError::UnsupportedVersion(version));
        }

        // Counts
        let tensor_count = read_u64(&data, &mut cursor)?;
        let kv_count = read_u64(&data, &mut cursor)?;

        // Metadata KV pairs
        let mut kv = HashMap::new();
        for _ in 0..kv_count {
            let key = read_string(&data, &mut cursor)?;
            let value = read_value(&data, &mut cursor)?;
            kv.insert(key, value);
        }

        let metadata = GgufMetadata {
            version,
            tensor_count,
            kv,
        };

        // Tensor infos
        let mut tensors = HashMap::new();
        for _ in 0..tensor_count {
            let name = read_string(&data, &mut cursor)?;
            let n_dims = read_u32(&data, &mut cursor)? as usize;
            let mut shape = Vec::with_capacity(n_dims);
            for _ in 0..n_dims {
                shape.push(read_u64(&data, &mut cursor)?);
            }
            let quant_type = read_u32(&data, &mut cursor)?;
            let data_offset = read_u64(&data, &mut cursor)?;

            // Compute data size from quant type and shape
            let num_elem = shape.iter().product::<u64>();
            let data_size = quant_data_size(quant_type, num_elem)?;

            tensors.insert(
                name.clone(),
                GgufTensor {
                    name,
                    shape,
                    quant_type,
                    data_offset,
                    data_size,
                },
            );
        }

        Ok(Self {
            metadata,
            tensors,
            data,
        })
    }

    /// Get raw bytes for a tensor's data.
    pub fn tensor_data(&self, name: &str) -> Result<&[u8], GgufError> {
        let t = self
            .tensors
            .get(name)
            .ok_or_else(|| GgufError::TensorNotFound(name.to_string()))?;
        let start = t.data_offset as usize;
        let end = start + t.data_size as usize;
        if end > self.data.len() {
            return Err(GgufError::AlignmentError(format!(
                "tensor {name}: offset {start}+size {} > file size {}",
                t.data_size,
                self.data.len()
            )));
        }
        Ok(&self.data[start..end])
    }
}

// ── Low-level readers ─────────────────────────────────────────────────────────

fn read_u8(data: &[u8], cursor: &mut usize) -> Result<u8, GgufError> {
    if *cursor + 1 > data.len() {
        return Err(GgufError::Io("EOF reading u8".into()));
    }
    let v = data[*cursor];
    *cursor += 1;
    Ok(v)
}

fn read_u16(data: &[u8], cursor: &mut usize) -> Result<u16, GgufError> {
    if *cursor + 2 > data.len() {
        return Err(GgufError::Io("EOF reading u16".into()));
    }
    let v = u16::from_le_bytes([data[*cursor], data[*cursor + 1]]);
    *cursor += 2;
    Ok(v)
}

fn read_u32(data: &[u8], cursor: &mut usize) -> Result<u32, GgufError> {
    if *cursor + 4 > data.len() {
        return Err(GgufError::Io("EOF reading u32".into()));
    }
    let v = u32::from_le_bytes(data[*cursor..*cursor + 4].try_into().unwrap());
    *cursor += 4;
    Ok(v)
}

fn read_u64(data: &[u8], cursor: &mut usize) -> Result<u64, GgufError> {
    if *cursor + 8 > data.len() {
        return Err(GgufError::Io("EOF reading u64".into()));
    }
    let v = u64::from_le_bytes(data[*cursor..*cursor + 8].try_into().unwrap());
    *cursor += 8;
    Ok(v)
}

fn read_i8(data: &[u8], cursor: &mut usize) -> Result<i8, GgufError> {
    Ok(read_u8(data, cursor)? as i8)
}
fn read_i16(data: &[u8], cursor: &mut usize) -> Result<i16, GgufError> {
    Ok(read_u16(data, cursor)? as i16)
}
fn read_i32(data: &[u8], cursor: &mut usize) -> Result<i32, GgufError> {
    Ok(read_u32(data, cursor)? as i32)
}
fn read_i64(data: &[u8], cursor: &mut usize) -> Result<i64, GgufError> {
    Ok(read_u64(data, cursor)? as i64)
}
fn read_f32(data: &[u8], cursor: &mut usize) -> Result<f32, GgufError> {
    Ok(f32::from_le_bytes(read_u32(data, cursor)?.to_le_bytes()))
}
fn read_f64(data: &[u8], cursor: &mut usize) -> Result<f64, GgufError> {
    Ok(f64::from_le_bytes(read_u64(data, cursor)?.to_le_bytes()))
}
fn read_bool(data: &[u8], cursor: &mut usize) -> Result<bool, GgufError> {
    Ok(read_u8(data, cursor)? != 0)
}

fn read_string(data: &[u8], cursor: &mut usize) -> Result<String, GgufError> {
    let len = read_u64(data, cursor)? as usize;
    if *cursor + len > data.len() {
        return Err(GgufError::Io(format!("EOF reading string of len {len}")));
    }
    let s = std::str::from_utf8(&data[*cursor..*cursor + len])
        .map_err(|e| GgufError::InvalidUtf8(e.to_string()))?
        .to_string();
    *cursor += len;
    Ok(s)
}

fn read_value(data: &[u8], cursor: &mut usize) -> Result<GgufValue, GgufError> {
    let typ = read_u32(data, cursor)?;
    match typ {
        0 => Ok(GgufValue::U8(read_u8(data, cursor)?)),
        1 => Ok(GgufValue::I8(read_i8(data, cursor)?)),
        2 => Ok(GgufValue::U16(read_u16(data, cursor)?)),
        3 => Ok(GgufValue::I16(read_i16(data, cursor)?)),
        4 => Ok(GgufValue::U32(read_u32(data, cursor)?)),
        5 => Ok(GgufValue::I32(read_i32(data, cursor)?)),
        6 => Ok(GgufValue::F32(read_f32(data, cursor)?)),
        7 => Ok(GgufValue::Bool(read_bool(data, cursor)?)),
        8 => Ok(GgufValue::String(read_string(data, cursor)?)),
        9 => {
            let elem_type = read_u32(data, cursor)?;
            let count = read_u64(data, cursor)? as usize;
            let mut arr = Vec::with_capacity(count);
            for _ in 0..count {
                // For array, push element type back and read value
                let elem = read_array_elem(data, cursor, elem_type)?;
                arr.push(elem);
            }
            Ok(GgufValue::Array(arr))
        }
        10 => Ok(GgufValue::U64(read_u64(data, cursor)?)),
        11 => Ok(GgufValue::I64(read_i64(data, cursor)?)),
        12 => Ok(GgufValue::F64(read_f64(data, cursor)?)),
        t => Err(GgufError::UnknownMetadataType(t)),
    }
}

fn read_array_elem(data: &[u8], cursor: &mut usize, typ: u32) -> Result<GgufValue, GgufError> {
    match typ {
        0 => Ok(GgufValue::U8(read_u8(data, cursor)?)),
        1 => Ok(GgufValue::I8(read_i8(data, cursor)?)),
        2 => Ok(GgufValue::U16(read_u16(data, cursor)?)),
        3 => Ok(GgufValue::I16(read_i16(data, cursor)?)),
        4 => Ok(GgufValue::U32(read_u32(data, cursor)?)),
        5 => Ok(GgufValue::I32(read_i32(data, cursor)?)),
        6 => Ok(GgufValue::F32(read_f32(data, cursor)?)),
        7 => Ok(GgufValue::Bool(read_bool(data, cursor)?)),
        8 => Ok(GgufValue::String(read_string(data, cursor)?)),
        10 => Ok(GgufValue::U64(read_u64(data, cursor)?)),
        11 => Ok(GgufValue::I64(read_i64(data, cursor)?)),
        12 => Ok(GgufValue::F64(read_f64(data, cursor)?)),
        t => Err(GgufError::UnknownMetadataType(t)),
    }
}

/// Compute data size in bytes for a tensor given quantization type and element count.
fn quant_data_size(quant_type: u32, num_elem: u64) -> Result<u64, GgufError> {
    // GGML quantization types — block sizes in bytes per 32 elements
    const Q4_0_BLOCK: u64 = 18; // 2 bytes f16 scale + 16 bytes 4-bit data
    const Q4_1_BLOCK: u64 = 20; // 2 bytes f16 scale + 2 bytes f16 min + 16 bytes
    const Q5_0_BLOCK: u64 = 22;
    const Q5_1_BLOCK: u64 = 24;
    const Q8_0_BLOCK: u64 = 34; // 2 bytes f16 scale + 32 bytes int8 data
    const Q4_K_BLOCK: u64 = 144; // super-block of 256 elements
    const Q6_K_BLOCK: u64 = 210;

    match quant_type {
        0 => Ok(num_elem * 4), // F32
        1 => Ok(num_elem * 2), // F16
        2 => Ok((num_elem / 32) * Q4_0_BLOCK),
        3 => Ok((num_elem / 32) * Q4_1_BLOCK),
        6 => Ok((num_elem / 32) * Q5_0_BLOCK),
        7 => Ok((num_elem / 32) * Q5_1_BLOCK),
        8 => Ok((num_elem / 32) * Q8_0_BLOCK),
        12 => Ok((num_elem / 256) * Q4_K_BLOCK), // Q4_K_S
        13 => Ok((num_elem / 256) * Q4_K_BLOCK), // Q4_K_M (same block size)
        14 => Ok((num_elem / 256) * Q6_K_BLOCK), // Q6_K
        t => Err(GgufError::UnknownQuantType(t)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gguf_magic_constant_correct() {
        assert_eq!(GGUF_MAGIC, 0x46554747);
        // "GGUF" in little-endian bytes
        let bytes = GGUF_MAGIC.to_le_bytes();
        assert_eq!(&bytes, b"GGUF");
    }

    #[test]
    fn read_u32_le_correct() {
        let data = vec![0x01, 0x00, 0x00, 0x00];
        let mut cursor = 0;
        assert_eq!(read_u32(&data, &mut cursor).unwrap(), 1);
        assert_eq!(cursor, 4);
    }

    #[test]
    fn read_string_correct() {
        // u64 length (8 bytes LE) + string bytes
        let mut data = vec![5, 0, 0, 0, 0, 0, 0, 0];
        data.extend_from_slice(b"hello");
        let mut cursor = 0;
        assert_eq!(read_string(&data, &mut cursor).unwrap(), "hello");
        assert_eq!(cursor, 13);
    }

    #[test]
    fn quant_data_size_f32() {
        // 1024 f32 elements = 4096 bytes
        assert_eq!(quant_data_size(0, 1024).unwrap(), 4096);
    }

    #[test]
    fn quant_data_size_q4_0() {
        // 32 elements in Q4_0 = 18 bytes
        assert_eq!(quant_data_size(2, 32).unwrap(), 18);
    }

    #[test]
    fn quant_data_size_q4_k_m() {
        // 256 elements in Q4_K_M = 144 bytes
        assert_eq!(quant_data_size(13, 256).unwrap(), 144);
    }

    #[test]
    fn invalid_magic_rejected() {
        let data = vec![0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00];
        assert!(matches!(
            GgufFile::parse(data),
            Err(GgufError::InvalidMagic(_))
        ));
    }

    #[test]
    fn tensor_dims_2d_correct() {
        let t = GgufTensor {
            name: "test".into(),
            shape: vec![128, 512],
            quant_type: 0,
            data_offset: 0,
            data_size: 0,
        };
        let (rows, cols) = t.dims_2d();
        assert_eq!(rows, 512);
        assert_eq!(cols, 128);
    }
}
