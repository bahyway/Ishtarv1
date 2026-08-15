//! DmBokTerm — a DAMA-DMBOK governance vocabulary entry (§6.5).
//!
//! The dictionary is a static compile-time table. BahyWay uses DAMA-DMBOK v2
//! as the authoritative data governance reference.

/// DAMA-DMBOK Knowledge Area — top-level governance domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KnowledgeArea {
    DataGovernance,
    DataArchitecture,
    DataModeling,
    DataStorageAndOperations,
    DataSecurity,
    DataIntegration,
    DocumentAndContent,
    ReferenceAndMasterData,
    DataWarehousing,
    Metadata,
    DataQuality,
}

impl KnowledgeArea {
    pub fn code(self) -> &'static str {
        match self {
            Self::DataGovernance          => "DG",
            Self::DataArchitecture        => "DA",
            Self::DataModeling            => "DM",
            Self::DataStorageAndOperations => "DSO",
            Self::DataSecurity            => "DS",
            Self::DataIntegration         => "DI",
            Self::DocumentAndContent      => "DC",
            Self::ReferenceAndMasterData  => "RMD",
            Self::DataWarehousing         => "DW",
            Self::Metadata                => "MD",
            Self::DataQuality             => "DQ",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::DataGovernance          => "Data Governance",
            Self::DataArchitecture        => "Data Architecture",
            Self::DataModeling            => "Data Modelling & Design",
            Self::DataStorageAndOperations => "Data Storage & Operations",
            Self::DataSecurity            => "Data Security",
            Self::DataIntegration         => "Data Integration & Interoperability",
            Self::DocumentAndContent      => "Document & Content Management",
            Self::ReferenceAndMasterData  => "Reference & Master Data",
            Self::DataWarehousing         => "Data Warehousing & Business Intelligence",
            Self::Metadata                => "Metadata Management",
            Self::DataQuality             => "Data Quality",
        }
    }

    /// Returns all 11 DAMA knowledge areas.
    pub fn all() -> &'static [KnowledgeArea] {
        &[
            KnowledgeArea::DataGovernance,
            KnowledgeArea::DataArchitecture,
            KnowledgeArea::DataModeling,
            KnowledgeArea::DataStorageAndOperations,
            KnowledgeArea::DataSecurity,
            KnowledgeArea::DataIntegration,
            KnowledgeArea::DocumentAndContent,
            KnowledgeArea::ReferenceAndMasterData,
            KnowledgeArea::DataWarehousing,
            KnowledgeArea::Metadata,
            KnowledgeArea::DataQuality,
        ]
    }

    /// DAMA-DMBOK v2 chapter number (1–11).
    pub fn chapter_number(self) -> u8 {
        match self {
            Self::DataGovernance          => 3,
            Self::DataArchitecture        => 4,
            Self::DataModeling            => 5,
            Self::DataStorageAndOperations => 6,
            Self::DataSecurity            => 7,
            Self::DataIntegration         => 8,
            Self::DocumentAndContent      => 9,
            Self::ReferenceAndMasterData  => 10,
            Self::DataWarehousing         => 11,
            Self::Metadata                => 12,
            Self::DataQuality             => 13,
        }
    }
}

/// A single DAMA-DMBOK vocabulary entry.
#[derive(Debug, Clone)]
pub struct DmBokTerm {
    /// Short code (e.g. "DQ.COMPLETENESS").
    pub code:        &'static str,
    /// Full label.
    pub label:       &'static str,
    /// Concise definition (one sentence).
    pub definition:  &'static str,
    /// Primary DAMA-DMBOK knowledge area.
    pub area:        KnowledgeArea,
}

impl DmBokTerm {
    pub const fn new(
        code:       &'static str,
        label:      &'static str,
        definition: &'static str,
        area:       KnowledgeArea,
    ) -> Self {
        DmBokTerm { code, label, definition, area }
    }
}

// ── Static dictionary ─────────────────────────────────────────────────────────

pub static DICTIONARY: &[DmBokTerm] = &[
    // ── Data Quality (DQ) ────────────────────────────────────────────────────
    DmBokTerm::new(
        "DQ.COMPLETENESS",
        "Data Completeness",
        "The degree to which all required data values are present.",
        KnowledgeArea::DataQuality,
    ),
    DmBokTerm::new(
        "DQ.ACCURACY",
        "Data Accuracy",
        "The degree to which data correctly represents the real-world entity.",
        KnowledgeArea::DataQuality,
    ),
    DmBokTerm::new(
        "DQ.FRESHNESS",
        "Data Freshness",
        "The degree to which data reflects the current state of the entity it represents.",
        KnowledgeArea::DataQuality,
    ),
    DmBokTerm::new(
        "DQ.CONSISTENCY",
        "Data Consistency",
        "The absence of contradiction between data values across records and systems.",
        KnowledgeArea::DataQuality,
    ),
    DmBokTerm::new(
        "DQ.VALIDITY",
        "Data Validity",
        "The degree to which data conforms to defined formats, types, and constraints.",
        KnowledgeArea::DataQuality,
    ),
    DmBokTerm::new(
        "DQ.UNIQUENESS",
        "Data Uniqueness",
        "The absence of duplicate records for the same real-world entity.",
        KnowledgeArea::DataQuality,
    ),
    DmBokTerm::new(
        "DQ.TIMELINESS",
        "Data Timeliness",
        "The degree to which data is available at the time it is needed for use.",
        KnowledgeArea::DataQuality,
    ),
    DmBokTerm::new(
        "DQ.INTEGRITY",
        "Data Integrity",
        "The overall correctness, consistency, and trustworthiness of data over its lifecycle.",
        KnowledgeArea::DataQuality,
    ),
    DmBokTerm::new(
        "DQ.CONFORMITY",
        "Data Conformity",
        "The degree to which data adheres to a defined standard, schema, or format.",
        KnowledgeArea::DataQuality,
    ),
    DmBokTerm::new(
        "DQ.FUZZY_SCORE",
        "Fuzzy Quality Score",
        "A composite 0–255 score (B11 byte) aggregating all fuzzy quality dimensions via membership functions.",
        KnowledgeArea::DataQuality,
    ),
    DmBokTerm::new(
        "DQ.B11_BYTE",
        "B11 Quality Byte",
        "The single-byte (0–255) sovereign quality encoding stored as the 11th byte of the particle identity.",
        KnowledgeArea::DataQuality,
    ),
    DmBokTerm::new(
        "DQ.GEM_TIER",
        "Gem Quality Tier",
        "The highest quality tier (B11 >= 200) representing near-perfect data records in BahyWay.",
        KnowledgeArea::DataQuality,
    ),
    DmBokTerm::new(
        "DQ.TRIBE_TIER",
        "Tribe Quality Tier",
        "The intermediate quality tier (B11 100–199) for records meeting minimum tribal standards.",
        KnowledgeArea::DataQuality,
    ),
    DmBokTerm::new(
        "DQ.ACTIVE_TIER",
        "Active Quality Tier",
        "The lower-intermediate quality tier (B11 50–99) for records still operationally active.",
        KnowledgeArea::DataQuality,
    ),
    DmBokTerm::new(
        "DQ.DEAD_TIER",
        "Dead Quality Tier",
        "The lowest quality tier (B11 < 50) for records flagged as unreliable or decomissioned.",
        KnowledgeArea::DataQuality,
    ),
    DmBokTerm::new(
        "DQ.ENTROPY",
        "Data Entropy",
        "A measure of disorder in the particle population — ratio of non-Golden particles to total particles.",
        KnowledgeArea::DataQuality,
    ),
    DmBokTerm::new(
        "DQ.COHERENCE",
        "Data Coherence",
        "The inverse of entropy — the proportion of the particle population in Golden state.",
        KnowledgeArea::DataQuality,
    ),
    // ── Reference & Master Data (RMD) ────────────────────────────────────────
    DmBokTerm::new(
        "RMD.MASTERDATA",
        "Master Data",
        "Shared data that provides consistent context for business transactions.",
        KnowledgeArea::ReferenceAndMasterData,
    ),
    DmBokTerm::new(
        "RMD.GOLDRECORD",
        "Golden Record",
        "The single, authoritative, trusted version of master data for an entity.",
        KnowledgeArea::ReferenceAndMasterData,
    ),
    DmBokTerm::new(
        "RMD.REFERENCE_DATA",
        "Reference Data",
        "Data used to classify or categorize other data, typically maintained in lookup tables.",
        KnowledgeArea::ReferenceAndMasterData,
    ),
    DmBokTerm::new(
        "RMD.HIERARCHY",
        "Data Hierarchy",
        "A structured parent-child relationship among master data entities (e.g. region > country > city).",
        KnowledgeArea::ReferenceAndMasterData,
    ),
    DmBokTerm::new(
        "RMD.SURVIVORSHIP",
        "Survivorship Rule",
        "The logic that determines which attribute values survive when merging duplicate records.",
        KnowledgeArea::ReferenceAndMasterData,
    ),
    DmBokTerm::new(
        "RMD.DEDUPLICATION",
        "Deduplication",
        "The process of identifying and removing duplicate records representing the same real-world entity.",
        KnowledgeArea::ReferenceAndMasterData,
    ),
    DmBokTerm::new(
        "RMD.MERGE_PURGE",
        "Merge-Purge",
        "A data integration pattern that merges duplicate records and purges the superseded copies.",
        KnowledgeArea::ReferenceAndMasterData,
    ),
    DmBokTerm::new(
        "RMD.PARTICLE_STATE",
        "Particle State",
        "The tri-state lifecycle status of a BahyWay sovereign particle: Golden, Fuzzy, or Dead.",
        KnowledgeArea::ReferenceAndMasterData,
    ),
    DmBokTerm::new(
        "RMD.FUZZY_RECORD",
        "Fuzzy Record",
        "A particle whose B11 score indicates insufficient quality for Golden promotion; requires steward review.",
        KnowledgeArea::ReferenceAndMasterData,
    ),
    DmBokTerm::new(
        "RMD.DEAD_RECORD",
        "Dead Record",
        "A particle that has been logically retired from active use; preserved in the journal for audit purposes.",
        KnowledgeArea::ReferenceAndMasterData,
    ),
    // ── Data Governance (DG) ─────────────────────────────────────────────────
    DmBokTerm::new(
        "DG.STEWARDSHIP",
        "Data Stewardship",
        "Formal accountability for managing data assets on behalf of data owners.",
        KnowledgeArea::DataGovernance,
    ),
    DmBokTerm::new(
        "DG.LINEAGE",
        "Data Lineage",
        "The record of data origins and transformations across its lifecycle.",
        KnowledgeArea::DataGovernance,
    ),
    DmBokTerm::new(
        "DG.POLICY",
        "Data Policy",
        "A formal statement of intent governing how data is managed, shared, and protected.",
        KnowledgeArea::DataGovernance,
    ),
    DmBokTerm::new(
        "DG.OWNERSHIP",
        "Data Ownership",
        "The accountability assigned to a person or role for the quality and proper use of a data asset.",
        KnowledgeArea::DataGovernance,
    ),
    DmBokTerm::new(
        "DG.ACCOUNTABILITY",
        "Data Accountability",
        "The obligation of individuals and organizations to answer for data-related decisions and outcomes.",
        KnowledgeArea::DataGovernance,
    ),
    DmBokTerm::new(
        "DG.COMPLIANCE",
        "Data Compliance",
        "Adherence to applicable laws, regulations, standards, and internal policies governing data.",
        KnowledgeArea::DataGovernance,
    ),
    DmBokTerm::new(
        "DG.FRAMEWORK",
        "Governance Framework",
        "The structured set of roles, policies, processes, and standards that govern data management.",
        KnowledgeArea::DataGovernance,
    ),
    DmBokTerm::new(
        "DG.COUNCIL",
        "Data Governance Council",
        "A cross-functional body responsible for approving data policies and resolving governance issues.",
        KnowledgeArea::DataGovernance,
    ),
    DmBokTerm::new(
        "DG.SLA",
        "Data Service Level Agreement",
        "A formal commitment defining the expected quality, availability, and timeliness of a data asset.",
        KnowledgeArea::DataGovernance,
    ),
    DmBokTerm::new(
        "DG.AUDIT_TRAIL",
        "Data Audit Trail",
        "An immutable chronological record of all changes and accesses to a data asset.",
        KnowledgeArea::DataGovernance,
    ),
    DmBokTerm::new(
        "DG.DATA_OWNER",
        "Data Owner",
        "The business role accountable for defining data access rules and quality expectations.",
        KnowledgeArea::DataGovernance,
    ),
    DmBokTerm::new(
        "DG.DATA_CUSTODIAN",
        "Data Custodian",
        "The technical role responsible for the physical storage, maintenance, and security of data.",
        KnowledgeArea::DataGovernance,
    ),
    // ── Data Architecture (DA) ───────────────────────────────────────────────
    DmBokTerm::new(
        "DA.BLUEPRINT",
        "Data Architecture Blueprint",
        "The master design document describing data structures, flows, and integration patterns.",
        KnowledgeArea::DataArchitecture,
    ),
    DmBokTerm::new(
        "DA.CANONICAL_MODEL",
        "Canonical Data Model",
        "A shared, agreed-upon data model that serves as the authoritative schema for integration.",
        KnowledgeArea::DataArchitecture,
    ),
    DmBokTerm::new(
        "DA.DATA_FLOW",
        "Data Flow",
        "The movement of data between systems, processes, and storage layers within an architecture.",
        KnowledgeArea::DataArchitecture,
    ),
    DmBokTerm::new(
        "DA.DOMAIN",
        "Data Domain",
        "A bounded subject area that groups related data entities under a single business concept.",
        KnowledgeArea::DataArchitecture,
    ),
    DmBokTerm::new(
        "DA.SCHEMA_EVOLUTION",
        "Schema Evolution",
        "The controlled process of changing a data schema over time while maintaining backward compatibility.",
        KnowledgeArea::DataArchitecture,
    ),
    DmBokTerm::new(
        "DA.INTEGRATION_LAYER",
        "Integration Layer",
        "The architectural tier responsible for mediating data exchange between heterogeneous systems.",
        KnowledgeArea::DataArchitecture,
    ),
    DmBokTerm::new(
        "DA.SOVEREIGN_PARTICLE",
        "Sovereign Particle",
        "The fundamental unit of data in BahyWay — a self-describing, immutable, quality-scored entity.",
        KnowledgeArea::DataArchitecture,
    ),
    DmBokTerm::new(
        "DA.EAV",
        "Entity-Attribute-Value",
        "A flexible data model where each attribute of an entity is stored as a discrete key-value triple.",
        KnowledgeArea::DataArchitecture,
    ),
    DmBokTerm::new(
        "DA.KAKI",
        "Kaki Identity",
        "BahyWay's 16-byte sovereign identity structure combining tribe, type, UUID hash, and quality byte.",
        KnowledgeArea::DataArchitecture,
    ),
    DmBokTerm::new(
        "DA.TRIBE",
        "Tribe",
        "A 16-bit sovereign namespace that partitions all data within a BahyWay deployment.",
        KnowledgeArea::DataArchitecture,
    ),
    DmBokTerm::new(
        "DA.ORBIT",
        "Particle Orbit",
        "The conceptual ring of particles in a tribe, ordered by epoch and quality score.",
        KnowledgeArea::DataArchitecture,
    ),
    DmBokTerm::new(
        "DA.BIG_RING",
        "Big Ring",
        "The complete ordered set of all sovereign particles across all epochs in a BahyWay tribe.",
        KnowledgeArea::DataArchitecture,
    ),
    // ── Data Modeling (DM) ───────────────────────────────────────────────────
    DmBokTerm::new(
        "DM.CONCEPTUAL",
        "Conceptual Data Model",
        "A high-level model representing key business entities and relationships without technical detail.",
        KnowledgeArea::DataModeling,
    ),
    DmBokTerm::new(
        "DM.LOGICAL",
        "Logical Data Model",
        "A model that defines data structures in terms of entities, attributes, and relationships independent of storage.",
        KnowledgeArea::DataModeling,
    ),
    DmBokTerm::new(
        "DM.PHYSICAL",
        "Physical Data Model",
        "A model that specifies exact storage structures, data types, indexes, and constraints.",
        KnowledgeArea::DataModeling,
    ),
    DmBokTerm::new(
        "DM.ENTITY",
        "Data Entity",
        "A distinct, identifiable object or concept about which data is stored.",
        KnowledgeArea::DataModeling,
    ),
    DmBokTerm::new(
        "DM.ATTRIBUTE",
        "Data Attribute",
        "A property or characteristic of an entity that describes it (e.g. name, date_of_birth).",
        KnowledgeArea::DataModeling,
    ),
    DmBokTerm::new(
        "DM.RELATIONSHIP",
        "Data Relationship",
        "An association between two or more data entities that has business meaning.",
        KnowledgeArea::DataModeling,
    ),
    DmBokTerm::new(
        "DM.NORMALIZATION",
        "Data Normalization",
        "The process of organizing data to reduce redundancy and improve integrity via normal forms.",
        KnowledgeArea::DataModeling,
    ),
    DmBokTerm::new(
        "DM.DENORMALIZATION",
        "Data Denormalization",
        "The deliberate introduction of redundancy into a model to improve read performance.",
        KnowledgeArea::DataModeling,
    ),
    DmBokTerm::new(
        "DM.ERD",
        "Entity-Relationship Diagram",
        "A visual representation of data entities and their relationships in a data model.",
        KnowledgeArea::DataModeling,
    ),
    DmBokTerm::new(
        "DM.ONTOLOGY",
        "Data Ontology",
        "A formal, machine-readable specification of concepts, their properties, and inter-relationships in a domain.",
        KnowledgeArea::DataModeling,
    ),
    // ── Data Storage & Operations (DSO) ─────────────────────────────────────
    DmBokTerm::new(
        "DSO.ARCHIVAL",
        "Data Archival",
        "The process of moving inactive data to long-term storage while preserving retrievability.",
        KnowledgeArea::DataStorageAndOperations,
    ),
    DmBokTerm::new(
        "DSO.JOURNAL",
        "Data Journal",
        "An append-only sequential log recording all events and state changes for a set of particles.",
        KnowledgeArea::DataStorageAndOperations,
    ),
    DmBokTerm::new(
        "DSO.APPEND_ONLY",
        "Append-Only Storage",
        "A storage model that prohibits modification or deletion — all changes are new appended records.",
        KnowledgeArea::DataStorageAndOperations,
    ),
    DmBokTerm::new(
        "DSO.CRASH_RECOVERY",
        "Crash Recovery",
        "The ability to restore a consistent database state after an unexpected failure using durable markers.",
        KnowledgeArea::DataStorageAndOperations,
    ),
    DmBokTerm::new(
        "DSO.FSYNC",
        "Fsync Durability",
        "The guarantee that data is physically written to storage media before a commit is acknowledged.",
        KnowledgeArea::DataStorageAndOperations,
    ),
    DmBokTerm::new(
        "DSO.PARTITION",
        "Data Partition",
        "A logical division of data into subsets, typically to improve query performance or scalability.",
        KnowledgeArea::DataStorageAndOperations,
    ),
    DmBokTerm::new(
        "DSO.SHARD",
        "Data Shard",
        "A horizontal partition of data across multiple storage nodes to distribute load.",
        KnowledgeArea::DataStorageAndOperations,
    ),
    DmBokTerm::new(
        "DSO.SNAPSHOT",
        "Data Snapshot",
        "A point-in-time capture of data state used for backup, reporting, or time-travel queries.",
        KnowledgeArea::DataStorageAndOperations,
    ),
    DmBokTerm::new(
        "DSO.REPLAY",
        "Journal Replay",
        "The process of re-applying journal entries to reconstruct data state after startup or recovery.",
        KnowledgeArea::DataStorageAndOperations,
    ),
    DmBokTerm::new(
        "DSO.RETENTION",
        "Data Retention",
        "A policy specifying how long data must be kept before it can be archived or destroyed.",
        KnowledgeArea::DataStorageAndOperations,
    ),
    // ── Data Security (DS) ───────────────────────────────────────────────────
    DmBokTerm::new(
        "DS.SOVEREIGNTY",
        "Data Sovereignty",
        "The principle that data is subject to the laws and governance of the tribe or jurisdiction where it originates.",
        KnowledgeArea::DataSecurity,
    ),
    DmBokTerm::new(
        "DS.CLEARANCE",
        "Data Clearance",
        "The authorization level required to access data of a given security classification.",
        KnowledgeArea::DataSecurity,
    ),
    DmBokTerm::new(
        "DS.ACCESS_CONTROL",
        "Access Control",
        "Mechanisms that restrict data access to authorized identities based on defined policies.",
        KnowledgeArea::DataSecurity,
    ),
    DmBokTerm::new(
        "DS.ENCRYPTION",
        "Data Encryption",
        "The transformation of data into an unreadable form to prevent unauthorized disclosure.",
        KnowledgeArea::DataSecurity,
    ),
    DmBokTerm::new(
        "DS.AUDIT_LOG",
        "Security Audit Log",
        "A tamper-evident record of all security-relevant events including access, changes, and violations.",
        KnowledgeArea::DataSecurity,
    ),
    DmBokTerm::new(
        "DS.CLASSIFICATION",
        "Data Classification",
        "The assignment of sensitivity labels to data assets to guide handling and protection requirements.",
        KnowledgeArea::DataSecurity,
    ),
    DmBokTerm::new(
        "DS.PRIVACY",
        "Data Privacy",
        "The right of individuals to control the collection, use, and disclosure of their personal data.",
        KnowledgeArea::DataSecurity,
    ),
    DmBokTerm::new(
        "DS.ANONYMIZATION",
        "Data Anonymization",
        "The irreversible process of removing or obscuring identifying information from data.",
        KnowledgeArea::DataSecurity,
    ),
    DmBokTerm::new(
        "DS.WAY_FILE",
        "WAY File",
        "A BahyWay .way security descriptor using WAYv2.0 language to declare sovereign access rules.",
        KnowledgeArea::DataSecurity,
    ),
    DmBokTerm::new(
        "DS.MUSARU",
        "Musaru Security",
        "BahyWay's sovereign security engine that enforces tribe-level data sovereignty checks.",
        KnowledgeArea::DataSecurity,
    ),
    // ── Data Integration (DI) ────────────────────────────────────────────────
    DmBokTerm::new(
        "DI.ETL",
        "Extract-Transform-Load",
        "A data integration pattern extracting data from source, transforming it, and loading into a target.",
        KnowledgeArea::DataIntegration,
    ),
    DmBokTerm::new(
        "DI.ELT",
        "Extract-Load-Transform",
        "A data integration variant where raw data is loaded first and transformed inside the target system.",
        KnowledgeArea::DataIntegration,
    ),
    DmBokTerm::new(
        "DI.LANDING_ZONE",
        "Landing Zone",
        "A staging area where raw source data lands before quality assessment and transformation.",
        KnowledgeArea::DataIntegration,
    ),
    DmBokTerm::new(
        "DI.PIPELINE",
        "Data Pipeline",
        "A sequence of automated steps that move and transform data from source to destination.",
        KnowledgeArea::DataIntegration,
    ),
    DmBokTerm::new(
        "DI.SCHEMA_COMPARISON",
        "Schema Comparison",
        "The analysis of two schema versions to identify added, removed, or changed attributes.",
        KnowledgeArea::DataIntegration,
    ),
    DmBokTerm::new(
        "DI.STAGING",
        "Data Staging",
        "A temporary holding area where data is cleaned and transformed before final loading.",
        KnowledgeArea::DataIntegration,
    ),
    DmBokTerm::new(
        "DI.TRANSFORMATION",
        "Data Transformation",
        "The process of converting data from one format, structure, or value space to another.",
        KnowledgeArea::DataIntegration,
    ),
    DmBokTerm::new(
        "DI.VALIDATION",
        "Data Validation",
        "The process of checking data against defined rules before acceptance into the target system.",
        KnowledgeArea::DataIntegration,
    ),
    DmBokTerm::new(
        "DI.RECONCILIATION",
        "Data Reconciliation",
        "The process of ensuring data consistency between source and target after a transfer.",
        KnowledgeArea::DataIntegration,
    ),
    DmBokTerm::new(
        "DI.DEAD_LETTER",
        "Dead Letter Queue",
        "A holding area for records that failed validation or processing, awaiting remediation.",
        KnowledgeArea::DataIntegration,
    ),
    // ── Document & Content (DC) ──────────────────────────────────────────────
    DmBokTerm::new(
        "DC.RECORD",
        "Business Record",
        "A document or data object that serves as evidence of a business event or decision.",
        KnowledgeArea::DocumentAndContent,
    ),
    DmBokTerm::new(
        "DC.CONTENT_TYPE",
        "Content Type",
        "A classification of document or content by format, purpose, or domain (e.g. contract, report).",
        KnowledgeArea::DocumentAndContent,
    ),
    DmBokTerm::new(
        "DC.RETENTION_POLICY",
        "Content Retention Policy",
        "Rules governing how long documents must be retained and when they may be destroyed.",
        KnowledgeArea::DocumentAndContent,
    ),
    DmBokTerm::new(
        "DC.VERSION_CONTROL",
        "Document Version Control",
        "The management of successive revisions of a document with change history and rollback capability.",
        KnowledgeArea::DocumentAndContent,
    ),
    DmBokTerm::new(
        "DC.IMMUTABILITY",
        "Content Immutability",
        "The property that once committed, a document cannot be altered — only superseded by new versions.",
        KnowledgeArea::DocumentAndContent,
    ),
    DmBokTerm::new(
        "DC.TEMPLATE",
        "Document Template",
        "A reusable structure defining required and optional fields for a class of business documents.",
        KnowledgeArea::DocumentAndContent,
    ),
    DmBokTerm::new(
        "DC.CUNEIFORM",
        "Cuneiform Encoding",
        "BahyWay's reference to ancient Sumerian record-keeping as the conceptual origin of append-only sovereign data.",
        KnowledgeArea::DocumentAndContent,
    ),
    // ── Data Warehousing (DW) ────────────────────────────────────────────────
    DmBokTerm::new(
        "DW.STAR_SCHEMA",
        "Star Schema",
        "A dimensional model with a central fact table connected to surrounding dimension tables.",
        KnowledgeArea::DataWarehousing,
    ),
    DmBokTerm::new(
        "DW.DIMENSION",
        "Dimension",
        "A descriptive attribute used to slice and filter facts in a dimensional data model.",
        KnowledgeArea::DataWarehousing,
    ),
    DmBokTerm::new(
        "DW.FACT_TABLE",
        "Fact Table",
        "A central table in a star schema containing quantitative measures and foreign keys to dimensions.",
        KnowledgeArea::DataWarehousing,
    ),
    DmBokTerm::new(
        "DW.OLAP",
        "Online Analytical Processing",
        "A category of software enabling fast, multidimensional analysis of large data volumes.",
        KnowledgeArea::DataWarehousing,
    ),
    DmBokTerm::new(
        "DW.CUBE",
        "OLAP Cube",
        "A multidimensional data structure enabling rapid aggregation and drill-down analysis.",
        KnowledgeArea::DataWarehousing,
    ),
    DmBokTerm::new(
        "DW.SLOWLY_CHANGING",
        "Slowly Changing Dimension",
        "A dimension whose attribute values change infrequently, requiring history-tracking strategies.",
        KnowledgeArea::DataWarehousing,
    ),
    DmBokTerm::new(
        "DW.DATA_MART",
        "Data Mart",
        "A subject-specific subset of a data warehouse optimized for a particular business domain.",
        KnowledgeArea::DataWarehousing,
    ),
    DmBokTerm::new(
        "DW.GOLDEN_RATIO",
        "Golden Ratio",
        "The proportion of Golden-state particles to total particles — a BahyWay sovereignty health metric.",
        KnowledgeArea::DataWarehousing,
    ),
    DmBokTerm::new(
        "DW.EPOCH",
        "Data Epoch",
        "A monotonically increasing sequence number marking the logical time of each event in the journal.",
        KnowledgeArea::DataWarehousing,
    ),
    DmBokTerm::new(
        "DW.TIME_TRAVEL",
        "Time-Travel Query",
        "The ability to query the projected state of a particle at any past epoch in the journal.",
        KnowledgeArea::DataWarehousing,
    ),
    // ── Metadata (MD) ────────────────────────────────────────────────────────
    DmBokTerm::new(
        "MD.PROVENANCE",
        "Metadata Provenance",
        "Information describing the origin, custody, and transformation history of data.",
        KnowledgeArea::Metadata,
    ),
    DmBokTerm::new(
        "MD.BUSINESS_METADATA",
        "Business Metadata",
        "Descriptive information about data assets in business terms — names, definitions, owners.",
        KnowledgeArea::Metadata,
    ),
    DmBokTerm::new(
        "MD.TECHNICAL_METADATA",
        "Technical Metadata",
        "Information about data structures, formats, schemas, and system-level properties.",
        KnowledgeArea::Metadata,
    ),
    DmBokTerm::new(
        "MD.OPERATIONAL_METADATA",
        "Operational Metadata",
        "Information about data processing events — run times, record counts, error rates.",
        KnowledgeArea::Metadata,
    ),
    DmBokTerm::new(
        "MD.DATA_CATALOG",
        "Data Catalog",
        "A curated inventory of data assets with metadata enabling discovery, understanding, and governance.",
        KnowledgeArea::Metadata,
    ),
    DmBokTerm::new(
        "MD.DATA_DICTIONARY",
        "Data Dictionary",
        "A repository defining the meaning, format, relationships, and constraints of all data elements.",
        KnowledgeArea::Metadata,
    ),
    DmBokTerm::new(
        "MD.ATTR_HASH",
        "Attribute Hash",
        "A FNV-1a 32-bit hash of an attribute name used as a compact, collision-resistant EAV key.",
        KnowledgeArea::Metadata,
    ),
    DmBokTerm::new(
        "MD.KAKI_METADATA",
        "Kaki Metadata",
        "Metadata embedded in the 16-byte Kaki identity: tribe, type, UUID hash, and B11 quality byte.",
        KnowledgeArea::Metadata,
    ),
    DmBokTerm::new(
        "MD.LINEAGE_GRAPH",
        "Lineage Graph",
        "A directed acyclic graph showing the derivation relationships between data assets.",
        KnowledgeArea::Metadata,
    ),
];

/// Look up a term by its code (case-sensitive).
pub fn lookup(code: &str) -> Option<&'static DmBokTerm> {
    DICTIONARY.iter().find(|t| t.code == code)
}

/// Return all terms in a given knowledge area.
pub fn by_area(area: KnowledgeArea) -> impl Iterator<Item = &'static DmBokTerm> {
    DICTIONARY.iter().filter(move |t| t.area == area)
}

/// Case-insensitive substring search on label + definition.
pub fn search(keyword: &str) -> Vec<&'static DmBokTerm> {
    let kw = keyword.to_lowercase();
    DICTIONARY.iter().filter(|t| {
        t.label.to_lowercase().contains(&kw) || t.definition.to_lowercase().contains(&kw)
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_known_code() {
        let t = lookup("DQ.FRESHNESS").expect("must exist");
        assert_eq!(t.label, "Data Freshness");
        assert_eq!(t.area, KnowledgeArea::DataQuality);
    }

    #[test]
    fn lookup_unknown_returns_none() {
        assert!(lookup("XX.UNKNOWN").is_none());
    }

    #[test]
    fn by_area_data_quality() {
        let dq: Vec<_> = by_area(KnowledgeArea::DataQuality).collect();
        assert!(dq.len() >= 6); // at least 6 DQ terms above
        assert!(dq.iter().all(|t| t.area == KnowledgeArea::DataQuality));
    }

    #[test]
    fn knowledge_area_codes_are_non_empty() {
        let areas = [
            KnowledgeArea::DataGovernance,
            KnowledgeArea::DataQuality,
            KnowledgeArea::ReferenceAndMasterData,
            KnowledgeArea::Metadata,
        ];
        for a in areas {
            assert!(!a.code().is_empty());
            assert!(!a.label().is_empty());
        }
    }

    #[test]
    fn golden_record_in_rmd() {
        let t = lookup("RMD.GOLDRECORD").unwrap();
        assert_eq!(t.area, KnowledgeArea::ReferenceAndMasterData);
    }

    #[test]
    fn dictionary_has_100_plus_terms() {
        assert!(DICTIONARY.len() >= 100, "Expected 100+ terms, got {}", DICTIONARY.len());
    }

    #[test]
    fn all_knowledge_areas_returns_11() {
        assert_eq!(KnowledgeArea::all().len(), 11);
    }

    #[test]
    fn chapter_numbers_in_range() {
        for ka in KnowledgeArea::all() {
            let ch = ka.chapter_number();
            assert!(ch >= 1 && ch <= 13, "chapter_number={ch} for {:?}", ka);
        }
    }

    #[test]
    fn search_golden_finds_results() {
        let hits = search("golden");
        assert!(!hits.is_empty(), "search('golden') should return results");
    }

    #[test]
    fn search_case_insensitive() {
        let lower = search("sovereign");
        let upper = search("SOVEREIGN");
        assert_eq!(lower.len(), upper.len());
    }

    #[test]
    fn all_areas_have_terms() {
        for ka in KnowledgeArea::all() {
            let terms: Vec<_> = by_area(*ka).collect();
            assert!(!terms.is_empty(), "Area {:?} has no terms", ka);
        }
    }

    #[test]
    fn no_duplicate_codes() {
        let mut seen = std::collections::HashSet::new();
        for t in DICTIONARY {
            assert!(seen.insert(t.code), "Duplicate code: {}", t.code);
        }
    }
}
