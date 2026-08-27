use egui::{Color32, RichText, Ui, Vec2};
use wpd_engine::{sovereign_segments, PipeMaterial, SegmentStatus};

use crate::theme;

pub fn draw(ui: &mut Ui) {
    ui.label(
        RichText::new("⛽  Baghdad WPD — Pipeline Defect Monitor")
            .color(theme::GOLD)
            .heading(),
    );
    ui.add_space(2.0);

    let segments = sovereign_segments();

    let n_defective = segments
        .iter()
        .filter(|s| s.status == SegmentStatus::Defective)
        .count();
    let n_suspected = segments
        .iter()
        .filter(|s| s.status == SegmentStatus::Suspected)
        .count();
    let n_active = segments.len() - n_defective - n_suspected;

    // Summary strip
    ui.horizontal(|ui| {
        ui.label(RichText::new(format!("⛔ Defective: {n_defective}")).color(theme::RED_HOT));
        ui.add_space(12.0);
        ui.label(RichText::new(format!("⚠ Suspected: {n_suspected}")).color(theme::ORANGE));
        ui.add_space(12.0);
        ui.label(RichText::new(format!("✓ Active: {n_active}")).color(theme::GREEN));
        ui.add_space(12.0);
        ui.label(
            RichText::new(format!("Total: {} segments", segments.len())).color(theme::TEXT_DIM),
        );
    });
    ui.separator();

    egui::ScrollArea::vertical()
        .id_salt("wpd_scroll")
        .show(ui, |ui| {
            for seg in &segments {
                let card_fill = match seg.status {
                    SegmentStatus::Defective => Color32::from_rgb(45, 10, 10),
                    SegmentStatus::Suspected => Color32::from_rgb(40, 25, 5),
                    _ => theme::BG_CARD,
                };

                theme::card_frame(card_fill).show(ui, |ui| {
                    // ── Header ───────────────────────────────────────
                    ui.horizontal(|ui| {
                        let (badge_c, badge) = status_badge(seg.status);
                        ui.label(RichText::new(badge).color(badge_c).small().strong());
                        ui.add_space(8.0);
                        ui.label(RichText::new(&seg.id).color(theme::CYAN).monospace());
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(format!(
                                "{}  ·  {}",
                                seg.sector.heptagram_id(),
                                seg.sector.planet_symbol()
                            ))
                            .color(theme::TEXT_DIM)
                            .small(),
                        );
                    });

                    // ── Sector name ───────────────────────────────────
                    ui.label(
                        RichText::new(seg.sector.name())
                            .color(theme::TEXT_MAIN)
                            .strong(),
                    );

                    // ── Defect score bar ──────────────────────────────
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Defect Risk ").color(theme::TEXT_DIM).small());
                        score_bar(ui, seg.defect_score, 180.0);
                        ui.label(
                            RichText::new(format!(" {:.0}%", seg.defect_score * 100.0))
                                .color(theme::defect_color(seg.defect_score))
                                .small()
                                .strong(),
                        );
                    });

                    // ── Density bar ────────────────────────────────────
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Density     ").color(theme::TEXT_DIM).small());
                        score_bar(ui, seg.sector.density_index(), 180.0);
                        ui.label(
                            RichText::new(format!(" {:.2}", seg.sector.density_index()))
                                .color(theme::CYAN)
                                .small(),
                        );
                    });

                    // ── Details row ────────────────────────────────────
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!(
                                "{}  ·  {}mm ⌀  ·  {} m",
                                material_name(seg.material),
                                seg.diameter_mm,
                                seg.length_m as u32,
                            ))
                            .color(theme::TEXT_DIM)
                            .small(),
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!("Installed: {}", seg.year_installed))
                                .color(theme::TEXT_DIM)
                                .small(),
                        );
                        ui.add_space(8.0);
                        if let Some(yr) = seg.last_inspected {
                            ui.label(
                                RichText::new(format!("Inspected: {yr}"))
                                    .color(theme::TEXT_DIM)
                                    .small(),
                            );
                        }
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(format!("Age: {} yrs", seg.age_years()))
                                .color(theme::ORANGE)
                                .small(),
                        );
                    });
                });

                ui.add_space(6.0);
            }
        });
}

fn status_badge(status: SegmentStatus) -> (Color32, &'static str) {
    match status {
        SegmentStatus::Defective => (theme::RED_HOT, "⛔ DEFECTIVE"),
        SegmentStatus::Suspected => (theme::ORANGE, "⚠  SUSPECTED"),
        SegmentStatus::Active => (theme::GREEN, "✓  ACTIVE"),
        SegmentStatus::Decommissioned => (theme::TEXT_DIM, "✗  DECOM"),
        SegmentStatus::Recovered => (theme::CYAN, "↑  RECOVERED"),
    }
}

fn material_name(mat: PipeMaterial) -> &'static str {
    match mat {
        PipeMaterial::AsbestosCement => "Asbestos Cement",
        PipeMaterial::CastIron => "Cast Iron",
        PipeMaterial::DuctileIron => "Ductile Iron",
        PipeMaterial::Plastic => "Plastic",
        PipeMaterial::Unknown => "Unknown",
    }
}

fn score_bar(ui: &mut Ui, score: f32, width: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 8.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 2.0, theme::BG_DARK);
    let filled = egui::Rect::from_min_size(rect.min, Vec2::new(width * score.clamp(0.0, 1.0), 8.0));
    ui.painter()
        .rect_filled(filled, 2.0, theme::defect_color(score));
}
