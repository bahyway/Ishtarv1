use egui::{Color32, FontFamily, FontId, Margin, Rounding, Style, TextStyle, Visuals};

// ── EnkiDB Manifold Dark palette ─────────────────────────────────────────────
// bg=#060610  gold=#FFD700  cyan=#00FFFF  green=#00FF88  violet=#CC44FF  orange=#FF8C00

pub const BG_DARK:   Color32 = Color32::from_rgb(6, 6, 16);
pub const BG_MID:    Color32 = Color32::from_rgb(10, 10, 28);
pub const BG_CARD:   Color32 = Color32::from_rgb(16, 16, 42);
pub const BG_HOVER:  Color32 = Color32::from_rgb(22, 22, 55);
pub const GOLD:      Color32 = Color32::from_rgb(255, 215, 0);
pub const CYAN:      Color32 = Color32::from_rgb(0, 255, 255);
pub const GREEN:     Color32 = Color32::from_rgb(0, 255, 136);
#[allow(dead_code)]
pub const VIOLET:    Color32 = Color32::from_rgb(204, 68, 255);
pub const ORANGE:    Color32 = Color32::from_rgb(255, 140, 0);
pub const RED_HOT:   Color32 = Color32::from_rgb(255, 60, 60);
pub const TEXT_DIM:  Color32 = Color32::from_rgb(130, 130, 155);
pub const TEXT_MAIN: Color32 = Color32::from_rgb(200, 200, 220);
pub const AXIS_LINE: Color32 = Color32::from_rgba_premultiplied(55, 55, 90, 180);
pub const RING_LINE: Color32 = Color32::from_rgba_premultiplied(40, 40, 75, 140);

pub fn apply(ctx: &egui::Context) {
    let mut style = Style::default();

    style.text_styles = [
        (TextStyle::Small,    FontId::new(11.0, FontFamily::Proportional)),
        (TextStyle::Body,     FontId::new(13.0, FontFamily::Proportional)),
        (TextStyle::Button,   FontId::new(13.0, FontFamily::Proportional)),
        (TextStyle::Heading,  FontId::new(17.0, FontFamily::Proportional)),
        (TextStyle::Monospace, FontId::new(12.0, FontFamily::Monospace)),
    ].into();

    style.spacing.item_spacing = egui::vec2(6.0, 4.0);
    style.spacing.button_padding = egui::vec2(8.0, 4.0);

    let mut vis = Visuals::dark();
    vis.window_fill                        = BG_DARK;
    vis.panel_fill                         = BG_DARK;
    vis.override_text_color                = Some(TEXT_MAIN);
    vis.extreme_bg_color                   = BG_MID;
    vis.code_bg_color                      = BG_CARD;
    vis.faint_bg_color                     = BG_MID;
    vis.selection.bg_fill                  = Color32::from_rgba_premultiplied(255, 215, 0, 40);
    vis.selection.stroke.color             = GOLD;
    vis.hyperlink_color                    = CYAN;
    vis.warn_fg_color                      = ORANGE;
    vis.error_fg_color                     = RED_HOT;

    vis.widgets.noninteractive.bg_fill     = BG_MID;
    vis.widgets.noninteractive.fg_stroke   = egui::Stroke::new(1.0, AXIS_LINE);
    vis.widgets.noninteractive.bg_stroke   = egui::Stroke::new(1.0, RING_LINE);
    vis.widgets.noninteractive.rounding    = Rounding::same(4.0);

    vis.widgets.inactive.bg_fill           = BG_CARD;
    vis.widgets.inactive.fg_stroke         = egui::Stroke::new(1.0, TEXT_DIM);
    vis.widgets.inactive.bg_stroke         = egui::Stroke::new(1.0, Color32::from_rgb(35, 35, 60));
    vis.widgets.inactive.rounding          = Rounding::same(4.0);

    vis.widgets.hovered.bg_fill            = BG_HOVER;
    vis.widgets.hovered.fg_stroke          = egui::Stroke::new(1.5, GOLD);
    vis.widgets.hovered.bg_stroke          = egui::Stroke::new(1.0, GOLD.linear_multiply(0.5));
    vis.widgets.hovered.rounding           = Rounding::same(4.0);

    vis.widgets.active.bg_fill             = BG_HOVER;
    vis.widgets.active.fg_stroke           = egui::Stroke::new(2.0, GOLD);
    vis.widgets.active.bg_stroke           = egui::Stroke::new(1.5, GOLD);
    vis.widgets.active.rounding            = Rounding::same(4.0);

    vis.widgets.open.bg_fill               = BG_HOVER;
    vis.widgets.open.fg_stroke             = egui::Stroke::new(1.5, CYAN);
    vis.widgets.open.rounding              = Rounding::same(4.0);

    style.visuals = vis;
    ctx.set_style(style);
}

/// Maps a defect score [0,1] to a severity color.
pub fn defect_color(score: f32) -> Color32 {
    if score >= 0.70 { RED_HOT }
    else if score >= 0.50 { ORANGE }
    else if score >= 0.30 { Color32::from_rgb(255, 200, 0) }
    else { GREEN }
}

/// Maps a sacred weight [0.85,1.0] to a priority color (lower = higher priority).
pub fn sacred_color(weight: f32) -> Color32 {
    if weight <= 0.87 { GOLD }
    else if weight <= 0.91 { CYAN }
    else if weight <= 0.96 { GREEN }
    else { TEXT_DIM }
}

/// Maps HPS score [0,1] to a quality color.
pub fn hps_color(hps: f32) -> Color32 {
    if hps >= 0.80 { GREEN }
    else if hps >= 0.50 { ORANGE }
    else { RED_HOT }
}

/// Card frame with themed fill and rounding.
pub fn card_frame(fill: Color32) -> egui::Frame {
    egui::Frame::none()
        .fill(fill)
        .inner_margin(Margin::same(10.0))
        .rounding(Rounding::same(4.0))
        .stroke(egui::Stroke::new(1.0, Color32::from_rgb(30, 30, 60)))
}
