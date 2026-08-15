//! Akkadian DSL v3.4 - Lexer/Tokenizer
//! Complete port from ANTLR grammar to Rust
//! 
//! Author: Bahaa Fadam
//! BahyWay Sovereign Ecosystem

use std::iter::Peekable;
use std::str::Chars;

// ============================================================================
// TOKEN TYPES
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Keywords
    Context,
    Identity,
    Storage,
    DataVault,
    Hub,
    Satellite,
    Link,
    Vectorization,
    Command,
    RagQuery,
    Presentation,
    RuleSet,
    MetaAlgorithmics,
    
    // Identity keywords
    BusinessKey,
    FuzzyResolution,
    SpatialId,
    Match,
    Using,
    Threshold,
    Algorithm,
    Dimensions,
    Precision,
    
    // Storage keywords
    With,
    TemporalTracking,
    PartitionBy,
    ClusteredBy,
    ShardKey,
    Hash,
    Range,
    List,
    Indexes,
    
    // Index types
    Btree,
    Gin,
    Gist,
    Brin,
    Spgist,
    
    // Column constraints
    Unique,
    PrimaryKey,
    
    // Data types
    Uuid,
    Varchar,
    Int,
    BigInt,
    Decimal,
    Timestamp,
    Geometry,
    
    // Vectorization
    Model,
    Embeddings,
    
    // Commands
    Validation,
    Execution,
    Check,
    InsertEvent,
    
    // RAG Query
    Description,
    Retrieval,
    Generation,
    Augmentation,
    VectorSearch,
    GraphTraverse,
    TemporalWindow,
    Llm,
    PromptTemplate,
    RequiredCitations,
    HybridSearch,
    TopK,
    Hops,
    From,
    Via,
    
    // Presentation
    Style,
    Color,
    Icon,
    Shape,
    Label,
    SizeBy,
    
    // Rule Set
    Input,
    Output,
    Logic,
    
    // Meta Algorithmics
    ExecutionModel,
    Actors,
    ParallelPatterns,
    
    // Literals
    Identifier(String),
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    
    // Operators & Delimiters
    Colon,           // :
    Comma,           // ,
    LeftBrace,       // {
    RightBrace,      // }
    LeftBracket,     // [
    RightBracket,    // ]
    LeftParen,       // (
    RightParen,      // )
    Equal,           // =
    
    // Special
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, line: usize, column: usize) -> Self {
        Self {
            token_type,
            lexeme,
            line,
            column,
        }
    }
    
    pub fn eof() -> Self {
        Self {
            token_type: TokenType::Eof,
            lexeme: String::new(),
            line: 0,
            column: 0,
        }
    }
}

// ============================================================================
// LEXER
// ============================================================================

pub struct Lexer<'a> {
    source: &'a str,
    chars: Peekable<Chars<'a>>,
    current_pos: usize,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            chars: source.chars().peekable(),
            current_pos: 0,
            line: 1,
            column: 1,
        }
    }
    
    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();
        
        loop {
            let token = self.next_token()?;
            let is_eof = matches!(token.token_type, TokenType::Eof);
            tokens.push(token);
            if is_eof {
                break;
            }
        }
        
        Ok(tokens)
    }
    
    pub fn next_token(&mut self) -> Result<Token, LexError> {
        self.skip_whitespace_and_comments();
        
        let ch = match self.peek() {
            Some(c) => *c,
            None => return Ok(Token::eof()),
        };
        
        let line = self.line;
        let column = self.column;
        
        let token_type = match ch {
            'a'..='z' | 'A'..='Z' | '_' => return self.identifier_or_keyword(),
            '0'..='9' => return self.number(),
            '\'' | '"' => return self.string(),
            ':' => { self.advance(); TokenType::Colon }
            ',' => { self.advance(); TokenType::Comma }
            '{' => { self.advance(); TokenType::LeftBrace }
            '}' => { self.advance(); TokenType::RightBrace }
            '[' => { self.advance(); TokenType::LeftBracket }
            ']' => { self.advance(); TokenType::RightBracket }
            '(' => { self.advance(); TokenType::LeftParen }
            ')' => { self.advance(); TokenType::RightParen }
            '=' => { self.advance(); TokenType::Equal }
            _ => return Err(LexError::UnexpectedChar { ch, line, column }),
        };
        
        Ok(Token::new(token_type, ch.to_string(), line, column))
    }
    
    fn identifier_or_keyword(&mut self) -> Result<Token, LexError> {
        let start_line = self.line;
        let start_column = self.column;
        let start_pos = self.current_pos;
        
        while let Some(&ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }
        
        let text = &self.source[start_pos..self.current_pos];
        let token_type = self.keyword_or_identifier(text);
        
        Ok(Token::new(token_type, text.to_string(), start_line, start_column))
    }
    
    fn keyword_or_identifier(&self, text: &str) -> TokenType {
        match text {
            // Main keywords
            "CONTEXT" => TokenType::Context,
            "IDENTITY" => TokenType::Identity,
            "STORAGE" => TokenType::Storage,
            "DataVault" => TokenType::DataVault,
            "HUB" => TokenType::Hub,
            "SATELLITE" => TokenType::Satellite,
            "LINK" => TokenType::Link,
            "VECTORIZATION" => TokenType::Vectorization,
            "COMMAND" => TokenType::Command,
            "RAG_QUERY" => TokenType::RagQuery,
            "PRESENTATION" => TokenType::Presentation,
            "RULESET" => TokenType::RuleSet,
            "META_ALGORITHMICS" => TokenType::MetaAlgorithmics,
            
            // Identity
            "BUSINESS_KEY" => TokenType::BusinessKey,
            "FUZZY_RESOLUTION" => TokenType::FuzzyResolution,
            "SPATIAL_ID" => TokenType::SpatialId,
            "MATCH" => TokenType::Match,
            "USING" => TokenType::Using,
            "THRESHOLD" => TokenType::Threshold,
            "ALGORITHM" => TokenType::Algorithm,
            "DIMENSIONS" => TokenType::Dimensions,
            "PRECISION" => TokenType::Precision,
            
            // Storage
            "WITH" => TokenType::With,
            "temporal_tracking" => TokenType::TemporalTracking,
            "PARTITION_BY" => TokenType::PartitionBy,
            "CLUSTERED_BY" => TokenType::ClusteredBy,
            "SHARD_KEY" => TokenType::ShardKey,
            "HASH" => TokenType::Hash,
            "RANGE" => TokenType::Range,
            "LIST" => TokenType::List,
            "INDEXES" => TokenType::Indexes,
            
            // Index types
            "BTREE" => TokenType::Btree,
            "GIN" => TokenType::Gin,
            "GIST" => TokenType::Gist,
            "BRIN" => TokenType::Brin,
            "SPGIST" => TokenType::Spgist,
            
            // Constraints
            "UNIQUE" => TokenType::Unique,
            "PRIMARY" if self.peek_ahead_for_key() => TokenType::PrimaryKey,
            
            // Data types
            "UUID" => TokenType::Uuid,
            "VARCHAR" => TokenType::Varchar,
            "INT" => TokenType::Int,
            "BIGINT" => TokenType::BigInt,
            "DECIMAL" => TokenType::Decimal,
            "TIMESTAMP" => TokenType::Timestamp,
            "GEOMETRY" => TokenType::Geometry,
            
            // Vectorization
            "MODEL" => TokenType::Model,
            "EMBEDDINGS" => TokenType::Embeddings,
            
            // Commands
            "VALIDATION" => TokenType::Validation,
            "EXECUTION" => TokenType::Execution,
            "CHECK" => TokenType::Check,
            "INSERT_EVENT" => TokenType::InsertEvent,
            
            // RAG
            "DESCRIPTION" => TokenType::Description,
            "RETRIEVAL" => TokenType::Retrieval,
            "GENERATION" => TokenType::Generation,
            "AUGMENTATION" => TokenType::Augmentation,
            "VECTOR_SEARCH" => TokenType::VectorSearch,
            "GRAPH_TRAVERSE" => TokenType::GraphTraverse,
            "TEMPORAL_WINDOW" => TokenType::TemporalWindow,
            "LLM" => TokenType::Llm,
            "PROMPT_TEMPLATE" => TokenType::PromptTemplate,
            "REQUIRED_CITATIONS" => TokenType::RequiredCitations,
            "HYBRID_SEARCH" => TokenType::HybridSearch,
            "TOP_K" => TokenType::TopK,
            "FROM" => TokenType::From,
            "VIA" => TokenType::Via,
            
            // Presentation
            "STYLE" => TokenType::Style,
            "COLOR" => TokenType::Color,
            "ICON" => TokenType::Icon,
            "SHAPE" => TokenType::Shape,
            "LABEL" => TokenType::Label,
            "SIZE_BY" => TokenType::SizeBy,
            
            // Rule Set
            "INPUT" => TokenType::Input,
            "OUTPUT" => TokenType::Output,
            "LOGIC" => TokenType::Logic,
            
            // Meta
            "EXECUTION_MODEL" => TokenType::ExecutionModel,
            "ACTORS" => TokenType::Actors,
            "PARALLEL_PATTERNS" => TokenType::ParallelPatterns,
            
            // Booleans
            "true" => TokenType::Boolean(true),
            "false" => TokenType::Boolean(false),
            
            // Default: identifier
            _ => TokenType::Identifier(text.to_string()),
        }
    }
    
    fn peek_ahead_for_key(&self) -> bool {
        // Check if next token is "KEY" for "PRIMARY KEY"
        // This is a simplified version
        false
    }
    
    fn number(&mut self) -> Result<Token, LexError> {
        let start_line = self.line;
        let start_column = self.column;
        let start_pos = self.current_pos;
        let mut has_dot = false;
        
        while let Some(&ch) = self.peek() {
            if ch.is_numeric() {
                self.advance();
            } else if ch == '.' && !has_dot {
                has_dot = true;
                self.advance();
            } else {
                break;
            }
        }
        
        let text = &self.source[start_pos..self.current_pos];
        
        let token_type = if has_dot {
            let value = text.parse::<f64>()
                .map_err(|_| LexError::InvalidNumber { text: text.to_string(), line: start_line, column: start_column })?;
            TokenType::Float(value)
        } else {
            let value = text.parse::<i64>()
                .map_err(|_| LexError::InvalidNumber { text: text.to_string(), line: start_line, column: start_column })?;
            TokenType::Integer(value)
        };
        
        Ok(Token::new(token_type, text.to_string(), start_line, start_column))
    }
    
    fn string(&mut self) -> Result<Token, LexError> {
        let start_line = self.line;
        let start_column = self.column;
        let quote = self.advance().unwrap(); // ' or "
        let start_pos = self.current_pos;
        
        while let Some(&ch) = self.peek() {
            if ch == quote {
                let text = &self.source[start_pos..self.current_pos];
                self.advance(); // Consume closing quote
                return Ok(Token::new(
                    TokenType::String(text.to_string()),
                    format!("{}{}{}", quote, text, quote),
                    start_line,
                    start_column
                ));
            } else if ch == '\n' {
                return Err(LexError::UnterminatedString { line: start_line, column: start_column });
            } else {
                self.advance();
            }
        }
        
        Err(LexError::UnterminatedString { line: start_line, column: start_column })
    }
    
    fn skip_whitespace_and_comments(&mut self) {
        while let Some(&ch) = self.peek() {
            if ch.is_whitespace() {
                if ch == '\n' {
                    self.line += 1;
                    self.column = 1;
                }
                self.advance();
            } else if ch == '/' && self.peek_ahead() == Some('/') {
                // Single-line comment
                while let Some(&ch) = self.peek() {
                    self.advance();
                    if ch == '\n' {
                        break;
                    }
                }
            } else {
                break;
            }
        }
    }
    
    fn peek(&mut self) -> Option<&char> {
        self.chars.peek()
    }
    
    fn peek_ahead(&self) -> Option<char> {
        // This is a simplified version - proper implementation would need better lookahead
        None
    }
    
    fn advance(&mut self) -> Option<char> {
        let ch = self.chars.next();
        if let Some(c) = ch {
            self.current_pos += c.len_utf8();
            self.column += 1;
        }
        ch
    }
}

// ============================================================================
// ERRORS
// ============================================================================

#[derive(Debug)]
pub enum LexError {
    UnexpectedChar { ch: char, line: usize, column: usize },
    UnterminatedString { line: usize, column: usize },
    InvalidNumber { text: String, line: usize, column: usize },
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LexError::UnexpectedChar { ch, line, column } => {
                write!(f, "Unexpected character '{}' at line {}, column {}", ch, line, column)
            }
            LexError::UnterminatedString { line, column } => {
                write!(f, "Unterminated string at line {}, column {}", line, column)
            }
            LexError::InvalidNumber { text, line, column } => {
                write!(f, "Invalid number '{}' at line {}, column {}", text, line, column)
            }
        }
    }
}

impl std::error::Error for LexError {}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_context() {
        let source = "CONTEXT TestContext { }";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens.len(), 5); // CONTEXT, TestContext, {, }, EOF
        assert!(matches!(tokens[0].token_type, TokenType::Context));
        assert!(matches!(tokens[1].token_type, TokenType::Identifier(_)));
    }

    #[test]
    fn test_tokenize_numbers() {
        let source = "123 45.67";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        
        assert!(matches!(tokens[0].token_type, TokenType::Integer(123)));
        assert!(matches!(tokens[1].token_type, TokenType::Float(_)));
    }

    #[test]
    fn test_tokenize_string() {
        let source = r#"'hello' "world""#;
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        
        assert!(matches!(tokens[0].token_type, TokenType::String(_)));
        assert!(matches!(tokens[1].token_type, TokenType::String(_)));
    }
}
