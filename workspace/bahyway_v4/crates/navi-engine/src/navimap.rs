//! NaviMap — sovereign spatial map format (replaces v3.5.1 hepta::HeptaMap).
//!
//! Text format (.navimap files):
//!   NODE <id> <lat> <lon> <alt> <sector_0-6> <tribe_hex>
//!   BEAM <from> <to> <distance>
//!   BEAM <from> <to> <distance> ONE_WAY
//!
//! Sector codes: 0=Centre 1=North 2=NorthEast 3=SouthEast 4=South 5=SouthWest 6=NorthWest

use crate::error::{NaviError, NaviResult};

/// A single waypoint node in the NaviMap.
#[derive(Debug, Clone)]
pub struct MapNode {
    pub id:        u32,
    pub lat:       f32,
    pub lon:       f32,
    pub alt:       f32,
    /// Sector index 0–6 (Centre=0, then clockwise: N=1 NE=2 SE=3 S=4 SW=5 NW=6).
    pub sector:    u8,
    /// Owning tribe (u16 sovereign identifier).
    pub tribe:     u16,
}

/// A directed connection between two MapNodes.
#[derive(Debug, Clone)]
pub struct MapBeam {
    pub from:     u32,
    pub to:       u32,
    /// Geometric distance in metres.
    pub distance: f32,
    /// If true, only the from→to direction is valid.
    pub one_way:  bool,
}

/// A complete navigable map — nodes + beams.
#[derive(Debug, Default)]
pub struct NaviMap {
    pub nodes: Vec<MapNode>,
    pub beams: Vec<MapBeam>,
}

impl NaviMap {
    pub fn new() -> Self { Self::default() }

    pub fn add_node(&mut self, node: MapNode)  { self.nodes.push(node); }
    pub fn add_beam(&mut self, beam: MapBeam)  { self.beams.push(beam); }

    pub fn node_count(&self) -> usize { self.nodes.len() }
    pub fn beam_count(&self) -> usize { self.beams.len() }

    pub fn find_node(&self, id: u32) -> Option<&MapNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    /// Parse a NaviMap from its text representation.
    ///
    /// Lines beginning with `#` are comments and are skipped.
    pub fn parse(src: &str) -> NaviResult<Self> {
        let mut map = NaviMap::new();
        for (line_no, raw) in src.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') { continue; }
            let parts: Vec<&str> = line.split_whitespace().collect();
            match parts.first().copied() {
                Some("NODE") => {
                    if parts.len() < 7 {
                        return Err(NaviError::MapParseError(
                            format!("line {}: NODE needs 6 fields", line_no + 1)
                        ));
                    }
                    let id    = parts[1].parse::<u32>().map_err(|_| NaviError::MapParseError(format!("line {}: bad node id", line_no + 1)))?;
                    let lat   = parts[2].parse::<f32>().map_err(|_| NaviError::MapParseError(format!("line {}: bad lat", line_no + 1)))?;
                    let lon   = parts[3].parse::<f32>().map_err(|_| NaviError::MapParseError(format!("line {}: bad lon", line_no + 1)))?;
                    let alt   = parts[4].parse::<f32>().map_err(|_| NaviError::MapParseError(format!("line {}: bad alt", line_no + 1)))?;
                    let sector= parts[5].parse::<u8>().map_err(|_| NaviError::MapParseError(format!("line {}: bad sector", line_no + 1)))?;
                    let tribe = u16::from_str_radix(parts[6].trim_start_matches("0x"), 16)
                        .map_err(|_| NaviError::MapParseError(format!("line {}: bad tribe hex", line_no + 1)))?;
                    if sector > 6 {
                        return Err(NaviError::MapParseError(format!("line {}: sector must be 0-6", line_no + 1)));
                    }
                    map.add_node(MapNode { id, lat, lon, alt, sector, tribe });
                }
                Some("BEAM") => {
                    if parts.len() < 4 {
                        return Err(NaviError::MapParseError(
                            format!("line {}: BEAM needs from, to, distance", line_no + 1)
                        ));
                    }
                    let from     = parts[1].parse::<u32>().map_err(|_| NaviError::MapParseError(format!("line {}: bad beam from", line_no + 1)))?;
                    let to       = parts[2].parse::<u32>().map_err(|_| NaviError::MapParseError(format!("line {}: bad beam to", line_no + 1)))?;
                    let distance = parts[3].parse::<f32>().map_err(|_| NaviError::MapParseError(format!("line {}: bad distance", line_no + 1)))?;
                    let one_way  = parts.get(4).copied() == Some("ONE_WAY");
                    map.add_beam(MapBeam { from, to, distance, one_way });
                }
                Some(tok) => {
                    return Err(NaviError::MapParseError(
                        format!("line {}: unknown token '{tok}'", line_no + 1)
                    ));
                }
                None => {}
            }
        }
        Ok(map)
    }
}

// ── Built-in test fixtures ────────────────────────────────────────────────────

/// Minimal 7-node map — one node per heptagram sector.
pub fn seven_node_map() -> NaviMap {
    let mut m = NaviMap::new();
    // Centre + 6 outer sectors; tribe 0x0001 = Najaf sovereign
    let nodes = [
        (1, 31.9900, 44.3200, 0.0, 0u8, 0x0001u16), // Centre
        (2, 32.0000, 44.3200, 0.0, 1,   0x0001),     // North
        (3, 31.9950, 44.3350, 0.0, 2,   0x0001),     // NorthEast
        (4, 31.9850, 44.3350, 0.0, 3,   0x0001),     // SouthEast
        (5, 31.9800, 44.3200, 0.0, 4,   0x0001),     // South
        (6, 31.9850, 44.3050, 0.0, 5,   0x0001),     // SouthWest
        (7, 31.9950, 44.3050, 0.0, 6,   0x0001),     // NorthWest
    ];
    for (id, lat, lon, alt, sector, tribe) in nodes {
        m.add_node(MapNode { id, lat, lon, alt, sector, tribe });
    }
    // Spoke BEAMs: Centre → each outer (bidirectional)
    for outer in 2u32..=7 {
        m.add_beam(MapBeam { from: 1, to: outer, distance: 500.0, one_way: false });
    }
    // Rim BEAMs: ring between adjacent outer nodes
    let rim = [(2,3),(3,4),(4,5),(5,6),(6,7),(7,2)];
    for (a, b) in rim {
        m.add_beam(MapBeam { from: a, to: b, distance: 400.0, one_way: false });
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seven_node_map_has_correct_counts() {
        let m = seven_node_map();
        assert_eq!(m.node_count(), 7);
        assert_eq!(m.beam_count(), 12); // 6 spokes + 6 rim
    }

    #[test]
    fn find_node_by_id() {
        let m = seven_node_map();
        assert!(m.find_node(1).is_some());
        assert!(m.find_node(99).is_none());
    }

    #[test]
    fn centre_node_has_sector_0() {
        let m = seven_node_map();
        assert_eq!(m.find_node(1).unwrap().sector, 0);
    }

    #[test]
    fn parse_minimal_map() {
        let src = "# test map\nNODE 1 31.99 44.32 0.0 0 0x0001\nNODE 2 32.00 44.32 0.0 1 0x0001\nBEAM 1 2 500.0";
        let m = NaviMap::parse(src).expect("parse failed");
        assert_eq!(m.node_count(), 2);
        assert_eq!(m.beam_count(), 1);
    }

    #[test]
    fn parse_one_way_beam() {
        let src = "NODE 1 31.99 44.32 0.0 0 0x0001\nNODE 2 32.00 44.32 0.0 1 0x0001\nBEAM 1 2 500.0 ONE_WAY";
        let m = NaviMap::parse(src).unwrap();
        assert!(m.beams[0].one_way);
    }

    #[test]
    fn parse_bidirectional_beam() {
        let src = "NODE 1 31.99 44.32 0.0 0 0x0001\nNODE 2 32.00 44.32 0.0 1 0x0001\nBEAM 1 2 500.0";
        let m = NaviMap::parse(src).unwrap();
        assert!(!m.beams[0].one_way);
    }

    #[test]
    fn parse_invalid_sector_errors() {
        let src = "NODE 1 31.99 44.32 0.0 9 0x0001";
        assert!(NaviMap::parse(src).is_err());
    }

    #[test]
    fn parse_bad_lat_errors() {
        let src = "NODE 1 BADLAT 44.32 0.0 0 0x0001";
        assert!(NaviMap::parse(src).is_err());
    }

    #[test]
    fn parse_unknown_token_errors() {
        let src = "EDGE 1 2 100.0";
        assert!(NaviMap::parse(src).is_err());
    }

    #[test]
    fn parse_comment_lines_skipped() {
        let src = "# header\n# another comment\nNODE 1 31.99 44.32 0.0 0 0x0001";
        let m = NaviMap::parse(src).unwrap();
        assert_eq!(m.node_count(), 1);
    }

    #[test]
    fn parse_empty_input_ok() {
        let m = NaviMap::parse("").unwrap();
        assert_eq!(m.node_count(), 0);
        assert_eq!(m.beam_count(), 0);
    }
}
