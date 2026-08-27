//! The 7-database Enki-family pipeline as an ordered state machine.
//! Enki prefix is LAW: every database name begins with "Enki".

/// The seven sovereign databases, in pipeline order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnkiDb {
    EnkiSDB, // Stage — ingestion landing
    EnkiODB, // Operational — cleansing/enrichment
    EnkiQDB, // Quarantine — verification/compliance
    EnkiDB,  // Golden Records — single source of truth
    EnkiDW,  // Warehouse — analytics/snapshots
    EnkiMDB, // Metadata — internal objects
    EnkiDDB, // Documents — files (internal + external)
}

impl EnkiDb {
    /// Pipeline order 0..7.
    pub const FLOW: [EnkiDb; 7] = [
        EnkiDb::EnkiSDB,
        EnkiDb::EnkiODB,
        EnkiDb::EnkiQDB,
        EnkiDb::EnkiDB,
        EnkiDb::EnkiDW,
        EnkiDb::EnkiMDB,
        EnkiDb::EnkiDDB,
    ];

    /// The sovereign name — ALWAYS Enki-prefixed.
    pub fn name(&self) -> &'static str {
        match self {
            EnkiDb::EnkiSDB => "EnkiSDB",
            EnkiDb::EnkiODB => "EnkiODB",
            EnkiDb::EnkiQDB => "EnkiQDB",
            EnkiDb::EnkiDB => "EnkiDB",
            EnkiDb::EnkiDW => "EnkiDW",
            EnkiDb::EnkiMDB => "EnkiMDB",
            EnkiDb::EnkiDDB => "EnkiDDB",
        }
    }

    /// Sovereign port 7001..7007.
    pub fn port(&self) -> u16 {
        7001 + self.flow_index() as u16
    }

    pub fn flow_index(&self) -> usize {
        EnkiDb::FLOW.iter().position(|d| d == self).unwrap()
    }

    /// The next database in the ETL flow (None past EnkiDDB).
    pub fn next(&self) -> Option<EnkiDb> {
        let i = self.flow_index();
        EnkiDb::FLOW.get(i + 1).copied()
    }
}

/// Particle lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleState {
    Birth,  // just extracted, KAKI minted at ingest
    Fuzzy,  // schema conflict, awaiting DataSteward (gray)
    Active, // passed, in operation (green + tribe shadow)
    Golden, // single source of truth in EnkiDB
    Aged,   // ARS-marked, moved to aged partition in EnkiDW
}

impl ParticleState {
    /// Canonical ColorID (RGB) for the state's shadow system.
    pub fn color_rgb(&self) -> (u8, u8, u8) {
        match self {
            ParticleState::Birth => (128, 128, 128), // gray-born
            ParticleState::Fuzzy => (128, 128, 128), // gray (await steward)
            ParticleState::Active => (68, 255, 136), // green
            ParticleState::Golden => (255, 215, 0),  // gold
            ParticleState::Aged => (90, 90, 120),    // dim
        }
    }

    /// Legal transitions (the Architect's sealed flow).
    pub fn can_advance_to(&self, next: ParticleState) -> bool {
        use ParticleState::*;
        matches!(
            (self, next),
            (Birth, Fuzzy) | (Birth, Active) |
            (Fuzzy, Active) | (Fuzzy, Birth) |     // steward may return to birth
            (Active, Golden) |
            (Golden, Aged)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seven_databases_all_enki_prefixed() {
        for db in EnkiDb::FLOW {
            assert!(
                db.name().starts_with("Enki"),
                "{} must be Enki-prefixed",
                db.name()
            );
        }
    }

    #[test]
    fn ports_are_7001_to_7007() {
        assert_eq!(EnkiDb::EnkiSDB.port(), 7001);
        assert_eq!(EnkiDb::EnkiDDB.port(), 7007);
    }

    #[test]
    fn flow_order_is_correct() {
        assert_eq!(EnkiDb::EnkiSDB.next(), Some(EnkiDb::EnkiODB));
        assert_eq!(EnkiDb::EnkiDB.next(), Some(EnkiDb::EnkiDW));
        assert_eq!(EnkiDb::EnkiDDB.next(), None);
    }

    #[test]
    fn golden_path_transitions_are_legal() {
        use ParticleState::*;
        assert!(Birth.can_advance_to(Active));
        assert!(Active.can_advance_to(Golden));
        assert!(Golden.can_advance_to(Aged));
    }

    #[test]
    fn illegal_transitions_rejected() {
        use ParticleState::*;
        assert!(!Birth.can_advance_to(Golden)); // must pass through gates
        assert!(!Aged.can_advance_to(Active)); // aged is terminal-ish
        assert!(!Golden.can_advance_to(Birth));
    }

    #[test]
    fn steward_may_return_fuzzy_to_birth() {
        assert!(ParticleState::Fuzzy.can_advance_to(ParticleState::Birth));
    }

    #[test]
    fn fuzzy_and_birth_are_gray() {
        assert_eq!(ParticleState::Fuzzy.color_rgb(), (128, 128, 128));
        assert_eq!(ParticleState::Golden.color_rgb(), (255, 215, 0));
    }
}
