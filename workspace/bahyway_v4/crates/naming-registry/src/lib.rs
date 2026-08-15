//! naming-registry — a structured, tested catalog of the Akkadian/
//! Sumerian names used across this ecosystem, feeding the DubSar
//! "Glossary" panel (see docs/SPEC-MAS-001.md-style honest-scope
//! precedent this crate follows).
//!
//! ## Honest scope (2026-07-25)
//!
//! This crate's `seed()` registry is **manually compiled**, not the
//! output of an automated markdown-parsing pipeline. This ecosystem's
//! naming decisions live as prose across ~15+ `SPEC-*`/`GL-*`/`PH-*`/
//! `BC-*` tablets and `docs/components/*.md` files with no single
//! consistent structured format; a reliable automated extractor over
//! free text is real follow-on engineering, not something to fake by
//! calling a brittle regex pass "automated." Every entry below is
//! sourced from a document this build actually read (recorded in
//! `source_doc` where one exists) or, where no dedicated naming-law
//! tablet exists yet, marked `NamingStatus::ExistingUnverified` rather
//! than assigned a fabricated citation.
//!
//! This registry is also **not exhaustive** — it seeds ~20 entries
//! this build could verify directly; the workspace has ~250 crates,
//! many carrying real Akkadian names never reviewed here. Extending
//! it is real, ongoing curation work, not a one-time task this crate
//! claims to finish.
//!
//! Every `crate_path` in this registry is verified to actually exist
//! on disk by this crate's own test suite (`tests::every_crate_path_exists`)
//! — a naming entry can never silently point at a crate that isn't
//! real.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MythoCategory {
    /// A named deity (Gibil, Marduk, Nabû, ...).
    Deity,
    /// A city or place name (Eridu, ...).
    City,
    /// A historical king or the era named after one (Gudea, ...).
    KingOrEra,
    /// A generic Akkadian/Sumerian word or phrase, not a proper name
    /// of a deity/city/king (Massartu "the watch", Birqu "lightning",
    /// šar pūḫi "substitute king").
    GenericTerm,
}

impl MythoCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            MythoCategory::Deity => "Deity",
            MythoCategory::City => "City",
            MythoCategory::KingOrEra => "KingOrEra",
            MythoCategory::GenericTerm => "GenericTerm",
        }
    }
}

/// What kind of thing in the system this name was assigned to, per
/// NL-001 ("gods name engines, cities name structures, kings name
/// eras, artifacts name formats [A1], practices name patterns [A2]")
/// and its observed extensions in this build (alert classes, agents,
/// protocols, index stacks, tools).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemRole {
    Engine,
    Calculus,
    AlertClass,
    Pattern,
    Agent,
    Suite,
    Tool,
    Protocol,
    IndexStack,
    /// A term explaining part of the language/architecture itself
    /// (e.g. why HeptaScript has seven dimensions) rather than naming
    /// a specific running component.
    Language,
}

impl SystemRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            SystemRole::Engine => "Engine",
            SystemRole::Calculus => "Calculus",
            SystemRole::AlertClass => "AlertClass",
            SystemRole::Pattern => "Pattern",
            SystemRole::Agent => "Agent",
            SystemRole::Suite => "Suite",
            SystemRole::Tool => "Tool",
            SystemRole::Protocol => "Protocol",
            SystemRole::IndexStack => "IndexStack",
            SystemRole::Language => "Language",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NamingStatus {
    /// Ratified by a real governing-law tablet this build read and
    /// cites in `source_doc`.
    SealedByLaw,
    /// A real, shipped name in this workspace, but with no dedicated
    /// naming-law tablet found justifying it -- the crate/module
    /// exists; the naming rationale is inferred from the name and
    /// surrounding doc comments only.
    ExistingUnverified,
    /// Reserved for future use (e.g. Ishum for a gas-pipeline
    /// calculus), not yet backing any shipped code.
    Reserved,
}

impl NamingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            NamingStatus::SealedByLaw => "SealedByLaw",
            NamingStatus::ExistingUnverified => "ExistingUnverified",
            NamingStatus::Reserved => "Reserved",
        }
    }
}

#[derive(Debug, Clone)]
pub struct NamingEntry {
    pub name: &'static str,
    pub mytho_category: MythoCategory,
    pub system_role: SystemRole,
    /// Free-form domain/crate tags (e.g. ["electricity", "grid"]).
    pub domain_tags: &'static [&'static str],
    pub status: NamingStatus,
    /// The governing-law tablet this naming decision is documented
    /// in, if one exists. `None` for `ExistingUnverified`/`Reserved`
    /// entries with no dedicated tablet.
    pub source_doc: Option<&'static str>,
    /// Path (relative to this workspace's root, e.g.
    /// `workspace/bahyway_v4`) to the real source file backing this
    /// name, if one exists yet.
    pub crate_path: Option<&'static str>,
    pub blurb: &'static str,
    /// Path (relative to this workspace's root) to a real image file for
    /// this entry, if one has been verified and committed. `None` is the
    /// honest default for every entry today -- no image assets exist in
    /// this repo yet. Never set this from an unverified source image
    /// (see the Nisaba entry's own note): a wrong attribution shown next
    /// to a sealed name is worse than no image at all.
    pub image_path: Option<&'static str>,
    /// Required whenever `image_path` is `Some` -- who/what the image is
    /// actually attributed to (e.g. a museum, a known public-domain
    /// source), so the Glossary panel never shows unsourced imagery.
    pub image_credit: Option<&'static str>,
}

pub struct Registry(pub Vec<NamingEntry>);

impl Registry {
    pub fn by_mytho_category(&self, cat: MythoCategory) -> Vec<&NamingEntry> {
        self.0.iter().filter(|e| e.mytho_category == cat).collect()
    }

    pub fn by_system_role(&self, role: SystemRole) -> Vec<&NamingEntry> {
        self.0.iter().filter(|e| e.system_role == role).collect()
    }

    pub fn by_domain(&self, tag: &str) -> Vec<&NamingEntry> {
        self.0.iter().filter(|e| e.domain_tags.contains(&tag)).collect()
    }

    pub fn find(&self, name: &str) -> Option<&NamingEntry> {
        self.0.iter().find(|e| e.name.eq_ignore_ascii_case(name))
    }

    /// Case-insensitive substring search over name, blurb, and domain
    /// tags -- the free-text search box behind the Glossary tab.
    pub fn search(&self, query: &str) -> Vec<&NamingEntry> {
        let q = query.to_lowercase();
        if q.is_empty() {
            return self.0.iter().collect();
        }
        self.0
            .iter()
            .filter(|e| {
                e.name.to_lowercase().contains(&q)
                    || e.blurb.to_lowercase().contains(&q)
                    || e.domain_tags.iter().any(|t| t.to_lowercase().contains(&q))
            })
            .collect()
    }
}

/// Reads a short, real excerpt of the source file backing `entry`, for
/// the Glossary tab's "snippet" view. `workspace_root` is the
/// directory containing `crates/`/`bin/` (e.g. `workspace/bahyway_v4`).
///
/// Returns `Err` with an honest explanation, not an empty string, when
/// there is nothing real to show: `Reserved`/conceptual entries have
/// no `crate_path` at all, and a path that fails to read (moved,
/// permissions, ...) is reported rather than silently swallowed.
pub fn read_snippet(workspace_root: &std::path::Path, entry: &NamingEntry, max_lines: usize) -> Result<String, String> {
    let Some(path) = entry.crate_path else {
        let why = match entry.status {
            NamingStatus::Reserved => "reserved for future use -- no code exists yet.",
            _ => "a conceptual/etymological reference only; it names no running component.",
        };
        return Err(format!("{} has no source file: {why}", entry.name));
    };
    let full = workspace_root.join(path);
    let content = std::fs::read_to_string(&full).map_err(|e| format!("could not read {full:?}: {e}"))?;
    Ok(content.lines().take(max_lines).collect::<Vec<_>>().join("\n"))
}

pub fn seed() -> Registry {
    use MythoCategory::*;
    use NamingStatus::*;
    use SystemRole::*;

    Registry(vec![
        NamingEntry {
            name: "Gibil",
            mytho_category: Deity,
            system_role: Calculus,
            domain_tags: &["electricity", "egd-engine"],
            status: SealedByLaw,
            source_doc: Some("docs/SPEC-FDD-001.md"),
            crate_path: Some("crates/egd-engine/src/gibil.rs"),
            blurb: "Smith-fire, the refining flame that shapes metal. Names EGDEngine's KCL-residual calculus over complex voltage phasors. Replaced an earlier 'Girra' proposal, which collided with girra-engine.",
            image_path: None,
            image_credit: None,
        },
        NamingEntry {
            name: "Birqu",
            mytho_category: GenericTerm,
            system_role: AlertClass,
            domain_tags: &["electricity", "egd-engine"],
            status: SealedByLaw,
            source_doc: Some("docs/SPEC-FDD-001.md"),
            crate_path: Some("crates/egd-engine/src/gibil.rs"),
            blurb: "Akkadian for 'lightning.' The alert class Gibil emits when a KCL residual exceeds tolerance -- electricity's counterpart to water's Milu.",
            image_path: None,
            image_credit: None,
        },
        NamingEntry {
            name: "Milu",
            mytho_category: GenericTerm,
            system_role: AlertClass,
            domain_tags: &["water", "enbilulu"],
            status: SealedByLaw,
            source_doc: Some("docs/marduk/GL-MRD-002-neberu-slicer.md"),
            crate_path: Some("crates/bahyway-algebra/src/enbilulu.rs"),
            blurb: "Akkadian for 'flood.' The water-domain alert emitted from Enbilulu's Têrtu diagnosis -- the founding precedent the Analysis-to-Solution Law (DETECT->PROVE->PREDICT->PRESCRIBE) generalized from.",
            image_path: None,
            image_credit: None,
        },
        NamingEntry {
            name: "Ishum",
            mytho_category: Deity,
            system_role: Calculus,
            domain_tags: &["gas-pipeline (not yet built)"],
            status: Reserved,
            source_doc: Some("docs/SPEC-MAS-001.md"),
            crate_path: None,
            blurb: "Erra's restraining companion in the Erra Epic -- holds Erra's destructive fire in check until judgment is warranted. Reserved for a future gas-pipeline calculus; 'Nergal' was considered first but collides with the real, shipped steward-lens::AlertSeverity::Nergal.",
            image_path: None,
            image_credit: None,
        },
        NamingEntry {
            name: "Massartu",
            mytho_category: GenericTerm,
            system_role: Pattern,
            domain_tags: &["cross-domain", "massartu-core"],
            status: SealedByLaw,
            source_doc: Some("docs/PH-002_Puhu_Law.md"),
            crate_path: Some("crates/massartu-core/src/lib.rs"),
            blurb: "Akkadian for 'the watch.' Names the domain-neutral DETECT->PROVE->PREDICT->PRESCRIBE pattern any Tribe's nucleus can be run through.",
            image_path: None,
            image_credit: None,
        },
        NamingEntry {
            name: "Nabu",
            mytho_category: Deity,
            system_role: Calculus,
            domain_tags: &["marduk-engine", "analytics"],
            status: SealedByLaw,
            source_doc: Some("docs/marduk/GL-MRD-002-neberu-slicer.md"),
            crate_path: Some("crates/marduk-engine/src/lib.rs"),
            blurb: "God of wisdom and writing, Marduk's son. Names MardukEngine's covariant-derivative calculus (Position/Motion/Curvature/Topology/Horizon verbs) over the g = diag(w1..w7) Hepta-Space manifold.",
            image_path: None,
            image_credit: None,
        },
        NamingEntry {
            name: "Marduk",
            mytho_category: Deity,
            system_role: Suite,
            domain_tags: &["marduk-engine", "cross-domain analytics"],
            status: SealedByLaw,
            source_doc: Some("docs/marduk/GL-MRD-002-neberu-slicer.md"),
            crate_path: Some("crates/marduk-engine/src/lib.rs"),
            blurb: "The cross-domain analytical flagship engine and its 'fifty names' namespace -- a place other domains' calculi (Girra, Gibil, ...) can eventually be retroactively absorbed into without being rebuilt.",
            image_path: None,
            image_credit: None,
        },
        NamingEntry {
            name: "Girra",
            mytho_category: Deity,
            system_role: Engine,
            domain_tags: &["general monitoring dashboard"],
            status: ExistingUnverified,
            source_doc: Some("docs/SPEC-FDD-001.md (naming-collision note only)"),
            crate_path: Some("crates/girra-engine/src/lib.rs"),
            blurb: "Fire god. Names the ecosystem's general sovereign monitoring dashboard. Predates this session's naming-law tablets -- no dedicated founding document was found, only the collision note recorded when Gibil was chosen instead for electricity.",
            image_path: None,
            image_credit: None,
        },
        NamingEntry {
            name: "Nusku",
            mytho_category: Deity,
            system_role: Engine,
            domain_tags: &["body-scan security classification"],
            status: ExistingUnverified,
            source_doc: Some("docs/SPEC-FDD-001.md (naming-collision note only)"),
            crate_path: Some("crates/nusku-engine/src/lib.rs"),
            blurb: "Light/fire god, Girra's companion in some traditions. Names the body-scan security classification family (nusku-engine/nusku-score/nusku-fuzzy). No dedicated founding tablet found.",
            image_path: None,
            image_credit: None,
        },
        NamingEntry {
            name: "Enbilulu",
            mytho_category: Deity,
            system_role: Calculus,
            domain_tags: &["water", "wpd-engine (adjacent, not derived from)"],
            status: SealedByLaw,
            source_doc: Some("docs/BC-ENV-001_Enbilulu_Calculus_2026-07-07.md"),
            crate_path: Some("crates/bahyway-algebra/src/enbilulu.rs"),
            blurb: "Canal-inspector god. Names the sealed 5-factor weighted water-defect score (weights 0.20/0.20/0.15/0.30/0.15) feeding TIAMAT bands. A weighted score, not a network-conservation model -- fdd-core/Gibil is explicitly new machinery, not an Enbilulu migration.",
            image_path: None,
            image_credit: None,
        },
        NamingEntry {
            name: "WPDEngine",
            mytho_category: GenericTerm,
            system_role: Engine,
            domain_tags: &["water", "spectral remote sensing"],
            status: SealedByLaw,
            source_doc: Some("docs/SPEC-FDD-001.md"),
            crate_path: Some("crates/wpd-engine/src/lib.rs"),
            blurb: "Descriptive-acronym engine name (not god-named, per NL-001's engine-naming precedent). A spectral/remote-sensing classifier: BaghdadSector heptagram geography, 12-band VNIR/SWIR/TIR signature matching.",
            image_path: None,
            image_credit: None,
        },
        NamingEntry {
            name: "Neberu",
            mytho_category: Deity,
            system_role: Language,
            domain_tags: &["marduk-engine (concept only, implementation deferred)"],
            status: SealedByLaw,
            source_doc: Some("docs/marduk/GL-MRD-002-neberu-slicer.md"),
            crate_path: None,
            blurb: "Marduk's star, 'the crossing point.' Names the Neberu Slicer -- a proposed Poincare-section technique for reading dense orbits. Sealed as a concept; implementation is explicitly deferred, no code exists yet.",
            image_path: None,
            image_credit: None,
        },
        NamingEntry {
            name: "Anu",
            mytho_category: Deity,
            system_role: IndexStack,
            domain_tags: &["indexing", "tribe hot-index"],
            status: SealedByLaw,
            source_doc: Some("docs/components/ANU_INDEX_STACK.md"),
            crate_path: Some("crates/anu-tribe-hotindex/src/lib.rs"),
            blurb: "Sky god, head of the pantheon. Names the ANU index stack (renamed from an earlier 'ENLIL' proposal) and the Anu Tribe HotIndex crate.",
            image_path: None,
            image_credit: None,
        },
        NamingEntry {
            name: "Eridu",
            mytho_category: City,
            system_role: Suite,
            domain_tags: &["runtime", "os"],
            status: SealedByLaw,
            source_doc: Some("docs/components/ERIDUOS.md"),
            crate_path: Some("crates/eridu-runtime/src/lib.rs"),
            blurb: "Tradition's oldest Sumerian city, seat of Enki. Names EriduOS, the sovereign runtime/scheduler/supervisor layer this ecosystem runs on.",
            image_path: None,
            image_credit: None,
        },
        NamingEntry {
            name: "Enki",
            mytho_category: Deity,
            system_role: Suite,
            domain_tags: &["whole ecosystem"],
            status: SealedByLaw,
            source_doc: Some("docs/BAHYWAY_ECOSYSTEM_V4_GLOSSARY.md"),
            crate_path: None,
            blurb: "God of wisdom, fresh water, and crafts, patron of Eridu. The ecosystem's own root name (EnkiDB) -- not one crate, but the name the whole sovereign database lineage is built under.",
            image_path: None,
            image_credit: None,
        },
        NamingEntry {
            name: "DUB.SAR",
            mytho_category: GenericTerm,
            system_role: Tool,
            domain_tags: &["ide", "documentation convention"],
            status: SealedByLaw,
            source_doc: Some("docs/components/HEPTASCRIPT_GLOSSARY.md"),
            crate_path: Some("crates/dubsar-ide/src/lib.rs"),
            blurb: "Akkadian/Sumerian for 'scribe.' Names the DubSar Theater IDE and the closing convention used across this ecosystem's sealed tablets ('Inscribed for DUB.SAR') -- reproducible science as a written record, not a floating opinion.",
            image_path: None,
            image_credit: None,
        },
        NamingEntry {
            name: "KAKI",
            mytho_category: GenericTerm,
            system_role: Language,
            domain_tags: &["identity", "eav", "tribe"],
            status: SealedByLaw,
            source_doc: Some("docs/components/KAKI_V4.md"),
            crate_path: Some("crates/enkidb-kaki/src/lib.rs"),
            blurb: "The ecosystem's sovereign identity/addressing primitive underlying every Tribe and EAV particle. A coined system term, not a direct deity/city/king borrowing.",
            image_path: None,
            image_credit: None,
        },
        NamingEntry {
            name: "Sebettu",
            mytho_category: Deity,
            system_role: Language,
            domain_tags: &["heptascript"],
            status: SealedByLaw,
            source_doc: Some("docs/components/HEPTASCRIPT_GLOSSARY.md"),
            crate_path: None,
            blurb: "The divine Seven -- one group with two attested aspects (the Erra Epic's warrior-gods and the Utukku Lemnutu 'evil Seven' incantation series), not two symmetric teams. Part of HeptaScript's 'why seven dimensions' historical background; names no running component.",
            image_path: None,
            image_credit: None,
        },
        NamingEntry {
            name: "Gudea",
            mytho_category: KingOrEra,
            system_role: Language,
            domain_tags: &["founding era"],
            status: SealedByLaw,
            source_doc: Some("docs/components/HEPTASCRIPT_GLOSSARY.md"),
            crate_path: None,
            blurb: "Ensi (ruler) of Lagash, whose Cylinder B records a seven-day temple dedication. Names this ecosystem's own 'Gudea 1.0' founding era -- a real, checkable historical anchor rather than an invented one.",
            image_path: None,
            image_credit: None,
        },
        NamingEntry {
            name: "sar puhi (Puhu)",
            mytho_category: GenericTerm,
            system_role: Language,
            domain_tags: &["naming law"],
            status: SealedByLaw,
            source_doc: Some("docs/PH-002_Puhu_Law.md"),
            crate_path: None,
            blurb: "The Mesopotamian substitute-king ritual: a stand-in occupied the throne during an omen of danger, absorbing the threat while the office continued unbroken. Names PH-002's Puhu Law: 'the Tribe is the throne, not the king' -- a domain nucleus can be substituted without breaking the pattern it plugs into.",
            image_path: None,
            image_credit: None,
        },
        NamingEntry {
            name: "Ninsun",
            mytho_category: Deity,
            system_role: Agent,
            domain_tags: &["ai-council", "steward-bridge"],
            status: ExistingUnverified,
            source_doc: None,
            crate_path: Some("crates/ninsun-agent/src/lib.rs"),
            blurb: "Gilgamesh's mother, a goddess known for wise counsel. Names an agent crate in this ecosystem's AI Council / Steward Bridge layer. No dedicated naming-law tablet found; role inferred from the crate name only.",
            image_path: None,
            image_credit: None,
        },
        NamingEntry {
            name: "Lamassu",
            mytho_category: GenericTerm,
            system_role: Engine,
            domain_tags: &["topological data analysis", "sentinel"],
            status: ExistingUnverified,
            source_doc: None,
            crate_path: Some("crates/lamassu-engine/src/lib.rs"),
            blurb: "The protective winged guardian figure placed at gateways. Names the TDA sentinel engine (persistent homology via bahyway-algebra -- corrected 2026-07-31: 'GeoEngine' was never a real crate; the original playbook_93_geo_engine.yml was superseded and absorbed into enkidb-indexes::hepta_shell, a different spatial-indexing concern). No dedicated naming-law tablet found.",
            image_path: None,
            image_credit: None,
        },
        NamingEntry {
            name: "Ninurta",
            mytho_category: Deity,
            system_role: Engine,
            domain_tags: &["critical slowing down", "bifurcation detection", "topological data analysis"],
            status: SealedByLaw,
            source_doc: Some("docs/14_decisions_adr/adr_018_topological_engine_division.md"),
            crate_path: Some("crates/ninurta-engine/src/lib.rs"),
            blurb: "Hero-god of decisive action, recoverer of the Tablet of Destinies, patron (as Ningirsu) of Girsu. Names the critical-transition engine that reads a particle-collection's complex IN MOTION -- restoring rate lambda, corroborating indicators, Fourier-surrogate significance -- and renders a tau-scored verdict via the Purussum Calculus. Verified unspent in this registry before sealing (2026-07-31). Ningirsu form reserved-and-unused to avoid collision with the Girsu workbench.",
            image_path: None,
            image_credit: None,
        },
        NamingEntry {
            name: "Tupsarrutu",
            mytho_category: GenericTerm,
            system_role: Engine,
            domain_tags: &["mathematics", "geometric algebra", "persistent homology"],
            status: SealedByLaw,
            source_doc: Some("docs/01_mathematics/top_algebra.md"),
            crate_path: Some("crates/bahyway-algebra/src/lib.rs"),
            blurb: "Akkadian for 'the scribal art' (root of DUB.SAR, the Architect's own scribe-title). Already sealed as TOP Algebra's own name ('TOP Algebra -- Tupsarrutu') before this crate existed under that content; adopted 2026-08-01 as bahyway-algebra's honorary title -- the crate name itself stays plain (Cargo package names are ASCII identifiers; a rename this deep, touching 135+ tests and every arsenal-indexed dependent, was rejected as unnecessary risk for a cosmetic gain). Diacritic form Tupsarrutu (Ṭupšarrūtu) is the display form; this ASCII entry is the registry-searchable one.",
            image_path: None,
            image_credit: None,
        },
        // ── The Seven Evil Spirits -- reserved, never client-facing ──────
        // Sealed 2026-08-01: the Architect's own instruction after
        // rejecting "Lilu" as a client-facing engine name (a lilu IS a
        // demon; naming the tool that holds a client's own documents
        // after one was the wrong register entirely). Their real use:
        // internal-only harm/corruption/threat category labels this
        // ecosystem deals with BY NAME but never brands anything
        // client-visible with -- the same principle Sebettu's own entry
        // above already cites (the Utukku Lemnutu "evil Seven"
        // incantation series is where these seven are drawn from).
        // Reserved, not yet backing shipped code -- a future internal
        // threat-taxonomy (e.g. extending vgca_validation::
        // CorruptionClass, or musaru-security's malware classification)
        // is the natural home when built.
        NamingEntry {
            name: "Utukku",
            mytho_category: GenericTerm,
            system_role: AlertClass,
            domain_tags: &["internal threat taxonomy (not yet built)"],
            status: Reserved,
            source_doc: None,
            crate_path: None,
            blurb: "\"The wicked Utukku who slays man alive on the plain.\" Reserved for a lethal/destructive corruption class (total, unrecoverable data loss) -- internal-only, never client-facing.",
            image_path: None,
            image_credit: None,
        },
        NamingEntry {
            name: "Alu",
            mytho_category: GenericTerm,
            system_role: AlertClass,
            domain_tags: &["internal threat taxonomy (not yet built)"],
            status: Reserved,
            source_doc: None,
            crate_path: None,
            blurb: "\"The wicked Alu who covers (man) like a garment.\" Reserved for a smothering/complete-overwrite corruption class -- internal-only, never client-facing.",
            image_path: None,
            image_credit: None,
        },
        NamingEntry {
            name: "Edimmu",
            mytho_category: GenericTerm,
            system_role: AlertClass,
            domain_tags: &["internal threat taxonomy (not yet built)"],
            status: Reserved,
            source_doc: None,
            crate_path: None,
            blurb: "\"The wicked Edimmu... who bind the body.\" The restless ghost of the unburied dead. Reserved for a recurring/reanimating corruption class -- the same conceptual territory SHAMASH Gate 4's own Zombie/reincarnation judgment already occupies, so this stays a reserved LABEL, not a competing mechanism. Internal-only, never client-facing.",
            image_path: None,
            image_credit: None,
        },
        NamingEntry {
            name: "Gallu",
            mytho_category: GenericTerm,
            system_role: AlertClass,
            domain_tags: &["internal threat taxonomy (not yet built)"],
            status: Reserved,
            source_doc: None,
            crate_path: None,
            blurb: "\"The wicked... Gallu, who bind the body.\" The underworld's bailiff-demon, sent to drag the condemned away. Reserved for a forced-quarantine/seizure corruption class -- internal-only, never client-facing.",
            image_path: None,
            image_credit: None,
        },
        NamingEntry {
            name: "Lamme",
            mytho_category: Deity,
            system_role: AlertClass,
            domain_tags: &["internal threat taxonomy (not yet built)"],
            status: Reserved,
            source_doc: None,
            crate_path: None,
            blurb: "\"The Lamme (Lamashtu)... who cause disease in the body.\" The child-endangering demoness. Reserved for a degenerative/spreading corruption class -- internal-only, never client-facing, and never reused for anything client-visible given the real historical harm this figure represents.",
            image_path: None,
            image_credit: None,
        },
        NamingEntry {
            name: "Lammea",
            mytho_category: Deity,
            system_role: AlertClass,
            domain_tags: &["internal threat taxonomy (not yet built)"],
            status: Reserved,
            source_doc: None,
            crate_path: None,
            blurb: "\"The Lammea (Labasu)... who cause disease in the body.\" Lamashtu's companion demon. Reserved alongside Lamme for a degenerative/spreading corruption class -- internal-only, never client-facing.",
            image_path: None,
            image_credit: None,
        },
        NamingEntry {
            name: "Lilu",
            mytho_category: GenericTerm,
            system_role: AlertClass,
            domain_tags: &["internal threat taxonomy (not yet built)"],
            status: Reserved,
            source_doc: None,
            crate_path: None,
            blurb: "\"The Lilu who wanders in the plain.\" Considered and REJECTED 2026-08-01 as the client-document-ingestion engine's name (a lilu is a demon born of unfulfilled death -- the wrong register entirely for a tool holding a client's own data). Reserved instead for an orphaned/unlinked-particle corruption class (a particle with no home tribe, drifting) -- internal-only, never client-facing.",
            image_path: None,
            image_credit: None,
        },
        NamingEntry {
            name: "SipparStore",
            mytho_category: City,
            system_role: Suite,
            domain_tags: &["client documentation", "EnkiDDB", "SPPS", "ingest/write side", "pairs with Nuzi"],
            status: Reserved,
            source_doc: Some("docs/14_decisions_adr/adr_021_eshnunna_susa_nuzi_naming_seal.md"),
            crate_path: None,
            blurb: "Ancient Sumerian/Babylonian city on the Euphrates, site of the Ebabbar temple's real, celebrated tablet library/archive -- a bulk accumulated repository. Abbreviated SPPS. Sealed 2026-08-01, narrowed 2026-08-01 (ADR-021) to the INGEST/WRITE side of the client-document archive inside EnkiDDB specifically -- where documents land and get persisted -- paired with Nuzi, which now holds the retrieval/query side. Same shape as the Write=Journal/Read=Datafiles CQRS split already used by every one of the 7 Types EnkiDB. Reserved, not yet backing shipped code.",
            image_path: None,
            image_credit: None,
        },
        NamingEntry {
            name: "Nuzi",
            mytho_category: City,
            system_role: Suite,
            domain_tags: &["client documentation", "EnkiDDB", "retrieval/query side", "pairs with SipparStore"],
            status: Reserved,
            source_doc: Some("docs/14_decisions_adr/adr_021_eshnunna_susa_nuzi_naming_seal.md"),
            crate_path: None,
            blurb: "Ancient Hurrian-period city (modern Yorghan Tepe, near Kirkuk), famous for thousands of individually retrieved family, legal, and administrative tablets -- adoptions, land transfers, loans, inheritance. Reserved 2026-08-01 (ADR-021) as the RETRIEVAL/QUERY side of the client-document archive inside EnkiDDB: looking up one record and proving its provenance/lineage, paired with SipparStore's ingest/write side. Also formally reserved for this exact role ('inward archive') in the unsealed NL-001 draft law (2026-07-11), independently of this session's own naming search. Reserved, not yet backing shipped code.",
            image_path: None,
            image_credit: None,
        },
        NamingEntry {
            name: "Susa",
            mytho_category: City,
            system_role: Engine,
            domain_tags: &["outward gateway", "import threshold", "EnkiDDB", "EnkiMDB"],
            status: SealedByLaw,
            source_doc: Some("docs/14_decisions_adr/adr_021_eshnunna_susa_nuzi_naming_seal.md"),
            crate_path: Some("crates/susa-engine/src/lib.rs"),
            blurb: "Capital of Elam, later an Achaemenid royal seat -- a gateway city where major flows converged, and where the Code of Hammurabi stele was found (carried there as war spoil), charging it with law and cross-boundary exchange. Names the already-real `SusaEngine`, 'the sovereign import THRESHOLD... the guardian of the boundary' where external artifacts cross in before minting (documents -> EnkiDDB, service/app structures + metadata -> EnkiMDB). Registered 2026-08-01 (ADR-021) -- the code (9/9 tests passing) predates this entry; no new name was needed for client-document ingestion because this engine already fills that role.",
            image_path: None,
            image_credit: None,
        },
        NamingEntry {
            name: "Eshnunna",
            mytho_category: City,
            system_role: Engine,
            domain_tags: &["columnar storage", "KISPU HeadStore", "surrogate offset retrieval", "Step 0b"],
            status: SealedByLaw,
            source_doc: Some("docs/14_decisions_adr/adr_021_eshnunna_susa_nuzi_naming_seal.md"),
            crate_path: Some("crates/eshnunna-engine/src/lib.rs"),
            blurb: "City in the Diyala region (Tell Asmar), personally significant to the Architect (its remains sit in his own neighborhood) and historically famous for the Laws of Eshnunna, one of the earliest codified legal systems, predating Hammurabi by roughly two centuries -- a place that organized scattered claims into ordered, retrievable law. Names `EshnunnaEngine`, the sovereign columnar data-file engine: surrogate u32 -> fixed byte offset -> mmap'd column value, fixing the real journal-walk retrieval bottleneck that made HeptaScript queries degrade from ~5s at 100 particles to hours at 10,000. Sealed 2026-08-01 (ADR-021); not yet wired into the live read path.",
            image_path: None,
            image_credit: None,
        },
        NamingEntry {
            name: "Purussum",
            mytho_category: GenericTerm,
            system_role: Calculus,
            domain_tags: &["critical slowing down", "bifurcation detection", "ninurta-engine"],
            status: SealedByLaw,
            source_doc: Some("docs/14_decisions_adr/adr_018_topological_engine_division.md"),
            crate_path: Some("crates/ninurta-engine/src/purussum.rs"),
            blurb: "'Decision, verdict, a legal determination' -- from parasu, 'to cut, to separate, to decide,' carrying both the bifurcation-fork and the decision-output in one root. Names NinurtaEngine's own calculus (detrend -> lambda -> surrogate significance -> verdict), never modified with an adjective (an earlier 'Purussum Ensu' candidate was rejected as undignified -- the humility belongs in the tau-score, not the name). Verified unspent before sealing (2026-07-31).",
            image_path: None,
            image_credit: None,
        },
        NamingEntry {
            name: "Namtar",
            mytho_category: Deity,
            system_role: Agent,
            domain_tags: &["kaki", "death-split channel"],
            status: ExistingUnverified,
            source_doc: None,
            crate_path: Some("crates/namtar-kaki/src/lib.rs"),
            blurb: "Messenger of fate/death in the underworld. Names the death-split/legacy KAKI channel crate. No dedicated naming-law tablet found.",
            image_path: None,
            image_credit: None,
        },
        NamingEntry {
            name: "Enkidu",
            mytho_category: Deity,
            system_role: Protocol,
            domain_tags: &["tcp wire protocol"],
            status: ExistingUnverified,
            source_doc: None,
            crate_path: Some("crates/enkidu-protocol/src/lib.rs"),
            blurb: "Gilgamesh's companion. Names the ENKIDU TCP wire protocol (SPSC ring, buffer pool, frame codec). No dedicated naming-law tablet found justifying the protocol/companion metaphor specifically.",
            image_path: None,
            image_credit: None,
        },
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn no_duplicate_names() {
        let reg = seed();
        let mut names: Vec<String> = reg.0.iter().map(|e| e.name.to_lowercase()).collect();
        let before = names.len();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), before, "duplicate name found in the registry");
    }

    #[test]
    fn sealed_by_law_entries_always_cite_a_source_doc() {
        let reg = seed();
        for e in &reg.0 {
            if e.status == NamingStatus::SealedByLaw {
                assert!(e.source_doc.is_some(), "{} is SealedByLaw but has no source_doc", e.name);
            }
        }
    }

    #[test]
    fn reserved_entries_have_no_crate_path_yet() {
        let reg = seed();
        for e in &reg.0 {
            if e.status == NamingStatus::Reserved {
                assert!(e.crate_path.is_none(), "{} is Reserved but already claims a crate_path -- update its status", e.name);
            }
        }
    }

    /// Every entry that claims a `crate_path` must point at a file
    /// that actually exists in this workspace -- a naming entry can
    /// never silently reference a crate that isn't real.
    #[test]
    fn every_crate_path_exists() {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
        let reg = seed();
        for e in &reg.0 {
            if let Some(p) = e.crate_path {
                let full = workspace_root.join(p);
                assert!(full.exists(), "{}: crate_path {:?} does not exist at {:?}", e.name, p, full);
            }
        }
    }

    /// An image with no stated credit is exactly the Nisaba-attribution
    /// problem again (an uploaded image confidently labeled without ever
    /// verifying the source) -- fail the build before it ships, not after
    /// a wrong attribution is live in the Glossary panel.
    #[test]
    fn every_image_has_a_credit() {
        let reg = seed();
        for e in &reg.0 {
            if e.image_path.is_some() {
                assert!(e.image_credit.is_some(), "{}: image_path is set but image_credit is None", e.name);
            }
        }
    }

    /// Every image_path that IS set must point at a real, committed file
    /// -- same law as every_crate_path_exists, applied to images.
    #[test]
    fn every_image_path_exists() {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
        let reg = seed();
        for e in &reg.0 {
            if let Some(p) = e.image_path {
                let full = workspace_root.join(p);
                assert!(full.exists(), "{}: image_path {:?} does not exist at {:?}", e.name, p, full);
            }
        }
    }

    #[test]
    fn queries_are_consistent_with_the_full_set() {
        let reg = seed();
        let deities = reg.by_mytho_category(MythoCategory::Deity);
        assert!(!deities.is_empty());
        assert!(deities.iter().all(|e| e.mytho_category == MythoCategory::Deity));

        let electricity = reg.by_domain("electricity");
        assert!(electricity.len() >= 2, "expected Gibil and Birqu at least");

        assert!(reg.find("gibil").is_some(), "find() must be case-insensitive");
        assert!(reg.find("GIBIL").is_some());
        assert!(reg.find("nonexistent-name").is_none());
    }

    #[test]
    fn reserved_ishum_is_actually_reserved_not_shipped() {
        let reg = seed();
        let ishum = reg.find("Ishum").expect("Ishum must be seeded");
        assert_eq!(ishum.status, NamingStatus::Reserved);
        assert!(ishum.crate_path.is_none());
    }

    #[test]
    fn as_str_round_trips_every_variant_to_a_distinct_label() {
        let cats = [MythoCategory::Deity, MythoCategory::City, MythoCategory::KingOrEra, MythoCategory::GenericTerm];
        let mut labels: Vec<&str> = cats.iter().map(|c| c.as_str()).collect();
        labels.sort();
        labels.dedup();
        assert_eq!(labels.len(), cats.len());

        let roles = [
            SystemRole::Engine,
            SystemRole::Calculus,
            SystemRole::AlertClass,
            SystemRole::Pattern,
            SystemRole::Agent,
            SystemRole::Suite,
            SystemRole::Tool,
            SystemRole::Protocol,
            SystemRole::IndexStack,
            SystemRole::Language,
        ];
        let mut role_labels: Vec<&str> = roles.iter().map(|r| r.as_str()).collect();
        role_labels.sort();
        role_labels.dedup();
        assert_eq!(role_labels.len(), roles.len());
    }

    #[test]
    fn search_is_case_insensitive_and_matches_name_blurb_and_domain() {
        let reg = seed();
        assert!(!reg.search("gibil").is_empty());
        assert!(!reg.search("GIBIL").is_empty());
        assert!(!reg.search("smith-fire").is_empty(), "must match against blurb text");
        assert!(!reg.search("electricity").is_empty(), "must match against domain_tags");
        assert!(reg.search("no-such-thing-in-the-registry").is_empty());
        assert_eq!(reg.search("").len(), reg.0.len(), "empty query returns everything");
    }

    #[test]
    fn read_snippet_returns_real_file_content_for_a_shipped_entry() {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
        let reg = seed();
        let gibil = reg.find("Gibil").expect("Gibil must be seeded");
        let snippet = read_snippet(&workspace_root, gibil, 60).expect("Gibil has a real crate_path and must read");
        assert!(snippet.contains("Gibil"), "snippet should mention Gibil somewhere in its own header comment");
    }

    #[test]
    fn read_snippet_is_honest_about_reserved_entries_having_no_file() {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
        let reg = seed();
        let ishum = reg.find("Ishum").expect("Ishum must be seeded");
        let err = read_snippet(&workspace_root, ishum, 60).unwrap_err();
        assert!(err.contains("reserved"), "error must explain why there's no snippet, got: {err}");
    }
}
