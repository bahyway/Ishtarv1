//! OOO vocabulary gate — pre-adoption scan for SQL/relational/foreign-
//! database vocabulary and ports. Sealed law, stated directly by the
//! Architect: BahyWay's Seven EnkiDB Types are Triple-O (Orbit-Oriented
//! Ontology) — never SQL, never a foreign database's product name, port,
//! or paradigm word, "because they ARE NOT OOO and will NEVER BE."
//!
//! `heptascript::operations::Operation::parse()` already enforces this at
//! the query-*language* level (`OperationError::SqlForbidden` rejects
//! `SELECT`/`INSERT`/`JOIN`/... as HeptaScript verb tokens) — but that
//! gate only ever sees real HeptaScript source text being parsed. It
//! never sees a Godot wizard scene's button label, a design-prototype
//! HTML file used as a layout reference, or a doc's prose — none of
//! those ever pass through the HeptaScript parser. This module is that
//! missing gate, over exactly those surfaces, following `zip_scan`'s own
//! pattern (pure pattern matching, no third-party dependency, scan
//! before adoption rather than trust and hope).
//!
//! Concretely: when an AI-generated or third-party mockup is used as a
//! *layout* reference for a real DubSar screen (as happened with an
//! uploaded prototype whose six database cards were labelled "Relational
//! SQL" / port 5432, "Object Document" / port 27017, "Multi-Model" / port
//! 9042 — Postgres's, MongoDB's, and Cassandra's real default ports),
//! this scanner is what a human or an Ansible playbook runs against the
//! adopted file *before* its content is trusted to survive into
//! `godot/dubsar-theater/`.

use std::fs;
use std::path::{Path, PathBuf};

/// Canonical forbidden vocabulary — paradigm words and foreign product
/// names. New entries must be appended (never reordered), same
/// stability convention `zip_scan::MALWARE_SIGS` uses, since a
/// `FoundTerm`'s index may be logged/referenced elsewhere.
pub const FORBIDDEN_TERMS: &[&str] = &[
    "SQL",             // 0
    "NoSQL",           // 1
    "Relational",      // 2
    "Multi-Model",     // 3 — explicitly named for EnkiMDB in enki_engines.gd
    "PostgreSQL",      // 4
    "Postgres",        // 5
    "MySQL",           // 6
    "MariaDB",         // 7
    "MongoDB",         // 8
    "Cassandra",       // 9
    "Redis",           // 10
    "SQLite",          // 11
    "Oracle Database", // 12
    "SQL Server",      // 13
    "DynamoDB",        // 14
    "CouchDB",         // 15
    "Neo4j",           // 16
    "Elasticsearch",   // 17
                       // Deliberately NOT included: bare SQL verbs (SELECT, INSERT, DELETE,
                       // GROUP BY, ORDER BY, JOIN, ...). A first pass included them and, run
                       // against the real repo, mostly matched ordinary English/GDScript —
                       // "select a node", "_select_database()", "database_selected" — not a
                       // single real violation. `operations::Operation::parse()` already
                       // owns SQL-verb rejection precisely, as an exact-token match against
                       // real *parsed* HeptaScript source, never prose (`OperationError::
                       // SqlForbidden`) — that is the correct tool for verbs; substring-
                       // scanning prose for them is not. This gate's real, non-redundant job
                       // is the surface that scanner never sees: product names, paradigm
                       // words, and ports, in UI copy and adopted prototypes.
];

/// Real default ports of foreign database products — "any foreign
/// database's port/protocol" per `enki_engines.gd`'s own stated law.
/// `(port, product)` pairs.
pub const FORBIDDEN_PORTS: &[(&str, &str)] = &[
    ("5432", "PostgreSQL"),
    ("3306", "MySQL/MariaDB"),
    ("27017", "MongoDB"),
    ("9042", "Cassandra"),
    ("6379", "Redis"),
    ("1433", "SQL Server"),
    ("1521", "Oracle"),
    ("5984", "CouchDB"),
    ("9200", "Elasticsearch"),
    ("7687", "Neo4j Bolt"),
];

/// Lines matching one of these (case-insensitive substring) are
/// statements *of* the OOO law, not violations of it — e.g. this very
/// module's own doc comments, `operations::Operation::parse`'s error
/// strings, and every doc's "Anti-SQL Law" section. A per-line exemption
/// is coarse by design: the threat model here is a wholesale mockup
/// (a card, a row, a paragraph) carrying real foreign vocabulary, not one
/// word hidden inside an already-compliant sentence about forbidding it.
const EXEMPT_PHRASES: &[&str] = &[
    "anti-sql",
    "sqlforbidden",
    "forbidden vocabulary",
    "no use of sql",
    "sql is forbidden",
    "never sql",
    "not sql",
    "not in sql",
    "sql-free",
];

/// One forbidden-vocabulary hit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    /// The forbidden term or port that matched.
    pub matched: String,
    /// 1-indexed line number within the scanned text.
    pub line: usize,
    /// The full line text, for a human to see the hit in context.
    pub context: String,
}

/// Result of scanning one file (or one in-memory text buffer).
#[derive(Debug, Clone)]
pub struct VocabScanReport {
    pub path: Option<PathBuf>,
    pub violations: Vec<Violation>,
}

impl VocabScanReport {
    pub fn is_clean(&self) -> bool {
        self.violations.is_empty()
    }
}

fn line_is_exempt(line_lower: &str) -> bool {
    EXEMPT_PHRASES.iter().any(|p| line_lower.contains(p))
}

/// How many lines back from the current one still count as "the same
/// law-statement sentence" for exemption purposes — a real, observed
/// case: `enki_engines.gd`'s own header wraps "Forbidden vocabulary
/// anywhere in this project: SQL, Relational, ..., Multi-Model (for
/// EnkiMDB), or any foreign database's port/protocol." across two
/// comment lines, so the exempt phrase ("forbidden vocabulary") and the
/// listed example ("Multi-Model") that phrase is exempting land on
/// different lines. A single-line-only check would flag the law
/// statement itself as a violation of the law it states.
const EXEMPT_WINDOW: usize = 2;

/// Scan raw text for forbidden vocabulary and foreign ports. Returns
/// every hit, not just the first — same "exhaustive, not first-match"
/// choice `zip_scan::scan_all` makes, since a caller reviewing a mockup
/// before adoption wants the complete list to clean up in one pass.
pub fn scan_text(text: &str) -> Vec<Violation> {
    let mut violations = Vec::new();
    let lines: Vec<&str> = text.lines().collect();
    let lines_lower: Vec<String> = lines.iter().map(|l| l.to_ascii_lowercase()).collect();

    for (idx, raw_line) in lines.iter().copied().enumerate() {
        let window_start = idx.saturating_sub(EXEMPT_WINDOW);
        let in_exempt_window = lines_lower[window_start..=idx]
            .iter()
            .any(|l| line_is_exempt(l));
        if in_exempt_window {
            continue;
        }
        let line_lower = &lines_lower[idx];

        for term in FORBIDDEN_TERMS {
            if line_lower.contains(&term.to_ascii_lowercase()) {
                violations.push(Violation {
                    matched: term.to_string(),
                    line: idx + 1,
                    context: raw_line.to_string(),
                });
            }
        }

        for (port, product) in FORBIDDEN_PORTS {
            if contains_port_token(raw_line, port) {
                violations.push(Violation {
                    matched: format!("{port} ({product})"),
                    line: idx + 1,
                    context: raw_line.to_string(),
                });
            }
        }
    }

    violations
}

/// True if `line` contains `port` as a standalone digit run, not as a
/// substring of a longer number — so a legitimate HEPT port like `70012`
/// (hypothetically) never false-positives against `1521`'s trailing
/// digits, and `enki_engines.gd`'s own real ports (7001..7005, 7102,
/// 7202) never collide with any entry in `FORBIDDEN_PORTS` in the first
/// place (none of them share a full-length match).
fn contains_port_token(line: &str, port: &str) -> bool {
    let bytes = line.as_bytes();
    let plen = port.len();
    let mut start = 0;
    while let Some(rel) = line[start..].find(port) {
        let pos = start + rel;
        let before_ok = pos == 0 || !bytes[pos - 1].is_ascii_digit();
        let after_pos = pos + plen;
        let after_ok = after_pos >= bytes.len() || !bytes[after_pos].is_ascii_digit();
        if before_ok && after_ok {
            return true;
        }
        start = pos + 1;
    }
    false
}

/// Scan one file on disk.
pub fn scan_file(path: &Path) -> std::io::Result<VocabScanReport> {
    let text = fs::read_to_string(path)?;
    Ok(VocabScanReport {
        path: Some(path.to_path_buf()),
        violations: scan_text(&text),
    })
}

/// Extensions this gate is meant to scan — UI/design surfaces, never
/// compiled Rust source (`operations::Operation::parse` already owns
/// vocabulary enforcement over real HeptaScript/Rust content).
pub const GATED_EXTENSIONS: &[&str] = &["gd", "tscn", "html", "htm", "md"];

/// Recursively scan every file under `root` whose extension is in
/// `extensions`, skipping version-control and Godot cache directories.
/// No third-party directory-walk dependency — `zip_scan`'s own doc
/// comment states the same "pure pattern matching, no third-party
/// library" law this follows.
pub fn scan_dir(root: &Path, extensions: &[&str]) -> std::io::Result<Vec<VocabScanReport>> {
    let mut reports = Vec::new();
    walk(root, extensions, &mut reports)?;
    Ok(reports)
}

fn walk(dir: &Path, extensions: &[&str], out: &mut Vec<VocabScanReport>) -> std::io::Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();
        if name == ".git"
            || name == ".godot"
            || name == "_retired_2026-07-10"
            || name.starts_with('.')
        {
            continue;
        }
        if path.is_dir() {
            walk(&path, extensions, out)?;
        } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if extensions.iter().any(|e| e.eq_ignore_ascii_case(ext)) {
                out.push(scan_file(&path)?);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_text_passes() {
        let text = "EnkiDB — Sovereign Particle Graph (flagship)\nHEPT port 7001\n";
        assert!(scan_text(text).is_empty());
    }

    #[test]
    fn detects_sql_product_name() {
        let text = "EnkiODB — Object Document Store, port 27017 (MongoDB-style)";
        let v = scan_text(text);
        assert!(v.iter().any(|x| x.matched == "MongoDB"));
        assert!(v.iter().any(|x| x.matched.starts_with("27017")));
    }

    #[test]
    fn detects_relational_and_sql_paradigm_words() {
        let text = "EnkiDB — Relational SQL Engine";
        let v = scan_text(text);
        assert!(v.iter().any(|x| x.matched == "Relational"));
        assert!(v.iter().any(|x| x.matched == "SQL"));
    }

    #[test]
    fn detects_multi_model_for_enkimdb() {
        let text = "EnkiMDB — Multi-Model Database";
        let v = scan_text(text);
        assert!(v.iter().any(|x| x.matched == "Multi-Model"));
    }

    #[test]
    fn detects_foreign_port_as_standalone_token() {
        let text = "default port: 5432";
        let v = scan_text(text);
        assert_eq!(v.len(), 1);
        assert!(v[0].matched.starts_with("5432"));
    }

    #[test]
    fn does_not_false_positive_on_a_longer_number_containing_a_forbidden_port() {
        // "15432" contains "5432" as a substring but is not the port 5432.
        let text = "session id 15432 assigned";
        assert!(scan_text(text).is_empty());
    }

    #[test]
    fn real_hept_ports_never_collide_with_forbidden_ports() {
        let text = "EnkiDB HEPT :7001\nEnkiMDB HEPT :7202\nEnkiDDB HEPT :7102\n";
        assert!(scan_text(text).is_empty());
    }

    #[test]
    fn anti_sql_law_statements_are_exempt() {
        let text = "As BahyWay.Ecosystem Law of Anti-SQL, no use of SQL Methods is permitted.";
        assert!(scan_text(text).is_empty());
    }

    #[test]
    fn multi_line_forbidden_vocabulary_law_statement_is_exempt() {
        // Real case found scanning enki_engines.gd itself: the exempt
        // phrase ("Forbidden vocabulary") and the listed example
        // ("Multi-Model") it's exempting land on different comment lines.
        let text = "# Forbidden vocabulary anywhere in this project: SQL, Relational, 5432,\n# Multi-Model (for EnkiMDB), or any foreign database's port/protocol.\n";
        assert!(scan_text(text).is_empty(), "{:?}", scan_text(text));
    }

    #[test]
    fn exempt_window_does_not_suppress_a_real_violation_several_lines_later() {
        let text = "# Forbidden vocabulary anywhere in this project: SQL.\nsome unrelated line\nsome unrelated line\nsome unrelated line\ndb = \"MongoDB\"\n";
        let v = scan_text(text);
        assert_eq!(v.len(), 1, "{:?}", v);
        assert_eq!(v[0].line, 5);
    }

    #[test]
    fn sql_forbidden_error_type_name_is_exempt() {
        let text = "Err(OperationError::SqlForbidden(token.to_string()))";
        assert!(scan_text(text).is_empty());
    }

    #[test]
    fn violation_reports_correct_line_number() {
        let text = "line one\nline two has SQL in it\nline three";
        let v = scan_text(text);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].line, 2);
        assert_eq!(v[0].context, "line two has SQL in it");
    }

    #[test]
    fn scan_dir_finds_violations_across_multiple_gated_files() {
        let dir = std::env::temp_dir().join(format!(
            "musaru_vocab_gate_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("clean.gd"),
            "extends Control\n# EnkiDB port 7001\n",
        )
        .unwrap();
        fs::write(dir.join("dirty.html"), "<div>MongoDB port 27017</div>\n").unwrap();
        fs::write(
            dir.join("ignored.rs"),
            "// MongoDB — not a gated extension\n",
        )
        .unwrap();

        let reports = scan_dir(&dir, GATED_EXTENSIONS).unwrap();
        assert_eq!(
            reports.len(),
            2,
            "only .gd and .html are gated extensions here"
        );
        let dirty = reports
            .iter()
            .find(|r| r.path.as_ref().unwrap().ends_with("dirty.html"))
            .unwrap();
        assert!(!dirty.is_clean());
        let clean = reports
            .iter()
            .find(|r| r.path.as_ref().unwrap().ends_with("clean.gd"))
            .unwrap();
        assert!(clean.is_clean());

        fs::remove_dir_all(&dir).unwrap();
    }
}
