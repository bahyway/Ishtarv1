#![forbid(unsafe_code)]
//! ZIP extractor — STORE (method 0) and DEFLATE (method 8), CRC-32 validated.
//! Inflate module copied inline from enkidw/src/zip_engine.rs.

use crate::error::ArchiveError;

/// One file extracted from a ZIP archive.
#[derive(Debug, Clone)]
pub struct ExtractedFile {
    pub name: String,
    pub data: Vec<u8>,
}

const LOCAL_SIG: u32 = 0x0403_4B50;
const STORE: u16 = 0;
const DEFLATE: u16 = 8;
const HDR_SIZE: usize = 30;
const FLAG_DATA_DESCRIPTOR: u16 = 0b0000_1000;

/// Extract all files from a ZIP byte slice.
/// Returns `ArchiveError::Truncated` for corrupt or truncated input.
pub fn extract_zip(data: &[u8]) -> Result<Vec<ExtractedFile>, ArchiveError> {
    if data.is_empty() {
        return Err(ArchiveError::EmptyInput);
    }
    let mut results = Vec::new();
    let mut pos = 0usize;

    loop {
        let Some(sig_pos) = find_sig(data, pos) else { break };
        pos = sig_pos;

        if pos + HDR_SIZE > data.len() {
            return Err(ArchiveError::Truncated);
        }

        let flags     = u16_le(data, pos + 6);
        let method    = u16_le(data, pos + 8);
        let crc_hdr    = u32_le(data, pos + 14);
        let csize      = u32_le(data, pos + 18) as usize;
        let usize_val  = u32_le(data, pos + 22) as usize;
        let fname_len = u16_le(data, pos + 26) as usize;
        let extra_len = u16_le(data, pos + 28) as usize;
        let data_start = pos + HDR_SIZE + fname_len + extra_len;

        // Reject data-descriptor entries with 0 compressed size
        if csize == 0 && (flags & FLAG_DATA_DESCRIPTOR) != 0 {
            pos = data_start;
            continue;
        }

        if data_start + csize > data.len() {
            return Err(ArchiveError::Truncated);
        }

        if pos + HDR_SIZE + fname_len > data.len() {
            return Err(ArchiveError::Truncated);
        }

        let name_bytes = &data[pos + HDR_SIZE .. pos + HDR_SIZE + fname_len];
        let name = core::str::from_utf8(name_bytes)
            .map_err(|_| ArchiveError::Truncated)?
            .to_string();

        // Skip directory entries
        if name.ends_with('/') {
            pos = data_start + csize;
            continue;
        }

        let compressed = &data[data_start .. data_start + csize];

        let file_data = match method {
            STORE => compressed.to_vec(),
            DEFLATE => {
                inflate::inflate(compressed, usize_val)
                    .ok_or(ArchiveError::Truncated)?
            }
            _ => {
                // Unsupported method — skip
                pos = data_start + csize;
                continue;
            }
        };

        // CRC-32 validation
        let actual_crc = crc32(&file_data);
        if crc_hdr != 0 && actual_crc != crc_hdr {
            return Err(ArchiveError::Truncated);
        }

        results.push(ExtractedFile { name, data: file_data });
        pos = data_start + csize;
    }

    Ok(results)
}

// ── CRC-32 (reflected, polynomial 0xEDB88320) ─────────────────────────────────

pub(crate) fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            if crc & 1 == 1 { crc = (crc >> 1) ^ 0xEDB8_8320; }
            else             { crc >>= 1; }
        }
    }
    crc ^ 0xFFFF_FFFF
}

/// Public entry point for gzip.rs to call inflate without going through ZIP parsing.
pub(crate) fn inflate_raw(input: &[u8], expected_len: usize) -> Option<Vec<u8>> {
    inflate::inflate(input, expected_len)
}

// ── Helper readers ─────────────────────────────────────────────────────────────

fn find_sig(data: &[u8], from: usize) -> Option<usize> {
    let needle = LOCAL_SIG.to_le_bytes();
    for i in from..data.len().saturating_sub(3) {
        if data[i..i+4] == needle { return Some(i); }
    }
    None
}

fn u16_le(data: &[u8], off: usize) -> u16 {
    u16::from_le_bytes([data[off], data[off+1]])
}

fn u32_le(data: &[u8], off: usize) -> u32 {
    u32::from_le_bytes(data[off..off+4].try_into().unwrap())
}

// ── Build helper (STORE only — for tests) ─────────────────────────────────────

pub fn build_store_zip(name: &str, data: &[u8]) -> Vec<u8> {
    let fname  = name.as_bytes();
    let crc    = crc32(data);
    let dlen   = data.len() as u32;
    let flen   = fname.len() as u16;

    let mut out = Vec::with_capacity(HDR_SIZE + fname.len() + data.len() + 68);
    out.extend_from_slice(&LOCAL_SIG.to_le_bytes());
    out.extend_from_slice(&20u16.to_le_bytes());     // version needed
    out.extend_from_slice(&0u16.to_le_bytes());      // flags
    out.extend_from_slice(&STORE.to_le_bytes());     // method
    out.extend_from_slice(&0u16.to_le_bytes());      // mod time
    out.extend_from_slice(&0u16.to_le_bytes());      // mod date
    out.extend_from_slice(&crc.to_le_bytes());
    out.extend_from_slice(&dlen.to_le_bytes());
    out.extend_from_slice(&dlen.to_le_bytes());
    out.extend_from_slice(&flen.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());      // extra len
    out.extend_from_slice(fname);
    out.extend_from_slice(data);
    out
}

// ── DEFLATE inflate (RFC 1951) — pure safe Rust, copied from zip_engine.rs ────

mod inflate {
    const MAX_BITS: u8 = 15;

    struct BitReader<'a> {
        data: &'a [u8],
        buf:  u32,
        bits: u8,
        pos:  usize,
    }

    impl<'a> BitReader<'a> {
        fn new(data: &'a [u8]) -> Self {
            Self { data, buf: 0, bits: 0, pos: 0 }
        }
        fn fill(&mut self) {
            while self.bits <= 24 {
                let byte = if self.pos < self.data.len() {
                    let b = self.data[self.pos]; self.pos += 1; b
                } else { 0u8 };
                self.buf |= (byte as u32) << self.bits;
                self.bits += 8;
            }
        }
        #[inline]
        fn peek_bits(&mut self, n: u8) -> u32 {
            self.fill();
            self.buf & ((1u32 << n) - 1)
        }
        #[inline]
        fn consume(&mut self, n: u8) { self.buf >>= n; self.bits -= n; }
        #[inline]
        fn read_bits(&mut self, n: u8) -> u32 { let v = self.peek_bits(n); self.consume(n); v }
        fn align_to_byte(&mut self) { self.consume(self.bits % 8); }
        fn read_u16_le(&mut self) -> u16 {
            let lo = self.read_bits(8) as u16;
            let hi = self.read_bits(8) as u16;
            lo | (hi << 8)
        }
    }

    #[derive(Clone, Copy)]
    struct E { sym: u16, len: u8 }
    struct Huff { table: Vec<E> }

    impl Huff {
        fn build(lengths: &[u8]) -> Option<Self> {
            let mut count = [0u16; 16];
            for &l in lengths {
                if l > 0 { if l > MAX_BITS { return None; } count[l as usize] += 1; }
            }
            let mut code = 0u32;
            let mut next = [0u32; 16];
            for bits in 1..=MAX_BITS as usize {
                code = (code + count[bits - 1] as u32) << 1;
                next[bits] = code;
            }
            let sz = 1usize << MAX_BITS;
            let mut table = vec![E { sym: 0xFFFF, len: 0 }; sz];
            for (sym, &len) in lengths.iter().enumerate() {
                if len == 0 { continue; }
                let c = next[len as usize];
                next[len as usize] += 1;
                let rev_c = rev(c, len) as usize;
                let extra = (MAX_BITS - len) as usize;
                for fill in 0..(1usize << extra) {
                    table[rev_c | (fill << len)] = E { sym: sym as u16, len };
                }
            }
            Some(Huff { table })
        }
        #[inline]
        fn decode(&self, r: &mut BitReader) -> Option<u16> {
            let idx = r.peek_bits(MAX_BITS) as usize;
            let e = self.table[idx];
            if e.len == 0 { return None; }
            r.consume(e.len);
            Some(e.sym)
        }
    }

    fn rev(v: u32, bits: u8) -> u32 {
        let mut r = 0u32; let mut v = v;
        for _ in 0..bits { r = (r << 1) | (v & 1); v >>= 1; }
        r
    }

    const LEN_BASE:  [u16; 29] = [3,4,5,6,7,8,9,10,11,13,15,17,19,23,27,31,35,43,51,59,67,83,99,115,131,163,195,227,258];
    const LEN_XBITS: [u8; 29]  = [0,0,0,0,0,0,0,0,1,1,1,1,2,2,2,2,3,3,3,3,4,4,4,4,5,5,5,5,0];
    const DST_BASE:  [u32; 30] = [1,2,3,4,5,7,9,13,17,25,33,49,65,97,129,193,257,385,513,769,1025,1537,2049,3073,4097,6145,8193,12289,16385,24577];
    const DST_XBITS: [u8; 30]  = [0,0,0,0,1,1,2,2,3,3,4,4,5,5,6,6,7,7,8,8,9,9,10,10,11,11,12,12,13,13];
    const CL_ORDER: [usize; 19] = [16,17,18,0,8,7,9,6,10,5,11,4,12,3,13,2,14,1,15];

    fn fixed_ll_lengths() -> [u8; 288] {
        let mut l = [0u8; 288];
        for i in   0..=143 { l[i] = 8; }
        for i in 144..=255 { l[i] = 9; }
        for i in 256..=279 { l[i] = 7; }
        for i in 280..=287 { l[i] = 8; }
        l
    }

    fn back_copy(out: &mut Vec<u8>, dist: usize, len: usize) -> Option<()> {
        if dist == 0 || dist > out.len() { return None; }
        let start = out.len() - dist;
        out.reserve(len);
        for i in 0..len { let b = out[start + (i % dist)]; out.push(b); }
        Some(())
    }

    fn decode_block(r: &mut BitReader, out: &mut Vec<u8>, ll: &Huff, dst: &Huff) -> Option<()> {
        loop {
            let sym = ll.decode(r)?;
            if sym < 256 { out.push(sym as u8); }
            else if sym == 256 { break; }
            else {
                let li = (sym - 257) as usize;
                if li >= LEN_BASE.len() { return None; }
                let len = LEN_BASE[li] as usize + r.read_bits(LEN_XBITS[li]) as usize;
                let di  = dst.decode(r)? as usize;
                if di >= DST_BASE.len() { return None; }
                let dist = DST_BASE[di] as usize + r.read_bits(DST_XBITS[di]) as usize;
                back_copy(out, dist, len)?;
            }
        }
        Some(())
    }

    fn stored_block(r: &mut BitReader, out: &mut Vec<u8>) -> Option<()> {
        r.align_to_byte();
        let len  = r.read_u16_le() as usize;
        let nlen = r.read_u16_le() as usize;
        if (len ^ nlen) != 0xFFFF { return None; }
        out.reserve(len);
        for _ in 0..len { out.push(r.read_bits(8) as u8); }
        Some(())
    }

    fn fixed_block(r: &mut BitReader, out: &mut Vec<u8>) -> Option<()> {
        let ll  = Huff::build(&fixed_ll_lengths())?;
        let dst = Huff::build(&[5u8; 32])?;
        decode_block(r, out, &ll, &dst)
    }

    fn dynamic_block(r: &mut BitReader, out: &mut Vec<u8>) -> Option<()> {
        let hlit  = r.read_bits(5) as usize + 257;
        let hdist = r.read_bits(5) as usize + 1;
        let hclen = r.read_bits(4) as usize + 4;
        let mut cl = [0u8; 19];
        for i in 0..hclen { cl[CL_ORDER[i]] = r.read_bits(3) as u8; }
        let cl_huff = Huff::build(&cl)?;
        let total = hlit + hdist;
        let mut lengths = vec![0u8; total];
        let mut i = 0;
        while i < total {
            let sym = cl_huff.decode(r)? as u8;
            match sym {
                0..=15 => { lengths[i] = sym; i += 1; }
                16 => {
                    if i == 0 { return None; }
                    let rep = r.read_bits(2) as usize + 3;
                    let prev = lengths[i - 1];
                    for _ in 0..rep { if i >= total { return None; } lengths[i] = prev; i += 1; }
                }
                17 => {
                    let rep = r.read_bits(3) as usize + 3;
                    for _ in 0..rep { if i >= total { return None; } lengths[i] = 0; i += 1; }
                }
                18 => {
                    let rep = r.read_bits(7) as usize + 11;
                    for _ in 0..rep { if i >= total { return None; } lengths[i] = 0; i += 1; }
                }
                _ => return None,
            }
        }
        let ll  = Huff::build(&lengths[..hlit])?;
        let dst = Huff::build(&lengths[hlit..])?;
        decode_block(r, out, &ll, &dst)
    }

    /// Decompress raw DEFLATE data. Returns None on any decode error.
    pub fn inflate(input: &[u8], expected_len: usize) -> Option<Vec<u8>> {
        let mut r   = BitReader::new(input);
        let cap     = expected_len.min(64 * 1024 * 1024);
        let mut out = Vec::with_capacity(cap);
        loop {
            let bfinal = r.read_bits(1);
            let btype  = r.read_bits(2);
            match btype {
                0b00 => stored_block(&mut r, &mut out)?,
                0b01 => fixed_block(&mut r, &mut out)?,
                0b10 => dynamic_block(&mut r, &mut out)?,
                _    => return None,
            }
            if bfinal == 1 { break; }
        }
        Some(out)
    }
}
