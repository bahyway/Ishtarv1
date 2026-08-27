//! Particle — the 7D EAV row type.

use akkvalue::AkkValue;
use core::fmt;
use enkidb_kaki::IdentityKaki;

// ── Constants ─────────────────────────────────────────────────────────────────

/// MVP shard: single-shard operation.
pub const SHARD_DEFAULT: u32 = 0;
/// MVP proof context: no proof required.
pub const PROOF_CTX_NONE: u32 = 0;
/// Base particle (not a materialized view).
pub const MATERIALIZATION_BASE: u32 = 0;

// ── Particle ──────────────────────────────────────────────────────────────────

/// One 7D EAV assertion: (E, A, V, T, S, Z, M).
///
/// Particles are immutable once minted — they model the append-only event log.
/// "Updating" an attribute means minting a new Particle at a later T.
#[derive(Debug, Clone)]
pub struct Particle {
    /// E — entity this assertion is about.
    pub entity: IdentityKaki,
    /// A — which attribute is being asserted.
    pub attribute: String,
    /// V — the asserted value.
    pub value: AkkValue,
    /// T — time of this assertion (UNIX epoch seconds).
    pub timestamp: u64,
    /// S — shard (0 at MVP).
    pub shard: u32,
    /// Z — proof context (0 = unverified, non-zero = proof ID at MVP).
    pub proof_ctx: u32,
    /// M — materialization tag (0 = base particle, non-zero = derived view).
    pub mat_tag: u32,
}

impl Particle {
    /// Construct a base particle (S=0, Z=0, M=0) — the common MVP path.
    pub fn base(
        entity: IdentityKaki,
        attribute: impl Into<String>,
        value: AkkValue,
        timestamp: u64,
    ) -> Self {
        Self {
            entity,
            attribute: attribute.into(),
            value,
            timestamp,
            shard: SHARD_DEFAULT,
            proof_ctx: PROOF_CTX_NONE,
            mat_tag: MATERIALIZATION_BASE,
        }
    }

    /// Full constructor — all 7 dimensions explicit.
    pub fn new(
        entity: IdentityKaki,
        attribute: impl Into<String>,
        value: AkkValue,
        timestamp: u64,
        shard: u32,
        proof_ctx: u32,
        mat_tag: u32,
    ) -> Self {
        Self {
            entity,
            attribute: attribute.into(),
            value,
            timestamp,
            shard,
            proof_ctx,
            mat_tag,
        }
    }

    /// Returns `true` if this is a base particle (not materialized).
    #[inline]
    pub fn is_base(&self) -> bool {
        self.mat_tag == MATERIALIZATION_BASE
    }

    /// Returns `true` if proof context is unverified (Z=0).
    #[inline]
    pub fn is_unverified(&self) -> bool {
        self.proof_ctx == PROOF_CTX_NONE
    }

    /// Returns `true` if this particle holds a null value.
    #[inline]
    pub fn is_null(&self) -> bool {
        self.value.is_null()
    }

    /// Returns the attribute key (E, A) tuple for grouping current-value queries.
    pub fn entity_attr_key(&self) -> (&IdentityKaki, &str) {
        (&self.entity, &self.attribute)
    }
}

impl fmt::Display for Particle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "P({entity}, {attr}, {value}, t={ts})",
            entity = self.entity,
            attr = self.attribute,
            value = self.value,
            ts = self.timestamp,
        )
    }
}

// ── ParticleBirthState ────────────────────────────────────────────────────────

/// The set of birth-state attributes for an entity — written once, sealed, immutable.
///
/// This models the KAKI-vs-EAV separation: identity is in KAKI; context is in EAV.
/// Birth-state attributes are written atomically at the entity's creation epoch
/// and never re-written under the same attribute names.
///
/// Sealing discipline: Way declares the immutability rule; HeptaScript enforces
/// at compile time; Nabu enforces at runtime; seal hash detects tampering.
#[derive(Debug, Clone)]
pub struct ParticleBirthState {
    /// The entity this birth state belongs to.
    pub entity: IdentityKaki,
    /// Creation epoch (all birth-state particles share this T).
    pub born_at: u64,
    /// The birth-state attribute assertions (sealed at born_at).
    pub attributes: Vec<(String, AkkValue)>,
}

impl ParticleBirthState {
    pub fn new(entity: IdentityKaki, born_at: u64) -> Self {
        Self {
            entity,
            born_at,
            attributes: Vec::new(),
        }
    }

    /// Add a birth-state attribute. Call only during entity creation.
    pub fn with(mut self, attribute: impl Into<String>, value: AkkValue) -> Self {
        self.attributes.push((attribute.into(), value));
        self
    }

    /// Materialise as Particles, all sharing born_at as timestamp.
    pub fn into_particles(self) -> Vec<Particle> {
        self.attributes
            .into_iter()
            .map(|(attr, value)| Particle::base(self.entity, attr, value, self.born_at))
            .collect()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use akkvalue::AkkValue;
    use bahyway_core::TribeId;
    use enkidb_kaki::{mint::KakiMinter, KakiRole};

    fn make_kaki() -> IdentityKaki {
        let m = KakiMinter::new(TribeId::from_u16(0x0001));
        IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap()
    }

    #[test]
    fn base_particle_defaults() {
        let k = make_kaki();
        let p = Particle::base(
            k,
            "quality.score",
            AkkValue::QualityScore(200),
            1_700_000_000,
        );
        assert!(p.is_base());
        assert!(p.is_unverified());
        assert!(!p.is_null());
        assert_eq!(p.shard, 0);
        assert_eq!(p.proof_ctx, 0);
        assert_eq!(p.mat_tag, 0);
    }

    #[test]
    fn null_particle_detection() {
        let k = make_kaki();
        let p = Particle::base(k, "maybe.empty", AkkValue::Null, 0);
        assert!(p.is_null());
    }

    #[test]
    fn full_7d_constructor() {
        let k = make_kaki();
        let p = Particle::new(k, "geo.coord", AkkValue::Int(42), 100, 3, 7, 1);
        assert_eq!(p.shard, 3);
        assert_eq!(p.proof_ctx, 7);
        assert_eq!(p.mat_tag, 1);
        assert!(!p.is_base());
        assert!(!p.is_unverified());
    }

    #[test]
    fn display_shows_key_fields() {
        let k = make_kaki();
        let p = Particle::base(k, "name.given", AkkValue::Text("Bahaa".into()), 999);
        let s = p.to_string();
        assert!(s.contains("name.given"), "missing attr: {s}");
        assert!(s.contains("Bahaa"), "missing value: {s}");
        assert!(s.contains("t=999"), "missing timestamp: {s}");
    }

    #[test]
    fn entity_attr_key() {
        let k = make_kaki();
        let p = Particle::base(k, "tribe.id", AkkValue::Int(1), 0);
        let (_, attr) = p.entity_attr_key();
        assert_eq!(attr, "tribe.id");
    }

    #[test]
    fn birth_state_into_particles() {
        let k = make_kaki();
        let bst = ParticleBirthState::new(k, 42_000)
            .with("birth.creator", AkkValue::Text("system".into()))
            .with("birth.tribe", AkkValue::Int(1));
        let particles = bst.into_particles();
        assert_eq!(particles.len(), 2);
        for p in &particles {
            assert_eq!(p.timestamp, 42_000);
            assert!(p.is_base());
        }
        assert_eq!(particles[0].attribute, "birth.creator");
        assert_eq!(particles[1].attribute, "birth.tribe");
    }

    #[test]
    fn quality_score_stays_in_0_240() {
        // ADR-001: quality_byte ∈ [0, 240], not 255
        let k = make_kaki();
        let p = Particle::base(k, "quality.quality_byte", AkkValue::QualityScore(240), 0);
        if let AkkValue::QualityScore(q) = p.value {
            assert!(q <= 240, "quality byte exceeded 240: {q}");
        } else {
            panic!("wrong value type");
        }
    }

    #[test]
    fn geo_coordinate_stored_as_akkvalue() {
        use akkvalue::AkkCoordinate;
        let k = make_kaki();
        let geo = AkkCoordinate::new(32.0031, 44.3282); // Najaf, Iraq
        let p = Particle::base(k, "location.coord", AkkValue::Coordinate(geo), 0);
        if let AkkValue::Coordinate(c) = &p.value {
            assert!((c.lat - 32.0031).abs() < 1e-6);
        } else {
            panic!("wrong value type");
        }
    }
}
