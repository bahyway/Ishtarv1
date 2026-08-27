//! ETL Flow Panel — live five-tier EnkiDB pipeline visualization.
//!
//! Shows:
//!   EnkiSDB → (valid) → EnkiODB → (valid) → EnkiDW
//!   EnkiDB  → (valid) → EnkiODB
//!   EnkiSDB → (not valid) → EnkiQDB
//!
//! Each tier box carries two color layers:
//!   1. Tribal color ring  — particle's own Color_ID(RGB) from ATTR_COLOR_RGB
//!   2. Stage fill         — fixed ETL state color (Gray/Blue/Green/Gold)

use egui::{Align2, Color32, FontId, Painter, Pos2, Rect, RichText, Rounding, Stroke, Ui, Vec2};

// ── ETL stage colors (dual-layer model) ──────────────────────────────────────

/// Fixed ETL processing stage color (computed by visualizer from pipeline state).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EtlStageColor {
    /// Gray — arrived in SDB, awaiting validation sweep.
    Pending,
    /// Blue — in transit / being processed.
    Processing,
    /// Green — validated and active in ODB.
    Active,
    /// Gold — archived in DW (cold storage).
    Archived,
    /// Red — quarantined (permanent failure).
    Quarantined,
}

impl EtlStageColor {
    pub fn to_color32(self) -> Color32 {
        match self {
            Self::Pending => Color32::from_rgb(120, 120, 120), // Gray
            Self::Processing => Color32::from_rgb(60, 120, 220), // Blue
            Self::Active => Color32::from_rgb(60, 180, 80),    // Green
            Self::Archived => Color32::from_rgb(200, 160, 40), // Gold
            Self::Quarantined => Color32::from_rgb(200, 40, 40), // Red
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Pending => "PENDING",
            Self::Processing => "PROCESSING",
            Self::Active => "ACTIVE",
            Self::Archived => "ARCHIVED",
            Self::Quarantined => "QUARANTINED",
        }
    }
}

// ── Data model ─────────────────────────────────────────────────────────────

/// Live state snapshot for the ETL flow panel.
/// Populated by the runtime from the actual SDB/QDB/ODB stores.
#[derive(Debug, Clone, Default)]
pub struct EtlFlowState {
    // EnkiSDB counters
    pub sdb_pending: usize,
    pub sdb_promoted: usize,
    pub sdb_quarantined: usize,
    /// Representative tribal RGB for the SDB tier.
    pub sdb_tribal_rgb: [u8; 3],
    pub sdb_malware_hits: usize,

    // EnkiQDB counters
    pub qdb_total: usize,
    pub qdb_malware_count: usize,
    /// Representative tribal RGB for the QDB tier.
    pub qdb_tribal_rgb: [u8; 3],

    // EnkiODB counters
    pub odb_total: usize,
    /// Representative tribal RGB for the ODB tier.
    pub odb_tribal_rgb: [u8; 3],

    // EnkiDW counters
    pub dw_total: usize,
    /// Representative tribal RGB for the DW tier.
    pub dw_tribal_rgb: [u8; 3],

    // EnkiDB (GUI transactional) counters
    pub db_total: usize,
    /// Representative tribal RGB for the DB tier.
    pub db_tribal_rgb: [u8; 3],

    // Diagnosis stats (from DiagnosisEngine last run)
    pub diag_watch: usize,
    pub diag_warning: usize,
    pub diag_critical: usize,

    // Scheduler tick info
    pub current_tick: u64,
    pub sweep_interval: u64,
    pub ticks_until_sweep: u64,
}

impl EtlFlowState {
    pub fn demo() -> Self {
        EtlFlowState {
            sdb_pending: 42,
            sdb_promoted: 180,
            sdb_quarantined: 8,
            sdb_tribal_rgb: [100, 180, 220],
            sdb_malware_hits: 2,
            qdb_total: 8,
            qdb_malware_count: 2,
            qdb_tribal_rgb: [200, 60, 60],
            odb_total: 180,
            odb_tribal_rgb: [80, 200, 120],
            dw_total: 520,
            dw_tribal_rgb: [200, 160, 40],
            db_total: 15,
            db_tribal_rgb: [120, 120, 220],
            diag_watch: 7,
            diag_warning: 3,
            diag_critical: 1,
            current_tick: 4320,
            sweep_interval: 900,
            ticks_until_sweep: 213,
        }
    }
}

// ── Panel entry point ─────────────────────────────────────────────────────────

pub fn draw(ui: &mut Ui, state: &EtlFlowState) {
    ui.heading(RichText::new("EnkiDB Five-Tier ETL Flow").color(Color32::from_rgb(220, 190, 100)));
    ui.add_space(4.0);

    // ── Scheduler tick bar ────────────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.label(RichText::new("⏱ Tick:").color(Color32::GRAY));
        ui.label(RichText::new(format!("{}", state.current_tick)).color(Color32::WHITE));
        ui.separator();
        ui.label(RichText::new("Sweep in:").color(Color32::GRAY));
        let remaining = state.ticks_until_sweep;
        let color = if remaining < 100 {
            Color32::YELLOW
        } else {
            Color32::WHITE
        };
        ui.label(RichText::new(format!("{remaining} ticks")).color(color));
        ui.separator();
        let ratio = if state.sweep_interval > 0 {
            1.0 - (remaining as f32 / state.sweep_interval as f32)
        } else {
            0.0
        };
        let filled = (ratio * 120.0) as usize;
        let bar = format!(
            "[{}{}]",
            "█".repeat(filled.min(12)),
            "░".repeat(12 - filled.min(12))
        );
        ui.label(RichText::new(bar).color(Color32::from_rgb(80, 160, 255)));
    });

    ui.add_space(8.0);
    ui.separator();
    ui.add_space(8.0);

    // ── Tier boxes in a horizontal flow ──────────────────────────────────
    let available = ui.available_size();
    let panel_height = available.y.min(420.0);
    let (response, painter) =
        ui.allocate_painter(Vec2::new(available.x, panel_height), egui::Sense::hover());

    let r = response.rect;
    painter.rect_filled(r, Rounding::same(6.0), Color32::from_rgb(18, 18, 26));

    // Layout constants
    let box_w: f32 = 140.0;
    let box_h: f32 = 110.0;
    let margin: f32 = 24.0;
    let top_y: f32 = r.top() + 40.0;
    let bot_y: f32 = r.top() + 220.0;
    let sdb_x = r.left() + margin;

    // Tier box positions (center x, center y)
    let sdb_cx = sdb_x + box_w / 2.0;
    let sdb_cy = top_y + box_h / 2.0;

    let odb_cx = sdb_cx + box_w + 80.0;
    let odb_cy = sdb_cy;

    let dw_cx = odb_cx + box_w + 80.0;
    let dw_cy = sdb_cy;

    let qdb_cx = sdb_cx;
    let qdb_cy = bot_y + box_h / 2.0;

    let db_cx = odb_cx + box_w + 80.0;
    let db_cy = bot_y + box_h / 2.0;

    // ── Draw tiers ────────────────────────────────────────────────────────
    draw_tier(
        &painter,
        sdb_cx,
        sdb_cy,
        box_w,
        box_h,
        "EnkiSDB",
        "Stage DB",
        &format!(
            "Pending: {}\nPromoted: {}\nQuarantined: {}\nMalware hits: {}",
            state.sdb_pending, state.sdb_promoted, state.sdb_quarantined, state.sdb_malware_hits
        ),
        EtlStageColor::Pending,
        state.sdb_tribal_rgb,
    );

    draw_tier(
        &painter,
        odb_cx,
        odb_cy,
        box_w,
        box_h,
        "EnkiODB",
        "Operational DB",
        &format!("Active: {}", state.odb_total),
        EtlStageColor::Active,
        state.odb_tribal_rgb,
    );

    draw_tier(
        &painter,
        dw_cx,
        dw_cy,
        box_w,
        box_h,
        "EnkiDW",
        "Datawarehouse",
        &format!("Archived: {}", state.dw_total),
        EtlStageColor::Archived,
        state.dw_tribal_rgb,
    );

    draw_tier(
        &painter,
        qdb_cx,
        qdb_cy,
        box_w,
        box_h,
        "EnkiQDB",
        "Quarantine",
        &format!(
            "Quarantined: {}\nMalware: {}",
            state.qdb_total, state.qdb_malware_count
        ),
        EtlStageColor::Quarantined,
        state.qdb_tribal_rgb,
    );

    draw_tier(
        &painter,
        db_cx,
        db_cy,
        box_w,
        box_h,
        "EnkiDB",
        "Transactional",
        &format!("GUI commits: {}", state.db_total),
        EtlStageColor::Processing,
        state.db_tribal_rgb,
    );

    // ── Draw arrows ───────────────────────────────────────────────────────
    // SDB → ODB (valid)
    draw_arrow(
        &painter,
        Pos2::new(sdb_cx + box_w / 2.0, sdb_cy),
        Pos2::new(odb_cx - box_w / 2.0, odb_cy),
        Color32::from_rgb(80, 220, 80),
        "valid",
    );
    // ODB → DW (valid)
    draw_arrow(
        &painter,
        Pos2::new(odb_cx + box_w / 2.0, odb_cy),
        Pos2::new(dw_cx - box_w / 2.0, dw_cy),
        Color32::from_rgb(200, 160, 40),
        "archive",
    );
    // SDB → QDB (invalid)
    draw_arrow(
        &painter,
        Pos2::new(sdb_cx, sdb_cy + box_h / 2.0),
        Pos2::new(qdb_cx, qdb_cy - box_h / 2.0),
        Color32::from_rgb(200, 60, 60),
        "quarantine",
    );
    // DB → ODB (valid)
    draw_arrow(
        &painter,
        Pos2::new(db_cx - box_w / 2.0, db_cy),
        Pos2::new(odb_cx + box_w / 2.0, odb_cy + 20.0),
        Color32::from_rgb(80, 160, 255),
        "commit",
    );

    // ── Diagnosis summary row ─────────────────────────────────────────────
    let diag_y = qdb_cy + box_h / 2.0 + 16.0;
    painter.text(
        Pos2::new(r.left() + margin, diag_y),
        Align2::LEFT_TOP,
        format!(
            "Diagnosis — Watch: {}  Warning: {}  Critical: {}",
            state.diag_watch, state.diag_warning, state.diag_critical
        ),
        FontId::proportional(13.0),
        if state.diag_critical > 0 {
            Color32::from_rgb(255, 80, 80)
        } else if state.diag_warning > 0 {
            Color32::YELLOW
        } else {
            Color32::from_rgb(160, 220, 255)
        },
    );

    // ── Legend ────────────────────────────────────────────────────────────
    let legend_x = r.right() - 200.0;
    let legend_y = r.top() + 10.0;
    painter.text(
        Pos2::new(legend_x, legend_y),
        Align2::LEFT_TOP,
        "■ Stage color  ○ Tribal color",
        FontId::proportional(11.0),
        Color32::GRAY,
    );
}

// ── Drawing helpers ───────────────────────────────────────────────────────────

// Each parameter is a distinct rendering input (geometry, labels, stage
// color, and tribal color) -- not artificial padding, so bundling into a
// params struct is a real API-design tradeoff rather than a mechanical fix.
#[allow(clippy::too_many_arguments)]
fn draw_tier(
    painter: &Painter,
    cx: f32,
    cy: f32,
    w: f32,
    h: f32,
    title: &str,
    subtitle: &str,
    body: &str,
    stage: EtlStageColor,
    tribal_rgb: [u8; 3],
) {
    let rect = Rect::from_center_size(Pos2::new(cx, cy), Vec2::new(w, h));

    // Background fill with stage color (dimmed)
    let stage_col = stage.to_color32();
    let bg = Color32::from_rgba_premultiplied(
        stage_col.r() / 5,
        stage_col.g() / 5,
        stage_col.b() / 5,
        240,
    );
    painter.rect_filled(rect, Rounding::same(8.0), bg);

    // Tier border — stage color
    painter.rect_stroke(rect, Rounding::same(8.0), Stroke::new(2.0_f32, stage_col));

    // Tribal color dot (top-right corner) — Color_ID(RGB) layer
    let tribal_dot = Pos2::new(rect.right() - 10.0, rect.top() + 10.0);
    painter.circle_filled(
        tribal_dot,
        6.0,
        Color32::from_rgb(tribal_rgb[0], tribal_rgb[1], tribal_rgb[2]),
    );
    painter.circle_stroke(tribal_dot, 6.0, Stroke::new(1.0_f32, Color32::WHITE));

    // Title
    painter.text(
        Pos2::new(rect.left() + 8.0, rect.top() + 8.0),
        Align2::LEFT_TOP,
        title,
        FontId::proportional(14.0),
        Color32::WHITE,
    );

    // Subtitle / tier type
    painter.text(
        Pos2::new(rect.left() + 8.0, rect.top() + 26.0),
        Align2::LEFT_TOP,
        subtitle,
        FontId::proportional(10.0),
        Color32::GRAY,
    );

    // Stage label (colored)
    painter.text(
        Pos2::new(rect.left() + 8.0, rect.top() + 40.0),
        Align2::LEFT_TOP,
        stage.label(),
        FontId::proportional(10.0),
        stage_col,
    );

    // Body stats
    let mut line_y = rect.top() + 56.0;
    for line in body.lines() {
        painter.text(
            Pos2::new(rect.left() + 8.0, line_y),
            Align2::LEFT_TOP,
            line,
            FontId::proportional(11.0),
            Color32::from_rgb(200, 200, 200),
        );
        line_y += 14.0;
    }
}

fn draw_arrow(painter: &Painter, from: Pos2, to: Pos2, color: Color32, label: &str) {
    painter.line_segment([from, to], Stroke::new(2.0_f32, color));

    // Arrow head
    let dir = (to - from).normalized();
    let perp = Vec2::new(-dir.y, dir.x);
    let tip = to;
    let base1 = tip - dir * 10.0 + perp * 5.0;
    let base2 = tip - dir * 10.0 - perp * 5.0;
    painter.add(egui::Shape::convex_polygon(
        vec![tip, base1, base2],
        color,
        Stroke::NONE,
    ));

    // Label at midpoint
    let mid = from + (to - from) * 0.5;
    painter.text(
        Pos2::new(mid.x + 4.0, mid.y - 12.0),
        Align2::LEFT_TOP,
        label,
        FontId::proportional(10.0),
        color,
    );
}

// ── Tests (pure data, no egui context needed) ─────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_colors_distinct() {
        let colors: Vec<Color32> = [
            EtlStageColor::Pending,
            EtlStageColor::Processing,
            EtlStageColor::Active,
            EtlStageColor::Archived,
            EtlStageColor::Quarantined,
        ]
        .iter()
        .map(|s| s.to_color32())
        .collect();

        // All five stage colors must be distinct
        for i in 0..colors.len() {
            for j in (i + 1)..colors.len() {
                assert_ne!(colors[i], colors[j], "stage colors {i} and {j} must differ");
            }
        }
    }

    #[test]
    fn demo_state_has_non_zero_counts() {
        let s = EtlFlowState::demo();
        assert!(s.sdb_pending > 0);
        assert!(s.odb_total > 0);
        assert!(s.dw_total > 0);
        assert!(s.qdb_total > 0);
    }

    #[test]
    fn etl_stage_labels_non_empty() {
        for stage in [
            EtlStageColor::Pending,
            EtlStageColor::Processing,
            EtlStageColor::Active,
            EtlStageColor::Archived,
            EtlStageColor::Quarantined,
        ] {
            assert!(!stage.label().is_empty());
        }
    }
}
