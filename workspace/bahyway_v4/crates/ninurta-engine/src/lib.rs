//! ninurta-engine — critical-slowing-down / bifurcation detection over
//! particle-collection time series.
//!
//! Per the Topological Engine Division (GL-TED-001): three engines read
//! one substrate. `bahyway-algebra` holds the mathematics (geometric
//! algebra, simplicial complexes, persistent homology -- computed once).
//! `lamassu-engine` reads a STATIC complex for structure (beta_0/beta_1/
//! beta_2 as entities/relational loops/voids). This crate reads a
//! complex (or any scalar observable) IN MOTION for transition -- via
//! the Purussum Calculus: the restoring rate lambda, corroborating
//! indicators (variance, lag-1 autocorrelation), Fourier-surrogate
//! significance testing, and a beta_1-tightening cross-check against
//! `lamassu-engine`'s own readings where a BIGRING's dual alarm applies.
//!
//! Ninurta owns transition; it does not reimplement structure discovery
//! (that stays LamassuEngine's) or the underlying math (that stays
//! `bahyway-algebra`'s). Ninurta says "the shape approaches a fork,
//! confidence tau" -- never "transition at tick T." That humility is
//! load-bearing, not decoration: see `purussum::PurussumVerdict`.
#![forbid(unsafe_code)]

pub mod detrend;
pub mod indicators;
pub mod purussum;
pub mod surrogate;

pub use purussum::{render_verdict, trichotomy_for_lambda, LambdaTrichotomy, PurussumVerdict, SIGNIFICANCE_THRESHOLD};
