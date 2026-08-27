//! tamuzai.rs — 𒀭 TamuzAI sovereign chat panel for DubSar IDE.
use egui::{Color32, Frame, RichText, ScrollArea, Stroke, TextEdit};
use enkidullm_chat::{ChatEngine, MessageKind};

pub fn draw(ui: &mut egui::Ui, engine: &mut ChatEngine) {
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("𒀭 TamuzAI")
                    .size(18.0)
                    .color(Color32::from_rgb(255, 215, 0)),
            );
            ui.separator();
            ui.label(
                RichText::new(engine.status_line())
                    .size(12.0)
                    .color(Color32::GRAY),
            );
        });
        ui.separator();
        let available = ui.available_height() - 60.0;
        ScrollArea::vertical()
            .max_height(available)
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for msg in &engine.panel.messages {
                    let (label_color, bg_color) = match msg.kind {
                        MessageKind::User => (
                            Color32::from_rgb(100, 180, 255),
                            Color32::from_rgb(20, 30, 45),
                        ),
                        MessageKind::Assistant => (
                            Color32::from_rgb(255, 215, 0),
                            Color32::from_rgb(30, 25, 10),
                        ),
                        MessageKind::Command => (
                            Color32::from_rgb(0, 255, 136),
                            Color32::from_rgb(10, 30, 20),
                        ),
                        MessageKind::Error => (
                            Color32::from_rgb(255, 80, 80),
                            Color32::from_rgb(40, 10, 10),
                        ),
                        MessageKind::System => (Color32::GRAY, Color32::from_rgb(20, 20, 25)),
                    };
                    Frame::none()
                        .fill(bg_color)
                        .stroke(Stroke::new(1.0_f32, label_color.linear_multiply(0.3)))
                        .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new(msg.kind.prefix())
                                    .size(11.0)
                                    .color(label_color)
                                    .strong(),
                            );
                            ui.label(
                                RichText::new(&msg.content)
                                    .size(13.0)
                                    .color(Color32::LIGHT_GRAY),
                            );
                        });
                    ui.add_space(2.0);
                }
            });
        ui.separator();
        ui.horizontal(|ui| {
            let input = TextEdit::singleline(&mut engine.panel.input)
                .hint_text("Ask TamuzAI... or type !help for commands")
                .desired_width(ui.available_width() - 70.0);
            let response = ui.add(input);
            let send = ui.button(RichText::new("Send 𒁾").color(Color32::from_rgb(255, 215, 0)));
            if (send.clicked()
                || (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))))
                && !engine.panel.input.is_empty()
            {
                let msg = engine.panel.take_input();
                let _ = engine.chat(&msg);
            }
        });
    });
}
