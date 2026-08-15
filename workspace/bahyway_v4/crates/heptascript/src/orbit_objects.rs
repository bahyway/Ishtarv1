//! Sovereign Orbit Objects — Triple-O equivalents of database
//! objects, with NO SQL DDL. Each is a kind of meta-particle.

/// The six sovereign object kinds (replacing SQL objects).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrbitObjectKind {
    /// Replaces VIEW — an orbit materialized from a constraint.
    MaterializedOrbit,
    /// Replaces TYPE — a tribe's schema definition.
    TribeDefinition,
    /// Replaces TRIGGER — fires on an orbital transition.
    OrbitReaction,
    /// Replaces UDF — a pure function over particles.
    ParticleTransformer,
    /// Replaces STORED PROCEDURE — an ordered transformer chain.
    OrbitSequence,
    /// Replaces SNAP-IN — packaged cross-tribe logic.
    CrossTribeModule,
}

impl OrbitObjectKind {
    pub const ALL: [OrbitObjectKind; 6] = [
        OrbitObjectKind::MaterializedOrbit,
        OrbitObjectKind::TribeDefinition,
        OrbitObjectKind::OrbitReaction,
        OrbitObjectKind::ParticleTransformer,
        OrbitObjectKind::OrbitSequence,
        OrbitObjectKind::CrossTribeModule,
    ];

    pub fn sovereign_name(&self) -> &'static str {
        match self {
            OrbitObjectKind::MaterializedOrbit => "MATERIALIZED_ORBIT",
            OrbitObjectKind::TribeDefinition => "TRIBE_DEFINITION",
            OrbitObjectKind::OrbitReaction => "ORBIT_REACTION",
            OrbitObjectKind::ParticleTransformer => "PARTICLE_TRANSFORMER",
            OrbitObjectKind::OrbitSequence => "ORBIT_SEQUENCE",
            OrbitObjectKind::CrossTribeModule => "CROSSTRIBE_MODULE",
        }
    }

    /// The SQL object this sovereignly replaces (for docs only).
    pub fn replaces(&self) -> &'static str {
        match self {
            OrbitObjectKind::MaterializedOrbit => "view",
            OrbitObjectKind::TribeDefinition => "type",
            OrbitObjectKind::OrbitReaction => "trigger",
            OrbitObjectKind::ParticleTransformer => "udf",
            OrbitObjectKind::OrbitSequence => "procedure",
            OrbitObjectKind::CrossTribeModule => "snap-in",
        }
    }

    /// Is this object refreshed by the Aged Recognition Service?
    pub fn ars_refreshed(&self) -> bool {
        matches!(self, OrbitObjectKind::MaterializedOrbit)
    }
}

/// A declared sovereign object (name + kind + the W5H2 body text).
#[derive(Debug, Clone, PartialEq)]
pub struct OrbitObject {
    pub kind: OrbitObjectKind,
    pub name: String,
    pub w5h2_body: String,
}

impl OrbitObject {
    pub fn new(kind: OrbitObjectKind, name: &str, body: &str) -> Result<Self, String> {
        // Sovereign guard: reject SQL SYNTAX patterns. Note WHERE/WHEN
        // are legitimate W5H2 clause words, so we key on SQL-specific
        // constructs (SELECT..., FROM <table>, JOIN..ON, INSERT INTO).
        let upper = body.to_ascii_uppercase();
        for kw in ["SELECT ", "INSERT INTO", "UPDATE ", "DELETE FROM",
                   " FROM ", " JOIN ", " INNER JOIN", " LEFT JOIN"] {
            if upper.contains(kw) {
                return Err(format!("SQL construct '{}' forbidden in orbit object body", kw.trim()));
            }
        }
        Ok(OrbitObject { kind, name: name.to_string(), w5h2_body: body.to_string() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn six_sovereign_object_kinds() {
        assert_eq!(OrbitObjectKind::ALL.len(), 6);
    }

    #[test]
    fn each_kind_names_its_sql_replacement() {
        assert_eq!(OrbitObjectKind::MaterializedOrbit.replaces(), "view");
        assert_eq!(OrbitObjectKind::OrbitReaction.replaces(), "trigger");
        assert_eq!(OrbitObjectKind::OrbitSequence.replaces(), "procedure");
    }

    #[test]
    fn only_materialized_orbit_is_ars_refreshed() {
        assert!(OrbitObjectKind::MaterializedOrbit.ars_refreshed());
        assert!(!OrbitObjectKind::TribeDefinition.ars_refreshed());
    }

    #[test]
    fn valid_w5h2_body_accepted() {
        let o = OrbitObject::new(
            OrbitObjectKind::MaterializedOrbit,
            "golden_customers",
            "WHO customer WHAT golden WHEN active WHERE tribe=root PRESENT identity",
        );
        assert!(o.is_ok());
    }

    #[test]
    fn sql_body_rejected() {
        let o = OrbitObject::new(
            OrbitObjectKind::MaterializedOrbit,
            "bad",
            "SELECT * FROM customers",
        );
        assert!(o.is_err());
    }
}
