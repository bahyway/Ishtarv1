//! links — pure-text discovery of cross-document references, so a batch
//! of ingested documents can be linked to the OTHER real documents they
//! reference by path, without a `regex` crate.
//!
//! Evaluated from an external design review (2026-07-19): genuinely
//! useful gap `infer_collection` never covered (that only classifies a
//! document by its OWN path, never what it points at). Implemented in
//! pure std, matching `concepts::ConceptRegistry::scan_mentions`'s own
//! no-regex, no-NLP discipline — this scans for the same two patterns a
//! human author naturally writes when referencing another file:
//!
//!   1. Markdown links: `[text](./scripts/deploy.sh)`
//!   2. Inline code spans: `` `scripts/etl.py` ``
//!
//! Only paths ending in a recognized extension count -- a bare word in
//! backticks (`` `foo` ``) is not a path reference.

/// Extensions this scanner treats as "this looks like a real file
/// reference," not prose. Deliberately narrow and explicit rather than
/// "anything with a dot" -- a version number like `v1.2` or a decimal
/// like `2.5` must never be mistaken for a path.
const KNOWN_EXTENSIONS: &[&str] = &["md", "sh", "py", "rs", "yml", "yaml"];

/// Scan `text` for path-shaped references to other documents/scripts.
/// Returns each distinct reference once, in first-seen order. The
/// strings returned are exactly as written in the source text (e.g.
/// `./scripts/deploy.sh`) -- resolving them against real ingested
/// documents is the caller's job (see
/// `crate::writenode::WriteNode::ingest_directory_categorized_checked`,
/// which resolves by file stem against the batch it just ingested).
pub fn discover_referenced_paths(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    scan_markdown_links(text, &mut found);
    scan_backtick_paths(text, &mut found);

    let mut seen = std::collections::HashSet::new();
    found.retain(|p| seen.insert(p.clone()));
    found
}

/// `[label](path)` -- find every `](`, then the next `)`, and keep the
/// slice between them if it looks like a known-extension path. Ignores
/// nested parentheses inside the path (real file paths never have them).
fn scan_markdown_links(text: &str, out: &mut Vec<String>) {
    let mut search_from = 0;
    while let Some(rel) = text[search_from..].find("](") {
        let start = search_from + rel + 2;
        let Some(rel_end) = text[start..].find(')') else {
            break;
        };
        let end = start + rel_end;
        let candidate = text[start..end].trim();
        if has_known_extension(candidate) && !candidate.contains(char::is_whitespace) {
            out.push(candidate.to_string());
        }
        search_from = end + 1;
    }
}

/// `` `path` `` -- every inline code span, kept only if its content is a
/// single bare, known-extension path (no spaces -- a code span containing
/// a full shell command like `` `python3 etl.py --dry-run` `` is prose
/// about a command, not a path reference, and is deliberately excluded).
fn scan_backtick_paths(text: &str, out: &mut Vec<String>) {
    for (i, part) in text.split('`').enumerate() {
        // `str::split('`')` alternates outside/inside text; odd indices
        // are the content of a `...` span.
        if i % 2 == 1 {
            let candidate = part.trim();
            if has_known_extension(candidate) && !candidate.contains(char::is_whitespace) {
                out.push(candidate.to_string());
            }
        }
    }
}

fn has_known_extension(candidate: &str) -> bool {
    match candidate.rsplit_once('.') {
        Some((stem, ext)) => !stem.is_empty() && KNOWN_EXTENSIONS.contains(&ext),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_a_markdown_link_reference() {
        let text = "See the [deploy script](./scripts/deploy.sh) for details.";
        assert_eq!(
            discover_referenced_paths(text),
            vec!["./scripts/deploy.sh".to_string()]
        );
    }

    #[test]
    fn finds_a_backtick_bare_path_reference() {
        let text = "Run `scripts/etl.py` to process the export.";
        assert_eq!(
            discover_referenced_paths(text),
            vec!["scripts/etl.py".to_string()]
        );
    }

    #[test]
    fn ignores_a_backtick_span_that_is_a_command_not_a_bare_path() {
        let text = "Run `python3 scripts/etl.py --dry-run` first.";
        assert!(discover_referenced_paths(text).is_empty());
    }

    #[test]
    fn ignores_a_backtick_span_with_no_known_extension() {
        let text = "Set `ENKIDDB_TEAM_ALLOWLIST` before starting.";
        assert!(discover_referenced_paths(text).is_empty());
    }

    #[test]
    fn ignores_a_version_number_in_backticks() {
        let text = "Requires `walkdir 2.5` or newer.";
        assert!(discover_referenced_paths(text).is_empty());
    }

    #[test]
    fn ignores_a_markdown_link_to_a_web_url() {
        let text = "See [the RFC](https://example.com/rfc.html) for background.";
        // .html is not a recognized extension -- deliberately narrow.
        assert!(discover_referenced_paths(text).is_empty());
    }

    #[test]
    fn finds_multiple_distinct_references_in_document_order() {
        let text = "Deploy via [deploy.sh](./scripts/deploy.sh), then run `scripts/etl.py`.";
        assert_eq!(
            discover_referenced_paths(text),
            vec![
                "./scripts/deploy.sh".to_string(),
                "scripts/etl.py".to_string()
            ]
        );
    }

    #[test]
    fn each_reference_reported_once_even_if_mentioned_repeatedly() {
        let text = "Run `deploy.sh`. Later, run `deploy.sh` again.";
        assert_eq!(
            discover_referenced_paths(text),
            vec!["deploy.sh".to_string()]
        );
    }

    #[test]
    fn no_references_returns_empty() {
        assert!(discover_referenced_paths("Just plain prose, nothing to link.").is_empty());
    }
}
