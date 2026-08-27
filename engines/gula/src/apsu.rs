//! APSÛ MIRROR — the underside of the membrane (PB-604).
//! The upper face of the membrane is ontology: which patterns exist.
//! The underside is the Apsû — Enki's freshwater abyss — praxis: what
//! each pattern can DO in the world.
//!
//! When PHANTOM-RECOVERED fires on the upper face, the Apsû reflects it
//! downward as the pattern's derivative along the domain axis: hold the
//! abstract signature fixed, differentiate the Tribe face across concrete
//! carriers. Reflections are APPLICATION CANDIDATES — journaled prose
//! recommendations under GL-LSN-001 §2.4, never sealed by the agent.
//!
//! Also home of ABSENCE CERTIFICATION — the Zaqīqu court run in reverse
//! for humanitarian clearance (mine action, UXO): a suspected hazard is
//! an Unnamed Particle held prune-exempt until recovered (excavated) or
//! its void is certified empty. Certification is boolean on ZERO
//! remaining suspects — the Never-Averaged Theorem. A field that is
//! "99.8% clear" is not clear: no one encounters a percentage.

use crate::Signature;

#[derive(Debug, Clone)]
pub struct ApplicationCandidate {
    /// Upper-face pattern this application reflects.
    pub pattern_key: String,
    /// Concrete carrier domain on the underside.
    pub domain: String,
    /// Sealed engines already able to carry this application.
    pub carrier_engines: Vec<String>,
    /// One-paragraph dossier — recommendation prose (§2.4), never a seal.
    pub dossier: String,
}

/// Tribe face → concrete carrier domains. Deterministic substitution
/// table; extending it is a tablet edit, not a code rewrite.
const TRIBE_CARRIERS: &[(&str, &[&str])] = &[
    (
        "pipe",
        &[
            "water-mains",
            "immersed-rail-tunnel",
            "district-heat-network",
            "estuary-pipeline",
        ],
    ),
    ("membrane", &["tunnel-lining", "dam-face", "levee"]),
    ("vine", &["orchard-irrigation", "greenhouse-lines"]),
];

/// Engines that carry each domain (sealed corpus references).
fn carriers_for(domain: &str) -> Vec<String> {
    let base: &[&str] = match domain {
        "immersed-rail-tunnel" => &["wpdengine", "nanshe", "igigi", "sibu"],
        "water-mains" => &["wpdengine", "igigi"],
        "district-heat-network" | "estuary-pipeline" => &["wpdengine", "nanshe"],
        "tunnel-lining" | "dam-face" | "levee" => &["nanshe", "lamassu"],
        _ => &["nanshe"],
    };
    base.iter().map(|s| s.to_string()).collect()
}

fn dossier_for(sig: &Signature, domain: &str) -> String {
    match domain {
        "immersed-rail-tunnel" => format!(
            "An immersed-tube rail tunnel is a pipe at civilizational scale: a Tribe of \
             segments with joints as vulnerable organs, tidal/thermal loading as the \
             literal {} orbit, and a rupture ceiling per GL-HSI-001-A2. Carriers: \
             Bārûtu wall-as-Masku-organ channels for the lining; Ninety-Day Kanīku \
             horizon for joint-seal prophecy; WPDEngine Cramér–Rao pinpoint gate \
             (σ_d ≤ 0.10 m) for leak localization; Šību for gasket aging; Igigi \
             Three Bells alarming on acceleration, never weather.",
            sig.orbit_face
        ),
        _ => format!(
            "Underside reflection of {} onto carrier '{}': same genome, concrete flesh. \
             Recommendation prose only — the seal is Bahaa's.",
            sig.key(),
            domain
        ),
    }
}

/// The mirror: reflect an upper-face pattern into its Apsû applications.
pub fn reflect(sig: &Signature) -> Vec<ApplicationCandidate> {
    let carriers = TRIBE_CARRIERS
        .iter()
        .find(|(t, _)| *t == sig.tribe_face)
        .map(|(_, d)| *d)
        .unwrap_or(&[]);
    carriers
        .iter()
        .map(|domain| ApplicationCandidate {
            pattern_key: sig.key(),
            domain: (*domain).to_string(),
            carrier_engines: carriers_for(domain),
            dossier: dossier_for(sig, domain),
        })
        .collect()
}

/// ABSENCE CERTIFICATION — the Never-Averaged gate.
/// `suspects_remaining` counts Unnamed Particles still held prune-exempt
/// inside the polygon (suspected hazards neither recovered nor disproven).
/// Certification is granted ONLY at zero. There is no partial certificate,
/// no percentage, no averaging — ever.
pub fn certify_absence(polygon_id: &str, suspects_remaining: usize) -> Result<String, String> {
    if suspects_remaining == 0 {
        Ok(format!(
            "ABSENCE CERTIFIED · polygon {} · zero unnamed particles remain · \
             Kanīku chain may now seal the handover",
            polygon_id
        ))
    } else {
        Err(format!(
            "CERTIFICATION REFUSED · polygon {} · {} unnamed particle(s) still held \
             prune-exempt · Never-Averaged Theorem: a field that is almost clear is \
             not clear — no one encounters a percentage",
            polygon_id, suspects_remaining
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovered_rupture_pipe_reflects_the_tunnel() {
        let sig = Signature::new("rupture", "limit-cycle", "pipe");
        let apps = reflect(&sig);
        assert!(apps.iter().any(|a| a.domain == "immersed-rail-tunnel"));
        let tunnel = apps.iter().find(|a| a.domain == "immersed-rail-tunnel").unwrap();
        assert!(tunnel.carrier_engines.contains(&"wpdengine".to_string()));
        assert!(tunnel.dossier.contains("GL-HSI-001-A2"));
        assert!(tunnel.dossier.contains("limit-cycle"));
    }

    #[test]
    fn unknown_tribe_reflects_nothing_rather_than_inventing() {
        let sig = Signature::new("x", "y", "unknown-tribe");
        assert!(reflect(&sig).is_empty(), "no carrier table entry → no fabricated applications");
    }

    #[test]
    fn never_averaged_refuses_one_remaining_suspect() {
        let refused = certify_absence("MF-ALPHA-07", 1);
        assert!(refused.is_err());
        assert!(refused.unwrap_err().contains("Never-Averaged"));
    }

    #[test]
    fn certification_granted_only_at_zero() {
        assert!(certify_absence("MF-ALPHA-07", 0).is_ok());
        assert!(certify_absence("MF-ALPHA-07", 340).is_err());
    }
}
