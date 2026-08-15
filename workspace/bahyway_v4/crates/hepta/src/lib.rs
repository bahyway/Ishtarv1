#![forbid(unsafe_code)]
//! hepta — Sovereign Spatial DSL v4.0
//!
//! Hand-written lexer + recursive-descent parser for `.hepta` files.
//! Zero external crate dependencies.  No serde, no pest, no thiserror.
//!
//! # Quick start
//! ```
//! let source = "@HEPTA_VERSION 3.5.1\n@COORDINATE_SYSTEM KAKI_7D\n";
//! let file = hepta::parse(source).unwrap();
//! assert_eq!(file.sectors.len(), 0);
//! ```

pub mod error;
pub mod kaki;
pub mod ast;
pub mod token;
pub mod parser;
pub mod coord;

// ─── Top-level re-exports ─────────────────────────────────────────────────────

pub use error::HeptaError;
pub use parser::parse;

pub use ast::{
    HeptaFile, HeptaHeader,
    AnchorBlock, SectorBlock, NodeBlock,
    BeamStmt, BeamProperties,
    AttribStmt, AttribValue,
    NodeRef, Coord, Distance, DistanceUnit,
    SectorGeometry, ChordLink,
};

pub use kaki::{
    PlanetNode, HeptaSector, CoordSystem, ChordType, Kaki7dPk,
};

pub use coord::{
    HEPTA_STEP_DEG, HEPTA_STEP_RAD,
    heptagram_vertex,
    chord_length, perimeter_segment,
    angular_granularity_advantage,
    chord_path_reduction, mean_hepta_advantage,
    assign_sector, haversine_m,
    density_from_distance, wgs84_to_kaki7d,
};
