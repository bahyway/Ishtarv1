pub mod etl_flow;
pub mod glossary;
pub mod hepta;
pub mod najaf;
pub mod nergal;
pub mod particles;
pub mod uros_layout;
pub mod wpd;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ActivePanel {
    UrOS,
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
pub mod eaagent;
pub mod tamuzai;
