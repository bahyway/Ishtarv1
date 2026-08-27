//! damadmbok-dictionary — DAMA-DMBOK data governance vocabulary (§6.5).
//!
//! Provides a comprehensive sovereign dictionary of 100+ DAMA-DMBOK terms
//! covering all 11 knowledge areas, BahyWay component alignments, and
//! FuzzyDimension ↔ DAMA term mappings.

pub mod alignment;
pub mod fuzzy_mapping;
pub mod term;

pub use term::{by_area, lookup, search, DmBokTerm, KnowledgeArea, DICTIONARY};

pub use alignment::{alignments_by_crate, alignments_for, BahywayAlignment, ALIGNMENTS};

pub use fuzzy_mapping::{
    dimension_to_dama, dimensions_for_dama, DimensionMapping, DIMENSION_MAPPINGS,
};
