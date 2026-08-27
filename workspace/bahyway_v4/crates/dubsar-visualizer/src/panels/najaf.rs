use std::f32::consts::PI;

use egui::{pos2, Color32, FontId, Painter, Pos2, RichText, Shape, Stroke, Ui, Vec2};

use crate::theme;

// ── Static sector data ────────────────────────────────────────────────────────
// (English name, Arabic name, sacred_weight, approx_graves)
// Wadi al-Salam is estimated at ~6 million total burials.
const SECTORS: [(&str, &str, f32, u32); 7] = [
    (
        "Entrance",
        "\u{0627}\u{0644}\u{0645}\u{062F}\u{062E}\u{0644}",
        1.00,
        50_000,
    ),
    (
        "Martyrs",
        "\u{0627}\u{0644}\u{0634}\u{0647}\u{062F}\u{0627}\u{0621}",
        0.85,
        800_000,
    ),
    (
        "Saints",
        "\u{0627}\u{0644}\u{0623}\u{0648}\u{0644}\u{064A}\u{0627}\u{0621}",
        0.90,
        950_000,
    ),
    (
        "Memorisers",
        "\u{0627}\u{0644}\u{062D}\u{0641}\u{0651}\u{0627}\u{0638}",
        0.95,
        700_000,
    ),
    (
        "Believers",
        "\u{0627}\u{0644}\u{0645}\u{0624}\u{0645}\u{0646}\u{064A}\u{0646}",
        1.00,
        1_200_000,
    ),
    (
        "Scholars",
        "\u{0627}\u{0644}\u{0639}\u{0644}\u{0645}\u{0627}\u{0621}",
        0.92,
        600_000,
    ),
    (
        "Prophets",
        "\u{0627}\u{0644}\u{0623}\u{0646}\u{0628}\u{064A}\u{0627}\u{0621}",
        0.88,
        850_000,
    ),
];

// Map: idx=0 → centre; idx=1..=6 → rim nodes at N,NE,SE,S,SW,NW (60° intervals)
fn node_pos(center: Pos2, radius: f32, idx: usize) -> Pos2 {
    if idx == 0 {
        return center;
    }
    let angle = -PI / 2.0 + (idx as f32 - 1.0) * 2.0 * PI / 6.0;
    pos2(
        center.x + radius * angle.cos(),
        center.y + radius * angle.sin(),
    )
}

pub fn draw(ui: &mut Ui) {
    ui.label(
        RichText::new("\u{262B}  Najaf — Wadi al-Salam · 7 Sacred Zones")
            .color(theme::GOLD)
            .heading(),
    );

    let total_graves: u32 = SECTORS.iter().map(|s| s.3).sum();
    ui.label(
        RichText::new(format!(
            "World's largest Islamic cemetery  ·  ~{} approx. burials",
            fmt_num(total_graves)
        ))
        .color(theme::TEXT_DIM)
        .small(),
    );
    ui.separator();

    ui.columns(2, |cols| {
        // ── Left: heptagram map ───────────────────────────────────────
        let left = &mut cols[0];
        let map_sz = left
            .available_width()
            .min(left.available_height() - 40.0)
            .max(200.0);
        let (resp, painter) = left.allocate_painter(Vec2::splat(map_sz), egui::Sense::hover());
        draw_map(&painter, resp.rect);

        // ── Right: sector cards ───────────────────────────────────────
        let right = &mut cols[1];
        egui::ScrollArea::vertical()
            .id_salt("najaf_cards")
            .show(right, |ui| {
                for (i, &(eng, arab, weight, graves)) in SECTORS.iter().enumerate() {
                    let sec_color = theme::sacred_color(weight);
                    theme::card_frame(theme::BG_CARD).show(ui, |ui| {
                        // Header: sector index + names
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(format!("H{}", i))
                                    .color(theme::TEXT_DIM)
                                    .small()
                                    .monospace(),
                            );
                            ui.add_space(4.0);
                            // Arabic renders as boxes without an Arabic font,
                            // but is included for completeness and correct copy-paste.
                            ui.label(RichText::new(arab).color(sec_color).size(14.0));
                            ui.label(RichText::new(format!("  ({eng})")).color(theme::TEXT_DIM));
                        });

                        // Sacred weight + grave count
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new("Sacred Weight:")
                                    .color(theme::TEXT_DIM)
                                    .small(),
                            );
                            ui.label(
                                RichText::new(format!("{:.2}", weight))
                                    .color(sec_color)
                                    .small()
                                    .strong(),
                            );
                            ui.add_space(10.0);
                            ui.label(RichText::new("Graves:").color(theme::TEXT_DIM).small());
                            ui.label(RichText::new(fmt_num(graves)).color(theme::CYAN).small());
                        });

                        // Pilgrimage priority bar (inverted: lower weight → higher priority)
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Priority  ").color(theme::TEXT_DIM).small());
                            let priority = ((1.0 - weight) / 0.15).clamp(0.0, 1.0);
                            priority_bar(ui, priority, sec_color, 120.0);
                        });
                    });
                    ui.add_space(4.0);
                }
            });
    });
}

fn draw_map(painter: &Painter, rect: egui::Rect) {
    let center = rect.center();
    let radius = rect.width().min(rect.height()) * 0.36;

    // Background
    painter.circle_filled(
        center,
        radius * 1.08,
        Color32::from_rgba_premultiplied(6, 10, 4, 230),
    );

    // Spokes: centre → each rim node
    for rim in 1..=6usize {
        let rim_pt = node_pos(center, radius, rim);
        painter.line_segment([center, rim_pt], Stroke::new(1.0_f32, theme::AXIS_LINE));
    }

    // Rim ring: connect rim nodes in order
    {
        let rim_pts: Vec<Pos2> = (1..=6).map(|i| node_pos(center, radius, i)).collect();
        painter.add(Shape::closed_line(
            rim_pts,
            Stroke::new(1.0_f32, theme::RING_LINE),
        ));
    }

    // Sector nodes + labels
    for (i, &(eng, arab, weight, _)) in SECTORS.iter().enumerate() {
        let pos = node_pos(center, radius, i);
        let col = theme::sacred_color(weight);
        let node_r = if i == 0 { 18.0_f32 } else { 13.0_f32 };

        // Node fill + border
        painter.circle_filled(pos, node_r, col.linear_multiply(0.22));
        painter.circle_stroke(pos, node_r, Stroke::new(1.5_f32, col));

        // Label position: outward from center for rim nodes
        let lbl = if i == 0 {
            pos2(pos.x, pos.y + node_r + 14.0)
        } else {
            let angle = -PI / 2.0 + (i as f32 - 1.0) * 2.0 * PI / 6.0;
            pos2(
                pos.x + (node_r + 13.0) * angle.cos(),
                pos.y + (node_r + 13.0) * angle.sin(),
            )
        };

        painter.text(
            lbl,
            egui::Align2::CENTER_CENTER,
            arab,
            FontId::proportional(10.0),
            col,
        );
        painter.text(
            pos2(lbl.x, lbl.y + 11.0),
            egui::Align2::CENTER_CENTER,
            eng,
            FontId::proportional(9.0),
            theme::TEXT_DIM,
        );
    }

    // Title label
    painter.text(
        pos2(rect.center().x, rect.min.y + 10.0),
        egui::Align2::CENTER_TOP,
        "Wadi al-Salam",
        FontId::proportional(11.0),
        theme::GOLD,
    );
}

fn priority_bar(ui: &mut Ui, priority: f32, color: Color32, width: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 6.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 2.0, theme::BG_DARK);
    let fill =
        egui::Rect::from_min_size(rect.min, Vec2::new(width * priority.clamp(0.0, 1.0), 6.0));
    ui.painter().rect_filled(fill, 2.0, color);
}

fn fmt_num(n: u32) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{}K", n / 1_000)
    } else {
        format!("{n}")
    }
}
