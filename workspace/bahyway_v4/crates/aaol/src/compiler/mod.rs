//! AkkadianAOL v3.5 compiler pipeline — `.akk` source → AST → CodeGen.
//!
//! Pipeline:
//!   `.akk` source → `lexer::Lexer` → `Vec<SpannedToken>`
//!                → `parser::Parser` → `AkkFile` AST
//!                → `semantic::SemanticAnalyser` → checked AST
//!                → `codegen::CodeGen` → Rust / Python / JSON / PS / XML

pub mod lexer;
pub mod parser;
pub mod semantic;
pub mod codegen;
pub mod lsp;

pub use lexer::{Lexer, AkkToken, AOLCat, Span, SpannedToken, LexError};
pub use parser::{
    Parser, ParseError,
    AkkFile, AkkTablet, AkkDecl, AkkField, AkkExpr,
    AkkSignDecl, AkkWordDecl, AkkAolMapDecl, AkkMapping,
    AkkSeekDecl, AkkIngestDecl, AkkPolicyDecl, AkkRule,
    AkkAnnotation, AnnotationKind, RuleKind,
};
pub use semantic::{SemanticAnalyser, SemanticDiag, DiagLevel};
pub use codegen::{CodeGen, CodeGenTarget, CodeGenOutput};
