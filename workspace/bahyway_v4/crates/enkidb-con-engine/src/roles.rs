#![forbid(unsafe_code)]
//! Sovereign roles for the Connection Engine.

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SovereignRole {
    Client = 0,
    DataSteward = 1,
    TabletWriter = 2,
    DubSar = 3,
}

impl SovereignRole {
    /// Returns true if this role may issue write operations.
    pub fn can_write(&self) -> bool {
        matches!(self, Self::TabletWriter | Self::DubSar)
    }

    /// Returns true if this role is exempt from cross-tribe isolation.
    pub fn cross_tribe_exempt(&self) -> bool {
        matches!(self, Self::DubSar)
    }
}
