//! 𒁾 UrOS v4.0 — Sovereign IDE Layout
//!
//! Dual-sidebar layout:
//!   Left  sidebar — miniature viz panels (click → floating window)
//!   Centre        — 5 sovereign language tabs with editor + runner
//!   Right sidebar — live viz panels (click → floating window)
//!   NISABA        — floating hologram agent (summon top-right)
//!
//! Corrected 2026-07-17: this panel was first landed calling the hologram
//! agent "NABA" — a typo carried forward from an upload's own internal
//! inconsistency (Rust identifiers said NABA, its displayed title already
//! said NISABA). The Architect corrected it against real evidence: NISABA
//! 𒀭𒀭𒂗𒍪 is the Sumerian goddess of writing/scribal wisdom, patron of the
//! É-DUBBA, with a real Godot hologram implementation (playbook 55) that
//! narrates query results and witnesses É-DUBBA seal events -- exactly
//! this panel's role. "NABA" was never a real name.
//!
//! NAMING COLLISION, resolved 2026-07-18: this repo already had a real,
//! sealed `nisaba` crate (Pattern Discovery Service -- GA clustering over
//! 7D particle trajectories, Pattern-KAKI minting, E7 Viazovska 126-fanout
//! constraint). The Architect granted NISABA the role of Internal
//! Interface/orchestrator across the Ecosystem, which is what actually
//! joins the two: `nisaba::NisabaOrchestrator` now lives in that same
//! crate, and this hologram panel is its live front end -- real
//! `story_engine::StoryEngine` projections over the real journal feed
//! `story-sentinel`'s alert bridge, and real `ParticleDemo`/`IdentityKaki`
//! data feeds `lamassu-engine`'s TDA scan, both via the orchestrator.
//! Per CSR-08 (reaffirmed for this role), NISABA only observes and
//! proposes here -- see `nisaba::orchestrator`'s doc comment; nothing in
//! this panel writes to the ecosystem.
//!
//! Language tabs:
//!   1. HeptaScript  (.hepta) — W5H2 sovereign query language
//!   2. AkkadianAOL  (.akk)   — sovereign orchestration language
//!   3. PMPVD        (.pmpvd) — particles meta-programming
//!   4. Way v2.0     (.way)   — security policy language
//!   5. TemplateEngine(.tpl)  — EAV schema templates
//!
//! The Run button's output is a scripted preview, not a live execution
//! against the real HeptaScript/AAOL compilers — same demo-scaffolding
//! status as this crate's seeded `ParticleDemo` data, not a claim that
//! these languages execute for real from this panel.
//!
//! DUB.SAR 𒁾 — BahyWay.Ecosystem v4.0 | Pure Rust | eframe 0.29

use crate::app::ParticleDemo;
use crate::theme;
use bee_mdm_bus::BeeMdmBusState;
use egui::{Color32, FontId, Pos2, RichText, Rounding, Stroke, TextEdit, Ui, Vec2, Window};
use enkidb_journal::Journal;
use enkidb_kaki::IdentityKaki;
use enkidb_snapshot::SnapshotRecord;
use nisaba::NisabaOrchestrator;
use std::sync::{Arc, RwLock};
use story_engine::StoryEngine;

/// Sampling parameters for NISABA's LamassuEngine scans over this panel's
/// seeded particle cloud. Matches the scale `lamassu-engine`'s own tests
/// use — the exact radius doesn't matter for a demo cloud, only that the
/// math is real (`bahyway_algebra::persistence`, not a canned string).
const LAMASSU_MAX_EPSILON: f64 = 1.2;
const LAMASSU_R_MAX: f64 = 10.0;
const LAMASSU_H_MAX: f64 = 5.0;

// ── Language tab definitions ──────────────────────────────────────────
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LangTab {
    HeptaScript,
    AkkadianAol,
    Pmpvd,
    Way,
    Template,
}

impl LangTab {
    pub fn label(self) -> &'static str {
        match self {
            Self::HeptaScript => "⬡ HeptaScript",
            Self::AkkadianAol => "𒀭 AAOL .akk",
            Self::Pmpvd => "◈ PMPVD",
            Self::Way => "🔒 Way v2.0",
            Self::Template => "📋 Templates",
        }
    }
    pub fn extension(self) -> &'static str {
        match self {
            Self::HeptaScript => ".hepta",
            Self::AkkadianAol => ".akk",
            Self::Pmpvd => ".pmpvd",
            Self::Way => ".way",
            Self::Template => ".tpl",
        }
    }
    pub fn placeholder(self) -> &'static str {
        match self {
            Self::HeptaScript => {
                "\
-- HeptaScript W5H2 Query (.hepta)
WHO   Citizens.E
WHAT  E[name.given, city.name]
WHERE E[city.name] = \"Najaf\"
WHEN  AT EPOCH NOW
WHY   QUALITY_BYTE > 200
HOW   BY E[name.given] ASC
HOW_MUCH LIMIT 50"
            }
            Self::AkkadianAol => {
                "\
-- AkkadianAOL (.akk)
PARTICLE najaf_grave {
    person.name_arabic: TEXT(accuracy=0.98)
    death.date:         TIMESTAMP(timeliness=fresh)
    grave.coord:        COORDINATE(validity=WGS84)
    TRIBE:              NAJAF_CEMETERY
    KAKI:               auto
}"
            }
            Self::Pmpvd => {
                "\
-- PMPVD (.pmpvd)
PIPELINE najaf_intake v4.0 {
  FLOW intake {
    SOURCE: LandingZone(path=/beemdm/zip_landing)
    PIPE:   Musaru(scan=malware)
    PIPE:   ZipEngine(format=all)
    SINK:   PermanentStorage(db=enkidb, fsync=PerCommit)
  }
}"
            }
            Self::Way => {
                "\
-- Way v2.0 (.way)
sovereign NajafPolicy {
    version: \"4.0.2\";
    domain_byte: 0x0001;
    hepta {
        D1 (Accuracy):     0.25;
        D2 (Completeness): 0.20;
        D3 (Consistency):  0.15;
        D4 (Validity):     0.20;
        D5 (Uniqueness):   0.10;
        D6 (Timeliness):   0.05;
        D7 (Integrity):    0.05;
    }
    rules {
        IF role = ARCHITECT AND seal = SargonSeal
        THEN access = FULL_WRITE;
    }
}"
            }
            Self::Template => {
                "\
-- TemplateEngine (.tpl)
TEMPLATE civil_registry {
    FIELD name.given:    TEXT  required
    FIELD name.family:   TEXT  required
    FIELD birth.date:    DATE  required
    FIELD national.id:   KAKI  required
    FIELD district.code: TEXT  optional
    ORIGIN: ILKUM_DOC
}"
            }
        }
    }
}

// ── Floating panel state ──────────────────────────────────────────────
#[derive(Default)]
pub struct FloatingPanels {
    pub etl_3d: bool,
    pub beemdm: bool,
    pub nergal: bool,
    pub particles: bool,
    pub hepta_radar: bool,
    pub etl_flow: bool,
    pub b11_gauge: bool,
    pub story: bool,
    pub nisaba: bool,
}

// ── NISABA agent state ──────────────────────────────────────────────────
pub struct NisabaState {
    pub input: String,
    pub history: Vec<(String, String)>, // (user, agent)
}
impl Default for NisabaState {
    fn default() -> Self {
        Self {
            input: String::new(),
            history: vec![(
                String::new(),
                "𒀭 NISABA active — sovereign hologram agent ready.\n\
                 Ask about your .hepta queries, KAKI particles,\n\
                 B11 scores, or BeeMDM pipeline status."
                    .into(),
            )],
        }
    }
}

// ── UrOS layout state ──────────────────────────────────────────────
pub struct EriduState {
    pub lang_tab: LangTab,
    pub editors: [String; 5], // one buffer per language tab
    pub outputs: [String; 5], // one output per language tab
    pub running: [bool; 5],
    pub left_open: bool,
    pub right_open: bool,
    pub float: FloatingPanels,
    pub nisaba: NisabaState,
    pub selected_p: usize,
    /// NISABA's Internal Interface — real alert-engine/StewardStation/
    /// LamassuEngine orchestration, granted 2026-07-18. See module doc.
    pub nisaba_orch: NisabaOrchestrator,
}

impl Default for EriduState {
    fn default() -> Self {
        let tabs = [
            LangTab::HeptaScript,
            LangTab::AkkadianAol,
            LangTab::Pmpvd,
            LangTab::Way,
            LangTab::Template,
        ];
        let mut editors: [String; 5] = Default::default();
        for (i, t) in tabs.iter().enumerate() {
            editors[i] = t.placeholder().to_string();
        }
        Self {
            lang_tab: LangTab::HeptaScript,
            editors,
            outputs: Default::default(),
            running: Default::default(),
            left_open: true,
            right_open: true,
            float: Default::default(),
            nisaba: Default::default(),
            selected_p: 0,
            // This desktop UI is a development/demo surface — it never
            // installs an autonomy grant, so it always runs fully
            // propose-only regardless of environment tag.
            nisaba_orch: NisabaOrchestrator::new(
                nisaba::NisabaEnvironment::Test,
                LAMASSU_MAX_EPSILON,
                LAMASSU_R_MAX,
                LAMASSU_H_MAX,
            ),
        }
    }
}

// ── Main entry point ──────────────────────────────────────────────────
pub fn draw(
    ui: &mut Ui,
    state: &mut EriduState,
    bus: &Arc<RwLock<BeeMdmBusState>>,
    particles: &[ParticleDemo],
    identities: &[IdentityKaki],
    journal: &Journal,
    snapshots: &[SnapshotRecord],
) {
    let bus_state = bus.read().ok();

    // ── Top bar: title + NISABA summon ────────────────────────────────
    egui::TopBottomPanel::top("eridu_top")
        .exact_height(36.0)
        .frame(
            egui::Frame::none()
                .fill(Color32::from_rgb(4, 4, 16))
                .stroke(Stroke::new(1.0_f32, theme::GOLD)),
        )
        .show_inside(ui, |ui| {
            ui.horizontal_centered(|ui| {
                ui.label(
                    RichText::new("𒁾  UrOS v4.0")
                        .color(theme::GOLD)
                        .size(15.0)
                        .strong(),
                );
                ui.separator();
                if ui
                    .button(if state.left_open {
                        "◀ Left"
                    } else {
                        "▶ Left"
                    })
                    .clicked()
                {
                    state.left_open = !state.left_open;
                }
                if ui
                    .button(if state.right_open {
                        "Right ▶"
                    } else {
                        "Right ◀"
                    })
                    .clicked()
                {
                    state.right_open = !state.right_open;
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let nisaba_col = if state.float.nisaba {
                        theme::GOLD
                    } else {
                        theme::TEXT_DIM
                    };
                    if ui
                        .button(RichText::new("𒀭 NISABA").color(nisaba_col).strong())
                        .clicked()
                    {
                        state.float.nisaba = !state.float.nisaba;
                    }
                });
            });
        });

    // ── Left sidebar ────────────────────────────────────────────────
    if state.left_open {
        egui::SidePanel::left("eridu_left")
            .resizable(true)
            .default_width(220.0)
            .min_width(160.0)
            .max_width(320.0)
            .frame(
                egui::Frame::none()
                    .fill(Color32::from_rgb(6, 6, 20))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(30, 30, 80))),
            )
            .show_inside(ui, |ui| {
                draw_left_sidebar(ui, state, &bus_state);
            });
    }

    // ── Right sidebar ───────────────────────────────────────────────
    if state.right_open {
        egui::SidePanel::right("eridu_right")
            .resizable(true)
            .default_width(240.0)
            .min_width(180.0)
            .max_width(340.0)
            .frame(
                egui::Frame::none()
                    .fill(Color32::from_rgb(6, 6, 20))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(30, 30, 80))),
            )
            .show_inside(ui, |ui| {
                draw_right_sidebar(ui, state, &bus_state, particles);
            });
    }

    // ── Centre: 5 language tabs ─────────────────────────────────────
    egui::CentralPanel::default()
        .frame(egui::Frame::none().fill(Color32::from_rgb(6, 8, 18)))
        .show_inside(ui, |ui| {
            draw_centre(ui, state);
        });

    // ── Floating panels ─────────────────────────────────────────────
    draw_floats(
        ui.ctx(),
        state,
        &bus_state,
        particles,
        identities,
        journal,
        snapshots,
    );
}

// ── Left sidebar content ──────────────────────────────────────────────
fn draw_left_sidebar(
    ui: &mut Ui,
    state: &mut EriduState,
    bus_state: &Option<std::sync::RwLockReadGuard<BeeMdmBusState>>,
) {
    ui.add_space(6.0);
    sidebar_label(ui, "VISUALISATIONS");
    ui.add_space(4.0);

    mini_panel(
        ui,
        "⇄  ETL 3D",
        "Transparency Dashboard",
        Color32::from_rgb(0, 180, 255),
        &mut state.float.etl_3d,
    );

    mini_panel(
        ui,
        "⬡  BeeMDM",
        "Pipeline Gate Monitor",
        Color32::from_rgb(0, 220, 120),
        &mut state.float.beemdm,
    );

    mini_panel(
        ui,
        "𒀭𒉺𒂵 Nergal",
        "IRKALLA Threat Towers",
        Color32::from_rgb(128, 0, 32),
        &mut state.float.nergal,
    );

    mini_panel(
        ui,
        "◉  Particles",
        "Sovereign Orbit Cloud",
        Color32::from_rgb(200, 160, 40),
        &mut state.float.particles,
    );

    ui.add_space(12.0);
    sidebar_label(ui, "PIPELINE STATUS");
    ui.add_space(4.0);

    if let Some(bus) = bus_state {
        let malware = bus.etl.sdb_malware_hits;
        let detect = bus.gates.g7_nergal();
        let golden = bus.total_golden;
        let proc = bus.total_processed;

        stat_mini(ui, "Processed", &fmt_n(proc), theme::TEXT_MAIN);
        stat_mini(ui, "Golden", &fmt_n(golden), theme::GOLD);
        stat_mini(
            ui,
            "Jailed (IRKALLA)",
            &fmt_n(bus.total_quarantined),
            Color32::from_rgb(200, 40, 40),
        );
        if malware > 0 || detect > 0 {
            stat_mini(
                ui,
                "⛔ Threats",
                &format!("{}", malware + detect),
                Color32::from_rgb(220, 40, 60),
            );
        }

        ui.add_space(8.0);
        let b11 = bus.fuzzy.b11 as f32;
        let frac = (b11 / 240.0).clamp(0.0, 1.0);
        let b11_c = if b11 >= 200.0 {
            theme::GOLD
        } else if b11 >= 140.0 {
            theme::CYAN
        } else {
            Color32::from_rgb(200, 40, 40)
        };
        ui.label(
            RichText::new(format!("B11  {:.0}/240", b11))
                .color(b11_c)
                .size(10.0),
        );
        let (bar, _) = ui.allocate_exact_size(
            Vec2::new(ui.available_width() - 8.0, 5.0),
            egui::Sense::hover(),
        );
        ui.painter()
            .rect_filled(bar, Rounding::same(2.0), Color32::from_rgb(20, 20, 40));
        let fill = egui::Rect::from_min_size(bar.min, Vec2::new(bar.width() * frac, 5.0));
        ui.painter().rect_filled(fill, Rounding::same(2.0), b11_c);
    } else {
        ui.label(
            RichText::new("Bus disconnected")
                .color(theme::TEXT_DIM)
                .size(10.0),
        );
    }
}

// ── Right sidebar content ─────────────────────────────────────────────
fn draw_right_sidebar(
    ui: &mut Ui,
    state: &mut EriduState,
    bus_state: &Option<std::sync::RwLockReadGuard<BeeMdmBusState>>,
    particles: &[ParticleDemo],
) {
    ui.add_space(6.0);
    sidebar_label(ui, "LIVE PANELS");
    ui.add_space(4.0);

    mini_panel(
        ui,
        "⬡  Hepta 7D",
        "Particle Quality Radar",
        theme::GOLD,
        &mut state.float.hepta_radar,
    );

    mini_panel(
        ui,
        "⇄  ETL Flow",
        "Pipeline Tier View",
        Color32::from_rgb(0, 160, 255),
        &mut state.float.etl_flow,
    );

    mini_panel(
        ui,
        "◈  B11 Gauge",
        "Quality Score Gauge",
        theme::CYAN,
        &mut state.float.b11_gauge,
    );

    mini_panel(
        ui,
        "📖 StoryEngine",
        "Particle Audit Trail",
        Color32::from_rgb(200, 100, 255),
        &mut state.float.story,
    );

    ui.add_space(10.0);
    sidebar_label(ui, "HEPTA 7D — QUICK VIEW");
    ui.add_space(4.0);

    if !particles.is_empty() {
        let p = &particles[state.selected_p.min(particles.len() - 1)];
        ui.label(RichText::new(&p.label).color(theme::GOLD).size(11.0));

        let avail = ui.available_width().min(200.0);
        let (resp, painter) =
            ui.allocate_painter(Vec2::new(avail, avail * 0.85), egui::Sense::hover());
        let rect = resp.rect;
        let center = rect.center();
        let radius = avail * 0.38;

        painter.rect_filled(rect, Rounding::same(4.0), Color32::from_rgb(6, 6, 20));

        use std::f32::consts::PI;
        for ring in 1..=4u32 {
            let r = radius * ring as f32 / 4.0;
            let pts: Vec<Pos2> = (0..7)
                .map(|i| {
                    let a = -PI / 2.0 + i as f32 * 2.0 * PI / 7.0;
                    Pos2::new(center.x + r * a.cos(), center.y + r * a.sin())
                })
                .collect();
            painter.add(egui::Shape::closed_line(
                pts,
                Stroke::new(0.4_f32, Color32::from_rgb(30, 40, 80)),
            ));
        }
        for i in 0..7usize {
            let a = -PI / 2.0 + i as f32 * 2.0 * PI / 7.0;
            painter.line_segment(
                [
                    center,
                    Pos2::new(center.x + radius * a.cos(), center.y + radius * a.sin()),
                ],
                Stroke::new(0.4_f32, Color32::from_rgb(40, 50, 100)),
            );
        }
        let pts: Vec<Pos2> = p
            .dims
            .iter()
            .enumerate()
            .map(|(i, &v)| {
                let a = -PI / 2.0 + i as f32 * 2.0 * PI / 7.0;
                Pos2::new(
                    center.x + radius * v * a.cos(),
                    center.y + radius * v * a.sin(),
                )
            })
            .collect();
        painter.add(egui::Shape::convex_polygon(
            pts.clone(),
            Color32::from_rgba_premultiplied(200, 160, 40, 60),
            Stroke::new(1.5_f32, theme::GOLD),
        ));
        for (i, &pt) in pts.iter().enumerate() {
            painter.circle_filled(pt, 3.0, crate::panels::hepta::DIM_COLORS[i]);
        }
        painter.text(
            center,
            egui::Align2::CENTER_CENTER,
            format!("HPS {:.0}%", p.hps * 100.0),
            FontId::proportional(10.0),
            theme::CYAN,
        );

        ui.add_space(4.0);
        ui.horizontal(|ui| {
            if ui.small_button("◀").clicked() && state.selected_p > 0 {
                state.selected_p -= 1;
            }
            ui.label(
                RichText::new(format!("{}/{}", state.selected_p + 1, particles.len()))
                    .color(theme::TEXT_DIM)
                    .size(10.0),
            );
            if ui.small_button("▶").clicked() && state.selected_p < particles.len() - 1 {
                state.selected_p += 1;
            }
        });
    }

    if let Some(bus) = bus_state {
        let threats: usize = bus.gates.counts.iter().sum();
        if threats > 0 {
            ui.add_space(10.0);
            sidebar_label(ui, "𒀭𒉺𒂵 NERGAL ALERT");
            ui.add_space(2.0);
            egui::Frame::none()
                .fill(Color32::from_rgb(30, 4, 8))
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(128, 0, 32)))
                .rounding(Rounding::same(4.0))
                .inner_margin(egui::Margin::same(6.0))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(format!("⛔ {} threat(s) detected", threats))
                            .color(Color32::from_rgb(220, 40, 60))
                            .size(11.0)
                            .strong(),
                    );
                    if ui.small_button("Open Nergal →").clicked() {
                        state.float.nergal = true;
                    }
                });
        }
    }
}

// ── Centre: 5 language tabs ───────────────────────────────────────────
fn draw_centre(ui: &mut Ui, state: &mut EriduState) {
    let tabs = [
        LangTab::HeptaScript,
        LangTab::AkkadianAol,
        LangTab::Pmpvd,
        LangTab::Way,
        LangTab::Template,
    ];

    ui.horizontal(|ui| {
        ui.add_space(4.0);
        for tab in tabs {
            let active = state.lang_tab == tab;
            let col = if active { theme::GOLD } else { theme::TEXT_DIM };
            let fill = if active {
                theme::BG_HOVER
            } else {
                Color32::TRANSPARENT
            };
            let btn = egui::Button::new(RichText::new(tab.label()).color(col).size(12.0))
                .fill(fill)
                .rounding(Rounding::same(4.0));
            if ui.add(btn).clicked() {
                state.lang_tab = tab;
            }
            ui.add_space(2.0);
        }
        let ext = state.lang_tab.extension();
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                RichText::new(ext)
                    .color(theme::TEXT_DIM)
                    .size(10.0)
                    .monospace(),
            );
        });
    });

    ui.separator();

    let idx = tabs.iter().position(|&t| t == state.lang_tab).unwrap_or(0);

    let avail_h = ui.available_height();
    let editor_h = avail_h * 0.62;
    let output_h = avail_h * 0.35;

    egui::Frame::none()
        .fill(Color32::from_rgb(8, 10, 24))
        .stroke(Stroke::new(0.5_f32, Color32::from_rgb(30, 40, 80)))
        .rounding(Rounding::same(4.0))
        .inner_margin(egui::Margin::same(6.0))
        .show(ui, |ui| {
            ui.set_min_height(editor_h);
            ui.set_max_height(editor_h);
            egui::ScrollArea::both()
                .id_salt(format!("editor_{idx}"))
                .show(ui, |ui| {
                    let editor = TextEdit::multiline(&mut state.editors[idx])
                        .font(egui::TextStyle::Monospace)
                        .desired_rows(24)
                        .desired_width(f32::INFINITY)
                        .code_editor()
                        .text_color(Color32::from_rgb(200, 220, 255));
                    ui.add(editor);
                });
        });

    ui.add_space(4.0);

    ui.horizontal(|ui| {
        let run_label = if state.running[idx] {
            "⏳ Running..."
        } else {
            "▶  Run"
        };
        let run_btn = egui::Button::new(RichText::new(run_label).color(Color32::BLACK).strong())
            .fill(theme::GOLD)
            .min_size(Vec2::new(80.0, 26.0))
            .rounding(Rounding::same(4.0));
        if ui.add(run_btn).clicked() && !state.running[idx] {
            state.outputs[idx] = run_sovereign_lang(state.lang_tab, &state.editors[idx]);
        }
        ui.add_space(8.0);
        if ui.small_button("Clear").clicked() {
            state.outputs[idx].clear();
        }
        if ui.small_button("Copy to clipboard").clicked() {
            ui.output_mut(|o| o.copied_text = state.outputs[idx].clone());
        }
    });

    ui.add_space(2.0);

    egui::Frame::none()
        .fill(Color32::from_rgb(4, 12, 8))
        .stroke(Stroke::new(0.5_f32, Color32::from_rgb(0, 60, 30)))
        .rounding(Rounding::same(4.0))
        .inner_margin(egui::Margin::same(6.0))
        .show(ui, |ui| {
            ui.set_min_height(output_h);
            ui.label(
                RichText::new("── Output (scripted preview, not live execution) ──")
                    .color(theme::TEXT_DIM)
                    .size(10.0),
            );
            ui.add_space(2.0);
            egui::ScrollArea::both()
                .id_salt(format!("output_{idx}"))
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    let out_col = if state.outputs[idx].contains("Error")
                        || state.outputs[idx].contains("error")
                    {
                        Color32::from_rgb(220, 80, 80)
                    } else {
                        Color32::from_rgb(0, 220, 120)
                    };
                    ui.label(
                        RichText::new(if state.outputs[idx].is_empty() {
                            "-- press ▶ Run to preview --".into()
                        } else {
                            state.outputs[idx].clone()
                        })
                        .color(out_col)
                        .monospace()
                        .size(11.0),
                    );
                });
        });
}

// ── Floating windows ──────────────────────────────────────────────────
fn draw_floats(
    ctx: &egui::Context,
    state: &mut EriduState,
    bus_state: &Option<std::sync::RwLockReadGuard<BeeMdmBusState>>,
    particles: &[ParticleDemo],
    identities: &[IdentityKaki],
    journal: &Journal,
    snapshots: &[SnapshotRecord],
) {
    if state.float.etl_3d {
        Window::new("⇄ ETL 3D Transparency")
            .open(&mut state.float.etl_3d)
            .resizable(true)
            .default_size([720.0, 500.0])
            .show(ctx, |ui| {
                if let Some(bus) = bus_state {
                    ui.label(
                        RichText::new("ETL Transparency Dashboard")
                            .color(Color32::from_rgb(0, 180, 255))
                            .strong(),
                    );
                    ui.separator();
                    etl_snapshot_view(ui, bus);
                }
            });
    }

    if state.float.beemdm {
        Window::new("⬡ BeeMDM Monitor")
            .open(&mut state.float.beemdm)
            .resizable(true)
            .default_size([600.0, 420.0])
            .show(ctx, |ui| {
                if let Some(bus) = bus_state {
                    beemdm_float_view(ui, bus);
                }
            });
    }

    if state.float.nergal {
        Window::new("𒀭𒉺𒂵 Nergal — IRKALLA")
            .open(&mut state.float.nergal)
            .resizable(true)
            .default_size([680.0, 460.0])
            .show(ctx, |ui| {
                if let Some(bus) = bus_state {
                    nergal_float_view(ui, bus);
                }
            });
    }

    if state.float.hepta_radar {
        Window::new("⬡ Hepta 7D Radar")
            .open(&mut state.float.hepta_radar)
            .resizable(true)
            .default_size([500.0, 480.0])
            .show(ctx, |ui| {
                crate::panels::hepta::draw(ui, particles, &mut state.selected_p);
            });
    }

    if state.float.etl_flow {
        Window::new("⇄ ETL Flow")
            .open(&mut state.float.etl_flow)
            .resizable(true)
            .default_size([600.0, 400.0])
            .show(ctx, |ui| {
                ui.label(
                    RichText::new("ETL Flow — Pipeline Tier View")
                        .color(Color32::from_rgb(0, 160, 255))
                        .strong(),
                );
            });
    }

    if state.float.particles {
        Window::new("◉ Sovereign Particles")
            .open(&mut state.float.particles)
            .resizable(true)
            .default_size([500.0, 420.0])
            .show(ctx, |ui| {
                crate::panels::particles::draw(
                    ui,
                    particles,
                    &mut state.selected_p,
                    &mut crate::panels::InspectorTab::default(),
                    &enkidb_journal::Journal::new(0),
                    &[],
                    &[],
                );
            });
    }

    if state.float.nisaba {
        Window::new("𒀭 NISABA — Sovereign Hologram Agent")
            .open(&mut state.float.nisaba)
            .resizable(true)
            .default_size([460.0, 520.0])
            .frame(
                egui::Frame::window(&ctx.style())
                    .fill(Color32::from_rgb(4, 8, 24))
                    .stroke(Stroke::new(1.5_f32, Color32::from_rgb(0, 200, 255))),
            )
            .show(ctx, |ui| {
                draw_nisaba(
                    ui,
                    &mut state.nisaba,
                    &mut state.nisaba_orch,
                    particles,
                    identities,
                    journal,
                    snapshots,
                );
            });
    }
}

// ── NISABA agent panel ──────────────────────────────────────────────────
fn draw_nisaba(
    ui: &mut Ui,
    nisaba: &mut NisabaState,
    orch: &mut NisabaOrchestrator,
    particles: &[ParticleDemo],
    identities: &[IdentityKaki],
    journal: &Journal,
    snapshots: &[SnapshotRecord],
) {
    ui.label(
        RichText::new("𒀭 NISABA — Internal Interface & TDA Sentinel")
            .color(Color32::from_rgb(0, 220, 255))
            .size(13.0)
            .strong(),
    );
    ui.label(
        RichText::new("Observes, orchestrates, proposes — Architect ratifies (CSR-08)")
            .color(Color32::from_rgb(0, 140, 180))
            .size(10.0),
    );
    ui.separator();

    // ── Live digest, read directly from the real orchestrator ─────────
    let digest = orch.digest();
    ui.horizontal(|ui| {
        stat_mini(
            ui,
            "Steward queue",
            &digest.steward_queue_len.to_string(),
            if digest.steward_queue_len > 0 {
                Color32::from_rgb(220, 80, 80)
            } else {
                theme::TEXT_DIM
            },
        );
        ui.add_space(10.0);
        stat_mini(
            ui,
            "Orbit scans",
            &digest.tribe_readings_count.to_string(),
            theme::CYAN,
        );
        ui.add_space(10.0);
        stat_mini(ui, "Notes", &digest.note_count.to_string(), theme::GOLD);
        ui.add_space(10.0);
        stat_mini(
            ui,
            "Self-resolved",
            &digest.autonomous_resolutions_count.to_string(),
            Color32::from_rgb(0, 220, 120),
        );
    });
    if let Some((env, _class)) = orch.active_autonomy() {
        ui.label(
            RichText::new(format!("𒀭 Autonomy grant active: {env:?}, Watch-band only"))
                .color(Color32::from_rgb(0, 220, 120))
                .size(9.0),
        );
    } else {
        ui.label(
            RichText::new("𒀭 No autonomy grant — fully propose-only")
                .color(theme::TEXT_DIM)
                .size(9.0),
        );
    }
    ui.add_space(4.0);

    ui.horizontal(|ui| {
        if ui.button(RichText::new("🔭 Scan Orbits (TDA)").color(theme::GOLD)).clicked() {
            let cloud: Vec<([u8; 16], f64)> = identities.iter().zip(particles.iter())
                .map(|(id, p)| {
                    let mut bytes = [0u8; 16];
                    bytes.copy_from_slice(id.bytes());
                    (bytes, (1.0 - p.hps as f64).clamp(0.0, 1.0))
                })
                .collect();
            orch.observe_tribe_topology(0x0001, &cloud);
            let narration = orch.notes().last()
                .map(|n| n.narration.clone())
                .unwrap_or_else(|| "No particles to scan.".into());
            nisaba.history.push((String::new(), format!("𒀭 {narration}")));
        }
        if ui.button(RichText::new("📖 Scan StoryEngine (Alerts)").color(theme::GOLD)).clicked() {
            let engine = StoryEngine::new(journal, snapshots);
            let projections: Vec<_> = identities.iter().map(|id| engine.project(id)).collect();
            let raised = orch.observe_story_projections(&projections);
            let summary = if raised == 0 {
                "𒀭 StoryEngine scan: all particles nominal, nothing sent to the Data Steward.".to_string()
            } else {
                let mut lines = vec![format!(
                    "𒀭 StoryEngine scan: {raised} high-priority finding(s) sent to the Data Steward:"
                )];
                for note in orch.notes().iter().rev().take(raised) {
                    lines.push(format!("  · {}", note.narration));
                }
                lines.join("\n")
            };
            nisaba.history.push((String::new(), summary));
        }
    });

    ui.separator();

    egui::ScrollArea::vertical()
        .id_salt("nisaba_history")
        .max_height(300.0)
        .stick_to_bottom(true)
        .show(ui, |ui| {
            for (user_msg, agent_msg) in &nisaba.history {
                if !user_msg.is_empty() {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("You: ").color(theme::GOLD).size(11.0));
                        ui.label(RichText::new(user_msg).color(theme::TEXT_MAIN).size(11.0));
                    });
                    ui.add_space(2.0);
                }
                if !agent_msg.is_empty() {
                    egui::Frame::none()
                        .fill(Color32::from_rgb(4, 16, 28))
                        .rounding(Rounding::same(4.0))
                        .inner_margin(egui::Margin::same(6.0))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new(agent_msg)
                                    .color(Color32::from_rgb(0, 220, 255))
                                    .size(11.0)
                                    .monospace(),
                            );
                        });
                    ui.add_space(4.0);
                }
            }
        });

    ui.separator();

    ui.horizontal(|ui| {
        let input = TextEdit::singleline(&mut nisaba.input)
            .hint_text("Ask about particles, B11, pipeline, alerts, orbits...")
            .desired_width(ui.available_width() - 70.0);
        let resp = ui.add(input);
        let send = ui.button(RichText::new("Ask 𒀭").color(theme::GOLD));

        if (send.clicked() || (resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))))
            && !nisaba.input.is_empty()
        {
            let question = nisaba.input.clone();
            nisaba.input.clear();
            let answer = nisaba_respond(&question, orch);
            nisaba.history.push((question, answer));
        }
    });
}

// ── NISABA sovereign responses ────────────────────────────────────────
// Domain answers (b11, kaki, hepta, way...) stay scripted reference text —
// they describe fixed sovereign constants, not live state. Anything about
// alerts, the steward queue, or orbit/pattern readings answers from the
// real `NisabaOrchestrator` digest instead of a canned string.
fn nisaba_respond(question: &str, orch: &NisabaOrchestrator) -> String {
    let q = question.to_lowercase();
    if q.contains("alert") || q.contains("steward") || q.contains("queue") || q.contains("digest") {
        let digest = orch.digest();
        if digest.steward_queue_len == 0 {
            "𒀭 The Data Steward queue is empty — no high-priority findings raised. \
             Try \"Scan StoryEngine (Alerts)\" to run a fresh pass."
                .into()
        } else {
            format!(
                "𒀭 {} finding(s) await the Data Steward's review.\n\
                 Nothing has been actioned automatically — CSR-08: I propose, you ratify.",
                digest.steward_queue_len,
            )
        }
    } else if q.contains("orbit")
        || q.contains("pattern")
        || q.contains("topology")
        || q.contains("golden") && q.contains("signature")
    {
        match orch.tribe_readings().last() {
            Some(reading) => format!(
                "𒀭 Last orbit scan: {} signature, {} component(s), {} particle(s) sampled.\n\
                 GOLDEN = one loud persistent loop. FUZZY = short-lived loops. DEAD = dust, no loop.",
                reading.signature.label(), reading.component_count, reading.sample_size,
            ),
            None => "𒀭 No orbit scan yet — try \"Scan Orbits (TDA)\" above.".into(),
        }
    } else if q.contains("b11") || q.contains("quality") {
        "B11 is the sovereign quality byte (0–240).\n\
         GEM ≥ 200 (H(P) ≥ 0.833) — Golden Record\n\
         TRIBE ≥ 140 — Near-golden, active\n\
         ACTIVE ≥ 100 — Operational\n\
         FUZZY < 100 — Pending review\n\
         Formula: B11 = round(H(P) × 240)\n\
         Divisor: 240 (Plimpton 322)"
            .into()
    } else if q.contains("kaki") {
        "KAKI is the 16-byte sovereign primary key.\n\
         κ[0..4] minted_id · κ[4..6] tribe_id\n\
         κ[6] kaki_type · κ[7] kaki_role\n\
         κ[12..14] timestamp · κ[14..16] checksum.\n\
         Minted via KakiMinter — never manually assigned."
            .into()
    } else if q.contains("hepta") || q.contains(".hepta") {
        "HeptaScript (.hepta) — W5H2 sovereign query language.\n\
         Seven clauses: WHO WHAT WHERE WHEN WHY HOW HOW_MUCH\n\
         Example:\n\
         WHO   Citizens.E\n\
         WHAT  E[name.given]\n\
         WHERE E[city.name] = \"Najaf\"\n\
         WHY   QUALITY_BYTE > 200\n\
         HOW_MUCH LIMIT 50"
            .into()
    } else if q.contains("nergal")
        || q.contains("threat")
        || q.contains("malware")
        || q.contains("irkalla")
    {
        format!(
            "𒀭𒉺𒂵 Nergal — Sovereign AV Engine.\n\
             Jail: {} (tribe 0x{:04X})\n\
             G7 of the 7 Hepta ETL gates.\n\
             Contained particles are terminal — no unseal path exists.",
            storage_sector::SOVEREIGN_NAME.to_uppercase(),
            storage_sector::IRKALLA_TRIBE,
        )
    } else if q.contains("pipeline") || q.contains("etl") {
        "BeeMDM ETL — 7 Hepta gates (G1..G7), tracked live via\n\
         bee-mdm-bus::GateLoads. G7 is the Nergal threat gate."
            .into()
    } else if q.contains("way") || q.contains("seal") || q.contains("policy") {
        "Way v2.0 (.way) — Sovereign Security Policy Language.\n\
         Structure: sovereign { hepta {} rules {} }\n\
         Hepta weights must sum to 1.0 across 7 dimensions."
            .into()
    } else {
        format!(
            "𒀭 NISABA: I received your query about \"{question}\".\n\
                 Ask me about: b11, kaki, hepta, nergal, irkalla, pipeline, way, alerts, orbits."
        )
    }
}

// ── Sovereign language executor (scripted preview) ─────────────────────
fn run_sovereign_lang(tab: LangTab, source: &str) -> String {
    match tab {
        LangTab::HeptaScript => {
            let has_who = source.contains("WHO");
            let has_what = source.contains("WHAT");
            if !has_who || !has_what {
                return "Error: HeptaScript requires WHO and WHAT clauses.".into();
            }
            "✓ HeptaScript (.hepta) — query parsed (preview, not live execution)\n\
             → This panel does not yet run against a live EnkiDB journal."
                .into()
        }
        LangTab::AkkadianAol => {
            if source.contains("PARTICLE")
                || source.contains("TRIBE")
                || source.contains("RULE")
                || source.contains("PIPELINE")
            {
                "✓ AkkadianAOL (.akk) — source parsed (preview, not live execution)".into()
            } else {
                "Error: No PARTICLE, TRIBE, RULE or PIPELINE found.".into()
            }
        }
        LangTab::Pmpvd => {
            if source.contains("PIPELINE") || source.contains("FLOW") {
                "✓ PMPVD (.pmpvd) — pipeline declaration parsed (preview)".into()
            } else {
                "Error: PMPVD requires PIPELINE or FLOW declaration.".into()
            }
        }
        LangTab::Way => {
            if source.contains("sovereign") && source.contains("hepta") {
                "✓ Way v2.0 (.way) — policy parsed (preview)\n\
                 → Weight sum check requires live SemanticValidator (not wired here)."
                    .into()
            } else {
                "Error: Way v2.0 requires sovereign { } block with hepta { } section.".into()
            }
        }
        LangTab::Template => {
            if source.contains("TEMPLATE") {
                "✓ TemplateEngine (.tpl) — schema parsed (preview)".into()
            } else {
                "Error: Template requires TEMPLATE declaration.".into()
            }
        }
    }
}

// ── Float view helpers ────────────────────────────────────────────────
fn etl_snapshot_view(ui: &mut Ui, bus: &BeeMdmBusState) {
    let cols = [
        (
            "SDB Pending",
            bus.etl.sdb_pending,
            Color32::from_rgb(220, 140, 20),
        ),
        (
            "SDB Promoted",
            bus.etl.sdb_promoted,
            Color32::from_rgb(0, 220, 120),
        ),
        (
            "QDB Quarantine",
            bus.etl.qdb_total,
            Color32::from_rgb(200, 40, 40),
        ),
        (
            "ODB Active",
            bus.etl.odb_total,
            Color32::from_rgb(0, 160, 255),
        ),
        (
            "DW Golden",
            bus.etl.dw_total,
            Color32::from_rgb(200, 160, 40),
        ),
    ];
    ui.columns(cols.len(), |colui| {
        for (i, (label, val, col)) in cols.iter().enumerate() {
            colui[i].vertical_centered(|ui| {
                ui.label(
                    RichText::new(fmt_n(*val as u64))
                        .color(*col)
                        .size(22.0)
                        .strong(),
                );
                ui.label(
                    RichText::new(*label)
                        .color(Color32::from_rgb(140, 140, 160))
                        .size(10.0),
                );
            });
        }
    });
    ui.add_space(8.0);
    ui.label(
        RichText::new(format!(
            "Tick: {}  |  Next sweep: {} ticks  |  Malware hits: {}",
            bus.etl.current_tick, bus.etl.ticks_until_sweep, bus.etl.sdb_malware_hits
        ))
        .color(Color32::from_rgb(100, 100, 140))
        .size(10.0),
    );
}

fn beemdm_float_view(ui: &mut Ui, bus: &BeeMdmBusState) {
    ui.label(
        RichText::new("BeeMDM Pipeline — Gate Loads")
            .color(Color32::from_rgb(0, 220, 120))
            .strong(),
    );
    ui.separator();
    let labels = ["G1", "G2", "G3", "G4", "G5", "G6", "G7 𒀭𒉺𒂵Nergal"];
    let max_v = bus.gates.counts.iter().copied().max().unwrap_or(1).max(1);
    for (i, &label) in labels.iter().enumerate() {
        let n = bus.gates.counts[i];
        let frac = n as f32 / max_v as f32;
        let col = if i == 6 {
            Color32::from_rgb(200, 40, 60)
        } else {
            Color32::from_rgb(0, 180, 100)
        };
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(label)
                    .color(Color32::from_rgb(140, 140, 160))
                    .size(11.0),
            );
            let (bar, _) = ui.allocate_exact_size(Vec2::new(120.0, 12.0), egui::Sense::hover());
            ui.painter()
                .rect_filled(bar, Rounding::same(3.0), Color32::from_rgb(20, 30, 50));
            let fill = egui::Rect::from_min_size(bar.min, Vec2::new(120.0 * frac, 12.0));
            ui.painter().rect_filled(fill, Rounding::same(3.0), col);
            ui.label(
                RichText::new(format!("{n:>3}"))
                    .color(Color32::WHITE)
                    .size(11.0),
            );
        });
    }
    ui.add_space(8.0);
    ui.label(
        RichText::new(format!(
            "Golden rate: {:.1}%  |  Total: {}",
            bus.golden_rate_pct(),
            fmt_n(bus.total_processed)
        ))
        .color(Color32::from_rgb(140, 140, 160))
        .size(10.0),
    );
}

fn nergal_float_view(ui: &mut Ui, bus: &BeeMdmBusState) {
    ui.label(
        RichText::new("𒀭𒉺𒂵 Nergal — IRKALLA Threat Intelligence")
            .color(Color32::from_rgb(180, 0, 40))
            .strong(),
    );
    ui.separator();
    let cats = [
        "Viruses",
        "Ransomware",
        "Trojans",
        "Spyware",
        "Worms",
        "Adware",
        "Rootkits",
    ];
    let total_threats: usize = bus.gates.counts.iter().sum();
    ui.label(
        RichText::new(format!("Total detected: {total_threats}"))
            .color(Color32::from_rgb(220, 40, 60))
            .size(13.0)
            .strong(),
    );
    ui.add_space(6.0);
    ui.columns(4, |cols| {
        for (i, &cat) in cats.iter().enumerate() {
            let n = bus.gates.counts[i];
            cols[i % 4].group(|ui| {
                ui.label(
                    RichText::new(cat)
                        .color(Color32::from_rgb(160, 60, 80))
                        .size(10.0),
                );
                ui.label(
                    RichText::new(format!("{n}"))
                        .color(if n > 0 {
                            Color32::from_rgb(220, 40, 60)
                        } else {
                            Color32::from_rgb(60, 60, 80)
                        })
                        .size(18.0)
                        .strong(),
                );
            });
        }
    });
    ui.add_space(8.0);
    ui.label(
        RichText::new(format!(
            "Jail: {} (tribe 0x{:04X})",
            storage_sector::SOVEREIGN_NAME.to_uppercase(),
            storage_sector::IRKALLA_TRIBE,
        ))
        .color(Color32::from_rgb(100, 40, 60))
        .size(10.0),
    );
}

// ── Sidebar helpers ───────────────────────────────────────────────────
fn sidebar_label(ui: &mut Ui, text: &str) {
    ui.label(
        RichText::new(text)
            .color(theme::TEXT_DIM)
            .size(9.0)
            .strong(),
    );
}

fn mini_panel(ui: &mut Ui, title: &str, desc: &str, col: Color32, open: &mut bool) {
    let is_open = *open;
    let border = if is_open {
        col
    } else {
        Color32::from_rgb(30, 30, 60)
    };
    let fill = if is_open {
        Color32::from_rgba_premultiplied(col.r(), col.g(), col.b(), 18)
    } else {
        Color32::from_rgb(8, 8, 24)
    };

    egui::Frame::none()
        .fill(fill)
        .stroke(Stroke::new(0.8_f32, border))
        .rounding(Rounding::same(5.0))
        .inner_margin(egui::Margin::symmetric(8.0, 6.0))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width() - 4.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new(title).color(col).size(11.0).strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let btn_label = if is_open { "✕" } else { "⊞" };
                    if ui.small_button(btn_label).clicked() {
                        *open = !*open;
                    }
                });
            });
            ui.label(RichText::new(desc).color(theme::TEXT_DIM).size(9.0));
        });
    ui.add_space(4.0);
}

fn stat_mini(ui: &mut Ui, label: &str, value: &str, val_col: Color32) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).color(theme::TEXT_DIM).size(10.0));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(RichText::new(value).color(val_col).size(11.0).strong());
        });
    });
}

fn fmt_n(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out.chars().rev().collect()
}
