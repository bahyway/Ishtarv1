//! damadmbok-dictionary — DAMA-DMBOK data governance vocabulary (§6.5).
//!
//! Provides a comprehensive sovereign dictionary of 100+ DAMA-DMBOK terms
//! covering all 11 knowledge areas, BahyWay component alignments, and
//! FuzzyDimension ↔ DAMA term mappings.

pub mod term;
pub mod alignment;
pub mod fuzzy_mapping;

pub use term::{
    DmBokTerm,
    KnowledgeArea,
    DICTIONARY,
    lookup,
    by_area,
    search,
};

pub use alignment::{
    BahywayAlignment,
    ALIGNMENTS,
    alignments_for,
    alignments_by_crate,
};

pub use fuzzy_mapping::{
    DimensionMapping,
    DIMENSION_MAPPINGS,
    dimension_to_dama,
    dimensions_for_dama,
};
