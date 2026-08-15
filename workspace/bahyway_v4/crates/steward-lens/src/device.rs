#![forbid(unsafe_code)]
//! Device tier taxonomy for StewardLens (SL-TIER-001).

/// Four device tiers for StewardLens deployment.
///
/// Higher tiers are supersets: a Tablet can do everything a Smartphone can.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum DeviceTier {
    /// Smartwatch — haptic feedback + 2-button acknowledge only.
    /// Receives MAROON and NERGAL alerts via short vibration patterns.
    Smartwatch = 0,
    /// Smartphone — 30 FPS WGPU 2D map + AR camera overlay.
    /// Full alert display, tap to open Šàtamu decision form.
    Smartphone = 1,
    /// Tablet — 60 FPS WGPU 3D + AICouncil vote viewer panel.
    /// Side-by-side particle map and anomaly timeline.
    Tablet = 2,
    /// Desktop — 120 FPS multi-monitor + HeptaScript editor.
    /// Full steward workstation: pattern editor, Šàtamu audit trail,
    /// AICouncil deliberation log, ENKI-GENESIS simulation viewer.
    Desktop = 3,
}

impl DeviceTier {
    pub fn as_str(self) -> &'static str {
        match self {
            DeviceTier::Smartwatch => "Smartwatch",
            DeviceTier::Smartphone => "Smartphone",
            DeviceTier::Tablet     => "Tablet",
            DeviceTier::Desktop    => "Desktop",
        }
    }

    /// Target rendering FPS for this tier (0 = no rendering).
    pub fn target_fps(self) -> u8 {
        match self {
            DeviceTier::Smartwatch => 0,
            DeviceTier::Smartphone => 30,
            DeviceTier::Tablet     => 60,
            DeviceTier::Desktop    => 120,
        }
    }

    /// Whether this tier can display the full AICouncil vote breakdown.
    pub fn can_show_ai_council(self) -> bool { self >= DeviceTier::Tablet }

    /// Whether this tier supports the HeptaScript editor.
    pub fn can_edit_heptascript(self) -> bool { self == DeviceTier::Desktop }
}

impl core::fmt::Display for DeviceTier {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier_ordering() {
        assert!(DeviceTier::Desktop > DeviceTier::Tablet);
        assert!(DeviceTier::Tablet  > DeviceTier::Smartphone);
        assert!(DeviceTier::Smartphone > DeviceTier::Smartwatch);
    }

    #[test]
    fn fps_values() {
        assert_eq!(DeviceTier::Smartwatch.target_fps(), 0);
        assert_eq!(DeviceTier::Smartphone.target_fps(), 30);
        assert_eq!(DeviceTier::Tablet.target_fps(), 60);
        assert_eq!(DeviceTier::Desktop.target_fps(), 120);
    }

    #[test]
    fn capabilities() {
        assert!(!DeviceTier::Smartphone.can_show_ai_council());
        assert!(DeviceTier::Tablet.can_show_ai_council());
        assert!(DeviceTier::Desktop.can_show_ai_council());
        assert!(!DeviceTier::Tablet.can_edit_heptascript());
        assert!(DeviceTier::Desktop.can_edit_heptascript());
    }
}
