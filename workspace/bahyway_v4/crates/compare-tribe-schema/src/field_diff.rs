// ============================================================================
// compare-tribe-schema/src/field_diff.rs
// Granular per-field change descriptions between two schema versions.
// ============================================================================
#![allow(missing_docs)]

use crate::schema_version::FieldType;

/// One specific change detected in a field between V1 and V2.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldChange {
    /// Field exists in V2 but not V1.
    Added,
    /// Field existed in V1 but is gone in V2 — breaking.
    Removed,
    /// Data type changed — always breaking (existing data may not coerce).
    TypeChanged { from: FieldType, to: FieldType },
    /// Max length changed — breaking if shortened (may truncate existing values).
    LengthChanged { from: Option<u16>, to: Option<u16> },
    /// Column position changed — soft (affects positional parsers only).
    PositionChanged { from: u16, to: u16 },
    /// Mandatory flag changed — breaking if false→true (existing nulls fail).
    MandatoryChanged { was: bool, now: bool },
    /// Nullable flag changed — soft if true→false in same direction as mandatory.
    NullableChanged { was: bool, now: bool },
}

impl FieldChange {
    /// True if this change can corrupt existing data or invalidate loaded particles.
    pub fn is_breaking(&self) -> bool {
        matches!(
            self,
            FieldChange::Removed
                | FieldChange::TypeChanged { .. }
                | FieldChange::MandatoryChanged {
                    was: false,
                    now: true
                }
                | FieldChange::LengthChanged {
                    from: _,
                    to: Some(_)
                } // shrinkage
        )
    }

    pub fn description(&self) -> String {
        match self {
            FieldChange::Added => "field added".to_string(),
            FieldChange::Removed => "field removed (breaking)".to_string(),
            FieldChange::TypeChanged { from, to } => {
                format!("type {} → {} (breaking)", from.as_str(), to.as_str())
            }
            FieldChange::LengthChanged { from, to } => format!("max_length {:?} → {:?}", from, to),
            FieldChange::PositionChanged { from, to } => format!("position {from} → {to}"),
            FieldChange::MandatoryChanged { was, now } => format!(
                "mandatory {was} → {now}{}",
                if !was && *now { " (breaking)" } else { "" }
            ),
            FieldChange::NullableChanged { was, now } => format!("nullable {was} → {now}"),
        }
    }
}

/// All detected changes for one field between two schema versions.
#[derive(Debug, Clone)]
pub struct FieldDiff {
    pub field_name: String,
    pub attr_hash: u32,
    pub changes: Vec<FieldChange>,
}

impl FieldDiff {
    /// True if any change in this field is breaking.
    pub fn has_breaking_change(&self) -> bool {
        self.changes.iter().any(|c| c.is_breaking())
    }

    /// Collect human-readable descriptions of all changes.
    pub fn change_descriptions(&self) -> Vec<String> {
        self.changes.iter().map(|c| c.description()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removed_is_breaking() {
        assert!(FieldChange::Removed.is_breaking());
    }

    #[test]
    fn type_change_is_breaking() {
        assert!(FieldChange::TypeChanged {
            from: FieldType::Text,
            to: FieldType::Integer
        }
        .is_breaking());
    }

    #[test]
    fn mandatory_false_to_true_is_breaking() {
        assert!(FieldChange::MandatoryChanged {
            was: false,
            now: true
        }
        .is_breaking());
    }

    #[test]
    fn mandatory_true_to_false_is_not_breaking() {
        assert!(!FieldChange::MandatoryChanged {
            was: true,
            now: false
        }
        .is_breaking());
    }

    #[test]
    fn position_change_is_not_breaking() {
        assert!(!FieldChange::PositionChanged { from: 1, to: 2 }.is_breaking());
    }

    #[test]
    fn added_is_not_breaking() {
        assert!(!FieldChange::Added.is_breaking());
    }

    #[test]
    fn field_diff_has_breaking_detects_any() {
        let diff = FieldDiff {
            field_name: "x".to_string(),
            attr_hash: 0,
            changes: vec![
                FieldChange::PositionChanged { from: 1, to: 2 },
                FieldChange::Removed,
            ],
        };
        assert!(diff.has_breaking_change());
    }

    #[test]
    fn field_diff_all_soft() {
        let diff = FieldDiff {
            field_name: "x".to_string(),
            attr_hash: 0,
            changes: vec![FieldChange::PositionChanged { from: 1, to: 3 }],
        };
        assert!(!diff.has_breaking_change());
    }
}
