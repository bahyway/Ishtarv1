//! §A2.2 — D*(z) = argmin_m max_Θ D_Θ(m), s.t. no full doors (§A2.4).
//! §A2.3 — LIVES ARE NEVER AVERAGED: no blend path exists in this module.
use crate::fadam::{danger_at, danger_path};
use crate::types::*;

pub struct Assignment {
    pub zone_id: u16,
    pub mv: Move,
    pub worst: f64,
    pub minimax_flag: bool, // templates materially disagreed
}

/// Solve directives for all zones. `declared`: Some(Θ) when a siren has
/// declared a template (the max collapses to a singleton — the siren
/// narrows, it never invents); None = standing minimax over all of S.
pub fn solve(
    zones: &[Zone], refuges: &[Refuge], sealed: &[ThreatTemplate],
    declared: Option<&ThreatTemplate>,
) -> Result<Vec<Assignment>, &'static str> {
    // F5: only sealed templates judge.
    if sealed.iter().any(|t| !t.sealed) {
        return Err("F5: unsealed template in S — drafts may rehearse, never judge");
    }
    if let Some(t) = declared {
        if !t.sealed { return Err("F5: declared siren references an unsealed template"); }
    }
    let scen: Vec<&ThreatTemplate> = match declared {
        Some(t) => vec![t],
        None => sealed.iter().collect(),
    };
    // most endangered claim doors first (§A2.2 assignment order)
    let mut order: Vec<&Zone> = zones.iter().collect();
    order.sort_by(|a, b| {
        let wa = scen.iter().map(|t| danger_at(a.cx, a.cy, t, a.eps))
            .fold(0.0f64, f64::max);
        let wb = scen.iter().map(|t| danger_at(b.cx, b.cy, t, b.eps))
            .fold(0.0f64, f64::max);
        wb.partial_cmp(&wa).unwrap()
    });
    let mut load: std::collections::HashMap<&str, u64> =
        refuges.iter().map(|r| (r.id.as_str(), 0u64)).collect();
    let mut out = Vec::with_capacity(zones.len());
    for z in order {
        let mut best: Option<(f64, f64, &Refuge, f64, f64)> = None; // (cost, worst, r, wmin, wmax)
        for r in refuges {
            // §A2.4 capacity honesty: NO FULL DOORS.
            if load[r.id.as_str()] + z.pop as u64 > r.cap as u64 { continue; }
            let mut wmax = 0.0f64;
            let mut wmin = f64::MAX;
            for t in &scen {
                let w = danger_path((z.cx, z.cy), (r.x, r.y), r.shield, t, z.eps);
                if w > wmax { wmax = w; }
                if w < wmin { wmin = w; }
            }
            let dist = ((r.x - z.cx).powi(2) + (r.y - z.cy).powi(2)).sqrt();
            let cost = wmax * 2.0 + dist / 650.0; // lexicographic tie-break by distance
            if best.map_or(true, |(c, ..)| cost < c) {
                best = Some((cost, wmax, r, wmin, wmax));
            }
        }
        let asg = match best {
            Some((_, worst, r, wmin, wmax)) => {
                *load.get_mut(r.id.as_str()).unwrap() += z.pop as u64;
                let bearing = (r.y - z.cy).atan2(r.x - z.cx);
                let hold = r.shield < 1.0 && worst > 0.45 + z.eps.get();
                Assignment {
                    zone_id: z.id,
                    mv: if hold {
                        Move::HoldUnderground { refuge_id: r.id.clone() }
                    } else {
                        Move::ToRefuge { refuge_id: r.id.clone(), bearing }
                    },
                    worst,
                    minimax_flag: declared.is_none() && (wmax - wmin) > 0.15,
                }
            }
            // every door full → lawful degradation to egress, never a lie
            None => Assignment {
                zone_id: z.id,
                mv: Move::Egress { bearing: (z.cy - 350.0).atan2(z.cx - 380.0) },
                worst: 1.0 + z.eps.get(),
                minimax_flag: true,
            },
        };
        out.push(asg);
    }
    out.sort_by_key(|a| a.zone_id);
    Ok(out)
}
