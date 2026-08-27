//! §A2.8 — the degradation ladder: the full table, pre-computed and
//! pre-sealed, small enough for print, SMS, siren, human relay.
use crate::minimax::Assignment;
use crate::types::Move;

pub fn tablet_line(a: &Assignment, siren: &Option<String>) -> String {
    let d = match &a.mv {
        Move::ToRefuge { refuge_id, bearing } =>
            format!("MOVE {} -> {}", bearing_word(*bearing), refuge_id),
        Move::HoldUnderground { refuge_id } =>
            format!("HOLD UNDERGROUND at {}", refuge_id),
        Move::Egress { bearing } =>
            format!("EGRESS {} — all doors full, clear the shell", bearing_word(*bearing)),
    };
    format!("K-{:03} | {} | {}{}",
        a.zone_id, siren.as_deref().unwrap_or("no siren"), d,
        if a.minimax_flag { " [MINIMAX]" } else { "" })
}
pub fn bearing_word(b: f64) -> &'static str {
    const W: [&str; 8] = ["EAST","SOUTHEAST","SOUTH","SOUTHWEST",
                          "WEST","NORTHWEST","NORTH","NORTHEAST"];
    let t = std::f64::consts::TAU;
    W[(((b % t + t) % t) / (t / 8.0)).round() as usize % 8]
}
