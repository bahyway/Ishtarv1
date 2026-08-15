//! Scalar expression nodes used in Filter and Projection plans.

#![forbid(unsafe_code)]

/// A scalar expression that can appear in a filter predicate or projection.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// A literal 64-bit unsigned integer.
    LitU64(u64),
    /// A literal UTF-8 string.
    LitStr(String),
    /// A literal boolean.
    LitBool(bool),
    /// Reference to a named field (entity, attr, value, time, shard, proof_ctx, materialization).
    Field(Field),
    /// Equality test: left = right.
    Eq(Box<Expr>, Box<Expr>),
    /// Less-than: left < right.
    Lt(Box<Expr>, Box<Expr>),
    /// Greater-than: left > right.
    Gt(Box<Expr>, Box<Expr>),
    /// Logical AND.
    And(Box<Expr>, Box<Expr>),
    /// Logical OR.
    Or(Box<Expr>, Box<Expr>),
    /// Logical NOT.
    Not(Box<Expr>),
}

/// Named fields from the 7D EAV tuple: (E, A, V, T, S, Z, M).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Field {
    /// Entity identifier.
    Entity,
    /// Attribute identifier.
    Attr,
    /// Value (raw bytes, projected as u64 when used in comparisons).
    Value,
    /// Logical timestamp.
    Time,
    /// Shard identifier.
    Shard,
    /// Proof context (zk circuit selector).
    ProofCtx,
    /// Materialization epoch.
    Materialization,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eq_expr() {
        let e = Expr::Eq(
            Box::new(Expr::Field(Field::Entity)),
            Box::new(Expr::LitU64(42)),
        );
        assert!(matches!(e, Expr::Eq(_, _)));
    }

    #[test]
    fn test_and_expr() {
        let lhs = Expr::LitBool(true);
        let rhs = Expr::LitBool(false);
        let e = Expr::And(Box::new(lhs), Box::new(rhs));
        assert!(matches!(e, Expr::And(_, _)));
    }
}
