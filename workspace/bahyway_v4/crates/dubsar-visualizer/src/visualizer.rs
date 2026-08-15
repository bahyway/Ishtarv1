//! DubSarVisualizer — ColorID text representation for the DubSar IDE (§4.5).
//!
//! DubSar (Sumerian: "tablet-writer") is the IDE. The visualizer renders
//! ColorRgb values as ANSI 256-color escape codes and ASCII drift bars
//! for terminal / DubSar display.

use score_engine::color_id::ColorRgb;

/// A text rendering of a particle's ColorID state.
#[derive(Debug, Clone)]
pub struct ColorIdDisplay {
    /// ANSI-compatible 256-color escape sequence string.
    pub ansi_fg:    String,
    /// Short symbolic label: Golden / Drifting / Gray.
    pub label:      &'static str,
    /// Drift bar ASCII art: "▓▓▓▒▒░░░" (8 chars, full=█ empty=░).
    pub drift_bar:  String,
    /// Numeric drift distance from pure gold.
    pub drift:      f32,
}

/// Convert a ColorRgb to its display representation.
pub fn render(rgb: &ColorRgb) -> ColorIdDisplay {
    // drift_from_gray measures distance from Gray (0x80,0x80,0x80).
    // We invert: drift-from-golden = distance from (0xFF,0xFF,0xFF).
    let dr = (0xFF_u32).saturating_sub(rgb.r as u32) as f32;
    let dg = (0xFF_u32).saturating_sub(rgb.g as u32) as f32;
    let db = (0xFF_u32).saturating_sub(rgb.b as u32) as f32;
    let drift = (dr * dr + dg * dg + db * db).sqrt();
    let label = if drift < 50.0 {
        "Golden"
    } else if drift < 200.0 {
        "Drifting"
    } else {
        "Gray"
    };

    // ANSI 256-color: ESC[38;2;<r>;<g>;<b>m
    let ansi_fg = format!("\x1b[38;2;{};{};{}m", rgb.r, rgb.g, rgb.b);

    // Drift bar: 8 segments, each covers drift/37.5 (300/8=37.5 max drift range)
    let filled = ((drift / 37.5).min(8.0) as usize).min(8);
    let bar: String = (0..8)
        .map(|i| if i < filled { '█' } else { '░' })
        .collect();

    ColorIdDisplay { ansi_fg, label, drift_bar: bar, drift }
}

#[cfg(test)]
mod tests {
    use super::*;
    use score_engine::color_id::ColorRgb;

    #[test]
    fn golden_rgb_label_is_golden() {
        let d = render(&ColorRgb::GOLDEN);
        assert_eq!(d.label, "Golden");
        assert!(d.drift < 50.0);
    }

    #[test]
    fn gray_rgb_label_is_gray() {
        let d = render(&ColorRgb::GRAY);
        assert_eq!(d.label, "Gray");
        assert!(d.drift >= 200.0, "drift={}", d.drift);
    }

    #[test]
    fn drift_bar_is_8_chars() {
        let d = render(&ColorRgb { r: 200, g: 200, b: 200 });
        assert_eq!(d.drift_bar.chars().count(), 8);
    }

    #[test]
    fn golden_drift_bar_all_empty() {
        let d = render(&ColorRgb::GOLDEN);
        assert!(d.drift_bar.chars().all(|c| c == '░'),
            "golden should have zero filled segments");
    }

    #[test]
    fn ansi_escape_contains_rgb() {
        let rgb = ColorRgb { r: 100, g: 150, b: 200 };
        let d   = render(&rgb);
        assert!(d.ansi_fg.contains("100"));
        assert!(d.ansi_fg.contains("150"));
        assert!(d.ansi_fg.contains("200"));
    }
}
