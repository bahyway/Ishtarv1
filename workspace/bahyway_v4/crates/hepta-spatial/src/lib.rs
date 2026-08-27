#![forbid(unsafe_code)]
//! hepta — Sovereign Spatial DSL v4.0
//!
//! Hand-written lexer + recursive-descent parser for `.hepta` files.
//! Zero external crate dependencies.  No serde, no pest, no thiserror.
//!
//! # Quick start
//! ```
//! let source = "@HEPTA_VERSION 3.5.1\n@COORDINATE_SYSTEM KAKI_7D\n";
//! let file = hepta_spatial::parse(source).unwrap();
//! assert_eq!(file.sectors.len(), 0);
//! ```

pub mod ast;
pub mod coord;
pub mod error;
pub mod kaki;
pub mod parser;
pub mod token;

// ─── Top-level re-exports ─────────────────────────────────────────────────────

pub use error::HeptaError;
pub use parser::parse;

pub use ast::{
    AnchorBlock, AttribStmt, AttribValue, BeamProperties, BeamStmt, ChordLink, Coord, Distance,
    DistanceUnit, HeptaFile, HeptaHeader, NodeBlock, NodeRef, SectorBlock, SectorGeometry,
};

pub use kaki::{ChordType, CoordSystem, HeptaSector, Kaki7dPk, PlanetNode};

pub use coord::{
    angular_granularity_advantage, assign_sector, chord_length, chord_path_reduction,
    density_from_distance, haversine_m, heptagram_vertex, mean_hepta_advantage, perimeter_segment,
    wgs84_to_kaki7d, HEPTA_STEP_DEG, HEPTA_STEP_RAD,
};
