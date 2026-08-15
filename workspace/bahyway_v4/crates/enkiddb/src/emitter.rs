//! DocumentEmitter — mints a document's Identity-Kaki and turns its parsed
//! structure into real, KAKI-sealed `enkidb_particles::Particle` rows.

use std::time::{SystemTime, UNIX_EPOCH};

use akkvalue::AkkValue;
use enkidb_kaki::{IdentityKaki, KakiRole, KakiMinter};
use enkidb_particles::Particle;

use crate::concepts::ConceptEntry;
use crate::document::{BodyType, DocumentStructure, Section};
use crate::orbit::DocOrbit;

pub struct DocumentEmitter<'a> {
    minter: &'a KakiMinter,
}

impl<'a> DocumentEmitter<'a> {
    pub fn new(minter: &'a KakiMinter) -> Self {
        Self { minter }
    }

    /// Borrow the minter this emitter was built with -- callers that need
    /// to mint their own Event-Kakis alongside this emitter's Identity-Kakis
    /// (e.g. `WriteNode` journaling each document/section as its own event)
    /// use this instead of holding a second, separate `KakiMinter`.
    pub fn minter(&self) -> &KakiMinter {
        self.minter
    }

    /// Mint a new document Identity-Kaki and emit all its particles.
    ///
    /// Returns the document's Identity-Kaki plus every particle minted for
    /// it (meta/head/body/code/hist orbits). Storage/persistence is the
    /// caller's concern — this only produces the particle rows.
    pub fn emit_document(&self, structure: &DocumentStructure) -> (IdentityKaki, Vec<Particle>) {
        let doc_kaki = IdentityKaki::try_from_kaki(self.minter.identity(KakiRole::Kishib))
            .expect("KakiMinter::identity always mints kaki_type=Identity");
        (doc_kaki, self.document_particles(doc_kaki, structure))
    }

    /// Build the exact same particle set `emit_document` would, but against
    /// an ALREADY-KNOWN `doc_kaki` instead of minting a fresh one.
    ///
    /// FIXED: without this, a caller that reuses a stable identity for
    /// content it has already seen before (e.g. `shakkanakku::pb_catalog`'s
    /// content-hash dedup, which deliberately reuses the same
    /// `IdentityKaki` for identical bytes seen again) had no way to
    /// re-journal that document's own particles under its known identity —
    /// only `emit_document` existed, and it always mints fresh. Every
    /// `enkidb-readnode::Generation` is a complete, independent snapshot of
    /// whatever journal it was built from (proved by
    /// `enkidb-readnode::generation::tests::
    /// two_versions_of_the_same_sovereign_name_are_independently_addressable`,
    /// "v4.2's generation must not see v4.0's documents") — so a caller
    /// that mints once and then only ever re-links (never re-journals) an
    /// already-known document produces materializations that silently lose
    /// that document's own title/body/etc. the moment a NEW generation is
    /// promoted, even though its identity and fresh links still exist.
    /// This lets a caller re-journal the SAME content under the SAME
    /// identity every time it's seen, keeping every generation
    /// self-sufficient.
    pub fn emit_document_for(&self, doc_kaki: IdentityKaki, structure: &DocumentStructure) -> Vec<Particle> {
        self.document_particles(doc_kaki, structure)
    }

    fn document_particles(&self, doc_kaki: IdentityKaki, structure: &DocumentStructure) -> Vec<Particle> {
        let now = Self::now_secs();
        let mut particles = Vec::new();

        // META
        particles.push(Particle::base(
            doc_kaki,
            DocOrbit::Meta.attr("title"),
            AkkValue::Text(structure.title.clone()),
            now,
        ));

        // HEAD
        for header in &structure.headers {
            let field = match header.level {
                1 => "h1",
                2 => "h2",
                3 => "h3",
                4 => "h4",
                5 => "h5",
                _ => "h6",
            };
            particles.push(Particle::base(
                doc_kaki,
                DocOrbit::Head.attr(field),
                AkkValue::Text(header.text.clone()),
                now,
            ));
            particles.push(Particle::base(
                doc_kaki,
                DocOrbit::Head.attr("anchor"),
                AkkValue::Text(header.anchor.clone()),
                now,
            ));
            particles.push(Particle::base(
                doc_kaki,
                DocOrbit::Head.attr("order"),
                AkkValue::Int(header.order),
                now,
            ));
        }

        // BODY
        for element in &structure.body {
            let field = match element.element_type {
                BodyType::Paragraph => "paragraph",
            };
            particles.push(Particle::base(
                doc_kaki,
                DocOrbit::Body.attr(field),
                AkkValue::Text(element.content.clone()),
                now,
            ));
            particles.push(Particle::base(
                doc_kaki,
                DocOrbit::Body.attr("order"),
                AkkValue::Int(element.order),
                now,
            ));
        }

        // CODE
        for block in &structure.code_blocks {
            particles.push(Particle::base(
                doc_kaki,
                DocOrbit::Code.attr("block"),
                AkkValue::Text(block.code.clone()),
                now,
            ));
            particles.push(Particle::base(
                doc_kaki,
                DocOrbit::Code.attr("language"),
                AkkValue::Text(block.language.clone()),
                now,
            ));
            particles.push(Particle::base(
                doc_kaki,
                DocOrbit::Code.attr("order"),
                AkkValue::Int(block.order),
                now,
            ));
        }

        // HIST — birth event
        particles.push(Particle::base(
            doc_kaki,
            DocOrbit::Hist.attr("event"),
            AkkValue::Text("BIRTH".to_string()),
            now,
        ));

        particles
    }

    /// Emit a CrossTribe-Kaki link particle from one document to another.
    /// Returns two particles: the target reference and its description.
    pub fn emit_link(
        &self,
        source: IdentityKaki,
        target: IdentityKaki,
        description: &str,
    ) -> [Particle; 2] {
        let now = Self::now_secs();
        [
            Particle::base(
                source,
                DocOrbit::Link.attr("target"),
                AkkValue::KakiPk(*target.bytes()),
                now,
            ),
            Particle::base(
                source,
                DocOrbit::Link.attr("description"),
                AkkValue::Text(description.to_string()),
                now,
            ),
        ]
    }

    /// Records that `old` (an already-minted document Identity-Kaki) has
    /// been superseded by `new`, with `reason` explaining why -- an
    /// APPEND on `old`'s OWN existing identity, never a new mint and
    /// never a delete (ADR-006). This is the mechanism ADR-014 Decision 2
    /// promised: "why did v4.5 replace v4.4" becomes a queryable
    /// `hist.reason` on `old`, not lost history. Storage is the caller's
    /// concern, same contract as `emit_document`/`emit_link`.
    pub fn emit_supersession(&self, old: IdentityKaki, new: IdentityKaki, reason: &str) -> [Particle; 3] {
        let now = Self::now_secs();
        [
            Particle::base(old, DocOrbit::Hist.attr("event"), AkkValue::Text("SUPERSEDED".to_string()), now),
            Particle::base(old, DocOrbit::Hist.attr("superseded_by"), AkkValue::KakiPk(*new.bytes()), now),
            Particle::base(old, DocOrbit::Hist.attr("reason"), AkkValue::Text(reason.to_string()), now),
        ]
    }

    /// Emit a `meta.collection` particle categorizing `doc` by KIND
    /// (e.g. "architecture-reference", "glossary", "component",
    /// "playbook-record", "concept-law", "crate-manual"...) -- orthogonal
    /// to `DocOrbit`, which classifies WHICH PART of a document a particle
    /// is, not what kind of document it belongs to. Call alongside
    /// `emit_document`/`emit_document`'s particles.
    pub fn emit_collection(&self, doc: IdentityKaki, collection: &str) -> Particle {
        Particle::base(
            doc,
            DocOrbit::Meta.attr("collection"),
            AkkValue::Text(collection.to_string()),
            Self::now_secs(),
        )
    }

    /// Mint one child Identity-Kaki per section of `structure` (see
    /// `DocumentStructure::sections`), each carrying:
    ///   - `meta.title`     -- "<parent_title> § <header text>" (or just
    ///                         `parent_title` for a headerless preamble)
    ///   - `meta.section_order` -- the section's position in the document
    ///   - `body.summary`   -- the RAG "key": header text + a preview of
    ///                         the section's body, for cheap topical search
    ///   - `body.text`      -- the RAG "value": the section's full,
    ///                         untouched text (every paragraph and code
    ///                         block, verbatim) -- nothing is summarized
    ///                         away, only indexed by a summary alongside it
    ///   - `link.target`/`link.description` -- a link back to `parent`
    ///     (reusing `emit_link`'s CrossTribe-style mechanism), so every
    ///     chunk stays traceable to the document it came from
    ///   - `hist.event = "BIRTH"`
    /// Returns one `(section Identity-Kaki, particles)` pair per section, in
    /// document order. No particles are appended to the Journal here --
    /// storage is the caller's concern, same contract as `emit_document`.
    pub fn emit_sections(
        &self,
        parent: IdentityKaki,
        parent_title: &str,
        structure: &DocumentStructure,
    ) -> Vec<(IdentityKaki, Vec<Particle>)> {
        let now = Self::now_secs();
        structure
            .sections()
            .into_iter()
            .enumerate()
            .map(|(idx, section)| {
                let section_kaki = IdentityKaki::try_from_kaki(self.minter.identity(KakiRole::Zikru))
                    .expect("KakiMinter::identity always mints kaki_type=Identity");

                let title = match &section.header {
                    Some(h) => format!("{parent_title} \u{a7} {}", h.text),
                    None => parent_title.to_string(),
                };
                let full_text = Self::section_full_text(&section);
                let summary = Self::heuristic_summary(&section, &title);

                let mut particles = vec![
                    Particle::base(section_kaki, DocOrbit::Meta.attr("title"), AkkValue::Text(title), now),
                    Particle::base(section_kaki, DocOrbit::Meta.attr("section_order"), AkkValue::Int(idx as i64), now),
                    Particle::base(section_kaki, DocOrbit::Body.attr("summary"), AkkValue::Text(summary), now),
                    Particle::base(section_kaki, DocOrbit::Body.attr("text"), AkkValue::Text(full_text), now),
                    Particle::base(section_kaki, DocOrbit::Hist.attr("event"), AkkValue::Text("BIRTH".to_string()), now),
                ];
                particles.extend(self.emit_link(section_kaki, parent, "section-of"));

                (section_kaki, particles)
            })
            .collect()
    }

    /// Mint one child Identity-Kaki per detected concept mention (see
    /// `crate::concepts::ConceptRegistry::scan_mentions`), each carrying:
    ///   - `meta.title`         -- the concept's name
    ///   - `meta.concept_kind`  -- "gate" / "crate" / "sovereign-name"
    ///   - `link.target`/`link.description` -- a link back to `parent`
    ///     (reusing `emit_link`, same mechanism `emit_sections` uses for
    ///     its "section-of" edges), tagged `"mentioned-in"`
    ///   - `hist.event = "BIRTH"`
    /// Same shape and same "storage is the caller's concern" contract as
    /// `emit_sections` — deliberately mirrored, not a new pattern.
    pub fn emit_concept_mentions(
        &self,
        parent: IdentityKaki,
        mentions: &[ConceptEntry],
    ) -> Vec<(IdentityKaki, Vec<Particle>)> {
        let now = Self::now_secs();
        mentions
            .iter()
            .map(|mention| {
                let mention_kaki = IdentityKaki::try_from_kaki(self.minter.identity(KakiRole::Zikru))
                    .expect("KakiMinter::identity always mints kaki_type=Identity");
                let mut particles = vec![
                    Particle::base(mention_kaki, DocOrbit::Meta.attr("title"), AkkValue::Text(mention.name.clone()), now),
                    Particle::base(
                        mention_kaki,
                        DocOrbit::Meta.attr("concept_kind"),
                        AkkValue::Text(mention.kind.as_str().to_string()),
                        now,
                    ),
                    Particle::base(mention_kaki, DocOrbit::Hist.attr("event"), AkkValue::Text("BIRTH".to_string()), now),
                ];
                particles.extend(self.emit_link(mention_kaki, parent, "mentioned-in"));
                (mention_kaki, particles)
            })
            .collect()
    }

    /// The section's full, untouched text: header text (if any), every
    /// paragraph, then every fenced code block -- concatenated verbatim.
    /// This is the RAG "value" payload; it must never be truncated or
    /// paraphrased, only indexed by a separate summary.
    fn section_full_text(section: &Section) -> String {
        let mut out = String::new();
        if let Some(h) = &section.header {
            out.push_str(&h.text);
            out.push('\n');
        }
        for b in &section.body {
            out.push_str(&b.content);
            out.push('\n');
        }
        for c in &section.code_blocks {
            out.push_str("```");
            out.push_str(&c.language);
            out.push('\n');
            out.push_str(&c.code);
            out.push_str("```\n");
        }
        out
    }

    /// Heuristic (non-LLM) summary: the section's title plus the first ~200
    /// characters of its body text. This is the RAG "key" -- searched
    /// against instead of the noisy full text; once matched, the caller
    /// fetches `body.text` for the real payload. Deliberately scoped
    /// honestly: an LLM-generated summary would be richer, but no LLM
    /// document-summarization path exists in the ecosystem yet
    /// (`enkidullm-codex` is not built) -- this heuristic ships now rather
    /// than blocking indexing on that.
    fn heuristic_summary(section: &Section, title: &str) -> String {
        let preview: String = section
            .body
            .iter()
            .map(|b| b.content.as_str())
            .collect::<Vec<_>>()
            .join(" ")
            .chars()
            .take(200)
            .collect();
        if preview.is_empty() {
            title.to_string()
        } else {
            format!("{title}: {preview}")
        }
    }

    fn now_secs() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::DocumentParser;
    use bahyway_core::TribeId;

    fn minter() -> KakiMinter {
        KakiMinter::new(TribeId::from_u16(0x00E0))
    }

    #[test]
    fn emits_identity_kaki_with_kishib_role() {
        let m = minter();
        let doc = DocumentParser::parse_markdown("# Hello\n\nWorld.\n");
        let emitter = DocumentEmitter::new(&m);
        let (kaki, _particles) = emitter.emit_document(&doc);
        assert_eq!(kaki.role(), KakiRole::Kishib);
    }

    #[test]
    fn emits_expected_particle_count() {
        let m = minter();
        let doc = DocumentParser::parse_markdown(
            "# Title\n\n## Section\n\nA paragraph.\n\n```rust\nfn f() {}\n```\n",
        );
        let emitter = DocumentEmitter::new(&m);
        let (_kaki, particles) = emitter.emit_document(&doc);

        // 1 meta.title + 3 head.* + 2 body.* + 3 code.* + 1 hist.event
        assert_eq!(particles.len(), 10);
    }

    #[test]
    fn all_particles_share_document_entity() {
        let m = minter();
        let doc = DocumentParser::parse_markdown("# T\n\nBody text.\n");
        let emitter = DocumentEmitter::new(&m);
        let (kaki, particles) = emitter.emit_document(&doc);
        for p in &particles {
            assert_eq!(p.entity, kaki);
        }
    }

    #[test]
    fn particle_attributes_are_orbit_namespaced() {
        let m = minter();
        let doc = DocumentParser::parse_markdown("# T\n\nBody text.\n");
        let emitter = DocumentEmitter::new(&m);
        let (_kaki, particles) = emitter.emit_document(&doc);
        assert!(particles.iter().any(|p| p.attribute == "meta.title"));
        assert!(particles.iter().any(|p| p.attribute == "body.paragraph"));
        assert!(particles.iter().any(|p| p.attribute == "hist.event"));
    }

    #[test]
    fn link_particle_carries_target_kaki_pk() {
        let m = minter();
        let doc_a = DocumentParser::parse_markdown("# A\n");
        let doc_b = DocumentParser::parse_markdown("# B\n");
        let emitter = DocumentEmitter::new(&m);
        let (kaki_a, _) = emitter.emit_document(&doc_a);
        let (kaki_b, _) = emitter.emit_document(&doc_b);

        let [target_particle, desc_particle] = emitter.emit_link(kaki_a, kaki_b, "see also");
        assert_eq!(target_particle.attribute, "link.target");
        match target_particle.value {
            AkkValue::KakiPk(bytes) => assert_eq!(bytes, *kaki_b.bytes()),
            _ => panic!("expected KakiPk value"),
        }
        assert_eq!(desc_particle.attribute, "link.description");
    }

    #[test]
    fn emit_supersession_targets_the_old_kaki_not_a_new_one() {
        let m = minter();
        let doc_v44 = DocumentParser::parse_markdown("# v4.4\n");
        let doc_v45 = DocumentParser::parse_markdown("# v4.5\n");
        let emitter = DocumentEmitter::new(&m);
        let (kaki_v44, _) = emitter.emit_document(&doc_v44);
        let (kaki_v45, _) = emitter.emit_document(&doc_v45);

        let particles = emitter.emit_supersession(kaki_v44, kaki_v45, "v4.5 closes the KAKI-minting gap v4.4 left open");
        for p in &particles {
            assert_eq!(p.entity, kaki_v44, "supersession particles must target the OLD document's own identity");
        }
    }

    #[test]
    fn emit_supersession_carries_event_link_and_reason() {
        let m = minter();
        let doc_a = DocumentParser::parse_markdown("# A\n");
        let doc_b = DocumentParser::parse_markdown("# B\n");
        let emitter = DocumentEmitter::new(&m);
        let (kaki_a, _) = emitter.emit_document(&doc_a);
        let (kaki_b, _) = emitter.emit_document(&doc_b);

        let particles = emitter.emit_supersession(kaki_a, kaki_b, "because the Architect said so");

        let event = particles.iter().find(|p| p.attribute == "hist.event").unwrap();
        match &event.value {
            AkkValue::Text(t) => assert_eq!(t, "SUPERSEDED"),
            _ => panic!("expected Text"),
        }

        let link = particles.iter().find(|p| p.attribute == "hist.superseded_by").unwrap();
        match link.value {
            AkkValue::KakiPk(bytes) => assert_eq!(bytes, *kaki_b.bytes(), "must link to the NEW document's identity"),
            _ => panic!("expected KakiPk"),
        }

        let reason = particles.iter().find(|p| p.attribute == "hist.reason").unwrap();
        match &reason.value {
            AkkValue::Text(t) => assert_eq!(t, "because the Architect said so"),
            _ => panic!("expected Text"),
        }
    }

    #[test]
    fn emit_collection_tags_the_document_kind() {
        let m = minter();
        let doc = DocumentParser::parse_markdown("# T\n");
        let emitter = DocumentEmitter::new(&m);
        let (kaki, _) = emitter.emit_document(&doc);
        let p = emitter.emit_collection(kaki, "architecture-reference");
        assert_eq!(p.attribute, "meta.collection");
        assert_eq!(p.entity, kaki);
        match p.value {
            AkkValue::Text(t) => assert_eq!(t, "architecture-reference"),
            _ => panic!("expected Text value"),
        }
    }

    #[test]
    fn emit_sections_mints_one_child_per_section_and_links_to_parent() {
        let m = minter();
        let doc = DocumentParser::parse_markdown(
            "# Guide\n\n## Alpha\n\nAlpha body text.\n\n## Beta\n\nBeta body text.\n\n```rust\nfn f() {}\n```\n",
        );
        let emitter = DocumentEmitter::new(&m);
        let (parent, _) = emitter.emit_document(&doc);
        let sections = emitter.emit_sections(parent, &doc.title, &doc);

        assert_eq!(sections.len(), 2);
        let (alpha_kaki, alpha_particles) = &sections[0];
        assert_ne!(alpha_kaki.bytes(), parent.bytes());

        let title = alpha_particles.iter().find(|p| p.attribute == "meta.title").unwrap();
        match &title.value {
            AkkValue::Text(t) => assert_eq!(t, "Guide § Alpha"),
            _ => panic!("expected Text"),
        }

        let link_target = alpha_particles.iter().find(|p| p.attribute == "link.target").unwrap();
        match link_target.value {
            AkkValue::KakiPk(bytes) => assert_eq!(bytes, *parent.bytes()),
            _ => panic!("expected KakiPk"),
        }

        let (_beta_kaki, beta_particles) = &sections[1];
        let text = beta_particles.iter().find(|p| p.attribute == "body.text").unwrap();
        match &text.value {
            AkkValue::Text(t) => {
                assert!(t.contains("Beta body text."));
                assert!(t.contains("fn f() {}"), "code block must survive verbatim in the section value");
            }
            _ => panic!("expected Text"),
        }
    }

    #[test]
    fn section_summary_never_exceeds_a_bounded_preview_length() {
        let m = minter();
        let long_body = "word ".repeat(200);
        let md = format!("# T\n\n## Section\n\n{long_body}\n");
        let doc = DocumentParser::parse_markdown(&md);
        let emitter = DocumentEmitter::new(&m);
        let (parent, _) = emitter.emit_document(&doc);
        let sections = emitter.emit_sections(parent, &doc.title, &doc);

        let summary = sections[0].1.iter().find(|p| p.attribute == "body.summary").unwrap();
        match &summary.value {
            AkkValue::Text(t) => assert!(t.len() < long_body.len(), "summary must be shorter than the full body"),
            _ => panic!("expected Text"),
        }
    }
}
