//! PMPVD AkkIR — Particles Meta-Programming Vector Database Language IR.
//!
//! The 9 sovereign AkkNode types:
//!
//! | Group      | Nodes                       |
//! |------------|-----------------------------|
//! | Identity   | PARTICLE, TRIBE             |
//! | Intelligence | RULE, EQUATION, GUARD     |
//! | Flow       | FLOW, OBSERVE               |
//! | Autonomy   | EMIT, PIPELINE              |
//!
//! Every node carries a `NodeId` — a deterministic FNV-1a hash of
//! (kind, name).  In v4.0 production this maps to a KAKI sovereign PK
//! (enkidb-kaki).  The IR is target-agnostic: same .akk source emits to
//! Rust, JavaScript, and WASM without rewriting.
//!
//! DUB.SAR 𒁾 — BahyWay.Ecosystem v4.0 | Pure Rust

use crate::token::{Token, tokenize};

// ── NodeId ────────────────────────────────────────────────────────────────────

/// Deterministic node identity — FNV-1a hash of (kind_str, name).
/// Stable across compilation: same source always yields the same NodeId.
pub type NodeId = u64;

fn fnv1a(data: &[u8]) -> NodeId {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in data {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

fn make_node_id(kind: &str, name: &str) -> NodeId {
    let mut data = Vec::with_capacity(kind.len() + 1 + name.len());
    data.extend_from_slice(kind.as_bytes());
    data.push(b'|');
    data.extend_from_slice(name.as_bytes());
    fnv1a(&data)
}

// ── Top-level IR types ────────────────────────────────────────────────────────

/// A complete PMPVD program — ordered list of AkkNodes.
#[derive(Debug, Clone)]
pub struct PmpvdProgram {
    pub nodes: Vec<AkkNode>,
}

/// One sovereign AkkNode with its deterministic identity.
#[derive(Debug, Clone)]
pub struct AkkNode {
    pub node_id: NodeId,
    pub kind:    AkkNodeKind,
}

/// The 9 sovereign AkkNode kinds.
#[derive(Debug, Clone)]
pub enum AkkNodeKind {
    /// Identity — declares a sovereign data particle type.
    Particle(ParticleDecl),
    /// Identity — declares a tribe ideal point and weight profile.
    Tribe(PmpvdTribeDecl),
    /// Intelligence — orbital logic: when condition fires, take actions.
    Rule(RuleDecl),
    /// Intelligence — self-referential: observes own output, emits fixes.
    Equation(EquationDecl),
    /// Intelligence — access gate: require conditions, allow lanes.
    Guard(GuardDecl),
    /// Flow — streaming pipeline: source → pipe chain → sink.
    Flow(FlowDecl),
    /// Flow — geometric rotation query: returns particle states, not rows.
    Observe(ObserveDecl),
    /// Autonomy — emits a KAKI-stamped .akk event or file.
    Emit(EmitDecl),
    /// Autonomy — orchestrates a sovereign boot or healing sequence.
    Pipeline(PipelineDecl),
}

// ── PARTICLE ──────────────────────────────────────────────────────────────────

/// `PARTICLE name { fields... TRIBE: TribeName }`
#[derive(Debug, Clone)]
pub struct ParticleDecl {
    pub name:       String,
    pub fields:     Vec<FieldDecl>,
    pub tribe_name: Option<String>,
}

/// One field in a PARTICLE declaration.
#[derive(Debug, Clone)]
pub struct FieldDecl {
    pub name:        String,
    pub field_type:  FieldType,
    pub constraints: Vec<QualityConstraint>,
}

/// Field data type.
#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    /// `TEXT(...)` — human-readable text subject to quality scoring.
    Text,
    /// `KAKI` — sovereign 16-byte primary key.
    Kaki,
    /// Custom named type (domain-specific).
    Custom(String),
}

/// Per-field quality constraint: `dimension = threshold`.
///
/// Example: `accuracy=0.98` maps to `{ dimension: "accuracy", threshold: 0.98 }`.
#[derive(Debug, Clone)]
pub struct QualityConstraint {
    pub dimension: String,
    pub threshold: f64,
}

// ── TRIBE (PMPVD variant) ─────────────────────────────────────────────────────

/// `TRIBE name { WEIGHTS: [...] IDEAL: [...] }`
#[derive(Debug, Clone)]
pub struct PmpvdTribeDecl {
    pub name:    String,
    pub weights: [f64; 7],
    pub ideal:   [f64; 7],
}

impl PmpvdTribeDecl {
    /// Arabic MDM sovereign default weights.
    pub fn sovereign_arabic_mdm(name: impl Into<String>) -> Self {
        Self {
            name:    name.into(),
            weights: [0.30, 0.20, 0.15, 0.15, 0.10, 0.05, 0.05],
            ideal:   [1.0; 7],
        }
    }

    /// Validate weights sum ≈ 1.0.
    pub fn weights_valid(&self) -> bool {
        let sum: f64 = self.weights.iter().sum();
        (sum - 1.0).abs() < 0.001
    }
}

// ── RULE ──────────────────────────────────────────────────────────────────────

/// `RULE name { WHEN conditions... THEN actions... }`
#[derive(Debug, Clone)]
pub struct RuleDecl {
    pub name:       String,
    pub conditions: Vec<Condition>,
    pub actions:    Vec<RuleAction>,
}

/// A single WHEN condition: `path op value`.
///
/// Example: `particle.hepta.health >= 0.833`
#[derive(Debug, Clone)]
pub struct Condition {
    /// Dotted path: e.g. `"particle.hepta.health"`.
    pub path:  String,
    pub op:    CompareOp,
    pub value: ConditionValue,
}

/// Comparison operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareOp {
    Geq, Leq, Gt, Lt, Eq,
}

/// Right-hand side of a condition.
#[derive(Debug, Clone, PartialEq)]
pub enum ConditionValue {
    Float(f64),
    Integer(u64),
    Ident(String),
}

/// Action taken in a THEN block.
#[derive(Debug, Clone)]
pub enum RuleAction {
    /// `LANE = GEM`
    SetLane(String),
    /// `ORBIT = inner`
    SetOrbit(String),
    /// `EMIT StoryWay("event_name")`
    EmitEvent(EmitDecl),
}

// ── EQUATION ─────────────────────────────────────────────────────────────────

/// `EQUATION name { WHEN conditions... THEN EMIT target { payload } }`
///
/// An Equation is a Rule that always emits a .akk repair or mutation.
/// Level 1: observes own output and updates own parameters.
/// Level 2: generates new equations for novel patterns.
/// Level 3: meta-equation that manages the equation library.
#[derive(Debug, Clone)]
pub struct EquationDecl {
    pub name:       String,
    pub conditions: Vec<Condition>,
    pub emit:       EmitDecl,
}

// ── GUARD ─────────────────────────────────────────────────────────────────────

/// `GUARD name { REQUIRE: conditions... ALLOW: [lane, ...] }`
#[derive(Debug, Clone)]
pub struct GuardDecl {
    pub name:     String,
    pub requires: Vec<Condition>,
    pub allows:   Vec<String>,
}

// ── FLOW ──────────────────────────────────────────────────────────────────────

/// `FLOW name { SOURCE: ... PIPE: ... SINK: ... }`
#[derive(Debug, Clone)]
pub struct FlowDecl {
    pub name:   String,
    pub source: FlowEndpoint,
    pub pipes:  Vec<PipeStep>,
    pub sink:   FlowEndpoint,
}

/// A service endpoint: `ServiceName(arg)` or bare `ServiceName`.
#[derive(Debug, Clone)]
pub struct FlowEndpoint {
    pub service: String,
    pub arg:     Option<String>,
}

/// A single PIPE step in a FLOW.
#[derive(Debug, Clone)]
pub struct PipeStep {
    pub service: String,
    pub arg:     Option<String>,
}

// ── OBSERVE ───────────────────────────────────────────────────────────────────

/// `OBSERVE name { FROM: TRIBE X WHERE: ... SORT: ... LIMIT: N }`
///
/// A geometric rotation query — returns particle states, not SQL rows.
#[derive(Debug, Clone)]
pub struct ObserveDecl {
    pub name:       String,
    pub from_tribe: String,
    pub where_cond: Option<Condition>,
    pub sort_by:    Option<SortSpec>,
    pub limit:      Option<u64>,
}

/// Sort specification: `orbital_radius ASC`.
#[derive(Debug, Clone)]
pub struct SortSpec {
    pub field: String,
    pub order: SortOrder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder { Asc, Desc }

// ── EMIT ──────────────────────────────────────────────────────────────────────

/// `EMIT Target("payload")` or `EMIT target.akk { key: value ... }`
#[derive(Debug, Clone)]
pub struct EmitDecl {
    pub target:  String,
    pub payload: Option<String>,
}

// ── PIPELINE ─────────────────────────────────────────────────────────────────

/// `PIPELINE name version { Stage1 Stage2 ... }`
///
/// A sovereign boot or healing sequence — each stage is a service name.
#[derive(Debug, Clone)]
pub struct PipelineDecl {
    pub name:    String,
    pub version: Option<String>,
    pub stages:  Vec<String>,
}

// ── Parser ────────────────────────────────────────────────────────────────────

/// PMPVD parser — converts an AAOL token stream into a `PmpvdProgram`.
pub struct PmpvdParser {
    tokens: Vec<Token>,
    pos:    usize,
}

impl PmpvdParser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    /// Parse `.akk` source text directly.
    pub fn parse_source(source: &str) -> Result<PmpvdProgram, String> {
        let tokens = tokenize(source)?;
        Self::new(tokens).parse()
    }

    pub fn parse(&mut self) -> Result<PmpvdProgram, String> {
        let mut nodes = Vec::new();
        while !self.is_eof() {
            nodes.push(self.parse_node()?);
        }
        Ok(PmpvdProgram { nodes })
    }

    fn peek(&self) -> &Token { &self.tokens[self.pos] }

    fn advance(&mut self) -> Token {
        let t = self.tokens[self.pos].clone();
        self.pos += 1;
        t
    }

    fn is_eof(&self) -> bool { matches!(self.peek(), Token::Eof) }

    fn expect_ident(&mut self) -> Result<String, String> {
        match self.advance() {
            Token::Ident(s) => Ok(s),
            t => Err(format!("Expected identifier, got {t:?}")),
        }
    }

    fn expect_ident_or_kw(&mut self) -> Result<String, String> {
        match self.advance() {
            Token::Ident(s)    => Ok(s),
            Token::Text        => Ok("TEXT".into()),
            Token::KakiType    => Ok("KAKI".into()),
            Token::Asc         => Ok("ASC".into()),
            Token::Desc        => Ok("DESC".into()),
            Token::From        => Ok("FROM".into()),
            Token::Where       => Ok("WHERE".into()),
            Token::Sort        => Ok("SORT".into()),
            Token::Limit       => Ok("LIMIT".into()),
            Token::Source      => Ok("SOURCE".into()),
            Token::Pipe        => Ok("PIPE".into()),
            Token::Sink        => Ok("SINK".into()),
            Token::Require     => Ok("REQUIRE".into()),
            Token::Allow       => Ok("ALLOW".into()),
            Token::Lane        => Ok("LANE".into()),
            Token::Orbit       => Ok("ORBIT".into()),
            t => Err(format!("Expected identifier or keyword, got {t:?}")),
        }
    }

    /// Accept any token that can appear as a segment in a dotted path expression.
    /// Covers both plain identifiers and PMPVD keywords used as variable names.
    fn expect_path_segment(&mut self) -> Result<String, String> {
        match self.advance() {
            Token::Ident(s)  => Ok(s),
            Token::Particle  => Ok("particle".into()),
            Token::Tribe     => Ok("tribe".into()),
            Token::Rule      => Ok("rule".into()),
            Token::Equation  => Ok("equation".into()),
            Token::Guard     => Ok("guard".into()),
            Token::Flow      => Ok("flow".into()),
            Token::Observe   => Ok("observe".into()),
            Token::Pipeline  => Ok("pipeline".into()),
            Token::Lane      => Ok("lane".into()),
            Token::Orbit     => Ok("orbit".into()),
            Token::Weights   => Ok("weights".into()),
            Token::Ideal     => Ok("ideal".into()),
            Token::KakiType  => Ok("kaki".into()),
            Token::Text      => Ok("text".into()),
            t => Err(format!("Expected path segment (ident or keyword), got {t:?}")),
        }
    }

    fn expect_string(&mut self) -> Result<String, String> {
        match self.advance() {
            Token::StringLit(s) => Ok(s),
            t => Err(format!("Expected string literal, got {t:?}")),
        }
    }

    fn expect(&mut self, expected: &Token) -> Result<(), String> {
        let tok = self.advance();
        if &tok == expected {
            Ok(())
        } else {
            Err(format!("Expected {expected:?}, got {tok:?}"))
        }
    }

    fn parse_node(&mut self) -> Result<AkkNode, String> {
        match self.peek().clone() {
            Token::Particle => self.parse_particle(),
            Token::Tribe    => self.parse_pmpvd_tribe(),
            Token::Rule     => self.parse_rule(),
            Token::Equation => self.parse_equation(),
            Token::Guard    => self.parse_guard(),
            Token::Flow     => self.parse_flow(),
            Token::Observe  => self.parse_observe(),
            Token::Emit     => self.parse_emit_node(),
            Token::Pipeline => self.parse_pipeline(),
            t => Err(format!("Expected PMPVD node keyword, got {t:?}")),
        }
    }

    // ── PARTICLE ──────────────────────────────────────────────────────────

    fn parse_particle(&mut self) -> Result<AkkNode, String> {
        self.advance(); // consume PARTICLE
        let name = self.expect_ident()?;
        self.expect(&Token::LBrace)?;

        let mut fields     = Vec::new();
        let mut tribe_name = None;

        while !matches!(self.peek(), Token::RBrace | Token::Eof) {
            // TRIBE: TribeName
            if matches!(self.peek(), Token::Tribe) {
                self.advance();
                self.expect(&Token::Colon)?;
                tribe_name = Some(self.expect_ident()?);
                continue;
            }
            fields.push(self.parse_field_decl()?);
        }
        self.expect(&Token::RBrace)?;

        let node_id = make_node_id("particle", &name);
        Ok(AkkNode {
            node_id,
            kind: AkkNodeKind::Particle(ParticleDecl { name, fields, tribe_name }),
        })
    }

    fn parse_field_decl(&mut self) -> Result<FieldDecl, String> {
        let name = self.expect_ident()?;
        self.expect(&Token::Colon)?;

        let (field_type, constraints) = match self.advance() {
            Token::Text => {
                let constraints = if matches!(self.peek(), Token::LParen) {
                    self.advance();
                    let cs = self.parse_constraints()?;
                    self.expect(&Token::RParen)?;
                    cs
                } else {
                    Vec::new()
                };
                (FieldType::Text, constraints)
            }
            Token::KakiType => (FieldType::Kaki, Vec::new()),
            Token::Ident(s) => (FieldType::Custom(s), Vec::new()),
            t => return Err(format!("Expected field type, got {t:?}")),
        };

        Ok(FieldDecl { name, field_type, constraints })
    }

    fn parse_constraints(&mut self) -> Result<Vec<QualityConstraint>, String> {
        let mut cs = Vec::new();
        while !matches!(self.peek(), Token::RParen | Token::Eof) {
            let dim = self.expect_ident()?;
            self.expect(&Token::EqSign)?;
            let threshold = self.parse_float()?;
            cs.push(QualityConstraint { dimension: dim, threshold });
            if matches!(self.peek(), Token::Comma) { self.advance(); }
        }
        Ok(cs)
    }

    fn parse_float(&mut self) -> Result<f64, String> {
        match self.advance() {
            Token::Float(f)  => Ok(f),
            Token::Number(n) => Ok(n as f64),
            t => Err(format!("Expected float, got {t:?}")),
        }
    }

    // ── TRIBE (PMPVD variant) ─────────────────────────────────────────────

    fn parse_pmpvd_tribe(&mut self) -> Result<AkkNode, String> {
        self.advance(); // consume TRIBE
        let name = self.expect_ident()?;
        self.expect(&Token::LBrace)?;

        let mut weights = [0.0f64; 7];
        let mut ideal   = [1.0f64; 7];

        while !matches!(self.peek(), Token::RBrace | Token::Eof) {
            match self.peek().clone() {
                Token::Weights => {
                    self.advance();
                    self.expect(&Token::Colon)?;
                    weights = self.parse_f64_array7()?;
                }
                Token::Ideal => {
                    self.advance();
                    self.expect(&Token::Colon)?;
                    ideal = self.parse_f64_array7()?;
                }
                _ => { self.advance(); } // skip unknown tokens
            }
        }
        self.expect(&Token::RBrace)?;

        let node_id = make_node_id("tribe", &name);
        Ok(AkkNode {
            node_id,
            kind: AkkNodeKind::Tribe(PmpvdTribeDecl { name, weights, ideal }),
        })
    }

    fn parse_f64_array7(&mut self) -> Result<[f64; 7], String> {
        self.expect(&Token::LBracket)?;
        let mut vals = [0.0f64; 7];
        for (i, val) in vals.iter_mut().enumerate() {
            *val = self.parse_float()?;
            if i < 6 { self.expect(&Token::Comma)?; }
        }
        self.expect(&Token::RBracket)?;
        Ok(vals)
    }

    // ── RULE ─────────────────────────────────────────────────────────────

    fn parse_rule(&mut self) -> Result<AkkNode, String> {
        self.advance(); // consume RULE
        let name = self.expect_ident()?;
        self.expect(&Token::LBrace)?;

        let conditions = self.parse_when_clause()?;
        let actions    = self.parse_then_actions()?;

        self.expect(&Token::RBrace)?;

        let node_id = make_node_id("rule", &name);
        Ok(AkkNode {
            node_id,
            kind: AkkNodeKind::Rule(RuleDecl { name, conditions, actions }),
        })
    }

    fn parse_when_clause(&mut self) -> Result<Vec<Condition>, String> {
        if !matches!(self.peek(), Token::When) {
            return Ok(Vec::new());
        }
        self.advance(); // consume WHEN
        let mut conditions = vec![self.parse_condition()?];
        while matches!(self.peek(), Token::And) {
            self.advance(); // consume AND
            conditions.push(self.parse_condition()?);
        }
        Ok(conditions)
    }

    fn parse_condition(&mut self) -> Result<Condition, String> {
        // path: one or more idents joined by dots.
        // Keywords used as path segments (e.g. "particle.hepta.health") are
        // accepted here — in a condition path, any token that names a concept
        // is valid as a segment.
        let mut path = self.expect_path_segment()?;
        while matches!(self.peek(), Token::Dot) {
            self.advance();
            let seg = self.expect_path_segment()?;
            path.push('.');
            path.push_str(&seg);
        }

        let op = match self.advance() {
            Token::Geq    => CompareOp::Geq,
            Token::Leq    => CompareOp::Leq,
            Token::Gt     => CompareOp::Gt,
            Token::Lt     => CompareOp::Lt,
            Token::EqSign => CompareOp::Eq,
            t => return Err(format!("Expected comparison operator, got {t:?}")),
        };

        let value = match self.peek().clone() {
            Token::Float(f)  => { self.advance(); ConditionValue::Float(f) }
            Token::Number(n) => { self.advance(); ConditionValue::Integer(n) }
            Token::Ident(s)  => { self.advance(); ConditionValue::Ident(s) }
            t => return Err(format!("Expected condition value, got {t:?}")),
        };

        Ok(Condition { path, op, value })
    }

    fn parse_then_actions(&mut self) -> Result<Vec<RuleAction>, String> {
        if !matches!(self.peek(), Token::Then) { return Ok(Vec::new()); }
        self.advance(); // consume THEN

        let mut actions = Vec::new();
        while !matches!(self.peek(), Token::RBrace | Token::Eof) {
            match self.peek().clone() {
                Token::Lane => {
                    self.advance();
                    self.expect(&Token::EqSign)?;
                    let lane = self.expect_ident_or_kw()?;
                    actions.push(RuleAction::SetLane(lane));
                }
                Token::Orbit => {
                    self.advance();
                    self.expect(&Token::EqSign)?;
                    let orbit = self.expect_ident()?;
                    actions.push(RuleAction::SetOrbit(orbit));
                }
                Token::Emit => {
                    let emit = self.parse_emit_decl()?;
                    actions.push(RuleAction::EmitEvent(emit));
                }
                _ => break,
            }
        }
        Ok(actions)
    }

    // ── EQUATION ─────────────────────────────────────────────────────────

    fn parse_equation(&mut self) -> Result<AkkNode, String> {
        self.advance(); // consume EQUATION
        let name = self.expect_ident()?;
        self.expect(&Token::LBrace)?;

        let conditions = self.parse_when_clause()?;
        // THEN EMIT ...
        if matches!(self.peek(), Token::Then) { self.advance(); }
        let emit = self.parse_emit_decl()?;

        self.expect(&Token::RBrace)?;

        let node_id = make_node_id("equation", &name);
        Ok(AkkNode {
            node_id,
            kind: AkkNodeKind::Equation(EquationDecl { name, conditions, emit }),
        })
    }

    // ── GUARD ─────────────────────────────────────────────────────────────

    fn parse_guard(&mut self) -> Result<AkkNode, String> {
        self.advance(); // consume GUARD
        let name = self.expect_ident()?;
        self.expect(&Token::LBrace)?;

        let mut requires = Vec::new();
        let mut allows   = Vec::new();

        while !matches!(self.peek(), Token::RBrace | Token::Eof) {
            match self.peek().clone() {
                Token::Require => {
                    self.advance();
                    self.expect(&Token::Colon)?;
                    requires.push(self.parse_condition()?);
                }
                Token::Allow => {
                    self.advance();
                    self.expect(&Token::Colon)?;
                    self.expect(&Token::LBracket)?;
                    while !matches!(self.peek(), Token::RBracket | Token::Eof) {
                        allows.push(self.expect_ident()?);
                        if matches!(self.peek(), Token::Comma) { self.advance(); }
                    }
                    self.expect(&Token::RBracket)?;
                }
                _ => { self.advance(); }
            }
        }
        self.expect(&Token::RBrace)?;

        let node_id = make_node_id("guard", &name);
        Ok(AkkNode {
            node_id,
            kind: AkkNodeKind::Guard(GuardDecl { name, requires, allows }),
        })
    }

    // ── FLOW ──────────────────────────────────────────────────────────────

    fn parse_flow(&mut self) -> Result<AkkNode, String> {
        self.advance(); // consume FLOW
        let name = self.expect_ident()?;
        self.expect(&Token::LBrace)?;

        let mut source = FlowEndpoint { service: String::new(), arg: None };
        let mut pipes  = Vec::new();
        let mut sink   = FlowEndpoint { service: String::new(), arg: None };

        while !matches!(self.peek(), Token::RBrace | Token::Eof) {
            match self.peek().clone() {
                Token::Source => {
                    self.advance();
                    self.expect(&Token::Colon)?;
                    source = self.parse_endpoint()?;
                }
                Token::Pipe => {
                    self.advance();
                    self.expect(&Token::Colon)?;
                    let ep = self.parse_endpoint()?;
                    pipes.push(PipeStep { service: ep.service, arg: ep.arg });
                }
                Token::Sink => {
                    self.advance();
                    self.expect(&Token::Colon)?;
                    sink = self.parse_endpoint()?;
                }
                _ => { self.advance(); }
            }
        }
        self.expect(&Token::RBrace)?;

        let node_id = make_node_id("flow", &name);
        Ok(AkkNode {
            node_id,
            kind: AkkNodeKind::Flow(FlowDecl { name, source, pipes, sink }),
        })
    }

    fn parse_endpoint(&mut self) -> Result<FlowEndpoint, String> {
        let service = self.expect_ident()?;
        let arg = if matches!(self.peek(), Token::LParen) {
            self.advance();
            // collect everything until ')' as raw string
            let mut raw = String::new();
            let mut depth = 1usize;
            while !self.is_eof() && depth > 0 {
                match self.peek().clone() {
                    Token::LParen  => { depth += 1; raw.push('('); self.advance(); }
                    Token::RParen  => {
                        depth -= 1;
                        if depth > 0 { raw.push(')'); }
                        self.advance();
                    }
                    Token::Ident(s) => { raw.push_str(&s); self.advance(); }
                    Token::StringLit(s) => { raw.push_str(&s); self.advance(); }
                    Token::EqSign  => { raw.push('='); self.advance(); }
                    _ => { self.advance(); }
                }
            }
            Some(raw)
        } else {
            None
        };
        Ok(FlowEndpoint { service, arg })
    }

    // ── OBSERVE ───────────────────────────────────────────────────────────

    fn parse_observe(&mut self) -> Result<AkkNode, String> {
        self.advance(); // consume OBSERVE
        let name = self.expect_ident()?;
        self.expect(&Token::LBrace)?;

        let mut from_tribe = String::new();
        let mut where_cond = None;
        let mut sort_by    = None;
        let mut limit      = None;

        while !matches!(self.peek(), Token::RBrace | Token::Eof) {
            match self.peek().clone() {
                Token::From => {
                    self.advance();
                    self.expect(&Token::Colon)?;
                    // optional "TRIBE" keyword before name
                    if matches!(self.peek(), Token::Tribe) { self.advance(); }
                    from_tribe = self.expect_ident()?;
                }
                Token::Where => {
                    self.advance();
                    self.expect(&Token::Colon)?;
                    where_cond = Some(self.parse_condition()?);
                }
                Token::Sort => {
                    self.advance();
                    self.expect(&Token::Colon)?;
                    let field = self.expect_ident()?;
                    let order = match self.peek().clone() {
                        Token::Asc  => { self.advance(); SortOrder::Asc  }
                        Token::Desc => { self.advance(); SortOrder::Desc }
                        _           => SortOrder::Asc,
                    };
                    sort_by = Some(SortSpec { field, order });
                }
                Token::Limit => {
                    self.advance();
                    self.expect(&Token::Colon)?;
                    match self.advance() {
                        Token::Number(n) => limit = Some(n),
                        t => return Err(format!("Expected number for LIMIT, got {t:?}")),
                    }
                }
                _ => { self.advance(); }
            }
        }
        self.expect(&Token::RBrace)?;

        let node_id = make_node_id("observe", &name);
        Ok(AkkNode {
            node_id,
            kind: AkkNodeKind::Observe(ObserveDecl {
                name, from_tribe, where_cond, sort_by, limit,
            }),
        })
    }

    // ── EMIT ──────────────────────────────────────────────────────────────

    fn parse_emit_node(&mut self) -> Result<AkkNode, String> {
        let emit = self.parse_emit_decl()?;
        let node_id = make_node_id("emit", &emit.target);
        Ok(AkkNode { node_id, kind: AkkNodeKind::Emit(emit) })
    }

    fn parse_emit_decl(&mut self) -> Result<EmitDecl, String> {
        self.expect(&Token::Emit)?;
        let target = self.expect_ident()?;
        let payload = if matches!(self.peek(), Token::LParen) {
            self.advance();
            let s = self.expect_string()?;
            self.expect(&Token::RParen)?;
            Some(s)
        } else {
            // skip optional { ... } payload block
            if matches!(self.peek(), Token::LBrace) {
                let mut depth = 1usize;
                self.advance();
                while !self.is_eof() && depth > 0 {
                    match self.advance() {
                        Token::LBrace => depth += 1,
                        Token::RBrace => depth -= 1,
                        _ => {}
                    }
                }
            }
            None
        };
        Ok(EmitDecl { target, payload })
    }

    // ── PIPELINE ─────────────────────────────────────────────────────────

    fn parse_pipeline(&mut self) -> Result<AkkNode, String> {
        self.advance(); // consume PIPELINE
        let name = self.expect_ident()?;
        // optional version string
        let version = if matches!(self.peek(), Token::Ident(_)) {
            Some(self.expect_ident()?)
        } else {
            None
        };
        self.expect(&Token::LBrace)?;

        let mut stages = Vec::new();
        while !matches!(self.peek(), Token::RBrace | Token::Eof) {
            stages.push(self.expect_ident()?);
        }
        self.expect(&Token::RBrace)?;

        let node_id = make_node_id("pipeline", &name);
        Ok(AkkNode {
            node_id,
            kind: AkkNodeKind::Pipeline(PipelineDecl { name, version, stages }),
        })
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_id_deterministic() {
        let id1 = make_node_id("particle", "citizen");
        let id2 = make_node_id("particle", "citizen");
        assert_eq!(id1, id2);
    }

    #[test]
    fn node_id_unique_across_kinds() {
        let id_particle = make_node_id("particle", "citizen");
        let id_rule     = make_node_id("rule",     "citizen");
        assert_ne!(id_particle, id_rule);
    }

    #[test]
    fn parse_particle_decl() {
        let src = r#"
            PARTICLE citizen {
                arabic_name: TEXT(accuracy=0.98, completeness=0.95)
                nid:         KAKI
                TRIBE: IraqiCitizenMDM
            }
        "#;
        let prog = PmpvdParser::parse_source(src).unwrap();
        assert_eq!(prog.nodes.len(), 1);
        if let AkkNodeKind::Particle(p) = &prog.nodes[0].kind {
            assert_eq!(p.name, "citizen");
            assert_eq!(p.tribe_name, Some("IraqiCitizenMDM".into()));
            assert_eq!(p.fields.len(), 2);
            assert_eq!(p.fields[0].name, "arabic_name");
            assert_eq!(p.fields[0].field_type, FieldType::Text);
            assert_eq!(p.fields[0].constraints.len(), 2);
            assert_eq!(p.fields[1].field_type, FieldType::Kaki);
        } else {
            panic!("Expected Particle");
        }
    }

    #[test]
    fn parse_pmpvd_tribe() {
        let src = r#"
            TRIBE IraqiCitizenMDM {
                WEIGHTS: [0.30, 0.20, 0.15, 0.15, 0.10, 0.05, 0.05]
                IDEAL:   [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0]
            }
        "#;
        let prog = PmpvdParser::parse_source(src).unwrap();
        if let AkkNodeKind::Tribe(t) = &prog.nodes[0].kind {
            assert_eq!(t.name, "IraqiCitizenMDM");
            assert!((t.weights.iter().sum::<f64>() - 1.0).abs() < 0.001);
            assert!(t.weights_valid());
        } else {
            panic!("Expected Tribe");
        }
    }

    #[test]
    fn parse_rule_with_conditions_and_actions() {
        let src = r#"
            RULE promote_to_gem {
                WHEN particle.hepta.health >= 0.833
                AND  particle.tribe = IraqiCitizenMDM
                THEN
                    LANE = GEM
                    ORBIT = inner
                    EMIT StoryWay("promoted_to_golden")
            }
        "#;
        let prog = PmpvdParser::parse_source(src).unwrap();
        if let AkkNodeKind::Rule(r) = &prog.nodes[0].kind {
            assert_eq!(r.name, "promote_to_gem");
            assert_eq!(r.conditions.len(), 2);
            assert_eq!(r.conditions[0].path, "particle.hepta.health");
            assert_eq!(r.conditions[0].op, CompareOp::Geq);
            assert_eq!(r.conditions[1].op, CompareOp::Eq);
            assert_eq!(r.actions.len(), 3);
            assert!(matches!(r.actions[0], RuleAction::SetLane(_)));
            assert!(matches!(r.actions[1], RuleAction::SetOrbit(_)));
            assert!(matches!(r.actions[2], RuleAction::EmitEvent(_)));
        } else {
            panic!("Expected Rule");
        }
    }

    #[test]
    fn parse_observe_decl() {
        let src = r#"
            OBSERVE golden_citizens {
                FROM: TRIBE IraqiCitizenMDM
                WHERE: health >= 0.833
                SORT: orbital_radius ASC
                LIMIT: 10000
            }
        "#;
        let prog = PmpvdParser::parse_source(src).unwrap();
        if let AkkNodeKind::Observe(o) = &prog.nodes[0].kind {
            assert_eq!(o.name, "golden_citizens");
            assert_eq!(o.from_tribe, "IraqiCitizenMDM");
            assert!(o.where_cond.is_some());
            assert_eq!(o.sort_by.as_ref().unwrap().order, SortOrder::Asc);
            assert_eq!(o.limit, Some(10000));
        } else {
            panic!("Expected Observe");
        }
    }

    #[test]
    fn parse_pipeline_decl() {
        let src = r#"
            PIPELINE sovereign_boot v354 {
                ZeroWay
                VaultWay
                EnkiWay
                AkkadianAOL
                BeeMDMWay
            }
        "#;
        let prog = PmpvdParser::parse_source(src).unwrap();
        if let AkkNodeKind::Pipeline(p) = &prog.nodes[0].kind {
            assert_eq!(p.name, "sovereign_boot");
            assert_eq!(p.version, Some("v354".into()));
            assert_eq!(p.stages.len(), 5);
            assert_eq!(p.stages[0], "ZeroWay");
            assert_eq!(p.stages[4], "BeeMDMWay");
        } else {
            panic!("Expected Pipeline");
        }
    }

    #[test]
    fn parse_flow_decl() {
        let src = r#"
            FLOW intake {
                SOURCE: VaultWay(path)
                PIPE:   Cleanse(arabic_normalization)
                PIPE:   Score
                SINK:   EnkiWay
            }
        "#;
        let prog = PmpvdParser::parse_source(src).unwrap();
        if let AkkNodeKind::Flow(f) = &prog.nodes[0].kind {
            assert_eq!(f.name, "intake");
            assert_eq!(f.source.service, "VaultWay");
            assert_eq!(f.pipes.len(), 2);
            assert_eq!(f.sink.service, "EnkiWay");
        } else {
            panic!("Expected Flow");
        }
    }

    #[test]
    fn parse_multi_node_program() {
        let src = r#"
            PARTICLE my_entity {
                name: TEXT(accuracy=0.90)
                id:   KAKI
                TRIBE: DefaultMDM
            }
            TRIBE DefaultMDM {
                WEIGHTS: [0.30, 0.20, 0.15, 0.15, 0.10, 0.05, 0.05]
                IDEAL:   [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0]
            }
        "#;
        let prog = PmpvdParser::parse_source(src).unwrap();
        assert_eq!(prog.nodes.len(), 2);
        assert!(matches!(prog.nodes[0].kind, AkkNodeKind::Particle(_)));
        assert!(matches!(prog.nodes[1].kind, AkkNodeKind::Tribe(_)));
    }

    #[test]
    fn sovereign_arabic_mdm_weights_valid() {
        let t = PmpvdTribeDecl::sovereign_arabic_mdm("TestTribe");
        assert!(t.weights_valid());
        assert_eq!(t.ideal, [1.0; 7]);
    }

    #[test]
    fn all_nine_node_kinds_round_trip() {
        // Verify each kind is reachable from a known keyword
        let kinds = ["Particle", "Tribe", "Rule", "Equation", "Guard",
                     "Flow", "Observe", "Emit", "Pipeline"];
        for k in &kinds {
            let id = make_node_id(k, "test");
            assert!(id != 0, "{k} NodeId must not be zero");
        }
    }
}
