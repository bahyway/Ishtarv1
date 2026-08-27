//! AkkadianAOL v3.5 — Semantic Analysis (Phase 1B stub)
//! Responsibilities (Phase 2+):
//!   - Type checking against data-structure-station types
//!   - @cite annotation validation against ScholarlySource enum
//!   - AkkadiSeal verification (via si crate)
//!   - KAKI B7 class byte assignment (ADR-014)
//!   - Scope resolution for tablet imports

use crate::compiler::lexer::Span;
use crate::compiler::parser::AkkFile;

#[derive(Debug, Clone)]
pub struct SemanticDiag {
    pub level: DiagLevel,
    pub message: String,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagLevel {
    Error,
    Warning,
    Info,
}

pub struct SemanticAnalyser {
    pub diagnostics: Vec<SemanticDiag>,
}

impl SemanticAnalyser {
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
        }
    }

    /// Phase 1B stub — passes AST unchanged with no diagnostics.
    pub fn analyse(&mut self, ast: AkkFile) -> AkkFile {
        ast
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.level == DiagLevel::Error)
    }
}

impl Default for SemanticAnalyser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;

    #[test]
    fn test_semantic_stub_compiles() {
        let (toks, _) = Lexer::lex("tablet test {}");
        let ast = Parser::new(toks).parse();
        let mut analyser = SemanticAnalyser::new();
        let _checked = analyser.analyse(ast);
        assert!(!analyser.has_errors());
    }
}
