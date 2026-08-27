//! Minimal Markdown -> DocumentStructure parser.
//!
//! Recognises: `# ` title, `## `/`### ` headers, fenced code blocks, and
//! blank-line-separated paragraphs. No external dependencies.

use crate::document::{BodyElement, BodyType, CodeBlock, DocumentStructure, Header};

pub struct DocumentParser;

impl DocumentParser {
    pub fn parse_markdown(text: &str) -> DocumentStructure {
        let mut structure = DocumentStructure {
            title: "Untitled".to_string(),
            ..Default::default()
        };

        let lines: Vec<&str> = text.lines().collect();
        let mut i = 0;
        let mut order_counter: i64 = 0;

        while i < lines.len() {
            let line = lines[i];

            if line.starts_with("# ") && structure.title == "Untitled" {
                structure.title = line[2..].trim().to_string();
                i += 1;
                continue;
            }

            if let Some(stripped) = line.strip_prefix("## ") {
                structure.headers.push(Header {
                    level: 2,
                    text: stripped.trim().to_string(),
                    anchor: Self::make_anchor(stripped),
                    order: order_counter,
                });
                order_counter += 1;
                i += 1;
                continue;
            }
            if let Some(stripped) = line.strip_prefix("### ") {
                structure.headers.push(Header {
                    level: 3,
                    text: stripped.trim().to_string(),
                    anchor: Self::make_anchor(stripped),
                    order: order_counter,
                });
                order_counter += 1;
                i += 1;
                continue;
            }

            if let Some(stripped) = line.strip_prefix("```") {
                let language = stripped.trim().to_string();
                let mut code = String::new();
                i += 1;
                while i < lines.len() && !lines[i].starts_with("```") {
                    code.push_str(lines[i]);
                    code.push('\n');
                    i += 1;
                }
                structure.code_blocks.push(CodeBlock {
                    code,
                    language,
                    order: order_counter,
                });
                order_counter += 1;
                i += 1; // skip closing fence
                continue;
            }

            if !line.trim().is_empty() {
                let mut paragraph = line.to_string();
                i += 1;
                while i < lines.len()
                    && !lines[i].trim().is_empty()
                    && !lines[i].starts_with('#')
                    && !lines[i].starts_with("```")
                {
                    paragraph.push(' ');
                    paragraph.push_str(lines[i].trim());
                    i += 1;
                }
                structure.body.push(BodyElement {
                    element_type: BodyType::Paragraph,
                    content: paragraph,
                    order: order_counter,
                });
                order_counter += 1;
                continue;
            }

            i += 1;
        }

        structure
    }

    fn make_anchor(text: &str) -> String {
        text.to_lowercase()
            .replace(' ', "-")
            .replace(|c: char| !c.is_alphanumeric() && c != '-', "")
    }

    /// Two real shapes seen across this repo's own playbooks: a header
    /// alone on its line ending in ':' ("USAGE:"), and a header
    /// immediately followed by prose on the SAME line ("WHY THIS
    /// EXISTS: the Architect's own instruction..." -- PB-269/270/271 all
    /// do this). Both are "an ALL-CAPS, 2+-word phrase up to the first
    /// colon" -- what follows that colon (if anything) is body text, not
    /// part of the header, and must not be discarded.
    fn header_prefix(s: &str) -> Option<(&str, &str)> {
        let colon = s.find(':')?;
        let (prefix, rest) = (s[..colon].trim(), s[colon + 1..].trim());
        let word_count = prefix.split_whitespace().count();
        let has_alpha = prefix.chars().any(|c| c.is_alphabetic());
        let all_upper = prefix
            .chars()
            .filter(|c| c.is_alphabetic())
            .all(|c| c.is_uppercase());
        (word_count >= 2 && has_alpha && all_upper).then_some((prefix, rest))
    }

    /// Extract a numbered playbook's leading `#`-comment header block
    /// (everything before the first non-comment, non-blank line -- the
    /// `- name: ...` that starts the actual YAML play) as a
    /// `DocumentStructure`, so it can be minted/versioned through the
    /// same document pipeline markdown docs use, under
    /// `enkiddb::PB_DOCS_TRIBE_ID` rather than `DOCS_TRIBE_ID`.
    ///
    /// Not a markdown parse -- playbook headers use their own convention
    /// (`# ===...` rules, a `# PB-NNN -- Title` first line, then ALL-CAPS
    /// `WORD(S):`-style pseudo-headers like `WHY THIS EXISTS:` or
    /// `USAGE:`). Title is the first non-separator, non-blank comment
    /// line; a comment line is a level-2 pseudo-header when, after
    /// stripping a trailing `:`, every alphabetic character in it is
    /// uppercase and it contains more than one word (so a real sentence
    /// ending in a capitalized acronym doesn't get misread as a
    /// section). Everything else is grouped into paragraphs at blank-
    /// comment-line boundaries, same as `parse_markdown`'s paragraphs.
    ///
    /// Returns `None` only when the file has no leading comment block at
    /// all (defensive -- every real numbered playbook in this repo has
    /// one, but this must not panic on one that doesn't).
    pub fn parse_playbook_header(yaml_text: &str) -> Option<DocumentStructure> {
        let mut header_lines: Vec<&str> = Vec::new();
        for line in yaml_text.lines() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                header_lines.push(
                    trimmed
                        .trim_start_matches('#')
                        .strip_prefix(' ')
                        .unwrap_or(trimmed.trim_start_matches('#')),
                );
            } else if trimmed.is_empty() && !header_lines.is_empty() {
                header_lines.push("");
            } else {
                break;
            }
        }
        if header_lines.is_empty() {
            return None;
        }

        let is_separator = |s: &str| !s.is_empty() && s.chars().all(|c| c == '=' || c == '-');

        let mut structure = DocumentStructure {
            title: "Untitled".to_string(),
            ..Default::default()
        };
        let mut order_counter: i64 = 0;
        let mut paragraph_buf: Vec<&str> = Vec::new();

        let flush_paragraph =
            |structure: &mut DocumentStructure, order_counter: &mut i64, buf: &mut Vec<&str>| {
                if !buf.is_empty() {
                    structure.body.push(BodyElement {
                        element_type: BodyType::Paragraph,
                        content: buf.join(" "),
                        order: *order_counter,
                    });
                    *order_counter += 1;
                    buf.clear();
                }
            };

        for raw in &header_lines {
            let line = raw.trim_end();
            if is_separator(line) {
                flush_paragraph(&mut structure, &mut order_counter, &mut paragraph_buf);
                continue;
            }
            if line.trim().is_empty() {
                flush_paragraph(&mut structure, &mut order_counter, &mut paragraph_buf);
                continue;
            }
            if structure.title == "Untitled" {
                structure.title = line.trim().to_string();
                continue;
            }
            if let Some((prefix, rest)) = Self::header_prefix(line.trim()) {
                flush_paragraph(&mut structure, &mut order_counter, &mut paragraph_buf);
                structure.headers.push(Header {
                    level: 2,
                    text: prefix.to_string(),
                    anchor: Self::make_anchor(prefix),
                    order: order_counter,
                });
                order_counter += 1;
                if !rest.is_empty() {
                    paragraph_buf.push(rest);
                }
                continue;
            }
            paragraph_buf.push(line.trim());
        }
        flush_paragraph(&mut structure, &mut order_counter, &mut paragraph_buf);

        Some(structure)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::BodyType;

    #[test]
    fn parses_title_headers_and_paragraph() {
        let md = "# My Title\n\n## Section One\n\nThis is a paragraph.\n";
        let doc = DocumentParser::parse_markdown(md);

        assert_eq!(doc.title, "My Title");
        assert_eq!(doc.headers.len(), 1);
        assert_eq!(doc.headers[0].level, 2);
        assert_eq!(doc.headers[0].text, "Section One");
        assert_eq!(doc.headers[0].anchor, "section-one");
        assert_eq!(doc.body.len(), 1);
        assert_eq!(doc.body[0].element_type, BodyType::Paragraph);
        assert_eq!(doc.body[0].content, "This is a paragraph.");
    }

    #[test]
    fn parses_fenced_code_block() {
        let md = "# T\n\n```rust\nfn main() {}\n```\n";
        let doc = DocumentParser::parse_markdown(md);

        assert_eq!(doc.code_blocks.len(), 1);
        assert_eq!(doc.code_blocks[0].language, "rust");
        assert!(doc.code_blocks[0].code.contains("fn main()"));
    }

    #[test]
    fn missing_title_defaults_to_untitled() {
        let md = "## Just a section\n";
        let doc = DocumentParser::parse_markdown(md);
        assert_eq!(doc.title, "Untitled");
    }

    #[test]
    fn preserves_order_across_element_types() {
        let md = "# T\n\nFirst paragraph.\n\n## Header\n\nSecond paragraph.\n";
        let doc = DocumentParser::parse_markdown(md);
        assert_eq!(doc.body[0].order, 0);
        assert_eq!(doc.headers[0].order, 1);
        assert_eq!(doc.body[1].order, 2);
    }

    #[test]
    fn playbook_header_extracts_title_and_pseudo_headers() {
        let yaml = "\
# =============================================================================
# PB-999 -- Example playbook title. BahyWay.Ecosystem v4.0 -- 2026-07-30
#
# WHY THIS EXISTS: a real reason, spanning
# more than one comment line for the test.
#
# THIS PLAYBOOK DELIBERATELY DOES NOT:
#   - do the thing.
# =============================================================================
- name: \"PB-999 -- Example\"
  hosts: localhost
";
        let doc = DocumentParser::parse_playbook_header(yaml).expect("real leading comment block");
        assert_eq!(
            doc.title,
            "PB-999 -- Example playbook title. BahyWay.Ecosystem v4.0 -- 2026-07-30"
        );
        assert!(
            doc.headers.iter().any(|h| h.text == "WHY THIS EXISTS"),
            "headers: {:?}",
            doc.headers
        );
        assert!(
            doc.headers
                .iter()
                .any(|h| h.text == "THIS PLAYBOOK DELIBERATELY DOES NOT"),
            "headers: {:?}",
            doc.headers
        );
        assert!(doc.body.iter().any(|b| b.content.contains("a real reason")));
    }

    #[test]
    fn playbook_header_returns_none_for_a_file_with_no_leading_comment() {
        let yaml = "- name: \"no header at all\"\n  hosts: localhost\n";
        assert!(DocumentParser::parse_playbook_header(yaml).is_none());
    }

    #[test]
    fn playbook_header_parses_this_repos_own_real_pb_269_file() {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("playbooks/playbook_269_retire_eriduous_vdi_confirm_baremetal_control_node.yml");
        let text =
            std::fs::read_to_string(&path).expect("PB-269 should exist in this real repo checkout");
        let doc = DocumentParser::parse_playbook_header(&text)
            .expect("PB-269 has a real leading comment header");
        assert!(doc.title.contains("PB-269"), "title: {:?}", doc.title);
        assert!(
            doc.headers
                .iter()
                .any(|h| h.text.contains("WHY THIS EXISTS")),
            "headers: {:?}",
            doc.headers
        );
        assert!(!doc.body.is_empty());
    }
}
