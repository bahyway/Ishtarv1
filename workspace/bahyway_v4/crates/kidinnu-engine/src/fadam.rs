//! Every danger functional D_Θ is a Fadam functional (GL-ALG-001-A1):
//! F1 anonymity (reads positions, never identities), F2 rotation handled
//! by bearing-relative geometry, F3 Lipschitz by bounded gradients of the
//! soft front, F4 epsilon added ALWAYS, F5 sealed-judge checked by caller.
use crate::types::{EpsilonFloor, ThreatTemplate};

/// Danger of a point under a sealed template, F4-floored.
pub fn danger_at(x: f64, y: f64, t: &ThreatTemplate, eps: EpsilonFloor) -> f64 {
    let dx = x - t.src_x;
    let dy = y - t.src_y;
    let d = (dx * dx + dy * dy).sqrt() + 1.0;
    let align = ((dy.atan2(dx)) - (t.bearing + std::f64::consts::PI)).cos();
    let raw = (1.0 - d / (150.0 * t.spread)).max(0.0)
        * (0.5 + 0.5 * align.max(0.0));
    raw + eps.get() // F4: never below the confessed floor — including at Θ itself
}

/// Danger of a straight walk, sampled at midpoint and endpoint (path law).
pub fn danger_path(
    from: (f64, f64), to: (f64, f64), shield: f64,
    t: &ThreatTemplate, eps: EpsilonFloor,
) -> f64 {
    let mid = ((from.0 + to.0) / 2.0, (from.1 + to.1) / 2.0);
    let a = danger_at(mid.0, mid.1, t, eps);
    let b = danger_at(to.0, to.1, t, eps) * shield;
    a.max(b)
}
