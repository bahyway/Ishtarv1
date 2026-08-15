pub mod etl_flow;
pub mod eridu_layout;
pub mod glossary;
pub mod hepta;
pub mod najaf;
pub mod nergal;
pub mod particles;
pub mod wpd;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ActivePanel {
    EriduOS,
    Hepta,
    Particles,
    Wpd,
    Najaf,
    EtlFlow,
    TamuzAI,
    EaAgent,
    Nergal,
    Glossary,
}

/// Which inspector tab is shown in the Houdini-style particle inspector.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum InspectorTab {
    Radar,
    #[default]
    Attributes,
    Story,
}
pub mod tamuzai;
pub mod eaagent;
