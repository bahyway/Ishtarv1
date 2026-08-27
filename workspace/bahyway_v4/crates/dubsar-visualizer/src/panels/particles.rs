use egui::{pos2, Color32, FontId, RichText, Ui, Vec2};

use bahyway_core::ParticleState;
use enkidb_journal::Journal;
use enkidb_kaki::IdentityKaki;
use enkidb_snapshot::SnapshotRecord;
use story_engine::projection::{
    ATTR_COLOR_RGB, ATTR_SNAPSHOT_DATE, ATTR_SNAPSHOT_FREQUENCY, ATTR_SNAPSHOT_STATE,
};
use story_engine::StoryEngine;

use crate::app::{
    ParticleDemo, ATTR_DEFECT_SCORE, ATTR_EVENT_TYPE, ATTR_FRESHNESS, ATTR_QUALITY,
    ATTR_SECTOR_NAME, ATTR_SEGMENT_REF, ATTR_STATE,
};
use crate::panels::InspectorTab;
use crate::theme;

// ── Entry point ───────────────────────────────────────────────────────────────

pub fn draw(
    ui: &mut Ui,
    particles: &[ParticleDemo],
    selected: &mut usize,
    tab: &mut InspectorTab,
    journal: &Journal,
    identities: &[IdentityKaki],
    snapshots: &[SnapshotRecord],
) {
    if particles.is_empty() {
        ui.label("No particles.");
        return;
    }
    *selected = (*selected).min(particles.len() - 1);

    let mut new_sel = *selected;

    // ── Left: Houdini node list ───────────────────────────────────────
    egui::SidePanel::left("particle_node_panel")
        .resizable(true)
        .default_width(230.0)
        .min_width(160.0)
        .max_width(340.0)
        .frame(
            egui::Frame::none()
                .fill(theme::BG_MID)
                .inner_margin(egui::Margin::same(0.0)),
        )
        .show_inside(ui, |ui| {
            ui.add_space(6.0);
            ui.label(
                RichText::new("  Particle Nodes")
                    .color(theme::GOLD)
                    .heading(),
            );
            ui.add_space(4.0);
            ui.separator();
            egui::ScrollArea::vertical()
                .id_salt("pnode_list")
                .show(ui, |ui| {
                    for (i, p) in particles.iter().enumerate() {
                        if draw_node(ui, p, i, i == *selected) {
                            new_sel = i;
                        }
                    }
                });
        });

    *selected = new_sel;

    let identity = &identities[*selected];

    // ── Right: Parameter inspector ────────────────────────────────────
    egui::CentralPanel::default()
        .frame(
            egui::Frame::none()
                .fill(theme::BG_DARK)
                .inner_margin(egui::Margin::same(10.0)),
        )
        .show_inside(ui, |ui| {
            draw_inspector(ui, &particles[*selected], tab, identity, journal, snapshots);
        });
}

// ── Compact node row ──────────────────────────────────────────────────────────

fn draw_node(ui: &mut Ui, p: &ParticleDemo, idx: usize, is_sel: bool) -> bool {
    let node_h = 60.0_f32;
    let clicked = ui
        .push_id(idx, |ui| {
            let w = ui.available_width();
            let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, node_h), egui::Sense::click());

            let bg = if is_sel {
                theme::BG_HOVER
            } else if resp.hovered() {
                Color32::from_rgba_premultiplied(20, 20, 45, 200)
            } else {
                Color32::TRANSPARENT
            };
            ui.painter().rect_filled(rect, 0.0, bg);

            let sc = state_color(p.state);
            let bar = egui::Rect::from_min_size(
                rect.min + Vec2::new(0.0, 3.0),
                Vec2::new(3.0, node_h - 6.0),
            );
            ui.painter().rect_filled(bar, 1.5, sc);

            let (cr, cg, cb) = p.rgb;
            let dot_center = pos2(rect.min.x + 16.0, rect.center().y);
            ui.painter()
                .circle_filled(dot_center, 6.0, Color32::from_rgb(cr, cg, cb));
            ui.painter()
                .circle_stroke(dot_center, 6.0, egui::Stroke::new(1.0_f32, sc));

            let tx = rect.min.x + 28.0;

            ui.painter().text(
                pos2(tx, rect.min.y + 8.0),
                egui::Align2::LEFT_TOP,
                &p.id,
                FontId::monospace(11.0),
                theme::CYAN,
            );
            ui.painter().text(
                pos2(tx, rect.min.y + 24.0),
                egui::Align2::LEFT_TOP,
                &p.label,
                FontId::proportional(12.0),
                if is_sel {
                    theme::GOLD
                } else {
                    theme::TEXT_MAIN
                },
            );
            ui.painter().text(
                pos2(tx, rect.min.y + 42.0),
                egui::Align2::LEFT_TOP,
                format!("HPS {:.0}%  ·  {}", p.hps * 100.0, p.state),
                FontId::proportional(10.0),
                theme::hps_color(p.hps),
            );

            resp.clicked()
        })
        .inner;

    let sep = egui::Rect::from_min_size(ui.cursor().min, Vec2::new(ui.available_width(), 1.0));
    ui.painter().rect_filled(sep, 0.0, theme::RING_LINE);
    ui.add_space(1.0);

    clicked
}

// ── Inspector panel ───────────────────────────────────────────────────────────

fn draw_inspector(
    ui: &mut Ui,
    p: &ParticleDemo,
    tab: &mut InspectorTab,
    identity: &IdentityKaki,
    journal: &Journal,
    snapshots: &[SnapshotRecord],
) {
    ui.horizontal(|ui| {
        let (r, g, b) = p.rgb;
        let swatch = Color32::from_rgb(r, g, b);
        let (sw_rect, _) = ui.allocate_exact_size(Vec2::new(16.0, 16.0), egui::Sense::hover());
        ui.painter().rect_filled(sw_rect, 3.0, swatch);
        ui.add_space(6.0);
        ui.label(
            RichText::new(&p.label)
                .color(theme::GOLD)
                .strong()
                .size(15.0),
        );
        ui.add_space(8.0);
        ui.label(RichText::new(p.state).color(state_color(p.state)).strong());
        ui.add_space(8.0);
        ui.label(
            RichText::new(format!("HPS {:.0}%", p.hps * 100.0)).color(theme::hps_color(p.hps)),
        );
    });
    ui.add_space(4.0);

    ui.horizontal(|ui| {
        itab(ui, tab, InspectorTab::Radar, "⬡  Radar");
        itab(ui, tab, InspectorTab::Attributes, "⚙  Attributes");
        itab(ui, tab, InspectorTab::Story, "\u{1F4DC}  Story");
    });
    ui.separator();
    ui.add_space(4.0);

    match *tab {
        InspectorTab::Radar => tab_radar(ui, p),
        InspectorTab::Attributes => tab_attributes(ui, p),
        InspectorTab::Story => tab_story(ui, p, identity, journal, snapshots),
    }
}

fn itab(ui: &mut Ui, current: &mut InspectorTab, target: InspectorTab, label: &str) {
    let active = *current == target;
    let text = if active {
        RichText::new(label).color(theme::GOLD).strong()
    } else {
        RichText::new(label).color(theme::TEXT_DIM)
    };
    if ui
        .add(egui::Button::new(text).fill(if active {
            theme::BG_HOVER
        } else {
            Color32::TRANSPARENT
        }))
        .clicked()
    {
        *current = target;
    }
    ui.add_space(2.0);
}

// ── Tab: Radar ────────────────────────────────────────────────────────────────

fn tab_radar(ui: &mut Ui, p: &ParticleDemo) {
    let sz = ui
        .available_width()
        .min(ui.available_height() - 36.0)
        .max(200.0);
    let (resp, painter) = ui.allocate_painter(Vec2::splat(sz), egui::Sense::hover());
    crate::panels::hepta::draw_radar(&painter, resp.rect, p);

    ui.add_space(6.0);
    ui.horizontal_wrapped(|ui| {
        for (i, &v) in p.dims.iter().enumerate() {
            ui.colored_label(
                crate::panels::hepta::DIM_COLORS[i],
                format!("D{}:{:.2}", i + 1, v),
            );
            if i < 6 {
                ui.label(RichText::new(" · ").color(theme::TEXT_DIM));
            }
        }
    });
}

// ── Tab: Attributes ───────────────────────────────────────────────────────────

fn tab_attributes(ui: &mut Ui, p: &ParticleDemo) {
    egui::ScrollArea::vertical()
        .id_salt("attrs_scroll")
        .show(ui, |ui| {
            section_hdr(ui, "\u{12000}  KAKI Identity");
            theme::card_frame(theme::BG_CARD).show(ui, |ui| {
                ui.label(
                    RichText::new(format!("{:<12}{}", "full_key", p.kaki_hex))
                        .color(theme::TEXT_DIM)
                        .small()
                        .monospace(),
                );
                ui.add_space(6.0);

                const DIM_LBL: [&str; 7] = [
                    "D1 Identity  ",
                    "D2 Tribe     ",
                    "D3 Domain    ",
                    "D4 Quality   ",
                    "D5 Freshness ",
                    "D6 Temporal  ",
                    "D7 Integrity ",
                ];
                for (i, &v) in p.dims.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(DIM_LBL[i])
                                .color(theme::TEXT_DIM)
                                .small()
                                .monospace(),
                        );
                        dim_bar(ui, v, crate::panels::hepta::DIM_COLORS[i], 120.0);
                        ui.label(
                            RichText::new(format!("  {:.3}", v))
                                .color(crate::panels::hepta::DIM_COLORS[i])
                                .small(),
                        );
                    });
                }
            });
            ui.add_space(8.0);

            section_hdr(ui, "\u{26A1}  Mandatory Attributes");
            theme::card_frame(theme::BG_CARD).show(ui, |ui| {
                arow(ui, "id", &p.id, theme::CYAN);
                arow(ui, "label", &p.label, theme::TEXT_MAIN);
                arow(ui, "state", p.state, state_color(p.state));
                arow(
                    ui,
                    "hps",
                    &format!("{:.3}  ({:.0}%)", p.hps, p.hps * 100.0),
                    theme::hps_color(p.hps),
                );
                ui.horizontal(|ui| {
                    let (r, g, b) = p.rgb;
                    let col = Color32::from_rgb(r, g, b);
                    ui.label(
                        RichText::new(format!("{:<13}", "color_id"))
                            .color(theme::TEXT_DIM)
                            .small()
                            .monospace(),
                    );
                    let (sw, _) =
                        ui.allocate_exact_size(Vec2::new(12.0, 12.0), egui::Sense::hover());
                    ui.painter().rect_filled(sw, 2.0, col);
                    ui.label(
                        RichText::new(format!("  rgb({r}, {g}, {b})"))
                            .color(col)
                            .small()
                            .monospace(),
                    );
                });
            });
            ui.add_space(8.0);

            section_hdr(ui, "\u{25CE}  Optional Attributes");
            theme::card_frame(theme::BG_CARD).show(ui, |ui| {
                let sector = sector_from_id(&p.id);
                arow(ui, "sector", sector, theme::TEXT_MAIN);
                arow(ui, "tribe", "0x0001", theme::VIOLET);
                arow(ui, "pipeline_type", "Water", theme::CYAN);
                arow(ui, "engine", "wpd-engine v4.0", theme::TEXT_DIM);
            });
        });
}

fn sector_from_id(id: &str) -> &'static str {
    if id.len() < 7 {
        return "Unknown";
    }
    match &id[4..6] {
        "GZ" => "Green Zone  (H7-00 SUN)",
        "KZ" => "Al-Kadhimiya  (H7-01 MOON)",
        "SC" => "Sadr City  (H7-02 MERCURY)",
        "KR" => "Karrada  (H7-03 VENUS)",
        "RS" => "Rashid  (H7-04 MARS)",
        "JD" => "Al-Jadria  (H7-05 JUPITER)",
        "MN" => "Al-Mansour  (H7-06 SATURN)",
        _ => "Unknown",
    }
}

// ── Tab: Story (real StoryTellingEngine projection) ───────────────────────────

fn tab_story(
    ui: &mut Ui,
    p: &ParticleDemo,
    identity: &IdentityKaki,
    journal: &Journal,
    snapshots: &[SnapshotRecord],
) {
    let engine = StoryEngine::new(journal, snapshots);
    let projected = engine.project(identity);
    let mut history: Vec<_> = journal.read_particle_history(identity);
    history.sort_by_key(|e| e.epoch);

    egui::ScrollArea::vertical()
        .id_salt("story_scroll")
        .show(ui, |ui| {
            // ── Projected State Summary ────────────────────────────────
            theme::card_frame(theme::BG_CARD).show(ui, |ui| {
                ui.horizontal(|ui| {
                    let (state_lbl, state_col) = match projected.state {
                        ParticleState::Golden => ("Golden", theme::GOLD),
                        ParticleState::Fuzzy => ("Drifting", theme::ORANGE),
                        ParticleState::Dead => ("Gray", theme::TEXT_DIM),
                    };
                    ui.label(
                        RichText::new("Projected State")
                            .color(theme::TEXT_DIM)
                            .small(),
                    );
                    ui.add_space(8.0);
                    ui.label(RichText::new(state_lbl).color(state_col).strong().small());
                    ui.add_space(16.0);
                    ui.label(
                        RichText::new(format!("{} events replayed", projected.events_seen))
                            .color(theme::CYAN)
                            .small(),
                    );
                    if projected.from_snapshot {
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new("⚡ snapshot-accelerated")
                                .color(theme::GOLD)
                                .small(),
                        );
                    }
                });
                if let Some(q) = projected.get(ATTR_QUALITY) {
                    arow(ui, "quality", &String::from_utf8_lossy(q), theme::GREEN);
                }
                if let Some(f) = projected.get(ATTR_FRESHNESS) {
                    arow(ui, "freshness", &String::from_utf8_lossy(f), theme::CYAN);
                }
                if let Some(s) = projected.get(ATTR_SECTOR_NAME) {
                    arow(ui, "sector", &String::from_utf8_lossy(s), theme::TEXT_MAIN);
                }
                if let Some(d) = projected.get(ATTR_DEFECT_SCORE) {
                    let dv = String::from_utf8_lossy(d);
                    let df: f32 = dv.parse().unwrap_or(0.0);
                    arow(ui, "defect", &dv, theme::defect_color(df));
                }
            });
            ui.add_space(8.0);

            ui.label(
                RichText::new(format!(
                    "Event Timeline  ·  {} entries  ·  {}",
                    history.len(),
                    p.id
                ))
                .color(theme::TEXT_DIM)
                .small(),
            );
            ui.add_space(6.0);

            // ── Event entries ──────────────────────────────────────────
            for entry in &history {
                let event_type = entry
                    .eav
                    .iter()
                    .find(|t| t.attr_hash == ATTR_EVENT_TYPE)
                    .map(|t| String::from_utf8_lossy(&t.value).into_owned())
                    .unwrap_or_else(|| "EVENT".to_string());

                let ev_color = event_type_color(&event_type);

                theme::card_frame(theme::BG_CARD).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(&event_type)
                                .color(ev_color)
                                .small()
                                .strong()
                                .monospace(),
                        );
                        ui.add_space(10.0);
                        ui.label(
                            RichText::new(format!("epoch {}", entry.epoch))
                                .color(theme::TEXT_DIM)
                                .small()
                                .monospace(),
                        );
                    });
                    // All EAV triples except event_type
                    for triple in &entry.eav {
                        if triple.attr_hash == ATTR_EVENT_TYPE {
                            continue;
                        }
                        let name = attr_name(triple.attr_hash);
                        let val = String::from_utf8_lossy(&triple.value);
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(format!("  {:<16}", name))
                                    .color(theme::TEXT_DIM)
                                    .small()
                                    .monospace(),
                            );
                            ui.label(
                                RichText::new(val.as_ref())
                                    .color(theme::TEXT_MAIN)
                                    .small()
                                    .monospace(),
                            );
                        });
                    }
                });
                ui.add_space(3.0);
            }
        });
}

// ── Drawing helpers ───────────────────────────────────────────────────────────

fn section_hdr(ui: &mut Ui, title: &str) {
    ui.label(RichText::new(title).color(theme::CYAN).strong());
    ui.add_space(3.0);
}

fn arow(ui: &mut Ui, key: &str, val: &str, color: Color32) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!("{:<13}", key))
                .color(theme::TEXT_DIM)
                .small()
                .monospace(),
        );
        ui.label(RichText::new(val).color(color).small().monospace());
    });
}

fn dim_bar(ui: &mut Ui, value: f32, color: Color32, width: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 7.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 2.0, theme::BG_DARK);
    let fill = egui::Rect::from_min_size(rect.min, Vec2::new(width * value.clamp(0.0, 1.0), 7.0));
    ui.painter().rect_filled(fill, 2.0, color);
}

fn state_color(state: &str) -> Color32 {
    match state {
        "Golden" => theme::GOLD,
        "Drifting" => theme::ORANGE,
        _ => theme::TEXT_DIM,
    }
}

fn attr_name(hash: u32) -> &'static str {
    match hash {
        ATTR_STATE => "state",
        ATTR_QUALITY => "quality",
        ATTR_FRESHNESS => "freshness",
        ATTR_COLOR_RGB => "color_rgb",
        ATTR_SNAPSHOT_DATE => "snapshot_date",
        ATTR_SNAPSHOT_STATE => "snapshot_state",
        ATTR_SNAPSHOT_FREQUENCY => "snapshot_frequency",
        ATTR_SEGMENT_REF => "segment_ref",
        ATTR_DEFECT_SCORE => "defect_score",
        ATTR_SECTOR_NAME => "sector_name",
        ATTR_EVENT_TYPE => "event_type",
        _ => "?unknown",
    }
}

fn event_type_color(event_type: &str) -> Color32 {
    match event_type {
        "REGISTERED" => theme::CYAN,
        "INITIAL_SCAN" => theme::TEXT_MAIN,
        "SCORED" => theme::GREEN,
        "SNAPSHOT_TAKEN" => theme::GOLD,
        "VERIFIED" => theme::GREEN,
        "QUALITY_WARNING" => theme::ORANGE,
        "QUALITY_ALERT" => theme::RED_HOT,
        "MONITORED" => theme::ORANGE,
        "QUARANTINED" => theme::RED_HOT,
        "LIVE" => theme::VIOLET,
        _ => theme::TEXT_MAIN,
    }
}
