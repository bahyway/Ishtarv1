//! DocumentStructure — the parsed intermediate form of a document,
//! before it is minted into KAKI-sealed particles.

/// A parsed document, ready for emission into EnkiDDB particles.
#[derive(Clone, Debug, Default)]
pub struct DocumentStructure {
    pub title: String,
    pub headers: Vec<Header>,
    pub body: Vec<BodyElement>,
    pub code_blocks: Vec<CodeBlock>,
}

#[derive(Clone, Debug)]
pub struct Header {
    pub level: u8,
    pub text: String,
    pub anchor: String,
    pub order: i64,
}

#[derive(Clone, Debug)]
pub struct BodyElement {
    pub element_type: BodyType,
    pub content: String,
    pub order: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BodyType {
    Paragraph,
}

#[derive(Clone, Debug)]
pub struct CodeBlock {
    pub code: String,
    pub language: String,
    pub order: i64,
}

/// One logical section of a document: a header plus every body element and
/// code block that falls after it (and before the next header), grouped by
/// the shared `order` field every element carries. `header: None` marks the
/// preamble -- content that appears before the document's first header, if
/// any. Every body element and code block in the source `DocumentStructure`
/// appears in exactly one `Section`, in its original relative order --
/// this is a grouping, not a re-parse, so nothing is lost or reordered.
#[derive(Clone, Debug)]
pub struct Section {
    pub header: Option<Header>,
    pub body: Vec<BodyElement>,
    pub code_blocks: Vec<CodeBlock>,
}

impl DocumentStructure {
    /// Group headers/body/code_blocks into sections at header boundaries.
    pub fn sections(&self) -> Vec<Section> {
        let mut headers: Vec<&Header> = self.headers.iter().collect();
        headers.sort_by_key(|h| h.order);
        let mut body: Vec<&BodyElement> = self.body.iter().collect();
        body.sort_by_key(|b| b.order);
        let mut code_blocks: Vec<&CodeBlock> = self.code_blocks.iter().collect();
        code_blocks.sort_by_key(|c| c.order);

        let mut sections: Vec<Section> = Vec::new();
        let mut current = Section {
            header: None,
            body: Vec::new(),
            code_blocks: Vec::new(),
        };
        let (mut hi, mut bi, mut ci) = (0, 0, 0);

        loop {
            let next_h = headers.get(hi).map(|h| h.order);
            let next_b = body.get(bi).map(|b| b.order);
            let next_c = code_blocks.get(ci).map(|c| c.order);
            let min_order = [next_h, next_b, next_c].into_iter().flatten().min();
            let Some(min_order) = min_order else { break };

            if next_h == Some(min_order) {
                sections.push(std::mem::replace(
                    &mut current,
                    Section {
                        header: Some(headers[hi].clone()),
                        body: Vec::new(),
                        code_blocks: Vec::new(),
                    },
                ));
                hi += 1;
            } else if next_b == Some(min_order) {
                current.body.push(body[bi].clone());
                bi += 1;
            } else {
                current.code_blocks.push(code_blocks[ci].clone());
                ci += 1;
            }
        }
        sections.push(current);

        // Drop a leading empty preamble (no content appeared before the
        // first header) -- keep it when there really is preamble content.
        if let Some(first) = sections.first() {
            if first.header.is_none() && first.body.is_empty() && first.code_blocks.is_empty() {
                sections.remove(0);
            }
        }
        sections
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header(text: &str, order: i64) -> Header {
        Header {
            level: 2,
            text: text.to_string(),
            anchor: text.to_lowercase(),
            order,
        }
    }

    fn para(text: &str, order: i64) -> BodyElement {
        BodyElement {
            element_type: BodyType::Paragraph,
            content: text.to_string(),
            order,
        }
    }

    #[test]
    fn preamble_before_first_header_becomes_its_own_section() {
        let doc = DocumentStructure {
            title: "T".to_string(),
            headers: vec![header("First", 1)],
            body: vec![para("intro", 0), para("body one", 2)],
            code_blocks: vec![],
        };
        let sections = doc.sections();
        assert_eq!(sections.len(), 2);
        assert!(sections[0].header.is_none());
        assert_eq!(sections[0].body.len(), 1);
        assert_eq!(sections[0].body[0].content, "intro");
        assert_eq!(sections[1].header.as_ref().unwrap().text, "First");
        assert_eq!(sections[1].body.len(), 1);
    }

    #[test]
    fn no_preamble_yields_no_leading_empty_section() {
        let doc = DocumentStructure {
            title: "T".to_string(),
            headers: vec![header("Only", 0)],
            body: vec![para("body", 1)],
            code_blocks: vec![],
        };
        let sections = doc.sections();
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].header.as_ref().unwrap().text, "Only");
    }

    #[test]
    fn every_element_lands_in_exactly_one_section() {
        let doc = DocumentStructure {
            title: "T".to_string(),
            headers: vec![header("A", 0), header("B", 3)],
            body: vec![para("a1", 1), para("a2", 2), para("b1", 4)],
            code_blocks: vec![CodeBlock {
                code: "fn f() {}\n".to_string(),
                language: "rust".to_string(),
                order: 5,
            }],
        };
        let sections = doc.sections();
        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0].header.as_ref().unwrap().text, "A");
        assert_eq!(sections[0].body.len(), 2);
        assert_eq!(sections[1].header.as_ref().unwrap().text, "B");
        assert_eq!(sections[1].body.len(), 1);
        assert_eq!(sections[1].code_blocks.len(), 1);

        let total_body: usize = sections.iter().map(|s| s.body.len()).sum();
        let total_code: usize = sections.iter().map(|s| s.code_blocks.len()).sum();
        assert_eq!(total_body, doc.body.len());
        assert_eq!(total_code, doc.code_blocks.len());
    }

    #[test]
    fn empty_document_has_no_sections() {
        let doc = DocumentStructure::default();
        assert!(doc.sections().is_empty());
    }
}
