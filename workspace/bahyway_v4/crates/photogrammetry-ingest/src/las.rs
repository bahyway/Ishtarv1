//! LAS (ASPRS "LASer" format) binary point-cloud parsing — the industry
//! standard for aerial LiDAR/photogrammetry point clouds.
//!
//! HONEST SCOPE: this implements the real LAS 1.2-1.4 public-header-block
//! layout and point data record formats 0-3 (the common baseline: X/Y/Z,
//! intensity, classification, optionally GPS time and/or RGB) directly
//! against the ASPRS spec's documented byte offsets — no external crate,
//! no LASzip. Two things are deliberately NOT supported, and rejected
//! with a clear error rather than silently misparsed:
//!   - **LAZ (LASzip-compressed) data.** Detected either by a `.laz`
//!     extension (rejected in `lib.rs::parse_point_cloud_file`) or by
//!     the point-data-format byte's high bit being set (the real LASzip
//!     convention for a compressed format flag inside an otherwise-LAS
//!     file). Decompressing LASzip is a real, separate compression
//!     scheme this crate does not reimplement.
//!   - **Point data record formats 4-10** (LAS 1.4 extended formats,
//!     wave-packet formats) — these use different core byte layouts
//!     (e.g. format 6+ has a 16-bit classification field, not 8-bit) that
//!     were not verified against the spec here; only 0-3 are implemented.

use crate::{ParseError, PointCloud};

const HEADER_MIN_LEN: usize = 179; // enough to read scale/offset (byte 179 is where Max X begins)
const LASZIP_COMPRESSED_BIT: u8 = 0x80;

fn u16_le(b: &[u8], off: usize) -> Result<u16, ParseError> {
    let s: [u8; 2] = b.get(off..off + 2).ok_or_else(too_short)?.try_into().unwrap();
    Ok(u16::from_le_bytes(s))
}
fn u32_le(b: &[u8], off: usize) -> Result<u32, ParseError> {
    let s: [u8; 4] = b.get(off..off + 4).ok_or_else(too_short)?.try_into().unwrap();
    Ok(u32::from_le_bytes(s))
}
fn i32_le(b: &[u8], off: usize) -> Result<i32, ParseError> {
    let s: [u8; 4] = b.get(off..off + 4).ok_or_else(too_short)?.try_into().unwrap();
    Ok(i32::from_le_bytes(s))
}
fn f64_le(b: &[u8], off: usize) -> Result<f64, ParseError> {
    let s: [u8; 8] = b.get(off..off + 8).ok_or_else(too_short)?.try_into().unwrap();
    Ok(f64::from_le_bytes(s))
}
fn u16_le_rgb(b: &[u8], off: usize) -> Result<u8, ParseError> {
    // LAS RGB channels are stored as u16 (commonly the high byte carries
    // the real 8-bit value) -- downscale to a single 0-255 byte.
    Ok((u16_le(b, off)? >> 8) as u8)
}
fn too_short() -> ParseError {
    ParseError("LAS file truncated: header or point record shorter than the spec requires".to_string())
}

pub fn parse_las(bytes: &[u8]) -> Result<PointCloud, ParseError> {
    if bytes.len() < HEADER_MIN_LEN || &bytes[0..4] != b"LASF" {
        return Err(ParseError("not a LAS file (expected 'LASF' signature)".to_string()));
    }
    let version_major = bytes[24];
    let version_minor = bytes[25];
    let offset_to_point_data = u32_le(bytes, 96)? as usize;
    let point_data_format_byte = bytes[104];
    let point_data_record_length = u16_le(bytes, 105)? as usize;
    let num_point_records = u32_le(bytes, 107)? as usize;

    if point_data_format_byte & LASZIP_COMPRESSED_BIT != 0 {
        return Err(ParseError(
            "this LAS file's point data is LASzip-compressed (compressed-format bit set) -- \
             LAZ decompression is not implemented; decompress with an external tool first"
                .to_string(),
        ));
    }
    let format = point_data_format_byte;
    if format > 3 {
        return Err(ParseError(format!(
            "point data record format {format} (LAS {version_major}.{version_minor}) is not \
             supported -- only formats 0-3 are implemented (see las module doc)"
        )));
    }

    let x_scale = f64_le(bytes, 131)?;
    let y_scale = f64_le(bytes, 139)?;
    let z_scale = f64_le(bytes, 147)?;
    let x_offset = f64_le(bytes, 155)?;
    let y_offset = f64_le(bytes, 163)?;
    let z_offset = f64_le(bytes, 171)?;

    let has_rgb = matches!(format, 2 | 3);
    let rgb_rel_offset = match format {
        2 => 20, // format 0's 20-byte core, then RGB
        3 => 28, // format 1's 20-byte core + 8-byte GPS time, then RGB
        _ => 0,
    };

    let mut points = Vec::with_capacity(num_point_records);
    let mut colors: Option<Vec<[u8; 3]>> = if has_rgb { Some(Vec::with_capacity(num_point_records)) } else { None };

    for i in 0..num_point_records {
        let rec_start = offset_to_point_data + i * point_data_record_length;
        let rec = bytes
            .get(rec_start..rec_start + point_data_record_length)
            .ok_or_else(|| ParseError(format!("LAS file truncated: expected {num_point_records} point records, ran out at record {i}")))?;
        let raw_x = i32_le(rec, 0)?;
        let raw_y = i32_le(rec, 4)?;
        let raw_z = i32_le(rec, 8)?;
        points.push([
            raw_x as f64 * x_scale + x_offset,
            raw_y as f64 * y_scale + y_offset,
            raw_z as f64 * z_scale + z_offset,
        ]);
        if has_rgb {
            let r = u16_le_rgb(rec, rgb_rel_offset)?;
            let g = u16_le_rgb(rec, rgb_rel_offset + 2)?;
            let b = u16_le_rgb(rec, rgb_rel_offset + 4)?;
            colors.as_mut().unwrap().push([r, g, b]);
        }
    }

    Ok(PointCloud { points, colors })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Hand-assembles a minimal, spec-correct LAS 1.2 file in memory: a
    /// real public header block (227 bytes) plus `format 0` point
    /// records (X/Y/Z as scaled/offset i32, padded to the rest of the
    /// 20-byte record with zeros) -- enough to prove this module's
    /// reader recovers exactly the coordinates that went in, against a
    /// real byte layout, not a stubbed one.
    fn build_minimal_las_format0(points_raw: &[(i32, i32, i32)], scale: f64, offset: f64) -> Vec<u8> {
        let mut h = vec![0u8; 227];
        h[0..4].copy_from_slice(b"LASF");
        h[24] = 1; // version major
        h[25] = 2; // version minor
        h[94..96].copy_from_slice(&227u16.to_le_bytes()); // header size
        h[96..100].copy_from_slice(&227u32.to_le_bytes()); // offset to point data
        h[104] = 0; // point data format 0
        h[105..107].copy_from_slice(&20u16.to_le_bytes()); // record length
        h[107..111].copy_from_slice(&(points_raw.len() as u32).to_le_bytes());
        h[131..139].copy_from_slice(&scale.to_le_bytes());
        h[139..147].copy_from_slice(&scale.to_le_bytes());
        h[147..155].copy_from_slice(&scale.to_le_bytes());
        h[155..163].copy_from_slice(&offset.to_le_bytes());
        h[163..171].copy_from_slice(&offset.to_le_bytes());
        h[171..179].copy_from_slice(&offset.to_le_bytes());

        for &(x, y, z) in points_raw {
            let mut rec = vec![0u8; 20];
            rec[0..4].copy_from_slice(&x.to_le_bytes());
            rec[4..8].copy_from_slice(&y.to_le_bytes());
            rec[8..12].copy_from_slice(&z.to_le_bytes());
            h.extend_from_slice(&rec);
        }
        h
    }

    #[test]
    fn round_trips_real_scaled_offset_coordinates() {
        // scale 0.001, offset 500.0: raw i32 12345 -> 500.0 + 12345*0.001 = 512.345
        let bytes = build_minimal_las_format0(&[(12345, -6789, 0), (0, 0, 100000)], 0.001, 500.0);
        let cloud = parse_las(&bytes).unwrap();
        assert_eq!(cloud.points.len(), 2);
        assert!((cloud.points[0][0] - 512.345).abs() < 1e-9);
        assert!((cloud.points[0][1] - (500.0 - 6.789)).abs() < 1e-9);
        assert!((cloud.points[0][2] - 500.0).abs() < 1e-9);
        assert!((cloud.points[1][2] - 600.0).abs() < 1e-9);
        assert!(cloud.colors.is_none());
    }

    #[test]
    fn rejects_truncated_file() {
        let err = parse_las(&[0u8; 50]).unwrap_err();
        assert!(err.0.contains("LASF") || err.0.contains("truncated"));
    }

    #[test]
    fn rejects_unsupported_point_format() {
        let mut bytes = build_minimal_las_format0(&[(0, 0, 0)], 1.0, 0.0);
        bytes[104] = 6; // LAS 1.4 extended format, not implemented
        let err = parse_las(&bytes).unwrap_err();
        assert!(err.0.contains("not supported"), "{}", err.0);
    }

    #[test]
    fn rejects_laszip_compressed_flag() {
        let mut bytes = build_minimal_las_format0(&[(0, 0, 0)], 1.0, 0.0);
        bytes[104] = 0 | LASZIP_COMPRESSED_BIT; // format 0, compressed bit set
        let err = parse_las(&bytes).unwrap_err();
        assert!(err.0.contains("LASzip"), "{}", err.0);
    }

    #[test]
    fn rejects_non_las_file() {
        let err = parse_las(&[0u8; 300]).unwrap_err();
        assert!(err.0.contains("LASF"));
    }
}
