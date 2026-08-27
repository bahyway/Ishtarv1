//! AnuGovernor (formerly Shakkanakku) — the governor. One command executes
//! the corpus; warnings proceed, majors await the Architect; the report is
//! sealed.
//!
//! This binary is the native-window face (feature "gui") or the headless
//! terminal face (no features). Both drive the `anu_governor` library crate
//! (see src/lib.rs) — the same engine the browser-facing `anu-governor-web`
//! binary (src/bin/anu_governor_web.rs) drives.

#[cfg(feature = "gui")]
use anu_governor::app;
#[cfg(not(feature = "gui"))]
use anu_governor::{config, report, runner};

#[cfg(feature = "gui")]
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            // Deliberately modest and NOT maximized: on eriduous-vdi the
            // compositor's "screen" is a virtual desktop wider than the
            // actual viewer pane the Architect sees, so with_maximized(true)
            // made the window fill that oversized virtual canvas instead of
            // the visible viewport — worse than the original problem. There
            // is no reliable way to ask winit for the *visible* viewer size
            // before the window exists, so this picks a size small enough
            // to fit inside a handheld/VDI viewer by default; the Architect
            // can still grow it with the border-drag resize (see app.rs).
            .with_inner_size([900.0, 620.0])
            .with_min_inner_size([640.0, 420.0])
            // No OS titlebar/chrome (some compositors, e.g. bare VDI Wayland
            // sessions, never draw one) — the app paints its own title bar
            // with close/minimize/maximize and a resize grip instead.
            .with_decorations(false),
        ..Default::default()
    };
    eframe::run_native(
        "AnuGovernor 𒀭 — BahyWay Governor",
        options,
        Box::new(|cc| Ok(Box::new(app::AnuGovernorApp::new(cc)))),
    )
}

/// Headless mode (build with --no-default-features): run the corpus in the
/// terminal, auto-abort on major (no Architect present), seal the report.
#[cfg(not(feature = "gui"))]
fn main() -> anyhow::Result<()> {
    use anu_governor::model::*;
    use std::path::Path;

    let cfg_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "anu-governor.toml".into());
    let cfg = config::Config::load(Path::new(&cfg_path))?;
    let params: Vec<(String, String)> = cfg
        .parameters
        .iter()
        .map(|p| (p.key.clone(), p.value.clone()))
        .collect();

    let (tx, rx) = crossbeam_channel::unbounded();
    let (ctl_tx, ctl_rx) = crossbeam_channel::unbounded();
    let started = chrono::Utc::now();
    let n = cfg.run.playbooks.len();
    let mut statuses = vec![PbStatus::Pending; n];
    let mut events: Vec<ErrorEvent> = Vec::new();

    let handle = runner::spawn_runner(cfg.clone(), params.clone(), tx, ctl_rx);
    for ev in rx.iter() {
        match ev {
            RunnerEvent::PbStarted(i) => {
                statuses[i] = PbStatus::Running;
                println!("▶ [{}/{}] {}", i + 1, n, cfg.run.playbooks[i]);
            }
            RunnerEvent::PbOk(i) => statuses[i] = PbStatus::Ok,
            RunnerEvent::PbWarned(i, e) => {
                statuses[i] = PbStatus::Warned;
                println!("  ⚠ {} — {}", e.error_type, e.message);
                events.push(e);
            }
            RunnerEvent::PbFailed(i, e) => {
                statuses[i] = PbStatus::Failed;
                println!("  ✖ MAJOR — {} — headless: aborting", e.message);
                events.push(e);
                let _ = ctl_tx.send(Ctl::Abort);
            }
            RunnerEvent::Log(l) => println!("  {l}"),
            RunnerEvent::Finished | RunnerEvent::Aborted => break,
        }
    }
    let _ = handle.join();

    let run = report::RunSummary {
        started,
        finished: chrono::Utc::now(),
        playbooks: &cfg.run.playbooks,
        statuses: &statuses,
        events: &events,
        parameters: &params,
        fixes_generated: &[],
    };
    let path = report::generate(&cfg.run.report_dir, &cfg.run.seal_key, &run)?;
    println!("\n𒁾 sealed report: {}", path.display());
    println!("   verified: {}", report::verify(&path)?);
    Ok(())
}
