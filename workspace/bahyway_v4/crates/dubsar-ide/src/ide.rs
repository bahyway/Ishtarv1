//! DubSarIde — language services for the DubSar IDE (§12).
//!
//! DubSar (Sumerian: "tablet-writer") supports:
//!   - AAOL (.akk) source parsing and validation
//!   - HeptaScript (.hepta) query parsing and syntax checking
//!   - Template field completion hints

use aaol::ast::Parser as AaolParser;
use aaol::tokenize as aaol_tokenize;
use heptascript::parse_query;
use template_engine::TemplateRegistry;

/// The kind of source file being checked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceKind {
    Aaol,
    HeptaScript,
}

/// Diagnostic from the IDE language service.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub kind: SourceKind,
    pub message: String,
    pub is_error: bool,
}

/// DubSar IDE language service — validates source files.
pub struct DubSarIde {
    pub registry: TemplateRegistry,
}

impl DubSarIde {
    pub fn new() -> Self {
        DubSarIde {
            registry: TemplateRegistry::new(),
        }
    }

    /// Parse and validate an AAOL `.akk` source.
    pub fn check_aaol(&self, source: &str) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let tokens = match aaol_tokenize(source) {
            Ok(t) => t,
            Err(e) => {
                diags.push(Diagnostic {
                    kind: SourceKind::Aaol,
                    message: format!("Lexer error: {e}"),
                    is_error: true,
                });
                return diags;
            }
        };
        let mut parser = AaolParser::new(tokens);
        if let Err(e) = parser.parse() {
            diags.push(Diagnostic {
                kind: SourceKind::Aaol,
                message: format!("Parse error: {e}"),
                is_error: true,
            });
        }
        diags
    }

    /// Parse and validate a HeptaScript `.hepta` query.
    pub fn check_hepta(&self, source: &str) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        if let Err(e) = parse_query(source) {
            diags.push(Diagnostic {
                kind: SourceKind::HeptaScript,
                message: format!("Parse error: {e}"),
                is_error: true,
            });
        }
        diags
    }

    /// Return field completion hints for a template by name.
    pub fn complete_fields(&self, template_name: &str) -> Vec<String> {
        self.registry
            .get(template_name)
            .map(|t| t.fields.iter().map(|f| f.label.to_string()).collect())
            .unwrap_or_default()
    }
}

impl Default for DubSarIde {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_aaol_no_diagnostics() {
        let ide = DubSarIde::new();
        let src = "tribe Sumer { actor Scribe { role Zikru } }";
        let diags = ide.check_aaol(src);
        assert!(diags.is_empty(), "valid AAOL should have no diagnostics");
    }

    #[test]
    fn invalid_aaol_has_error() {
        let ide = DubSarIde::new();
        let diags = ide.check_aaol("tribe { }"); // missing name
        assert!(!diags.is_empty());
        assert!(diags[0].is_error);
    }

    #[test]
    fn valid_hepta_no_diagnostics() {
        let ide = DubSarIde::new();
        let diags = ide.check_hepta("WHO Citizens.E\nWHERE E[state] = \"Golden\"");
        assert!(diags.is_empty());
    }

    #[test]
    fn invalid_hepta_has_error() {
        let ide = DubSarIde::new();
        let diags = ide.check_hepta("SELECT ??? INVALID");
        assert!(!diags.is_empty());
        assert!(diags[0].is_error);
    }

    #[test]
    fn complete_fields_unknown_template() {
        let ide = DubSarIde::new();
        let hints = ide.complete_fields("nonexistent");
        assert!(hints.is_empty());
    }

    #[test]
    fn complete_fields_known_template() {
        use template_engine::{FieldSpec, Template};
        let mut ide = DubSarIde::new();
        ide.registry.register(Template::default_template(
            "t.test",
            "Test",
            &[FieldSpec::new(0x1A4B, "state", true)],
        ));
        let hints = ide.complete_fields("t.test");
        assert_eq!(hints, vec!["state"]);
    }
}
