//! eaagent.rs — 𒂗𒆠 EaAgent sovereign math panel for DubSar IDE.
use egui::{ScrollArea, TextEdit, RichText, Color32, Frame, Stroke};
use ea_agent_chat::{EaChatEngine, EaMessageKind};

pub fn draw(ui: &mut egui::Ui, engine: &mut EaChatEngine) {
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new("𒂗𒆠 EaAgent").size(18.0).color(Color32::from_rgb(0, 200, 255)));
            ui.separator();
            ui.label(RichText::new(engine.status_line()).size(12.0).color(Color32::GRAY));
        });
        ui.separator();
        let available = ui.available_height() - 60.0;
        ScrollArea::vertical()
            .max_height(available)
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for msg in &engine.panel.messages {
                    let [r, g, b] = msg.kind.color_rgb();
                    let label_color = Color32::from_rgb(r, g, b);
                    let bg_color = match msg.kind {
                        EaMessageKind::User   => Color32::from_rgb(20, 30, 45),
                        EaMessageKind::Agent  => Color32::from_rgb(10, 25, 35),
                        EaMessageKind::Math   => Color32::from_rgb(10, 30, 20),
                        EaMessageKind::Error  => Color32::from_rgb(40, 10, 10),
                        EaMessageKind::System => Color32::from_rgb(20, 20, 25),
                    };
                    Frame::none()
                        .fill(bg_color)
                        .stroke(Stroke::new(1.0, label_color.linear_multiply(0.3)))
                        .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                        .show(ui, |ui| {
                            ui.label(RichText::new(msg.kind.prefix()).size(11.0).color(label_color).strong());
                            ui.label(RichText::new(&msg.content).size(13.0).color(Color32::LIGHT_GRAY));
                        });
                    ui.add_space(2.0);
                }
            });
        ui.separator();
        ui.horizontal(|ui| {
            let input = TextEdit::singleline(&mut engine.panel.input)
                .hint_text("Ask EaAgent math... or !solve x^2-5x+6 · !b11 · !help")
                .desired_width(ui.available_width() - 80.0);
            let response = ui.add(input);
            let send = ui.button(RichText::new("Solve 𒂗𒆠").color(Color32::from_rgb(0, 200, 255)));
            if (send.clicked() || (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))))
                && !engine.panel.input.is_empty()
            {
                let msg = engine.panel.take_input();
                let _ = engine.chat(&msg);
            }
        });
    });
}
