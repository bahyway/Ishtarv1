//! HeptaScript v2.0 W5H2 Parser — converts a token stream into a HeptaQuery AST.
//!
//! ## Grammar summary (v2.0)
//!
//! ```text
//! query     := node? across? who what? where* when? orbital? why*
//!              tier? state? nash? pattern* lineage? gate? satamu?
//!              how? how_much?
//!
//! node      := NODE db_target (| db_target)*
//! db_target := ALL | EnkiDB | EnkiDW | EnkiSDB | EnkiODB | EnkiQDB | EnkiMDB | EnkiDDB
//!            | NARUDU | ENKI_PATTERN
//!
//! across    := ACROSS (BIGRING Ident | ALL)
//!
//! who       := WHO TribeName.VarName (BOUND_TO TribeName.VarName)*
//! what      := WHAT projection (, projection)*
//! where     := WHERE condition (AND|OR condition)*
//! when      := WHEN (AT EPOCH (N|NOW) | BEFORE EPOCH N | AFTER EPOCH N)
//! orbital   := ORBITAL (N .. N | NOW)
//! why       := WHY why_cond (AND|OR why_cond)*
//!
//! tier      := TIER tier_val (| tier_val)*
//! tier_val  := HOT | WARM | COLD | CRYSTALLIZED
//!
//! state     := STATE state_val (| state_val)*
//! state_val := EMERGING | STABLE | CANONICAL | DEPRECATED
//!
//! nash      := NASH (SCORE op float | BREAKING)
//!
//! pattern   := PATTERN pattern_cond         -- may repeat, all ANDed
//! pattern_cond := TYPE pt | CONFIDENCE op f | MAX_CONSTITUENTS op n
//!
//! lineage   := LINEAGE (DEPTH N | FULL)
//! gate      := GATE (Ident (, Ident)* | ALL)
//! satamu    := SATAMU (REQUIRED | BYPASS Str)
//!
//! how       := HOW BY VarName[attr] (ASC|DESC)?
//! how_much  := HOW_MUCH LIMIT N | HOW_MUCH TOP N BY VarName[attr] (ASC|DESC)?
//! ```

use crate::token::Token;
use crate::query::*;

/// Parse error.
#[derive(Debug, Clone)]
pub struct ParseError(pub String);

impl core::fmt::Display for ParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "HeptaScript parse error: {}", self.0)
    }
}

/// Parse a HeptaScript source string into a `HeptaQuery`.
pub fn parse_query(source: &str) -> Result<HeptaQuery, ParseError> {
    let tokens = crate::token::tokenize(source).map_err(ParseError)?;
    Parser::new(tokens).parse()
}

// ── Internal parser ───────────────────────────────────────────────────────────

struct Parser {
    tokens: Vec<Token>,
    pos:    usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn advance(&mut self) -> Token {
        let t = self.tokens[self.pos].clone();
        self.pos += 1;
        t
    }

    fn expect(&mut self, expected: &Token) -> Result<(), ParseError> {
        let tok = self.advance();
        if &tok == expected {
            Ok(())
        } else {
            Err(ParseError(format!("expected {expected:?}, got {tok:?}")))
        }
    }

    fn parse(&mut self) -> Result<HeptaQuery, ParseError> {
        // Sovereign Operation verb (optional, leads everything else,
        // including NODE/ACROSS). Absent = ORBIT, so every pre-existing
        // query keeps parsing identically.
        let verb = self.maybe_parse_verb();

        // v2.0 routing clauses (before WHO)
        let node   = self.maybe_parse_node()?;
        let across = self.maybe_parse_across()?;

        // v1 core clauses
        let who      = self.parse_who()?;
        let what     = self.maybe_parse_what()?;
        let r#where  = self.maybe_parse_where()?;
        let when     = self.maybe_parse_when()?;
        let orbital  = self.maybe_parse_orbital()?;
        let why      = self.maybe_parse_why()?;

        // v2.0 filter / intelligence clauses (after WHY)
        let tier    = self.maybe_parse_tier()?;
        let state   = self.maybe_parse_state()?;
        let nash    = self.maybe_parse_nash()?;
        let pattern = self.parse_pattern_conditions()?;
        let lineage = self.maybe_parse_lineage()?;
        let gate    = self.maybe_parse_gate()?;
        let satamu  = self.maybe_parse_satamu()?;

        // Ordering and cardinality
        let how      = self.maybe_parse_how()?;
        let how_much = self.maybe_parse_how_much()?;

        // v2.1 aggregate clauses (after HOW_MUCH). GRAVITY (defines the
        // grouping) parses before MEASURE (defines what's measured per
        // group) -- "partition by X, then measure Y per partition" is the
        // natural reading order, and the parser has no trailing-token
        // check, so getting this order wrong would silently drop whichever
        // clause came second in the source rather than error.
        let gravity = self.maybe_parse_gravity()?;
        let measure = self.maybe_parse_measure()?;

        // Production execution hints (1B+ particles, <1 sec)
        let anchor         = self.maybe_parse_anchor()?;
        let stream         = self.maybe_parse_stream();
        let derive_station = self.maybe_parse_derive_station()?;
        let abort_scan     = self.maybe_parse_abort_scan()?;
        let filter_order   = self.parse_filter_order()?;

        Ok(HeptaQuery {
            verb,
            node, across, who, what, r#where, when, why, how, how_much,
            tier, state, nash, pattern, lineage, gate, satamu, orbital,
            measure, gravity,
            tribes: Vec::new(),
            anchor, stream, derive_station, abort_scan, filter_order,
        })
    }

    // ── Sovereign Operation verb ─────────────────────────────────────────────

    fn maybe_parse_verb(&mut self) -> QueryVerb {
        let verb = match self.peek() {
            Token::VerbOrbit   => QueryVerb::Orbit,
            Token::VerbEmit    => QueryVerb::Emit,
            Token::VerbProve   => QueryVerb::Prove,
            Token::VerbSync    => QueryVerb::Sync,
            Token::VerbWitness => QueryVerb::Witness,
            _ => return QueryVerb::Orbit,
        };
        self.advance();
        verb
    }

    // ── v2.0: NODE ───────────────────────────────────────────────────────────

    fn maybe_parse_node(&mut self) -> Result<Option<NodeClause>, ParseError> {
        if !matches!(self.peek(), Token::Node) { return Ok(None); }
        self.advance();
        let mut targets = vec![self.parse_db_target()?];
        while matches!(self.peek(), Token::Pipe) {
            self.advance();
            targets.push(self.parse_db_target()?);
        }
        Ok(Some(NodeClause { targets }))
    }

    fn parse_db_target(&mut self) -> Result<DbTarget, ParseError> {
        match self.advance() {
            Token::NodeAll       => Ok(DbTarget::All),
            Token::DbEnkiDb      => Ok(DbTarget::EnkiDb),
            Token::DbEnkiDw      => Ok(DbTarget::EnkiDw),
            Token::DbEnkiSdb     => Ok(DbTarget::EnkiSdb),
            Token::DbEnkiOdb     => Ok(DbTarget::EnkiOdb),
            Token::DbEnkiQdb     => Ok(DbTarget::EnkiQdb),
            Token::DbEnkiMdb     => Ok(DbTarget::EnkiMdb),
            Token::DbEnkiDdb     => Ok(DbTarget::EnkiDdb),
            Token::DbNarudu      => Ok(DbTarget::Narudu),
            Token::DbEnkiPattern => Ok(DbTarget::EnkiPattern),
            t => Err(ParseError(format!(
                "expected a database target (EnkiDB/EnkiMDB/NARUDU/ALL/…), got {t:?}"
            ))),
        }
    }

    // ── v2.0: ACROSS ─────────────────────────────────────────────────────────

    fn maybe_parse_across(&mut self) -> Result<Option<AcrossClause>, ParseError> {
        if !matches!(self.peek(), Token::Across) { return Ok(None); }
        self.advance();
        match self.peek().clone() {
            Token::Bigring => {
                self.advance();
                let name = match self.advance() {
                    Token::Ident(s) => s,
                    t => return Err(ParseError(format!("expected client name after BIGRING, got {t:?}"))),
                };
                Ok(Some(AcrossClause::Bigring(name)))
            }
            Token::NodeAll => {
                self.advance();
                Ok(Some(AcrossClause::All))
            }
            t => Err(ParseError(format!("expected BIGRING or ALL after ACROSS, got {t:?}"))),
        }
    }

    // ── v1: WHO ──────────────────────────────────────────────────────────────

    fn parse_who(&mut self) -> Result<WhoClause, ParseError> {
        self.expect(&Token::Who)?;
        let primary = self.parse_entity_binding()?;
        let mut bound_to = Vec::new();
        while matches!(self.peek(), Token::BoundTo) {
            self.advance();
            bound_to.push(self.parse_entity_binding()?);
        }
        Ok(WhoClause { primary, bound_to })
    }

    fn parse_entity_binding(&mut self) -> Result<EntityBinding, ParseError> {
        let tribe = match self.advance() {
            Token::Ident(s) => s,
            t => return Err(ParseError(format!("expected tribe name (Ident), got {t:?}"))),
        };
        self.expect(&Token::Dot)?;
        let var = match self.advance() {
            Token::Ident(s) => s,
            t => return Err(ParseError(format!("expected variable name (Ident), got {t:?}"))),
        };
        Ok(EntityBinding { tribe, var })
    }

    // ── v1: WHAT ─────────────────────────────────────────────────────────────

    fn maybe_parse_what(&mut self) -> Result<Option<WhatClause>, ParseError> {
        if !matches!(self.peek(), Token::What) { return Ok(None); }
        self.advance();
        let mut projections = vec![self.parse_projection()?];
        while matches!(self.peek(), Token::Comma) {
            self.advance();
            projections.push(self.parse_projection()?);
        }
        Ok(Some(WhatClause { projections }))
    }

    fn parse_projection(&mut self) -> Result<Projection, ParseError> {
        let var = match self.advance() {
            Token::Ident(s) => s,
            t => return Err(ParseError(format!("expected variable in projection, got {t:?}"))),
        };
        self.expect(&Token::LBracket)?;
        let attrs = if matches!(self.peek(), Token::Star) {
            self.advance();
            vec![]
        } else {
            let mut list = vec![self.parse_dotted_name()?];
            while matches!(self.peek(), Token::Comma) {
                self.advance();
                list.push(self.parse_dotted_name()?);
            }
            list
        };
        self.expect(&Token::RBracket)?;
        Ok(Projection { var, attrs })
    }

    fn parse_dotted_name(&mut self) -> Result<String, ParseError> {
        let tok = self.advance();
        let first = self.token_as_ident(tok)?;
        let mut name = first;
        while matches!(self.peek(), Token::Dot) {
            self.advance();
            let tok = self.advance();
            let part = self.token_as_ident(tok)?;
            name.push('.');
            name.push_str(&part);
        }
        Ok(name)
    }

    /// Convert any word-like token (including v2.0 keyword tokens) to its string
    /// representation when used as an EAV attribute name inside `[...]`.
    fn token_as_ident(&self, tok: Token) -> Result<String, ParseError> {
        match tok {
            Token::Ident(s) => Ok(s),
            // v2.0 keywords that commonly appear as attribute names
            Token::NashScore        => Ok("score".into()),
            Token::LineageFull      => Ok("full".into()),
            Token::PatternType      => Ok("type".into()),
            Token::LineageDepth     => Ok("depth".into()),
            Token::NashBreaking     => Ok("breaking".into()),
            Token::SatamuRequired   => Ok("required".into()),
            Token::SatamuBypass     => Ok("bypass".into()),
            Token::TierHot          => Ok("hot".into()),
            Token::TierWarm         => Ok("warm".into()),
            Token::TierCold         => Ok("cold".into()),
            Token::TierCrystallized => Ok("crystallized".into()),
            Token::StateEmerging    => Ok("emerging".into()),
            Token::StateStable      => Ok("stable".into()),
            Token::StateCanonical   => Ok("canonical".into()),
            Token::StateDeprecated  => Ok("deprecated".into()),
            Token::PatternConfidence    => Ok("confidence".into()),
            Token::PatternConstituents  => Ok("max_constituents".into()),
            // Base v2.0 clause keywords — also valid EAV attribute names (e.g. E[state])
            Token::Tier    => Ok("tier".into()),
            Token::State   => Ok("state".into()),
            Token::Nash    => Ok("nash".into()),
            Token::Pattern => Ok("pattern".into()),
            Token::Lineage => Ok("lineage".into()),
            Token::Gate    => Ok("gate".into()),
            Token::Satamu  => Ok("satamu".into()),
            Token::Node    => Ok("node".into()),
            Token::Across  => Ok("across".into()),
            Token::Orbital => Ok("orbital".into()),
            // Sovereign Operation verb keywords — also valid EAV attribute
            // names (e.g. E[witness], hist.event's own "BIRTH" value is
            // unrelated but the word "emit" is a plausible attr fragment).
            Token::VerbOrbit   => Ok("orbit".into()),
            Token::VerbEmit    => Ok("emit".into()),
            Token::VerbProve   => Ok("prove".into()),
            Token::VerbSync    => Ok("sync".into()),
            Token::VerbWitness => Ok("witness".into()),
            // Also allow v1 sub-keywords (already worked via Ident in v1, now explicit)
            Token::Lane       => Ok("lane".into()),
            Token::By         => Ok("by".into()),
            Token::Top        => Ok("top".into()),
            Token::Asc        => Ok("asc".into()),
            Token::Desc       => Ok("desc".into()),
            t => Err(ParseError(format!("expected attribute name, got {t:?}"))),
        }
    }

    // ── v1: WHERE ────────────────────────────────────────────────────────────

    fn maybe_parse_where(&mut self) -> Result<Vec<WhereCondition>, ParseError> {
        if !matches!(self.peek(), Token::Where) { return Ok(vec![]); }
        self.advance();
        let first = self.parse_where_condition(Combinator::And)?;
        let mut conditions = vec![first];
        loop {
            match self.peek().clone() {
                Token::And => {
                    self.advance();
                    conditions.push(self.parse_where_condition(Combinator::And)?);
                }
                Token::Or => {
                    self.advance();
                    conditions.push(self.parse_where_condition(Combinator::Or)?);
                }
                Token::Ident(_) => {
                    conditions.push(self.parse_where_condition(Combinator::And)?);
                }
                _ => break,
            }
        }
        Ok(conditions)
    }

    fn parse_where_condition(&mut self, combinator: Combinator) -> Result<WhereCondition, ParseError> {
        let var = match self.advance() {
            Token::Ident(s) => s,
            t => return Err(ParseError(format!("expected variable in WHERE condition, got {t:?}"))),
        };
        self.expect(&Token::LBracket)?;
        let attr = self.parse_dotted_name()?;
        self.expect(&Token::RBracket)?;

        let test = if matches!(self.peek(), Token::Exists) {
            self.advance();
            ConditionTest::Exists
        } else if matches!(self.peek(), Token::Not) {
            self.advance();
            self.expect(&Token::Exists)?;
            ConditionTest::NotExists
        } else {
            let op  = self.parse_op()?;
            let val = self.parse_value()?;
            ConditionTest::Cmp { op, val }
        };

        Ok(WhereCondition { combinator, var, attr, test })
    }

    // ── v1: WHEN ─────────────────────────────────────────────────────────────

    fn maybe_parse_when(&mut self) -> Result<Option<WhenClause>, ParseError> {
        if !matches!(self.peek(), Token::When) { return Ok(None); }
        self.advance();
        let clause = match self.peek().clone() {
            Token::At => {
                self.advance();
                self.expect(&Token::Epoch)?;
                let epoch_ref = if matches!(self.peek(), Token::Now) {
                    self.advance();
                    EpochRef::Now
                } else {
                    match self.advance() {
                        Token::Int(n) if n >= 0 => EpochRef::Abs(n as u64),
                        t => return Err(ParseError(format!("expected epoch integer or NOW, got {t:?}"))),
                    }
                };
                WhenClause::AtEpoch(epoch_ref)
            }
            Token::Before => {
                self.advance();
                self.expect(&Token::Epoch)?;
                match self.advance() {
                    Token::Int(n) if n >= 0 => WhenClause::BeforeEpoch(n as u64),
                    t => return Err(ParseError(format!("expected epoch integer, got {t:?}"))),
                }
            }
            Token::After => {
                self.advance();
                self.expect(&Token::Epoch)?;
                match self.advance() {
                    Token::Int(n) if n >= 0 => WhenClause::AfterEpoch(n as u64),
                    t => return Err(ParseError(format!("expected epoch integer, got {t:?}"))),
                }
            }
            t => return Err(ParseError(format!("expected AT/BEFORE/AFTER in WHEN clause, got {t:?}"))),
        };
        Ok(Some(clause))
    }

    // ── v2.0: ORBITAL ────────────────────────────────────────────────────────

    fn maybe_parse_orbital(&mut self) -> Result<Option<OrbitalClause>, ParseError> {
        if !matches!(self.peek(), Token::Orbital) { return Ok(None); }
        self.advance();
        match self.peek().clone() {
            Token::Now => {
                self.advance();
                Ok(Some(OrbitalClause::Now))
            }
            Token::Int(start) if start >= 0 => {
                self.advance();
                self.expect(&Token::DotDot)?;
                let end = match self.advance() {
                    Token::Int(n) if n >= 0 => n as u64,
                    t => return Err(ParseError(format!("expected end orbital (integer), got {t:?}"))),
                };
                Ok(Some(OrbitalClause::Range { start: start as u64, end }))
            }
            t => Err(ParseError(format!("expected NOW or integer after ORBITAL, got {t:?}"))),
        }
    }

    // ── v1: WHY ──────────────────────────────────────────────────────────────

    fn maybe_parse_why(&mut self) -> Result<Vec<WhyCondition>, ParseError> {
        if !matches!(self.peek(), Token::Why) { return Ok(vec![]); }
        self.advance();
        let first = self.parse_why_condition(Combinator::And)?;
        let mut conditions = vec![first];
        loop {
            match self.peek().clone() {
                Token::And => {
                    self.advance();
                    conditions.push(self.parse_why_condition(Combinator::And)?);
                }
                Token::Or => {
                    self.advance();
                    conditions.push(self.parse_why_condition(Combinator::Or)?);
                }
                Token::Lane | Token::QualityByte => {
                    conditions.push(self.parse_why_condition(Combinator::And)?);
                }
                _ => break,
            }
        }
        Ok(conditions)
    }

    fn parse_why_condition(&mut self, combinator: Combinator) -> Result<WhyCondition, ParseError> {
        match self.peek().clone() {
            Token::Lane => {
                self.advance();
                let op  = self.parse_op()?;
                let val = self.parse_lane_value()?;
                Ok(WhyCondition::Lane { combinator, op, val })
            }
            Token::QualityByte => {
                self.advance();
                let op = self.parse_op()?;
                let n = match self.advance() {
                    Token::Int(n) if n >= 0 && n <= 255 => n as u8,
                    t => return Err(ParseError(format!("expected quality_byte 0–255, got {t:?}"))),
                };
                Ok(WhyCondition::QualityByte { combinator, op, val: n })
            }
            Token::Ident(_) => {
                let var = match self.advance() { Token::Ident(s) => s, _ => unreachable!() };
                self.expect(&Token::LBracket)?;
                let attr = self.parse_dotted_name()?;
                self.expect(&Token::RBracket)?;
                if matches!(self.peek(), Token::Not) {
                    self.advance();
                    self.expect(&Token::Exists)?;
                    Ok(WhyCondition::AttrNotExists { combinator, var, attr })
                } else {
                    self.expect(&Token::Exists)?;
                    Ok(WhyCondition::AttrExists { combinator, var, attr })
                }
            }
            t => Err(ParseError(format!("expected WHY condition (LANE/QUALITY_BYTE/VarName), got {t:?}"))),
        }
    }

    fn parse_lane_value(&mut self) -> Result<LaneValue, ParseError> {
        match self.advance() {
            Token::Ident(s) => match s.to_lowercase().as_str() {
                "gold"   => Ok(LaneValue::Gold),
                "silver" => Ok(LaneValue::Silver),
                "white"  => Ok(LaneValue::White),
                "gray"   => Ok(LaneValue::Gray),
                "red"    => Ok(LaneValue::Red),
                "black"  => Ok(LaneValue::Black),
                other    => Err(ParseError(format!("unknown lane value: {other}"))),
            },
            t => Err(ParseError(format!("expected lane value (Gold/Silver/…), got {t:?}"))),
        }
    }

    // ── v2.0: TIER ───────────────────────────────────────────────────────────

    fn maybe_parse_tier(&mut self) -> Result<Option<TierClause>, ParseError> {
        if !matches!(self.peek(), Token::Tier) { return Ok(None); }
        self.advance();
        let mut tiers = vec![self.parse_tier_value()?];
        while matches!(self.peek(), Token::Pipe) {
            self.advance();
            tiers.push(self.parse_tier_value()?);
        }
        Ok(Some(TierClause { tiers }))
    }

    fn parse_tier_value(&mut self) -> Result<StorageTier, ParseError> {
        match self.advance() {
            Token::TierHot          => Ok(StorageTier::Hot),
            Token::TierWarm         => Ok(StorageTier::Warm),
            Token::TierCold         => Ok(StorageTier::Cold),
            Token::TierCrystallized => Ok(StorageTier::Crystallized),
            t => Err(ParseError(format!("expected tier value (HOT/WARM/COLD/CRYSTALLIZED), got {t:?}"))),
        }
    }

    // ── v2.0: STATE ──────────────────────────────────────────────────────────

    fn maybe_parse_state(&mut self) -> Result<Option<StateClause>, ParseError> {
        if !matches!(self.peek(), Token::State) { return Ok(None); }
        self.advance();
        let mut states = vec![self.parse_state_value()?];
        while matches!(self.peek(), Token::Pipe) {
            self.advance();
            states.push(self.parse_state_value()?);
        }
        Ok(Some(StateClause { states }))
    }

    fn parse_state_value(&mut self) -> Result<LifecycleState, ParseError> {
        match self.advance() {
            Token::StateEmerging   => Ok(LifecycleState::Emerging),
            Token::StateStable     => Ok(LifecycleState::Stable),
            Token::StateCanonical  => Ok(LifecycleState::Canonical),
            Token::StateDeprecated => Ok(LifecycleState::Deprecated),
            t => Err(ParseError(format!("expected lifecycle state (EMERGING/STABLE/CANONICAL/DEPRECATED), got {t:?}"))),
        }
    }

    // ── v2.0: NASH ───────────────────────────────────────────────────────────

    fn maybe_parse_nash(&mut self) -> Result<Option<NashClause>, ParseError> {
        if !matches!(self.peek(), Token::Nash) { return Ok(None); }
        self.advance();
        match self.peek().clone() {
            Token::NashBreaking => {
                self.advance();
                Ok(Some(NashClause::Breaking))
            }
            Token::NashScore => {
                self.advance();
                let op = self.parse_op()?;
                let threshold = match self.advance() {
                    Token::Float(f) => f,
                    Token::Int(n)   => n as f64,
                    t => return Err(ParseError(format!("expected float threshold after NASH SCORE, got {t:?}"))),
                };
                Ok(Some(NashClause::Score { op, threshold }))
            }
            t => Err(ParseError(format!("expected SCORE or BREAKING after NASH, got {t:?}"))),
        }
    }

    // ── v2.0: PATTERN ────────────────────────────────────────────────────────

    fn parse_pattern_conditions(&mut self) -> Result<Vec<PatternCondition>, ParseError> {
        let mut conditions = vec![];
        while matches!(self.peek(), Token::Pattern) {
            self.advance();
            conditions.push(self.parse_one_pattern_condition()?);
        }
        Ok(conditions)
    }

    fn parse_one_pattern_condition(&mut self) -> Result<PatternCondition, ParseError> {
        match self.peek().clone() {
            Token::PatternType => {
                self.advance();
                let kind = self.parse_pattern_kind()?;
                Ok(PatternCondition::Type(kind))
            }
            Token::PatternConfidence => {
                self.advance();
                let op = self.parse_op()?;
                let v = match self.advance() {
                    Token::Float(f) => f,
                    Token::Int(n)   => n as f64,
                    t => return Err(ParseError(format!("expected float after PATTERN CONFIDENCE, got {t:?}"))),
                };
                Ok(PatternCondition::Confidence { op, value: v })
            }
            Token::PatternConstituents => {
                self.advance();
                let op = self.parse_op()?;
                let v = match self.advance() {
                    Token::Int(n) if n >= 0 => n as u32,
                    t => return Err(ParseError(format!("expected integer after PATTERN MAX_CONSTITUENTS, got {t:?}"))),
                };
                Ok(PatternCondition::MaxConstituents { op, value: v })
            }
            t => Err(ParseError(format!("expected TYPE/CONFIDENCE/MAX_CONSTITUENTS after PATTERN, got {t:?}"))),
        }
    }

    fn parse_pattern_kind(&mut self) -> Result<PatternKind, ParseError> {
        match self.advance() {
            Token::PtCrowdFlow          => Ok(PatternKind::CrowdFlow),
            Token::PtAviationCorridor   => Ok(PatternKind::AviationCorridor),
            Token::PtAviationHolding    => Ok(PatternKind::AviationHolding),
            Token::PtIndoorHallway      => Ok(PatternKind::IndoorHallway),
            Token::PtIndoorRoom         => Ok(PatternKind::IndoorRoom),
            Token::PtIndoorTransition   => Ok(PatternKind::IndoorTransition),
            Token::PtWaterFlow          => Ok(PatternKind::WaterFlow),
            Token::PtCustom             => Ok(PatternKind::Custom),
            t => Err(ParseError(format!("expected pattern type name (CrowdFlow/…), got {t:?}"))),
        }
    }

    // ── v2.0: LINEAGE ────────────────────────────────────────────────────────

    fn maybe_parse_lineage(&mut self) -> Result<Option<LineageClause>, ParseError> {
        if !matches!(self.peek(), Token::Lineage) { return Ok(None); }
        self.advance();
        match self.peek().clone() {
            Token::LineageFull => {
                self.advance();
                Ok(Some(LineageClause::Full))
            }
            Token::LineageDepth => {
                self.advance();
                let n = match self.advance() {
                    Token::Int(n) if n > 0 => n as u32,
                    t => return Err(ParseError(format!("expected depth integer after LINEAGE DEPTH, got {t:?}"))),
                };
                Ok(Some(LineageClause::Depth(n)))
            }
            t => Err(ParseError(format!("expected DEPTH or FULL after LINEAGE, got {t:?}"))),
        }
    }

    // ── v2.0: GATE ───────────────────────────────────────────────────────────

    fn maybe_parse_gate(&mut self) -> Result<Option<GateClause>, ParseError> {
        if !matches!(self.peek(), Token::Gate) { return Ok(None); }
        self.advance();
        if matches!(self.peek(), Token::NodeAll) {
            self.advance();
            return Ok(Some(GateClause { gates: vec![GateTarget::All] }));
        }
        let first_name = match self.advance() {
            Token::Ident(s) => s,
            t => return Err(ParseError(format!("expected gate name or ALL after GATE, got {t:?}"))),
        };
        let mut gates = vec![GateTarget::Named(first_name)];
        while matches!(self.peek(), Token::Comma) {
            self.advance();
            let name = match self.advance() {
                Token::Ident(s) => s,
                t => return Err(ParseError(format!("expected gate name, got {t:?}"))),
            };
            gates.push(GateTarget::Named(name));
        }
        Ok(Some(GateClause { gates }))
    }

    // ── v2.0: SATAMU ─────────────────────────────────────────────────────────

    fn maybe_parse_satamu(&mut self) -> Result<Option<SatamuClause>, ParseError> {
        if !matches!(self.peek(), Token::Satamu) { return Ok(None); }
        self.advance();
        match self.peek().clone() {
            Token::SatamuRequired => {
                self.advance();
                Ok(Some(SatamuClause::Required))
            }
            Token::SatamuBypass => {
                self.advance();
                let reason = match self.advance() {
                    Token::Str(s) => s,
                    t => return Err(ParseError(format!("expected string reason after SATAMU BYPASS, got {t:?}"))),
                };
                Ok(Some(SatamuClause::Bypass(reason)))
            }
            t => Err(ParseError(format!("expected REQUIRED or BYPASS after SATAMU, got {t:?}"))),
        }
    }

    // ── v1: HOW ──────────────────────────────────────────────────────────────

    fn maybe_parse_how(&mut self) -> Result<Option<HowClause>, ParseError> {
        if !matches!(self.peek(), Token::How) { return Ok(None); }
        self.advance();
        self.expect(&Token::By)?;
        let var = match self.advance() {
            Token::Ident(s) => s,
            t => return Err(ParseError(format!("expected variable in HOW BY, got {t:?}"))),
        };
        self.expect(&Token::LBracket)?;
        let attr = self.parse_dotted_name()?;
        self.expect(&Token::RBracket)?;
        let dir = self.parse_sort_dir();
        Ok(Some(HowClause { var, attr, dir }))
    }

    fn parse_sort_dir(&mut self) -> SortDir {
        match self.peek() {
            Token::Desc => { self.advance(); SortDir::Desc }
            Token::Asc  => { self.advance(); SortDir::Asc  }
            _           => SortDir::Asc,
        }
    }

    // ── v1: HOW_MUCH ─────────────────────────────────────────────────────────

    fn maybe_parse_how_much(&mut self) -> Result<Option<HowMuchClause>, ParseError> {
        if !matches!(self.peek(), Token::HowMuch) { return Ok(None); }
        self.advance();
        match self.peek().clone() {
            Token::Limit => {
                self.advance();
                let n = match self.advance() {
                    Token::Int(n) if n > 0 => n as usize,
                    t => return Err(ParseError(format!("expected positive integer for LIMIT, got {t:?}"))),
                };
                Ok(Some(HowMuchClause::Limit(n)))
            }
            Token::Top => {
                self.advance();
                let n = match self.advance() {
                    Token::Int(n) if n > 0 => n as usize,
                    t => return Err(ParseError(format!("expected positive integer for TOP, got {t:?}"))),
                };
                self.expect(&Token::By)?;
                let var = match self.advance() {
                    Token::Ident(s) => s,
                    t => return Err(ParseError(format!("expected variable in TOP BY, got {t:?}"))),
                };
                self.expect(&Token::LBracket)?;
                let attr = self.parse_dotted_name()?;
                self.expect(&Token::RBracket)?;
                let dir = self.parse_sort_dir();
                Ok(Some(HowMuchClause::Top { n, var, attr, dir }))
            }
            t => Err(ParseError(format!("expected LIMIT or TOP after HOW_MUCH, got {t:?}"))),
        }
    }

    // ── v2.1: MEASURE ────────────────────────────────────────────────────────

    fn maybe_parse_measure(&mut self) -> Result<Option<MeasureClause>, ParseError> {
        if !matches!(self.peek(), Token::Measure) { return Ok(None); }
        self.advance();
        match self.peek().clone() {
            Token::Dense => {
                self.advance();
                Ok(Some(MeasureClause::Dense))
            }
            Token::Flux => {
                self.advance();
                let (var, attr) = self.parse_var_attr_bracket()?;
                Ok(Some(MeasureClause::Flux { var, attr }))
            }
            Token::RotorMean => {
                self.advance();
                let (var, attr) = self.parse_var_attr_bracket()?;
                Ok(Some(MeasureClause::RotorMean { var, attr }))
            }
            t => Err(ParseError(format!("expected DENSE, FLUX, or ROTOR_MEAN after MEASURE, got {t:?}"))),
        }
    }

    // ── v2.1: GRAVITY ────────────────────────────────────────────────────────

    fn maybe_parse_gravity(&mut self) -> Result<Option<GravityClause>, ParseError> {
        if !matches!(self.peek(), Token::Gravity) { return Ok(None); }
        self.advance();
        let (var, attr) = self.parse_var_attr_bracket()?;
        let mode = match self.peek().clone() {
            Token::Band => {
                self.advance();
                let width = match self.advance() {
                    Token::Float(f) => f,
                    Token::Int(n)   => n as f64,
                    t => return Err(ParseError(format!("expected numeric width after GRAVITY ... BAND, got {t:?}"))),
                };
                if width <= 0.0 {
                    return Err(ParseError(format!("GRAVITY BAND width must be positive, got {width}")));
                }
                GravityMode::Band(width)
            }
            Token::MaxGroups => {
                self.advance();
                let n = match self.advance() {
                    Token::Int(n) if n > 0 => n as usize,
                    t => return Err(ParseError(format!("expected positive integer for MAX_GROUPS, got {t:?}"))),
                };
                GravityMode::ExactMatchCapped(n)
            }
            // No bound mode given at all -- refused here, not just at Gate
            // G4: exact-match grouping with no cap risks the same O(N)
            // grouping-structure growth as an unbounded SQL GROUP BY on a
            // high-cardinality column. GRAVITY always requires BAND or
            // MAX_GROUPS explicitly; there is no unbounded default.
            t => return Err(ParseError(format!(
                "GRAVITY requires BAND <width> or MAX_GROUPS <n> -- exact-match grouping with no cap can grow \
                 its grouping structure toward O(N) memory on a high-cardinality attribute. got {t:?}"
            ))),
        };
        Ok(Some(GravityClause { var, attr, mode }))
    }

    /// Shared `VarName[dotted.attr]` parser, used by MEASURE FLUX/ROTOR_MEAN
    /// and GRAVITY -- the same attribute-reference shape HOW_MUCH TOP already
    /// uses for its `BY VarName[attr]`.
    fn parse_var_attr_bracket(&mut self) -> Result<(String, String), ParseError> {
        let var = match self.advance() {
            Token::Ident(s) => s,
            t => return Err(ParseError(format!("expected variable name, got {t:?}"))),
        };
        self.expect(&Token::LBracket)?;
        let attr = self.parse_dotted_name()?;
        self.expect(&Token::RBracket)?;
        Ok((var, attr))
    }

    // ── Production execution hints ────────────────────────────────────────────

    fn maybe_parse_anchor(&mut self) -> Result<AnchorStrategy, ParseError> {
        if !matches!(self.peek(), Token::Anchor) { return Ok(AnchorStrategy::Auto); }
        self.advance();
        match self.advance() {
            Token::AnchorAuto          => Ok(AnchorStrategy::Auto),
            Token::AnchorSurrogateTime => Ok(AnchorStrategy::SurrogateTime),
            Token::AnchorStateStation  => Ok(AnchorStrategy::StateStation),
            Token::AnchorE7First       => Ok(AnchorStrategy::E7First),
            Token::AnchorFullScan      => Ok(AnchorStrategy::FullScan),
            t => Err(ParseError(format!(
                "expected anchor strategy (AUTO/SURROGATE_TIME/STATE_STATION/E7_FIRST/FULL_SCAN), got {t:?}"
            ))),
        }
    }

    fn maybe_parse_stream(&mut self) -> bool {
        if matches!(self.peek(), Token::Stream) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn maybe_parse_derive_station(&mut self) -> Result<Option<String>, ParseError> {
        if !matches!(self.peek(), Token::DeriveStation) { return Ok(None); }
        self.advance();
        match self.advance() {
            Token::Ident(s) | Token::Str(s) => Ok(Some(s)),
            t => Err(ParseError(format!("expected station name after DERIVE_STATION, got {t:?}"))),
        }
    }

    fn maybe_parse_abort_scan(&mut self) -> Result<Option<u64>, ParseError> {
        if !matches!(self.peek(), Token::AbortScan) { return Ok(None); }
        self.advance();
        match self.advance() {
            Token::Int(n) if n > 0 => Ok(Some(n as u64)),
            t => Err(ParseError(format!("expected positive integer for ABORT_SCAN, got {t:?}"))),
        }
    }

    fn parse_filter_order(&mut self) -> Result<Vec<FilterStage>, ParseError> {
        if !matches!(self.peek(), Token::FilterOrder) { return Ok(Vec::new()); }
        self.advance();
        let mut stages = Vec::new();
        loop {
            let stage = match self.peek() {
                Token::FsSurrogateRange => { self.advance(); FilterStage::SurrogateRange }
                Token::FsOrbitalRange   => { self.advance(); FilterStage::OrbitalRange }
                Token::FsState          => { self.advance(); FilterStage::State }
                Token::FsDeriveStation  => { self.advance(); FilterStage::DeriveStation }
                Token::FsLane           => { self.advance(); FilterStage::Lane }
                Token::FsQualityByte    => { self.advance(); FilterStage::QualityByte }
                Token::FsEavAttr        => { self.advance(); FilterStage::EavAttr }
                Token::FsE7Region       => { self.advance(); FilterStage::E7Region }
                _ => break,
            };
            stages.push(stage);
            // allow optional comma separator between stages
            if matches!(self.peek(), Token::Comma) { self.advance(); }
        }
        if stages.is_empty() {
            return Err(ParseError("FILTER_ORDER requires at least one stage".into()));
        }
        Ok(stages)
    }

    // ── Shared helpers ────────────────────────────────────────────────────────

    fn parse_op(&mut self) -> Result<Op, ParseError> {
        match self.advance() {
            Token::Eq  => Ok(Op::Eq),
            Token::Neq => Ok(Op::Neq),
            Token::Gt  => Ok(Op::Gt),
            Token::Lt  => Ok(Op::Lt),
            Token::Gte => Ok(Op::Gte),
            Token::Lte => Ok(Op::Lte),
            t => Err(ParseError(format!("expected comparison operator (=, !=, >, …), got {t:?}"))),
        }
    }

    fn parse_value(&mut self) -> Result<HeptaValue, ParseError> {
        match self.advance() {
            Token::Str(s)   => Ok(HeptaValue::Text(s)),
            Token::Int(n)   => Ok(HeptaValue::Int(n)),
            Token::Float(f) => Ok(HeptaValue::Float(f)),
            Token::Ident(s) => match s.to_lowercase().as_str() {
                "true"  => Ok(HeptaValue::Bool(true)),
                "false" => Ok(HeptaValue::Bool(false)),
                "null"  => Ok(HeptaValue::Null),
                _ => Err(ParseError(format!("unknown value identifier: {s}"))),
            },
            t => Err(ParseError(format!("expected value literal (string/int/float/bool), got {t:?}"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── v1 regression tests ───────────────────────────────────────────────────

    #[test]
    fn parse_minimal_who() {
        let q = parse_query("WHO Citizens.E").unwrap();
        assert_eq!(q.who.primary.tribe, "Citizens");
        assert_eq!(q.who.primary.var,   "E");
        assert!(q.who.bound_to.is_empty());
        assert!(q.node.is_none());
        assert_eq!(q.verb, QueryVerb::Orbit, "no verb keyword must default to ORBIT");
    }

    #[test]
    fn parse_sovereign_verbs_lead_the_query() {
        assert_eq!(parse_query("ORBIT\nWHO T.E").unwrap().verb, QueryVerb::Orbit);
        assert_eq!(parse_query("EMIT\nWHO T.E").unwrap().verb, QueryVerb::Emit);
        assert_eq!(parse_query("PROVE\nWHO T.E").unwrap().verb, QueryVerb::Prove);
        assert_eq!(parse_query("SYNC\nWHO T.E").unwrap().verb, QueryVerb::Sync);
        assert_eq!(parse_query("WITNESS\nWHO T.E").unwrap().verb, QueryVerb::Witness);
    }

    #[test]
    fn verb_precedes_node_and_across() {
        let q = parse_query("PROVE\nNODE EnkiDDB\nWHO T.E").unwrap();
        assert_eq!(q.verb, QueryVerb::Prove);
        assert_eq!(q.node.unwrap().targets, vec![DbTarget::EnkiDdb]);
    }

    #[test]
    fn parse_who_bound_to() {
        let q = parse_query("WHO Graves.G BOUND_TO Citizens.C").unwrap();
        assert_eq!(q.who.primary.var,   "G");
        assert_eq!(q.who.bound_to.len(), 1);
        assert_eq!(q.who.bound_to[0].tribe, "Citizens");
    }

    #[test]
    fn parse_what_specific_attrs() {
        let q = parse_query("WHO T.E\nWHAT E[name.given, birth.year]").unwrap();
        let what = q.what.unwrap();
        assert_eq!(what.projections[0].attrs, vec!["name.given", "birth.year"]);
    }

    #[test]
    fn parse_what_star() {
        let q = parse_query("WHO T.E\nWHAT E[*]").unwrap();
        assert!(q.what.unwrap().projections[0].attrs.is_empty());
    }

    #[test]
    fn parse_where_string_eq() {
        let q = parse_query("WHO T.E\nWHERE E[city.name] = \"Najaf\"").unwrap();
        let cond = &q.r#where[0];
        assert_eq!(cond.attr, "city.name");
        assert_eq!(cond.test, ConditionTest::Cmp { op: Op::Eq, val: HeptaValue::Text("Najaf".into()) });
    }

    #[test]
    fn parse_where_exists() {
        let q = parse_query("WHO T.E\nWHERE E[phone] EXISTS").unwrap();
        assert_eq!(q.r#where[0].test, ConditionTest::Exists);
    }

    #[test]
    fn parse_where_not_exists() {
        let q = parse_query("WHO T.E\nWHERE E[phone] NOT EXISTS").unwrap();
        assert_eq!(q.r#where[0].test, ConditionTest::NotExists);
    }

    #[test]
    fn parse_when_at_epoch_now() {
        let q = parse_query("WHO T.E\nWHEN AT EPOCH NOW").unwrap();
        assert_eq!(q.when, Some(WhenClause::AtEpoch(EpochRef::Now)));
    }

    #[test]
    fn parse_when_before_epoch() {
        let q = parse_query("WHO T.E\nWHEN BEFORE EPOCH 1680000000").unwrap();
        assert_eq!(q.when, Some(WhenClause::BeforeEpoch(1_680_000_000)));
    }

    #[test]
    fn parse_why_quality_byte() {
        let q = parse_query("WHO T.E\nWHY QUALITY_BYTE < 100").unwrap();
        match &q.why[0] {
            WhyCondition::QualityByte { op, val, .. } => {
                assert_eq!(*op, Op::Lt);
                assert_eq!(*val, 100u8);
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn parse_why_lane() {
        let q = parse_query("WHO T.E\nWHY LANE != Black").unwrap();
        match &q.why[0] {
            WhyCondition::Lane { op, val, .. } => {
                assert_eq!(*op, Op::Neq);
                assert_eq!(*val, LaneValue::Black);
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn parse_how_by_desc() {
        let q = parse_query("WHO T.E\nHOW BY E[score] DESC").unwrap();
        let how = q.how.unwrap();
        assert_eq!(how.attr, "score");
        assert_eq!(how.dir, SortDir::Desc);
    }

    #[test]
    fn parse_how_much_limit() {
        let q = parse_query("WHO T.E\nHOW_MUCH LIMIT 100").unwrap();
        assert_eq!(q.how_much, Some(HowMuchClause::Limit(100)));
    }

    #[test]
    fn parse_how_much_top() {
        let q = parse_query("WHO T.E\nHOW_MUCH TOP 5 BY E[score] DESC").unwrap();
        match q.how_much.unwrap() {
            HowMuchClause::Top { n, attr, dir, .. } => {
                assert_eq!(n, 5);
                assert_eq!(attr, "score");
                assert_eq!(dir, SortDir::Desc);
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    // ── v2.1 MEASURE / GRAVITY tests ─────────────────────────────────────────

    #[test]
    fn parse_measure_dense() {
        let q = parse_query("WHO T.E\nMEASURE DENSE").unwrap();
        assert_eq!(q.measure, Some(MeasureClause::Dense));
    }

    #[test]
    fn parse_measure_flux() {
        let q = parse_query("WHO T.E\nMEASURE FLUX E[pressure]").unwrap();
        match q.measure.unwrap() {
            MeasureClause::Flux { var, attr } => {
                assert_eq!(var, "E");
                assert_eq!(attr, "pressure");
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn parse_measure_rotor_mean() {
        let q = parse_query("WHO T.E\nMEASURE ROTOR_MEAN E[orientation]").unwrap();
        match q.measure.unwrap() {
            MeasureClause::RotorMean { var, attr } => {
                assert_eq!(var, "E");
                assert_eq!(attr, "orientation");
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn parse_gravity_band() {
        let q = parse_query("WHO T.E\nGRAVITY E[pressure] BAND 5.0").unwrap();
        let g = q.gravity.unwrap();
        assert_eq!(g.attr, "pressure");
        assert_eq!(g.mode, GravityMode::Band(5.0));
    }

    #[test]
    fn parse_gravity_max_groups() {
        let q = parse_query("WHO T.E\nGRAVITY E[station] MAX_GROUPS 1000").unwrap();
        let g = q.gravity.unwrap();
        assert_eq!(g.attr, "station");
        assert_eq!(g.mode, GravityMode::ExactMatchCapped(1000));
    }

    /// The real safety property: GRAVITY with no BAND/MAX_GROUPS bound is a
    /// parse error, not a silently-unbounded default. This is what stands
    /// between an authoring mistake and an O(N)-memory grouping structure
    /// on a billion-particle Tribe.
    #[test]
    fn parse_gravity_without_bound_is_refused() {
        let err = parse_query("WHO T.E\nGRAVITY E[station]").unwrap_err();
        assert!(err.0.contains("BAND"), "{}", err.0);
        assert!(err.0.contains("MAX_GROUPS"), "{}", err.0);
    }

    #[test]
    fn parse_gravity_band_rejects_nonpositive_width() {
        let err = parse_query("WHO T.E\nGRAVITY E[pressure] BAND 0").unwrap_err();
        assert!(err.0.contains("positive"), "{}", err.0);
    }

    #[test]
    fn parse_measure_and_gravity_combine() {
        // The real GROUP-BY+aggregate shape: count particles per station.
        let q = parse_query(
            "WHO T.E\nGRAVITY E[station] MAX_GROUPS 1000\nMEASURE DENSE"
        ).unwrap();
        assert!(q.gravity.is_some());
        assert_eq!(q.measure, Some(MeasureClause::Dense));
    }

    // ── v2.0 new clause tests ─────────────────────────────────────────────────

    #[test]
    fn parse_node_single() {
        let q = parse_query("NODE EnkiMDB\nWHO T.E").unwrap();
        let node = q.node.unwrap();
        assert_eq!(node.targets, vec![DbTarget::EnkiMdb]);
    }

    #[test]
    fn parse_node_all() {
        let q = parse_query("NODE ALL\nWHO T.E").unwrap();
        assert_eq!(q.node.unwrap().targets, vec![DbTarget::All]);
    }

    #[test]
    fn parse_node_multi_pipe() {
        let q = parse_query("NODE EnkiDB | EnkiDW | NARUDU\nWHO T.E").unwrap();
        let targets = q.node.unwrap().targets;
        assert_eq!(targets.len(), 3);
        assert_eq!(targets[0], DbTarget::EnkiDb);
        assert_eq!(targets[1], DbTarget::EnkiDw);
        assert_eq!(targets[2], DbTarget::Narudu);
    }

    #[test]
    fn parse_across_bigring() {
        let q = parse_query("ACROSS BIGRING ClientAlpha\nWHO T.E").unwrap();
        assert_eq!(q.across, Some(AcrossClause::Bigring("ClientAlpha".into())));
    }

    #[test]
    fn parse_across_all() {
        let q = parse_query("ACROSS ALL\nWHO T.E").unwrap();
        assert_eq!(q.across, Some(AcrossClause::All));
    }

    #[test]
    fn parse_tier_single() {
        let q = parse_query("WHO T.E\nTIER HOT").unwrap();
        assert_eq!(q.tier.unwrap().tiers, vec![StorageTier::Hot]);
    }

    #[test]
    fn parse_tier_multi() {
        let q = parse_query("WHO T.E\nTIER HOT | WARM | COLD").unwrap();
        let tiers = q.tier.unwrap().tiers;
        assert_eq!(tiers, vec![StorageTier::Hot, StorageTier::Warm, StorageTier::Cold]);
    }

    #[test]
    fn parse_state_canonical() {
        let q = parse_query("WHO T.E\nSTATE CANONICAL").unwrap();
        assert_eq!(q.state.unwrap().states, vec![LifecycleState::Canonical]);
    }

    #[test]
    fn parse_state_multi() {
        let q = parse_query("WHO T.E\nSTATE EMERGING | STABLE").unwrap();
        let states = q.state.unwrap().states;
        assert_eq!(states, vec![LifecycleState::Emerging, LifecycleState::Stable]);
    }

    #[test]
    fn parse_nash_score() {
        let q = parse_query("WHO T.E\nNASH SCORE < 0.20").unwrap();
        match q.nash.unwrap() {
            NashClause::Score { op, threshold } => {
                assert_eq!(op, Op::Lt);
                assert!((threshold - 0.20).abs() < 1e-9);
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn parse_nash_breaking() {
        let q = parse_query("WHO T.E\nNASH BREAKING").unwrap();
        assert_eq!(q.nash, Some(NashClause::Breaking));
    }

    #[test]
    fn parse_pattern_type() {
        let q = parse_query("WHO T.E\nPATTERN TYPE CrowdFlow").unwrap();
        assert_eq!(q.pattern.len(), 1);
        assert_eq!(q.pattern[0], PatternCondition::Type(PatternKind::CrowdFlow));
    }

    #[test]
    fn parse_pattern_confidence() {
        let q = parse_query("WHO T.E\nPATTERN CONFIDENCE >= 0.85").unwrap();
        match &q.pattern[0] {
            PatternCondition::Confidence { op, value } => {
                assert_eq!(*op, Op::Gte);
                assert!((*value - 0.85).abs() < 1e-9);
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn parse_pattern_multi_conditions() {
        let src = "WHO T.E\nPATTERN TYPE IndoorRoom\nPATTERN CONFIDENCE >= 0.60\nPATTERN MAX_CONSTITUENTS <= 126";
        let q = parse_query(src).unwrap();
        assert_eq!(q.pattern.len(), 3);
        assert_eq!(q.pattern[0], PatternCondition::Type(PatternKind::IndoorRoom));
    }

    #[test]
    fn parse_lineage_depth() {
        let q = parse_query("WHO T.E\nLINEAGE DEPTH 5").unwrap();
        assert_eq!(q.lineage, Some(LineageClause::Depth(5)));
    }

    #[test]
    fn parse_lineage_full() {
        let q = parse_query("WHO T.E\nLINEAGE FULL").unwrap();
        assert_eq!(q.lineage, Some(LineageClause::Full));
    }

    #[test]
    fn parse_gate_named() {
        let q = parse_query("WHO T.E\nGATE DataSteward").unwrap();
        assert_eq!(q.gate.unwrap().gates, vec![GateTarget::Named("DataSteward".into())]);
    }

    #[test]
    fn parse_gate_multi_named() {
        let q = parse_query("WHO T.E\nGATE DataSteward, DataCleansing").unwrap();
        let gates = q.gate.unwrap().gates;
        assert_eq!(gates.len(), 2);
        assert_eq!(gates[0], GateTarget::Named("DataSteward".into()));
        assert_eq!(gates[1], GateTarget::Named("DataCleansing".into()));
    }

    #[test]
    fn parse_gate_all() {
        let q = parse_query("WHO T.E\nGATE ALL").unwrap();
        assert_eq!(q.gate.unwrap().gates, vec![GateTarget::All]);
    }

    #[test]
    fn parse_satamu_required() {
        let q = parse_query("WHO T.E\nSATAMU REQUIRED").unwrap();
        assert_eq!(q.satamu, Some(SatamuClause::Required));
    }

    #[test]
    fn parse_satamu_bypass() {
        let q = parse_query(r#"WHO T.E
SATAMU BYPASS "AI Council unanimous approval 2026-06-30""#).unwrap();
        match q.satamu.unwrap() {
            SatamuClause::Bypass(reason) => {
                assert!(reason.contains("AI Council"));
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn parse_orbital_range() {
        let q = parse_query("WHO T.E\nORBITAL 100 .. 500").unwrap();
        assert_eq!(q.orbital, Some(OrbitalClause::Range { start: 100, end: 500 }));
    }

    #[test]
    fn parse_orbital_now() {
        let q = parse_query("WHO T.E\nORBITAL NOW").unwrap();
        assert_eq!(q.orbital, Some(OrbitalClause::Now));
    }

    #[test]
    fn parse_full_v2_query() {
        let src = "NODE EnkiMDB | ENKI_PATTERN\n\
                   ACROSS BIGRING ClientAlpha\n\
                   WHO Tribes.E\n\
                   WHAT E[event.type, location.zone]\n\
                   WHERE E[event.type] = \"entry\"\n\
                   WHEN AFTER EPOCH 1680000000\n\
                   ORBITAL 100 .. 500\n\
                   WHY LANE != Black\n\
                   TIER HOT | WARM\n\
                   STATE CANONICAL\n\
                   NASH SCORE < 0.20\n\
                   PATTERN TYPE CrowdFlow\n\
                   PATTERN CONFIDENCE >= 0.85\n\
                   LINEAGE DEPTH 3\n\
                   GATE DataSteward, DataCleansing\n\
                   SATAMU REQUIRED\n\
                   HOW BY E[location.zone] ASC\n\
                   HOW_MUCH LIMIT 1000";
        let q = parse_query(src).unwrap();

        // v2.0 clauses all populated
        assert_eq!(q.node.as_ref().unwrap().targets.len(), 2);
        assert_eq!(q.across, Some(AcrossClause::Bigring("ClientAlpha".into())));
        assert_eq!(q.tier.as_ref().unwrap().tiers.len(), 2);
        assert_eq!(q.state.as_ref().unwrap().states, vec![LifecycleState::Canonical]);
        assert_eq!(q.nash, Some(NashClause::Score { op: Op::Lt, threshold: 0.20 }));
        assert_eq!(q.pattern.len(), 2);
        assert_eq!(q.lineage, Some(LineageClause::Depth(3)));
        assert_eq!(q.gate.as_ref().unwrap().gates.len(), 2);
        assert_eq!(q.satamu, Some(SatamuClause::Required));
        assert_eq!(q.orbital, Some(OrbitalClause::Range { start: 100, end: 500 }));

        // v1 clauses still work
        assert_eq!(q.who.primary.tribe, "Tribes");
        assert!(q.what.is_some());
        assert_eq!(q.r#where.len(), 1);
        assert_eq!(q.how_much, Some(HowMuchClause::Limit(1000)));
    }

    #[test]
    fn db_target_port_numbers() {
        assert_eq!(DbTarget::EnkiDb.port(),  Some(7001));
        assert_eq!(DbTarget::EnkiDw.port(),  Some(7002));
        assert_eq!(DbTarget::EnkiMdb.port(), Some(7006));
        assert_eq!(DbTarget::EnkiDdb.port(), Some(7007));
        assert_eq!(DbTarget::Narudu.port(),  None);
        assert_eq!(DbTarget::All.port(),     None);
    }
}
