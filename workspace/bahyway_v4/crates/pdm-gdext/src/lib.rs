//! pdm-gdext — the GDExtension bridge exposing real, already-tested
//! topology/GA math to Godot for DubSar PDM (Particle Data Modeling).
//!
//! Two real math sources, wrapped rather than reimplemented:
//!   - `bahyway_algebra::topology` — the 20 canonical topological
//!     operators (set-theoretic, DE-9IM relational, algebraic topology,
//!     morphological), 115 passing tests as of this writing.
//!   - `najaf_engine::topology` — the simplicial-complex machinery
//!     (barycentric containment, ghost-point reconstruction) built for
//!     Wadi al-Salam grave topology and WPD alleyway inference.
//!
//! Same shape as `kupru-gdext`: one coarse-grained bridge class,
//! stateless methods, Godot-native types only (no long-lived Rust
//! object handles crossing the FFI boundary). A 7D point is always a
//! `PackedFloat64Array` of exactly 7 elements on the Godot side.
//!
//! SCOPE NOTE (deliberate, matching the "start light" plan): this first
//! pass wraps the pure-data functions only. `bahyway_algebra::topology`'s
//! five vector-calculus operators (`exterior_derivative`, `gradient`,
//! `divergence`, `curl`, `laplacian`) take a Rust closure `F: Fn(&HeptaPoint)
//! -> Scalar` as their field — that can't cross the FFI boundary as a
//! plain value. Wrapping those would mean accepting a Godot `Callable`
//! and invoking it once per finite-difference sample, which is a real,
//! separate design decision (per-call FFI overhead × several samples per
//! derivative) deliberately deferred rather than bolted on here.
//! Similarly, `najaf_engine::topology::resolve_stack`/`infer_pipeline`
//! operate on `IdentityKaki` particles, not raw coordinates -- exposing
//! those needs a KAKI-construction bridge that doesn't exist yet, so
//! they're left for a follow-up pass, not silently reimplemented here.

use bahyway_algebra::topology as geo;
use godot::prelude::*;
use najaf_engine::topology::{self as pdm_topo, SimplicialComplex};

type VDict = Dictionary<GString, Variant>;

struct PdmExtension;

#[gdextension]
unsafe impl ExtensionLibrary for PdmExtension {}

#[derive(GodotClass)]
#[class(base=RefCounted, init)]
struct PdmBridge {
    base: Base<RefCounted>,
}

#[godot_api]
impl PdmBridge {
    // ── GROUP 1: Set-theoretic (bahyway_algebra::topology) ──────────

    /// Int(A) — true if `p` is strictly inside the open ball at
    /// `centroid` with the given `radius`.
    #[func]
    fn interior(&self, p: PackedFloat64Array, centroid: PackedFloat64Array, radius: f64) -> VDict {
        match (Self::to_point(&p), Self::to_point(&centroid)) {
            (Some(p), Some(c)) => Self::ok_bool(geo::interior(&p, &c, radius)),
            _ => Self::error_dict("p and centroid must each be a PackedFloat64Array of length 7"),
        }
    }

    /// ∂A — true if `p` is within `epsilon` of the ball's surface.
    #[func]
    fn boundary(&self, p: PackedFloat64Array, centroid: PackedFloat64Array, radius: f64, epsilon: f64) -> VDict {
        match (Self::to_point(&p), Self::to_point(&centroid)) {
            (Some(p), Some(c)) => Self::ok_bool(geo::boundary(&p, &c, radius, epsilon)),
            _ => Self::error_dict("p and centroid must each be a PackedFloat64Array of length 7"),
        }
    }

    /// Cl(A) — true if `p` is inside or on the surface (closed ball).
    #[func]
    fn closure(&self, p: PackedFloat64Array, centroid: PackedFloat64Array, radius: f64) -> VDict {
        match (Self::to_point(&p), Self::to_point(&centroid)) {
            (Some(p), Some(c)) => Self::ok_bool(geo::closure(&p, &c, radius)),
            _ => Self::error_dict("p and centroid must each be a PackedFloat64Array of length 7"),
        }
    }

    /// Ext(A) — true if `p` is outside the closed ball.
    #[func]
    fn exterior(&self, p: PackedFloat64Array, centroid: PackedFloat64Array, radius: f64) -> VDict {
        match (Self::to_point(&p), Self::to_point(&centroid)) {
            (Some(p), Some(c)) => Self::ok_bool(geo::exterior(&p, &c, radius)),
            _ => Self::error_dict("p and centroid must each be a PackedFloat64Array of length 7"),
        }
    }

    // ── GROUP 2: DE-9IM relations between two balls ──────────────────

    #[func]
    fn disjoint(&self, c1: PackedFloat64Array, r1: f64, c2: PackedFloat64Array, r2: f64) -> VDict {
        Self::relate(&c1, r1, &c2, r2, geo::disjoint)
    }

    #[func]
    fn intersects(&self, c1: PackedFloat64Array, r1: f64, c2: PackedFloat64Array, r2: f64) -> VDict {
        Self::relate(&c1, r1, &c2, r2, geo::intersects)
    }

    #[func]
    fn within(&self, c_a: PackedFloat64Array, r_a: f64, c_b: PackedFloat64Array, r_b: f64) -> VDict {
        Self::relate(&c_a, r_a, &c_b, r_b, geo::within)
    }

    #[func]
    fn touches(&self, c1: PackedFloat64Array, r1: f64, c2: PackedFloat64Array, r2: f64) -> VDict {
        Self::relate(&c1, r1, &c2, r2, geo::touches)
    }

    #[func]
    fn overlaps(&self, c1: PackedFloat64Array, r1: f64, c2: PackedFloat64Array, r2: f64) -> VDict {
        Self::relate(&c1, r1, &c2, r2, geo::overlaps)
    }

    // ── GROUP 3: Algebraic topology ───────────────────────────────────

    /// ∂ₙ — the boundary faces of an n-simplex. `vertices` is an Array
    /// of PackedFloat64Array(7); returns "faces" as an Array of Arrays
    /// of PackedFloat64Array(7) (one inner array per face).
    #[func]
    fn boundary_n(&self, vertices: Array<PackedFloat64Array>) -> VDict {
        let pts: Option<Vec<[f64; 7]>> = vertices.iter_shared().map(|v| Self::to_point(&v)).collect();
        let Some(pts) = pts else {
            return Self::error_dict("every vertex must be a PackedFloat64Array of length 7");
        };
        let faces = geo::boundary_n(&pts);
        // Godot's Array<T> forbids nesting anything but Array<Variant> --
        // each face (itself an Array<PackedFloat64Array>) has to be boxed
        // as a Variant before it can live inside the outer array.
        let mut out: Array<Variant> = Array::new();
        for face in faces {
            let mut face_arr: Array<PackedFloat64Array> = Array::new();
            for v in face {
                face_arr.push(&PackedFloat64Array::from(v.as_slice()));
            }
            out.push(&face_arr.to_variant());
        }
        let mut d = VDict::new();
        d.set("ok", true);
        d.set("faces", &out);
        d
    }

    // ── GROUP 4: Morphological operators ──────────────────────────────

    #[func]
    fn dilation(&self, centroid: PackedFloat64Array, radius: f64, se_radius: f64) -> VDict {
        Self::morph(&centroid, radius, se_radius, geo::dilation)
    }

    #[func]
    fn erosion(&self, centroid: PackedFloat64Array, radius: f64, se_radius: f64) -> VDict {
        Self::morph(&centroid, radius, se_radius, geo::erosion)
    }

    #[func]
    fn opening(&self, centroid: PackedFloat64Array, radius: f64, se_radius: f64) -> VDict {
        Self::morph(&centroid, radius, se_radius, geo::opening)
    }

    #[func]
    fn closing(&self, centroid: PackedFloat64Array, radius: f64, se_radius: f64) -> VDict {
        Self::morph(&centroid, radius, se_radius, geo::closing)
    }

    // ── najaf_engine::topology — simplicial complex ───────────────────

    /// Barycentric containment test: is `p` inside the simplex spanned
    /// by `vertices` (2-8 points, up to a 7-simplex)?
    #[func]
    fn particle_in_complex(&self, p: PackedFloat64Array, vertices: Array<PackedFloat64Array>) -> VDict {
        let Some(p) = Self::to_point(&p) else {
            return Self::error_dict("p must be a PackedFloat64Array of length 7");
        };
        let pts: Option<Vec<[f64; 7]>> = vertices.iter_shared().map(|v| Self::to_point(&v)).collect();
        let Some(pts) = pts else {
            return Self::error_dict("every vertex must be a PackedFloat64Array of length 7");
        };
        let complex = SimplicialComplex::new(pts);
        Self::ok_bool(pdm_topo::is_particle_in_complex(&p, &complex))
    }

    /// Reconstruct a missing point's coordinate as the centroid of its
    /// known neighbours -- used for e.g. a grave whose KAKI exists but
    /// whose physical position was never recorded.
    #[func]
    fn reconstruct_ghost(&self, neighbours: Array<PackedFloat64Array>) -> VDict {
        let pts: Option<Vec<[f64; 7]>> = neighbours.iter_shared().map(|v| Self::to_point(&v)).collect();
        let Some(pts) = pts else {
            return Self::error_dict("every neighbour must be a PackedFloat64Array of length 7");
        };
        match pdm_topo::reconstruct_ghost(&pts) {
            Some(ghost) => {
                let mut d = VDict::new();
                d.set("ok", true);
                d.set("ghost", &PackedFloat64Array::from(ghost.as_slice()));
                d
            }
            None => Self::error_dict("neighbours must not be empty"),
        }
    }

    // ── internals ────────────────────────────────────────────────────

    fn to_point(arr: &PackedFloat64Array) -> Option<[f64; 7]> {
        if arr.len() != 7 {
            return None;
        }
        let mut out = [0.0f64; 7];
        for i in 0..7 {
            out[i] = arr.get(i as usize)?;
        }
        Some(out)
    }

    fn relate(
        c1: &PackedFloat64Array,
        r1: f64,
        c2: &PackedFloat64Array,
        r2: f64,
        f: impl Fn(&[f64; 7], f64, &[f64; 7], f64) -> bool,
    ) -> VDict {
        match (Self::to_point(c1), Self::to_point(c2)) {
            (Some(a), Some(b)) => Self::ok_bool(f(&a, r1, &b, r2)),
            _ => Self::error_dict("both centroids must be a PackedFloat64Array of length 7"),
        }
    }

    fn morph(
        centroid: &PackedFloat64Array,
        radius: f64,
        se_radius: f64,
        f: impl Fn([f64; 7], f64, f64) -> ([f64; 7], f64),
    ) -> VDict {
        match Self::to_point(centroid) {
            Some(c) => {
                let (out_c, out_r) = f(c, radius, se_radius);
                let mut d = VDict::new();
                d.set("ok", true);
                d.set("centroid", &PackedFloat64Array::from(out_c.as_slice()));
                d.set("radius", out_r);
                d
            }
            None => Self::error_dict("centroid must be a PackedFloat64Array of length 7"),
        }
    }

    fn ok_bool(value: bool) -> VDict {
        let mut d = VDict::new();
        d.set("ok", true);
        d.set("value", value);
        d
    }

    fn error_dict(msg: &str) -> VDict {
        let mut d = VDict::new();
        d.set("ok", false);
        d.set("error", msg);
        d
    }
}
