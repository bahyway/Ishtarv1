//! ASCII PLY (Polygon File Format / Stanford Triangle Format) parsing —
//! the most common dense-point-cloud export format from photogrammetry
//! tools (OpenDroneMap, Meshroom, Agisoft all emit it).
//!
//! HONEST SCOPE: only `format ascii 1.0` is supported. `binary_little_
//! endian`/`binary_big_endian` PLY (a real, common variant some tools
//! prefer for file size) is explicitly rejected with a clear error
//! rather than silently misparsed as text — not implemented here.
//! Non-vertex elements (commonly `face`, for a meshed rather than raw
//! point-cloud export) are skipped by line count, never parsed as
//! geometry: this crate reads point clouds, not meshes.

use crate::{ParseError, PointCloud};

struct ElementDecl {
    name: String,
    count: usize,
    properties: Vec<String>,
}

pub fn parse_ply(bytes: &[u8]) -> Result<PointCloud, ParseError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| ParseError("PLY file is not valid UTF-8 (likely a binary-format PLY, which is not supported -- see ply module doc)".to_string()))?;
    let mut lines = text.lines();

    let magic = lines.next().unwrap_or("").trim();
    if magic != "ply" {
        return Err(ParseError(format!("not a PLY file (expected 'ply' magic line, got '{magic}')")));
    }

    let mut elements: Vec<ElementDecl> = Vec::new();
    let mut format_seen = false;
    for line in &mut lines {
        let line = line.trim();
        if line.is_empty() || line.starts_with("comment") {
            continue;
        }
        if let Some(fmt) = line.strip_prefix("format ") {
            if !fmt.trim().starts_with("ascii") {
                return Err(ParseError(format!(
                    "unsupported PLY format '{}': only ASCII PLY is supported",
                    fmt.trim()
                )));
            }
            format_seen = true;
        } else if let Some(rest) = line.strip_prefix("element ") {
            let mut parts = rest.split_whitespace();
            let name = parts.next().ok_or_else(|| ParseError("malformed 'element' line".to_string()))?;
            let count: usize = parts
                .next()
                .ok_or_else(|| ParseError("malformed 'element' line: missing count".to_string()))?
                .parse()
                .map_err(|_| ParseError("malformed 'element' line: count is not a number".to_string()))?;
            elements.push(ElementDecl { name: name.to_string(), count, properties: Vec::new() });
        } else if let Some(rest) = line.strip_prefix("property ") {
            let elem = elements
                .last_mut()
                .ok_or_else(|| ParseError("'property' line before any 'element'".to_string()))?;
            // "property float x" or "property list uchar int vertex_indices"
            // -- in both cases the LAST whitespace-separated token is the
            // property's name, which is all this parser needs.
            let name = rest
                .split_whitespace()
                .last()
                .ok_or_else(|| ParseError("malformed 'property' line".to_string()))?;
            elem.properties.push(name.to_string());
        } else if line == "end_header" {
            break;
        } else {
            return Err(ParseError(format!("unrecognized PLY header line: '{line}'")));
        }
    }
    if !format_seen {
        return Err(ParseError("PLY file has no 'format' line".to_string()));
    }

    let vertex_elem = elements
        .iter()
        .find(|e| e.name == "vertex")
        .ok_or_else(|| ParseError("PLY file declares no 'vertex' element".to_string()))?;
    let x_idx = prop_index(vertex_elem, "x")?;
    let y_idx = prop_index(vertex_elem, "y")?;
    let z_idx = prop_index(vertex_elem, "z")?;
    let color_idx = match (
        vertex_elem.properties.iter().position(|p| p == "red"),
        vertex_elem.properties.iter().position(|p| p == "green"),
        vertex_elem.properties.iter().position(|p| p == "blue"),
    ) {
        (Some(r), Some(g), Some(b)) => Some((r, g, b)),
        _ => None,
    };

    let mut points = Vec::new();
    let mut colors: Option<Vec<[u8; 3]>> = color_idx.map(|_| Vec::new());

    for elem in &elements {
        if elem.name != "vertex" {
            // Not a point-cloud geometry element (e.g. 'face') -- skip
            // its lines by count without interpreting them.
            for _ in 0..elem.count {
                lines.next();
            }
            continue;
        }
        for row in 0..elem.count {
            let line = lines.next().ok_or_else(|| {
                ParseError(format!("PLY file truncated: expected {} vertex rows, got {row}", elem.count))
            })?;
            let fields: Vec<&str> = line.split_whitespace().collect();
            let get = |idx: usize| -> Result<f64, ParseError> {
                fields
                    .get(idx)
                    .ok_or_else(|| ParseError(format!("vertex row has too few fields: '{line}'")))?
                    .parse::<f64>()
                    .map_err(|_| ParseError(format!("vertex field is not a number: '{line}'")))
            };
            points.push([get(x_idx)?, get(y_idx)?, get(z_idx)?]);
            if let (Some((r, g, b)), Some(cols)) = (color_idx, colors.as_mut()) {
                let byte = |idx: usize| -> Result<u8, ParseError> {
                    fields
                        .get(idx)
                        .ok_or_else(|| ParseError(format!("vertex color field missing: '{line}'")))?
                        .parse::<u8>()
                        .map_err(|_| ParseError(format!("vertex color field is not a byte: '{line}'")))
                };
                cols.push([byte(r)?, byte(g)?, byte(b)?]);
            }
        }
    }

    Ok(PointCloud { points, colors })
}

fn prop_index(elem: &ElementDecl, name: &str) -> Result<usize, ParseError> {
    elem.properties
        .iter()
        .position(|p| p == name)
        .ok_or_else(|| ParseError(format!("PLY 'vertex' element has no '{name}' property")))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIMPLE_PLY: &str = "ply\nformat ascii 1.0\ncomment made by test\nelement vertex 3\nproperty float x\nproperty float y\nproperty float z\nend_header\n0.0 0.0 0.0\n1.0 0.0 0.0\n0.0 1.0 0.0\n";

    #[test]
    fn parses_simple_vertex_only_ply() {
        let cloud = parse_ply(SIMPLE_PLY.as_bytes()).unwrap();
        assert_eq!(cloud.points.len(), 3);
        assert_eq!(cloud.points[1], [1.0, 0.0, 0.0]);
        assert!(cloud.colors.is_none());
    }

    #[test]
    fn parses_ply_with_vertex_colors() {
        let text = "ply\nformat ascii 1.0\nelement vertex 2\nproperty float x\nproperty float y\nproperty float z\nproperty uchar red\nproperty uchar green\nproperty uchar blue\nend_header\n0 0 0 255 0 0\n1 1 1 0 255 0\n";
        let cloud = parse_ply(text.as_bytes()).unwrap();
        assert_eq!(cloud.points.len(), 2);
        let colors = cloud.colors.unwrap();
        assert_eq!(colors[0], [255, 0, 0]);
        assert_eq!(colors[1], [0, 255, 0]);
    }

    #[test]
    fn skips_face_element_without_parsing_it_as_geometry() {
        // A meshed export: vertices followed by a face list. Face rows
        // must be skipped by count, never mistaken for vertex data.
        let text = "ply\nformat ascii 1.0\nelement vertex 3\nproperty float x\nproperty float y\nproperty float z\nelement face 1\nproperty list uchar int vertex_indices\nend_header\n0 0 0\n1 0 0\n0 1 0\n3 0 1 2\n";
        let cloud = parse_ply(text.as_bytes()).unwrap();
        assert_eq!(cloud.points.len(), 3);
    }

    #[test]
    fn rejects_binary_ply_with_a_clear_error() {
        let text = "ply\nformat binary_little_endian 1.0\nelement vertex 1\nproperty float x\nend_header\n";
        let err = parse_ply(text.as_bytes()).unwrap_err();
        assert!(err.0.contains("ASCII"), "{}", err.0);
    }

    #[test]
    fn rejects_non_ply_file() {
        let err = parse_ply(b"not a ply file at all").unwrap_err();
        assert!(err.0.contains("not a PLY file"));
    }

    #[test]
    fn rejects_vertex_element_missing_xyz() {
        let text = "ply\nformat ascii 1.0\nelement vertex 1\nproperty float intensity\nend_header\n42\n";
        let err = parse_ply(text.as_bytes()).unwrap_err();
        assert!(err.0.contains("'x'"), "{}", err.0);
    }
}
