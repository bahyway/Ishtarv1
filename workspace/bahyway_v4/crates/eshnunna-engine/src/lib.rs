//! EshnunnaEngine -- the sovereign columnar data-file engine.
//!
//! Named for Eshnunna (Tell Asmar, Diyala region), home of the Laws of
//! Eshnunna -- one of the earliest codified legal systems, predating
//! Hammurabi by roughly two centuries: a place that organized scattered
//! claims into ordered, retrievable law. This engine does the same for
//! particle column values.
//!
//! ## The problem this solves
//!
//! `enkidb-indexes::SurrogateMap` prunes a query down to a small surrogate
//! set in O(log N) per lookup -- but until now there was no columnar data
//! file to fetch the actual particle *values* from by that surrogate.
//! Retrieval fell back to an O(n) journal walk, which is why HeptaScript
//! queries degraded from ~5 seconds at 100 particles to hours at 10,000:
//! the indexes pruned fast, then retrieval crawled the journal anyway.
//!
//! ## The fix
//!
//! `SurrogateMap::build` assigns surrogate `i` to `kakis[i]` -- dense,
//! sequential, starting at 0. `ColumnWriter` relies on that exact contract:
//! rows are appended in surrogate order, so surrogate `i`'s value always
//! lands at a fixed, computable byte offset. `ColumnReader` then fetches
//! any surrogate's value with one mmap read at that offset -- no scan, no
//! walk, law-like certainty instead of search.
//!
//! Built entirely on existing `enkidb-storage` primitives
//! (`AppendWriter`, `MmapReader`) -- no new dependency, no unsafe code.

#![forbid(unsafe_code)]

pub mod column;

pub use column::{ColumnReader, ColumnWriter, EshnunnaError};
