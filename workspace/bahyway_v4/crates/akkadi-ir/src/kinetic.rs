//! 𒁾 Kinetic Forces — df/dt = I_phys + I_mem + I_learn
//!
//! | Force    | Term   | Sovereign Source                              |
//! |----------|--------|-----------------------------------------------|
//! | Physical | I_phys | Vault — Gray-Rot, IO friction, storage        |
//! | Memory   | I_mem  | Enki + Story — EAV history, WAL replay        |
//! | Learning | I_learn| AkkadianAOL — .akk rewrite rules generated    |

/// The kind of kinetic force acting on a particle.
#[derive(Debug, Clone, PartialEq)]
pub enum ForceKind {
    Physical  { gray_rot: f32, io_friction: f32 },
    Memory    { staleness: f32 },
    Learning  { rule_confidence: f32 },
    Composite { phys: f32, mem: f32, learn: f32 },
}

impl ForceKind {
    pub fn magnitude(&self) -> f32 {
        match self {
            Self::Physical  { gray_rot, io_friction }  => -(gray_rot + io_friction) / 2.0,
            Self::Memory    { staleness }              => -*staleness,
            Self::Learning  { rule_confidence }        => *rule_confidence,
            Self::Composite { phys, mem, learn }       => phys + mem + learn,
        }
    }

    pub fn akk_keyword(&self) -> &'static str {
        match self {
            Self::Physical  { .. } => "I_phys",
            Self::Memory    { .. } => "I_mem",
            Self::Learning  { .. } => "I_learn",
            Self::Composite { .. } => "I_composite",
        }
    }
}

/// A kinetic force with its origin and target dimension.
#[derive(Debug, Clone)]
pub struct KineticForce {
    pub kind:      ForceKind,
    pub dimension: Option<crate::quality::HeptaDim>,
    pub source:    String,
}

impl KineticForce {
    pub fn physical(gray_rot: f32, io_friction: f32, source: &str) -> Self {
        Self { kind: ForceKind::Physical { gray_rot, io_friction }, dimension: None, source: source.to_string() }
    }

    pub fn memory(staleness: f32, source: &str) -> Self {
        Self { kind: ForceKind::Memory { staleness }, dimension: None, source: source.to_string() }
    }

    pub fn learning(rule_confidence: f32, source: &str) -> Self {
        Self { kind: ForceKind::Learning { rule_confidence }, dimension: None, source: source.to_string() }
    }

    pub fn df_dt(&self) -> f32 { self.kind.magnitude() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn physical_is_negative() { assert!(KineticForce::physical(0.4, 0.3, "Vault").df_dt() < 0.0); }
    #[test] fn memory_is_negative()   { assert!(KineticForce::memory(0.7, "Story").df_dt() < 0.0); }
    #[test] fn learning_is_positive() { assert!(KineticForce::learning(0.9, "AkkadianAOL").df_dt() > 0.0); }

    #[test]
    fn zero_physical_is_zero() {
        assert!((KineticForce::physical(0.0, 0.0, "Vault").df_dt()).abs() < 0.001);
    }

    #[test]
    fn akk_keywords_correct() {
        assert_eq!(ForceKind::Physical { gray_rot: 0.0, io_friction: 0.0 }.akk_keyword(), "I_phys");
        assert_eq!(ForceKind::Memory   { staleness: 0.0 }.akk_keyword(), "I_mem");
        assert_eq!(ForceKind::Learning { rule_confidence: 0.0 }.akk_keyword(), "I_learn");
    }
}
