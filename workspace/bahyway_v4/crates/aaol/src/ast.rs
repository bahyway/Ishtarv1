//! AAOL AST — abstract syntax tree for `.akk` programs.

/// A complete AAOL program.
#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Statement>,
}

/// Top-level statement in an AAOL program.
#[derive(Debug, Clone)]
pub enum Statement {
    /// `tribe <Name> { <body> }`
    TribeDecl(TribeDecl),
    /// `when event "<name>" then <action>;`
    EventRule(EventRule),
    /// `route <event> to <actor>;`
    RouteDecl(RouteDecl),
}

/// `tribe Name { actor* }`
#[derive(Debug, Clone)]
pub struct TribeDecl {
    pub name: String,
    pub actors: Vec<ActorDecl>,
}

/// `actor Name { role <role> }`
#[derive(Debug, Clone)]
pub struct ActorDecl {
    pub name: String,
    pub role: Option<String>,
}

/// `when event "evt" then <action>;`
#[derive(Debug, Clone)]
pub struct EventRule {
    pub event_name: String,
    pub action: Action,
}

/// Action in an event rule.
#[derive(Debug, Clone)]
pub enum Action {
    EmitSnapshot,
    Custom(String),
}

/// `route <event> to <actor>;`
#[derive(Debug, Clone)]
pub struct RouteDecl {
    pub event_name: String,
    pub actor_name: String,
}

/// Minimal recursive-descent parser — converts token stream to Program AST.
///
/// Grammar (informal):
///   program    := statement*
///   statement  := tribe_decl | event_rule | route_decl
///   tribe_decl := TRIBE IDENT LBRACE actor_decl* RBRACE
///   actor_decl := ACTOR IDENT LBRACE role_decl? RBRACE
///   role_decl  := ROLE IDENT SEMICOLON?
///   event_rule := WHEN EVENT STRING THEN action SEMICOLON
///   route_decl := ROUTE IDENT TO IDENT SEMICOLON
///   action     := EMIT SNAPSHOT | IDENT
pub struct Parser {
    tokens: Vec<super::token::Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<super::token::Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    pub fn parse(&mut self) -> Result<Program, String> {
        let mut stmts = Vec::new();
        while !self.is_eof() {
            stmts.push(self.parse_statement()?);
        }
        Ok(Program { statements: stmts })
    }

    fn peek(&self) -> &super::token::Token {
        &self.tokens[self.pos]
    }

    fn advance(&mut self) -> super::token::Token {
        let t = self.tokens[self.pos].clone();
        self.pos += 1;
        t
    }

    fn is_eof(&self) -> bool {
        matches!(self.peek(), super::token::Token::Eof)
    }

    fn expect_ident(&mut self) -> Result<String, String> {
        match self.advance() {
            super::token::Token::Ident(s) => Ok(s),
            t => Err(format!("Expected identifier, got {t:?}")),
        }
    }

    fn expect_string(&mut self) -> Result<String, String> {
        match self.advance() {
            super::token::Token::StringLit(s) => Ok(s),
            t => Err(format!("Expected string literal, got {t:?}")),
        }
    }

    fn expect_token(&mut self, expected: &super::token::Token) -> Result<(), String> {
        let tok = self.advance();
        if &tok == expected {
            Ok(())
        } else {
            Err(format!("Expected {expected:?}, got {tok:?}"))
        }
    }

    fn parse_statement(&mut self) -> Result<Statement, String> {
        use super::token::Token::*;
        match self.peek().clone() {
            Tribe => self.parse_tribe_decl().map(Statement::TribeDecl),
            When => self.parse_event_rule().map(Statement::EventRule),
            Route => self.parse_route_decl().map(Statement::RouteDecl),
            t => Err(format!("Unexpected token at statement level: {t:?}")),
        }
    }

    fn parse_tribe_decl(&mut self) -> Result<TribeDecl, String> {
        use super::token::Token::*;
        self.advance(); // consume TRIBE
        let name = self.expect_ident()?;
        self.expect_token(&LBrace)?;
        let mut actors = Vec::new();
        while !matches!(self.peek(), RBrace | Eof) {
            actors.push(self.parse_actor_decl()?);
        }
        self.expect_token(&RBrace)?;
        Ok(TribeDecl { name, actors })
    }

    fn parse_actor_decl(&mut self) -> Result<ActorDecl, String> {
        use super::token::Token::*;
        self.expect_token(&Actor)?;
        let name = self.expect_ident()?;
        self.expect_token(&LBrace)?;
        let role = if matches!(self.peek(), Role) {
            self.advance(); // consume ROLE
            let r = self.expect_ident()?;
            // optional semicolon
            if matches!(self.peek(), Semicolon) {
                self.advance();
            }
            Some(r)
        } else {
            None
        };
        self.expect_token(&RBrace)?;
        Ok(ActorDecl { name, role })
    }

    fn parse_event_rule(&mut self) -> Result<EventRule, String> {
        use super::token::Token::*;
        self.advance(); // consume WHEN
        self.expect_token(&Event)?;
        let event_name = self.expect_string()?;
        self.expect_token(&Then)?;
        let action = self.parse_action()?;
        if matches!(self.peek(), Semicolon) {
            self.advance();
        }
        Ok(EventRule { event_name, action })
    }

    fn parse_action(&mut self) -> Result<Action, String> {
        use super::token::Token::*;
        match self.advance() {
            Emit => {
                if matches!(self.peek(), Snapshot) {
                    self.advance();
                }
                Ok(Action::EmitSnapshot)
            }
            Ident(s) => Ok(Action::Custom(s)),
            t => Err(format!("Expected action, got {t:?}")),
        }
    }

    fn parse_route_decl(&mut self) -> Result<RouteDecl, String> {
        use super::token::Token::*;
        self.advance(); // consume ROUTE
        let event_name = self.expect_ident()?;
        self.expect_token(&To)?;
        let actor_name = self.expect_ident()?;
        if matches!(self.peek(), Semicolon) {
            self.advance();
        }
        Ok(RouteDecl {
            event_name,
            actor_name,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::tokenize;

    fn parse(src: &str) -> Program {
        let toks = tokenize(src).unwrap();
        Parser::new(toks).parse().unwrap()
    }

    #[test]
    fn parse_empty_tribe() {
        let p = parse("tribe Sumer { }");
        assert_eq!(p.statements.len(), 1);
        if let Statement::TribeDecl(t) = &p.statements[0] {
            assert_eq!(t.name, "Sumer");
            assert!(t.actors.is_empty());
        } else {
            panic!("expected TribeDecl");
        }
    }

    #[test]
    fn parse_tribe_with_actor() {
        let p = parse("tribe Uruk { actor Scribe { role Zikru } }");
        if let Statement::TribeDecl(t) = &p.statements[0] {
            assert_eq!(t.actors.len(), 1);
            assert_eq!(t.actors[0].name, "Scribe");
            assert_eq!(t.actors[0].role, Some("Zikru".into()));
        } else {
            panic!("expected TribeDecl");
        }
    }

    #[test]
    fn parse_event_rule() {
        let p = parse("when event \"particle.created\" then emit snapshot;");
        assert_eq!(p.statements.len(), 1);
        if let Statement::EventRule(r) = &p.statements[0] {
            assert_eq!(r.event_name, "particle.created");
            assert!(matches!(r.action, Action::EmitSnapshot));
        } else {
            panic!("expected EventRule");
        }
    }

    #[test]
    fn parse_route_decl() {
        let p = parse("route particle_created to Scribe;");
        if let Statement::RouteDecl(r) = &p.statements[0] {
            assert_eq!(r.event_name, "particle_created");
            assert_eq!(r.actor_name, "Scribe");
        } else {
            panic!("expected RouteDecl");
        }
    }

    #[test]
    fn parse_multiple_statements() {
        let src = r#"
            tribe Sumer { actor Scribe { role Zikru } }
            when event "snap" then emit snapshot;
            route snap to Scribe;
        "#;
        let p = parse(src);
        assert_eq!(p.statements.len(), 3);
    }
}
