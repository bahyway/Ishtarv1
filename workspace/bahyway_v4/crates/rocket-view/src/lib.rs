//! rocket-view — HS-EXT-003 (The Rocket View): HeptaScript tribe-scope
//! selection and ring-granularity topology. Pure Rust, zero
//! dependencies.
//!
//! Runtime law preserved (§0): all topological quantities are computed
//! by LamassuEngine at design cadence and written as EAV attributes.
//! This crate only READS them -- there is no function here that computes
//! Betti numbers, persistence, or NESU from raw geometry; `TribeAttributes`
//! stands in for what Lamassu has already written.
#![forbid(unsafe_code)]

use std::collections::HashMap;

/// §1.1 — SCOPE, the altitude clause. Five altitudes, matching the
/// camera tiers of the V-2 dolly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    Particle,
    SubRing,
    Tribe,
    Federation,
    Union,
}

/// Each SCOPE tier binds to one camera altitude (§4) -- query-at-altitude
/// = view-at-altitude.
pub fn scope_altitude(scope: Scope) -> u8 {
    match scope {
        Scope::Particle => 0,
        Scope::SubRing => 1,
        Scope::Tribe => 2,
        Scope::Federation => 3,
        Scope::Union => 4,
    }
}

/// §1.2 — IN BIGRING, the ring container noun.
#[derive(Debug, Clone)]
pub struct BigRing {
    pub name: String,
    pub tribes: Vec<String>,
}

pub fn orbit_tribes_in_bigring(ring: &BigRing) -> &[String] {
    &ring.tribes
}

/// §1.3 — CONSTELLATION, cross-tier selection noun.
#[derive(Debug, Clone)]
pub struct Constellation {
    pub members: Vec<String>,
}

/// EAV-backed attributes for one tribe-summary particle, standing in for
/// what LamassuEngine has already written at medium cadence.
#[derive(Debug, Clone, Default)]
pub struct TribeAttributes {
    pub betti: HashMap<u32, u32>,
    pub persistence: HashMap<u32, f64>,
    /// The filtration scale at which this tribe's H0 component finally
    /// merges with the main structure. `None` = never merges within the
    /// tested range -- infinitely isolated.
    pub h0_merge_scale: Option<f64>,
    pub ascent_share: f64,
    pub horizon: Option<f64>,
}

pub fn betti(attrs: &TribeAttributes, k: u32) -> u32 {
    *attrs.betti.get(&k).unwrap_or(&0)
}

pub fn persist(attrs: &TribeAttributes, feature_index: u32) -> f64 {
    *attrs.persistence.get(&feature_index).unwrap_or(&0.0)
}

/// §2 — NESU: isolation is not distance at one moment; it is refusal to
/// merge across all scales. Infinite persistence = never merges =
/// conceptually isolated.
pub fn nesu(attrs: &TribeAttributes) -> f64 {
    attrs.h0_merge_scale.unwrap_or(f64::INFINITY)
}

pub fn ascent_share(attrs: &TribeAttributes) -> f64 {
    attrs.ascent_share
}

pub fn horizon(attrs: &TribeAttributes) -> Option<f64> {
    attrs.horizon
}

/// Q1/Q4 — PROVE NESU(tribe) ABOVE nesu_horizon.
pub fn is_isolated(attrs: &TribeAttributes, nesu_horizon: f64) -> bool {
    nesu(attrs) > nesu_horizon
}

/// Q2 — PROVE BETTI(0) IS 1: the constellation's members form one
/// connected story (caller supplies the union topology's attributes).
pub fn constellation_is_one_story(union_attrs: &TribeAttributes) -> bool {
    betti(union_attrs, 0) == 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scope_altitudes_are_distinct_and_ordered() {
        let scopes = [Scope::Particle, Scope::SubRing, Scope::Tribe, Scope::Federation, Scope::Union];
        let altitudes: Vec<u8> = scopes.iter().map(|&s| scope_altitude(s)).collect();
        for w in altitudes.windows(2) {
            assert!(w[1] > w[0], "altitudes must strictly increase with the rocket's ascent");
        }
    }

    #[test]
    fn in_bigring_selects_exactly_its_tribes() {
        let ring = BigRing { name: "MedicalFederation".into(), tribes: vec!["Pharmacy".into(), "Laboratory".into()] };
        assert_eq!(orbit_tribes_in_bigring(&ring), &["Pharmacy".to_string(), "Laboratory".to_string()]);
    }

    // NESU: a tribe whose H0 component never merges is infinitely isolated.
    #[test]
    fn nesu_never_merges_is_infinite() {
        let attrs = TribeAttributes { h0_merge_scale: None, ..Default::default() };
        assert!(nesu(&attrs).is_infinite());
        assert!(is_isolated(&attrs, 100.0), "an infinitely isolated tribe is isolated at any horizon");
    }

    // NESU: a tribe that merges quickly is not isolated at a typical horizon.
    #[test]
    fn nesu_quick_merge_not_isolated() {
        let attrs = TribeAttributes { h0_merge_scale: Some(0.5), ..Default::default() };
        assert_eq!(nesu(&attrs), 0.5);
        assert!(!is_isolated(&attrs, 5.0));
    }

    // Q2 — constellation coherence: connected (BETTI(0)=1) vs fragmented.
    #[test]
    fn constellation_coherence_check() {
        let mut connected = TribeAttributes::default();
        connected.betti.insert(0, 1);
        assert!(constellation_is_one_story(&connected));

        let mut fragmented = TribeAttributes::default();
        fragmented.betti.insert(0, 3);
        assert!(!constellation_is_one_story(&fragmented), "3 components means the constellation itself is fragmented");
    }

    #[test]
    fn ascent_and_horizon_read_through() {
        let attrs = TribeAttributes { ascent_share: 0.92, horizon: Some(14.0), ..Default::default() };
        assert_eq!(ascent_share(&attrs), 0.92);
        assert_eq!(horizon(&attrs), Some(14.0));
    }
}
