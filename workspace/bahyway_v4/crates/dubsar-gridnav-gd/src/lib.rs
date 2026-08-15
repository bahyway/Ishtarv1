//! Grid navigation lens for DubSar Theater. Godot renders the stage
//! (towers as markers, overhead lines and underground cables as
//! layered strands, junctions and converters as glyphs); this bridge
//! owns truth: tiers, horizons, passports, exclusion proofs.

use egd_engine::assets::{AssetRegistry, GridRef};
use egd_engine::{dispatch, exclusion, horizon, state_estimate};
use egd_engine::{Edge, Gibil, Network, Node, Phasor};
use fdd_core::residuals;
use godot::prelude::*;

struct GridNavExtension;

#[gdextension]
unsafe impl ExtensionLibrary for GridNavExtension {}

/// Parses a KAKI identity from a hex string. Unlike a "default to zero
/// on bad input" parser, this hard-fails on malformed input: the whole
/// point of this module is identity-over-appearance disambiguation
/// (GL-EGD-001 clause 3), so a silently-mangled KAKI is worse than no
/// KAKI at all.
fn parse_kaki_hex(hex: &str) -> Option<[u8; 16]> {
    let hex = hex.trim();
    if hex.len() != 32 {
        return None;
    }
    let mut kaki = [0u8; 16];
    for (i, chunk) in hex.as_bytes().chunks(2).enumerate() {
        let s = std::str::from_utf8(chunk).ok()?;
        kaki[i] = u8::from_str_radix(s, 16).ok()?;
    }
    Some(kaki)
}

#[derive(GodotClass)]
#[class(base=RefCounted, init)]
pub struct GridNavLens {
    registry: AssetRegistry,
    /// kappa history per asset index: (t_days, kappa)
    histories: Vec<Vec<(f64, f64)>>,
    threshold: f64,
    /// The physics network: KCL topology + injections, built up via
    /// add_node/add_edge (from a demo fixture today; the real EnkiDB
    /// feed remains the deferred item GL-EGD-001 already names).
    network: Network<Phasor>,
    /// Voltages known by direct measurement (e.g. the feeder head).
    known_voltages: Vec<(usize, Phasor)>,
    /// The last successful state-estimation result -- empty until
    /// `estimate_voltages()` succeeds.
    estimated_phi: Vec<Phasor>,
    base: Base<RefCounted>,
}

#[godot_api]
impl GridNavLens {
    #[func]
    fn set_threshold(&mut self, t: f64) {
        self.threshold = t;
    }

    // ── Physics network (KCL topology) ──────────────────────────────

    /// Adds one node with a known injection (current, in amps-real /
    /// amps-imaginary). Returns the new node's index.
    #[func]
    fn add_node(&mut self, injection_re: f64, injection_im: f64) -> i64 {
        let id = self.network.nodes.len() as u32;
        self.network.nodes.push(Node { id, injection: Phasor::new(injection_re, injection_im) });
        (id) as i64
    }

    /// Adds one edge (admittance `coeff_re + i*coeff_im`) between two
    /// existing node indices. Returns the new edge's index.
    #[func]
    fn add_edge(&mut self, from: i64, to: i64, coeff_re: f64, coeff_im: f64, in_service: bool) -> i64 {
        let idx = self.network.edges.len();
        self.network.edges.push(Edge {
            from: from as u32,
            to: to as u32,
            coeff: Phasor::new(coeff_re, coeff_im),
            in_service,
        });
        idx as i64
    }

    #[func]
    fn set_edge_in_service(&mut self, edge_idx: i64, in_service: bool) {
        if let Some(e) = self.network.edges.get_mut(edge_idx as usize) {
            e.in_service = in_service;
        }
    }

    /// Declares a directly-measured voltage at a node (e.g. the feeder
    /// head). `estimate_voltages()` solves for every other node from
    /// these plus each node's known injection.
    #[func]
    fn set_known_voltage(&mut self, node_idx: i64, re: f64, im: f64) {
        self.known_voltages.push((node_idx as usize, Phasor::new(re, im)));
    }

    #[func]
    fn clear_known_voltages(&mut self) {
        self.known_voltages.clear();
        self.estimated_phi.clear();
    }

    /// Solves the linear KCL system for every node's voltage
    /// (`egd_engine::state_estimate`, exact solve -- see
    /// docs/GL-EGD-001.md for what that does and doesn't cover).
    /// Returns false (and logs why) if the system is singular, e.g. no
    /// known-voltage node reaches some component of the network.
    #[func]
    fn estimate_voltages(&mut self) -> bool {
        match state_estimate::solve_voltages(&self.network, &self.known_voltages) {
            Ok(v) => {
                self.estimated_phi = v;
                true
            }
            Err(e) => {
                godot_error!("GridNavLens.estimate_voltages: {e:?} -- add more known_voltage points or check topology");
                false
            }
        }
    }

    /// The last estimated voltage at a node, as (re, im). Zero vector
    /// if `estimate_voltages()` hasn't succeeded yet or the index is
    /// out of range.
    #[func]
    fn estimated_voltage(&self, node_idx: i64) -> Vector2 {
        match self.estimated_phi.get(node_idx as usize) {
            Some(p) => Vector2::new(p.re as f32, p.im as f32),
            None => Vector2::ZERO,
        }
    }

    /// The KCL residual (kappa) at a node under the last estimated
    /// voltage state. Requires `estimate_voltages()` to have succeeded
    /// with a phi vector covering the whole network; returns -1.0
    /// otherwise (so callers can tell "no defect" from "no estimate").
    #[func]
    fn node_kappa(&self, node_idx: i64) -> f64 {
        if self.estimated_phi.len() != self.network.nodes.len() {
            return -1.0;
        }
        match residuals(&self.network, &self.estimated_phi, &Gibil) {
            Ok(r) => r.get(node_idx as usize).copied().unwrap_or(-1.0),
            Err(_) => -1.0,
        }
    }

    /// N-1 contingency: how many nodes would lose service if this edge
    /// failed right now, under the last estimated voltage state.
    /// 0 = proven safe; -1 = no estimate yet or invalid edge index.
    #[func]
    fn n1_unserved_count(&self, edge_idx: i64, tol: f64) -> i64 {
        if self.estimated_phi.len() != self.network.nodes.len() {
            return -1;
        }
        if edge_idx < 0 || edge_idx as usize >= self.network.edges.len() {
            return -1;
        }
        match exclusion::simulate_exclusion(&self.network, &self.estimated_phi, edge_idx as usize, tol) {
            Ok(exclusion::Verdict::Proven) => 0,
            Ok(exclusion::Verdict::Refused(unserved)) => unserved.len() as i64,
            Err(_) => -1,
        }
    }

    /// Risk-ranked dispatch priority for an asset: combines its Gibil
    /// horizon tier with the N-1 exposure of the edge it maps to (0 if
    /// the asset maps to a node, or if no estimate is available yet).
    #[func]
    fn dispatch_priority_for_asset(&self, asset_idx: i64, tol: f64) -> f64 {
        let Some(asset) = self.registry.assets.get(asset_idx as usize) else { return 0.0 };
        let tier = match self.histories.get(asset_idx as usize) {
            Some(h) => horizon::tier(h, self.threshold),
            None => return 0.0,
        };
        let unserved = match asset.grid_ref {
            GridRef::Edge(idx) => self.n1_unserved_count(idx as i64, tol).max(0) as usize,
            GridRef::Node(_) => 0,
        };
        dispatch::dispatch_priority(tier, unserved)
    }

    /// Loads one asset (called per row from the EnkiDB feed;
    /// GeoEngine-provided lat/lon passthrough). Returns false and logs
    /// a Godot error if `kaki_hex` is not exactly 32 valid hex
    /// characters — the caller must not proceed as if the asset were
    /// registered.
    #[func]
    fn add_asset(
        &mut self,
        kaki_hex: GString,
        kind: i64,
        is_edge: bool,
        grid_index: i64,
        lat: f64,
        lon: f64,
        depth_m: f64,
    ) -> bool {
        use egd_engine::assets::{Asset, AssetKind};
        let Some(kaki) = parse_kaki_hex(&kaki_hex.to_string()) else {
            godot_error!(
                "GridNavLens.add_asset: malformed KAKI hex '{}' (need 32 hex chars) -- asset REJECTED, not registered with a zeroed identity",
                kaki_hex
            );
            return false;
        };
        let kind = match kind {
            0 => AssetKind::Tower,
            1 => AssetKind::OverheadLine,
            2 => AssetKind::UndergroundCable,
            3 => AssetKind::Junction,
            _ => AssetKind::Converter,
        };
        self.registry.assets.push(Asset {
            kaki,
            kind,
            grid_ref: if is_edge { GridRef::Edge(grid_index as usize) } else { GridRef::Node(grid_index as u32) },
            lat,
            lon,
            depth_m,
            passport: vec![],
        });
        self.histories.push(vec![]);
        true
    }

    #[func]
    fn add_passport_field(&mut self, asset_idx: i64, name: GString, value: GString) {
        if let Some(a) = self.registry.assets.get_mut(asset_idx as usize) {
            a.passport.push((name.to_string(), value.to_string()));
        }
    }

    #[func]
    fn push_kappa(&mut self, asset_idx: i64, t_days: f64, kappa: f64) {
        if let Some(h) = self.histories.get_mut(asset_idx as usize) {
            h.push((t_days, kappa));
        }
    }

    /// Stage tier per asset: 0 = Sound (green), 1 = Horizon (amber, see
    /// horizon_days), 2 = Birqu (red).
    #[func]
    fn asset_tier(&self, asset_idx: i64) -> i64 {
        match self.histories.get(asset_idx as usize).map(|h| horizon::tier(h, self.threshold)) {
            Some(horizon::Tier::Birqu) => 2,
            Some(horizon::Tier::Horizon(_)) => 1,
            _ => 0,
        }
    }

    /// Days until predicted defect (-1 when not in window). The number
    /// the maintenance planner reads.
    #[func]
    fn horizon_days(&self, asset_idx: i64) -> f64 {
        match self.histories.get(asset_idx as usize).map(|h| horizon::tier(h, self.threshold)) {
            Some(horizon::Tier::Horizon(d)) => d,
            Some(horizon::Tier::Birqu) => 0.0,
            _ => -1.0,
        }
    }

    /// Identity panel: kaki hex + passport lines. THIS is how the crew
    /// tells two look-alike converters apart.
    #[func]
    fn passport_text(&self, asset_idx: i64) -> GString {
        match self.registry.assets.get(asset_idx as usize) {
            Some(a) => {
                let kaki: String = a.kaki.iter().map(|b| format!("{b:02x}")).collect();
                let mut out =
                    format!("KAKI {kaki}\nkind {:?}\nlat {} lon {} depth {} m\n", a.kind, a.lat, a.lon, a.depth_m);
                for (k, v) in &a.passport {
                    out += &format!("{k}: {v}\n");
                }
                GString::from(out.as_str())
            }
            None => GString::from("unknown asset"),
        }
    }

    #[func]
    fn asset_count(&self) -> i64 {
        self.registry.assets.len() as i64
    }

    #[func]
    fn asset_position(&self, asset_idx: i64) -> Vector3 {
        match self.registry.assets.get(asset_idx as usize) {
            // Stage placement only — real geodesy is GeoEngine's.
            Some(a) => Vector3::new(a.lon as f32, -(a.depth_m as f32), a.lat as f32),
            None => Vector3::ZERO,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::parse_kaki_hex;

    #[test]
    fn sixteen_byte_hex_parses() {
        let hex: String = (0u8..16).map(|b| format!("{b:02x}")).collect();
        assert_eq!(hex.len(), 32);
        let kaki = parse_kaki_hex(&hex).expect("valid hex must parse");
        assert_eq!(kaki, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
    }

    #[test]
    fn wrong_length_hex_is_rejected() {
        assert_eq!(parse_kaki_hex("00112233"), None);
        assert_eq!(parse_kaki_hex(""), None);
    }

    #[test]
    fn non_hex_characters_are_rejected_not_zeroed() {
        let bad = "zz112233445566778899aabbccddeef0";
        // deliberately malformed (contains 'z') -- must be rejected, not
        // silently parsed as a zeroed KAKI.
        assert_eq!(parse_kaki_hex(&bad[..32]), None);
    }
}
