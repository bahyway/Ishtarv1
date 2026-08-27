#![forbid(unsafe_code)]

// ============================================================================
// hepta/src/token.rs — Hand-written lexer for the Hepta Spatial DSL
//
// No external deps.  The lexer produces a flat Vec<(Token, usize)> where
// the usize is the 1-based source line number for error reporting.
// ============================================================================

use crate::error::HeptaError;

// ─── Token ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Unit {
    Km,
    M,
    Ms,
    Deg,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Directives
    DirVersion,     // @HEPTA_VERSION
    DirCoordSystem, // @COORDINATE_SYSTEM
    DirProjectId,   // @PROJECT_ID

    // Block keywords
    KwAnchor, // ANCHOR
    KwSector, // SECTOR
    KwNode,   // NODE
    KwBeam,   // BEAM
    KwPath,   // PATH
    KwAttrib, // ATTRIB

    // Compound bracket tokens
    NodeRef { sector_index: u8, label: String }, // [H7-01:MOON]
    Chord(u8),                                   // [CHORD-7/2]
    AttribKey { domain: String, key: String },   // [emergency:level]

    // Literals
    Str(String), // "..."
    Int(i64),
    Float(f64),
    Hex(u128), // 0x...
    Bool(bool),
    Percent(f64),              // 90% → 0.90
    NumberWithUnit(f64, Unit), // 15km | 14ms | 51.42deg
    Version(String),           // "3.5.1" (multi-dot numeric)

    // Arrow
    Arrow, // ->

    // Punctuation
    LBrace,
    RBrace,
    LParen,
    RParen,
    Comma,
    Colon,
    Eq,

    // Generic identifier (keywords, symbols, field names)
    Ident(String),
}

// ─── Lexer ───────────────────────────────────────────────────────────────────

pub struct Lexer<'a> {
    src: &'a [u8],
    pos: usize,
    line: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            src: input.as_bytes(),
            pos: 0,
            line: 1,
        }
    }

    pub fn tokenize(mut self) -> Result<Vec<(Token, usize)>, HeptaError> {
        let mut out = Vec::new();
        loop {
            self.skip_ws_and_comments();
            if self.pos >= self.src.len() {
                break;
            }
            let line = self.line;
            let tok = self.next_token()?;
            out.push((tok, line));
        }
        Ok(out)
    }

    // ── helpers ──────────────────────────────────────────────────────────────

    fn peek(&self) -> Option<u8> {
        self.src.get(self.pos).copied()
    }
    fn peek_at(&self, d: usize) -> Option<u8> {
        self.src.get(self.pos + d).copied()
    }

    fn advance(&mut self) -> Option<u8> {
        let b = self.src.get(self.pos).copied();
        if b == Some(b'\n') {
            self.line += 1;
        }
        self.pos += 1;
        b
    }

    fn skip_ws_and_comments(&mut self) {
        loop {
            match self.peek() {
                Some(b' ' | b'\t' | b'\r' | b'\n') => {
                    self.advance();
                }
                Some(b'#') => {
                    while !matches!(self.peek(), None | Some(b'\n')) {
                        self.advance();
                    }
                }
                _ => break,
            }
        }
    }

    /// True if the next `s.len()` bytes match `s` AND the byte after is not
    /// alphanumeric or underscore (word boundary check).
    fn peek_word(&self, s: &[u8]) -> bool {
        if self.src.len() < self.pos + s.len() {
            return false;
        }
        if &self.src[self.pos..self.pos + s.len()] != s {
            return false;
        }
        let after = self.src.get(self.pos + s.len()).copied();
        !matches!(after, Some(b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_'))
    }

    fn err(&self, msg: impl Into<String>) -> HeptaError {
        HeptaError::ParseError {
            line: self.line,
            message: msg.into(),
        }
    }

    // ── scanning ─────────────────────────────────────────────────────────────

    fn next_token(&mut self) -> Result<Token, HeptaError> {
        match self.peek().unwrap() {
            b'@' => self.scan_directive(),
            b'[' => self.scan_bracket(),
            b'"' => self.scan_string(),
            b'0' if matches!(self.peek_at(1), Some(b'x' | b'X')) => self.scan_hex(),
            b'0'..=b'9' => self.scan_number(false),
            b'-' => self.scan_dash(),
            b'{' => {
                self.advance();
                Ok(Token::LBrace)
            }
            b'}' => {
                self.advance();
                Ok(Token::RBrace)
            }
            b'(' => {
                self.advance();
                Ok(Token::LParen)
            }
            b')' => {
                self.advance();
                Ok(Token::RParen)
            }
            b',' => {
                self.advance();
                Ok(Token::Comma)
            }
            b':' => {
                self.advance();
                Ok(Token::Colon)
            }
            b'=' => {
                self.advance();
                Ok(Token::Eq)
            }
            b'A'..=b'Z' | b'a'..=b'z' | b'_' => self.scan_ident(),
            b => Err(self.err(format!("unexpected character '{}'", b as char))),
        }
    }

    fn scan_directive(&mut self) -> Result<Token, HeptaError> {
        self.advance(); // consume '@'
        let name = self.scan_ident_raw();
        match name.as_str() {
            "HEPTA_VERSION" => Ok(Token::DirVersion),
            "COORDINATE_SYSTEM" => Ok(Token::DirCoordSystem),
            "PROJECT_ID" => Ok(Token::DirProjectId),
            _ => Err(self.err(format!("unknown directive '@{}'", name))),
        }
    }

    fn scan_bracket(&mut self) -> Result<Token, HeptaError> {
        self.advance(); // consume '['
        let mut inner = String::new();
        loop {
            match self.peek() {
                None | Some(b'\n') => return Err(self.err("unclosed '['")),
                Some(b']') => {
                    self.advance();
                    break;
                }
                Some(b) => {
                    inner.push(b as char);
                    self.advance();
                }
            }
        }
        let s = inner.trim();
        if let Some(rest) = s.strip_prefix("H7-") {
            // [H7-NN:LABEL]
            let colon = rest
                .find(':')
                .ok_or_else(|| self.err(format!("invalid node ref '[{}]'", s)))?;
            let idx: u8 = rest[..colon]
                .parse()
                .map_err(|_| self.err(format!("bad sector index in '[{}]'", s)))?;
            let label = rest[colon + 1..].trim().to_string();
            Ok(Token::NodeRef {
                sector_index: idx,
                label,
            })
        } else if let Some(rest) = s.strip_prefix("CHORD-7/") {
            // [CHORD-7/N]
            let k: u8 = rest
                .trim()
                .parse()
                .map_err(|_| self.err(format!("bad chord '[{}]'", s)))?;
            Ok(Token::Chord(k))
        } else if let Some(colon) = s.find(':') {
            // [domain:key]
            let domain = s[..colon].trim().to_string();
            let key = s[colon + 1..].trim().to_string();
            Ok(Token::AttribKey { domain, key })
        } else {
            Ok(Token::Ident(s.to_string()))
        }
    }

    fn scan_string(&mut self) -> Result<Token, HeptaError> {
        self.advance(); // consume opening '"'
        let mut s = String::new();
        loop {
            match self.advance() {
                None => return Err(self.err("unterminated string literal")),
                Some(b'"') => break, // first '"' closes string — no escape sequences
                Some(b) => s.push(b as char),
            }
        }
        Ok(Token::Str(s))
    }

    fn scan_hex(&mut self) -> Result<Token, HeptaError> {
        self.advance(); // '0'
        self.advance(); // 'x'
        let start = self.pos;
        while self.peek().map(|b| b.is_ascii_hexdigit()).unwrap_or(false) {
            self.advance();
        }
        let hex = core::str::from_utf8(&self.src[start..self.pos]).unwrap();
        if hex.is_empty() {
            return Err(self.err("empty hex literal after 0x"));
        }
        u128::from_str_radix(hex, 16)
            .map(Token::Hex)
            .map_err(|_| self.err(format!("hex literal too large: 0x{}", hex)))
    }

    fn scan_number(&mut self, negative: bool) -> Result<Token, HeptaError> {
        let start = self.pos;
        while self.peek().map(|b| b.is_ascii_digit()).unwrap_or(false) {
            self.advance();
        }

        // Decimal part?
        if self.peek() == Some(b'.') && self.peek_at(1).map(|b| b.is_ascii_digit()).unwrap_or(false)
        {
            self.advance(); // '.'
            while self.peek().map(|b| b.is_ascii_digit()).unwrap_or(false) {
                self.advance();
            }
            // Second dot → version string (e.g. "3.5.1")
            if self.peek() == Some(b'.')
                && self.peek_at(1).map(|b| b.is_ascii_digit()).unwrap_or(false)
            {
                self.advance(); // second '.'
                while self.peek().map(|b| b.is_ascii_digit()).unwrap_or(false) {
                    self.advance();
                }
                let s = core::str::from_utf8(&self.src[start..self.pos])
                    .unwrap()
                    .to_string();
                return Ok(Token::Version(s));
            }
            let raw = core::str::from_utf8(&self.src[start..self.pos]).unwrap();
            let mut f: f64 = raw.parse().unwrap();
            if negative {
                f = -f;
            }
            return self.apply_unit_or_percent(f, true);
        }

        // Integer
        let raw = core::str::from_utf8(&self.src[start..self.pos]).unwrap();
        let mut n: i64 = raw.parse().unwrap();
        if negative {
            n = -n;
        }

        // Percent?
        if self.peek() == Some(b'%') {
            self.advance();
            return Ok(Token::Percent(n as f64 / 100.0));
        }
        self.apply_unit_or_percent_i(n)
    }

    fn scan_dash(&mut self) -> Result<Token, HeptaError> {
        self.advance(); // consume '-'
        if self.peek() == Some(b'>') {
            self.advance();
            return Ok(Token::Arrow);
        }
        if self.peek().map(|b| b.is_ascii_digit()).unwrap_or(false) {
            return self.scan_number(true);
        }
        Err(self.err("unexpected '-' (not followed by '>' or digit)"))
    }

    fn scan_ident(&mut self) -> Result<Token, HeptaError> {
        let name = self.scan_ident_raw();
        Ok(match name.as_str() {
            "true" => Token::Bool(true),
            "false" => Token::Bool(false),
            "ANCHOR" => Token::KwAnchor,
            "SECTOR" => Token::KwSector,
            "NODE" => Token::KwNode,
            "BEAM" => Token::KwBeam,
            "PATH" => Token::KwPath,
            "ATTRIB" => Token::KwAttrib,
            _ => Token::Ident(name),
        })
    }

    fn scan_ident_raw(&mut self) -> String {
        let mut s = String::new();
        while self
            .peek()
            .map(|b| b.is_ascii_alphanumeric() || b == b'_')
            .unwrap_or(false)
        {
            s.push(self.advance().unwrap() as char);
        }
        s
    }

    fn apply_unit_or_percent(&mut self, f: f64, _is_float: bool) -> Result<Token, HeptaError> {
        if self.peek() == Some(b'%') {
            self.advance();
            return Ok(Token::Percent(f / 100.0));
        }
        if self.peek_word(b"km") {
            self.pos += 2;
            return Ok(Token::NumberWithUnit(f, Unit::Km));
        }
        if self.peek_word(b"ms") {
            self.pos += 2;
            return Ok(Token::NumberWithUnit(f, Unit::Ms));
        }
        if self.peek_word(b"deg") {
            self.pos += 3;
            return Ok(Token::NumberWithUnit(f, Unit::Deg));
        }
        if self.peek_word(b"m") {
            self.pos += 1;
            return Ok(Token::NumberWithUnit(f, Unit::M));
        }
        Ok(Token::Float(f))
    }

    fn apply_unit_or_percent_i(&mut self, n: i64) -> Result<Token, HeptaError> {
        if self.peek() == Some(b'%') {
            self.advance();
            return Ok(Token::Percent(n as f64 / 100.0));
        }
        if self.peek_word(b"km") {
            self.pos += 2;
            return Ok(Token::NumberWithUnit(n as f64, Unit::Km));
        }
        if self.peek_word(b"ms") {
            self.pos += 2;
            return Ok(Token::NumberWithUnit(n as f64, Unit::Ms));
        }
        if self.peek_word(b"deg") {
            self.pos += 3;
            return Ok(Token::NumberWithUnit(n as f64, Unit::Deg));
        }
        if self.peek_word(b"m") {
            self.pos += 1;
            return Ok(Token::NumberWithUnit(n as f64, Unit::M));
        }
        Ok(Token::Int(n))
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn tok(src: &str) -> Vec<Token> {
        Lexer::new(src)
            .tokenize()
            .unwrap()
            .into_iter()
            .map(|(t, _)| t)
            .collect()
    }

    #[test]
    fn directives() {
        let t = tok("@HEPTA_VERSION 3.5.1\n@COORDINATE_SYSTEM KAKI_7D\n@PROJECT_ID \"Foo\"");
        assert_eq!(t[0], Token::DirVersion);
        assert_eq!(t[1], Token::Version("3.5.1".into()));
        assert_eq!(t[2], Token::DirCoordSystem);
        assert_eq!(t[3], Token::Ident("KAKI_7D".into()));
        assert_eq!(t[4], Token::DirProjectId);
        assert_eq!(t[5], Token::Str("Foo".into()));
    }

    #[test]
    fn node_ref() {
        let t = tok("[H7-01:MOON]");
        assert_eq!(
            t[0],
            Token::NodeRef {
                sector_index: 1,
                label: "MOON".into()
            }
        );
    }

    #[test]
    fn chord() {
        let t = tok("[CHORD-7/2]");
        assert_eq!(t[0], Token::Chord(2));
    }

    #[test]
    fn attrib_key() {
        let t = tok("[emergency:level]");
        assert_eq!(
            t[0],
            Token::AttribKey {
                domain: "emergency".into(),
                key: "level".into()
            }
        );
    }

    #[test]
    fn attrib_key_with_underscore() {
        let t = tok("[enki:last_cleansed]");
        assert_eq!(
            t[0],
            Token::AttribKey {
                domain: "enki".into(),
                key: "last_cleansed".into()
            }
        );
    }

    #[test]
    fn number_units() {
        let t = tok("15km 14ms 51.42deg 500m");
        assert_eq!(t[0], Token::NumberWithUnit(15.0, Unit::Km));
        assert_eq!(t[1], Token::NumberWithUnit(14.0, Unit::Ms));
        assert_eq!(t[2], Token::NumberWithUnit(51.42, Unit::Deg));
        assert_eq!(t[3], Token::NumberWithUnit(500.0, Unit::M));
    }

    #[test]
    fn percent() {
        let t = tok("90% 0.85%");
        assert!((t[0] == Token::Percent(0.90)));
        // 0.85% as float → 0.0085
        if let Token::Percent(p) = t[1] {
            assert!((p - 0.0085).abs() < 1e-9);
        } else {
            panic!("expected Percent");
        }
    }

    #[test]
    fn hex_literal() {
        let t = tok("0x01007D425147484441440000");
        assert!(matches!(t[0], Token::Hex(_)));
    }

    #[test]
    fn bool_literals() {
        let t = tok("true false");
        assert_eq!(t[0], Token::Bool(true));
        assert_eq!(t[1], Token::Bool(false));
    }

    #[test]
    fn arrow() {
        let t = tok("->");
        assert_eq!(t[0], Token::Arrow);
    }

    #[test]
    fn block_keywords() {
        let t = tok("ANCHOR SECTOR NODE BEAM PATH ATTRIB");
        assert_eq!(
            t,
            vec![
                Token::KwAnchor,
                Token::KwSector,
                Token::KwNode,
                Token::KwBeam,
                Token::KwPath,
                Token::KwAttrib,
            ]
        );
    }

    #[test]
    fn skip_comments() {
        let t = tok("# whole line\nANCHOR # inline\nSECTOR");
        assert_eq!(t, vec![Token::KwAnchor, Token::KwSector]);
    }

    #[test]
    fn string_closes_at_first_quote() {
        // The grammar has NO escape sequences — first '"' always ends the string.
        let t = tok(r#""A""B""#); // parsed as "A" then "B" (two tokens)
        assert_eq!(t.len(), 2);
        assert_eq!(t[0], Token::Str("A".into()));
        assert_eq!(t[1], Token::Str("B".into()));
    }

    #[test]
    fn negative_number() {
        let t = tok("-0.55");
        if let Token::Float(f) = t[0] {
            assert!((f + 0.55).abs() < 1e-12);
        } else {
            panic!("expected Float");
        }
    }

    #[test]
    fn radial_sector_call() {
        let t = tok("RADIAL_SECTOR(0, 51.42)");
        assert_eq!(t[0], Token::Ident("RADIAL_SECTOR".into()));
        assert_eq!(t[1], Token::LParen);
        assert_eq!(t[2], Token::Int(0));
        assert_eq!(t[3], Token::Comma);
        assert_eq!(t[4], Token::Float(51.42));
        assert_eq!(t[5], Token::RParen);
    }
}
