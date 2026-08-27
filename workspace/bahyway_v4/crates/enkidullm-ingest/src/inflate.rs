//! Native DEFLATE decompressor (RFC 1951) — pure Rust, zero external deps.
//!
//! Handles all three block types:
//!   BTYPE=00  Stored (uncompressed)
//!   BTYPE=01  Fixed Huffman codes
//!   BTYPE=10  Dynamic Huffman codes
//!
//! Used by both the PDF FlateDecode stream handler and the Epub ZIP reader.

// ── RFC 1951 constant tables ──────────────────────────────────────────────────

/// Length base values for codes 257–285.
const LENGTH_BASE: [u16; 29] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131,
    163, 195, 227, 258,
];
/// Extra bits for length codes 257–285.
const LENGTH_EXTRA: [u8; 29] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
];
/// Distance base values for codes 0–29.
const DIST_BASE: [u32; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
    2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
];
/// Extra bits for distance codes 0–29.
const DIST_EXTRA: [u8; 30] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13,
    13,
];
/// Code length alphabet reorder (RFC 1951 §3.2.7).
const CL_ORDER: [usize; 19] = [
    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
];

// ── Bit reader ────────────────────────────────────────────────────────────────

struct BitReader<'a> {
    data: &'a [u8],
    pos: usize,
    buf: u32, // bit accumulator
    bits: u8, // valid bits in buf
}

impl<'a> BitReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            pos: 0,
            buf: 0,
            bits: 0,
        }
    }

    fn fill(&mut self) {
        while self.bits <= 24 && self.pos < self.data.len() {
            self.buf |= (self.data[self.pos] as u32) << self.bits;
            self.bits += 8;
            self.pos += 1;
        }
    }

    fn read_bits(&mut self, n: u8) -> Option<u32> {
        self.fill();
        if self.bits < n {
            return None;
        }
        let v = self.buf & ((1u32 << n) - 1);
        self.buf >>= n;
        self.bits -= n;
        Some(v)
    }

    fn align_to_byte(&mut self) {
        let rem = self.bits & 7;
        if rem > 0 {
            self.buf >>= rem;
            self.bits -= rem;
        }
        // Rewind pos by the number of pre-loaded bytes still in the buffer so
        // subsequent raw byte reads start at the correct stream position.
        let buffered = (self.bits / 8) as usize;
        self.pos -= buffered;
        self.buf = 0;
        self.bits = 0;
    }
}

// ── Canonical Huffman decoder ─────────────────────────────────────────────────

/// Canonical Huffman decoder — built from code lengths per RFC 1951 §3.2.2.
struct HuffDecoder {
    /// Sorted (bit_length, code, symbol) triples.
    table: Vec<(u8, u16, u16)>,
}

impl HuffDecoder {
    fn build(lengths: &[u8]) -> Self {
        let max_len = lengths.iter().copied().max().unwrap_or(0) as usize;
        let mut count = vec![0u32; max_len + 1];
        for &l in lengths {
            if l > 0 {
                count[l as usize] += 1;
            }
        }

        let mut first_code = vec![0u16; max_len + 2];
        for i in 1..=max_len {
            first_code[i] = (first_code[i - 1] + count[i - 1] as u16) << 1;
        }

        let mut next = first_code.clone();
        let mut table = Vec::new();
        for (sym, &l) in lengths.iter().enumerate() {
            if l > 0 {
                let code = next[l as usize];
                next[l as usize] += 1;
                table.push((l, code, sym as u16));
            }
        }
        table.sort_unstable_by_key(|&(l, c, _)| ((l as u32) << 16) | c as u32);
        Self { table }
    }

    fn decode(&self, reader: &mut BitReader) -> Option<u16> {
        let mut code = 0u16;
        let mut len = 0u8;
        for _ in 0..16u8 {
            code = (code << 1) | reader.read_bits(1)? as u16;
            len += 1;
            for &(l, c, sym) in &self.table {
                if l == len && c == code {
                    return Some(sym);
                }
                if l > len {
                    break;
                }
            }
        }
        None
    }
}

// ── Fixed Huffman tables ──────────────────────────────────────────────────────

fn fixed_lit_lengths() -> Vec<u8> {
    let mut v = vec![0u8; 288];
    for x in v.iter_mut().take(144) {
        *x = 8;
    }
    for x in v.iter_mut().take(256).skip(144) {
        *x = 9;
    }
    for x in v.iter_mut().take(280).skip(256) {
        *x = 7;
    }
    for x in v.iter_mut().take(288).skip(280) {
        *x = 8;
    }
    v
}

fn fixed_dist_lengths() -> Vec<u8> {
    vec![5u8; 32]
}

// ── Block decompressor ────────────────────────────────────────────────────────

fn decompress_block(
    reader: &mut BitReader,
    lit: &HuffDecoder,
    dist: &HuffDecoder,
    out: &mut Vec<u8>,
) -> Result<(), &'static str> {
    loop {
        let sym = lit.decode(reader).ok_or("truncated: literal")?;
        if sym < 256 {
            out.push(sym as u8);
        } else if sym == 256 {
            break;
        } else {
            let li = (sym - 257) as usize;
            if li >= 29 {
                return Err("invalid length code");
            }
            let extra_len = reader
                .read_bits(LENGTH_EXTRA[li])
                .ok_or("truncated: len extra")?;
            let length = LENGTH_BASE[li] as u32 + extra_len;

            let dc = dist.decode(reader).ok_or("truncated: dist code")? as usize;
            if dc >= 30 {
                return Err("invalid distance code");
            }
            let extra_dist = reader
                .read_bits(DIST_EXTRA[dc])
                .ok_or("truncated: dist extra")?;
            let distance = DIST_BASE[dc] + extra_dist;

            if distance as usize > out.len() {
                return Err("back-ref before start");
            }
            let start = out.len() - distance as usize;
            for i in 0..length as usize {
                let b = out[start + (i % distance as usize)];
                out.push(b);
            }
        }
    }
    Ok(())
}

fn read_dyn_lengths(
    reader: &mut BitReader,
    cl: &HuffDecoder,
    total: usize,
) -> Result<Vec<u8>, &'static str> {
    let mut lens = Vec::with_capacity(total);
    while lens.len() < total {
        let sym = cl.decode(reader).ok_or("truncated: code length")?;
        match sym {
            0..=15 => lens.push(sym as u8),
            16 => {
                let n = reader.read_bits(2).ok_or("truncated: rep16")? as usize + 3;
                let last = *lens.last().ok_or("rep16 with no prev")?;
                for _ in 0..n {
                    lens.push(last);
                }
            }
            17 => {
                let n = reader.read_bits(3).ok_or("truncated: rep17")? as usize + 3;
                lens.extend(std::iter::repeat_n(0, n));
            }
            18 => {
                let n = reader.read_bits(7).ok_or("truncated: rep18")? as usize + 11;
                lens.extend(std::iter::repeat_n(0, n));
            }
            _ => return Err("invalid cl symbol"),
        }
    }
    if lens.len() != total {
        return Err("cl count mismatch");
    }
    Ok(lens)
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Decompress a DEFLATE-compressed byte slice (RFC 1951, no zlib/gzip wrapper).
pub fn inflate(compressed: &[u8]) -> Result<Vec<u8>, &'static str> {
    let mut reader = BitReader::new(compressed);
    let mut out = Vec::new();

    loop {
        let bfinal = reader.read_bits(1).ok_or("truncated: bfinal")?;
        let btype = reader.read_bits(2).ok_or("truncated: btype")?;

        match btype {
            0b00 => {
                reader.align_to_byte();
                // After align_to_byte the bit buffer is empty; read raw bytes.
                let len = {
                    let lo = *reader
                        .data
                        .get(reader.pos)
                        .ok_or("truncated: stored len lo")? as u16;
                    let hi = *reader
                        .data
                        .get(reader.pos + 1)
                        .ok_or("truncated: stored len hi")? as u16;
                    reader.pos += 2;
                    lo | (hi << 8)
                };
                let nlen = {
                    let lo = *reader
                        .data
                        .get(reader.pos)
                        .ok_or("truncated: stored nlen lo")? as u16;
                    let hi = *reader
                        .data
                        .get(reader.pos + 1)
                        .ok_or("truncated: stored nlen hi")? as u16;
                    reader.pos += 2;
                    lo | (hi << 8)
                };
                if (len ^ nlen) != 0xFFFF {
                    return Err("stored nlen mismatch");
                }
                let end = reader.pos + len as usize;
                if end > reader.data.len() {
                    return Err("truncated: stored data");
                }
                out.extend_from_slice(&reader.data[reader.pos..end]);
                reader.pos = end;
            }
            0b01 => {
                let lit = HuffDecoder::build(&fixed_lit_lengths());
                let dist = HuffDecoder::build(&fixed_dist_lengths());
                decompress_block(&mut reader, &lit, &dist, &mut out)?;
            }
            0b10 => {
                let hlit = reader.read_bits(5).ok_or("truncated: hlit")? as usize + 257;
                let hdist = reader.read_bits(5).ok_or("truncated: hdist")? as usize + 1;
                let hclen = reader.read_bits(4).ok_or("truncated: hclen")? as usize + 4;

                let mut cl_lens = [0u8; 19];
                for i in 0..hclen {
                    cl_lens[CL_ORDER[i]] =
                        reader.read_bits(3).ok_or("truncated: hclen bits")? as u8;
                }
                let cl = HuffDecoder::build(&cl_lens);

                let combined = read_dyn_lengths(&mut reader, &cl, hlit + hdist)?;
                let lit = HuffDecoder::build(&combined[..hlit]);
                let dist = HuffDecoder::build(&combined[hlit..]);
                decompress_block(&mut reader, &lit, &dist, &mut out)?;
            }
            _ => return Err("reserved btype"),
        }

        if bfinal == 1 {
            break;
        }
    }
    Ok(out)
}

/// Decompress a zlib-wrapped DEFLATE stream (PDF FlateDecode uses zlib header).
/// Strips the 2-byte zlib header and 4-byte Adler-32 trailer, then inflates.
pub fn inflate_zlib(data: &[u8]) -> Result<Vec<u8>, &'static str> {
    if data.len() < 6 {
        return Err("zlib: too short");
    }
    // zlib header: CMF byte (0x78) + FLG byte; skip both
    let deflate_data = if data[0] == 0x78 { &data[2..] } else { data };
    // Strip trailing 4-byte Adler-32 (if present)
    let end = if deflate_data.len() > 4 {
        deflate_data.len() - 4
    } else {
        deflate_data.len()
    };
    inflate(&deflate_data[..end])
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stored_block_roundtrip() {
        // Manually constructed stored block: BFINAL=1, BTYPE=00
        // LEN=5, NLEN=~5=0xFFFA, then 5 bytes "hello"
        let data = b"hello";
        let len = data.len() as u16;
        let nlen = !len;
        let mut block = vec![
            0b00000001u8, // BFINAL=1, BTYPE=00 (bits 0,1,2 = 1,0,0)
        ];
        block.extend_from_slice(&len.to_le_bytes());
        block.extend_from_slice(&nlen.to_le_bytes());
        block.extend_from_slice(data);
        let result = inflate(&block).unwrap();
        assert_eq!(result, b"hello");
    }

    #[test]
    fn inflate_fixed_huffman() {
        // DEFLATE fixed-Huffman stream for b"aaaaaa" (6 literal 'a' codes + end-of-block).
        // Manually computed: BFINAL=1, BTYPE=01; 'a'=97 → code 145 (8-bit), ×6;
        // end-of-block 256 → code 0 (7-bit). Total 58 bits → 8 bytes.
        // Byte sequence: 0x4b 0x4c 0x4c 0x4c 0x4c 0x4c 0x04 0x00
        let deflated = [0x4bu8, 0x4c, 0x4c, 0x4c, 0x4c, 0x4c, 0x04, 0x00];
        let result = inflate(&deflated).unwrap();
        assert_eq!(result, b"aaaaaa");
    }

    #[test]
    fn inflate_zlib_wrapper() {
        // Python: zlib.compress(b"hello world")
        let compressed = [
            0x78, 0x9c, 0xcb, 0x48, 0xcd, 0xc9, 0xc9, 0x57, 0x28, 0xcf, 0x2f, 0xca, 0x49, 0x01,
            0x00, 0x1a, 0x0b, 0x04, 0x5d,
        ];
        let result = inflate_zlib(&compressed).unwrap();
        assert_eq!(result, b"hello world");
    }

    #[test]
    fn inflate_stored_large() {
        // Two stored blocks chained
        let content = vec![0xABu8; 300];
        // Build a two-block stored stream
        let mut stream = Vec::new();
        // Block 1: first 200 bytes, BFINAL=0
        stream.push(0b00000000u8); // BFINAL=0, BTYPE=00
        let l1 = 200u16;
        stream.extend_from_slice(&l1.to_le_bytes());
        stream.extend_from_slice(&(!l1).to_le_bytes());
        stream.extend_from_slice(&content[..200]);
        // Block 2: last 100 bytes, BFINAL=1
        stream.push(0b00000001u8); // BFINAL=1, BTYPE=00
        let l2 = 100u16;
        stream.extend_from_slice(&l2.to_le_bytes());
        stream.extend_from_slice(&(!l2).to_le_bytes());
        stream.extend_from_slice(&content[200..]);
        let result = inflate(&stream).unwrap();
        assert_eq!(result, content);
    }

    #[test]
    fn empty_stored_block() {
        let mut block = vec![0b00000001u8]; // BFINAL=1, BTYPE=00
        block.extend_from_slice(&0u16.to_le_bytes());
        block.extend_from_slice(&0xFFFFu16.to_le_bytes());
        let result = inflate(&block).unwrap();
        assert!(result.is_empty());
    }
}
