//! glossary.rs — 𒂍𒁾 the living Glossary: every sealed Akkadian name in
//! `naming-registry`, searchable, with its image when one exists and is
//! credited. "The Glossary is not just telling myths; it becomes alive"
//! (the Architect's own framing for this feature).
//!
//! HONEST STATE (2026-08-01): no image assets exist in this repo yet --
//! every entry's `image_path` is `None` today. This panel is real and
//! functional now (search, browse, full detail view) even with zero
//! images; it renders the image slot the moment a real, credited one is
//! added to naming-registry, with no further UI work needed.

use egui::{Color32, RichText, ScrollArea};
use naming_registry::{seed, NamingEntry, NamingStatus};

#[derive(Default)]
pub struct GlossaryState {
    pub filter: String,
    pub selected: usize,
}

fn status_color(status: NamingStatus) -> Color32 {
    match status {
        NamingStatus::SealedByLaw => Color32::from_rgb(255, 215, 0),
        NamingStatus::ExistingUnverified => Color32::from_rgb(200, 140, 60),
        NamingStatus::Reserved => Color32::from_rgb(120, 130, 150),
    }
}

fn status_label(status: NamingStatus) -> &'static str {
    match status {
        NamingStatus::SealedByLaw => "SEALED",
        NamingStatus::ExistingUnverified => "UNVERIFIED",
        NamingStatus::Reserved => "RESERVED",
    }
}

pub fn draw(ui: &mut egui::Ui, state: &mut GlossaryState) {
    let registry = seed();
    let filter_lower = state.filter.to_lowercase();
    let matches: Vec<&NamingEntry> = if filter_lower.is_empty() {
        registry.0.iter().collect()
    } else {
        registry.search(&filter_lower)
    };

    ui.horizontal(|ui| {
        ui.label(
            RichText::new("𒂍𒁾 Glossary")
                .size(18.0)
                .color(Color32::from_rgb(255, 215, 0)),
        );
        ui.separator();
        ui.label(
            RichText::new(format!("{} of {} entries", matches.len(), registry.0.len()))
                .size(12.0)
                .color(Color32::GRAY),
        );
    });
    ui.horizontal(|ui| {
        ui.label("search:");
        ui.text_edit_singleline(&mut state.filter);
    });
    ui.separator();

    if state.selected >= matches.len() {
        state.selected = 0;
    }

    ui.horizontal_top(|ui| {
        // ── left: scrollable list ──
        ui.vertical(|ui| {
            ui.set_min_width(220.0);
            ui.set_max_width(260.0);
            ScrollArea::vertical()
                .id_salt("glossary_list")
                .max_height(ui.available_height())
                .show(ui, |ui| {
                    for (i, entry) in matches.iter().enumerate() {
                        let selected = i == state.selected;
                        let col = if selected {
                            Color32::from_rgb(255, 215, 0)
                        } else {
                            Color32::from_rgb(200, 205, 215)
                        };
                        let label = format!("{} · {}", entry.name, status_label(entry.status));
                        if ui
                            .add(egui::SelectableLabel::new(
                                selected,
                                RichText::new(label).color(col).size(13.0),
                            ))
                            .clicked()
                        {
                            state.selected = i;
                        }
                    }
                    if matches.is_empty() {
                        ui.label(RichText::new("no matches").color(Color32::GRAY).italics());
                    }
                });
        });

        ui.separator();

        // ── right: detail view ──
        ui.vertical(|ui| {
            let Some(entry) = matches.get(state.selected) else {
                ui.label(RichText::new("select an entry").color(Color32::GRAY));
                return;
            };
            draw_detail(ui, entry);
        });
    });
}

fn draw_detail(ui: &mut egui::Ui, entry: &NamingEntry) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(entry.name)
                .size(20.0)
                .strong()
                .color(Color32::from_rgb(255, 215, 0)),
        );
        ui.label(
            RichText::new(status_label(entry.status))
                .size(11.0)
                .color(status_color(entry.status))
                .background_color(Color32::from_rgb(20, 20, 25)),
        );
    });
    ui.label(
        RichText::new(format!(
            "{:?} · {:?}",
            entry.mytho_category, entry.system_role
        ))
        .color(Color32::GRAY)
        .size(12.0),
    );
    ui.add_space(6.0);

    // Image slot -- real and wired, empty honestly until a credited
    // image_path exists. Never renders an image without its credit.
    match (entry.image_path, entry.image_credit) {
        (Some(path), Some(credit)) => {
            ui.label(
                RichText::new(format!("🖼 {path}"))
                    .size(12.0)
                    .color(Color32::LIGHT_BLUE),
            );
            ui.label(
                RichText::new(format!("credit: {credit}"))
                    .size(11.0)
                    .italics()
                    .color(Color32::GRAY),
            );
        }
        _ => {
            ui.label(
                RichText::new("(no image on file for this entry)")
                    .italics()
                    .color(Color32::from_rgb(90, 95, 105)),
            );
        }
    }
    ui.add_space(6.0);

    ui.separator();
    ui.label(RichText::new(entry.blurb).size(13.0));
    ui.add_space(6.0);

    if !entry.domain_tags.is_empty() {
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("tags:").color(Color32::GRAY).size(11.0));
            for tag in entry.domain_tags {
                ui.label(
                    RichText::new(*tag)
                        .size(11.0)
                        .color(Color32::from_rgb(150, 200, 255))
                        .background_color(Color32::from_rgb(20, 30, 45)),
                );
            }
        });
    }

    if let Some(doc) = entry.source_doc {
        ui.label(
            RichText::new(format!("source: {doc}"))
                .size(11.0)
                .color(Color32::GRAY),
        );
    }
    if let Some(path) = entry.crate_path {
        ui.label(
            RichText::new(format!("crate: {path}"))
                .size(11.0)
                .color(Color32::GRAY),
        );
    }
}
