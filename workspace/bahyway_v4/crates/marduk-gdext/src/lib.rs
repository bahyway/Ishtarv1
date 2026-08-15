//! MardukEngine bridge for DubSar Theater's Marduk panel. Same shape
//! as `pdm-gdext` and `kupru-gdext`: one coarse-grained bridge class,
//! stateless methods, Godot-native types only. Wraps exactly the three
//! verbs `marduk-engine` implements (Position, Horizon, Topology) --
//! Motion and Curvature aren't here because they aren't built yet
//! (see docs/marduk/IMPL-MRD-001-position-horizon-first-slice.md), not
//! because this bridge forgot them.
//!
//! A Hepta Space point is always a `PackedFloat64Array` of exactly 7
//! elements, matching the convention `pdm-gdext` already established
//! for 7D points crossing the FFI boundary.

use godot::prelude::*;
use marduk_engine::topology::RelationGraph;

struct MardukExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MardukExtension {}

#[derive(GodotClass)]
#[class(base=RefCounted, init)]
pub struct MardukBridge {
    base: Base<RefCounted>,
}

/// Pure-Rust core of the 7-element validation, kept independent of any
/// Godot type so it's testable without a live engine (PackedFloat64Array
/// can't be constructed outside one -- see mod tests below for why this
/// split exists).
fn to_seven_slice(s: &[f64]) -> Option<[f64; 7]> {
    if s.len() != 7 {
        return None;
    }
    Some([s[0], s[1], s[2], s[3], s[4], s[5], s[6]])
}

fn to_seven(a: &PackedFloat64Array, label: &str) -> Option<[f64; 7]> {
    match to_seven_slice(a.as_slice()) {
        Some(x) => Some(x),
        None => {
            godot_error!("MardukBridge: {label} must be exactly 7 elements, got {}", a.len());
            None
        }
    }
}

#[godot_api]
impl MardukBridge {
    // ── Position verb ────────────────────────────────────────────────

    /// H(P) = 1/(1+r): 1.0 at the template, falling with distance.
    /// Returns -1.0 (impossible for a real H(P) value) and logs an
    /// error if p/t/w aren't each exactly 7 elements.
    #[func]
    fn template_score(&self, p: PackedFloat64Array, t: PackedFloat64Array, w: PackedFloat64Array) -> f64 {
        let (Some(p), Some(t), Some(w)) =
            (to_seven(&p, "p"), to_seven(&t, "t"), to_seven(&w, "w"))
        else {
            return -1.0;
        };
        marduk_engine::template_score(&p, &t, &w)
    }

    /// The Hepta Space radial coordinate r (geodesic distance to the
    /// template under the flat metric g = diag(w)). Returns -1.0 (an
    /// impossible distance) and logs an error on malformed input.
    #[func]
    fn radial_coordinate(&self, p: PackedFloat64Array, t: PackedFloat64Array, w: PackedFloat64Array) -> f64 {
        let (Some(p), Some(t), Some(w)) =
            (to_seven(&p, "p"), to_seven(&t, "t"), to_seven(&w, "w"))
        else {
            return -1.0;
        };
        marduk_engine::radial_coordinate(&p, &t, &w)
    }

    // ── Horizon verb ─────────────────────────────────────────────────

    /// Predicted days until a particle's B11 trend crosses the GEM
    /// floor (200), from an interleaved [t0, v0, t1, v1, ...] history.
    ///   >= 0.0  -> predicted crossing in that many days (0.0 = now)
    ///   -1.0    -> stable/improving; never crosses
    ///   -2.0    -> malformed input (odd length, or fewer than 2 samples)
    #[func]
    fn golden_horizon_days(&self, history_interleaved: PackedFloat64Array) -> f64 {
        if history_interleaved.len() % 2 != 0 {
            godot_error!("MardukBridge.golden_horizon_days: interleaved history must have an even length");
            return -2.0;
        }
        let slice = history_interleaved.as_slice();
        let history: Vec<(f64, f64)> = slice.chunks(2).map(|c| (c[0], c[1])).collect();
        match marduk_engine::golden_horizon(&history) {
            Ok(Some(d)) => d,
            Ok(None) => -1.0,
            Err(_) => -2.0,
        }
    }

    // ── Topology verb ────────────────────────────────────────────────

    /// (beta0, beta1) of a relation graph: `node_count` nodes, edges as
    /// an interleaved [a0, b0, a1, b1, ...] index array. beta0 =
    /// connected components; beta1 = independent loop count (the
    /// graph-theoretic signature of a mule ring / illegal-connection
    /// ring). Returns (-1, -1) on malformed input (odd-length edge
    /// array, or a node index >= node_count).
    #[func]
    fn betti_numbers(&self, node_count: i64, edges_interleaved: PackedInt64Array) -> Vector2i {
        if edges_interleaved.len() % 2 != 0 {
            godot_error!("MardukBridge.betti_numbers: interleaved edge array must have an even length");
            return Vector2i::new(-1, -1);
        }
        if node_count < 0 {
            godot_error!("MardukBridge.betti_numbers: node_count must be >= 0");
            return Vector2i::new(-1, -1);
        }
        let n = node_count as usize;
        let slice = edges_interleaved.as_slice();
        let mut edges = Vec::with_capacity(slice.len() / 2);
        for pair in slice.chunks(2) {
            let (a, b) = (pair[0], pair[1]);
            if a < 0 || b < 0 || a as usize >= n || b as usize >= n {
                godot_error!("MardukBridge.betti_numbers: edge ({a}, {b}) out of range for node_count={node_count}");
                return Vector2i::new(-1, -1);
            }
            edges.push((a as usize, b as usize));
        }
        let g = RelationGraph { node_count: n, edges };
        Vector2i::new(marduk_engine::betti_0(&g) as i32, marduk_engine::betti_1(&g) as i32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_seven_slice_accepts_exactly_seven_elements() {
        let s = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        assert_eq!(to_seven_slice(&s), Some([0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0]));
    }

    #[test]
    fn to_seven_slice_rejects_wrong_length() {
        assert_eq!(to_seven_slice(&[1.0]), None);
        assert_eq!(to_seven_slice(&[]), None);
        assert_eq!(to_seven_slice(&[0.0; 8]), None);
    }
}
