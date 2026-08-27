use std::f32::consts::PI;

use egui::epaint::{PathShape, PathStroke};
use egui::{pos2, Color32, FontId, Painter, Pos2, RichText, Shape, Stroke, Ui, Vec2};

use crate::app::ParticleDemo;
use crate::theme;

const AXIS_LABELS: [&str; 7] = [
    "D1 Identity",
    "D2 Tribe",
    "D3 Domain",
    "D4 Quality",
    "D5 Freshness",
    "D6 Temporal",
    "D7 Integrity",
];

pub(crate) const DIM_COLORS: [Color32; 7] = [
    Color32::from_rgb(255, 215, 0),   // D1 gold
    Color32::from_rgb(0, 255, 255),   // D2 cyan
    Color32::from_rgb(204, 68, 255),  // D3 violet
    Color32::from_rgb(0, 255, 136),   // D4 green
    Color32::from_rgb(255, 140, 0),   // D5 orange
    Color32::from_rgb(100, 180, 255), // D6 sky-blue
    Color32::from_rgb(255, 100, 100), // D7 rose
];

fn axis_pt(center: Pos2, radius: f32, axis: usize, value: f32) -> Pos2 {
    let angle = -PI / 2.0 + (axis as f32) * 2.0 * PI / 7.0;
    pos2(
        center.x + radius * value * angle.cos(),
        center.y + radius * value * angle.sin(),
    )
}

pub fn draw(ui: &mut Ui, particles: &[ParticleDemo], selected: &mut usize) {
    if particles.is_empty() {
        ui.label("No particles.");
        return;
    }
    *selected = (*selected).min(particles.len() - 1);

    ui.columns(2, |cols| {
        // ── Left column: particle list ────────────────────────────────
        let left = &mut cols[0];
        left.label(RichText::new("Particles").color(theme::GOLD).heading());
        left.add_space(4.0);
        egui::ScrollArea::vertical()
            .id_salt("hepta_list")
            .show(left, |ui| {
                for (i, p) in particles.iter().enumerate() {
                    let is_sel = i == *selected;
                    let txt = if is_sel {
                        RichText::new(format!("▶ {}  {}", p.id, p.label)).color(theme::GOLD)
                    } else {
                        RichText::new(format!("  {}  {}", p.id, p.label)).color(theme::TEXT_DIM)
                    };
                    if ui.selectable_label(is_sel, txt).clicked() {
                        *selected = i;
                    }
                    if is_sel {
                        ui.label(
                            RichText::new(format!(
                                "    HPS {:.0}%  ·  State: {}",
                                p.hps * 100.0,
                                p.state
                            ))
                            .color(theme::CYAN)
                            .small(),
                        );
                        ui.label(
                            RichText::new(format!("    {}", p.kaki_hex))
                                .color(theme::TEXT_DIM)
                                .small()
                                .monospace(),
                        );
                    }
                    ui.add_space(2.0);
                }
            });

        // ── Right column: radar chart ─────────────────────────────────
        let right = &mut cols[1];
        let particle = &particles[*selected];
        right.label(
            RichText::new(format!("⬡  {}  —  Hepta 7D", particle.label))
                .color(theme::GOLD)
                .heading(),
        );
        right.add_space(4.0);

        let sz = right
            .available_width()
            .min(right.available_height() - 60.0)
            .max(200.0);
        let (resp, painter) = right.allocate_painter(Vec2::splat(sz), egui::Sense::hover());
        draw_radar(&painter, resp.rect, particle);

        right.add_space(6.0);
        right.horizontal_wrapped(|ui| {
            for (i, &v) in particle.dims.iter().enumerate() {
                ui.colored_label(DIM_COLORS[i], format!("D{}:{:.2}", i + 1, v));
                if i < 6 {
                    ui.label(RichText::new(" · ").color(theme::TEXT_DIM));
                }
            }
        });
    });
}

pub(crate) fn draw_radar(painter: &Painter, rect: egui::Rect, p: &ParticleDemo) {
    let center = rect.center();
    let radius = rect.width().min(rect.height()) * 0.36;

    // Background disc
    painter.circle_filled(
        center,
        radius * 1.08,
        Color32::from_rgba_premultiplied(8, 8, 22, 230),
    );

    // Five concentric ring heptagons
    for ring in 1u32..=5 {
        let r = radius * ring as f32 / 5.0;
        let ring_pts: Vec<Pos2> = (0..7).map(|i| axis_pt(center, r, i, 1.0)).collect();
        painter.add(Shape::closed_line(
            ring_pts,
            Stroke::new(0.5_f32, theme::RING_LINE),
        ));
    }

    // Axis lines + labels
    for (i, label) in AXIS_LABELS.iter().enumerate() {
        let outer = axis_pt(center, radius, i, 1.0);
        painter.line_segment([center, outer], Stroke::new(0.8_f32, theme::AXIS_LINE));

        let lbl_pos = axis_pt(center, radius * 1.17, i, 1.0);
        painter.text(
            lbl_pos,
            egui::Align2::CENTER_CENTER,
            *label,
            FontId::proportional(10.0),
            theme::TEXT_DIM,
        );
    }

    // Data polygon — filled
    let data_pts: Vec<Pos2> = (0..7)
        .map(|i| axis_pt(center, radius, i, p.dims[i]))
        .collect();

    painter.add(Shape::Path(PathShape {
        points: data_pts.clone(),
        closed: true,
        fill: Color32::from_rgba_premultiplied(255, 215, 0, 38),
        stroke: PathStroke::new(2.0_f32, theme::GOLD),
    }));

    // Dim data points
    for (i, &pt) in data_pts.iter().enumerate() {
        painter.circle_filled(pt, 5.0, DIM_COLORS[i]);
        painter.circle_stroke(
            pt,
            6.5,
            Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(255, 255, 255, 50)),
        );
    }

    // Centre dot
    painter.circle_filled(center, 4.0, theme::GOLD);

    // HPS label in lower-centre of disc
    let (r, g, b) = p.rgb;
    let swatch_color = Color32::from_rgb(r, g, b);
    painter.text(
        pos2(center.x, center.y + radius * 0.50),
        egui::Align2::CENTER_CENTER,
        format!("HPS {:.0}%", p.hps * 100.0),
        FontId::proportional(13.0),
        swatch_color,
    );
}
