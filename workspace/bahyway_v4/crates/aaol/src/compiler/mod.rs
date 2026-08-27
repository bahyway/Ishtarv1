//! AkkadianAOL v3.5 compiler pipeline — `.akk` source → AST → CodeGen.
//!
//! Pipeline:
//!   `.akk` source → `lexer::Lexer` → `Vec<SpannedToken>`
//!                → `parser::Parser` → `AkkFile` AST
//!                → `semantic::SemanticAnalyser` → checked AST
//!                → `codegen::CodeGen` → Rust / Python / JSON / PS / XML

pub mod codegen;
pub mod lexer;
pub mod lsp;
pub mod parser;
pub mod semantic;

pub use codegen::{CodeGen, CodeGenOutput, CodeGenTarget};
pub use lexer::{AOLCat, AkkToken, LexError, Lexer, Span, SpannedToken};
pub use parser::{
    AkkAnnotation, AkkAolMapDecl, AkkDecl, AkkExpr, AkkField, AkkFile, AkkIngestDecl, AkkMapping,
    AkkPolicyDecl, AkkRule, AkkSeekDecl, AkkSignDecl, AkkTablet, AkkWordDecl, AnnotationKind,
    ParseError, Parser, RuleKind,
};
pub use semantic::{DiagLevel, SemanticAnalyser, SemanticDiag};
