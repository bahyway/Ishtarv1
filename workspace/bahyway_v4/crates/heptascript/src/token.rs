//! HeptaScript v2.0 Lexer — W5H2 sovereign query language for the EAV particle space.
//!
//! # v1 clauses (unchanged):
//!   WHO   WHAT   WHERE   WHEN   WHY   HOW   HOW_MUCH   ROUTE
//!
//! # v2.0 new clauses (cross-database, pattern intelligence, federation):
//!
//!   NODE    — target EnkiDB type(s): EnkiDB | EnkiDW | EnkiMDB | NARUDU | ENKI_PATTERN | ALL
//!   ACROSS  — cross-BIGRING federation: ACROSS BIGRING ClientX | ACROSS ALL
//!   TIER    — storage tier filter: HOT | WARM | COLD | CRYSTALLIZED
//!   STATE   — pattern / particle lifecycle: EMERGING | STABLE | CANONICAL | DEPRECATED
//!   NASH    — Nash equilibrium filter: NASH SCORE < 0.20 | NASH BREAKING
//!   PATTERN — query ENKI-PATTERN registry: TYPE CrowdFlow | CONFIDENCE >= 0.85
//!   LINEAGE — causality traversal: LINEAGE DEPTH 3 | LINEAGE FULL
//!   GATE    — pass through an ETL station: GATE DataSteward | GATE ALL
//!   SATAMU  — governance override: SATAMU REQUIRED | SATAMU BYPASS "reason"
//!   ORBITAL — orbital time range: ORBITAL 100 .. 200 | ORBITAL NOW

/// Lexical token produced by the HeptaScript lexer.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // ── W5H2 clause keywords (v1) ────────────────────────────────────────────
    Who,
    What,
    Where,
    When,
    Why,
    How,
    HowMuch,

    // ── Sovereign Operation verbs (optional, precede everything else) ────────
    // See operations.rs for the conceptual five; only these are parseable.
    VerbOrbit,   // ORBIT   — read/scan (default when no verb keyword given)
    VerbEmit,    // EMIT    — read filtered to birth-event particles
    VerbProve,   // PROVE   — read with full per-epoch history projection
    VerbSync,    // SYNC    — read plus an order-independent state fingerprint
    VerbWitness, // WITNESS — read plus a SHA3-256 content digest

    // ── v2.0 clause keywords ─────────────────────────────────────────────────
    Node,    // NODE clause — target database type(s)
    Across,  // ACROSS clause — cross-BIGRING federation
    Tier,    // TIER clause — storage tier filter
    State,   // STATE clause — lifecycle filter
    Nash,    // NASH clause — Nash equilibrium filter
    Pattern, // PATTERN clause — ENKI-PATTERN registry query
    Lineage, // LINEAGE clause — causality depth
    Gate,    // GATE clause — ETL station routing
    Satamu,  // SATAMU clause — governance override

    // ── v2.0 NODE targets ────────────────────────────────────────────────────
    NodeAll,       // ALL
    DbEnkiDb,      // EnkiDB   (port 7001 — operational OLTP)
    DbEnkiDw,      // EnkiDW   (port 7002 — data warehouse)
    DbEnkiSdb,     // EnkiSDB  (port 7003 — security)
    DbEnkiOdb,     // EnkiODB  (port 7004 — operational between-gate)
    DbEnkiQdb,     // EnkiQDB  (port 7005 — quantum probability)
    DbEnkiMdb,     // EnkiMDB  (port 7006 — master data)
    DbEnkiDdb,     // EnkiDDB  (port 7007 — client documentation)
    DbNarudu,      // NARUDU   (pattern event bus)
    DbEnkiPattern, // ENKI_PATTERN (pattern registry)
    Pipe,          // | separator for multiple targets

    // ── v2.0 ACROSS sub-keywords ─────────────────────────────────────────────
    Bigring, // BIGRING keyword in ACROSS clause

    // ── v2.0 TIER values ─────────────────────────────────────────────────────
    TierHot,          // HOT
    TierWarm,         // WARM
    TierCold,         // COLD
    TierCrystallized, // CRYSTALLIZED

    // ── v2.0 STATE values (pattern / particle lifecycle) ─────────────────────
    StateEmerging,   // EMERGING   (confidence ≥ 0.60)
    StateStable,     // STABLE     (confidence ≥ 0.85)
    StateCanonical,  // CANONICAL
    StateDeprecated, // DEPRECATED

    // ── v2.0 NASH sub-keywords ───────────────────────────────────────────────
    NashScore,    // SCORE
    NashBreaking, // BREAKING

    // ── v2.0 PATTERN sub-keywords ────────────────────────────────────────────
    PatternType,         // TYPE
    PatternConfidence,   // CONFIDENCE
    PatternConstituents, // MAX_CONSTITUENTS

    // Pattern type names
    PtCrowdFlow,        // CrowdFlow
    PtAviationCorridor, // AviationCorridor
    PtAviationHolding,  // AviationHolding
    PtIndoorHallway,    // IndoorHallway
    PtIndoorRoom,       // IndoorRoom
    PtIndoorTransition, // IndoorTransition
    PtWaterFlow,        // WaterFlow
    PtCustom,           // Custom

    // ── v2.0 LINEAGE sub-keywords ────────────────────────────────────────────
    LineageDepth, // DEPTH
    LineageFull,  // FULL

    // ── v2.0 SATAMU sub-keywords ─────────────────────────────────────────────
    SatamuRequired, // REQUIRED
    SatamuBypass,   // BYPASS

    // ── v2.0 ORBITAL sub-keyword ─────────────────────────────────────────────
    Orbital, // ORBITAL keyword
    DotDot,  // .. range operator

    // ── Production execution hint keywords (1B+ particles, <1 sec) ───────────
    Anchor,              // ANCHOR clause keyword
    AnchorAuto,          // AUTO (default anchor strategy)
    AnchorSurrogateTime, // SURROGATE_TIME
    AnchorStateStation,  // STATE_STATION
    AnchorE7First,       // E7_FIRST
    AnchorFullScan,      // FULL_SCAN
    Stream,              // STREAM — emit results via callback
    DeriveStation,       // DERIVE_STATION — station derived from journal
    AbortScan,           // ABORT_SCAN N — safety valve
    FilterOrder,         // FILTER_ORDER — explicit predicate pipeline keyword
    // FilterStage tokens (values after FILTER_ORDER)
    FsSurrogateRange, // SURROGATE_RANGE
    FsOrbitalRange,   // ORBITAL_RANGE
    FsState,          // STATE_FILTER (distinct from STATE clause)
    FsDeriveStation,  // DERIVE_STATION_FILTER
    FsLane,           // LANE_FILTER
    FsQualityByte,    // QUALITY_BYTE_FILTER
    FsEavAttr,        // EAV_ATTR
    FsE7Region,       // E7_REGION

    // ── WHO sub-keywords (v1) ────────────────────────────────────────────────
    BoundTo,

    // ── WHEN sub-keywords (v1) ───────────────────────────────────────────────
    At,
    Epoch,
    Before,
    After,
    Now,

    // ── WHY sub-keywords (v1) ────────────────────────────────────────────────
    Lane,
    QualityByte,

    // ── HOW / HOW_MUCH sub-keywords (v1) ────────────────────────────────────
    By,
    Asc,
    Desc,
    Limit,
    Top,

    // ── v2.1 MEASURE / GRAVITY (Anti-SQL aggregate clauses) ───────────────────
    // Not the fifth-plus sovereign verb (Operation is a closed set of exactly
    // five: ORBIT/EMIT/PROVE/SYNC/WITNESS) -- these are ordinary clauses,
    // same footing as HOW_MUCH, that combine with any of the five.
    Measure,   // MEASURE clause keyword
    Dense,     // DENSE      -- real O(N) tally (COUNT-equivalent)
    Flux,      // FLUX       -- Multivector::add fold (SUM-equivalent)
    RotorMean, // ROTOR_MEAN -- circular mean / arithmetic mean (AVG-equivalent)
    Gravity,   // GRAVITY clause keyword -- partition (GROUP BY-equivalent)
    Band,      // BAND width -- numeric bucket partition mode
    MaxGroups, // MAX_GROUPS cap -- required safety bound for exact-match mode

    // ── Logical ──────────────────────────────────────────────────────────────
    And,
    Or,
    Not,
    Exists,

    // ── Comparison operators ─────────────────────────────────────────────────
    Eq,  // =
    Neq, // !=
    Gt,  // >
    Lt,  // <
    Gte, // >=
    Lte, // <=

    // ── Punctuation ──────────────────────────────────────────────────────────
    Dot,      // .
    Comma,    // ,
    Star,     // *
    LBracket, // [
    RBracket, // ]

    // ── NaviMap / modular-index queries (ROUTE — retained) ───────────────────
    Route,
    Index,
    Modular,
    AttrWeight,
    AttrLevel,
    AttrSignature,
    AttrTotalMass,
    AttrPeakCost,

    // ── Literals ─────────────────────────────────────────────────────────────
    Ident(String),
    Str(String),
    Int(i64),
    Float(f64),

    Eof,
}

/// Tokenize a HeptaScript source string.
pub fn tokenize(source: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = source.chars().collect();
    let mut i = 0;
    let n = chars.len();

    while i < n {
        let ch = chars[i];

        // Whitespace
        if ch.is_whitespace() {
            i += 1;
            continue;
        }

        // Comments: -- to end of line
        if ch == '-' && i + 1 < n && chars[i + 1] == '-' {
            while i < n && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }

        // Negative integer: -<digits>
        if ch == '-' && i + 1 < n && chars[i + 1].is_ascii_digit() {
            i += 1;
            let start = i;
            while i < n && chars[i].is_ascii_digit() {
                i += 1;
            }
            let num_str: String = chars[start..i].iter().collect();
            let v: i64 = num_str
                .parse::<i64>()
                .map(|v| -v)
                .map_err(|_| format!("Bad negative integer: -{num_str}"))?;
            tokens.push(Token::Int(v));
            continue;
        }

        // Two-char operators
        if ch == '!' && i + 1 < n && chars[i + 1] == '=' {
            tokens.push(Token::Neq);
            i += 2;
            continue;
        }
        if ch == '>' && i + 1 < n && chars[i + 1] == '=' {
            tokens.push(Token::Gte);
            i += 2;
            continue;
        }
        if ch == '<' && i + 1 < n && chars[i + 1] == '=' {
            tokens.push(Token::Lte);
            i += 2;
            continue;
        }
        // .. range operator (must come before single '.')
        if ch == '.' && i + 1 < n && chars[i + 1] == '.' {
            tokens.push(Token::DotDot);
            i += 2;
            continue;
        }

        // | pipe operator
        if ch == '|' {
            tokens.push(Token::Pipe);
            i += 1;
            continue;
        }

        // Single-char operators and punctuation
        match ch {
            '=' => {
                tokens.push(Token::Eq);
                i += 1;
                continue;
            }
            '>' => {
                tokens.push(Token::Gt);
                i += 1;
                continue;
            }
            '<' => {
                tokens.push(Token::Lt);
                i += 1;
                continue;
            }
            '.' => {
                tokens.push(Token::Dot);
                i += 1;
                continue;
            }
            ',' => {
                tokens.push(Token::Comma);
                i += 1;
                continue;
            }
            '*' => {
                tokens.push(Token::Star);
                i += 1;
                continue;
            }
            '[' => {
                tokens.push(Token::LBracket);
                i += 1;
                continue;
            }
            ']' => {
                tokens.push(Token::RBracket);
                i += 1;
                continue;
            }
            _ => {}
        }

        // Quoted string: "..." with "" escape for embedded quotes
        if ch == '"' {
            i += 1;
            let mut s = String::new();
            while i < n {
                if chars[i] == '"' {
                    if i + 1 < n && chars[i + 1] == '"' {
                        s.push('"');
                        i += 2;
                    } else {
                        i += 1;
                        break;
                    }
                } else {
                    s.push(chars[i]);
                    i += 1;
                }
            }
            tokens.push(Token::Str(s));
            continue;
        }

        // Numbers: integer or float
        if ch.is_ascii_digit() {
            let start = i;
            let mut is_float = false;
            while i < n && chars[i].is_ascii_digit() {
                i += 1;
            }
            if i < n && chars[i] == '.' && i + 1 < n && chars[i + 1].is_ascii_digit() {
                is_float = true;
                i += 1;
                while i < n && chars[i].is_ascii_digit() {
                    i += 1;
                }
            }
            let num_str: String = chars[start..i].iter().collect();
            if is_float {
                let f: f64 = num_str
                    .parse()
                    .map_err(|_| format!("Bad float: {num_str}"))?;
                tokens.push(Token::Float(f));
            } else {
                let v: i64 = num_str.parse().map_err(|_| format!("Bad int: {num_str}"))?;
                tokens.push(Token::Int(v));
            }
            continue;
        }

        // Identifiers / keywords
        if ch.is_alphabetic() || ch == '_' {
            let start = i;
            while i < n && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            let tok = match word.as_str() {
                // W5H2 clause keywords (v1)
                "WHO" | "who" => Token::Who,
                "WHAT" | "what" => Token::What,
                "WHERE" | "where" => Token::Where,
                "WHEN" | "when" => Token::When,
                "WHY" | "why" => Token::Why,
                "HOW" | "how" => Token::How,
                "HOW_MUCH" | "how_much" => Token::HowMuch,

                // Sovereign Operation verbs (optional, lead the whole query)
                "ORBIT" | "orbit" => Token::VerbOrbit,
                "EMIT" | "emit" => Token::VerbEmit,
                "PROVE" | "prove" => Token::VerbProve,
                "SYNC" | "sync" => Token::VerbSync,
                "WITNESS" | "witness" => Token::VerbWitness,

                // v2.0 clause keywords
                "NODE" | "node" => Token::Node,
                "ACROSS" | "across" => Token::Across,
                "TIER" | "tier" => Token::Tier,
                "STATE" | "state" => Token::State,
                "NASH" | "nash" => Token::Nash,
                "PATTERN" | "pattern" => Token::Pattern,
                "LINEAGE" | "lineage" => Token::Lineage,
                "GATE" | "gate" => Token::Gate,
                "SATAMU" | "satamu" => Token::Satamu,
                "ORBITAL" | "orbital" => Token::Orbital,

                // NODE targets
                "ALL" | "all" => Token::NodeAll,
                "EnkiDB" | "ENKIDB" => Token::DbEnkiDb,
                "EnkiDW" | "ENKIDW" => Token::DbEnkiDw,
                "EnkiSDB" | "ENKISDB" => Token::DbEnkiSdb,
                "EnkiODB" | "ENKIODB" => Token::DbEnkiOdb,
                "EnkiQDB" | "ENKIQDB" => Token::DbEnkiQdb,
                "EnkiMDB" | "ENKIMDB" => Token::DbEnkiMdb,
                "EnkiDDB" | "ENKIDDB" => Token::DbEnkiDdb,
                "NARUDU" | "narudu" => Token::DbNarudu,
                "ENKI_PATTERN" | "enki_pattern" => Token::DbEnkiPattern,

                // ACROSS sub-keyword
                "BIGRING" | "bigring" => Token::Bigring,

                // TIER values
                "HOT" | "hot" => Token::TierHot,
                "WARM" | "warm" => Token::TierWarm,
                "COLD" | "cold" => Token::TierCold,
                "CRYSTALLIZED" | "crystallized" => Token::TierCrystallized,

                // STATE values
                "EMERGING" | "emerging" => Token::StateEmerging,
                "STABLE" | "stable" => Token::StateStable,
                "CANONICAL" | "canonical" => Token::StateCanonical,
                "DEPRECATED" | "deprecated" => Token::StateDeprecated,

                // NASH sub-keywords
                "SCORE" | "score" => Token::NashScore,
                "BREAKING" | "breaking" => Token::NashBreaking,

                // PATTERN sub-keywords
                "TYPE" | "type" => Token::PatternType,
                "CONFIDENCE" | "confidence" => Token::PatternConfidence,
                "MAX_CONSTITUENTS" | "max_constituents" => Token::PatternConstituents,

                // Pattern type names
                "CrowdFlow" | "CROWDFLOW" => Token::PtCrowdFlow,
                "AviationCorridor" | "AVIATIONCORRIDOR" => Token::PtAviationCorridor,
                "AviationHolding" | "AVIATIONHOLDING" => Token::PtAviationHolding,
                "IndoorHallway" | "INDOORHALLWAY" => Token::PtIndoorHallway,
                "IndoorRoom" | "INDOORROOM" => Token::PtIndoorRoom,
                "IndoorTransition" | "INDOORTRANSITION" => Token::PtIndoorTransition,
                "WaterFlow" | "WATERFLOW" => Token::PtWaterFlow,
                "Custom" | "CUSTOM" => Token::PtCustom,

                // LINEAGE sub-keywords
                "DEPTH" | "depth" => Token::LineageDepth,
                "FULL" | "full" => Token::LineageFull,

                // SATAMU sub-keywords
                "REQUIRED" | "required" => Token::SatamuRequired,
                "BYPASS" | "bypass" => Token::SatamuBypass,

                // WHO sub-keywords (v1)
                "BOUND_TO" | "bound_to" => Token::BoundTo,

                // WHEN sub-keywords (v1)
                "AT" | "at" => Token::At,
                "EPOCH" | "epoch" => Token::Epoch,
                "BEFORE" | "before" => Token::Before,
                "AFTER" | "after" => Token::After,
                "NOW" | "now" => Token::Now,

                // WHY sub-keywords (v1)
                "LANE" | "lane" => Token::Lane,
                "QUALITY_BYTE" | "quality_byte" => Token::QualityByte,

                // HOW / HOW_MUCH sub-keywords (v1)
                "BY" | "by" => Token::By,
                "ASC" | "asc" => Token::Asc,
                "DESC" | "desc" => Token::Desc,
                "LIMIT" | "limit" => Token::Limit,
                "TOP" | "top" => Token::Top,

                // v2.1 MEASURE / GRAVITY (Anti-SQL aggregate clauses)
                "MEASURE" | "measure" => Token::Measure,
                "DENSE" | "dense" => Token::Dense,
                "FLUX" | "flux" => Token::Flux,
                "ROTOR_MEAN" | "rotor_mean" => Token::RotorMean,
                "GRAVITY" | "gravity" => Token::Gravity,
                "BAND" | "band" => Token::Band,
                "MAX_GROUPS" | "max_groups" => Token::MaxGroups,

                // Logical (v1)
                "AND" | "and" => Token::And,
                "OR" | "or" => Token::Or,
                "NOT" | "not" => Token::Not,
                "EXISTS" | "exists" => Token::Exists,

                // Production execution hints
                "ANCHOR" | "anchor" => Token::Anchor,
                "AUTO" | "auto" => Token::AnchorAuto,
                "SURROGATE_TIME" | "surrogate_time" => Token::AnchorSurrogateTime,
                "STATE_STATION" | "state_station" => Token::AnchorStateStation,
                "E7_FIRST" | "e7_first" => Token::AnchorE7First,
                "FULL_SCAN" | "full_scan" => Token::AnchorFullScan,
                "STREAM" | "stream" => Token::Stream,
                "DERIVE_STATION" | "derive_station" => Token::DeriveStation,
                "ABORT_SCAN" | "abort_scan" => Token::AbortScan,
                "FILTER_ORDER" | "filter_order" => Token::FilterOrder,
                "SURROGATE_RANGE" | "surrogate_range" => Token::FsSurrogateRange,
                "ORBITAL_RANGE" | "orbital_range" => Token::FsOrbitalRange,
                "STATE_FILTER" | "state_filter" => Token::FsState,
                "DERIVE_STATION_FILTER" | "derive_station_filter" => Token::FsDeriveStation,
                "LANE_FILTER" | "lane_filter" => Token::FsLane,
                "QUALITY_BYTE_FILTER" | "quality_byte_filter" => Token::FsQualityByte,
                "EAV_ATTR" | "eav_attr" => Token::FsEavAttr,
                "E7_REGION" | "e7_region" => Token::FsE7Region,

                // NaviMap (retained from v1)
                "ROUTE" | "route" => Token::Route,
                "INDEX" | "index" => Token::Index,
                "MODULAR" | "modular" => Token::Modular,
                "weight" => Token::AttrWeight,
                "level" => Token::AttrLevel,
                "signature" => Token::AttrSignature,
                "total_mass" => Token::AttrTotalMass,
                "peak_cost" => Token::AttrPeakCost,

                _ => Token::Ident(word),
            };
            tokens.push(tok);
            continue;
        }

        return Err(format!("Unexpected character: {ch:?}"));
    }

    tokens.push(Token::Eof);
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── v1 clause tests (regression) ─────────────────────────────────────────

    #[test]
    fn tokenize_who_clause() {
        let toks = tokenize("WHO Citizens.E").unwrap();
        assert_eq!(toks[0], Token::Who);
        assert_eq!(toks[1], Token::Ident("Citizens".into()));
        assert_eq!(toks[2], Token::Dot);
        assert_eq!(toks[3], Token::Ident("E".into()));
    }

    #[test]
    fn tokenize_what_clause() {
        let toks = tokenize("WHAT E[name.given, birth.year]").unwrap();
        assert_eq!(toks[0], Token::What);
        assert_eq!(toks[1], Token::Ident("E".into()));
        assert_eq!(toks[2], Token::LBracket);
        assert_eq!(toks[3], Token::Ident("name".into()));
        assert_eq!(toks[4], Token::Dot);
        assert_eq!(toks[5], Token::Ident("given".into()));
        assert_eq!(toks[6], Token::Comma);
    }

    #[test]
    fn tokenize_where_clause_with_string() {
        let toks = tokenize(r#"WHERE E[city.name] = "Najaf""#).unwrap();
        assert_eq!(toks[0], Token::Where);
        assert_eq!(toks[7], Token::Eq);
        assert_eq!(toks[8], Token::Str("Najaf".into()));
    }

    #[test]
    fn tokenize_where_clause_with_int() {
        let toks = tokenize("WHERE E[birth.year] < 1990").unwrap();
        assert_eq!(toks[7], Token::Lt);
        assert_eq!(toks[8], Token::Int(1990));
    }

    #[test]
    fn tokenize_when_at_epoch_now() {
        let toks = tokenize("WHEN AT EPOCH NOW").unwrap();
        assert_eq!(toks[0], Token::When);
        assert_eq!(toks[1], Token::At);
        assert_eq!(toks[2], Token::Epoch);
        assert_eq!(toks[3], Token::Now);
    }

    #[test]
    fn tokenize_when_before_epoch() {
        let toks = tokenize("WHEN BEFORE EPOCH 1680000000").unwrap();
        assert_eq!(toks[1], Token::Before);
        assert_eq!(toks[3], Token::Int(1680000000));
    }

    #[test]
    fn tokenize_why_quality_byte() {
        let toks = tokenize("WHY QUALITY_BYTE < 100").unwrap();
        assert_eq!(toks[0], Token::Why);
        assert_eq!(toks[1], Token::QualityByte);
        assert_eq!(toks[2], Token::Lt);
        assert_eq!(toks[3], Token::Int(100));
    }

    #[test]
    fn tokenize_why_lane() {
        let toks = tokenize("WHY LANE != Black").unwrap();
        assert_eq!(toks[1], Token::Lane);
        assert_eq!(toks[2], Token::Neq);
        assert_eq!(toks[3], Token::Ident("Black".into()));
    }

    #[test]
    fn tokenize_how_clause() {
        let toks = tokenize("HOW BY E[birth.year] DESC").unwrap();
        assert_eq!(toks[0], Token::How);
        assert_eq!(toks[1], Token::By);
        assert_eq!(toks[8], Token::Desc);
    }

    #[test]
    fn tokenize_how_much_limit() {
        let toks = tokenize("HOW_MUCH LIMIT 100").unwrap();
        assert_eq!(toks[0], Token::HowMuch);
        assert_eq!(toks[1], Token::Limit);
        assert_eq!(toks[2], Token::Int(100));
    }

    #[test]
    fn tokenize_how_much_top() {
        let toks = tokenize("HOW_MUCH TOP 10 BY E[score] ASC").unwrap();
        assert_eq!(toks[0], Token::HowMuch);
        assert_eq!(toks[1], Token::Top);
        assert_eq!(toks[2], Token::Int(10));
        assert_eq!(toks[8], Token::Asc);
    }

    #[test]
    fn tokenize_bound_to() {
        let toks = tokenize("WHO Graves.G BOUND_TO Citizens.C").unwrap();
        assert_eq!(toks[4], Token::BoundTo);
    }

    #[test]
    fn tokenize_exists_keyword() {
        let toks = tokenize("WHERE E[leak.reported] EXISTS").unwrap();
        assert_eq!(toks[7], Token::Exists);
    }

    #[test]
    fn tokenize_not_exists() {
        let toks = tokenize("WHERE E[leak.reported] NOT EXISTS").unwrap();
        assert_eq!(toks[7], Token::Not);
        assert_eq!(toks[8], Token::Exists);
    }

    #[test]
    fn tokenize_star_projection() {
        let toks = tokenize("WHAT E[*]").unwrap();
        assert_eq!(toks[2], Token::LBracket);
        assert_eq!(toks[3], Token::Star);
        assert_eq!(toks[4], Token::RBracket);
    }

    #[test]
    fn tokenize_negative_int() {
        let toks = tokenize("-42").unwrap();
        assert_eq!(toks[0], Token::Int(-42));
    }

    #[test]
    fn tokenize_float() {
        let toks = tokenize("0.8").unwrap();
        assert_eq!(toks[0], Token::Float(0.8));
    }

    #[test]
    fn tokenize_comment_skipped() {
        let toks = tokenize("WHO -- this is a comment\nCitizens.E").unwrap();
        assert_eq!(toks[0], Token::Who);
        assert_eq!(toks[1], Token::Ident("Citizens".into()));
    }

    #[test]
    fn tokenize_and_or_operators() {
        let toks = tokenize("AND OR").unwrap();
        assert_eq!(toks[0], Token::And);
        assert_eq!(toks[1], Token::Or);
    }

    #[test]
    fn tokenize_quoted_string_with_escaped_quote() {
        let toks = tokenize(r#""She said ""hello"""#).unwrap();
        assert_eq!(toks[0], Token::Str("She said \"hello\"".into()));
    }

    #[test]
    fn tokenize_route_query() {
        let toks =
            tokenize("ROUTE NaviLattice7 WHERE index.weight = -1 AND index.level = 7").unwrap();
        assert_eq!(toks[0], Token::Route);
        assert_eq!(toks[5], Token::AttrWeight);
    }

    #[test]
    fn tokenize_full_w5h2_query() {
        let src =
            "WHO Citizens.E\nWHAT E[name.given]\nWHERE E[city.name] = \"Najaf\"\nHOW_MUCH LIMIT 10";
        let toks = tokenize(src).unwrap();
        assert_eq!(toks[0], Token::Who);
        assert_eq!(*toks.last().unwrap(), Token::Eof);
    }

    // ── v2.0 new clause tests ─────────────────────────────────────────────────

    #[test]
    fn tokenize_node_single() {
        let toks = tokenize("NODE EnkiMDB").unwrap();
        assert_eq!(toks[0], Token::Node);
        assert_eq!(toks[1], Token::DbEnkiMdb);
    }

    #[test]
    fn tokenize_node_multi_pipe() {
        // NODE EnkiDB | EnkiDW | NARUDU
        let toks = tokenize("NODE EnkiDB | EnkiDW | NARUDU").unwrap();
        assert_eq!(toks[0], Token::Node);
        assert_eq!(toks[1], Token::DbEnkiDb);
        assert_eq!(toks[2], Token::Pipe);
        assert_eq!(toks[3], Token::DbEnkiDw);
        assert_eq!(toks[4], Token::Pipe);
        assert_eq!(toks[5], Token::DbNarudu);
    }

    #[test]
    fn tokenize_node_all() {
        let toks = tokenize("NODE ALL").unwrap();
        assert_eq!(toks[0], Token::Node);
        assert_eq!(toks[1], Token::NodeAll);
    }

    #[test]
    fn tokenize_across_bigring() {
        let toks = tokenize("ACROSS BIGRING ClientAlpha").unwrap();
        assert_eq!(toks[0], Token::Across);
        assert_eq!(toks[1], Token::Bigring);
        assert_eq!(toks[2], Token::Ident("ClientAlpha".into()));
    }

    #[test]
    fn tokenize_across_all() {
        let toks = tokenize("ACROSS ALL").unwrap();
        assert_eq!(toks[0], Token::Across);
        assert_eq!(toks[1], Token::NodeAll);
    }

    #[test]
    fn tokenize_tier_hot() {
        let toks = tokenize("TIER HOT").unwrap();
        assert_eq!(toks[0], Token::Tier);
        assert_eq!(toks[1], Token::TierHot);
    }

    #[test]
    fn tokenize_tier_multi() {
        // TIER HOT | WARM
        let toks = tokenize("TIER HOT | WARM").unwrap();
        assert_eq!(toks[1], Token::TierHot);
        assert_eq!(toks[2], Token::Pipe);
        assert_eq!(toks[3], Token::TierWarm);
    }

    #[test]
    fn tokenize_state_canonical() {
        let toks = tokenize("STATE CANONICAL").unwrap();
        assert_eq!(toks[0], Token::State);
        assert_eq!(toks[1], Token::StateCanonical);
    }

    #[test]
    fn tokenize_nash_score() {
        let toks = tokenize("NASH SCORE < 0.20").unwrap();
        assert_eq!(toks[0], Token::Nash);
        assert_eq!(toks[1], Token::NashScore);
        assert_eq!(toks[2], Token::Lt);
        assert_eq!(toks[3], Token::Float(0.20));
    }

    #[test]
    fn tokenize_nash_breaking() {
        let toks = tokenize("NASH BREAKING").unwrap();
        assert_eq!(toks[0], Token::Nash);
        assert_eq!(toks[1], Token::NashBreaking);
    }

    #[test]
    fn tokenize_pattern_type() {
        let toks = tokenize("PATTERN TYPE CrowdFlow").unwrap();
        assert_eq!(toks[0], Token::Pattern);
        assert_eq!(toks[1], Token::PatternType);
        assert_eq!(toks[2], Token::PtCrowdFlow);
    }

    #[test]
    fn tokenize_pattern_confidence() {
        let toks = tokenize("PATTERN CONFIDENCE >= 0.85").unwrap();
        assert_eq!(toks[0], Token::Pattern);
        assert_eq!(toks[1], Token::PatternConfidence);
        assert_eq!(toks[2], Token::Gte);
        assert_eq!(toks[3], Token::Float(0.85));
    }

    #[test]
    fn tokenize_lineage_depth() {
        let toks = tokenize("LINEAGE DEPTH 5").unwrap();
        assert_eq!(toks[0], Token::Lineage);
        assert_eq!(toks[1], Token::LineageDepth);
        assert_eq!(toks[2], Token::Int(5));
    }

    #[test]
    fn tokenize_lineage_full() {
        let toks = tokenize("LINEAGE FULL").unwrap();
        assert_eq!(toks[0], Token::Lineage);
        assert_eq!(toks[1], Token::LineageFull);
    }

    #[test]
    fn tokenize_gate_station() {
        let toks = tokenize("GATE DataSteward").unwrap();
        assert_eq!(toks[0], Token::Gate);
        assert_eq!(toks[1], Token::Ident("DataSteward".into()));
    }

    #[test]
    fn tokenize_gate_all() {
        let toks = tokenize("GATE ALL").unwrap();
        assert_eq!(toks[0], Token::Gate);
        assert_eq!(toks[1], Token::NodeAll);
    }

    #[test]
    fn tokenize_satamu_required() {
        let toks = tokenize("SATAMU REQUIRED").unwrap();
        assert_eq!(toks[0], Token::Satamu);
        assert_eq!(toks[1], Token::SatamuRequired);
    }

    #[test]
    fn tokenize_satamu_bypass() {
        let toks = tokenize(r#"SATAMU BYPASS "AI Council approved""#).unwrap();
        assert_eq!(toks[0], Token::Satamu);
        assert_eq!(toks[1], Token::SatamuBypass);
        assert_eq!(toks[2], Token::Str("AI Council approved".into()));
    }

    #[test]
    fn tokenize_orbital_range() {
        let toks = tokenize("ORBITAL 100 .. 200").unwrap();
        assert_eq!(toks[0], Token::Orbital);
        assert_eq!(toks[1], Token::Int(100));
        assert_eq!(toks[2], Token::DotDot);
        assert_eq!(toks[3], Token::Int(200));
    }

    #[test]
    fn tokenize_sovereign_verbs() {
        assert_eq!(tokenize("ORBIT").unwrap()[0], Token::VerbOrbit);
        assert_eq!(tokenize("emit").unwrap()[0], Token::VerbEmit);
        assert_eq!(tokenize("PROVE").unwrap()[0], Token::VerbProve);
        assert_eq!(tokenize("sync").unwrap()[0], Token::VerbSync);
        assert_eq!(tokenize("WITNESS").unwrap()[0], Token::VerbWitness);
    }

    #[test]
    fn tokenize_orbital_now() {
        let toks = tokenize("ORBITAL NOW").unwrap();
        assert_eq!(toks[0], Token::Orbital);
        assert_eq!(toks[1], Token::Now);
    }

    #[test]
    fn tokenize_full_v2_query() {
        let src = "NODE EnkiMDB | ENKI_PATTERN\n\
                   ACROSS BIGRING ClientAlpha\n\
                   WHO Tribes.E\n\
                   WHERE E[event.type] = \"entry\"\n\
                   TIER HOT | WARM\n\
                   STATE CANONICAL\n\
                   NASH SCORE < 0.20\n\
                   PATTERN TYPE CrowdFlow\n\
                   PATTERN CONFIDENCE >= 0.85\n\
                   LINEAGE DEPTH 3\n\
                   GATE DataSteward\n\
                   SATAMU REQUIRED\n\
                   ORBITAL 100 .. 500\n\
                   HOW_MUCH LIMIT 1000";
        let toks = tokenize(src).unwrap();
        assert_eq!(toks[0], Token::Node);
        assert_eq!(*toks.last().unwrap(), Token::Eof);
        // Should have no error — the full round-trip tokenizes cleanly.
        assert!(toks.len() > 30);
    }

    #[test]
    fn tokenize_enki_pattern_db() {
        let toks = tokenize("NODE ENKI_PATTERN").unwrap();
        assert_eq!(toks[1], Token::DbEnkiPattern);
    }

    #[test]
    fn pipe_separator() {
        let toks = tokenize("a | b").unwrap();
        assert_eq!(toks[1], Token::Pipe);
    }

    // ── v2.1 MEASURE / GRAVITY clause tests ──────────────────────────────────

    #[test]
    fn tokenize_measure_dense() {
        let toks = tokenize("MEASURE DENSE").unwrap();
        assert_eq!(toks[0], Token::Measure);
        assert_eq!(toks[1], Token::Dense);
    }

    #[test]
    fn tokenize_measure_flux() {
        let toks = tokenize("MEASURE FLUX E[pressure]").unwrap();
        assert_eq!(toks[0], Token::Measure);
        assert_eq!(toks[1], Token::Flux);
        assert_eq!(toks[2], Token::Ident("E".into()));
    }

    #[test]
    fn tokenize_measure_rotor_mean() {
        let toks = tokenize("MEASURE ROTOR_MEAN E[orientation]").unwrap();
        assert_eq!(toks[0], Token::Measure);
        assert_eq!(toks[1], Token::RotorMean);
    }

    #[test]
    fn tokenize_gravity_band() {
        let toks = tokenize("GRAVITY E[pressure] BAND 5.0").unwrap();
        assert_eq!(toks[0], Token::Gravity);
        assert_eq!(toks[5], Token::Band);
        assert_eq!(toks[6], Token::Float(5.0));
    }

    #[test]
    fn tokenize_gravity_max_groups() {
        let toks = tokenize("GRAVITY E[station] MAX_GROUPS 1000").unwrap();
        assert_eq!(toks[0], Token::Gravity);
        assert_eq!(toks[5], Token::MaxGroups);
        assert_eq!(toks[6], Token::Int(1000));
    }
}
