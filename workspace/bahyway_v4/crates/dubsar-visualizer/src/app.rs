use bahyway_core::TribeId;
use eframe::{App, CreationContext, Frame};
use egui::{Context, RichText};
use enkidb_journal::{EavTriple, Journal, JournalEntry};
use enkidb_kaki::{EventKaki, IdentityKaki, KakiMinter, KakiRole};
use enkidb_snapshot::SnapshotRecord;

use crate::panels::etl_flow::EtlFlowState;
use crate::panels::{ActivePanel, InspectorTab};
use crate::theme;

// ── WPD domain EAV attribute hashes (visualizer demo journal) ─────────────────
// Stored alongside the mandatory story-engine attrs so the Story tab can decode them.
pub(crate) const ATTR_SEGMENT_REF: u32 = 0xB001; // e.g. "BGH-GZ-001"
pub(crate) const ATTR_DEFECT_SCORE: u32 = 0xB002; // defect score string "0.12"
pub(crate) const ATTR_SECTOR_NAME: u32 = 0xB003; // human sector label
pub(crate) const ATTR_EVENT_TYPE: u32 = 0xB004; // display label for the Story tab

// Mandatory story-engine attrs re-exported for particles.rs
pub(crate) use story_engine::projection::{ATTR_FRESHNESS, ATTR_QUALITY, ATTR_STATE};

// ── Lightweight demo particle ─────────────────────────────────────────────────

/// UI representation of a sovereign particle.
/// `kaki_hex` is derived from the real `IdentityKaki` minted at startup.
#[derive(Clone)]
pub struct ParticleDemo {
    pub id: String,
    pub label: String,
    pub kaki_hex: String, // 32-char uppercase hex from the real IdentityKaki bytes
    pub dims: [f32; 7],
    pub hps: f32,
    pub rgb: (u8, u8, u8),
    pub state: &'static str,
}

// ── App state ─────────────────────────────────────────────────────────────────

pub struct DubSarApp {
    pub panel: ActivePanel,
    pub particles: Vec<ParticleDemo>,
    pub identities: Vec<IdentityKaki>, // parallel to particles — sovereign KAKIs
    pub journal: Journal,
    pub snapshots: Vec<SnapshotRecord>,
    pub selected: usize,
    pub inspector_tab: InspectorTab,
    pub etl_flow_state: EtlFlowState,
    pub tamuzai_state: enkidullm_chat::ChatEngine,
    pub eaagent_state: ea_agent_chat::EaChatEngine,
    pub bus: bee_mdm_bus::BusHandle, // real BeeMDM ETL gate state (Nergal + UrOS panels)
    pub eridu: crate::panels::uros_layout::EriduState,
    pub glossary_state: crate::panels::glossary::GlossaryState,
}

impl DubSarApp {
    pub fn new(cc: &CreationContext) -> Self {
        theme::apply(&cc.egui_ctx);

        let (mut particles, identities, journal, snapshots) = build_sovereign_particles();

        // Update kaki_hex to the real 32-char hex derived from the minted IdentityKaki
        for (p, id) in particles.iter_mut().zip(identities.iter()) {
            p.kaki_hex = id.bytes().iter().map(|b| format!("{b:02X}")).collect();
        }

        let (bus, _cmd_rx) = bee_mdm_bus::new_bus();
        bee_mdm_bus::populate_demo(&bus);

        DubSarApp {
            panel: ActivePanel::UrOS,
            particles,
            identities,
            journal,
            snapshots,
            selected: 0,
            inspector_tab: InspectorTab::default(),
            etl_flow_state: EtlFlowState::demo(),
            tamuzai_state: enkidullm_chat::ChatEngine::new(),
            eaagent_state: ea_agent_chat::EaChatEngine::new(),
            bus,
            eridu: crate::panels::uros_layout::EriduState::default(),
            glossary_state: crate::panels::glossary::GlossaryState::default(),
        }
    }
}

impl App for DubSarApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        egui::TopBottomPanel::top("nav_bar")
            .exact_height(40.0)
            .frame(
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(14, 16, 22))
                    .stroke(egui::Stroke::new(
                        1.0_f32,
                        egui::Color32::from_rgb(40, 50, 70),
                    )),
            )
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.add_space(10.0);
                    ui.label(
                        RichText::new("𒀭 DubSar IDE v4.0")
                            .color(theme::GOLD)
                            .size(15.0)
                            .strong(),
                    );
                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    // Enterprise Apps dropdown
                    let ent_panels: &[(ActivePanel, &str)] = &[
                        (ActivePanel::UrOS, "UrOS"),
                        (ActivePanel::Hepta, "Hepta 7D"),
                        (ActivePanel::Particles, "Particles"),
                        (ActivePanel::Wpd, "Baghdad WPD"),
                        (ActivePanel::Najaf, "Najaf"),
                        (ActivePanel::EtlFlow, "ETL Flow"),
                        (ActivePanel::Nergal, "Nergal AV"),
                    ];
                    let ent_active = ent_panels.iter().any(|(p, _)| *p == self.panel);
                    let ent_id = ui.make_persistent_id("dd_ent");
                    let ent_r = ui.add(
                        egui::Button::new(
                            RichText::new(if ent_active {
                                "Enterprise Apps *"
                            } else {
                                "Enterprise Apps v"
                            })
                            .color(if ent_active {
                                theme::GOLD
                            } else {
                                theme::TEXT_DIM
                            })
                            .size(13.0),
                        )
                        .fill(if ent_active {
                            egui::Color32::from_rgb(24, 22, 10)
                        } else {
                            egui::Color32::TRANSPARENT
                        })
                        .rounding(egui::Rounding::same(4.0)),
                    );
                    if ent_r.clicked() {
                        ui.memory_mut(|m| {
                            let v = m.data.get_temp_mut_or_default::<bool>(ent_id);
                            *v = !*v;
                        });
                    }
                    if ui.memory(|m| m.data.get_temp::<bool>(ent_id).unwrap_or(false)) {
                        egui::Window::new("ent_menu_win")
                            .title_bar(false)
                            .resizable(false)
                            .collapsible(false)
                            .fixed_pos(ent_r.rect.left_bottom())
                            .frame(
                                egui::Frame::none()
                                    .fill(egui::Color32::from_rgb(18, 22, 32))
                                    .stroke(egui::Stroke::new(
                                        1.0_f32,
                                        egui::Color32::from_rgb(60, 80, 120),
                                    ))
                                    .rounding(egui::Rounding::same(4.0))
                                    .inner_margin(egui::Margin::same(6.0)),
                            )
                            .show(ui.ctx(), |ui| {
                                ui.set_min_width(180.0);
                                for (panel, label) in ent_panels {
                                    let active = self.panel == *panel;
                                    let col = if active {
                                        theme::GOLD
                                    } else {
                                        egui::Color32::from_rgb(180, 190, 205)
                                    };
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                RichText::new(*label).color(col).size(13.0),
                                            )
                                            .fill(if active {
                                                egui::Color32::from_rgb(28, 26, 14)
                                            } else {
                                                egui::Color32::TRANSPARENT
                                            })
                                            .min_size(egui::Vec2::new(170.0, 26.0)),
                                        )
                                        .clicked()
                                    {
                                        self.panel = *panel;
                                        ui.memory_mut(|m| m.data.insert_temp(ent_id, false));
                                    }
                                }
                            });
                    }

                    ui.add_space(6.0);
                    ui.separator();
                    ui.add_space(6.0);

                    // AI Agents dropdown
                    let ai_panels: &[(ActivePanel, &str)] = &[
                        (ActivePanel::TamuzAI, "TamuzAI  𒀭"),
                        (ActivePanel::EaAgent, "EaAgent  𒂗𒆠"),
                    ];
                    let ai_active = ai_panels.iter().any(|(p, _)| *p == self.panel);
                    let ai_id = ui.make_persistent_id("dd_ai");
                    let ai_r = ui.add(
                        egui::Button::new(
                            RichText::new(if ai_active {
                                "AI Agents *"
                            } else {
                                "AI Agents v"
                            })
                            .color(if ai_active {
                                theme::GOLD
                            } else {
                                theme::TEXT_DIM
                            })
                            .size(13.0),
                        )
                        .fill(egui::Color32::TRANSPARENT)
                        .rounding(egui::Rounding::same(4.0)),
                    );
                    if ai_r.clicked() {
                        ui.memory_mut(|m| {
                            let v = m.data.get_temp_mut_or_default::<bool>(ai_id);
                            *v = !*v;
                        });
                    }
                    if ui.memory(|m| m.data.get_temp::<bool>(ai_id).unwrap_or(false)) {
                        egui::Window::new("ai_menu_win")
                            .title_bar(false)
                            .resizable(false)
                            .collapsible(false)
                            .fixed_pos(ai_r.rect.left_bottom())
                            .frame(
                                egui::Frame::none()
                                    .fill(egui::Color32::from_rgb(18, 22, 32))
                                    .stroke(egui::Stroke::new(
                                        1.0_f32,
                                        egui::Color32::from_rgb(60, 80, 120),
                                    ))
                                    .rounding(egui::Rounding::same(4.0))
                                    .inner_margin(egui::Margin::same(6.0)),
                            )
                            .show(ui.ctx(), |ui| {
                                ui.set_min_width(160.0);
                                for (panel, label) in ai_panels {
                                    let active = self.panel == *panel;
                                    let col = if active {
                                        theme::GOLD
                                    } else {
                                        egui::Color32::from_rgb(180, 190, 205)
                                    };
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                RichText::new(*label).color(col).size(13.0),
                                            )
                                            .fill(egui::Color32::TRANSPARENT)
                                            .min_size(egui::Vec2::new(150.0, 26.0)),
                                        )
                                        .clicked()
                                    {
                                        self.panel = *panel;
                                        ui.memory_mut(|m| m.data.insert_temp(ai_id, false));
                                    }
                                }
                            });
                    }

                    ui.add_space(6.0);
                    ui.separator();
                    ui.add_space(6.0);

                    // Glossary -- a single destination, not a dropdown group
                    // (2026-08-01: "the Glossary is not just telling myths;
                    // it becomes alive" -- the Architect's own framing).
                    {
                        let active = self.panel == ActivePanel::Glossary;
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new("𒂍𒁾 Glossary")
                                        .color(if active { theme::GOLD } else { theme::TEXT_DIM })
                                        .size(13.0),
                                )
                                .fill(if active {
                                    egui::Color32::from_rgb(24, 22, 10)
                                } else {
                                    egui::Color32::TRANSPARENT
                                })
                                .rounding(egui::Rounding::same(4.0)),
                            )
                            .clicked()
                        {
                            self.panel = ActivePanel::Glossary;
                        }
                    }

                    // Right side: live status from the real bee-mdm-bus
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(10.0);
                        if let Ok(b) = self.bus.read() {
                            let b11 = b.fuzzy.b11;
                            let b11_col = if b11 >= 200 {
                                egui::Color32::from_rgb(200, 160, 40)
                            } else if b11 >= 140 {
                                egui::Color32::from_rgb(60, 160, 180)
                            } else {
                                egui::Color32::from_rgb(160, 80, 90)
                            };
                            ui.label(
                                RichText::new(format!(
                                    "B11:{}/240  Gold:{}  Proc:{}",
                                    b11, b.total_golden, b.total_processed
                                ))
                                .color(egui::Color32::from_rgb(100, 110, 130))
                                .size(10.0),
                            );
                            ui.separator();
                            ui.label(
                                RichText::new(format!("B11:{b11}"))
                                    .color(b11_col)
                                    .size(11.0)
                                    .strong(),
                            );
                            ui.separator();
                            ui.label(
                                RichText::new("𒁾 BahyWay.Ecosystem v4.0")
                                    .color(egui::Color32::from_rgb(160, 130, 50))
                                    .size(11.0),
                            );
                        }
                    });
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| match self.panel {
            ActivePanel::UrOS => crate::panels::uros_layout::draw(
                ui,
                &mut self.eridu,
                &self.bus,
                &self.particles,
                &self.identities,
                &self.journal,
                &self.snapshots,
            ),
            ActivePanel::Hepta => {
                crate::panels::hepta::draw(ui, &self.particles, &mut self.selected)
            }
            ActivePanel::Particles => crate::panels::particles::draw(
                ui,
                &self.particles,
                &mut self.selected,
                &mut self.inspector_tab,
                &self.journal,
                &self.identities,
                &self.snapshots,
            ),
            ActivePanel::Wpd => crate::panels::wpd::draw(ui),
            ActivePanel::Najaf => crate::panels::najaf::draw(ui),
            ActivePanel::EtlFlow => crate::panels::etl_flow::draw(ui, &self.etl_flow_state),
            ActivePanel::TamuzAI => crate::panels::tamuzai::draw(ui, &mut self.tamuzai_state),
            ActivePanel::EaAgent => crate::panels::eaagent::draw(ui, &mut self.eaagent_state),
            ActivePanel::Nergal => crate::panels::nergal::draw(ui, &self.bus),
            ActivePanel::Glossary => crate::panels::glossary::draw(ui, &mut self.glossary_state),
        });
    }
}

// ── Sovereign particle construction ──────────────────────────────────────────

/// Particle seed data: (id, label, state_bytes, hps, dims, rgb, segment_ref, sector_name, defect)
type ParticleSeed = (
    &'static str,
    &'static str,
    &'static [u8],
    f32,
    [f32; 7],
    [u8; 3],
    &'static str,
    &'static str,
    f32,
);

#[rustfmt::skip]
const PARTICLE_SEED: &[ParticleSeed] = &[
    ("PKL-GZ-001","Sentinel Alpha",  b"GOLDEN",0.95,[0.92,0.30,0.03,0.95,0.88,0.70,0.91],[0,210,180], "BGH-GZ-001","Green Zone (H7-00 SUN)",    0.12),
    ("PKL-KZ-002","Kadhimiya Watch", b"FUZZY", 0.62,[0.78,0.85,0.03,0.62,0.55,0.48,0.74],[180,155,0], "BGH-KZ-001","Al-Kadhimiya (H7-01 MOON)",  0.45),
    ("PKL-SC-003","Sadr Pulse",      b"DEAD",  0.22,[0.55,1.00,0.03,0.22,0.18,0.35,0.44],[220,80,40],  "BGH-SC-001","Sadr City (H7-02 MERCURY)",  0.78),
    ("PKL-KR-004","Karrada Node",    b"GOLDEN",0.80,[0.84,0.75,0.03,0.80,0.76,0.62,0.83],[80,220,140], "BGH-KR-001","Karrada (H7-03 VENUS)",      0.28),
    ("PKL-RS-005","Rashid Trunk",    b"FUZZY", 0.45,[0.68,0.70,0.03,0.45,0.42,0.50,0.60],[200,120,40], "BGH-RS-001","Rashid (H7-04 MARS)",        0.55),
    ("PKL-JD-006","Jadria Fresh",    b"GOLDEN",0.98,[0.97,0.60,0.03,0.98,0.97,0.85,0.96],[0,255,200],  "BGH-JD-001","Al-Jadria (H7-05 JUPITER)",  0.08),
    ("PKL-MN-007","Mansour Beacon",  b"FUZZY", 0.74,[0.81,0.80,0.03,0.74,0.72,0.68,0.78],[120,180,255],"BGH-MN-001","Al-Mansour (H7-06 SATURN)", 0.32),
];

fn build_sovereign_particles() -> (
    Vec<ParticleDemo>,
    Vec<IdentityKaki>,
    Journal,
    Vec<SnapshotRecord>,
) {
    let minter = KakiMinter::new(TribeId::from_u16(0x0001));
    let mut journal = Journal::new(64);
    let mut snapshots = Vec::new();
    let mut identities = Vec::new();
    let mut particles = Vec::new();

    for &(pid, label, state_b, hps, dims, rgb, seg_ref, sector_name, defect) in PARTICLE_SEED {
        // Mint sovereign IdentityKaki for this particle
        let identity =
            IdentityKaki::try_from_kaki(minter.identity(KakiRole::Zikru)).expect("identity mint");

        // ── Epoch 100: REGISTERED ─────────────────────────────────────
        journal
            .append(event(
                &minter,
                &identity,
                100,
                vec![
                    eav(ATTR_STATE, b"FUZZY"),
                    eav(ATTR_EVENT_TYPE, b"REGISTERED"),
                    eav(ATTR_SEGMENT_REF, seg_ref.as_bytes()),
                ],
            ))
            .unwrap();

        // ── Epoch 200: INITIAL_SCAN ───────────────────────────────────
        let quality_str = format!("{hps:.3}");
        journal
            .append(event(
                &minter,
                &identity,
                200,
                vec![
                    eav(ATTR_QUALITY, quality_str.as_bytes()),
                    eav(ATTR_STATE, b"FUZZY"), // still fuzzy until SCORED
                    eav(ATTR_EVENT_TYPE, b"INITIAL_SCAN"),
                ],
            ))
            .unwrap();

        // ── Epoch 300: SCORED ─────────────────────────────────────────
        let freshness_str = format!("{:.3}", hps * 0.95_f32);
        let defect_str = format!("{defect:.3}");
        journal
            .append(event(
                &minter,
                &identity,
                300,
                vec![
                    eav(ATTR_STATE, state_b),
                    eav(ATTR_QUALITY, quality_str.as_bytes()),
                    eav(ATTR_FRESHNESS, freshness_str.as_bytes()),
                    eav(ATTR_SECTOR_NAME, sector_name.as_bytes()),
                    eav(ATTR_DEFECT_SCORE, defect_str.as_bytes()),
                    eav(ATTR_EVENT_TYPE, b"SCORED"),
                ],
            ))
            .unwrap();

        // ── State-specific events ─────────────────────────────────────
        match state_b {
            b"GOLDEN" => {
                // Epoch 350: SNAPSHOT_TAKEN (snapshot-accelerated replay)
                journal
                    .append(event(
                        &minter,
                        &identity,
                        350,
                        vec![
                            eav(story_engine::projection::ATTR_SNAPSHOT_DATE, b"1748563200"),
                            eav(ATTR_EVENT_TYPE, b"SNAPSHOT_TAKEN"),
                        ],
                    ))
                    .unwrap();
                snapshots.push(SnapshotRecord::at_birth(identity, 1_748_563_200));

                // Epoch 400: VERIFIED
                journal
                    .append(event(
                        &minter,
                        &identity,
                        400,
                        vec![
                            eav(ATTR_QUALITY, quality_str.as_bytes()),
                            eav(ATTR_EVENT_TYPE, b"VERIFIED"),
                        ],
                    ))
                    .unwrap();
            }
            b"FUZZY" => {
                // Epoch 350: QUALITY_WARNING
                journal
                    .append(event(
                        &minter,
                        &identity,
                        350,
                        vec![
                            eav(ATTR_DEFECT_SCORE, defect_str.as_bytes()),
                            eav(ATTR_EVENT_TYPE, b"QUALITY_WARNING"),
                        ],
                    ))
                    .unwrap();
                // Epoch 400: MONITORED
                journal
                    .append(event(
                        &minter,
                        &identity,
                        400,
                        vec![eav(ATTR_EVENT_TYPE, b"MONITORED")],
                    ))
                    .unwrap();
            }
            _ => {
                // Epoch 350: QUALITY_ALERT — state falls to DEAD
                journal
                    .append(event(
                        &minter,
                        &identity,
                        350,
                        vec![
                            eav(ATTR_STATE, b"DEAD"),
                            eav(ATTR_EVENT_TYPE, b"QUALITY_ALERT"),
                        ],
                    ))
                    .unwrap();
                // Epoch 400: QUARANTINED
                journal
                    .append(event(
                        &minter,
                        &identity,
                        400,
                        vec![eav(ATTR_EVENT_TYPE, b"QUARANTINED")],
                    ))
                    .unwrap();
            }
        }

        // ── Epoch 500: LIVE (current heartbeat) ───────────────────────
        journal
            .append(event(
                &minter,
                &identity,
                500,
                vec![
                    eav(ATTR_FRESHNESS, freshness_str.as_bytes()),
                    eav(ATTR_EVENT_TYPE, b"LIVE"),
                ],
            ))
            .unwrap();

        // Build ParticleDemo (kaki_hex will be overwritten in new() with real hex)
        let state_label: &'static str = match state_b {
            b"GOLDEN" => "Golden",
            b"DEAD" => "Gray",
            _ => "Drifting",
        };
        particles.push(ParticleDemo {
            id: pid.to_string(),
            label: label.to_string(),
            kaki_hex: String::new(), // filled in by new()
            dims,
            hps,
            rgb: (rgb[0], rgb[1], rgb[2]),
            state: state_label,
        });
        identities.push(identity);
    }

    (particles, identities, journal, snapshots)
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn event(
    minter: &KakiMinter,
    identity: &IdentityKaki,
    epoch: u32,
    triples: Vec<EavTriple>,
) -> JournalEntry {
    let ek = EventKaki::try_from_kaki(minter.event(KakiRole::Zikru)).expect("event mint");
    JournalEntry::new(ek, *identity, epoch, triples)
}

fn eav(attr_hash: u32, value: &[u8]) -> EavTriple {
    EavTriple::new(attr_hash, value.to_vec())
}
