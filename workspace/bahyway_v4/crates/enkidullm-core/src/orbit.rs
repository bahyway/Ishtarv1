//! BookOrbit — four EAV shells vibrating around the book KAKI nucleus.
//!
//! §2.4: No assessment data lives in the nucleus (tribe_id and kaki_role are
//! structural, decided at birth). Domain name, depth score, reputation — all Orbit.
//! §8.3: CrossTribe shell is COMPUTED on PROBE, never stored here.
//!
//! Shell update frequencies:
//!   Core           → never (immutable after first ingestion)
//!   Classification → slow (re-analysis triggered manually)
//!   Reputation     → medium (citation cache refresh schedule)
//!   Content        → fast (updated per ingest pass, per chunk)

/// IDU provenance state — matches the global IDU Rule §8.3 semantics.
/// Gold = verified canonical; Orange = unverified / suspect; Gray = deprecated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IduState { Gold, Orange, Gray }

impl IduState {
    pub fn label(self) -> &'static str {
        match self { Self::Gold => "GOLD", Self::Orange => "ORANGE", Self::Gray => "GRAY" }
    }

    /// Combine two IDU states: take the weaker (Gold+Orange → Orange, any+Gray → Gray).
    pub fn combine(a: IduState, b: IduState) -> IduState {
        match (a, b) {
            (IduState::Gold,   IduState::Gold)   => IduState::Gold,
            (IduState::Gray,   _             )   => IduState::Gray,
            (_,                IduState::Gray )   => IduState::Gray,
            _                                    => IduState::Orange,
        }
    }

    /// Degrade: used by audit when knowledge overlap without citation is found.
    pub fn degrade(self) -> IduState {
        match self {
            IduState::Gold   => IduState::Orange,
            IduState::Orange => IduState::Gray,
            IduState::Gray   => IduState::Gray,
        }
    }
}

/// Book archetype — structural type of the book (decided at classification time).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BookArchetype {
    Textbook, Reference, Monograph, Proceedings, PractitionerGuide, Standard,
}

impl BookArchetype {
    pub fn label(self) -> &'static str {
        match self {
            Self::Textbook         => "TEXTBOOK",
            Self::Reference        => "REFERENCE",
            Self::Monograph        => "MONOGRAPH",
            Self::Proceedings      => "PROCEEDINGS",
            Self::PractitionerGuide => "PRACTITIONER_GUIDE",
            Self::Standard         => "STANDARD",
        }
    }
}

/// A knowledge triple: subject → predicate → object.
/// Stored inside ContentShell.concept_graph as vertical EAV lines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeTriple {
    pub subject:   String,   // e.g. "chapter_1", "CAP_Theorem"
    pub predicate: String,   // e.g. "introduces", "depends_on", "contrasts_with", "cites"
    pub object:    String,   // e.g. "ACID_vs_BASE", "Kleppmann_2017"
}

impl KnowledgeTriple {
    pub fn new(subject: &str, predicate: &str, object: &str) -> Self {
        Self { subject: subject.into(), predicate: predicate.into(), object: object.into() }
    }
}

// ── Four Orbit Shells ─────────────────────────────────────────────────────────

/// Core shell: structural provenance, immutable after first ingestion.
#[derive(Debug, Clone)]
pub struct CoreShell {
    pub title:       String,
    pub author:      String,
    pub publisher:   Option<String>,
    pub isbn:        Option<String>,
    pub source_path: String,
    /// FNV-1a hash of source file bytes — structural provenance seal.
    pub source_seal: u32,
}

/// Classification shell: LLM-derived assessments, updated on re-analysis.
#[derive(Debug, Clone)]
pub struct ClassificationShell {
    pub domain:          Option<String>,
    /// [0,1]: 0 = introductory, 1 = deep research.
    pub technical_depth: Option<f32>,
    pub archetype:       Option<BookArchetype>,
    /// Classification confidence [0,1].
    pub confidence:      f32,
    /// Ratio of imperative recommendation sentences [0,1].
    pub practical_score: f32,
}

/// Reputation shell: external-signal assessment, refreshed on citation cache update.
#[derive(Debug, Clone)]
pub struct ReputationShell {
    pub citation_count:    u32,
    pub adoption_score:    f32,
    pub reputation_score:  f32,
    pub idu_state:         IduState,
    pub last_updated_secs: u64,
}

/// Content shell: chunk-level text analysis, updated per ingest pass.
#[derive(Debug, Clone)]
pub struct ContentShell {
    pub chapter_count:      u16,
    pub code_example_count: u32,
    pub word_count:         u64,
    /// Knowledge triples extracted from book content.
    pub concept_graph:      Vec<KnowledgeTriple>,
    /// FNV-1a hash of cited ISBNs — used for citation graph analysis.
    pub cited_isbn_hashes:  Vec<u32>,
    pub chapter_summaries:  Vec<String>,
    /// Unified embedding [256 f32] — native, no external model framework.
    pub embedding:          Vec<f32>,
}

/// The complete book orbit (four shells + version counter for temporal diffing).
#[derive(Debug, Clone)]
pub struct BookOrbit {
    pub core:           CoreShell,
    pub classification: ClassificationShell,
    pub reputation:     ReputationShell,
    pub content:        ContentShell,
    /// Incremented on every update — used by the DIFF query operator.
    pub orbit_version:  u64,
}

impl BookOrbit {
    /// Create a minimal orbit from core metadata (all assessment fields at neutral defaults).
    pub fn from_core(core: CoreShell) -> Self {
        Self {
            core,
            classification: ClassificationShell {
                domain: None, technical_depth: None, archetype: None,
                confidence: 0.0, practical_score: 0.0,
            },
            reputation: ReputationShell {
                citation_count: 0, adoption_score: 0.0,
                reputation_score: 0.0, idu_state: IduState::Orange,
                last_updated_secs: 0,
            },
            content: ContentShell {
                chapter_count: 0, code_example_count: 0, word_count: 0,
                concept_graph: Vec::new(), cited_isbn_hashes: Vec::new(),
                chapter_summaries: Vec::new(), embedding: Vec::new(),
            },
            orbit_version: 0,
        }
    }

    /// Bump version — called whenever any shell is updated.
    pub fn bump_version(&mut self) { self.orbit_version += 1; }

    /// Apply a reputation update from the cache refresh pipeline.
    pub fn update_reputation(&mut self, citation_count: u32, adoption_score: f32,
                              reputation_score: f32, now_secs: u64) {
        self.reputation.citation_count    = citation_count;
        self.reputation.adoption_score    = adoption_score;
        self.reputation.reputation_score  = reputation_score;
        self.reputation.last_updated_secs = now_secs;
        // Promote Orange → Gold when reputation score crosses the canonical threshold
        if reputation_score >= 0.85 && self.reputation.idu_state == IduState::Orange {
            self.reputation.idu_state = IduState::Gold;
        }
        self.bump_version();
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn core() -> CoreShell {
        CoreShell { title: "DDIA".into(), author: "Kleppmann".into(),
                    publisher: Some("O'Reilly".into()),
                    isbn: Some("978-1491903698".into()),
                    source_path: "/books/ddia.pdf".into(), source_seal: 0xCAFEBABE }
    }

    #[test]
    fn orbit_from_core_defaults() {
        let o = BookOrbit::from_core(core());
        assert_eq!(o.reputation.idu_state, IduState::Orange);
        assert_eq!(o.orbit_version, 0);
        assert!(o.classification.domain.is_none());
    }

    #[test]
    fn idu_state_combine() {
        assert_eq!(IduState::combine(IduState::Gold,   IduState::Gold),   IduState::Gold);
        assert_eq!(IduState::combine(IduState::Gold,   IduState::Orange), IduState::Orange);
        assert_eq!(IduState::combine(IduState::Orange, IduState::Gray),   IduState::Gray);
        assert_eq!(IduState::combine(IduState::Gray,   IduState::Gold),   IduState::Gray);
    }

    #[test]
    fn idu_state_degrade() {
        assert_eq!(IduState::Gold.degrade(),   IduState::Orange);
        assert_eq!(IduState::Orange.degrade(), IduState::Gray);
        assert_eq!(IduState::Gray.degrade(),   IduState::Gray);
    }

    #[test]
    fn reputation_update_promotes_to_gold() {
        let mut o = BookOrbit::from_core(core());
        o.update_reputation(9_500, 0.90, 0.88, 1_000_000);
        assert_eq!(o.reputation.idu_state, IduState::Gold);
        assert_eq!(o.reputation.citation_count, 9_500);
        assert_eq!(o.orbit_version, 1);
    }

    #[test]
    fn reputation_update_stays_orange_below_threshold() {
        let mut o = BookOrbit::from_core(core());
        o.update_reputation(100, 0.30, 0.60, 1_000_000);
        assert_eq!(o.reputation.idu_state, IduState::Orange);
    }

    #[test]
    fn knowledge_triple_construction() {
        let t = KnowledgeTriple::new("chapter_1", "introduces", "CAP_Theorem");
        assert_eq!(t.subject, "chapter_1");
        assert_eq!(t.predicate, "introduces");
        assert_eq!(t.object, "CAP_Theorem");
    }

    #[test]
    fn book_archetype_labels() {
        assert_eq!(BookArchetype::Textbook.label(),          "TEXTBOOK");
        assert_eq!(BookArchetype::PractitionerGuide.label(), "PRACTITIONER_GUIDE");
    }
}
