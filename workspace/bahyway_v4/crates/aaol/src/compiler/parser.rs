//! AkkadianAOL v3.5 — Sovereign Parser
//! AkkToken stream → AkkAST

#![allow(dead_code)]

use crate::compiler::lexer::{AkkToken, LexError, Lexer, Span, SpannedToken};

// ── AST Node types ────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct AkkFile {
    pub tablets: Vec<AkkTablet>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct AkkTablet {
    pub name: String,
    pub annotations: Vec<AkkAnnotation>,
    pub decls: Vec<AkkDecl>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum AkkDecl {
    Sign(AkkSignDecl),
    Word(AkkWordDecl),
    AolMap(AkkAolMapDecl),
    Seek(AkkSeekDecl),
    Ingest(AkkIngestDecl),
    Policy(AkkPolicyDecl),
}

#[derive(Debug, Clone)]
pub struct AkkSignDecl {
    pub name: String,
    pub annotations: Vec<AkkAnnotation>,
    pub fields: Vec<AkkField>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct AkkWordDecl {
    pub name: String,
    pub annotations: Vec<AkkAnnotation>,
    pub fields: Vec<AkkField>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct AkkAolMapDecl {
    pub mappings: Vec<AkkMapping>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct AkkMapping {
    pub fields: Vec<AkkField>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct AkkSeekDecl {
    pub fields: Vec<AkkField>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct AkkIngestDecl {
    pub fields: Vec<AkkField>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct AkkPolicyDecl {
    pub name: String,
    pub rules: Vec<AkkRule>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct AkkRule {
    pub kind: RuleKind,
    pub fields: Vec<AkkField>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleKind {
    Allow,
    Deny,
}

#[derive(Debug, Clone)]
pub struct AkkField {
    pub key: String,
    pub value: AkkExpr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct AkkAnnotation {
    pub kind: AnnotationKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum AnnotationKind {
    Cite(String),
    Seal,
    Domain(String),
    Version(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AkkExpr {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Glyph(char),
    MeszlRef(u32),
    Ident(String),
    Path(Vec<String>),
}

// ── Parse Error ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
}

impl ParseError {
    fn new(msg: impl Into<String>, span: Span) -> Self {
        Self {
            message: msg.into(),
            span,
        }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[ParseError at {}] {}", self.span, self.message)
    }
}

// ── Parser ────────────────────────────────────────────────────────────────────

pub struct Parser {
    tokens: Vec<SpannedToken>,
    pos: usize,
    pub errors: Vec<ParseError>,
}

impl Parser {
    pub fn new(tokens: Vec<SpannedToken>) -> Self {
        Self {
            tokens,
            pos: 0,
            errors: Vec::new(),
        }
    }

    fn peek(&self) -> &AkkToken {
        self.tokens
            .get(self.pos)
            .map(|st| &st.token)
            .unwrap_or(&AkkToken::Eof)
    }

    fn peek_span(&self) -> Span {
        self.tokens
            .get(self.pos)
            .map(|st| st.span)
            .unwrap_or(Span::dummy())
    }

    fn advance(&mut self) -> &SpannedToken {
        let t = &self.tokens[self.pos.min(self.tokens.len() - 1)];
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
        t
    }

    fn expect(&mut self, expected: &AkkToken) -> Option<Span> {
        if self.peek() == expected {
            let span = self.peek_span();
            self.advance();
            Some(span)
        } else {
            let span = self.peek_span();
            self.errors.push(ParseError::new(
                format!("expected {:?}, found {:?}", expected, self.peek()),
                span,
            ));
            None
        }
    }

    fn expect_ident(&mut self) -> Option<(String, Span)> {
        let span = self.peek_span();
        match self.peek().clone() {
            AkkToken::Ident(name) => {
                self.advance();
                Some((name, span))
            }
            other => {
                self.errors.push(ParseError::new(
                    format!("expected identifier, found {:?}", other),
                    span,
                ));
                None
            }
        }
    }

    fn skip_semis(&mut self) {
        while self.peek() == &AkkToken::Semi {
            self.advance();
        }
    }

    fn recover_to_rbrace(&mut self) {
        loop {
            match self.peek() {
                AkkToken::RBrace | AkkToken::Eof => break,
                _ => {
                    self.advance();
                }
            }
        }
    }

    // ── Annotation parsing ────────────────────────────────────────────────────

    fn parse_annotation(&mut self) -> Option<AkkAnnotation> {
        let start = self.peek_span();
        match self.peek().clone() {
            AkkToken::AtCite => {
                self.advance();
                let (name, _) = self.expect_ident()?;
                Some(AkkAnnotation {
                    kind: AnnotationKind::Cite(name),
                    span: start,
                })
            }
            AkkToken::AtSeal => {
                self.advance();
                Some(AkkAnnotation {
                    kind: AnnotationKind::Seal,
                    span: start,
                })
            }
            AkkToken::AtDomain => {
                self.advance();
                let span = self.peek_span();
                match self.peek().clone() {
                    AkkToken::StrLit(s) => {
                        self.advance();
                        Some(AkkAnnotation {
                            kind: AnnotationKind::Domain(s),
                            span: start,
                        })
                    }
                    _ => {
                        self.errors
                            .push(ParseError::new("@domain requires a string literal", span));
                        None
                    }
                }
            }
            AkkToken::AtVersion => {
                self.advance();
                let span = self.peek_span();
                match self.peek().clone() {
                    AkkToken::StrLit(s) => {
                        self.advance();
                        Some(AkkAnnotation {
                            kind: AnnotationKind::Version(s),
                            span: start,
                        })
                    }
                    _ => {
                        self.errors
                            .push(ParseError::new("@version requires a string literal", span));
                        None
                    }
                }
            }
            _ => None,
        }
    }

    fn is_annotation_token(&self) -> bool {
        matches!(
            self.peek(),
            AkkToken::AtCite | AkkToken::AtSeal | AkkToken::AtDomain | AkkToken::AtVersion
        )
    }

    fn parse_annotations(&mut self) -> Vec<AkkAnnotation> {
        let mut anns = Vec::new();
        while self.is_annotation_token() {
            if let Some(ann) = self.parse_annotation() {
                anns.push(ann);
            }
        }
        anns
    }

    // ── Expression parsing ────────────────────────────────────────────────────

    fn parse_expr(&mut self) -> Option<AkkExpr> {
        let span = self.peek_span();
        match self.peek().clone() {
            AkkToken::IntLit(n) => {
                self.advance();
                Some(AkkExpr::Int(n))
            }
            AkkToken::FloatLit(v) => {
                self.advance();
                Some(AkkExpr::Float(v))
            }
            AkkToken::StrLit(s) => {
                self.advance();
                Some(AkkExpr::Str(s))
            }
            AkkToken::BoolLit(b) => {
                self.advance();
                Some(AkkExpr::Bool(b))
            }
            AkkToken::GlyphLit(c) => {
                self.advance();
                Some(AkkExpr::Glyph(c))
            }
            AkkToken::MeszlLit(n) => {
                self.advance();
                Some(AkkExpr::MeszlRef(n))
            }
            AkkToken::Ident(name) => {
                self.advance();
                if self.peek() == &AkkToken::Dot {
                    let mut parts = vec![name];
                    while self.peek() == &AkkToken::Dot {
                        self.advance();
                        match self.peek().clone() {
                            AkkToken::Ident(part) => {
                                self.advance();
                                parts.push(part);
                            }
                            _ => break,
                        }
                    }
                    Some(AkkExpr::Path(parts))
                } else {
                    Some(AkkExpr::Ident(name))
                }
            }
            other => {
                self.errors.push(ParseError::new(
                    format!("expected expression, found {:?}", other),
                    span,
                ));
                None
            }
        }
    }

    // ── Field parsing ─────────────────────────────────────────────────────────

    fn parse_field(&mut self) -> Option<AkkField> {
        let start = self.peek_span();
        let (key, _) = self.expect_ident()?;
        self.expect(&AkkToken::Eq)?;
        let value = self.parse_expr()?;
        self.skip_semis();
        Some(AkkField {
            key,
            value,
            span: start,
        })
    }

    fn parse_fields(&mut self) -> Vec<AkkField> {
        let mut fields = Vec::new();
        loop {
            match self.peek() {
                AkkToken::RBrace | AkkToken::Eof => break,
                AkkToken::AtCite | AkkToken::AtSeal | AkkToken::AtDomain | AkkToken::AtVersion => {
                    self.parse_annotation();
                }
                AkkToken::Semi => {
                    self.advance();
                }
                _ => match self.parse_field() {
                    Some(f) => fields.push(f),
                    None => {
                        self.recover_to_rbrace();
                        break;
                    }
                },
            }
        }
        fields
    }

    // ── Declaration parsers ───────────────────────────────────────────────────

    fn parse_sign(&mut self) -> Option<AkkDecl> {
        let start = self.peek_span();
        self.advance();
        let (name, _) = self.expect_ident()?;
        self.expect(&AkkToken::LBrace)?;
        // Annotations may appear before or after fields — collect both in one pass.
        let mut annotations = Vec::new();
        let mut fields = Vec::new();
        loop {
            match self.peek() {
                AkkToken::RBrace | AkkToken::Eof => break,
                AkkToken::AtCite | AkkToken::AtSeal | AkkToken::AtDomain | AkkToken::AtVersion => {
                    if let Some(ann) = self.parse_annotation() {
                        annotations.push(ann);
                    }
                }
                AkkToken::Semi => {
                    self.advance();
                }
                _ => match self.parse_field() {
                    Some(f) => fields.push(f),
                    None => {
                        self.recover_to_rbrace();
                        break;
                    }
                },
            }
        }
        self.expect(&AkkToken::RBrace)?;
        Some(AkkDecl::Sign(AkkSignDecl {
            name,
            annotations,
            fields,
            span: start,
        }))
    }

    fn parse_word(&mut self) -> Option<AkkDecl> {
        let start = self.peek_span();
        self.advance();
        let (name, _) = self.expect_ident()?;
        self.expect(&AkkToken::LBrace)?;
        let mut annotations = Vec::new();
        let mut fields = Vec::new();
        loop {
            match self.peek() {
                AkkToken::RBrace | AkkToken::Eof => break,
                AkkToken::AtCite | AkkToken::AtSeal | AkkToken::AtDomain | AkkToken::AtVersion => {
                    if let Some(ann) = self.parse_annotation() {
                        annotations.push(ann);
                    }
                }
                AkkToken::Semi => {
                    self.advance();
                }
                _ => match self.parse_field() {
                    Some(f) => fields.push(f),
                    None => {
                        self.recover_to_rbrace();
                        break;
                    }
                },
            }
        }
        self.expect(&AkkToken::RBrace)?;
        Some(AkkDecl::Word(AkkWordDecl {
            name,
            annotations,
            fields,
            span: start,
        }))
    }

    fn parse_aol_map(&mut self) -> Option<AkkDecl> {
        let start = self.peek_span();
        self.advance();
        self.expect(&AkkToken::LBrace)?;
        let mut mappings = Vec::new();
        while self.peek() != &AkkToken::RBrace && self.peek() != &AkkToken::Eof {
            let map_start = self.peek_span();
            let fields = self.parse_fields();
            if !fields.is_empty() {
                mappings.push(AkkMapping {
                    fields,
                    span: map_start,
                });
            } else {
                break;
            }
        }
        self.expect(&AkkToken::RBrace)?;
        Some(AkkDecl::AolMap(AkkAolMapDecl {
            mappings,
            span: start,
        }))
    }

    fn parse_seek(&mut self) -> Option<AkkDecl> {
        let start = self.peek_span();
        self.advance();
        self.expect(&AkkToken::LBrace)?;
        let fields = self.parse_fields();
        self.expect(&AkkToken::RBrace)?;
        Some(AkkDecl::Seek(AkkSeekDecl {
            fields,
            span: start,
        }))
    }

    fn parse_ingest(&mut self) -> Option<AkkDecl> {
        let start = self.peek_span();
        self.advance();
        self.expect(&AkkToken::LBrace)?;
        let fields = self.parse_fields();
        self.expect(&AkkToken::RBrace)?;
        Some(AkkDecl::Ingest(AkkIngestDecl {
            fields,
            span: start,
        }))
    }

    fn parse_policy(&mut self) -> Option<AkkDecl> {
        let start = self.peek_span();
        self.advance();
        let (name, _) = self.expect_ident()?;
        self.expect(&AkkToken::LBrace)?;
        let mut rules = Vec::new();
        loop {
            match self.peek() {
                AkkToken::RBrace | AkkToken::Eof => break,
                AkkToken::Allow | AkkToken::Deny => {
                    let rule_start = self.peek_span();
                    let kind = if self.peek() == &AkkToken::Allow {
                        RuleKind::Allow
                    } else {
                        RuleKind::Deny
                    };
                    self.advance();
                    // Parse fields until the next rule keyword or end of policy block.
                    let mut fields = Vec::new();
                    loop {
                        match self.peek() {
                            AkkToken::RBrace | AkkToken::Eof | AkkToken::Allow | AkkToken::Deny => {
                                break
                            }
                            AkkToken::Semi => {
                                self.advance();
                            }
                            _ => match self.parse_field() {
                                Some(f) => fields.push(f),
                                None => {
                                    self.recover_to_rbrace();
                                    break;
                                }
                            },
                        }
                    }
                    rules.push(AkkRule {
                        kind,
                        fields,
                        span: rule_start,
                    });
                }
                AkkToken::Semi => {
                    self.advance();
                }
                _ => {
                    let span = self.peek_span();
                    self.errors.push(ParseError::new(
                        format!(
                            "expected allow/deny in policy block, found {:?}",
                            self.peek()
                        ),
                        span,
                    ));
                    self.recover_to_rbrace();
                    break;
                }
            }
        }
        self.expect(&AkkToken::RBrace)?;
        Some(AkkDecl::Policy(AkkPolicyDecl {
            name,
            rules,
            span: start,
        }))
    }

    fn parse_decl(&mut self) -> Option<AkkDecl> {
        match self.peek() {
            AkkToken::Sign => self.parse_sign(),
            AkkToken::Word => self.parse_word(),
            AkkToken::AolMap => self.parse_aol_map(),
            AkkToken::Seek => self.parse_seek(),
            AkkToken::Ingest => self.parse_ingest(),
            AkkToken::Policy => self.parse_policy(),
            AkkToken::Semi => {
                self.advance();
                None
            }
            other => {
                let span = self.peek_span();
                self.errors.push(ParseError::new(
                    format!("unexpected token in tablet body: {:?}", other),
                    span,
                ));
                self.advance();
                None
            }
        }
    }

    fn parse_tablet(&mut self) -> Option<AkkTablet> {
        let start = self.peek_span();
        self.advance();
        let (name, _) = self.expect_ident()?;
        self.expect(&AkkToken::LBrace)?;
        let annotations = self.parse_annotations();
        let mut decls = Vec::new();
        loop {
            match self.peek() {
                AkkToken::RBrace | AkkToken::Eof => break,
                _ => {
                    if let Some(decl) = self.parse_decl() {
                        decls.push(decl);
                    }
                }
            }
        }
        self.expect(&AkkToken::RBrace)?;
        Some(AkkTablet {
            name,
            annotations,
            decls,
            span: start,
        })
    }

    // ── Public API ────────────────────────────────────────────────────────────

    pub fn parse(&mut self) -> AkkFile {
        let start = self.peek_span();
        let mut tablets = Vec::new();
        loop {
            match self.peek() {
                AkkToken::Eof => break,
                AkkToken::Tablet => {
                    if let Some(t) = self.parse_tablet() {
                        tablets.push(t);
                    }
                }
                AkkToken::Semi => {
                    self.advance();
                }
                other => {
                    let span = self.peek_span();
                    self.errors.push(ParseError::new(
                        format!("expected 'tablet' at top level, found {:?}", other),
                        span,
                    ));
                    self.advance();
                }
            }
        }
        AkkFile {
            tablets,
            span: start,
        }
    }

    pub fn parse_source(src: &str) -> (AkkFile, Vec<LexError>, Vec<ParseError>) {
        let (tokens, lex_errors) = Lexer::lex(src);
        let mut parser = Parser::new(tokens);
        let ast = parser.parse();
        (ast, lex_errors, parser.errors)
    }
}

// ── AST helpers ───────────────────────────────────────────────────────────────

impl AkkFile {
    pub fn tablet(&self, name: &str) -> Option<&AkkTablet> {
        self.tablets.iter().find(|t| t.name == name)
    }
}

impl AkkTablet {
    pub fn sign(&self, name: &str) -> Option<&AkkSignDecl> {
        self.decls.iter().find_map(|d| match d {
            AkkDecl::Sign(s) if s.name == name => Some(s),
            _ => None,
        })
    }
    pub fn policy(&self, name: &str) -> Option<&AkkPolicyDecl> {
        self.decls.iter().find_map(|d| match d {
            AkkDecl::Policy(p) if p.name == name => Some(p),
            _ => None,
        })
    }
    pub fn version(&self) -> Option<&str> {
        self.annotations.iter().find_map(|a| match &a.kind {
            AnnotationKind::Version(v) => Some(v.as_str()),
            _ => None,
        })
    }
}

impl AkkSignDecl {
    pub fn field(&self, key: &str) -> Option<&AkkExpr> {
        self.fields.iter().find(|f| f.key == key).map(|f| &f.value)
    }
    pub fn cite(&self) -> Option<&str> {
        self.annotations.iter().find_map(|a| match &a.kind {
            AnnotationKind::Cite(s) => Some(s.as_str()),
            _ => None,
        })
    }
}

impl AkkWordDecl {
    pub fn field(&self, key: &str) -> Option<&AkkExpr> {
        self.fields.iter().find(|f| f.key == key).map(|f| &f.value)
    }
    pub fn cite(&self) -> Option<&str> {
        self.annotations.iter().find_map(|a| match &a.kind {
            AnnotationKind::Cite(s) => Some(s.as_str()),
            _ => None,
        })
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(src: &str) -> (AkkFile, Vec<ParseError>) {
        let (tokens, _) = Lexer::lex(src);
        let mut parser = Parser::new(tokens);
        let ast = parser.parse();
        (ast, parser.errors)
    }

    #[test]
    fn test_empty_file() {
        let (ast, errs) = parse("");
        assert!(errs.is_empty());
        assert!(ast.tablets.is_empty());
    }

    #[test]
    fn test_empty_tablet() {
        let (ast, errs) = parse("tablet my_tablet {}");
        assert!(errs.is_empty(), "{:?}", errs);
        assert_eq!(ast.tablets[0].name, "my_tablet");
    }

    #[test]
    fn test_tablet_version_annotation() {
        let (ast, errs) = parse(r#"tablet akkadian_dict { @version "3.5" }"#);
        assert!(errs.is_empty(), "{:?}", errs);
        assert_eq!(ast.tablet("akkadian_dict").unwrap().version(), Some("3.5"));
    }

    #[test]
    fn test_sign_declaration() {
        let src = r#"
            tablet test {
                sign an_dingir {
                    meszl = #29
                    glyph = 𒀭
                    meaning = "sky, heaven, god"
                    @cite MesZL
                }
            }
        "#;
        let (ast, errs) = parse(src);
        assert!(errs.is_empty(), "{:?}", errs);
        let sign = ast.tablet("test").unwrap().sign("an_dingir").unwrap();
        assert_eq!(sign.field("meszl"), Some(&AkkExpr::MeszlRef(29)));
        assert_eq!(sign.field("glyph"), Some(&AkkExpr::Glyph('𒀭')));
        assert_eq!(sign.cite(), Some("MesZL"));
    }

    #[test]
    fn test_policy_allow_deny() {
        let src = r#"
            tablet larsa_phone {
                policy camera_access {
                    deny  subject = "any_app"
                    allow subject = "akkadi_camera"
                }
            }
        "#;
        let (ast, errs) = parse(src);
        assert!(errs.is_empty(), "{:?}", errs);
        let policy = ast
            .tablet("larsa_phone")
            .unwrap()
            .policy("camera_access")
            .unwrap();
        assert_eq!(policy.rules[0].kind, RuleKind::Deny);
        assert_eq!(policy.rules[1].kind, RuleKind::Allow);
    }

    #[test]
    fn test_dotted_path_expr() {
        let src = r#"tablet test { seek { filter = kill_switch.camera } }"#;
        let (ast, errs) = parse(src);
        assert!(errs.is_empty(), "{:?}", errs);
        let seek = ast
            .tablet("test")
            .unwrap()
            .decls
            .iter()
            .find_map(|d| match d {
                AkkDecl::Seek(s) => Some(s),
                _ => None,
            })
            .unwrap();
        assert_eq!(
            seek.fields[0].value,
            AkkExpr::Path(vec!["kill_switch".into(), "camera".into()])
        );
    }

    #[test]
    fn test_parse_source_convenience() {
        let (ast, le, pe) = Parser::parse_source(r#"tablet test { sign x { val = 1 } }"#);
        assert!(le.is_empty());
        assert!(pe.is_empty());
        assert_eq!(ast.tablets.len(), 1);
    }
}
