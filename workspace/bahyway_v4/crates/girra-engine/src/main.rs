//! girra-dashboard -- the lamp. eframe/egui native window, the same
//! sovereign GUI stack dubsar-visualizer already uses. No HTML, no
//! browser, no third-party TSDB wire format.

use girra_engine::collector;
use girra_engine::ingest;
use girra_engine::metrics::{gauge_laws, Band, Registry, Shared};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

struct GirraApp {
    registry: Shared,
    prev_jiffies: Option<(u64, u64)>,
    last_collect: Instant,
}

impl GirraApp {
    fn collect_host(&mut self) {
        if self.last_collect.elapsed() < Duration::from_secs(1) {
            return;
        }
        self.last_collect = Instant::now();
        let ts = collector::now_millis();
        if let Some(cur) = collector::cpu_jiffies() {
            if let Some(prev) = self.prev_jiffies {
                let load = collector::cpu_load(prev, cur);
                if let Ok(mut r) = self.registry.lock() {
                    r.push("host_cpu", load, ts);
                }
            }
            self.prev_jiffies = Some(cur);
        }
        if let Some(m) = collector::mem_used_fraction() {
            if let Ok(mut r) = self.registry.lock() {
                r.push("host_mem", m, ts);
            }
        }
    }
}

fn band_color(b: Band) -> egui::Color32 {
    match b {
        Band::Green => egui::Color32::from_rgb(70, 160, 90),
        Band::Amber => egui::Color32::from_rgb(220, 170, 40),
        Band::Red => egui::Color32::from_rgb(200, 60, 50),
    }
}

impl eframe::App for GirraApp {
    fn update(&mut self, ctx: &egui::Context, _f: &mut eframe::Frame) {
        self.collect_host();
        ctx.request_repaint_after(Duration::from_millis(500));

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("𒆤 GIRRA — the lamp of BahyWay");
            ui.separator();
            let reg = self.registry.lock().unwrap();
            egui::Grid::new("gauges").num_columns(2).show(ui, |ui| {
                for (key, label, law) in gauge_laws() {
                    ui.label(label);
                    match reg.latest(key) {
                        Some(v) => {
                            let b = law.band(v);
                            ui.colored_label(band_color(b), format!("{v:.4}"));
                        }
                        None => {
                            ui.weak("no signal");
                        }
                    }
                    ui.end_row();
                }
            });
            ui.separator();
            ui.weak("Metrics are meant to become KAKI Event particles (EnkiODB wiring separate). WITNESS, not query.");
        });
    }
}

fn main() -> eframe::Result<()> {
    let registry: Shared = Arc::new(Mutex::new(Registry::default()));
    ingest::spawn_listener(registry.clone(), ingest::DEFAULT_PORT);
    let app = GirraApp {
        registry,
        prev_jiffies: None,
        last_collect: Instant::now() - Duration::from_secs(2),
    };
    eframe::run_native(
        "GirraEngine",
        eframe::NativeOptions::default(),
        Box::new(|_| Ok(Box::new(app))),
    )
}
