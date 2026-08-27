//! Hubble-Zooming service state — multi-level particle density navigator.
//! Pure Rust, no web_sys.

#[derive(Clone, PartialEq, Debug)]
pub enum ZoomLevel {
    Universe,
    Tribe(usize),
}

pub struct HubbleState {
    pub zoom: ZoomLevel,
}

impl HubbleState {
    pub fn new() -> Self {
        HubbleState {
            zoom: ZoomLevel::Universe,
        }
    }
}

impl Default for HubbleState {
    fn default() -> Self {
        Self::new()
    }
}

impl HubbleState {
    pub fn zoom_into_tribe(&mut self, ti: usize) {
        self.zoom = ZoomLevel::Tribe(ti);
    }
    pub fn zoom_out(&mut self) {
        self.zoom = ZoomLevel::Universe;
    }
}
