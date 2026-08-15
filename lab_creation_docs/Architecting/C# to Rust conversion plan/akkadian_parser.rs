//! Akkadian DSL v3.4 - Parser
//! Recursive descent parser based on AkkadianAstVisitor.cs
//! 
//! Author: Bahaa Fadam
//! BahyWay Sovereign Ecosystem

use crate::ast::*;
use crate::lexer::{Lexer, Token, TokenType};
use std::collections::HashMap;

// ============================================================================
// PARSER
// ============================================================================

pub struct AkkadianParser {
    tokens: Vec<Token>,
    current: usize,
}

impl AkkadianParser {
    pub fn new(source: &str) -> Result<Self, ParseError> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize()
            .map_err(|e| ParseError::LexError(e.to_string()))?;
        
        Ok(Self {
            tokens,
            current: 0,
        })
    }
    
    pub fn parse(&mut self) -> Result<AkkadianProgram, ParseError> {
        self.parse_program()
    }
    
    // ========================================================================
    // PROGRAM: context+ EOF
    // ========================================================================
    
    fn parse_program(&mut self) -> Result<AkkadianProgram, ParseError> {
        let mut program = AkkadianProgram::new();
        
        while !self.is_at_end() {
            if self.check(TokenType::Context) {
                program.contexts.push(self.parse_context()?);
            } else {
                self.advance();
            }
        }
        
        Ok(program)
    }
    
    // ========================================================================
    // CONTEXT: 'CONTEXT' IDENTIFIER '{' contextBody* '}'
    // ========================================================================
    
    fn parse_context(&mut self) -> Result<ContextNode, ParseError> {
        self.expect(TokenType::Context)?;
        let name = self.expect_identifier()?;
        self.expect(TokenType::LeftBrace)?;
        
        let mut context = ContextNode::new(name);
        
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            match &self.peek().token_type {
                TokenType::Identity => {
                    context.identities.push(self.parse_identity()?);
                }
                TokenType::Storage => {
                    context.storage = Some(self.parse_storage()?);
                }
                TokenType::Vectorization => {
                    context.vectorization = Some(self.parse_vectorization()?);
                }
                TokenType::Command => {
                    context.commands.push(self.parse_command()?);
                }
                TokenType::RagQuery => {
                    context.rag_queries.push(self.parse_rag_query()?);
                }
                TokenType::Presentation => {
                    context.presentation = Some(self.parse_presentation()?);
                }
                TokenType::RuleSet => {
                    context.rule_set = Some(self.parse_rule_set()?);
                }
                TokenType::MetaAlgorithmics => {
                    context.meta_algorithmics = Some(self.parse_meta_algorithmics()?);
                }
                _ => {
                    return Err(ParseError::UnexpectedToken {
                        expected: "context body element".to_string(),
                        found: self.peek().clone(),
                    });
                }
            }
        }
        
        self.expect(TokenType::RightBrace)?;
        Ok(context)
    }
    
    // ========================================================================
    // IDENTITY: 'IDENTITY' IDENTIFIER '{' identityBody* '}'
    // ========================================================================
    
    fn parse_identity(&mut self) -> Result<IdentityNode, ParseError> {
        self.expect(TokenType::Identity)?;
        let name = self.expect_identifier()?;
        self.expect(TokenType::LeftBrace)?;
        
        let mut identity = IdentityNode::new(name);
        
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            match &self.peek().token_type {
                TokenType::BusinessKey => {
                    self.advance(); // BUSINESS_KEY
                    self.expect(TokenType::Colon)?;
                    identity.business_keys = self.parse_identifier_list()?;
                }
                TokenType::FuzzyResolution => {
                    self.advance(); // FUZZY_RESOLUTION
                    self.expect(TokenType::LeftBrace)?;
                    
                    while !self.check(TokenType::RightBrace) {
                        identity.fuzzy_rules.push(self.parse_fuzzy_rule()?);
                    }
                    
                    self.expect(TokenType::RightBrace)?;
                }
                TokenType::SpatialId => {
                    identity.spatial_id = Some(self.parse_spatial_id()?);
                }
                _ => {
                    self.advance(); // Skip unknown
                }
            }
        }
        
        self.expect(TokenType::RightBrace)?;
        Ok(identity)
    }
    
    fn parse_fuzzy_rule(&mut self) -> Result<FuzzyRuleConfig, ParseError> {
        self.expect(TokenType::Match)?;
        self.expect(TokenType::LeftBracket)?;
        let fields = self.parse_identifier_list()?;
        self.expect(TokenType::RightBracket)?;
        
        self.expect(TokenType::Using)?;
        let algorithm = self.expect_identifier()?;
        
        self.expect(TokenType::Threshold)?;
        let threshold = self.expect_float()?;
        
        Ok(FuzzyRuleConfig {
            fields,
            algorithm,
            threshold,
        })
    }
    
    fn parse_spatial_id(&mut self) -> Result<SpatialIdConfig, ParseError> {
        self.expect(TokenType::SpatialId)?;
        self.expect(TokenType::LeftBrace)?;
        
        let mut config = SpatialIdConfig {
            algorithm: String::new(),
            dimensions: Vec::new(),
            precision: 64,
        };
        
        while !self.check(TokenType::RightBrace) {
            match &self.peek().token_type {
                TokenType::Algorithm => {
                    self.advance();
                    self.expect(TokenType::Colon)?;
                    config.algorithm = self.expect_identifier()?;
                }
                TokenType::Dimensions => {
                    self.advance();
                    self.expect(TokenType::Colon)?;
                    self.expect(TokenType::LeftBracket)?;
                    config.dimensions = self.parse_identifier_list()?;
                    self.expect(TokenType::RightBracket)?;
                }
                TokenType::Precision => {
                    self.advance();
                    self.expect(TokenType::Colon)?;
                    config.precision = self.expect_integer()? as u32;
                }
                _ => self.advance(),
            }
        }
        
        self.expect(TokenType::RightBrace)?;
        Ok(config)
    }
    
    // ========================================================================
    // STORAGE: 'STORAGE' 'DataVault' '{' storageBody* '}'
    // ========================================================================
    
    fn parse_storage(&mut self) -> Result<StorageNode, ParseError> {
        self.expect(TokenType::Storage)?;
        self.expect(TokenType::DataVault)?;
        self.expect(TokenType::LeftBrace)?;
        
        let mut storage = StorageNode::new();
        
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            let table = match &self.peek().token_type {
                TokenType::Hub => self.parse_hub()?,
                TokenType::Satellite => self.parse_satellite()?,
                TokenType::Link => self.parse_link()?,
                _ => {
                    self.advance();
                    continue;
                }
            };
            
            match table.table_type {
                TableType::Hub => storage.hubs.push(table),
                TableType::Satellite => storage.satellites.push(table),
                TableType::Link => storage.links.push(table),
            }
        }
        
        self.expect(TokenType::RightBrace)?;
        Ok(storage)
    }
    
    fn parse_hub(&mut self) -> Result<TableNode, ParseError> {
        self.expect(TokenType::Hub)?;
        self.expect(TokenType::Colon)?;
        let name = self.expect_identifier()?;
        
        let mut table = TableNode::new(name, TableType::Hub);
        self.parse_table_options(&mut table)?;
        self.parse_table_body(&mut table)?;
        
        Ok(table)
    }
    
    fn parse_satellite(&mut self) -> Result<TableNode, ParseError> {
        self.expect(TokenType::Satellite)?;
        self.expect(TokenType::Colon)?;
        let name = self.expect_identifier()?;
        
        let mut table = TableNode::new(name, TableType::Satellite);
        
        // Check for temporal_tracking or other options
        while self.check(TokenType::With) {
            self.advance(); // WITH
            if self.check(TokenType::TemporalTracking) {
                self.advance();
                table.has_temporal_tracking = true;
            } else {
                self.parse_table_option(&mut table)?;
            }
        }
        
        self.parse_table_body(&mut table)?;
        Ok(table)
    }
    
    fn parse_link(&mut self) -> Result<TableNode, ParseError> {
        self.expect(TokenType::Link)?;
        self.expect(TokenType::Colon)?;
        let name = self.expect_identifier()?;
        
        let mut table = TableNode::new(name, TableType::Link);
        self.parse_table_options(&mut table)?;
        self.parse_table_body(&mut table)?;
        
        Ok(table)
    }
    
    fn parse_table_options(&mut self, table: &mut TableNode) -> Result<(), ParseError> {
        while self.check(TokenType::With) {
            self.advance(); // WITH
            self.parse_table_option(table)?;
        }
        Ok(())
    }
    
    fn parse_table_option(&mut self, table: &mut TableNode) -> Result<(), ParseError> {
        match &self.peek().token_type {
            TokenType::PartitionBy => {
                self.advance();
                self.expect(TokenType::Colon)?;
                table.partition_strategy = Some(self.parse_partition_strategy()?);
            }
            TokenType::ClusteredBy => {
                self.advance();
                self.expect(TokenType::Colon)?;
                table.clustered_by = Some(self.expect_identifier()?);
            }
            _ => {
                self.advance();
            }
        }
        Ok(())
    }
    
    fn parse_partition_strategy(&mut self) -> Result<String, ParseError> {
        let strategy = match &self.peek().token_type {
            TokenType::Hash => "HASH",
            TokenType::Range => "RANGE",
            TokenType::List => "LIST",
            _ => return Err(ParseError::UnexpectedToken {
                expected: "HASH, RANGE, or LIST".to_string(),
                found: self.peek().clone(),
            }),
        };
        
        self.advance();
        self.expect(TokenType::LeftParen)?;
        let field = self.expect_identifier()?;
        self.expect(TokenType::RightParen)?;
        
        Ok(format!("{}({})", strategy, field))
    }
    
    fn parse_table_body(&mut self, table: &mut TableNode) -> Result<(), ParseError> {
        self.expect(TokenType::LeftBrace)?;
        
        // Parse columns
        table.columns = self.parse_column_list()?;
        
        // Parse indexes (optional)
        if self.check(TokenType::Indexes) {
            self.parse_indexes(table)?;
        }
        
        self.expect(TokenType::RightBrace)?;
        Ok(())
    }
    
    fn parse_column_list(&mut self) -> Result<Vec<ColumnNode>, ParseError> {
        let mut columns = Vec::new();
        
        while !self.check(TokenType::Indexes) && !self.check(TokenType::RightBrace) && !self.is_at_end() {
            if let TokenType::Identifier(_) = &self.peek().token_type {
                columns.push(self.parse_column()?);
                
                if self.check(TokenType::Comma) {
                    self.advance();
                }
            } else {
                break;
            }
        }
        
        Ok(columns)
    }
    
    fn parse_column(&mut self) -> Result<ColumnNode, ParseError> {
        let name = self.expect_identifier()?;
        self.expect(TokenType::Colon)?;
        let data_type = self.parse_data_type()?;
        
        let mut column = ColumnNode {
            name,
            data_type,
            is_primary_key: false,
            is_unique: false,
        };
        
        // Parse constraints
        while self.check(TokenType::Unique) || self.check(TokenType::PrimaryKey) {
            match &self.peek().token_type {
                TokenType::Unique => {
                    self.advance();
                    column.is_unique = true;
                }
                TokenType::PrimaryKey => {
                    self.advance();
                    column.is_primary_key = true;
                }
                _ => break,
            }
        }
        
        Ok(column)
    }
    
    fn parse_data_type(&mut self) -> Result<String, ParseError> {
        let type_token = self.advance();
        
        match &type_token.token_type {
            TokenType::Varchar => {
                self.expect(TokenType::LeftParen)?;
                let size = self.expect_integer()?;
                self.expect(TokenType::RightParen)?;
                Ok(format!("VARCHAR({})", size))
            }
            TokenType::Decimal => {
                self.expect(TokenType::LeftParen)?;
                let precision = self.expect_integer()?;
                self.expect(TokenType::Comma)?;
                let scale = self.expect_integer()?;
                self.expect(TokenType::RightParen)?;
                Ok(format!("DECIMAL({}, {})", precision, scale))
            }
            TokenType::Uuid => Ok("UUID".to_string()),
            TokenType::Int => Ok("INT".to_string()),
            TokenType::BigInt => Ok("BIGINT".to_string()),
            TokenType::Timestamp => Ok("TIMESTAMP".to_string()),
            TokenType::Geometry => Ok("GEOMETRY".to_string()),
            TokenType::Identifier(name) => Ok(name.clone()),
            _ => Err(ParseError::UnexpectedToken {
                expected: "data type".to_string(),
                found: type_token,
            }),
        }
    }
    
    fn parse_indexes(&mut self, table: &mut TableNode) -> Result<(), ParseError> {
        self.expect(TokenType::Indexes)?;
        self.expect(TokenType::LeftBrace)?;
        
        while !self.check(TokenType::RightBrace) {
            let index_type = self.expect_identifier()?;
            self.expect(TokenType::Colon)?;
            self.expect(TokenType::LeftBracket)?;
            let columns = self.parse_identifier_list()?;
            self.expect(TokenType::RightBracket)?;
            
            table.indexes.insert(index_type, columns);
        }
        
        self.expect(TokenType::RightBrace)?;
        Ok(())
    }
    
    // ========================================================================
    // OTHER BLOCKS (Simplified - expand as needed)
    // ========================================================================
    
    fn parse_vectorization(&mut self) -> Result<VectorizationNode, ParseError> {
        self.expect(TokenType::Vectorization)?;
        self.expect(TokenType::LeftBrace)?;
        
        let mut node = VectorizationNode::new();
        
        while !self.check(TokenType::RightBrace) {
            self.advance(); // Skip content for now
        }
        
        self.expect(TokenType::RightBrace)?;
        Ok(node)
    }
    
    fn parse_command(&mut self) -> Result<CommandNode, ParseError> {
        self.expect(TokenType::Command)?;
        let name = self.expect_identifier()?;
        self.expect(TokenType::LeftBrace)?;
        
        let node = CommandNode::new(name);
        
        // Skip body for now
        while !self.check(TokenType::RightBrace) {
            self.advance();
        }
        
        self.expect(TokenType::RightBrace)?;
        Ok(node)
    }
    
    fn parse_rag_query(&mut self) -> Result<RagQueryNode, ParseError> {
        self.expect(TokenType::RagQuery)?;
        let name = self.expect_identifier()?;
        self.expect(TokenType::LeftBrace)?;
        
        let node = RagQueryNode::new(name);
        
        // Skip body for now
        while !self.check(TokenType::RightBrace) {
            self.advance();
        }
        
        self.expect(TokenType::RightBrace)?;
        Ok(node)
    }
    
    fn parse_presentation(&mut self) -> Result<PresentationNode, ParseError> {
        self.expect(TokenType::Presentation)?;
        self.expect(TokenType::LeftBrace)?;
        
        let mut node = PresentationNode::new();
        
        while !self.check(TokenType::RightBrace) {
            if self.check(TokenType::Style) {
                node.styles.push(self.parse_style()?);
            } else {
                self.advance();
            }
        }
        
        self.expect(TokenType::RightBrace)?;
        Ok(node)
    }
    
    fn parse_style(&mut self) -> Result<StyleNode, ParseError> {
        self.expect(TokenType::Style)?;
        let target_entity = self.expect_identifier()?;
        self.expect(TokenType::LeftBrace)?;
        
        let mut style = StyleNode {
            target_entity,
            color: String::new(),
            icon: String::new(),
            shape: String::new(),
            size_by_field: String::new(),
        };
        
        while !self.check(TokenType::RightBrace) {
            match &self.peek().token_type {
                TokenType::Color => {
                    self.advance();
                    self.expect(TokenType::Colon)?;
                    style.color = self.expect_string()?;
                }
                TokenType::Shape => {
                    self.advance();
                    self.expect(TokenType::Colon)?;
                    style.shape = self.expect_identifier()?;
                }
                _ => self.advance(),
            }
        }
        
        self.expect(TokenType::RightBrace)?;
        Ok(style)
    }
    
    fn parse_rule_set(&mut self) -> Result<RuleSetNode, ParseError> {
        self.expect(TokenType::RuleSet)?;
        let name = self.expect_identifier()?;
        self.expect(TokenType::LeftBrace)?;
        
        let node = RuleSetNode::new(name);
        
        // Skip body for now
        while !self.check(TokenType::RightBrace) {
            self.advance();
        }
        
        self.expect(TokenType::RightBrace)?;
        Ok(node)
    }
    
    fn parse_meta_algorithmics(&mut self) -> Result<MetaAlgorithmicsNode, ParseError> {
        self.expect(TokenType::MetaAlgorithmics)?;
        self.expect(TokenType::LeftBrace)?;
        
        let node = MetaAlgorithmicsNode::new();
        
        // Skip body for now
        while !self.check(TokenType::RightBrace) {
            self.advance();
        }
        
        self.expect(TokenType::RightBrace)?;
        Ok(node)
    }
    
    // ========================================================================
    // HELPER METHODS
    // ========================================================================
    
    fn parse_identifier_list(&mut self) -> Result<Vec<String>, ParseError> {
        let mut list = Vec::new();
        
        list.push(self.expect_identifier()?);
        
        while self.check(TokenType::Comma) {
            self.advance();
            list.push(self.expect_identifier()?);
        }
        
        Ok(list)
    }
    
    fn expect(&mut self, token_type: TokenType) -> Result<Token, ParseError> {
        if self.check(token_type.clone()) {
            Ok(self.advance())
        } else {
            Err(ParseError::ExpectedToken {
                expected: format!("{:?}", token_type),
                found: self.peek().clone(),
            })
        }
    }
    
    fn expect_identifier(&mut self) -> Result<String, ParseError> {
        match &self.advance().token_type {
            TokenType::Identifier(name) => Ok(name.clone()),
            other => Err(ParseError::ExpectedIdentifier {
                found: format!("{:?}", other),
            }),
        }
    }
    
    fn expect_string(&mut self) -> Result<String, ParseError> {
        match &self.advance().token_type {
            TokenType::String(s) => Ok(s.clone()),
            other => Err(ParseError::ExpectedString {
                found: format!("{:?}", other),
            }),
        }
    }
    
    fn expect_integer(&mut self) -> Result<i64, ParseError> {
        match &self.advance().token_type {
            TokenType::Integer(n) => Ok(*n),
            other => Err(ParseError::ExpectedNumber {
                found: format!("{:?}", other),
            }),
        }
    }
    
    fn expect_float(&mut self) -> Result<f64, ParseError> {
        match &self.advance().token_type {
            TokenType::Float(f) => Ok(*f),
            TokenType::Integer(i) => Ok(*i as f64),
            other => Err(ParseError::ExpectedNumber {
                found: format!("{:?}", other),
            }),
        }
    }
    
    fn check(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        std::mem::discriminant(&self.peek().token_type) == std::mem::discriminant(&token_type)
    }
    
    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }
    
    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.tokens[self.current - 1].clone()
    }
    
    fn is_at_end(&self) -> bool {
        matches!(self.tokens[self.current].token_type, TokenType::Eof)
    }
}

// ============================================================================
// ERRORS
// ============================================================================

#[derive(Debug)]
pub enum ParseError {
    LexError(String),
    UnexpectedToken { expected: String, found: Token },
    ExpectedToken { expected: String, found: Token },
    ExpectedIdentifier { found: String },
    ExpectedString { found: String },
    ExpectedNumber { found: String },
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::LexError(msg) => write!(f, "Lexer error: {}", msg),
            ParseError::UnexpectedToken { expected, found } => {
                write!(f, "Expected {}, found {:?} at line {}", expected, found.token_type, found.line)
            }
            ParseError::ExpectedToken { expected, found } => {
                write!(f, "Expected token {}, found {:?} at line {}", expected, found.token_type, found.line)
            }
            ParseError::ExpectedIdentifier { found } => {
                write!(f, "Expected identifier, found {}", found)
            }
            ParseError::ExpectedString { found } => {
                write!(f, "Expected string, found {}", found)
            }
            ParseError::ExpectedNumber { found } => {
                write!(f, "Expected number, found {}", found)
            }
        }
    }
}

impl std::error::Error for ParseError {}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_context() {
        let source = "CONTEXT TestContext { }";
        let mut parser = AkkadianParser::new(source).unwrap();
        let program = parser.parse().unwrap();
        
        assert_eq!(program.contexts.len(), 1);
        assert_eq!(program.contexts[0].name, "TestContext");
    }

    #[test]
    fn test_parse_identity() {
        let source = r#"
            CONTEXT Test {
                IDENTITY Person {
                    BUSINESS_KEY: name, email
                }
            }
        "#;
        
        let mut parser = AkkadianParser::new(source).unwrap();
        let program = parser.parse().unwrap();
        
        assert_eq!(program.contexts[0].identities.len(), 1);
        assert_eq!(program.contexts[0].identities[0].business_keys.len(), 2);
    }

    #[test]
    fn test_parse_hub() {
        let source = r#"
            CONTEXT Test {
                STORAGE DataVault {
                    HUB: PersonHub {
                        id: UUID PRIMARY KEY,
                        name: VARCHAR(100)
                    }
                }
            }
        "#;
        
        let mut parser = AkkadianParser::new(source).unwrap();
        let program = parser.parse().unwrap();
        
        assert!(program.contexts[0].storage.is_some());
        let storage = program.contexts[0].storage.as_ref().unwrap();
        assert_eq!(storage.hubs.len(), 1);
        assert_eq!(storage.hubs[0].columns.len(), 2);
    }
}
