//! The governor's face — egui implementation of the AnuGovernor blueprint.
//! Layout mirrors the sealed HTML mockup region for region.

use crate::config::Config;
use crate::docpulse;
use crate::model::*;
use crate::{report, runner};
use crossbeam_channel::{Receiver, Sender};
use eframe::egui::{self, Color32, RichText};
use std::path::PathBuf;

// ── Ishtar palette ──────────────────────────────────────────────
const HONEY: Color32 = Color32::from_rgb(0xE8, 0xA3, 0x3D);
const WAX: Color32 = Color32::from_rgb(0xF2, 0xE8, 0xD5);
const DIM: Color32 = Color32::from_rgb(0xA9, 0xB4, 0xC4);
const OK: Color32 = Color32::from_rgb(0x7F, 0xCB, 0x8F);
const WARN: Color32 = Color32::from_rgb(0xE8, 0xC2, 0x4D);
const ERR: Color32 = Color32::from_rgb(0xE8, 0x6A, 0x5B);
const RUN: Color32 = Color32::from_rgb(0x7F, 0xB4, 0xE8);
const PANEL: Color32 = Color32::from_rgb(0x10, 0x1F, 0x35);
const BG: Color32 = Color32::from_rgb(0x0B, 0x16, 0x26);

/// The governor has two decks: the ansible-playbook Corpus, and the
/// Hala docs Reform & Pulse engine (sovereign name as of Phase 2
/// reconciliation, 2026-08-15; internally still `docpulse`/`uruinimgina-cli`
/// -- see docs/99_index/BAHYWAY_PHASE2_GLOSSARY.md "Hala" entry for why the binary
/// name and config file (`uruinimgina.toml`) were deliberately NOT renamed).
/// Both decks share the queue/error-grid/remedy panels, the halt banner,
/// and the RunnerEvent/Chronicle machinery — only the toolbar's input
/// fields and the stage list differ.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Corpus,
    Hala,
}

pub struct AnuGovernorApp {
    tab: Tab,

    cfg_path: String,
    cfg: Option<Config>,
    params: Vec<(String, String)>,
    new_key: String,
    new_val: String,

    phase: RunPhase,
    statuses: Vec<PbStatus>,
    events: Vec<ErrorEvent>,
    /// Which playbook the Architect is inspecting in the Remedy panel —
    /// settable from either the queue (any PB, including ones with no
    /// event at all) or a row in the errors/warnings grid.
    selected_pb: Option<usize>,
    /// The event actually driving the halt banner's "Apply remedy &
    /// retry" action. Kept separate from `selected_pb` so browsing the
    /// queue while halted can't silently retarget what Retry acts on.
    halt_event: Option<usize>,
    log: Vec<String>,
    fixes: Vec<PathBuf>,

    started: Option<chrono::DateTime<chrono::Utc>>,
    finished: Option<chrono::DateTime<chrono::Utc>>,
    report_path: Option<PathBuf>,
    status_line: String,

    rx: Option<Receiver<RunnerEvent>>,
    ctl_tx: Option<Sender<Ctl>>,

    // ── Hala (docpulse) tab state ──────────────────────────────
    dp_repo: String,
    dp_message: String,
    dp_limit_mb: u64,
    dp_archive: String,
    dp_ingest_dir: String,
    dp_enkiddb_root: String,
    dp_official_repo: String,
    dp_official_subdir: String,
    dp_official_branch: String,
    dp_auto_push_official: bool,

    dp_phase: RunPhase,
    dp_statuses: Vec<PbStatus>,
    dp_events: Vec<ErrorEvent>,
    dp_halt_event: Option<usize>,
    dp_log: Vec<String>,
    dp_started: Option<chrono::DateTime<chrono::Utc>>,
    dp_finished: Option<chrono::DateTime<chrono::Utc>>,

    dp_rx: Option<Receiver<RunnerEvent>>,
    dp_ctl_tx: Option<Sender<Ctl>>,
}

impl AnuGovernorApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = BG;
        visuals.window_fill = PANEL;
        visuals.override_text_color = Some(WAX);
        cc.egui_ctx.set_visuals(visuals);

        let mut app = Self {
            tab: Tab::Corpus,

            cfg_path: "anu-governor.toml".into(),
            cfg: None,
            params: vec![],
            new_key: String::new(),
            new_val: String::new(),
            phase: RunPhase::Idle,
            statuses: vec![],
            events: vec![],
            selected_pb: None,
            halt_event: None,
            log: vec![],
            fixes: vec![],
            started: None,
            finished: None,
            report_path: None,
            status_line: "load a config to begin".into(),
            rx: None,
            ctl_tx: None,

            dp_repo: String::new(),
            dp_message: String::new(),
            dp_limit_mb: 90,
            dp_archive: "~/Forge/_bahyway_large_archive".into(),
            dp_ingest_dir: "chronicle/enkiddb_inbox".into(),
            dp_enkiddb_root: "enkiddb_data".into(),
            dp_official_repo: String::new(),
            dp_official_subdir: "docs/bahyway-v4".into(),
            dp_official_branch: "docs-intake".into(),
            dp_auto_push_official: false,
            dp_phase: RunPhase::Idle,
            dp_statuses: vec![PbStatus::Pending; docpulse::STAGE_NAMES.len()],
            dp_events: vec![],
            dp_halt_event: None,
            dp_log: vec![],
            dp_started: None,
            dp_finished: None,
            dp_rx: None,
            dp_ctl_tx: None,
        };
        app.load_config();
        app
    }

    fn load_config(&mut self) {
        match Config::load(std::path::Path::new(&self.cfg_path)) {
            Ok(cfg) => {
                self.params = cfg
                    .parameters
                    .iter()
                    .map(|p| (p.key.clone(), p.value.clone()))
                    .collect();
                self.statuses = vec![PbStatus::Pending; cfg.run.playbooks.len()];
                self.status_line = format!(
                    "config ✓ {} playbooks resolved{}",
                    cfg.run.playbooks.len(),
                    if cfg.run.simulate { " · SIMULATE mode" } else { "" }
                );
                self.cfg = Some(cfg);
            }
            Err(e) => self.status_line = format!("config error: {e}"),
        }
    }

    fn start_run(&mut self) {
        let Some(cfg) = self.cfg.clone() else { return };
        self.statuses = vec![PbStatus::Pending; cfg.run.playbooks.len()];
        self.events.clear();
        self.fixes.clear();
        self.selected_pb = None;
        self.halt_event = None;
        self.log.clear();
        self.report_path = None;
        self.started = Some(chrono::Utc::now());
        self.finished = None;
        self.phase = RunPhase::Running;

        let (tx, rx) = crossbeam_channel::unbounded();
        let (ctl_tx, ctl_rx) = crossbeam_channel::unbounded();
        runner::spawn_runner(cfg, self.params.clone(), tx, ctl_rx);
        self.rx = Some(rx);
        self.ctl_tx = Some(ctl_tx);
    }

    fn pump_events(&mut self) {
        let Some(rx) = &self.rx else { return };
        let drained: Vec<RunnerEvent> = rx.try_iter().collect();
        for ev in drained {
            match ev {
                RunnerEvent::PbStarted(i) => {
                    if let Some(s) = self.statuses.get_mut(i) {
                        *s = PbStatus::Running;
                    }
                }
                RunnerEvent::PbOk(i) => {
                    if let Some(s) = self.statuses.get_mut(i) {
                        if matches!(self.phase, RunPhase::Halted(h) if h == i) {
                            self.phase = RunPhase::Running;
                        }
                        *s = PbStatus::Ok;
                    }
                }
                RunnerEvent::PbWarned(i, e) => {
                    if let Some(s) = self.statuses.get_mut(i) {
                        *s = PbStatus::Warned;
                    }
                    self.events.push(e);
                }
                RunnerEvent::PbFailed(i, e) => {
                    if let Some(s) = self.statuses.get_mut(i) {
                        *s = PbStatus::Failed;
                    }
                    self.events.push(e);
                    self.halt_event = Some(self.events.len() - 1);
                    self.selected_pb = Some(i);
                    self.phase = RunPhase::Halted(i);
                }
                RunnerEvent::Log(l) => self.log.push(l),
                RunnerEvent::Finished => {
                    self.phase = RunPhase::Finished;
                    self.finished = Some(chrono::Utc::now());
                }
                RunnerEvent::Aborted => {
                    self.phase = RunPhase::Aborted;
                    self.finished = Some(chrono::Utc::now());
                }
            }
        }
    }

    fn generate_report(&mut self) {
        let Some(cfg) = &self.cfg else { return };
        let run = report::RunSummary {
            started: self.started.unwrap_or_else(chrono::Utc::now),
            finished: self.finished.unwrap_or_else(chrono::Utc::now),
            playbooks: &cfg.run.playbooks,
            statuses: &self.statuses,
            events: &self.events,
            parameters: &self.params,
            fixes_generated: &self.fixes,
        };
        match report::generate(&cfg.run.report_dir, &cfg.run.seal_key, &run) {
            Ok(p) => {
                self.status_line = format!("𒁾 sealed report: {}", p.display());
                self.report_path = Some(p);
            }
            Err(e) => self.status_line = format!("report error: {e}"),
        }
    }

    // ── Hala (docpulse) ───────────────────────────────────────────

    fn start_docpulse(&mut self) {
        self.dp_statuses = vec![PbStatus::Pending; docpulse::STAGE_NAMES.len()];
        self.dp_events.clear();
        self.dp_halt_event = None;
        self.dp_log.clear();
        self.dp_started = Some(chrono::Utc::now());
        self.dp_finished = None;
        self.dp_phase = RunPhase::Running;

        let chronicle_dir = self
            .cfg
            .as_ref()
            .map(|c| c.run.chronicle_dir.clone())
            .unwrap_or_else(|| "chronicle".into());
        let cfg = docpulse::DocPulseCfg {
            repo_path: self.dp_repo.clone(),
            message: self.dp_message.clone(),
            limit_mb: self.dp_limit_mb,
            archive_dir: self.dp_archive.clone(),
            ingest_manifest_dir: self.dp_ingest_dir.clone(),
            chronicle_dir,
            enkiddb_output_root: self.dp_enkiddb_root.clone(),
            official_repo_path: self.dp_official_repo.clone(),
            official_repo_subdir: self.dp_official_subdir.clone(),
            official_repo_branch: self.dp_official_branch.clone(),
            auto_push_to_official_repo: self.dp_auto_push_official,
            // GUI keeps today's single-corpus behavior; the new
            // per-tribe override (architect_docs vs. official docs) is
            // exposed via uruinimgina-cli's TOML config, not this tab
            // yet -- no new GUI field added for a CLI-driven request.
            docs_tribe_id: enkiddb::DOCS_TRIBE_ID,
        };

        let (tx, rx) = crossbeam_channel::unbounded();
        let (ctl_tx, ctl_rx) = crossbeam_channel::unbounded();
        docpulse::spawn_docpulse(cfg, tx, ctl_rx);
        self.dp_rx = Some(rx);
        self.dp_ctl_tx = Some(ctl_tx);
    }

    fn pump_dp_events(&mut self) {
        let Some(rx) = &self.dp_rx else { return };
        let drained: Vec<RunnerEvent> = rx.try_iter().collect();
        for ev in drained {
            match ev {
                RunnerEvent::PbStarted(i) => {
                    if let Some(s) = self.dp_statuses.get_mut(i) {
                        *s = PbStatus::Running;
                    }
                }
                RunnerEvent::PbOk(i) => {
                    if let Some(s) = self.dp_statuses.get_mut(i) {
                        if matches!(self.dp_phase, RunPhase::Halted(h) if h == i) {
                            self.dp_phase = RunPhase::Running;
                        }
                        *s = PbStatus::Ok;
                    }
                }
                RunnerEvent::PbWarned(i, e) => {
                    if let Some(s) = self.dp_statuses.get_mut(i) {
                        *s = PbStatus::Warned;
                    }
                    self.dp_events.push(e);
                }
                RunnerEvent::PbFailed(i, e) => {
                    if let Some(s) = self.dp_statuses.get_mut(i) {
                        *s = PbStatus::Failed;
                    }
                    self.dp_events.push(e);
                    self.dp_halt_event = Some(self.dp_events.len() - 1);
                    self.dp_phase = RunPhase::Halted(i);
                }
                RunnerEvent::Log(l) => self.dp_log.push(l),
                RunnerEvent::Finished => {
                    self.dp_phase = RunPhase::Finished;
                    self.dp_finished = Some(chrono::Utc::now());
                }
                RunnerEvent::Aborted => {
                    self.dp_phase = RunPhase::Aborted;
                    self.dp_finished = Some(chrono::Utc::now());
                }
            }
        }
    }
}

/// Border-drag resize: with_decorations(false) means there's no OS resize
/// border either, so this makes the left/right/bottom edges and both
/// bottom corners grabbable (the top edge is left alone — the self-drawn
/// title bar already owns move/maximize/close there, and its buttons
/// sit close enough to that edge that a resize zone would fight them).
fn handle_border_resize(ctx: &egui::Context) {
    const MARGIN: f32 = 6.0;
    let rect = ctx.input(|i| i.screen_rect());
    let Some(pos) = ctx.input(|i| i.pointer.interact_pos()) else { return };

    let near_left = pos.x <= rect.left() + MARGIN;
    let near_right = pos.x >= rect.right() - MARGIN;
    let near_bottom = pos.y >= rect.bottom() - MARGIN;
    if !(near_left || near_right || near_bottom) {
        return;
    }
    use egui::{CursorIcon, ResizeDirection};
    let dir = match (near_left, near_right, near_bottom) {
        (true, _, true) => ResizeDirection::SouthWest,
        (_, true, true) => ResizeDirection::SouthEast,
        (true, false, false) => ResizeDirection::West,
        (false, true, false) => ResizeDirection::East,
        _ => ResizeDirection::South,
    };
    let cursor = match dir {
        ResizeDirection::SouthWest => CursorIcon::ResizeSouthWest,
        ResizeDirection::SouthEast => CursorIcon::ResizeSouthEast,
        ResizeDirection::West => CursorIcon::ResizeWest,
        ResizeDirection::East => CursorIcon::ResizeEast,
        _ => CursorIcon::ResizeSouth,
    };
    ctx.output_mut(|o| o.cursor_icon = cursor);
    if ctx.input(|i| i.pointer.primary_pressed()) {
        ctx.send_viewport_cmd(egui::ViewportCommand::BeginResize(dir));
    }
}

fn phase_badge(phase: RunPhase) -> (String, Color32) {
    match phase {
        RunPhase::Idle => ("IDLE".to_string(), DIM),
        RunPhase::Running => ("RUNNING".to_string(), RUN),
        RunPhase::Halted(i) => (
            format!("⏸ HALTED — MAJOR at stage {} · awaiting Architect (CSR-08)", i + 1),
            ERR,
        ),
        RunPhase::Finished => ("FINISHED".to_string(), OK),
        RunPhase::Aborted => ("ABORTED".to_string(), ERR),
    }
}

fn status_color(st: PbStatus) -> Color32 {
    match st {
        PbStatus::Ok => OK,
        PbStatus::Warned => WARN,
        PbStatus::Failed => ERR,
        PbStatus::Running => RUN,
        PbStatus::Skipped => DIM,
        PbStatus::Pending => Color32::from_rgb(0x5A, 0x66, 0x78),
    }
}

impl eframe::App for AnuGovernorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.pump_events();
        self.pump_dp_events();
        if matches!(self.phase, RunPhase::Running | RunPhase::Halted(_))
            || matches!(self.dp_phase, RunPhase::Running | RunPhase::Halted(_))
        {
            ctx.request_repaint_after(std::time::Duration::from_millis(80));
        }
        handle_border_resize(ctx);

        // ── title bar (self-drawn: with_decorations(false) means the OS/
        // compositor gives us none — drag-to-move, minimize, maximize and
        // close all have to live here) ──
        egui::TopBottomPanel::top("title").exact_height(32.0).show(ctx, |ui| {
            let bar_rect = ui.max_rect();
            let bar_resp = ui.interact(bar_rect, egui::Id::new("anu_governor_titlebar_drag"), egui::Sense::click_and_drag());
            let is_maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
            if bar_resp.double_clicked() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!is_maximized));
            } else if bar_resp.drag_started() {
                ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
            }

            ui.horizontal(|ui| {
                ui.label(RichText::new("𒀭 AnuGovernor").size(19.0).color(WAX));
                ui.add_space(10.0);
                let (txt, col) = match self.tab {
                    Tab::Corpus => phase_badge(self.phase),
                    Tab::Hala => phase_badge(self.dp_phase),
                };
                ui.label(RichText::new(txt).color(col).monospace());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let win_btn = |ui: &mut egui::Ui, glyph: &str, color: Color32| {
                        ui.add(
                            egui::Button::new(RichText::new(glyph).color(color).monospace().size(14.0))
                                .frame(false)
                                .min_size(egui::vec2(24.0, 24.0)),
                        )
                    };
                    if win_btn(ui, "✕", ERR).on_hover_text("Close").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    if win_btn(ui, if is_maximized { "❐" } else { "□" }, WAX)
                        .on_hover_text(if is_maximized { "Restore" } else { "Maximize" })
                        .clicked()
                    {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!is_maximized));
                    }
                    if win_btn(ui, "—", WAX).on_hover_text("Minimize").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                    }
                    // Dragging a border can leave the window an awkward
                    // size with no way back — this snaps it to the app's
                    // designed default (must match main.rs's with_inner_size).
                    if win_btn(ui, "⟲", WAX).on_hover_text("Reset window size").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(false));
                        // Must match main.rs's with_inner_size.
                        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(900.0, 620.0)));
                    }
                    ui.add_space(10.0);
                    ui.label(RichText::new(&self.status_line).color(DIM).small());
                });
            });
        });

        // ── tab bar ──
        egui::TopBottomPanel::top("tabs").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.tab, Tab::Corpus, "Corpus 𒑐");
                ui.selectable_value(&mut self.tab, Tab::Hala, "Hala 𒄩𒆷");
            });
        });

        match self.tab {
            Tab::Corpus => self.show_corpus_tab(ctx),
            Tab::Hala => self.show_hala_tab(ctx),
        }
    }
}

impl AnuGovernorApp {
    fn show_corpus_tab(&mut self, ctx: &egui::Context) {
        // ── toolbar + params ──
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("config").color(HONEY).small());
                ui.add(egui::TextEdit::singleline(&mut self.cfg_path).desired_width(220.0));
                if ui.button("Load").clicked() {
                    self.load_config();
                }
                ui.separator();
                let can_run = self.cfg.is_some() && matches!(self.phase, RunPhase::Idle | RunPhase::Finished | RunPhase::Aborted);
                if ui.add_enabled(can_run, egui::Button::new(RichText::new("▶ Run").color(Color32::BLACK)).fill(HONEY)).clicked() {
                    self.start_run();
                }
                let running = matches!(self.phase, RunPhase::Running | RunPhase::Halted(_));
                if ui.add_enabled(running, egui::Button::new(RichText::new("■ Abort").color(ERR))).clicked() {
                    if let Some(t) = &self.ctl_tx { let _ = t.send(Ctl::Abort); }
                }
            });
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("PARAMETERS").color(HONEY).small().monospace());
                let mut remove: Option<usize> = None;
                for (i, (k, v)) in self.params.iter().enumerate() {
                    ui.label(RichText::new(format!("{k} = {v}")).monospace().small().color(WAX)
                        .background_color(PANEL));
                    if ui.small_button(RichText::new("×").color(ERR)).clicked() {
                        remove = Some(i);
                    }
                }
                if let Some(i) = remove { self.params.remove(i); }
                ui.add(egui::TextEdit::singleline(&mut self.new_key).hint_text("key").desired_width(90.0));
                ui.add(egui::TextEdit::singleline(&mut self.new_val).hint_text("value").desired_width(130.0));
                if ui.button("+ add").clicked() && !self.new_key.is_empty() {
                    self.params.push((std::mem::take(&mut self.new_key), std::mem::take(&mut self.new_val)));
                }
            });
        });

        // ── progress ──
        egui::TopBottomPanel::top("progress").show(ctx, |ui| {
            let n = self.statuses.len().max(1);
            let done = self.statuses.iter().filter(|s| !matches!(s, PbStatus::Pending | PbStatus::Running)).count();
            let ok = self.statuses.iter().filter(|s| **s == PbStatus::Ok).count();
            let w = self.statuses.iter().filter(|s| **s == PbStatus::Warned).count();
            let f = self.statuses.iter().filter(|s| **s == PbStatus::Failed).count();
            let cur = self.statuses.iter().position(|s| *s == PbStatus::Running);
            ui.horizontal(|ui| {
                let name = cur
                    .and_then(|i| self.cfg.as_ref().map(|c| c.run.playbooks[i].clone()))
                    .unwrap_or_default();
                ui.label(RichText::new(format!("PB {}/{} {}", done, n, name)).monospace().small().color(WAX));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(format!("ok {ok} · warnings {w} · errors {f}")).monospace().small().color(DIM));
                });
            });
            ui.add(egui::ProgressBar::new(done as f32 / n as f32).desired_width(f32::INFINITY).fill(HONEY));

            // ── halt banner: the law rendered ──
            if let RunPhase::Halted(i) = self.phase {
                ui.add_space(4.0);
                egui::Frame::none().fill(Color32::from_rgb(0x2A, 0x15, 0x12)).inner_margin(8.0).show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new(format!(
                            "MAJOR · PB {} — execution halted by law. Warnings proceed; majors await the Architect.", i + 1
                        )).color(ERR).monospace().small());
                        if ui.button("Skip & continue").clicked() {
                            if let Some(s) = self.statuses.get_mut(i) { *s = PbStatus::Skipped; }
                            self.phase = RunPhase::Running;
                            if let Some(t) = &self.ctl_tx { let _ = t.send(Ctl::SkipContinue); }
                        }
                        let can_fix = self.halt_event
                            .and_then(|e| self.events.get(e))
                            .map(|e| e.remedy_id.is_some())
                            .unwrap_or(false);
                        if ui.add_enabled(can_fix, egui::Button::new(
                            RichText::new("Apply remedy playbook & retry").color(Color32::BLACK)).fill(HONEY)).clicked()
                        {
                            if let (Some(cfg), Some(sel)) = (&self.cfg, self.halt_event) {
                                if let Some(ev) = self.events.get(sel) {
                                    match runner::generate_fix(cfg, ev) {
                                        Ok(p) => {
                                            self.log.push(format!("𒁾 generated fix playbook: {}", p.display()));
                                            self.fixes.push(p);
                                            if let Some(t) = &self.ctl_tx { let _ = t.send(Ctl::Retry); }
                                        }
                                        Err(e) => self.log.push(format!("remedy error: {e}")),
                                    }
                                }
                            }
                        }
                    });
                });
            }
        });

        // ── footer: report ──
        egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let done = matches!(self.phase, RunPhase::Finished | RunPhase::Aborted);
                if ui.add_enabled(done, egui::Button::new(
                    RichText::new("Generate final report 𒁾").color(Color32::BLACK)).fill(HONEY)).clicked()
                {
                    self.generate_report();
                }
                if let Some(p) = &self.report_path {
                    ui.label(RichText::new(format!("sealed → {}", p.display())).monospace().small().color(OK));
                } else {
                    ui.label(RichText::new("run summary · per-PB outcomes · errors & remedies · parameter manifest · AkkadianSeal")
                        .small().color(DIM));
                }
                // No OS-drawn resize border under with_decorations(false) —
                // this grip is the only way to shrink/grow the window.
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let grip = ui.add(
                        egui::Label::new(RichText::new("⇲").color(DIM).monospace().size(15.0))
                            .sense(egui::Sense::drag()),
                    ).on_hover_text("Drag to resize");
                    if grip.drag_started() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::BeginResize(egui::ResizeDirection::SouthEast));
                    }
                });
            });
            if let Some(last) = self.log.last() {
                ui.label(RichText::new(last).monospace().small().color(DIM));
            }
        });

        // ── left: queue ──
        egui::SidePanel::left("queue").min_width(240.0).show(ctx, |ui| {
            ui.label(RichText::new("Playbook queue").color(WAX).size(15.0));
            ui.separator();
            egui::ScrollArea::vertical().id_salt("queue_scroll").show(ui, |ui| {
                if let Some(cfg) = &self.cfg {
                    for (i, pb) in cfg.run.playbooks.iter().enumerate() {
                        let st = self.statuses.get(i).copied().unwrap_or(PbStatus::Pending);
                        let col = status_color(st);
                        let text = RichText::new(format!("{} {}", st.icon(), pb)).monospace().small().color(col);
                        if ui.selectable_label(self.selected_pb == Some(i), text).clicked() {
                            self.selected_pb = Some(i);
                        }
                    }
                }
            });
        });

        // ── right: remedy ──
        egui::SidePanel::right("remedy").min_width(300.0).show(ctx, |ui| {
            ui.label(RichText::new("Remedy").color(WAX).size(15.0));
            ui.separator();
            let Some(pb_idx) = self.selected_pb else {
                ui.label(RichText::new("select a playbook in the queue or an event in the grid").color(DIM).small());
                return;
            };
            let Some(ev) = self.events.iter().find(|e| e.pb_index == pb_idx).cloned() else {
                let st = self.statuses.get(pb_idx).copied().unwrap_or(PbStatus::Pending);
                let name = self.cfg.as_ref()
                    .and_then(|c| c.run.playbooks.get(pb_idx))
                    .cloned()
                    .unwrap_or_default();
                ui.label(RichText::new(format!("{} {}", st.icon(), name)).monospace().small().color(WAX));
                ui.add_space(4.0);
                let msg = match st {
                    PbStatus::Skipped => "Skipped by the Architect at a MAJOR halt — no error/warning was recorded for this row itself; check the PB that actually halted execution for the remedy.",
                    PbStatus::Ok => "Completed clean — no error or warning was recorded for this playbook.",
                    PbStatus::Pending => "Not yet reached in the queue.",
                    PbStatus::Running => "Currently running.",
                    PbStatus::Failed | PbStatus::Warned => "No event on record for this playbook (unexpected — status/events are out of sync).",
                };
                ui.label(RichText::new(msg).small().color(DIM));
                return;
            };
            ui.label(RichText::new(format!("{} — {}", ev.severity, ev.playbook)).color(
                if ev.severity == Severity::Major { ERR } else { WARN }).monospace().small());
            ui.add_space(6.0);
            let remedy_rule = self.cfg.as_ref().and_then(|c| c.find_remedy(&ev.message).cloned());
            match remedy_rule {
                Some(rule) => {
                    ui.label(RichText::new("Diagnosis").color(HONEY));
                    ui.label(RichText::new(&rule.diagnosis).small());
                    ui.add_space(4.0);
                    ui.label(RichText::new("How to solve").color(HONEY));
                    ui.label(RichText::new(&rule.solution).small());
                    ui.add_space(6.0);
                    if ui.button(RichText::new(format!("Generate fix playbook ({})", rule.id))).clicked() {
                        if let Some(cfg) = &self.cfg {
                            match runner::generate_fix(cfg, &ev) {
                                Ok(p) => {
                                    self.log.push(format!("𒁾 fix written: {}", p.display()));
                                    self.fixes.push(p);
                                }
                                Err(e) => self.log.push(format!("remedy error: {e}")),
                            }
                        }
                    }
                    ui.add_space(6.0);
                    ui.label(RichText::new("Sealed on generation · running it = CSR-08 · chronicled")
                        .color(DIM).small());
                }
                None => {
                    ui.label(RichText::new("No rule in the knowledge base matches this message.").small());
                    ui.label(RichText::new("Solve it once — then add a [[remedy]] rule so AnuGovernor remembers forever.")
                        .color(DIM).small());
                }
            }
            ui.separator();
            ui.label(RichText::new("Message").color(HONEY).small());
            ui.label(RichText::new(&ev.message).monospace().small().color(DIM));
        });

        // ── center: errors & warnings grid ──
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label(RichText::new(format!("Errors & warnings — {} events", self.events.len())).color(WAX).size(15.0));
            ui.separator();
            use egui_extras::{Column, TableBuilder};
            TableBuilder::new(ui)
                .striped(true)
                .column(Column::exact(40.0))
                .column(Column::remainder())
                .column(Column::exact(70.0))
                .column(Column::exact(150.0))
                .sense(egui::Sense::click())
                .header(20.0, |mut h| {
                    for t in ["PB", "TASK", "SEV", "ERROR TYPE"] {
                        h.col(|ui| { ui.label(RichText::new(t).color(HONEY).monospace().small()); });
                    }
                })
                .body(|mut body| {
                    for ev in self.events.iter() {
                        body.row(20.0, |mut row| {
                            row.set_selected(self.selected_pb == Some(ev.pb_index));
                            row.col(|ui| { ui.label(RichText::new(format!("{}", ev.pb_index + 1)).monospace().small()); });
                            row.col(|ui| { ui.label(RichText::new(&ev.task).monospace().small()); });
                            row.col(|ui| {
                                let c = if ev.severity == Severity::Major { ERR } else { WARN };
                                ui.label(RichText::new(ev.severity.to_string()).color(c).monospace().small());
                            });
                            row.col(|ui| { ui.label(RichText::new(&ev.error_type).monospace().small()); });
                            if row.response().clicked() {
                                self.selected_pb = Some(ev.pb_index);
                            }
                        });
                    }
                });
        });
    }

    fn show_hala_tab(&mut self, ctx: &egui::Context) {
        // ── toolbar: the DocPulseCfg input fields ──
        egui::TopBottomPanel::top("dp_toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("repo").color(HONEY).small());
                ui.add(egui::TextEdit::singleline(&mut self.dp_repo).hint_text("~/Forge/bahyway_v4_docs").desired_width(220.0));
                ui.label(RichText::new("message").color(HONEY).small());
                ui.add(egui::TextEdit::singleline(&mut self.dp_message).desired_width(260.0));
            });
            ui.horizontal(|ui| {
                ui.label(RichText::new("limit MB").color(HONEY).small());
                let mut limit_str = self.dp_limit_mb.to_string();
                if ui.add(egui::TextEdit::singleline(&mut limit_str).desired_width(60.0)).changed() {
                    if let Ok(v) = limit_str.parse::<u64>() { self.dp_limit_mb = v; }
                }
                ui.label(RichText::new("archive").color(HONEY).small());
                ui.add(egui::TextEdit::singleline(&mut self.dp_archive).desired_width(260.0));
                ui.separator();
                let can_run = !self.dp_repo.trim().is_empty()
                    && matches!(self.dp_phase, RunPhase::Idle | RunPhase::Finished | RunPhase::Aborted);
                if ui.add_enabled(can_run, egui::Button::new(RichText::new("▶ Reform & Pulse").color(Color32::BLACK)).fill(HONEY)).clicked() {
                    self.start_docpulse();
                }
            });
            ui.horizontal(|ui| {
                ui.label(RichText::new("EnkiDDB root").color(HONEY).small());
                ui.add(egui::TextEdit::singleline(&mut self.dp_enkiddb_root).desired_width(160.0));
                ui.label(RichText::new("official repo").color(HONEY).small());
                ui.add(egui::TextEdit::singleline(&mut self.dp_official_repo).hint_text("empty = skip landing stage").desired_width(220.0));
                ui.label(RichText::new("subdir").color(HONEY).small());
                ui.add(egui::TextEdit::singleline(&mut self.dp_official_subdir).desired_width(140.0));
                ui.label(RichText::new("branch").color(HONEY).small());
                ui.add(egui::TextEdit::singleline(&mut self.dp_official_branch).desired_width(120.0));
                ui.checkbox(&mut self.dp_auto_push_official, "push")
                    .on_hover_text("Off by default: stages + commits locally on the branch above but never pushes. \
                        Turn on only when you're ready for this run to push that branch to origin — never main.");
                let running = matches!(self.dp_phase, RunPhase::Running | RunPhase::Halted(_));
                if ui.add_enabled(running, egui::Button::new(RichText::new("■ Abort").color(ERR))).clicked() {
                    if let Some(t) = &self.dp_ctl_tx { let _ = t.send(Ctl::Abort); }
                }
            });
        });

        // ── progress + halt banner ──
        egui::TopBottomPanel::top("dp_progress").show(ctx, |ui| {
            let n = self.dp_statuses.len().max(1);
            let done = self.dp_statuses.iter().filter(|s| !matches!(s, PbStatus::Pending | PbStatus::Running)).count();
            let cur = self.dp_statuses.iter().position(|s| *s == PbStatus::Running);
            ui.horizontal(|ui| {
                let name = cur.map(|i| docpulse::STAGE_NAMES[i]).unwrap_or_default();
                ui.label(RichText::new(format!("Stage {}/{} {}", done, n, name)).monospace().small().color(WAX));
            });
            ui.add(egui::ProgressBar::new(done as f32 / n as f32).desired_width(f32::INFINITY).fill(HONEY));

            // The blob audit is a MAJOR by design (CSR-08): the pulse halts
            // and the Architect rules. Skip is not offered meaningfully here
            // (the outgoing history still contains the oversized blob) —
            // only Retry-after-manual-amend or Abort.
            if let RunPhase::Halted(i) = self.dp_phase {
                ui.add_space(4.0);
                egui::Frame::none().fill(Color32::from_rgb(0x2A, 0x15, 0x12)).inner_margin(8.0).show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        let msg = self.dp_halt_event.and_then(|e| self.dp_events.get(e)).map(|e| e.message.clone())
                            .unwrap_or_else(|| format!("MAJOR at stage {}", i + 1));
                        ui.label(RichText::new(format!("MAJOR · {msg}")).color(ERR).monospace().small());
                        if ui.button("Retry after amend").clicked() {
                            if let Some(t) = &self.dp_ctl_tx { let _ = t.send(Ctl::Retry); }
                        }
                        if ui.button(RichText::new("Abort pulse").color(ERR)).clicked() {
                            if let Some(t) = &self.dp_ctl_tx { let _ = t.send(Ctl::Abort); }
                        }
                    });
                });
            }
        });

        // ── footer ──
        egui::TopBottomPanel::bottom("dp_footer").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let done = matches!(self.dp_phase, RunPhase::Finished | RunPhase::Aborted);
                if ui.add_enabled(done, egui::Button::new(
                    RichText::new("Generate final report 𒁾").color(Color32::BLACK)).fill(HONEY)).clicked()
                {
                    self.status_line = "Hala report generation reuses the Corpus report engine \
                        once reform/ingest outcomes are folded into RunSummary".into();
                }
                ui.label(RichText::new("reform manifest · per-stage outcomes · flagged blobs · ingestion inbox summary")
                    .small().color(DIM));
            });
            if let Some(last) = self.dp_log.last() {
                ui.label(RichText::new(last).monospace().small().color(DIM));
            }
        });

        // ── left: stage queue (same visual language as the Corpus queue) ──
        egui::SidePanel::left("dp_queue").min_width(240.0).show(ctx, |ui| {
            ui.label(RichText::new("Reform stages").color(WAX).size(15.0));
            ui.separator();
            egui::ScrollArea::vertical().id_salt("dp_queue_scroll").show(ui, |ui| {
                for (i, name) in docpulse::STAGE_NAMES.iter().enumerate() {
                    let st = self.dp_statuses.get(i).copied().unwrap_or(PbStatus::Pending);
                    let col = status_color(st);
                    let text = RichText::new(format!("{} {} · {}", st.icon(), i, name)).monospace().small().color(col);
                    ui.label(text);
                }
            });
        });

        // ── right: events (blob audit / commit warnings land here) ──
        egui::SidePanel::right("dp_events").min_width(300.0).show(ctx, |ui| {
            ui.label(RichText::new("Events").color(WAX).size(15.0));
            ui.separator();
            if self.dp_events.is_empty() {
                ui.label(RichText::new("no warnings or errors yet").color(DIM).small());
            }
            for ev in &self.dp_events {
                let c = if ev.severity == Severity::Major { ERR } else { WARN };
                ui.label(RichText::new(format!("{} · {}", ev.severity, ev.error_type)).color(c).monospace().small());
                ui.label(RichText::new(&ev.message).small().color(DIM));
                ui.add_space(4.0);
            }
        });

        // ── center: log (reform manifest / blob audit / ingest manifest lines) ──
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label(RichText::new("Log — reform, audit, pulse, ingestion").color(WAX).size(15.0));
            ui.separator();
            egui::ScrollArea::vertical().id_salt("dp_log_scroll").stick_to_bottom(true).show(ui, |ui| {
                for line in &self.dp_log {
                    ui.label(RichText::new(line).monospace().small().color(WAX));
                }
            });
        });
    }
}
