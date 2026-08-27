//! Query-time per-namespace read authorization — the decision logic for
//! "does this HeptaScript query try to read the `passport.*` namespace,
//! and if so, does the caller's privilege_level clear the bar?"
//!
//! HONEST SCOPE, checked before writing this: this module is pure
//! decision logic, real and fully unit-tested against the actual
//! `heptascript::parser::parse_query` AST (not a string/substring
//! heuristic — a WHERE clause whose literal VALUE happens to contain the
//! text "passport." would false-positive under a substring check but is
//! handled correctly here by only inspecting `Projection.attrs` and
//! `WhereCondition.attr`, the real attribute-name fields).
//!
//! It is NOT wired to any live network boundary. Checked before building
//! this: none of the 7 read-servers (`enkidb-read-server` through
//! `enkiddb-read-server`) have TLS or any per-connection credential
//! today — only `anu-governor-web` does (PB-194/195). Wiring this
//! authorizer to `enkimdb-read-server`'s live TCP QUERY handler would
//! need one of: (a) TLS on that server plus a real per-connection
//! credential exchange (its own real, separate scope — mirroring what
//! `anu-governor-web` needed its own dedicated build pass for), or
//! (b) a shared-secret session-token mechanism between processes, which
//! `anu-governor-web`'s own design deliberately does NOT support (its
//! HMAC signing key is generated fresh in memory per process start,
//! precisely so a stolen cookie can't be forged and a restart
//! invalidates every session — sharing that key across processes would
//! be a real, live change to that security property, not a "reuse").
//! Shipping either a plaintext-passphrase-over-an-unencrypted-socket
//! protocol or a silently-unenforced "gate" label would both be worse
//! than stating this plainly: the decision logic is real and ready: the
//! transport wiring is separate, not-yet-built work.

use heptascript::query::HeptaQuery;

/// Attribute-name prefixes this authorizer treats as restricted.
/// Deliberately a short, explicit list (not a wildcard "anything with a
/// dot") — only namespaces an Architect has actually asked to be gated
/// belong here; adding one is a deliberate act, not implied by pattern.
/// `anu_governor_run.*` (2026-07-29): the run-confirmation registry
/// carries operator identity/privilege_level for every AnuGovernor run —
/// same read-sensitivity rationale as `passport.*`, gated identically.
pub const RESTRICTED_NAMESPACES: &[&str] = &["passport.", "anu_governor_run."];

/// 5 = Administrator tier in the Architect's own 5-group model
/// (Architect=7/Gilgamesh, dataSteward/Administrator/Developer=Sargon 3
/// priorities, Other-Stakeholders=temporary) — same default and same
/// rationale already used for PB-265's infra-provisioning vault gate,
/// not re-decided here.
pub const RESTRICTED_NAMESPACE_MIN_PRIVILEGE: u8 = 5;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthzDecision {
    Allowed,
    /// Denied because the query touches a restricted namespace and the
    /// caller's privilege_level (if any) doesn't clear the bar.
    Denied {
        namespace: &'static str,
        required_privilege: u8,
    },
}

/// True if any WHAT projection or WHERE condition attribute starts with
/// one of `RESTRICTED_NAMESPACES`, OR if WHAT is absent/wildcard (`[*]`,
/// `projections` empty attrs) — a wildcard SELECT cannot be proven not
/// to touch a restricted namespace, so it is conservatively treated as
/// if it does. Returns the first matching namespace prefix, if any.
pub fn query_touches_restricted_namespace(query: &HeptaQuery) -> Option<&'static str> {
    let what_is_wildcard_or_absent = match &query.what {
        None => true,
        Some(w) => w.projections.is_empty() || w.projections.iter().any(|p| p.attrs.is_empty()),
    };
    if what_is_wildcard_or_absent {
        // Conservative: cannot prove the wildcard excludes a restricted
        // namespace, so report the first restricted namespace as the
        // reason (there is always at least one, per this module's own
        // non-empty RESTRICTED_NAMESPACES contract).
        return RESTRICTED_NAMESPACES.first().copied();
    }

    let attrs_touching = |attr: &str| {
        RESTRICTED_NAMESPACES
            .iter()
            .find(|ns| attr.starts_with(*ns))
            .copied()
    };

    if let Some(what) = &query.what {
        for projection in &what.projections {
            for attr in &projection.attrs {
                if let Some(ns) = attrs_touching(attr) {
                    return Some(ns);
                }
            }
        }
    }
    for cond in &query.r#where {
        if let Some(ns) = attrs_touching(&cond.attr) {
            return Some(ns);
        }
    }
    None
}

/// Decide ALLOW/DENY for one parsed query against an optional caller
/// privilege_level (`None` = unauthenticated caller). A query that
/// doesn't touch any restricted namespace is always `Allowed`,
/// regardless of privilege_level — this authorizer only ever narrows
/// access to specific namespaces, never broadens it.
pub fn authorize(query: &HeptaQuery, caller_privilege_level: Option<u8>) -> AuthzDecision {
    match query_touches_restricted_namespace(query) {
        None => AuthzDecision::Allowed,
        Some(namespace) => {
            let ok =
                caller_privilege_level.is_some_and(|lvl| lvl >= RESTRICTED_NAMESPACE_MIN_PRIVILEGE);
            if ok {
                AuthzDecision::Allowed
            } else {
                AuthzDecision::Denied {
                    namespace,
                    required_privilege: RESTRICTED_NAMESPACE_MIN_PRIVILEGE,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use heptascript::parser::parse_query;

    #[test]
    fn plain_query_with_no_passport_attrs_is_allowed_unauthenticated() {
        let q = parse_query("WHO T.E\nWHAT E[artifact.name]\nWHERE E[artifact.kind] = \"Crate\"")
            .unwrap();
        assert_eq!(authorize(&q, None), AuthzDecision::Allowed);
    }

    #[test]
    fn what_projection_on_passport_attr_is_denied_without_privilege() {
        let q = parse_query(
            "WHO T.E\nWHAT E[passport.privilege_level]\nWHERE E[artifact.kind] = \"Crate\"",
        )
        .unwrap();
        assert_eq!(
            authorize(&q, None),
            AuthzDecision::Denied {
                namespace: "passport.",
                required_privilege: 5
            }
        );
    }

    #[test]
    fn where_condition_on_passport_attr_is_denied_without_privilege() {
        let q =
            parse_query("WHO T.E\nWHAT E[artifact.name]\nWHERE E[passport.realm] = \"bahyway\"")
                .unwrap();
        assert!(matches!(authorize(&q, None), AuthzDecision::Denied { .. }));
    }

    #[test]
    fn anu_governor_run_attr_is_denied_without_privilege_and_allowed_at_5() {
        let q = parse_query(
            "WHO T.E\nWHAT E[anu_governor_run.outcome]\nWHERE E[artifact.kind] = \"Crate\"",
        )
        .unwrap();
        assert_eq!(
            authorize(&q, None),
            AuthzDecision::Denied {
                namespace: "anu_governor_run.",
                required_privilege: 5
            }
        );
        assert_eq!(authorize(&q, Some(5)), AuthzDecision::Allowed);
        assert!(matches!(
            authorize(&q, Some(4)),
            AuthzDecision::Denied { .. }
        ));
    }

    #[test]
    fn passport_attr_query_is_allowed_at_sufficient_privilege() {
        let q = parse_query(
            "WHO T.E\nWHAT E[passport.privilege_level]\nWHERE E[artifact.kind] = \"Crate\"",
        )
        .unwrap();
        assert_eq!(authorize(&q, Some(5)), AuthzDecision::Allowed);
        assert_eq!(authorize(&q, Some(7)), AuthzDecision::Allowed);
    }

    #[test]
    fn passport_attr_query_is_still_denied_below_threshold() {
        let q = parse_query(
            "WHO T.E\nWHAT E[passport.privilege_level]\nWHERE E[artifact.kind] = \"Crate\"",
        )
        .unwrap();
        assert!(matches!(
            authorize(&q, Some(4)),
            AuthzDecision::Denied { .. }
        ));
    }

    #[test]
    fn a_value_that_merely_contains_the_word_passport_in_a_where_literal_does_not_false_positive() {
        // AST-based check: only the ATTRIBUTE name matters, not the
        // comparison VALUE -- a query filtering on an unrelated attribute
        // whose string value happens to mention "passport." must not be
        // treated as touching the restricted namespace.
        let q = parse_query(
            "WHO T.E\nWHAT E[artifact.name]\nWHERE E[artifact.name] = \"passport.txt\"",
        )
        .unwrap();
        assert_eq!(authorize(&q, None), AuthzDecision::Allowed);
    }

    #[test]
    fn wildcard_what_is_conservatively_treated_as_touching_every_restricted_namespace() {
        let q = parse_query("WHO T.E\nWHAT E[*]\nWHERE E[artifact.kind] = \"Crate\"").unwrap();
        assert!(matches!(authorize(&q, None), AuthzDecision::Denied { .. }));
        assert_eq!(authorize(&q, Some(5)), AuthzDecision::Allowed);
    }

    #[test]
    fn absent_what_clause_is_conservatively_treated_as_touching_every_restricted_namespace() {
        let q = parse_query("WHO T.E\nWHERE E[artifact.kind] = \"Crate\"").unwrap();
        assert!(matches!(authorize(&q, None), AuthzDecision::Denied { .. }));
    }
}
