//! NullSemantics — EAV null handling for HeptaScript queries.
//!
//! In EnkiDB, NULL has row-absence semantics: an attribute is NULL if no Particle
//! exists for that (E, A) pair, OR if the latest Particle has AkkValue::Null.
//! This is different from SQL NULL-in-column — here null means "not asserted".

/// Describes how a null result arises in an EAV query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NullSemantics {
    /// No particle exists for (E, A) — the attribute was never asserted.
    RowAbsent,
    /// A particle exists but its value is AkkValue::Null — explicitly set to null.
    ExplicitNull,
    /// A particle exists with a non-null value — not null.
    Present,
}

impl NullSemantics {
    /// Returns `true` for either form of null (row-absent or explicit).
    pub fn is_null(&self) -> bool {
        matches!(self, Self::RowAbsent | Self::ExplicitNull)
    }

    /// Returns `true` only for explicit null (AkkValue::Null was stored).
    pub fn is_explicit_null(&self) -> bool {
        matches!(self, Self::ExplicitNull)
    }

    /// Returns `true` only for row-absent null.
    pub fn is_row_absent(&self) -> bool {
        matches!(self, Self::RowAbsent)
    }

    /// Classify from an optional particle value.
    /// Pass `None` for row-absent, `Some(&value)` for present-or-explicit-null.
    pub fn from_option(v: Option<&akkvalue::AkkValue>) -> Self {
        match v {
            None => Self::RowAbsent,
            Some(val) if val.is_null() => Self::ExplicitNull,
            Some(_) => Self::Present,
        }
    }

    /// HeptaScript query hint string for this null kind.
    pub fn query_form(&self) -> &'static str {
        match self {
            Self::RowAbsent => "NOT EXISTS",
            Self::ExplicitNull => "IS NULL",
            Self::Present => "IS NOT NULL",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use akkvalue::AkkValue;

    #[test]
    fn none_is_row_absent() {
        let n = NullSemantics::from_option(None);
        assert!(n.is_null());
        assert!(n.is_row_absent());
        assert!(!n.is_explicit_null());
        assert_eq!(n.query_form(), "NOT EXISTS");
    }

    #[test]
    fn explicit_null_is_null() {
        let n = NullSemantics::from_option(Some(&AkkValue::Null));
        assert!(n.is_null());
        assert!(n.is_explicit_null());
        assert!(!n.is_row_absent());
        assert_eq!(n.query_form(), "IS NULL");
    }

    #[test]
    fn present_value_is_not_null() {
        let n = NullSemantics::from_option(Some(&AkkValue::Int(42)));
        assert!(!n.is_null());
        assert_eq!(n, NullSemantics::Present);
        assert_eq!(n.query_form(), "IS NOT NULL");
    }

    #[test]
    fn bool_present_is_not_null() {
        let n = NullSemantics::from_option(Some(&AkkValue::Bool(false)));
        assert!(!n.is_null());
    }
}
