//! AkkadianAOL v3.5 — Sovereign Lexer
//! Tokenises .akk source files into an AkkToken stream.
//! ADR-013: Three-tier multilingual sovereignty.
//!   Tier 1 Canonical : Romanised Akkadian  (used in .akk source files)
//!   Tier 1 Semantic  : Arabic              (displayed in DubSar IDE)
//!   Tier 2 Docs      : English             (error messages, tooling)
//!
//! Pipeline: .akk source → Lexer → Vec<SpannedToken> → Parser

use std::fmt;

// ── AOLCat ────────────────────────────────────────────────────────────────────

/// Semantic category of an AkkadianAOL keyword.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AOLCat {
    Flow,
    Entity,
    Query,
    Ingest,
    Policy,
    LangPkg,
}

// ── Span ─────────────────────────────────────────────────────────────────────

/// Source position — used by LSP for diagnostics and hover.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end:   usize,
    pub line:  u32,
    pub col:   u32,
}

impl Span {
    pub fn new(start: usize, end: usize, line: u32, col: u32) -> Self {
        Self { start, end, line, col }
    }
    pub fn dummy() -> Self { Self { start: 0, end: 0, line: 0, col: 0 } }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line + 1, self.col + 1)
    }
}

// ── Token ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum AkkToken {
    // ── Module boundary ───────────────────────────────────────────────────────
    Tablet,
    // ── Entity declarations ───────────────────────────────────────────────────
    Sign, Word, AolMap,
    // ── Graph query ───────────────────────────────────────────────────────────
    Seek, Traverse, Where, Limit, Depth,
    // ── Pipeline ingestion ────────────────────────────────────────────────────
    Ingest, Station, Lane, Format,
    // ── Flow control ──────────────────────────────────────────────────────────
    Shumma, Ul, Adannu, Turru, Sharru,
    // ── Policy / Security ─────────────────────────────────────────────────────
    Policy, Allow, Deny, Firewall, Seal, Verify,
    // ── Language package ──────────────────────────────────────────────────────
    LangPkg, Use, Export,
    // ── Annotations ───────────────────────────────────────────────────────────
    AtCite, AtSeal, AtDomain, AtVersion,
    // ── Literals ─────────────────────────────────────────────────────────────
    IntLit(i64), FloatLit(f64), StrLit(String),
    BoolLit(bool), GlyphLit(char), MeszlLit(u32),
    // ── Identifiers ───────────────────────────────────────────────────────────
    Ident(String),
    // ── Punctuation ───────────────────────────────────────────────────────────
    LBrace, RBrace, LParen, RParen, LBracket, RBracket,
    Colon, ColonColon, Semi, Comma, Dot,
    Eq, EqEq, NotEq, Lt, Gt, LtEq, GtEq,
    Plus, Minus, Star, Slash,
    PipeForward, Arrow, FatArrow,
    // ── Special ───────────────────────────────────────────────────────────────
    Eof,
    Error(String),
}

impl AkkToken {
    pub fn aol_cat(&self) -> Option<AOLCat> {
        match self {
            AkkToken::Shumma | AkkToken::Ul
            | AkkToken::Adannu | AkkToken::Turru
            | AkkToken::Sharru                        => Some(AOLCat::Flow),

            AkkToken::Sign | AkkToken::Word
            | AkkToken::AolMap | AkkToken::Tablet     => Some(AOLCat::Entity),

            AkkToken::Seek | AkkToken::Traverse
            | AkkToken::Where | AkkToken::Limit
            | AkkToken::Depth                         => Some(AOLCat::Query),

            AkkToken::Ingest | AkkToken::Station
            | AkkToken::Lane | AkkToken::Format       => Some(AOLCat::Ingest),

            AkkToken::Policy | AkkToken::Allow
            | AkkToken::Deny | AkkToken::Firewall
            | AkkToken::Seal | AkkToken::Verify       => Some(AOLCat::Policy),

            AkkToken::LangPkg | AkkToken::Use
            | AkkToken::Export                        => Some(AOLCat::LangPkg),

            _                                         => None,
        }
    }

    pub fn canonical_spelling(&self) -> Option<&'static str> {
        match self {
            AkkToken::Tablet    => Some("tablet"),
            AkkToken::Sign      => Some("sign"),
            AkkToken::Word      => Some("word"),
            AkkToken::AolMap    => Some("aol_map"),
            AkkToken::Seek      => Some("seek"),
            AkkToken::Traverse  => Some("traverse"),
            AkkToken::Where     => Some("where"),
            AkkToken::Limit     => Some("limit"),
            AkkToken::Depth     => Some("depth"),
            AkkToken::Ingest    => Some("ingest"),
            AkkToken::Station   => Some("station"),
            AkkToken::Lane      => Some("lane"),
            AkkToken::Format    => Some("format"),
            AkkToken::Shumma    => Some("shumma"),
            AkkToken::Ul        => Some("ul"),
            AkkToken::Adannu    => Some("adannu"),
            AkkToken::Turru     => Some("turru"),
            AkkToken::Sharru    => Some("sharru"),
            AkkToken::Policy    => Some("policy"),
            AkkToken::Allow     => Some("allow"),
            AkkToken::Deny      => Some("deny"),
            AkkToken::Firewall  => Some("firewall"),
            AkkToken::Seal      => Some("seal"),
            AkkToken::Verify    => Some("verify"),
            AkkToken::LangPkg   => Some("lang_pkg"),
            AkkToken::Use       => Some("use"),
            AkkToken::Export    => Some("export"),
            _                   => None,
        }
    }

    pub fn is_block_opener(&self) -> bool {
        matches!(self,
            AkkToken::Tablet | AkkToken::Sign  | AkkToken::Word
            | AkkToken::AolMap | AkkToken::Seek | AkkToken::Ingest
            | AkkToken::Policy | AkkToken::Firewall | AkkToken::LangPkg
        )
    }
}

impl fmt::Display for AkkToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AkkToken::Ident(s)    => write!(f, "Ident({})", s),
            AkkToken::IntLit(n)   => write!(f, "Int({})", n),
            AkkToken::FloatLit(v) => write!(f, "Float({})", v),
            AkkToken::StrLit(s)   => write!(f, "Str({:?})", s),
            AkkToken::BoolLit(b)  => write!(f, "Bool({})", b),
            AkkToken::GlyphLit(c) => write!(f, "Glyph({})", c),
            AkkToken::MeszlLit(n) => write!(f, "MesZL#{}", n),
            AkkToken::Error(e)    => write!(f, "Error({})", e),
            other => {
                if let Some(kw) = other.canonical_spelling() {
                    write!(f, "{}", kw)
                } else {
                    write!(f, "{:?}", other)
                }
            }
        }
    }
}

// ── SpannedToken / LexError ───────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct SpannedToken {
    pub token: AkkToken,
    pub span:  Span,
}

impl SpannedToken {
    pub fn new(token: AkkToken, span: Span) -> Self { Self { token, span } }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LexError {
    pub message: String,
    pub span:    Span,
}

impl LexError {
    pub fn new(message: impl Into<String>, span: Span) -> Self {
        Self { message: message.into(), span }
    }
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[LexError at {}] {}", self.span, self.message)
    }
}

// ── Keyword map ───────────────────────────────────────────────────────────────

fn keyword(s: &str) -> Option<AkkToken> {
    match s {
        "tablet"    => Some(AkkToken::Tablet),
        "sign"      => Some(AkkToken::Sign),
        "word"      => Some(AkkToken::Word),
        "aol_map"   => Some(AkkToken::AolMap),
        "seek"      => Some(AkkToken::Seek),
        "traverse"  => Some(AkkToken::Traverse),
        "where"     => Some(AkkToken::Where),
        "limit"     => Some(AkkToken::Limit),
        "depth"     => Some(AkkToken::Depth),
        "ingest"    => Some(AkkToken::Ingest),
        "station"   => Some(AkkToken::Station),
        "lane"      => Some(AkkToken::Lane),
        "format"    => Some(AkkToken::Format),
        "shumma"    => Some(AkkToken::Shumma),
        "ul"        => Some(AkkToken::Ul),
        "adannu"    => Some(AkkToken::Adannu),
        "turru"     => Some(AkkToken::Turru),
        "sharru"    => Some(AkkToken::Sharru),
        "true"      => Some(AkkToken::BoolLit(true)),
        "false"     => Some(AkkToken::BoolLit(false)),
        "policy"    => Some(AkkToken::Policy),
        "allow"     => Some(AkkToken::Allow),
        "deny"      => Some(AkkToken::Deny),
        "firewall"  => Some(AkkToken::Firewall),
        "seal"      => Some(AkkToken::Seal),
        "verify"    => Some(AkkToken::Verify),
        "lang_pkg"  => Some(AkkToken::LangPkg),
        "use"       => Some(AkkToken::Use),
        "export"    => Some(AkkToken::Export),
        _           => None,
    }
}

// ── Lexer ─────────────────────────────────────────────────────────────────────

pub struct Lexer<'src> {
    src:        &'src str,
    pos:        usize,
    line:       u32,
    line_start: usize,
    pub errors: Vec<LexError>,
}

impl<'src> Lexer<'src> {
    pub fn new(src: &'src str) -> Self {
        Self { src, pos: 0, line: 0, line_start: 0, errors: Vec::new() }
    }

    fn col(&self) -> u32 { (self.pos - self.line_start) as u32 }

    fn current_span_at(&self, start: usize, sl: u32, sc: u32) -> Span {
        Span::new(start, self.pos, sl, sc)
    }

    fn peek(&self) -> Option<char> { self.src[self.pos..].chars().next() }

    fn peek2(&self) -> Option<char> {
        let mut chars = self.src[self.pos..].chars();
        chars.next();
        chars.next()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.src[self.pos..].chars().next()?;
        self.pos += c.len_utf8();
        if c == '\n' { self.line += 1; self.line_start = self.pos; }
        Some(c)
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            while self.peek().map(|c| c.is_whitespace()).unwrap_or(false) {
                self.advance();
            }
            if self.src[self.pos..].starts_with("--") {
                while self.peek().map(|c| c != '\n').unwrap_or(false) { self.advance(); }
                continue;
            }
            if self.src[self.pos..].starts_with("{-") {
                self.advance(); self.advance();
                loop {
                    if self.src[self.pos..].starts_with("-}") {
                        self.advance(); self.advance(); break;
                    }
                    if self.advance().is_none() { break; }
                }
                continue;
            }
            break;
        }
    }

    fn lex_string(&mut self, start: usize, sl: u32, sc: u32) -> SpannedToken {
        let mut s = String::new();
        loop {
            match self.advance() {
                None => {
                    let span = self.current_span_at(start, sl, sc);
                    self.errors.push(LexError::new("unterminated string literal", span));
                    return SpannedToken::new(AkkToken::Error("unterminated string".into()), span);
                }
                Some('"') => break,
                Some('\\') => match self.advance() {
                    Some('n')  => s.push('\n'),
                    Some('t')  => s.push('\t'),
                    Some('\\') => s.push('\\'),
                    Some('"')  => s.push('"'),
                    Some(c)    => s.push(c),
                    None       => break,
                },
                Some(c) => s.push(c),
            }
        }
        SpannedToken::new(AkkToken::StrLit(s), self.current_span_at(start, sl, sc))
    }

    fn lex_number(&mut self, first: char, start: usize, sl: u32, sc: u32) -> SpannedToken {
        let mut s = String::new();
        s.push(first);
        let mut is_float = false;
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() { s.push(c); self.advance(); }
            else if c == '.' && !is_float
                && self.peek2().map(|c2| c2.is_ascii_digit()).unwrap_or(false)
            {
                is_float = true; s.push(c); self.advance();
            } else { break; }
        }
        let span = self.current_span_at(start, sl, sc);
        if is_float {
            match s.parse::<f64>() {
                Ok(v)  => SpannedToken::new(AkkToken::FloatLit(v), span),
                Err(_) => {
                    self.errors.push(LexError::new(format!("invalid float: {}", s), span));
                    SpannedToken::new(AkkToken::Error(s), span)
                }
            }
        } else {
            match s.parse::<i64>() {
                Ok(v)  => SpannedToken::new(AkkToken::IntLit(v), span),
                Err(_) => {
                    self.errors.push(LexError::new(format!("invalid int: {}", s), span));
                    SpannedToken::new(AkkToken::Error(s), span)
                }
            }
        }
    }

    fn lex_meszl(&mut self, start: usize, sl: u32, sc: u32) -> SpannedToken {
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() { s.push(c); self.advance(); } else { break; }
        }
        let span = self.current_span_at(start, sl, sc);
        match s.parse::<u32>() {
            Ok(n)  => SpannedToken::new(AkkToken::MeszlLit(n), span),
            Err(_) => {
                self.errors.push(LexError::new("invalid MesZL number after #", span));
                SpannedToken::new(AkkToken::Error(format!("#{}", s)), span)
            }
        }
    }

    fn lex_ident_or_keyword(&mut self, first: char, start: usize, sl: u32, sc: u32) -> SpannedToken {
        let mut s = String::new();
        s.push(first);
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' { s.push(c); self.advance(); } else { break; }
        }
        let span = self.current_span_at(start, sl, sc);
        let tok = keyword(&s).unwrap_or(AkkToken::Ident(s));
        SpannedToken::new(tok, span)
    }

    fn lex_annotation(&mut self, start: usize, sl: u32, sc: u32) -> SpannedToken {
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' { s.push(c); self.advance(); } else { break; }
        }
        let span = self.current_span_at(start, sl, sc);
        let tok = match s.as_str() {
            "cite"    => AkkToken::AtCite,
            "seal"    => AkkToken::AtSeal,
            "domain"  => AkkToken::AtDomain,
            "version" => AkkToken::AtVersion,
            _ => {
                self.errors.push(LexError::new(format!("unknown annotation @{}", s), span));
                AkkToken::Error(format!("@{}", s))
            }
        };
        SpannedToken::new(tok, span)
    }

    fn next_token(&mut self) -> SpannedToken {
        self.skip_whitespace_and_comments();
        let start = self.pos;
        let sl    = self.line;
        let sc    = self.col();

        let c = match self.advance() {
            None    => return SpannedToken::new(AkkToken::Eof, Span::new(start, start, sl, sc)),
            Some(c) => c,
        };

        match c {
            c if ('\u{12000}'..='\u{1237F}').contains(&c) => {
                SpannedToken::new(AkkToken::GlyphLit(c), self.current_span_at(start, sl, sc))
            }
            '"'  => self.lex_string(start, sl, sc),
            '#'  => self.lex_meszl(start, sl, sc),
            '@'  => self.lex_annotation(start, sl, sc),
            c if c.is_ascii_digit() => self.lex_number(c, start, sl, sc),
            '-' if self.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) => {
                let digit = self.advance().unwrap();
                let mut st = self.lex_number(digit, start, sl, sc);
                match &mut st.token {
                    AkkToken::IntLit(n)   => *n = -*n,
                    AkkToken::FloatLit(v) => *v = -*v,
                    _ => {}
                }
                st
            }
            c if c.is_alphabetic() || c == '_' =>
                self.lex_ident_or_keyword(c, start, sl, sc),
            '{' => SpannedToken::new(AkkToken::LBrace,   Span::new(start, self.pos, sl, sc)),
            '}' => SpannedToken::new(AkkToken::RBrace,   Span::new(start, self.pos, sl, sc)),
            '(' => SpannedToken::new(AkkToken::LParen,   Span::new(start, self.pos, sl, sc)),
            ')' => SpannedToken::new(AkkToken::RParen,   Span::new(start, self.pos, sl, sc)),
            '[' => SpannedToken::new(AkkToken::LBracket, Span::new(start, self.pos, sl, sc)),
            ']' => SpannedToken::new(AkkToken::RBracket, Span::new(start, self.pos, sl, sc)),
            ';' => SpannedToken::new(AkkToken::Semi,     Span::new(start, self.pos, sl, sc)),
            ',' => SpannedToken::new(AkkToken::Comma,    Span::new(start, self.pos, sl, sc)),
            '+' => SpannedToken::new(AkkToken::Plus,     Span::new(start, self.pos, sl, sc)),
            '*' => SpannedToken::new(AkkToken::Star,     Span::new(start, self.pos, sl, sc)),
            ':' => {
                if self.peek() == Some(':') {
                    self.advance();
                    SpannedToken::new(AkkToken::ColonColon, Span::new(start, self.pos, sl, sc))
                } else {
                    SpannedToken::new(AkkToken::Colon, Span::new(start, self.pos, sl, sc))
                }
            }
            '=' => {
                if self.peek() == Some('=') {
                    self.advance();
                    SpannedToken::new(AkkToken::EqEq, Span::new(start, self.pos, sl, sc))
                } else if self.peek() == Some('>') {
                    self.advance();
                    SpannedToken::new(AkkToken::FatArrow, Span::new(start, self.pos, sl, sc))
                } else {
                    SpannedToken::new(AkkToken::Eq, Span::new(start, self.pos, sl, sc))
                }
            }
            '!' => {
                if self.peek() == Some('=') {
                    self.advance();
                    SpannedToken::new(AkkToken::NotEq, Span::new(start, self.pos, sl, sc))
                } else {
                    let span = Span::new(start, self.pos, sl, sc);
                    self.errors.push(LexError::new("unexpected '!' — did you mean '!='?", span));
                    SpannedToken::new(AkkToken::Error("!".into()), span)
                }
            }
            '<' => {
                if self.peek() == Some('=') {
                    self.advance();
                    SpannedToken::new(AkkToken::LtEq, Span::new(start, self.pos, sl, sc))
                } else {
                    SpannedToken::new(AkkToken::Lt, Span::new(start, self.pos, sl, sc))
                }
            }
            '>' => {
                if self.peek() == Some('=') {
                    self.advance();
                    SpannedToken::new(AkkToken::GtEq, Span::new(start, self.pos, sl, sc))
                } else {
                    SpannedToken::new(AkkToken::Gt, Span::new(start, self.pos, sl, sc))
                }
            }
            '-' => {
                if self.peek() == Some('>') {
                    self.advance();
                    SpannedToken::new(AkkToken::Arrow, Span::new(start, self.pos, sl, sc))
                } else {
                    SpannedToken::new(AkkToken::Minus, Span::new(start, self.pos, sl, sc))
                }
            }
            '/' => SpannedToken::new(AkkToken::Slash, Span::new(start, self.pos, sl, sc)),
            '.' => SpannedToken::new(AkkToken::Dot,   Span::new(start, self.pos, sl, sc)),
            '|' => {
                if self.peek() == Some('>') {
                    self.advance();
                    SpannedToken::new(AkkToken::PipeForward, Span::new(start, self.pos, sl, sc))
                } else {
                    let span = Span::new(start, self.pos, sl, sc);
                    self.errors.push(LexError::new("unexpected '|' — did you mean '|>'?", span));
                    SpannedToken::new(AkkToken::Error("|".into()), span)
                }
            }
            c => {
                let span = Span::new(start, self.pos, sl, sc);
                let msg = format!("unexpected character: {:?} (U+{:04X})", c, c as u32);
                self.errors.push(LexError::new(msg.clone(), span));
                SpannedToken::new(AkkToken::Error(msg), span)
            }
        }
    }

    pub fn tokenise(&mut self) -> Vec<SpannedToken> {
        let mut tokens = Vec::new();
        loop {
            let st = self.next_token();
            let is_eof = st.token == AkkToken::Eof;
            tokens.push(st);
            if is_eof { break; }
        }
        tokens
    }

    pub fn lex(src: &'src str) -> (Vec<SpannedToken>, Vec<LexError>) {
        let mut lexer = Lexer::new(src);
        let tokens = lexer.tokenise();
        (tokens, lexer.errors)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn tokens(src: &str) -> Vec<AkkToken> {
        let (toks, _) = Lexer::lex(src);
        toks.into_iter().map(|st| st.token).collect()
    }

    #[test]
    fn test_empty_source() {
        assert_eq!(tokens(""), vec![AkkToken::Eof]);
    }

    #[test]
    fn test_keywords_recognised() {
        let src = "tablet sign word aol_map seek ingest policy allow deny";
        let toks = tokens(src);
        assert_eq!(toks[0], AkkToken::Tablet);
        assert_eq!(toks[4], AkkToken::Seek);
        assert_eq!(toks[5], AkkToken::Ingest);
        assert_eq!(toks[6], AkkToken::Policy);
    }

    #[test]
    fn test_flow_keywords() {
        let toks = tokens("shumma ul adannu turru sharru");
        assert_eq!(toks[0], AkkToken::Shumma);
        assert_eq!(toks[4], AkkToken::Sharru);
    }

    #[test]
    fn test_annotations() {
        let toks = tokens("@cite @seal @domain @version");
        assert_eq!(toks[0], AkkToken::AtCite);
        assert_eq!(toks[1], AkkToken::AtSeal);
        assert_eq!(toks[2], AkkToken::AtDomain);
        assert_eq!(toks[3], AkkToken::AtVersion);
    }

    #[test]
    fn test_meszl_literal() {
        let toks = tokens("#29");
        assert_eq!(toks[0], AkkToken::MeszlLit(29));
    }

    #[test]
    fn test_cuneiform_glyph() {
        let toks = tokens("𒀭");
        assert_eq!(toks[0], AkkToken::GlyphLit('𒀭'));
    }

    #[test]
    fn test_line_comment_skipped() {
        let toks = tokens("tablet -- comment\nsign");
        assert_eq!(toks[0], AkkToken::Tablet);
        assert_eq!(toks[1], AkkToken::Sign);
    }

    #[test]
    fn test_block_comment_skipped() {
        let toks = tokens("tablet {- block -} sign");
        assert_eq!(toks[0], AkkToken::Tablet);
        assert_eq!(toks[1], AkkToken::Sign);
    }

    #[test]
    fn test_aol_cat_flow() {
        assert_eq!(AkkToken::Shumma.aol_cat(), Some(AOLCat::Flow));
    }

    #[test]
    fn test_aol_cat_entity() {
        assert_eq!(AkkToken::Sign.aol_cat(), Some(AOLCat::Entity));
    }

    #[test]
    fn test_span_line_tracking() {
        let (toks, _) = Lexer::lex("tablet\nsign");
        assert_eq!(toks[0].span.line, 0);
        assert_eq!(toks[1].span.line, 1);
    }
}
