//! akkadi-cli — Sovereign CLI + Notebook for BahyWay.Ecosystem
//!
//! ## CLI Commands
//! ```text
//! akkadi status                          — sovereign system health
//! akkadi kaki --inspect A3F2·91CC·...   — decode 16-byte PK
//! akkadi pipeline --stats               — 4 lanes × 7 stations
//! akkadi tribe --show izi               — tribe metrics
//! akkadi compare --sovereign --legacy   — EnkiDB vs PostgreSQL
//! akkadi notebook --run cells.akknb     — execute notebook
//! ```
//!
//! ## Sovereign Constants (ADR-001 locked)
//! ```text
//! KAKI_DISPLAY_SEP  · (U+00B7)
//! GEM_QUALITY       200
//! TRIBE_QUALITY     140
//! ACTIVE_QUALITY    100
//! quality_display   min(B11 / 240.0, 1.0)
//! GEM_RATE          35.4%
//! DEAD_CEILING      58.8%
//! STEWARD_RATE       9.46%
//! ```

#![allow(missing_docs)]

pub mod commands;
pub mod config;
pub mod client;
pub mod notebook;
pub mod output;

pub use config::AkkadiConfig;
pub use notebook::cell::{CellKind, NotebookCell, CellState};
pub use output::format::OutputFormat;

// ── Sovereign constants (ADR-001, locked) ────────────────────────────────────

/// U+00B7 middle-dot separator — sovereign KAKI display format
pub const KAKI_DISPLAY_SEP: char = '\u{00B7}';

/// B11 quality byte divisor — ADR-001 (NOT 255)
pub const QUALITY_DIVISOR: f32 = 240.0;

/// Gem quality floor — B11 ≥ 200
pub const GEM_QUALITY: u8 = 200;

/// Tribe quality floor — B11 ≥ 140
pub const TRIBE_QUALITY: u8 = 140;

/// Active quality floor — B11 ≥ 100
pub const ACTIVE_QUALITY: u8 = 100;

/// Dead ceiling — B11 < 100
pub const DEAD_QUALITY: u8 = 100;

/// Unresolved particle colour
pub const UNRESOLVED_COLOR: &str = "#FF44AA";

/// Gem rate baseline (ADR-004)
pub const GEM_RATE_BASELINE: f32 = 35.4;

/// Dead ceiling baseline (ADR-004)
pub const DEAD_CEILING_BASELINE: f32 = 58.8;

/// Steward escalation baseline (ADR-004)
pub const STEWARD_RATE_BASELINE: f32 = 9.46;

/// MDM duplicate threshold (ADR-004)
pub const MDM_DUPLICATE_THRESHOLD: f32 = 0.90;

/// MDM fuzzy threshold (ADR-004)
pub const MDM_FUZZY_THRESHOLD: f32 = 0.72;

/// VGCA fragmentation constant (ADR-008)
pub const VGCA_DELTA_FRAG: f32 = 0.35;

/// VGCA minimum entropy (ADR-008)
pub const VGCA_MIN_ENTROPY: f32 = 0.50;

/// HNSW EF construction
pub const HNSW_EF_CONSTRUCTION: usize = 200;

/// RAG cosine similarity threshold
pub const RAG_COSINE_THRESHOLD: f32 = 0.65;

/// EnkiDB HTTP port
pub const ENKIDB_PORT: u16 = 9080;

/// AkkadianAOL LSP port
pub const AOL_LSP_PORT: u16 = 9100;

/// 7 Ibn Wahshiyya planetary dimensions
pub const HEPTA_DIMS: [&str; 7] = ["ME", "GU", "SAG", "IZI", "A", "UD", "URU"];

/// 7 planetary names
pub const PLANET_NAMES: [&str; 7] = ["Saturn", "Jupiter", "Mars", "Sun", "Venus", "Mercury", "Moon"];

/// 7 planet glyphs (Unicode)
pub const PLANET_GLYPHS: [&str; 7] = ["♄", "♃", "♂", "☀", "♀", "☿", "☽"];

/// 7 Baghdad district names
pub const DISTRICTS: [&str; 7] = [
    "Al-Karkh", "Al-Rusafa", "Al-Mansour",
    "Sadr City", "Al-Kadhimiya", "Zayouna", "Al-Dora",
];

// ── Quality helpers ───────────────────────────────────────────────────────────

/// Sovereign formula: `min(B11 / 240.0, 1.0)` — ADR-001
#[inline]
pub fn quality_display(b11: u8) -> f32 {
    (b11 as f32 / QUALITY_DIVISOR).min(1.0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QualityTier {
    Gem,    // B11 ≥ 200 → 💎 Golden Record
    Tribe,  // B11 ≥ 140 → ◉ Active tribal
    Active, // B11 ≥ 100 → ● In pipeline
    Dead,   // B11 <  100 → ○ blackbox
}

impl QualityTier {
    pub fn from_b11(b11: u8) -> Self {
        match b11 {
            200..=255 => Self::Gem,
            140..=199 => Self::Tribe,
            100..=139 => Self::Active,
            _          => Self::Dead,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Gem    => "💎 GEM",
            Self::Tribe  => "◉ TRIBE",
            Self::Active => "● ACTIVE",
            Self::Dead   => "○ DEAD",
        }
    }

    pub fn color_hex(self) -> &'static str {
        match self {
            Self::Gem    => "#ffd700",
            Self::Tribe  => "#00d4c8",
            Self::Active => "#ffffff",
            Self::Dead   => "#888888",
        }
    }
}

// ── B10 domain helpers ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleKind {
    Record,    // 0x00–0x7F  💎 MDM data record
    AkkScript, // 0xE0–0xEF ◆ AkkadianAOL compiled artifact
    File,      // 0xF0–0xFF  🔷 file container
    Reserved,
}

impl ParticleKind {
    pub fn from_b10(b10: u8) -> Self {
        match b10 {
            0x00..=0x7F => Self::Record,
            0xE0..=0xEF => Self::AkkScript,
            0xF0..=0xFF => Self::File,
            _            => Self::Reserved,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Record    => "💎 Record",
            Self::AkkScript => "◆ AkkScript",
            Self::File      => "🔷 File",
            Self::Reserved  => "Reserved",
        }
    }

    pub fn enkidb_cf(self) -> &'static str {
        match self {
            Self::Record    => "kaki_records",
            Self::AkkScript => "kaki_akk",
            Self::File      => "kaki_files",
            Self::Reserved  => "unknown",
        }
    }
}

// ── KAKI display format ───────────────────────────────────────────────────────

/// Format 16 raw bytes as sovereign KAKI display string.
/// Format: 8 groups of 4 hex chars separated by U+00B7
/// Example: `A3F2·91CC·4B00·1234·5678·01C8·42FF·00AB`
pub fn format_kaki(bytes: &[u8; 16]) -> String {
    let hex: String = bytes.iter().map(|b| format!("{:02X}", b)).collect();
    hex.as_bytes()
        .chunks(4)
        .map(|c| std::str::from_utf8(c).unwrap_or("????"))
        .collect::<Vec<_>>()
        .join(&KAKI_DISPLAY_SEP.to_string())
}

/// Parse a KAKI display string back to 16 bytes.
pub fn parse_kaki(s: &str) -> Result<[u8; 16], String> {
    let clean: String = s.replace('\u{00B7}', "").replace('-', "").replace(' ', "");
    if clean.len() != 32 {
        return Err(format!("KAKI must be 32 hex chars, got {}", clean.len()));
    }
    let mut out = [0u8; 16];
    for i in 0..16 {
        out[i] = u8::from_str_radix(&clean[i*2..i*2+2], 16)
            .map_err(|e| format!("Invalid hex at position {}: {}", i, e))?;
    }
    Ok(out)
}

/// Inspect a KAKI — decode all bytes with sovereign meaning.
#[derive(Debug)]
pub struct KakiInspection {
    pub raw:          [u8; 16],
    pub display:      String,
    pub b0_b6:        [u8; 7],
    pub b7:           u8,
    pub b8_b9:        [u8; 2],
    pub b10:          u8,
    pub b11:          u8,
    pub b12:          u8,
    pub b13_b15:      [u8; 3],
    pub particle_kind: ParticleKind,
    pub quality_tier:  QualityTier,
    pub brightness:    f32,
}

impl KakiInspection {
    pub fn from_bytes(raw: [u8; 16]) -> Self {
        let mut b0_b6 = [0u8; 7];
        b0_b6.copy_from_slice(&raw[0..7]);
        let b8_b9 = [raw[8], raw[9]];
        let b13_b15 = [raw[13], raw[14], raw[15]];
        Self {
            display:      format_kaki(&raw),
            b0_b6,
            b7:           raw[7],
            b8_b9,
            b10:          raw[10],
            b11:          raw[11],
            b12:          raw[12],
            b13_b15,
            particle_kind: ParticleKind::from_b10(raw[10]),
            quality_tier:  QualityTier::from_b11(raw[11]),
            brightness:    quality_display(raw[11]),
            raw,
        }
    }
}
