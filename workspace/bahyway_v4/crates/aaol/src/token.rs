//! AAOL Lexer — tokenizer for Akkadian Actor Orchestration Language (`.akk`).
//!
//! Supports both the orchestration layer (tribe, actor, event, route) and
//! the full PMPVD language layer (particle, rule, equation, guard, flow,
//! observe, emit, pipeline).
//!
//! Example `.akk` (PMPVD):
//!   PARTICLE citizen {
//!     arabic_name: TEXT(accuracy=0.98)
//!     nid:         KAKI
//!     TRIBE: IraqiCitizenMDM
//!   }

/// Lexical token produced by the AAOL lexer.
///
/// `PartialEq` only — `f64` inside `Float` cannot derive `Eq`.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // ── Orchestration keywords ────────────────────────────────────────────
    Tribe,
    Actor,
    Role,
    When,
    Event,
    Then,
    Emit,
    Snapshot,
    Route,
    To,
    // ── PMPVD language keywords ───────────────────────────────────────────
    Particle,
    Rule,
    Equation,
    Guard,
    Flow,
    Observe,
    Pipeline,
    Weights,
    Ideal,
    And,
    Lane,
    Orbit,
    From,
    Where,
    Sort,
    Limit,
    Source,
    Pipe,
    Sink,
    Require,
    Allow,
    Asc,
    Desc,
    Text,
    KakiType,
    // ── Identifiers and literals ──────────────────────────────────────────
    Ident(String),
    StringLit(String),
    Number(u64),
    Float(f64),
    // ── Comparison operators ──────────────────────────────────────────────
    Geq,       // >=
    Leq,       // <=
    Gt,        // >
    Lt,        // <
    EqSign,    // =
    // ── Punctuation ───────────────────────────────────────────────────────
    LBrace,
    RBrace,
    LParen,
    RParen,
    LBracket,  // [
    RBracket,  // ]
    Comma,
    Colon,
    Semicolon,
    Dot,       // .
    // ── End of input ──────────────────────────────────────────────────────
    Eof,
}

/// Tokenize an AAOL/PMPVD source string.
/// Returns `Ok(Vec<Token>)` or `Err` with a description of the bad input.
pub fn tokenize(source: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = source.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];

        // Skip whitespace
        if ch.is_whitespace() { i += 1; continue; }

        // Skip line comments (#)
        if ch == '#' {
            while i < chars.len() && chars[i] != '\n' { i += 1; }
            continue;
        }

        // String literal
        if ch == '"' {
            i += 1;
            let start = i;
            while i < chars.len() && chars[i] != '"' { i += 1; }
            if i >= chars.len() {
                return Err("Unterminated string literal".into());
            }
            let s: String = chars[start..i].iter().collect();
            tokens.push(Token::StringLit(s));
            i += 1;
            continue;
        }

        // Number — integer or float
        if ch.is_ascii_digit() {
            let start = i;
            while i < chars.len() && chars[i].is_ascii_digit() { i += 1; }
            // Detect float: digits '.' digits
            if i < chars.len()
                && chars[i] == '.'
                && i + 1 < chars.len()
                && chars[i + 1].is_ascii_digit()
            {
                i += 1; // consume '.'
                while i < chars.len() && chars[i].is_ascii_digit() { i += 1; }
                let s: String = chars[start..i].iter().collect();
                let f: f64 = s.parse().map_err(|_| format!("Bad float: {s}"))?;
                tokens.push(Token::Float(f));
            } else {
                let s: String = chars[start..i].iter().collect();
                let n: u64 = s.parse().map_err(|_| format!("Bad number: {s}"))?;
                tokens.push(Token::Number(n));
            }
            continue;
        }

        // Identifier or keyword (case-insensitive for PMPVD keywords)
        if ch.is_alphabetic() || ch == '_' {
            let start = i;
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            let tok = match word.to_lowercase().as_str() {
                "tribe"    => Token::Tribe,
                "actor"    => Token::Actor,
                "role"     => Token::Role,
                "when"     => Token::When,
                "event"    => Token::Event,
                "then"     => Token::Then,
                "emit"     => Token::Emit,
                "snapshot" => Token::Snapshot,
                "route"    => Token::Route,
                "to"       => Token::To,
                "particle" => Token::Particle,
                "rule"     => Token::Rule,
                "equation" => Token::Equation,
                "guard"    => Token::Guard,
                "flow"     => Token::Flow,
                "observe"  => Token::Observe,
                "pipeline" => Token::Pipeline,
                "weights"  => Token::Weights,
                "ideal"    => Token::Ideal,
                "and"      => Token::And,
                "lane"     => Token::Lane,
                "orbit"    => Token::Orbit,
                "from"     => Token::From,
                "where"    => Token::Where,
                "sort"     => Token::Sort,
                "limit"    => Token::Limit,
                "source"   => Token::Source,
                "pipe"     => Token::Pipe,
                "sink"     => Token::Sink,
                "require"  => Token::Require,
                "allow"    => Token::Allow,
                "asc"      => Token::Asc,
                "desc"     => Token::Desc,
                "text"     => Token::Text,
                "kaki"     => Token::KakiType,
                _          => Token::Ident(word),
            };
            tokens.push(tok);
            continue;
        }

        // Multi-character operators (>= and <=)
        if ch == '>' || ch == '<' {
            let next = chars.get(i + 1).copied();
            if next == Some('=') {
                tokens.push(if ch == '>' { Token::Geq } else { Token::Leq });
                i += 2;
            } else {
                tokens.push(if ch == '>' { Token::Gt } else { Token::Lt });
                i += 1;
            }
            continue;
        }

        if ch == '=' { tokens.push(Token::EqSign); i += 1; continue; }

        // Punctuation
        let tok = match ch {
            '{' => Token::LBrace,
            '}' => Token::RBrace,
            '(' => Token::LParen,
            ')' => Token::RParen,
            '[' => Token::LBracket,
            ']' => Token::RBracket,
            ',' => Token::Comma,
            ':' => Token::Colon,
            ';' => Token::Semicolon,
            '.' => Token::Dot,
            c   => return Err(format!("Unexpected character: {c:?}")),
        };
        tokens.push(tok);
        i += 1;
    }

    tokens.push(Token::Eof);
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_tribe_block() {
        let src = "tribe Sumer { }";
        let toks = tokenize(src).unwrap();
        assert_eq!(toks[0], Token::Tribe);
        assert_eq!(toks[1], Token::Ident("Sumer".into()));
        assert_eq!(toks[2], Token::LBrace);
        assert_eq!(toks[3], Token::RBrace);
    }

    #[test]
    fn tokenize_string_literal() {
        let toks = tokenize("\"particle.created\"").unwrap();
        assert_eq!(toks[0], Token::StringLit("particle.created".into()));
    }

    #[test]
    fn tokenize_number() {
        let toks = tokenize("42").unwrap();
        assert_eq!(toks[0], Token::Number(42));
    }

    #[test]
    fn tokenize_float() {
        let toks = tokenize("0.833").unwrap();
        assert_eq!(toks[0], Token::Float(0.833));
    }

    #[test]
    fn tokenize_weights_array() {
        let toks = tokenize("[0.30, 0.20]").unwrap();
        assert_eq!(toks[0], Token::LBracket);
        assert_eq!(toks[1], Token::Float(0.30));
        assert_eq!(toks[2], Token::Comma);
        assert_eq!(toks[3], Token::Float(0.20));
        assert_eq!(toks[4], Token::RBracket);
    }

    #[test]
    fn tokenize_geq_operator() {
        let toks = tokenize("health >= 0.833").unwrap();
        assert_eq!(toks[0], Token::Ident("health".into()));
        assert_eq!(toks[1], Token::Geq);
        assert_eq!(toks[2], Token::Float(0.833));
    }

    #[test]
    fn tokenize_pmpvd_keywords() {
        let toks = tokenize("PARTICLE RULE EQUATION GUARD FLOW OBSERVE PIPELINE").unwrap();
        assert_eq!(toks[0], Token::Particle);
        assert_eq!(toks[1], Token::Rule);
        assert_eq!(toks[2], Token::Equation);
        assert_eq!(toks[3], Token::Guard);
        assert_eq!(toks[4], Token::Flow);
        assert_eq!(toks[5], Token::Observe);
        assert_eq!(toks[6], Token::Pipeline);
    }

    #[test]
    fn tokenize_kaki_text_types() {
        let toks = tokenize("name: TEXT(accuracy=0.90) id: KAKI").unwrap();
        assert_eq!(toks[0], Token::Ident("name".into()));
        assert_eq!(toks[1], Token::Colon);
        assert_eq!(toks[2], Token::Text);
        assert_eq!(toks[3], Token::LParen);
    }

    #[test]
    fn comment_skipped() {
        let toks = tokenize("tribe # this is a comment\nSumer").unwrap();
        assert_eq!(toks[0], Token::Tribe);
        assert_eq!(toks[1], Token::Ident("Sumer".into()));
    }

    #[test]
    fn eof_appended() {
        let toks = tokenize("").unwrap();
        assert_eq!(toks.last(), Some(&Token::Eof));
    }

    #[test]
    fn unterminated_string_is_error() {
        assert!(tokenize("\"unterminated").is_err());
    }

    #[test]
    fn dot_operator_tokenized() {
        // "particle" is a PMPVD keyword → Token::Particle, not Ident
        let toks = tokenize("particle.hepta.health").unwrap();
        assert_eq!(toks[0], Token::Particle);
        assert_eq!(toks[1], Token::Dot);
        assert_eq!(toks[2], Token::Ident("hepta".into()));
        assert_eq!(toks[3], Token::Dot);
        assert_eq!(toks[4], Token::Ident("health".into()));
    }
}
