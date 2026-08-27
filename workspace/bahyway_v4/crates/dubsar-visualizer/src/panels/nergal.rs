//! 𒀭𒉺𒂵 Nergal AV Panel — DubSar Visualizer sovereign security view.
//!
//! Reads the real G7 Nergal gate (`bee_mdm_bus::GateLoads::g7_nergal()`,
//! the 7th of the 7 Hepta ETL gates) plus the real SDB malware-hit and
//! quarantine counters — no invented data.
//!
//! Palette: MAROON #800020 sovereign threat colours (matches the
//! DEAD_EXPIRED/#404040 vs NERGAL-AV/#800000 colour separation already
//! sealed in utnapishtim's ColorID law — dead data and jailed threats
//! are never the same colour).
//!
//! DUB.SAR 𒁾 — BahyWay.Ecosystem v4.0 | Pure Rust | eframe 0.29

use bee_mdm_bus::BeeMdmBusState;
use egui::{Color32, RichText, Rounding, Stroke, Ui};
use std::sync::{Arc, RwLock};

// ── Nergal sovereign colour palette ──────────────────────────────────
const COL_MAROON: Color32 = Color32::from_rgb(128, 0, 32);
const COL_DIM: Color32 = Color32::from_rgb(140, 100, 110);
const COL_RED: Color32 = Color32::from_rgb(220, 40, 60);
const COL_GREEN: Color32 = Color32::from_rgb(0, 220, 120);
const COL_AMBER: Color32 = Color32::from_rgb(220, 140, 20);

// 7 threat category colours — one per Hepta ETL gate (G1..G7)
const CAT_COLS: [Color32; 7] = [
    Color32::from_rgb(200, 30, 60), // G1 Computer Viruses
    Color32::from_rgb(220, 80, 10), // G2 Ransomware
    Color32::from_rgb(180, 20, 80), // G3 Trojan Horses
    Color32::from_rgb(160, 0, 50),  // G4 Spyware & Keyloggers
    Color32::from_rgb(210, 100, 0), // G5 Network Worms
    Color32::from_rgb(180, 50, 10), // G6 Adware & PUPs
    Color32::from_rgb(128, 0, 32),  // G7 Rootkits (Nergal) — MAROON
];

const CAT_NAMES: [&str; 7] = [
    "Viruses",
    "Ransomware",
    "Trojans",
    "Spyware",
    "Worms",
    "Adware",
    "Rootkits",
];

/// Entry point — the Nergal AV dashboard panel.
pub fn draw(ui: &mut Ui, bus: &Arc<RwLock<BeeMdmBusState>>) {
    let state = match bus.read() {
        Ok(s) => s,
        Err(_) => {
            ui.label(RichText::new("Bus read error").color(COL_RED));
            return;
        }
    };

    // ── Title ──────────────────────────────────────────────────────
    ui.add_space(8.0);
    ui.label(
        RichText::new("𒀭𒉺𒂵  Nergal — Sovereign Threat Intelligence")
            .color(COL_MAROON)
            .size(16.0)
            .strong(),
    );
    ui.label(
        RichText::new(format!(
            "G7 Nergal gate load: {}  ·  Jail: {} (tribe 0x{:04X})",
            state.gates.g7_nergal(),
            storage_sector::SOVEREIGN_NAME.to_uppercase(),
            storage_sector::IRKALLA_TRIBE,
        ))
        .color(COL_DIM)
        .size(11.0),
    );
    ui.add_space(8.0);
    ui.separator();
    ui.add_space(8.0);

    // ── Stats row ─────────────────────────────────────────────────
    let threats: usize = state.gates.total_in_flight();
    let threat_col = if threats > 0 { COL_RED } else { COL_GREEN };
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!("Total gate load: {threats}"))
                .color(threat_col)
                .size(14.0)
                .strong(),
        );
        ui.separator();
        ui.label(
            RichText::new(format!("Malware hits: {}", state.etl.sdb_malware_hits))
                .color(if state.etl.sdb_malware_hits > 0 {
                    COL_RED
                } else {
                    COL_DIM
                })
                .size(12.0),
        );
        ui.separator();
        ui.label(
            RichText::new(format!("Jailed (IRKALLA): {}", state.total_quarantined))
                .color(COL_AMBER)
                .size(12.0),
        );
    });
    ui.add_space(12.0);

    // ── 7 gate category grid ──────────────────────────────────────
    ui.columns(4, |cols| {
        for (i, cat) in CAT_NAMES.iter().enumerate() {
            let col_i = i % 4;
            let n = state.gates.counts[i];
            let col = CAT_COLS[i];
            cols[col_i].group(|ui| {
                egui::Frame::none()
                    .fill(Color32::from_rgb(20, 4, 8))
                    .stroke(Stroke::new(1.0_f32, col))
                    .rounding(Rounding::same(4.0))
                    .inner_margin(egui::Margin::same(8.0))
                    .show(ui, |ui| {
                        ui.label(RichText::new(*cat).color(col).size(11.0));
                        ui.label(
                            RichText::new(format!("{n}"))
                                .color(if n > 0 {
                                    col
                                } else {
                                    Color32::from_rgb(40, 20, 25)
                                })
                                .size(24.0)
                                .strong(),
                        );
                    });
            });
        }
    });

    ui.add_space(16.0);
    ui.separator();
    ui.add_space(8.0);

    // ── Action buttons ────────────────────────────────────────────
    ui.horizontal(|ui| {
        if threats > 0
            && ui
                .add(
                    egui::Button::new(
                        RichText::new("Jail All Threats")
                            .color(Color32::BLACK)
                            .size(12.0)
                            .strong(),
                    )
                    .fill(COL_MAROON)
                    .rounding(Rounding::same(4.0))
                    .min_size(egui::Vec2::new(150.0, 32.0)),
                )
                .clicked()
        {
            let _ = bus
                .write()
                .map(|s| s.send_cmd(bee_mdm_bus::BusCmd::ForceSweep));
        }
        if ui
            .add(
                egui::Button::new(
                    RichText::new("Accelerate Scan")
                        .color(Color32::WHITE)
                        .size(12.0),
                )
                .fill(Color32::from_rgb(160, 10, 30))
                .rounding(Rounding::same(4.0))
                .min_size(egui::Vec2::new(150.0, 32.0)),
            )
            .clicked()
        {
            let _ = bus
                .write()
                .map(|s| s.send_cmd(bee_mdm_bus::BusCmd::AccelerateIntake));
        }
    });

    ui.add_space(12.0);

    // ── Gate loads bar chart ──────────────────────────────────────
    ui.label(
        RichText::new("Gate Loads (G1..G7)")
            .color(COL_DIM)
            .size(11.0)
            .strong(),
    );
    ui.add_space(4.0);
    let max_v = state.gates.counts.iter().copied().max().unwrap_or(1).max(1);
    for (i, &label) in CAT_NAMES.iter().enumerate() {
        let n = state.gates.counts[i];
        let frac = n as f32 / max_v as f32;
        let col = CAT_COLS[i];
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!("{label:<12}"))
                    .color(COL_DIM)
                    .size(10.0)
                    .monospace(),
            );
            let (bar, _) =
                ui.allocate_exact_size(egui::Vec2::new(200.0, 12.0), egui::Sense::hover());
            ui.painter()
                .rect_filled(bar, Rounding::same(2.0), Color32::from_rgb(20, 8, 10));
            if frac > 0.0 {
                let fill = egui::Rect::from_min_size(bar.min, egui::Vec2::new(200.0 * frac, 12.0));
                ui.painter().rect_filled(fill, Rounding::same(2.0), col);
            }
            ui.label(
                RichText::new(format!("{n:>3}"))
                    .color(Color32::from_rgb(180, 140, 150))
                    .size(10.0)
                    .monospace(),
            );
        });
    }

    // ── Alert banner if malware active ────────────────────────────
    if state.etl.sdb_malware_hits > 0 {
        ui.add_space(10.0);
        egui::Frame::none()
            .fill(Color32::from_rgb(60, 0, 10))
            .stroke(Stroke::new(1.5_f32, COL_RED))
            .rounding(Rounding::same(4.0))
            .inner_margin(egui::Margin::same(8.0))
            .show(ui, |ui| {
                ui.label(
                    RichText::new(format!(
                        "NERGAL ALERT — {} MALWARE HIT(S) — AWAITING STEWARD CLEARANCE",
                        state.etl.sdb_malware_hits
                    ))
                    .color(COL_RED)
                    .size(11.0)
                    .strong(),
                );
            });
    }
}
