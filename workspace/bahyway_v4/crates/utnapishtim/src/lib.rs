//! UTNAPISHTIM 𒌓𒍣𒅁𒀭
//! "He Who Found Life / He Who Saw Life"
//! Sovereign Application Factory — BahyWay.Ecosystem v4.0
//! DUB.SAR 𒁾
//!
//! SOVEREIGN RULES (sealed permanently):
//!   B11 = round(eav_quality × 240.0) — Plimpton 322 — NEVER 255
//!   ColourID from PĀŠIRU via golden angle — NEVER from KAKI bytes
//!   κ[6] = kaki_type_byte  κ[7] = kaki_role_byte  (never quality)
//!   Dead particles → #404040 dark grey
//!   KUR terminal   → #1A0A2E sovereign indigo
//!   #800000 Maroon → NERGAL AV engine ONLY — never used here
//!   Orbital radius → particle property (OrbitalSubtype via EAV)
//!                    NOT a tribe property

#![forbid(unsafe_code)]

pub mod generator;
pub mod godot;
pub mod manifest;
pub mod repository;
pub mod threejs;

// ── Sovereign constants ───────────────────────────────────────

/// B11 divisor — Plimpton 322. NEVER 255.
pub const PLIMPTON_322_DIVISOR: f32 = 240.0;

/// Golden angle for tribe ColourID — PĀŠIRU
/// hue = (tribe_id - 1) × 137.508° mod 360°
pub const GOLDEN_ANGLE_DEG: f64 = 137.507_764;

/// KUR threshold — quality below this → particle in Great Below
pub const KUR_THRESHOLD: f32 = 0.40;

/// Seven sealed orbital radii (particle property — NOT tribe property)
pub const ORBITAL_GOLDEN_GEM: f32 = 0.10;
pub const ORBITAL_GOLDEN_ALIVE: f32 = 0.30;
pub const ORBITAL_FUZZY_AGED: f32 = 0.40;
pub const ORBITAL_FUZZY_GRAY: f32 = 0.57;
pub const ORBITAL_FUZZY_DECAY: f32 = 0.72;
pub const ORBITAL_DEAD_EXPIRED: f32 = 0.85;
pub const ORBITAL_DEAD_SEALED: f32 = 0.95;

/// Sovereign colour constants
pub const COLOUR_GOLDEN_GEM: &str = "#FFD700"; // pure gold
pub const COLOUR_DEAD_EXPIRED: &str = "#404040"; // dark grey
pub const COLOUR_DEAD_SEALED: &str = "#282828"; // near-black PUZRU
pub const COLOUR_KUR: &str = "#1A0A2E"; // sovereign indigo
                                        // NOTE: #800000 Maroon = NERGAL AV engine — never used in UTNAPISHTIM

/// Compute B11 from EAV quality — Plimpton 322
pub fn compute_b11(eav_quality: f32) -> u8 {
    (eav_quality.clamp(0.0, 1.0) * PLIMPTON_322_DIVISOR).round() as u8
}

/// Compute golden-angle hue for a tribe (PĀŠIRU)
pub fn golden_hue(tribe_id: u16) -> f64 {
    ((tribe_id as f64 - 1.0) * GOLDEN_ANGLE_DEG) % 360.0
}

/// OrbitalSubtype → radius (particle property — not tribe property)
pub fn orbital_radius(subtype: u8) -> f32 {
    match subtype {
        0 => ORBITAL_GOLDEN_GEM,
        1 => ORBITAL_GOLDEN_ALIVE,
        2 => ORBITAL_FUZZY_AGED,
        3 => ORBITAL_FUZZY_GRAY,
        4 => ORBITAL_FUZZY_DECAY,
        5 => ORBITAL_DEAD_EXPIRED,
        6 => ORBITAL_DEAD_SEALED,
        _ => ORBITAL_GOLDEN_ALIVE,
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TribeDefinition {
    pub tribe_id: u16,
    pub tribe_name: String,
    /// hue = golden_hue(tribe_id) — PĀŠIRU, never from KAKI
    pub hue_deg: f64,
    /// RGB hex from PĀŠIRU — never from KAKI bytes
    pub colour_hex: String,
    /// Visual ring radius in Three.js/Godot (decorative only)
    /// NOT the same as particle orbital_radius
    pub ring_radius: f32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ClientTopology {
    pub client_id: u16,
    pub client_name: String,
    pub tribes: Vec<TribeDefinition>,
    pub sealed_at: i64,
}

impl ClientTopology {
    pub fn validate(&self) -> Result<(), String> {
        if self.tribes.is_empty() {
            return Err("topology must have at least one tribe".into());
        }
        if self.client_name.is_empty() {
            return Err("client_name must not be empty".into());
        }
        Ok(())
    }

    /// Build tribes with sovereign PĀŠIRU colours
    pub fn build_tribes(client_id: u16, names: &[&str]) -> Vec<TribeDefinition> {
        let _ = client_id;
        names
            .iter()
            .enumerate()
            .map(|(i, name)| {
                let tribe_id = (i + 1) as u16;
                let hue = golden_hue(tribe_id);
                let colour = hue_to_hex(hue);
                TribeDefinition {
                    tribe_id,
                    tribe_name: name.to_string(),
                    hue_deg: hue,
                    colour_hex: colour,
                    // Decorative ring radius — visual separation only
                    // NOT particle orbital_radius
                    ring_radius: 1.5 + (i as f32 * 1.2),
                }
            })
            .collect()
    }
}

pub fn hue_to_hex(hue: f64) -> String {
    let h = hue / 60.0;
    let s = 0.75f64;
    let l = 0.55f64;
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h % 2.0) - 1.0).abs());
    let m = l - c / 2.0;
    let (r, g, b) = match h as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    format!(
        "#{:02X}{:02X}{:02X}",
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_b11_plimpton_322_not_255() {
        // B11 MUST use 240, never 255
        assert_eq!(compute_b11(1.0), 240);
        assert_eq!(compute_b11(0.5), 120);
        assert_ne!((1.0f32 * 255.0).round() as u8, compute_b11(1.0));
    }

    #[test]
    fn test_dead_particle_not_maroon() {
        // #800000 Maroon = NERGAL AV engine ONLY
        // Dead particles = #404040 dark grey
        assert_eq!(COLOUR_DEAD_EXPIRED, "#404040");
        assert_ne!(COLOUR_DEAD_EXPIRED, "#800000");
    }

    #[test]
    fn test_kur_colour_sovereign_indigo() {
        assert_eq!(COLOUR_KUR, "#1A0A2E");
        assert_ne!(COLOUR_KUR, "#800000");
    }

    #[test]
    fn test_orbital_radius_is_particle_not_tribe_property() {
        // Tribe has ring_radius (decorative)
        // Particle has orbital_radius (from OrbitalSubtype via EAV)
        // They are different concepts
        assert_eq!(orbital_radius(0), ORBITAL_GOLDEN_GEM);
        assert_eq!(orbital_radius(1), ORBITAL_GOLDEN_ALIVE);
        assert_eq!(orbital_radius(6), ORBITAL_DEAD_SEALED);
    }

    #[test]
    fn test_golden_angle_tribe_colour() {
        let h1 = golden_hue(1);
        let h2 = golden_hue(2);
        assert!((h1 - 0.0).abs() < 0.001);
        assert!((h2 - 137.507_764).abs() < 0.001);
        // Same tribe_id always same colour — everywhere
        assert_eq!(golden_hue(5), golden_hue(5));
    }

    #[test]
    fn test_topology_validate() {
        let topo = ClientTopology {
            client_id: 1,
            client_name: "Najaf Cemetery".into(),
            tribes: ClientTopology::build_tribes(1, &["Graves", "Records"]),
            sealed_at: 0,
        };
        assert!(topo.validate().is_ok());
    }
}
