//! AkkadianAOL v3.5 — Sovereign Code Generator
//! AkkAST → target language source code
//!
//! Targets: Rust · Python · JSON (EnkiDB EAV) · PowerShell · XML

#![allow(dead_code)]

use crate::compiler::parser::{
    AkkDecl, AkkExpr, AkkFile, AkkPolicyDecl, AkkSignDecl, AkkTablet, AkkWordDecl, RuleKind,
};

// ── CodeGenTarget ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodeGenTarget {
    Rust,
    Python,
    Json,
    PowerShell,
    Xml,
}

impl std::fmt::Display for CodeGenTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rust => write!(f, "Rust"),
            Self::Python => write!(f, "Python"),
            Self::Json => write!(f, "JSON"),
            Self::PowerShell => write!(f, "PowerShell"),
            Self::Xml => write!(f, "XML"),
        }
    }
}

impl CodeGenTarget {
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Rust => "rs",
            Self::Python => "py",
            Self::Json => "json",
            Self::PowerShell => "ps1",
            Self::Xml => "xml",
        }
    }
}

#[derive(Debug, Clone)]
pub struct CodeGenOutput {
    pub filename: String,
    pub source: String,
    pub target: CodeGenTarget,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn expr_to_rust(expr: &AkkExpr) -> String {
    match expr {
        AkkExpr::Int(n) => n.to_string(),
        AkkExpr::Float(v) => format!("{}_f64", v),
        AkkExpr::Str(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
        AkkExpr::Bool(b) => b.to_string(),
        AkkExpr::Glyph(c) => format!("'{}'", c),
        AkkExpr::MeszlRef(n) => n.to_string(),
        AkkExpr::Ident(s) => s.clone(),
        AkkExpr::Path(parts) => parts.join("::"),
    }
}

fn expr_to_python(expr: &AkkExpr) -> String {
    match expr {
        AkkExpr::Int(n) => n.to_string(),
        AkkExpr::Float(v) => v.to_string(),
        AkkExpr::Str(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
        AkkExpr::Bool(b) => {
            if *b {
                "True".into()
            } else {
                "False".into()
            }
        }
        AkkExpr::Glyph(c) => format!("\"{}\"", c),
        AkkExpr::MeszlRef(n) => n.to_string(),
        AkkExpr::Ident(s) => format!("\"{}\"", s),
        AkkExpr::Path(parts) => format!("\"{}\"", parts.join(".")),
    }
}

fn expr_to_json(expr: &AkkExpr) -> String {
    match expr {
        AkkExpr::Int(n) => n.to_string(),
        AkkExpr::Float(v) => v.to_string(),
        AkkExpr::Str(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
        AkkExpr::Bool(b) => b.to_string(),
        AkkExpr::Glyph(c) => format!("\"{}\"", c),
        AkkExpr::MeszlRef(n) => n.to_string(),
        AkkExpr::Ident(s) => format!("\"{}\"", s),
        AkkExpr::Path(parts) => format!("\"{}\"", parts.join(".")),
    }
}

fn expr_to_ps(expr: &AkkExpr) -> String {
    match expr {
        AkkExpr::Int(n) => n.to_string(),
        AkkExpr::Float(v) => v.to_string(),
        AkkExpr::Str(s) => format!("\"{}\"", s.replace('"', "`\"")),
        AkkExpr::Bool(b) => {
            if *b {
                "$true".into()
            } else {
                "$false".into()
            }
        }
        AkkExpr::Glyph(c) => format!("\"{}\"", c),
        AkkExpr::MeszlRef(n) => n.to_string(),
        AkkExpr::Ident(s) => format!("\"{}\"", s),
        AkkExpr::Path(parts) => format!("\"{}\"", parts.join(".")),
    }
}

fn expr_to_xml(expr: &AkkExpr) -> String {
    match expr {
        AkkExpr::Int(n) => n.to_string(),
        AkkExpr::Float(v) => v.to_string(),
        AkkExpr::Str(s) => xml_escape(s),
        AkkExpr::Bool(b) => b.to_string(),
        AkkExpr::Glyph(c) => c.to_string(),
        AkkExpr::MeszlRef(n) => n.to_string(),
        AkkExpr::Ident(s) => xml_escape(s),
        AkkExpr::Path(parts) => xml_escape(&parts.join(".")),
    }
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect()
}

fn rust_type(key: &str, expr: &AkkExpr) -> &'static str {
    match key {
        "meszl" => "u32",
        "glyph" => "char",
        _ => match expr {
            AkkExpr::Int(_) => "i64",
            AkkExpr::Float(_) => "f64",
            AkkExpr::Str(_) => "&'static str",
            AkkExpr::Bool(_) => "bool",
            AkkExpr::Glyph(_) => "char",
            AkkExpr::MeszlRef(_) => "u32",
            AkkExpr::Ident(_) | AkkExpr::Path(_) => "&'static str",
        },
    }
}

// ── CodeGen ───────────────────────────────────────────────────────────────────

pub struct CodeGen {
    pub target: CodeGenTarget,
}

impl CodeGen {
    pub fn new(target: CodeGenTarget) -> Self {
        Self { target }
    }

    pub fn generate_all(&self, ast: &AkkFile) -> Vec<CodeGenOutput> {
        ast.tablets
            .iter()
            .map(|t| self.generate_tablet(t))
            .collect()
    }

    pub fn generate(&self, ast: &AkkFile) -> String {
        if ast.tablets.is_empty() {
            return format!(
                "// AkkadianAOL v3.5 CodeGen ({}) — empty file\n",
                self.target
            );
        }
        self.generate_all(ast)
            .iter()
            .map(|o| o.source.clone())
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    pub fn generate_tablet(&self, tablet: &AkkTablet) -> CodeGenOutput {
        let source = match self.target {
            CodeGenTarget::Rust => self.tablet_to_rust(tablet),
            CodeGenTarget::Python => self.tablet_to_python(tablet),
            CodeGenTarget::Json => self.tablet_to_json(tablet),
            CodeGenTarget::PowerShell => self.tablet_to_ps(tablet),
            CodeGenTarget::Xml => self.tablet_to_xml(tablet),
        };
        CodeGenOutput {
            filename: format!("{}.{}", tablet.name, self.target.extension()),
            source,
            target: self.target,
        }
    }

    // ── Rust ──────────────────────────────────────────────────────────────────

    fn tablet_to_rust(&self, tablet: &AkkTablet) -> String {
        let mut out = format!(
            "//! AkkadianAOL v3.5 — Generated Rust\n//! Tablet: {}\n",
            tablet.name
        );
        if let Some(ver) = tablet.version() {
            out.push_str(&format!("//! Version: {}\n", ver));
        }
        out.push_str("//! DO NOT EDIT — regenerate from .akk source\n\n");
        out.push_str("#![allow(dead_code, non_snake_case)]\n\n");
        out.push_str(&format!("pub mod {} {{\n", tablet.name));

        for decl in &tablet.decls {
            match decl {
                AkkDecl::Sign(sign) => out.push_str(&self.sign_to_rust(sign)),
                AkkDecl::Word(word) => out.push_str(&self.word_to_rust(word)),
                AkkDecl::Policy(policy) => out.push_str(&self.policy_to_rust(policy)),
                AkkDecl::Seek(seek) => {
                    out.push_str("    /// Generated seek query constants\n");
                    for field in &seek.fields {
                        let val = expr_to_rust(&field.value);
                        let ty = rust_type(&field.key, &field.value);
                        out.push_str(&format!(
                            "    pub const SEEK_{}: {} = {};\n",
                            field.key.to_uppercase(),
                            ty,
                            val
                        ));
                    }
                    out.push('\n');
                }
                _ => {}
            }
        }
        out.push_str("}\n");
        out
    }

    fn sign_to_rust(&self, sign: &AkkSignDecl) -> String {
        let struct_name = to_pascal_case(&sign.name);
        let mut out = String::new();
        if let Some(cite) = sign.cite() {
            out.push_str(&format!("    /// @cite {}\n", cite));
        }
        out.push_str(&format!("    /// Cuneiform sign: {}\n", sign.name));
        out.push_str("    #[derive(Debug, Clone, Copy)]\n");
        out.push_str(&format!("    pub struct {} {{\n", struct_name));
        for field in &sign.fields {
            out.push_str(&format!(
                "        pub {}: {},\n",
                field.key,
                rust_type(&field.key, &field.value)
            ));
        }
        out.push_str("    }\n\n");
        out.push_str(&format!("    impl {} {{\n", struct_name));
        out.push_str(&format!(
            "        pub const INSTANCE: {} = {} {{\n",
            struct_name, struct_name
        ));
        for field in &sign.fields {
            out.push_str(&format!(
                "            {}: {},\n",
                field.key,
                expr_to_rust(&field.value)
            ));
        }
        out.push_str("        };\n    }\n\n");
        out
    }

    fn word_to_rust(&self, word: &AkkWordDecl) -> String {
        let struct_name = to_pascal_case(&word.name);
        let mut out = String::new();
        if let Some(cite) = word.cite() {
            out.push_str(&format!("    /// @cite {}\n", cite));
        }
        out.push_str(&format!("    /// Lexeme: {}\n", word.name));
        out.push_str("    #[derive(Debug, Clone)]\n");
        out.push_str(&format!("    pub struct {} {{\n", struct_name));
        for field in &word.fields {
            out.push_str(&format!(
                "        pub {}: {},\n",
                field.key,
                rust_type(&field.key, &field.value)
            ));
        }
        out.push_str("    }\n\n");
        out
    }

    fn policy_to_rust(&self, policy: &AkkPolicyDecl) -> String {
        let mut out = format!(
            "    /// AkkadianAOL policy: {}\n    /// Sovereign enforcement\n",
            policy.name
        );
        out.push_str(&format!(
            "    pub fn enforce_{}(subject: &str, action: &str) -> bool {{\n",
            policy.name
        ));
        for rule in &policy.rules {
            let verdict = match rule.kind {
                RuleKind::Allow => "true",
                RuleKind::Deny => "false",
            };
            let subj = rule
                .fields
                .iter()
                .find(|f| f.key == "subject")
                .map(|f| expr_to_rust(&f.value))
                .unwrap_or_else(|| "\"*\"".into());
            let act = rule
                .fields
                .iter()
                .find(|f| f.key == "action")
                .map(|f| expr_to_rust(&f.value))
                .unwrap_or_else(|| "\"*\"".into());
            out.push_str(&format!(
                "        if subject == {} && action == {} {{ return {}; }}\n",
                subj, act, verdict
            ));
        }
        out.push_str("        false // sovereign default: deny\n    }\n\n");
        out
    }

    // ── Python ────────────────────────────────────────────────────────────────

    fn tablet_to_python(&self, tablet: &AkkTablet) -> String {
        let mut out = format!(
            "# AkkadianAOL v3.5 — Generated Python\n# Tablet: {}\n",
            tablet.name
        );
        if let Some(ver) = tablet.version() {
            out.push_str(&format!("# Version: {}\n", ver));
        }
        out.push_str("# DO NOT EDIT — regenerate from .akk source\n\nfrom dataclasses import dataclass\nfrom typing import Optional\n\n");
        for decl in &tablet.decls {
            match decl {
                AkkDecl::Sign(sign) => out.push_str(&self.sign_to_python(sign)),
                AkkDecl::Word(word) => out.push_str(&self.word_to_python(word)),
                AkkDecl::Policy(policy) => out.push_str(&self.policy_to_python(policy)),
                _ => {}
            }
        }
        out
    }

    fn sign_to_python(&self, sign: &AkkSignDecl) -> String {
        let class_name = to_pascal_case(&sign.name);
        let mut out = String::new();
        if let Some(cite) = sign.cite() {
            out.push_str(&format!("# @cite {}\n", cite));
        }
        out.push_str(&format!("@dataclass\nclass {}:\n", class_name));
        if sign.fields.is_empty() {
            out.push_str("    pass\n\n");
        } else {
            for field in &sign.fields {
                out.push_str(&format!(
                    "    {}: object = {}\n",
                    field.key,
                    expr_to_python(&field.value)
                ));
            }
            out.push_str(&format!(
                "\n{}_INSTANCE = {}(\n",
                sign.name.to_uppercase(),
                class_name
            ));
            for field in &sign.fields {
                out.push_str(&format!(
                    "    {}={},\n",
                    field.key,
                    expr_to_python(&field.value)
                ));
            }
            out.push_str(")\n\n");
        }
        out
    }

    fn word_to_python(&self, word: &AkkWordDecl) -> String {
        let class_name = to_pascal_case(&word.name);
        let mut out = String::new();
        if let Some(cite) = word.cite() {
            out.push_str(&format!("# @cite {}\n", cite));
        }
        out.push_str(&format!("@dataclass\nclass {}:\n", class_name));
        if word.fields.is_empty() {
            out.push_str("    pass\n\n");
        } else {
            for field in &word.fields {
                out.push_str(&format!(
                    "    {}: object = {}\n",
                    field.key,
                    expr_to_python(&field.value)
                ));
            }
            out.push('\n');
        }
        out
    }

    fn policy_to_python(&self, policy: &AkkPolicyDecl) -> String {
        let mut out = format!(
            "def enforce_{}(subject: str, action: str) -> bool:\n    \"\"\"AkkadianAOL policy: {}\"\"\"\n",
            policy.name, policy.name
        );
        for rule in &policy.rules {
            let verdict = match rule.kind {
                RuleKind::Allow => "True",
                RuleKind::Deny => "False",
            };
            let subj = rule
                .fields
                .iter()
                .find(|f| f.key == "subject")
                .map(|f| expr_to_python(&f.value))
                .unwrap_or_else(|| "\"*\"".into());
            let act = rule
                .fields
                .iter()
                .find(|f| f.key == "action")
                .map(|f| expr_to_python(&f.value))
                .unwrap_or_else(|| "\"*\"".into());
            out.push_str(&format!(
                "    if subject == {} and action == {}: return {}\n",
                subj, act, verdict
            ));
        }
        out.push_str("    return False  # sovereign default: deny\n\n");
        out
    }

    // ── JSON ──────────────────────────────────────────────────────────────────

    fn tablet_to_json(&self, tablet: &AkkTablet) -> String {
        let mut entities: Vec<String> = Vec::new();
        for decl in &tablet.decls {
            match decl {
                AkkDecl::Sign(sign) => entities.push(self.sign_to_json_entity(sign, &tablet.name)),
                AkkDecl::Word(word) => entities.push(self.word_to_json_entity(word, &tablet.name)),
                AkkDecl::Policy(policy) => {
                    entities.push(self.policy_to_json_entity(policy, &tablet.name))
                }
                _ => {}
            }
        }
        let mut out = format!(
            "{{\n  \"akkadian_aol_version\": \"3.5\",\n  \"tablet\": \"{}\",\n",
            tablet.name
        );
        if let Some(ver) = tablet.version() {
            out.push_str(&format!("  \"version\": \"{}\",\n", ver));
        }
        out.push_str("  \"entities\": [\n");
        out.push_str(&entities.join(",\n"));
        out.push_str("\n  ]\n}\n");
        out
    }

    fn sign_to_json_entity(&self, sign: &AkkSignDecl, tablet: &str) -> String {
        let mut attrs: Vec<String> = sign
            .fields
            .iter()
            .map(|f| {
                format!(
                    "          {{\"attribute\": \"{}\", \"value\": {}}}",
                    f.key,
                    expr_to_json(&f.value)
                )
            })
            .collect();
        if let Some(cite) = sign.cite() {
            attrs.push(format!(
                "          {{\"attribute\": \"cite\", \"value\": \"{}\"}}",
                cite
            ));
        }
        format!(
            "    {{\n      \"kind\": \"sign\",\n      \"name\": \"{}\",\n      \
             \"tablet\": \"{}\",\n      \"domain_b10\": \"0xE0\",\n      \
             \"attributes\": [\n{}\n      ]\n    }}",
            sign.name,
            tablet,
            attrs.join(",\n")
        )
    }

    fn word_to_json_entity(&self, word: &AkkWordDecl, tablet: &str) -> String {
        let mut attrs: Vec<String> = word
            .fields
            .iter()
            .map(|f| {
                format!(
                    "          {{\"attribute\": \"{}\", \"value\": {}}}",
                    f.key,
                    expr_to_json(&f.value)
                )
            })
            .collect();
        if let Some(cite) = word.cite() {
            attrs.push(format!(
                "          {{\"attribute\": \"cite\", \"value\": \"{}\"}}",
                cite
            ));
        }
        format!(
            "    {{\n      \"kind\": \"word\",\n      \"name\": \"{}\",\n      \
             \"tablet\": \"{}\",\n      \"domain_b10\": \"0xE1\",\n      \
             \"attributes\": [\n{}\n      ]\n    }}",
            word.name,
            tablet,
            attrs.join(",\n")
        )
    }

    fn policy_to_json_entity(&self, policy: &AkkPolicyDecl, tablet: &str) -> String {
        let rules: Vec<String> = policy
            .rules
            .iter()
            .map(|rule| {
                let kind = match rule.kind {
                    RuleKind::Allow => "allow",
                    RuleKind::Deny => "deny",
                };
                let fields: Vec<String> = rule
                    .fields
                    .iter()
                    .map(|f| format!("\"{}\":{}", f.key, expr_to_json(&f.value)))
                    .collect();
                format!(
                    "          {{\"kind\":\"{}\"{}}}",
                    kind,
                    if fields.is_empty() {
                        String::new()
                    } else {
                        format!(",{}", fields.join(","))
                    }
                )
            })
            .collect();
        format!(
            "    {{\n      \"kind\": \"policy\",\n      \"name\": \"{}\",\n      \
             \"tablet\": \"{}\",\n      \"domain_b10\": \"0xE5\",\n      \
             \"rules\": [\n{}\n      ]\n    }}",
            policy.name,
            tablet,
            rules.join(",\n")
        )
    }

    // ── PowerShell ────────────────────────────────────────────────────────────

    fn tablet_to_ps(&self, tablet: &AkkTablet) -> String {
        let mut out = format!(
            "# AkkadianAOL v3.5 — Generated PowerShell\n# Tablet: {}\n",
            tablet.name
        );
        if let Some(ver) = tablet.version() {
            out.push_str(&format!("# Version: {}\n", ver));
        }
        out.push_str("# DO NOT EDIT — regenerate from .akk source\n\n");
        for decl in &tablet.decls {
            match decl {
                AkkDecl::Sign(sign) => out.push_str(&self.sign_to_ps(sign)),
                AkkDecl::Word(word) => out.push_str(&self.word_to_ps(word)),
                AkkDecl::Policy(policy) => out.push_str(&self.policy_to_ps(policy)),
                _ => {}
            }
        }
        out
    }

    fn sign_to_ps(&self, sign: &AkkSignDecl) -> String {
        let mut out = format!("# Cuneiform sign: {}\n", sign.name);
        if let Some(cite) = sign.cite() {
            out.push_str(&format!("# @cite {}\n", cite));
        }
        out.push_str(&format!("${} = [PSCustomObject]@{{\n", sign.name));
        for field in &sign.fields {
            out.push_str(&format!(
                "    {} = {}\n",
                field.key,
                expr_to_ps(&field.value)
            ));
        }
        out.push_str("}\n\n");
        out
    }

    fn word_to_ps(&self, word: &AkkWordDecl) -> String {
        let mut out = format!("# Lexeme: {}\n", word.name);
        if let Some(cite) = word.cite() {
            out.push_str(&format!("# @cite {}\n", cite));
        }
        out.push_str(&format!("${} = [PSCustomObject]@{{\n", word.name));
        for field in &word.fields {
            out.push_str(&format!(
                "    {} = {}\n",
                field.key,
                expr_to_ps(&field.value)
            ));
        }
        out.push_str("}\n\n");
        out
    }

    fn policy_to_ps(&self, policy: &AkkPolicyDecl) -> String {
        let mut out = format!(
            "function Invoke-{}Policy {{\n    param([string]$Subject, [string]$Action)\n",
            to_pascal_case(&policy.name)
        );
        for rule in &policy.rules {
            let verdict = match rule.kind {
                RuleKind::Allow => "$true",
                RuleKind::Deny => "$false",
            };
            let subj = rule
                .fields
                .iter()
                .find(|f| f.key == "subject")
                .map(|f| expr_to_ps(&f.value))
                .unwrap_or_else(|| "\"*\"".into());
            let act = rule
                .fields
                .iter()
                .find(|f| f.key == "action")
                .map(|f| expr_to_ps(&f.value))
                .unwrap_or_else(|| "\"*\"".into());
            out.push_str(&format!(
                "    if ($Subject -eq {} -and $Action -eq {}) {{ return {} }}\n",
                subj, act, verdict
            ));
        }
        out.push_str("    return $false # sovereign default: deny\n}\n\n");
        out
    }

    // ── XML ───────────────────────────────────────────────────────────────────

    fn tablet_to_xml(&self, tablet: &AkkTablet) -> String {
        let mut out = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        out.push_str("<!-- AkkadianAOL v3.5 — Generated XML -->\n");
        out.push_str(&format!("<tablet name=\"{}\"", tablet.name));
        if let Some(ver) = tablet.version() {
            out.push_str(&format!(" version=\"{}\"", ver));
        }
        out.push_str(">\n");
        for decl in &tablet.decls {
            match decl {
                AkkDecl::Sign(sign) => out.push_str(&self.sign_to_xml(sign)),
                AkkDecl::Word(word) => out.push_str(&self.word_to_xml(word)),
                AkkDecl::Policy(policy) => out.push_str(&self.policy_to_xml(policy)),
                _ => {}
            }
        }
        out.push_str("</tablet>\n");
        out
    }

    fn sign_to_xml(&self, sign: &AkkSignDecl) -> String {
        let mut out = format!("  <sign name=\"{}\"", sign.name);
        if let Some(cite) = sign.cite() {
            out.push_str(&format!(" cite=\"{}\"", cite));
        }
        out.push_str(">\n");
        for field in &sign.fields {
            out.push_str(&format!(
                "    <field key=\"{}\">{}</field>\n",
                field.key,
                expr_to_xml(&field.value)
            ));
        }
        out.push_str("  </sign>\n");
        out
    }

    fn word_to_xml(&self, word: &AkkWordDecl) -> String {
        let mut out = format!("  <word name=\"{}\"", word.name);
        if let Some(cite) = word.cite() {
            out.push_str(&format!(" cite=\"{}\"", cite));
        }
        out.push_str(">\n");
        for field in &word.fields {
            out.push_str(&format!(
                "    <field key=\"{}\">{}</field>\n",
                field.key,
                expr_to_xml(&field.value)
            ));
        }
        out.push_str("  </word>\n");
        out
    }

    fn policy_to_xml(&self, policy: &AkkPolicyDecl) -> String {
        let mut out = format!("  <policy name=\"{}\">\n", policy.name);
        for rule in &policy.rules {
            let kind = match rule.kind {
                RuleKind::Allow => "allow",
                RuleKind::Deny => "deny",
            };
            out.push_str(&format!("    <rule kind=\"{}\">\n", kind));
            for field in &rule.fields {
                out.push_str(&format!(
                    "      <field key=\"{}\">{}</field>\n",
                    field.key,
                    expr_to_xml(&field.value)
                ));
            }
            out.push_str("    </rule>\n");
        }
        out.push_str("  </policy>\n");
        out
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;

    fn generate(src: &str, target: CodeGenTarget) -> String {
        let (toks, _) = Lexer::lex(src);
        let ast = Parser::new(toks).parse();
        CodeGen::new(target).generate(&ast)
    }

    fn sign_src() -> &'static str {
        r#"tablet akkadian_dict {
            @version "3.5"
            sign an_dingir {
                meszl = #29
                glyph = 𒀭
                meaning = "sky, heaven, god"
                @cite MesZL
            }
        }"#
    }

    fn policy_src() -> &'static str {
        r#"tablet larsa_phone {
            policy camera_access {
                deny  subject = "any_app"    action = "camera.stream"
                allow subject = "akkadi_cam" action = "camera.stream"
            }
        }"#
    }

    #[test]
    fn test_codegen_targets() {
        assert_eq!(CodeGenTarget::Rust.to_string(), "Rust");
        assert_eq!(CodeGenTarget::Json.to_string(), "JSON");
        assert_eq!(CodeGenTarget::PowerShell.to_string(), "PowerShell");
    }

    #[test]
    fn test_rust_module_generated() {
        assert!(generate(sign_src(), CodeGenTarget::Rust).contains("pub mod akkadian_dict"));
    }

    #[test]
    fn test_rust_struct_generated() {
        let out = generate(sign_src(), CodeGenTarget::Rust);
        assert!(out.contains("pub struct AnDingir"));
    }

    #[test]
    fn test_rust_policy_generated() {
        let out = generate(policy_src(), CodeGenTarget::Rust);
        assert!(out.contains("pub fn enforce_camera_access"));
        assert!(out.contains("false // sovereign default: deny"));
    }

    #[test]
    fn test_python_dataclass_generated() {
        assert!(generate(sign_src(), CodeGenTarget::Python).contains("@dataclass"));
    }

    #[test]
    fn test_json_structure_generated() {
        let out = generate(sign_src(), CodeGenTarget::Json);
        assert!(out.contains("\"akkadian_aol_version\": \"3.5\""));
        assert!(out.contains("\"domain_b10\": \"0xE0\""));
    }

    #[test]
    fn test_ps_object_generated() {
        assert!(generate(sign_src(), CodeGenTarget::PowerShell).contains("[PSCustomObject]@{"));
    }

    #[test]
    fn test_xml_structure_generated() {
        let out = generate(sign_src(), CodeGenTarget::Xml);
        assert!(out.contains("<?xml version=\"1.0\""));
        assert!(out.contains("<sign name=\"an_dingir\" cite=\"MesZL\">"));
    }

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("an_dingir"), "AnDingir");
        assert_eq!(to_pascal_case("camera_access"), "CameraAccess");
    }

    #[test]
    fn test_xml_escape() {
        assert_eq!(xml_escape("a & b < c"), "a &amp; b &lt; c");
    }
}
