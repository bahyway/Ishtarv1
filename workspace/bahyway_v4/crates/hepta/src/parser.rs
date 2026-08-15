#![forbid(unsafe_code)]

// ============================================================================
// hepta/src/parser.rs — Recursive-descent parser for the Hepta Spatial DSL
//
// Input:  &str (.hepta source)
// Output: HeptaFile  (fully typed AST)
//
// Grammar (informal):
//   file       = header block*
//   header     = (@HEPTA_VERSION version)? (@COORDINATE_SYSTEM ident)? (@PROJECT_ID str)?
//   block      = anchor | sector | node
//   anchor     = ANCHOR node_ref '{' anchor_field* '}'
//   sector     = SECTOR node_ref '{' (sector_field | beam | attrib)* '}'
//   node       = NODE   node_ref '{' (node_field   | beam | attrib)* '}'
//   beam       = (BEAM|PATH) chord '->' node_ref '{' beam_prop* '}'
//   attrib     = ATTRIB attrib_key '=' attrib_val
// ============================================================================

use crate::{
    ast::*,
    error::HeptaError,
    kaki::{ChordType, CoordSystem},
    token::{Lexer, Token, Unit},
};

// ─── Public API ──────────────────────────────────────────────────────────────

pub fn parse(source: &str) -> Result<HeptaFile, HeptaError> {
    let tokens = Lexer::new(source).tokenize()?;
    Parser::new(tokens).parse_file()
}

// ─── Parser ──────────────────────────────────────────────────────────────────

struct Parser {
    tokens: Vec<(Token, usize)>,
    pos:    usize,
}

impl Parser {
    fn new(tokens: Vec<(Token, usize)>) -> Self {
        Self { tokens, pos: 0 }
    }

    // ── navigation ───────────────────────────────────────────────────────────

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos).map(|(t, _)| t)
    }

    fn line(&self) -> usize {
        self.tokens.get(self.pos).map(|(_, l)| *l).unwrap_or(0)
    }

    fn advance(&mut self) -> Option<Token> {
        if self.pos < self.tokens.len() {
            let tok = self.tokens[self.pos].0.clone();
            self.pos += 1;
            Some(tok)
        } else {
            None
        }
    }

    fn err(&self, msg: impl Into<String>) -> HeptaError {
        HeptaError::ParseError { line: self.line(), message: msg.into() }
    }

    // ── expect helpers ───────────────────────────────────────────────────────

    fn expect_tok(&mut self, expected: &Token) -> Result<(), HeptaError> {
        match self.advance() {
            Some(ref t) if t == expected => Ok(()),
            Some(t) => Err(self.err(format!("expected {:?}, got {:?}", expected, t))),
            None    => Err(self.err(format!("expected {:?}, got EOF", expected))),
        }
    }

    fn expect_ident(&mut self) -> Result<String, HeptaError> {
        match self.advance() {
            Some(Token::Ident(s)) => Ok(s),
            Some(t) => Err(self.err(format!("expected identifier, got {:?}", t))),
            None    => Err(self.err("expected identifier, got EOF")),
        }
    }

    fn expect_str(&mut self) -> Result<String, HeptaError> {
        match self.advance() {
            Some(Token::Str(s)) => Ok(s),
            Some(t) => Err(self.err(format!("expected string, got {:?}", t))),
            None    => Err(self.err("expected string, got EOF")),
        }
    }

    fn expect_str_or_ident(&mut self) -> Result<String, HeptaError> {
        match self.advance() {
            Some(Token::Str(s))   => Ok(s),
            Some(Token::Ident(s)) => Ok(s),
            Some(t) => Err(self.err(format!("expected string or symbol, got {:?}", t))),
            None    => Err(self.err("expected string or symbol, got EOF")),
        }
    }

    fn expect_hex(&mut self) -> Result<u128, HeptaError> {
        match self.advance() {
            Some(Token::Hex(h)) => Ok(h),
            Some(t) => Err(self.err(format!("expected hex literal, got {:?}", t))),
            None    => Err(self.err("expected hex literal, got EOF")),
        }
    }

    fn expect_float(&mut self) -> Result<f64, HeptaError> {
        match self.advance() {
            Some(Token::Float(f))              => Ok(f),
            Some(Token::Int(n))                => Ok(n as f64),
            Some(Token::NumberWithUnit(f, _))  => Ok(f),
            Some(t) => Err(self.err(format!("expected float, got {:?}", t))),
            None    => Err(self.err("expected float, got EOF")),
        }
    }

    fn expect_node_ref(&mut self) -> Result<NodeRef, HeptaError> {
        match self.advance() {
            Some(Token::NodeRef { sector_index, label }) => Ok(NodeRef::new(sector_index, label)),
            Some(t) => Err(self.err(format!("expected [H7-NN:LABEL], got {:?}", t))),
            None    => Err(self.err("expected node ref, got EOF")),
        }
    }

    /// Consume a trailing comma if present.
    fn skip_comma(&mut self) {
        if matches!(self.peek(), Some(Token::Comma)) {
            self.advance();
        }
    }

    // ─── Top-level file ──────────────────────────────────────────────────────

    fn parse_file(&mut self) -> Result<HeptaFile, HeptaError> {
        let header = self.parse_header()?;
        let mut anchors = Vec::new();
        let mut sectors = Vec::new();
        let mut nodes   = Vec::new();

        loop {
            match self.peek() {
                None                  => break,
                Some(Token::KwAnchor) => { self.advance(); anchors.push(self.parse_anchor()?); }
                Some(Token::KwSector) => { self.advance(); sectors.push(self.parse_sector()?); }
                Some(Token::KwNode)   => { self.advance(); nodes.push(self.parse_node()?); }
                Some(t) => {
                    return Err(self.err(format!("unexpected token at top level: {:?}", t)));
                }
            }
        }

        Ok(HeptaFile { header, anchors, sectors, nodes })
    }

    // ─── Header ──────────────────────────────────────────────────────────────

    fn parse_header(&mut self) -> Result<HeptaHeader, HeptaError> {
        let mut header = HeptaHeader::default();
        loop {
            match self.peek() {
                Some(Token::DirVersion) => {
                    self.advance();
                    header.version = match self.advance() {
                        Some(Token::Version(v)) => v,
                        Some(Token::Ident(v))   => v,
                        Some(Token::Float(f))   => format!("{}", f),
                        Some(t) => return Err(self.err(format!("expected version string, got {:?}", t))),
                        None    => return Err(self.err("expected version string, got EOF")),
                    };
                }
                Some(Token::DirCoordSystem) => {
                    self.advance();
                    let sys = self.expect_ident()?;
                    header.coord_system = CoordSystem::from_str(&sys)?;
                }
                Some(Token::DirProjectId) => {
                    self.advance();
                    header.project_id = Some(self.expect_str()?);
                }
                _ => break,
            }
        }
        Ok(header)
    }

    // ─── ANCHOR block ────────────────────────────────────────────────────────

    fn parse_anchor(&mut self) -> Result<AnchorBlock, HeptaError> {
        let node = self.expect_node_ref()?;
        self.expect_tok(&Token::LBrace)?;

        let mut identity     = None;
        let mut center       = None;
        let mut radius       = None;
        let mut rotation_deg = None;
        let mut kaki_pk_base = None;
        let mut sacred_weight = None;

        loop {
            match self.peek() {
                None => return Err(self.err("unclosed ANCHOR block")),
                Some(Token::RBrace) => { self.advance(); break; }
                Some(Token::Ident(_)) => {
                    let key = self.expect_ident()?;
                    self.expect_tok(&Token::Colon)?;
                    match key.as_str() {
                        "identity"      => identity     = Some(self.expect_str()?),
                        "center"        => center       = Some(self.parse_coord()?),
                        "radius"        => radius       = Some(self.parse_distance()?),
                        "rotation"      => rotation_deg = Some(self.parse_angle()?),
                        "kaki_pk_base"  => kaki_pk_base = Some(self.expect_hex()?),
                        "sacred_weight" => sacred_weight = Some(self.expect_float()?),
                        _               => self.skip_value()?,
                    }
                    self.skip_comma();
                }
                Some(t) => return Err(self.err(format!("unexpected token in ANCHOR body: {:?}", t))),
            }
        }

        Ok(AnchorBlock { node, identity, center, radius, rotation_deg, kaki_pk_base, sacred_weight })
    }

    // ─── SECTOR block ────────────────────────────────────────────────────────

    fn parse_sector(&mut self) -> Result<SectorBlock, HeptaError> {
        let node = self.expect_node_ref()?;
        self.expect_tok(&Token::LBrace)?;

        let mut kaki_pk     = None;
        let mut identity    = None;
        let mut geometry    = None;
        let mut relative_to = None;
        let mut chord_link  = None;
        let mut condition   = None;
        let mut beams       = Vec::new();
        let mut attribs     = Vec::new();

        loop {
            match self.peek() {
                None => return Err(self.err("unclosed SECTOR block")),
                Some(Token::RBrace) => { self.advance(); break; }
                Some(Token::KwBeam | Token::KwPath) => beams.push(self.parse_beam()?),
                Some(Token::KwAttrib) => attribs.push(self.parse_attrib()?),
                Some(Token::Ident(_)) => {
                    let key = self.expect_ident()?;
                    self.expect_tok(&Token::Colon)?;
                    match key.as_str() {
                        "kaki_pk"     => kaki_pk     = Some(self.expect_hex()?),
                        "identity"    => identity    = Some(self.expect_str()?),
                        "geometry"    => geometry    = Some(self.parse_geometry()?),
                        "relative_to" => relative_to = Some(self.expect_node_ref()?),
                        "chord_link"  => chord_link  = Some(self.parse_chord_link()?),
                        "condition"   => condition   = Some(self.expect_str_or_ident()?),
                        _             => self.skip_value()?,
                    }
                    self.skip_comma();
                }
                Some(t) => return Err(self.err(format!("unexpected token in SECTOR body: {:?}", t))),
            }
        }

        Ok(SectorBlock { node, kaki_pk, identity, geometry, relative_to, chord_link, condition, beams, attribs })
    }

    // ─── NODE block ──────────────────────────────────────────────────────────

    fn parse_node(&mut self) -> Result<NodeBlock, HeptaError> {
        let node = self.expect_node_ref()?;
        self.expect_tok(&Token::LBrace)?;

        let mut kaki_pk     = None;
        let mut identity    = None;
        let mut relative_to = None;
        let mut condition   = None;
        let mut beams       = Vec::new();
        let mut attribs     = Vec::new();

        loop {
            match self.peek() {
                None => return Err(self.err("unclosed NODE block")),
                Some(Token::RBrace) => { self.advance(); break; }
                Some(Token::KwBeam | Token::KwPath) => beams.push(self.parse_beam()?),
                Some(Token::KwAttrib) => attribs.push(self.parse_attrib()?),
                Some(Token::Ident(_)) => {
                    let key = self.expect_ident()?;
                    self.expect_tok(&Token::Colon)?;
                    match key.as_str() {
                        "kaki_pk"     => kaki_pk     = Some(self.expect_hex()?),
                        "identity"    => identity    = Some(self.expect_str()?),
                        "relative_to" => relative_to = Some(self.expect_node_ref()?),
                        "condition"   => condition   = Some(self.expect_str_or_ident()?),
                        _             => self.skip_value()?,
                    }
                    self.skip_comma();
                }
                Some(t) => return Err(self.err(format!("unexpected token in NODE body: {:?}", t))),
            }
        }

        Ok(NodeBlock { node, kaki_pk, identity, relative_to, condition, beams, attribs })
    }

    // ─── BEAM / PATH ─────────────────────────────────────────────────────────

    fn parse_beam(&mut self) -> Result<BeamStmt, HeptaError> {
        let is_path = match self.advance() {
            Some(Token::KwPath) => true,
            _                   => false,
        };

        let chord = match self.advance() {
            Some(Token::Chord(k)) => ChordType::from_step(k)?,
            Some(t) => return Err(self.err(format!("expected [CHORD-7/2] or [CHORD-7/3], got {:?}", t))),
            None    => return Err(self.err("expected chord, got EOF")),
        };

        self.expect_tok(&Token::Arrow)?;
        let target = self.expect_node_ref()?;
        self.expect_tok(&Token::LBrace)?;

        let mut props = BeamProperties::default();

        loop {
            match self.peek() {
                None => return Err(self.err("unclosed BEAM block")),
                Some(Token::RBrace) => { self.advance(); break; }
                Some(Token::Ident(_)) => {
                    let key = self.expect_ident()?;
                    self.expect_tok(&Token::Colon)?;
                    match key.as_str() {
                        "latency" => props.latency_ms = Some(self.parse_latency()?),
                        "risk_weight" => props.risk_weight = Some(self.expect_float()?),
                        "capacity"    => props.capacity    = Some(self.expect_ident()?),
                        "security"    => props.security    = Some(self.expect_ident()?),
                        "status"      => props.status      = Some(self.expect_ident()?),
                        "hidden_path_probability" =>
                            props.hidden_path_probability = Some(self.expect_float()?),
                        "type"             => props.beam_type        = Some(self.expect_ident()?),
                        "passability"      => props.passability      = Some(self.expect_ident()?),
                        "historical_status"=> props.historical_status= Some(self.expect_ident()?),
                        _ => self.skip_value()?,
                    }
                    self.skip_comma();
                }
                Some(t) => return Err(self.err(format!("unexpected token in BEAM body: {:?}", t))),
            }
        }

        Ok(BeamStmt { is_path, chord, target, properties: props })
    }

    /// Parse `14ms` or a bare number as milliseconds.
    fn parse_latency(&mut self) -> Result<f64, HeptaError> {
        match self.advance() {
            Some(Token::NumberWithUnit(f, Unit::Ms)) => Ok(f),
            Some(Token::Float(f)) => Ok(f),
            Some(Token::Int(n))   => Ok(n as f64),
            Some(t) => Err(self.err(format!("expected latency (e.g. 14ms), got {:?}", t))),
            None    => Err(self.err("expected latency, got EOF")),
        }
    }

    // ─── ATTRIB ──────────────────────────────────────────────────────────────

    fn parse_attrib(&mut self) -> Result<AttribStmt, HeptaError> {
        self.advance(); // consume KwAttrib

        let (domain, key) = match self.advance() {
            Some(Token::AttribKey { domain, key }) => (domain, key),
            Some(t) => return Err(self.err(format!("expected attrib key [domain:key], got {:?}", t))),
            None    => return Err(self.err("expected attrib key, got EOF")),
        };

        self.expect_tok(&Token::Eq)?;

        let value = match self.advance() {
            Some(Token::Int(n))     => AttribValue::Integer(n),
            Some(Token::Float(f))   => AttribValue::Float(f),
            Some(Token::Bool(b))    => AttribValue::Bool(b),
            Some(Token::Hex(h))     => AttribValue::HexLiteral(h),
            Some(Token::Str(s))     => AttribValue::Text(s),
            Some(Token::Percent(p)) => AttribValue::Percent(p),
            Some(Token::Ident(s))   => AttribValue::Symbol(s),
            Some(t) => return Err(self.err(format!("expected attrib value, got {:?}", t))),
            None    => return Err(self.err("expected attrib value, got EOF")),
        };

        Ok(AttribStmt { domain, key, value })
    }

    // ─── Geometry ────────────────────────────────────────────────────────────

    fn parse_geometry(&mut self) -> Result<SectorGeometry, HeptaError> {
        let kind = self.expect_ident()?;
        self.expect_tok(&Token::LParen)?;
        let start_angle = self.expect_float()?;
        self.expect_tok(&Token::Comma)?;
        let end_angle = self.expect_float()?;
        self.expect_tok(&Token::RParen)?;
        Ok(SectorGeometry { kind, start_angle, end_angle })
    }

    fn parse_chord_link(&mut self) -> Result<ChordLink, HeptaError> {
        self.expect_tok(&Token::LBrace)?;
        let mut target    = None;
        let mut link_type = None;
        loop {
            match self.peek() {
                None => return Err(self.err("unclosed chord_link block")),
                Some(Token::RBrace) => { self.advance(); break; }
                Some(Token::Ident(_)) => {
                    let key = self.expect_ident()?;
                    self.expect_tok(&Token::Colon)?;
                    match key.as_str() {
                        "target" => target    = Some(self.expect_node_ref()?),
                        "type"   => link_type = Some(self.expect_str()?),
                        _        => self.skip_value()?,
                    }
                    self.skip_comma();
                }
                Some(t) => return Err(self.err(format!("unexpected token in chord_link: {:?}", t))),
            }
        }
        let target    = target.ok_or_else(|| self.err("chord_link missing 'target'"))?;
        let link_type = link_type.ok_or_else(|| self.err("chord_link missing 'type'"))?;
        Ok(ChordLink { target, link_type })
    }

    // ─── Coordinate & Distance ───────────────────────────────────────────────

    fn parse_coord(&mut self) -> Result<Coord, HeptaError> {
        self.expect_tok(&Token::LBrace)?;
        let mut lat = None; let mut lon = None;
        let mut x   = None; let mut y   = None;

        loop {
            match self.peek() {
                None => return Err(self.err("unclosed coord '{}'")),
                Some(Token::RBrace) => { self.advance(); break; }
                Some(Token::Ident(_)) => {
                    let field = self.expect_ident()?;
                    self.expect_tok(&Token::Colon)?;
                    let val = self.expect_float()?;
                    match field.as_str() {
                        "lat" => lat = Some(val),
                        "lon" => lon = Some(val),
                        "x"   => x   = Some(val),
                        "y"   => y   = Some(val),
                        _     => {}
                    }
                    self.skip_comma();
                }
                Some(t) => return Err(self.err(format!("unexpected token in coord: {:?}", t))),
            }
        }

        if let (Some(lat), Some(lon)) = (lat, lon) {
            Ok(Coord::LatLon { lat, lon })
        } else if let (Some(x), Some(y)) = (x, y) {
            Ok(Coord::Xy { x, y })
        } else {
            Err(self.err("coord must have lat+lon or x+y"))
        }
    }

    fn parse_distance(&mut self) -> Result<Distance, HeptaError> {
        match self.advance() {
            Some(Token::NumberWithUnit(v, Unit::Km)) => Ok(Distance { value: v, unit: DistanceUnit::Km }),
            Some(Token::NumberWithUnit(v, Unit::M))  => Ok(Distance { value: v, unit: DistanceUnit::M  }),
            Some(Token::Float(f))                    => Ok(Distance { value: f, unit: DistanceUnit::M  }),
            Some(Token::Int(n))                      => Ok(Distance { value: n as f64, unit: DistanceUnit::M }),
            Some(t) => Err(self.err(format!("expected distance (e.g. 15km), got {:?}", t))),
            None    => Err(self.err("expected distance, got EOF")),
        }
    }

    fn parse_angle(&mut self) -> Result<f64, HeptaError> {
        match self.advance() {
            Some(Token::NumberWithUnit(v, Unit::Deg)) => Ok(v),
            Some(Token::Float(f)) => Ok(f),
            Some(Token::Int(n))   => Ok(n as f64),
            Some(t) => Err(self.err(format!("expected angle (e.g. 51.42deg), got {:?}", t))),
            None    => Err(self.err("expected angle, got EOF")),
        }
    }

    // ─── Skip unknown value ───────────────────────────────────────────────────

    fn skip_value(&mut self) -> Result<(), HeptaError> {
        match self.peek().cloned() {
            None => return Err(self.err("expected value, got EOF")),
            Some(Token::LBrace) => {
                self.advance();
                let mut depth = 1usize;
                loop {
                    match self.advance() {
                        None                      => return Err(self.err("unclosed '{' in skipped value")),
                        Some(Token::LBrace)       => depth += 1,
                        Some(Token::RBrace) if depth == 1 => break,
                        Some(Token::RBrace)       => depth -= 1,
                        _                         => {}
                    }
                }
            }
            Some(Token::Ident(_)) => {
                self.advance();
                if matches!(self.peek(), Some(Token::LParen)) {
                    self.advance();
                    let mut depth = 1usize;
                    loop {
                        match self.advance() {
                            None                        => return Err(self.err("unclosed '(' in skipped value")),
                            Some(Token::LParen)         => depth += 1,
                            Some(Token::RParen) if depth == 1 => break,
                            Some(Token::RParen)         => depth -= 1,
                            _                           => {}
                        }
                    }
                }
            }
            _ => { self.advance(); }
        }
        Ok(())
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kaki::*;

    const BAGHDAD: &str = r#"
@HEPTA_VERSION 3.5.1
@COORDINATE_SYSTEM KAKI_7D
@PROJECT_ID "Baghdad_Heptagram_Primary"

ANCHOR [H7-00:SUN] {
    identity: "Green_Zone_Tigris_Anchor",
    center: {lat: 33.3300, lon: 44.3900},
    radius: 15km,
    rotation: 51.42deg,
    kaki_pk_base: 0x00007D425147484441440000
}

SECTOR [H7-01:MOON] {
    identity: "Sadr_City",
    kaki_pk: 0x01007D425147484441440000,
    geometry: RADIAL_SECTOR(0, 51.42),
    relative_to: [H7-00:SUN],

    BEAM [CHORD-7/2] -> [H7-03:VENUS] {
        latency: 14ms,
        risk_weight: 0.35,
        capacity: HIGH_FLOW
    }

    BEAM [CHORD-7/3] -> [H7-05:JUPITER] {
        latency: 22ms,
        risk_weight: 0.20,
        capacity: HIGH_FLOW,
        security: VAULT_L2
    }

    ATTRIB [emergency:level]     = ELEVATED
    ATTRIB [waste:demand]        = 90%
    ATTRIB [waste:containers]    = 612
    ATTRIB [population:density]  = HIGH
    ATTRIB [enki:last_cleansed]  = "2026-03-01"
}

SECTOR [H7-04:MARS] {
    identity: "New_Baghdad_Al_Mushtel",
    kaki_pk: 0x04007D425147484441440000,
    geometry: RADIAL_SECTOR(154.28, 205.71),
    relative_to: [H7-00:SUN],

    BEAM [CHORD-7/2] -> [H7-06:SATURN] {
        latency: 13ms,
        risk_weight: 0.55,
        status: INFRASTRUCTURE_STRESS
    }

    condition: "ELEVATED_RISK"
    ATTRIB [emergency:level]      = HIGH
    ATTRIB [war:risk_index]       = 0.55
}
"#;

    #[test]
    fn parses_header() {
        let f = parse(BAGHDAD).unwrap();
        assert_eq!(f.header.version, "3.5.1");
        assert_eq!(f.header.coord_system, CoordSystem::Kaki7d);
        assert_eq!(f.header.project_id.as_deref(), Some("Baghdad_Heptagram_Primary"));
    }

    #[test]
    fn parses_anchor() {
        let f = parse(BAGHDAD).unwrap();
        assert_eq!(f.anchors.len(), 1);
        let a = &f.anchors[0];
        assert_eq!(a.node.label, "SUN");
        assert_eq!(a.node.sector_index, 0);
        assert_eq!(a.node.planet, PlanetNode::Sun);
        assert_eq!(a.identity.as_deref(), Some("Green_Zone_Tigris_Anchor"));
        assert!(matches!(a.center, Some(Coord::LatLon { lat, lon }) if (lat - 33.33).abs() < 1e-6 && (lon - 44.39).abs() < 1e-6));
        assert!(matches!(&a.radius, Some(d) if d.unit == DistanceUnit::Km && (d.value - 15.0).abs() < 1e-9));
        assert!(matches!(a.rotation_deg, Some(r) if (r - 51.42).abs() < 1e-9));
        assert!(a.kaki_pk_base.is_some());
    }

    #[test]
    fn parses_sector_moon() {
        let f = parse(BAGHDAD).unwrap();
        assert_eq!(f.sectors.len(), 2);
        let moon = &f.sectors[0];
        assert_eq!(moon.node.label, "MOON");
        assert_eq!(moon.node.planet, PlanetNode::Moon);
        assert_eq!(moon.identity.as_deref(), Some("Sadr_City"));
        assert!(moon.geometry.is_some());
        assert_eq!(moon.geometry.as_ref().unwrap().kind, "RADIAL_SECTOR");
        assert!((moon.geometry.as_ref().unwrap().start_angle - 0.0).abs() < 1e-9);
        assert!((moon.geometry.as_ref().unwrap().end_angle - 51.42).abs() < 1e-9);
        assert_eq!(moon.relative_to.as_ref().unwrap().label, "SUN");
    }

    #[test]
    fn parses_beams() {
        let f = parse(BAGHDAD).unwrap();
        let moon = &f.sectors[0];
        assert_eq!(moon.beams.len(), 2);

        let b0 = &moon.beams[0];
        assert!(!b0.is_path);
        assert_eq!(b0.chord, ChordType::Chord72);
        assert_eq!(b0.target.label, "VENUS");
        assert!((b0.properties.latency_ms.unwrap() - 14.0).abs() < 1e-9);
        assert!((b0.properties.risk_weight.unwrap() - 0.35).abs() < 1e-9);
        assert_eq!(b0.properties.capacity.as_deref(), Some("HIGH_FLOW"));

        let b1 = &moon.beams[1];
        assert_eq!(b1.chord, ChordType::Chord73);
        assert_eq!(b1.target.label, "JUPITER");
        assert_eq!(b1.properties.security.as_deref(), Some("VAULT_L2"));
    }

    #[test]
    fn parses_attribs() {
        let f = parse(BAGHDAD).unwrap();
        let moon = &f.sectors[0];
        assert_eq!(moon.attribs.len(), 5);

        assert_eq!(moon.attribs[0].domain, "emergency");
        assert_eq!(moon.attribs[0].key, "level");
        assert_eq!(moon.attribs[0].value, AttribValue::Symbol("ELEVATED".into()));

        assert_eq!(moon.attribs[1].domain, "waste");
        assert_eq!(moon.attribs[1].key, "demand");
        if let AttribValue::Percent(p) = moon.attribs[1].value {
            assert!((p - 0.90).abs() < 1e-9);
        } else { panic!("expected Percent"); }

        assert_eq!(moon.attribs[2].value, AttribValue::Integer(612));

        assert_eq!(moon.attribs[4].value, AttribValue::Text("2026-03-01".into()));
    }

    #[test]
    fn condition_after_beams() {
        // MARS sector has condition AFTER the beam block
        let f = parse(BAGHDAD).unwrap();
        let mars = &f.sectors[1];
        assert_eq!(mars.condition.as_deref(), Some("ELEVATED_RISK"));
        assert_eq!(mars.beams.len(), 1);
        assert_eq!(mars.attribs.len(), 2);
    }

    #[test]
    fn beam_count() {
        let f = parse(BAGHDAD).unwrap();
        assert_eq!(f.beam_count(), 3); // 2 in MOON + 1 in MARS
    }

    #[test]
    fn attrib_count() {
        let f = parse(BAGHDAD).unwrap();
        assert_eq!(f.attrib_count(), 7); // 5 in MOON + 2 in MARS
    }

    #[test]
    fn float_attrib() {
        let f = parse(BAGHDAD).unwrap();
        let mars = &f.sectors[1];
        let risk = &mars.attribs[1];
        assert_eq!(risk.key, "risk_index");
        if let AttribValue::Float(v) = risk.value {
            assert!((v - 0.55).abs() < 1e-9);
        } else { panic!("expected Float"); }
    }

    #[test]
    fn sector_by_label() {
        let f = parse(BAGHDAD).unwrap();
        assert!(f.sector_by_label("MOON").is_some());
        assert!(f.sector_by_label("MARS").is_some());
        assert!(f.sector_by_label("JUPITER").is_none());
    }

    #[test]
    fn anchor_by_label() {
        let f = parse(BAGHDAD).unwrap();
        assert!(f.anchor_by_label("SUN").is_some());
        assert!(f.anchor_by_label("MOON").is_none());
    }

    #[test]
    fn empty_file() {
        let f = parse("").unwrap();
        assert!(f.anchors.is_empty());
        assert!(f.sectors.is_empty());
        assert!(f.nodes.is_empty());
    }

    #[test]
    fn path_keyword() {
        let src = r#"SECTOR [H7-02:MERCURY] {
            PATH [CHORD-7/2] -> [H7-01:MOON] {
                latency: 5ms
            }
        }"#;
        let f = parse(src).unwrap();
        assert!(f.sectors[0].beams[0].is_path);
        assert_eq!(f.sectors[0].beams[0].chord, ChordType::Chord72);
    }

    #[test]
    fn bool_attrib() {
        let src = r#"SECTOR [H7-02:MERCURY] {
            ATTRIB [heritage:protected] = true
        }"#;
        let f = parse(src).unwrap();
        assert_eq!(f.sectors[0].attribs[0].value, AttribValue::Bool(true));
    }

    #[test]
    fn hex_attrib() {
        let src = "SECTOR [H7-00:SUN] {\n    ATTRIB [kaki:pk] = 0xFF00\n}";
        let f = parse(src).unwrap();
        assert_eq!(f.sectors[0].attribs[0].value, AttribValue::HexLiteral(0xFF00));
    }
}
