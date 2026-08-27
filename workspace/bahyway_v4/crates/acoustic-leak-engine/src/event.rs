//! LeakAlertEvent -- the ONLY output of acoustic-leak-engine.
//!
//! LAW-ALE-4: this engine never mints KAKI. This event is serialized and
//! delivered to the real enkidb-ingest bridge, the sole lawful minting
//! site, where it becomes a KAKI particle whose mutable qualities live
//! entirely in EAV Mandatory Attributes (position, sigma_d, stage,
//! material, v, posterior).

use crate::material::PipeMaterial;
use crate::verdict::{LeakSite, Stage};

#[derive(Debug, Clone)]
pub struct EavAttr {
    pub attribute: &'static str,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct LeakAlertEvent {
    pub tribe_id: u16,
    pub segment_ref: String,
    pub stage: Stage,
    pub attrs: Vec<EavAttr>,
}

#[allow(clippy::too_many_arguments)]
pub fn build_alert(
    tribe_id: u16,
    segment_ref: &str,
    stage: Stage,
    material: PipeMaterial,
    v_mps: f64,
    sigma_v_rel: f64,
    position_m: f64,
    sigma_d_m: f64,
    site: LeakSite,
    posterior: f64,
) -> LeakAlertEvent {
    LeakAlertEvent {
        tribe_id,
        segment_ref: segment_ref.to_string(),
        stage,
        attrs: vec![
            EavAttr {
                attribute: "ale.material",
                value: format!("{material:?}"),
            },
            EavAttr {
                attribute: "ale.v_mps",
                value: format!("{v_mps:.3}"),
            },
            EavAttr {
                attribute: "ale.sigma_v_rel",
                value: format!("{sigma_v_rel:.6}"),
            },
            EavAttr {
                attribute: "ale.position_m",
                value: format!("{position_m:.3}"),
            },
            EavAttr {
                attribute: "ale.sigma_d_m",
                value: format!("{sigma_d_m:.3}"),
            },
            EavAttr {
                attribute: "ale.site",
                value: format!("{site:?}"),
            },
            EavAttr {
                attribute: "ale.posterior",
                value: format!("{posterior:.4}"),
            },
        ],
    }
}

/// Wire form for the real enkidb-ingest bridge (line-oriented, stdlib only).
pub fn to_bridge_wire(ev: &LeakAlertEvent) -> String {
    let mut s = format!(
        "ALE_ALERT tribe={} segment={} stage={:?}\n",
        ev.tribe_id, ev.segment_ref, ev.stage
    );
    for a in &ev.attrs {
        s.push_str(&format!("EAV {}={}\n", a.attribute, a.value));
    }
    s.push_str("END\n");
    s
}
