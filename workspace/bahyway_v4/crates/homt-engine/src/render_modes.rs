//! PA-16 Multi-Scale Particle Rendering — mode selection by shell density.
//!
//! At billion-particle scale, individual point sprites cannot be rendered for
//! every particle without overwhelming the GPU.  PA-16 defines three rendering
//! modes based on shell particle count, with a clean transition protocol that
//! **never loses individual particle KAKI accessibility**.
//!
//! Reference: Nabu_Markdown_Tool.md Theorem PA-16.
//!
//! # Rendering thresholds
//! ```text
//! |Σ_k| < POINT_SPRITE_THRESHOLD   → PointSprite  (fully individual)
//! |Σ_k| < INSTANCED_THRESHOLD      → Instanced    (GPU-batched)
//! |Σ_k| ≥ INSTANCED_THRESHOLD      → Volumetric   (density field + ray-march)
//! ```
//!
//! **Storytelling Preservation (Theorem PA-16 guarantee):** In all three modes
//! every particle retains a queryable KAKI.  Volumetric rendering is a
//! *perceptual compression*, not a data loss.  Clicking into a volumetric region
//! triggers a zoom-in that re-renders at Instanced or PointSprite resolution.

#![forbid(unsafe_code)]

/// Maximum shell particle count for fully-individual PointSprite rendering.
///
/// Below this threshold every particle is a separately-rendered textured quad.
/// Click-to-storytelling is direct (no zoom-in required).
pub const POINT_SPRITE_THRESHOLD: usize = 10_000;

/// Maximum shell particle count for GPU-instanced batch rendering.
///
/// Above `POINT_SPRITE_THRESHOLD` and below this, particles are packed into
/// a single instanced draw call.  Each instance carries KAKI-derived attributes
/// for color and position.  Click-to-storytelling via a picking buffer.
pub const INSTANCED_THRESHOLD: usize = 1_000_000;

/// The rendering strategy selected for a given orbital shell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderingMode {
    /// Fully individual particles as textured quads.
    ///
    /// Every particle visible and separately clickable.
    /// Used for small tribes, inner gold-quality shells, and zoomed-in views.
    PointSprite,

    /// GPU-side instanced rendering via a single draw call.
    ///
    /// Each instance carries KAKI-derived (r, g, b, position) attributes.
    /// Click-to-storytelling supported via picking buffer.
    Instanced,

    /// 3D density-field accumulation rendered via ray-marching.
    ///
    /// Individual particles blur into a volumetric cloud whose intensity at
    /// each point is proportional to local particle density.  The *shape*
    /// of the cloud encodes tribe health — tight bright center = healthy,
    /// thick sparse outer rim = many dead particles.
    ///
    /// Click-to-storytelling: clicking a region triggers zoom-in, which
    /// re-renders that sub-region in Instanced or PointSprite mode.
    Volumetric,
}

impl RenderingMode {
    /// Returns `true` if every individual particle is directly clickable
    /// without a zoom-in transition.
    pub fn supports_direct_click(&self) -> bool {
        matches!(self, RenderingMode::PointSprite | RenderingMode::Instanced)
    }

    /// Returns `true` if this mode requires a GPU compute shader for
    /// density accumulation.
    pub fn requires_compute_pass(&self) -> bool {
        matches!(self, RenderingMode::Volumetric)
    }

    /// Returns a short label for the DubSar IDE render-mode status bar.
    pub fn label(&self) -> &'static str {
        match self {
            RenderingMode::PointSprite => "PointSprite",
            RenderingMode::Instanced   => "Instanced",
            RenderingMode::Volumetric  => "Volumetric",
        }
    }
}

/// Select the appropriate rendering mode for a shell containing `particle_count` particles.
///
/// This is the PA-16 mode selector.  The caller (wgpu render loop) uses this
/// to choose between draw strategies per orbital shell per frame.
///
/// # Arguments
/// * `particle_count` — number of particles currently binned into this shell
///
/// # Returns
/// * `PointSprite`  if `particle_count < POINT_SPRITE_THRESHOLD`
/// * `Instanced`    if `POINT_SPRITE_THRESHOLD ≤ particle_count < INSTANCED_THRESHOLD`
/// * `Volumetric`   if `particle_count ≥ INSTANCED_THRESHOLD`
pub fn rendering_mode(particle_count: usize) -> RenderingMode {
    if particle_count < POINT_SPRITE_THRESHOLD {
        RenderingMode::PointSprite
    } else if particle_count < INSTANCED_THRESHOLD {
        RenderingMode::Instanced
    } else {
        RenderingMode::Volumetric
    }
}

/// Estimate rendering mode for the entire tribe based on total particle count
/// and expected shell distribution (uniform approximation).
///
/// Each tribe is decomposed into `n_shells` shells.  With uniform distribution,
/// the average shell has `total / n_shells` particles.  The *worst* (densest)
/// shell determines the most expensive mode required.
///
/// In practice the inner shells are denser — this is a conservative estimate.
pub fn tribe_rendering_mode(total_particles: usize, n_shells: usize) -> RenderingMode {
    if n_shells == 0 {
        return RenderingMode::PointSprite;
    }
    // Dense inner shells have more particles than average → use total as proxy
    // for the densest shell (conservative: whole tribe in one shell)
    rendering_mode(total_particles)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_particles_is_point_sprite() {
        assert_eq!(rendering_mode(0), RenderingMode::PointSprite);
    }

    #[test]
    fn test_below_threshold_is_point_sprite() {
        assert_eq!(rendering_mode(POINT_SPRITE_THRESHOLD - 1), RenderingMode::PointSprite);
    }

    #[test]
    fn test_at_sprite_threshold_is_instanced() {
        assert_eq!(rendering_mode(POINT_SPRITE_THRESHOLD), RenderingMode::Instanced);
    }

    #[test]
    fn test_mid_range_is_instanced() {
        let mid = (POINT_SPRITE_THRESHOLD + INSTANCED_THRESHOLD) / 2;
        assert_eq!(rendering_mode(mid), RenderingMode::Instanced);
    }

    #[test]
    fn test_at_instanced_threshold_is_volumetric() {
        assert_eq!(rendering_mode(INSTANCED_THRESHOLD), RenderingMode::Volumetric);
    }

    #[test]
    fn test_billion_is_volumetric() {
        assert_eq!(rendering_mode(1_000_000_000), RenderingMode::Volumetric);
    }

    #[test]
    fn test_direct_click_modes() {
        assert!(RenderingMode::PointSprite.supports_direct_click());
        assert!(RenderingMode::Instanced.supports_direct_click());
        assert!(!RenderingMode::Volumetric.supports_direct_click());
    }

    #[test]
    fn test_compute_pass_only_for_volumetric() {
        assert!(!RenderingMode::PointSprite.requires_compute_pass());
        assert!(!RenderingMode::Instanced.requires_compute_pass());
        assert!(RenderingMode::Volumetric.requires_compute_pass());
    }

    #[test]
    fn test_labels_non_empty() {
        assert!(!RenderingMode::PointSprite.label().is_empty());
        assert!(!RenderingMode::Instanced.label().is_empty());
        assert!(!RenderingMode::Volumetric.label().is_empty());
    }

    #[test]
    fn test_tribe_mode_zero_shells() {
        assert_eq!(tribe_rendering_mode(5000, 0), RenderingMode::PointSprite);
    }

    #[test]
    fn test_tribe_mode_billion_particles() {
        assert_eq!(tribe_rendering_mode(1_000_000_000, 5), RenderingMode::Volumetric);
    }

    #[test]
    fn test_threshold_constants_ordered() {
        assert!(POINT_SPRITE_THRESHOLD < INSTANCED_THRESHOLD);
    }
}
