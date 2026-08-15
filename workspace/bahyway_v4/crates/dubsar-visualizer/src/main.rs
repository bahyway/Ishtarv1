mod app;
mod panels;
mod theme;

fn main() -> eframe::Result<()> {
    let opts = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("DubSar IDE v4.0  \u{12000}  EnkiDB Sovereign Visualizer")
            .with_inner_size([1400.0, 860.0])
            .with_min_inner_size([900.0, 600.0]),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };
    eframe::run_native(
        "dubsar-visualizer",
        opts,
        Box::new(|cc| Ok(Box::new(app::DubSarApp::new(cc)))),
    )
}
