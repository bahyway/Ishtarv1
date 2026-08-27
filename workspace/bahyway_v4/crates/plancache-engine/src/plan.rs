//! Section A — sealed plans: a query compiled ONCE at design time, runtime
//! loads it and never re-plans. The admission law is real and enforced
//! here: a plan whose access paths include a full scan is refused sealing,
//! full stop — regardless of which specific index would have covered it.

/// How one clause of a query resolves against storage. `IndexCovered` means
/// the caller (the real planner, once wired) asserts this clause resolves
/// via a named index; `FullScan` means it would degrade to scanning the raw
/// journal or an unindexed segment. Only the caller who actually built the
/// plan can know which — this type does not guess.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccessPath {
    IndexCovered { index_name: String },
    FullScan,
}

/// A plan that has passed the admission law but is not yet sealed. The only
/// way to get a `SealedPlan` is through `seal_with` on one of these — there
/// is no other constructor for `SealedPlan`.
#[derive(Debug, Clone)]
pub struct UnsealedPlan {
    pub source_query: String,
    pub access_paths: Vec<AccessPath>,
}

/// Why a plan was refused sealing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanSealError {
    /// LAW: a plan requiring a raw full scan may never be sealed. Lists the
    /// zero-based indices of the offending access paths.
    RequiresFullScan { offending_clauses: Vec<usize> },
}

impl UnsealedPlan {
    /// Admission check (Section A.3, corrected): refuses a plan with any
    /// `FullScan` access path. Never guesses which index would have
    /// covered a clause — that's the real planner's job, not this type's.
    pub fn try_new(
        source_query: impl Into<String>,
        access_paths: Vec<AccessPath>,
    ) -> Result<Self, PlanSealError> {
        let offending: Vec<usize> = access_paths
            .iter()
            .enumerate()
            .filter(|(_, p)| matches!(p, AccessPath::FullScan))
            .map(|(i, _)| i)
            .collect();
        if !offending.is_empty() {
            return Err(PlanSealError::RequiresFullScan {
                offending_clauses: offending,
            });
        }
        Ok(UnsealedPlan {
            source_query: source_query.into(),
            access_paths,
        })
    }

    /// Seal with a caller-supplied signing function (e.g. `kupru`'s Ed25519
    /// signing, once wired) over the plan's canonical byte form.
    pub fn seal_with(self, sign: impl FnOnce(&[u8]) -> Vec<u8>) -> SealedPlan {
        let bytes = self.canonical_bytes();
        let seal = sign(&bytes);
        SealedPlan {
            source_query: self.source_query,
            access_paths: self.access_paths,
            seal,
        }
    }

    fn canonical_bytes(&self) -> Vec<u8> {
        let mut out = self.source_query.clone().into_bytes();
        for p in &self.access_paths {
            match p {
                AccessPath::IndexCovered { index_name } => {
                    out.extend_from_slice(b"|IDX:");
                    out.extend_from_slice(index_name.as_bytes());
                }
                AccessPath::FullScan => out.extend_from_slice(b"|SCAN"),
            }
        }
        out
    }
}

/// A sealed plan: runtime loads this and never re-plans. The seal is an
/// opaque signature over the plan's canonical bytes; verifying it is the
/// caller's responsibility (e.g. via the same `kupru` key that signed it).
#[derive(Debug, Clone)]
pub struct SealedPlan {
    pub source_query: String,
    pub access_paths: Vec<AccessPath>,
    pub seal: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admits_a_fully_index_covered_plan() {
        let plan = UnsealedPlan::try_new(
            "WHO T.E\nWHAT E[quality]\nWHERE E[tribe] = 1",
            vec![AccessPath::IndexCovered {
                index_name: "surrogate".into(),
            }],
        );
        assert!(plan.is_ok());
    }

    #[test]
    fn refuses_a_plan_requiring_full_scan() {
        let plan = UnsealedPlan::try_new(
            "WHO T.E\nWHAT E[*]",
            vec![
                AccessPath::IndexCovered {
                    index_name: "surrogate".into(),
                },
                AccessPath::FullScan,
            ],
        );
        assert_eq!(
            plan.unwrap_err(),
            PlanSealError::RequiresFullScan {
                offending_clauses: vec![1]
            }
        );
    }

    #[test]
    fn seal_with_produces_a_sealed_plan_carrying_the_signature() {
        let plan = UnsealedPlan::try_new(
            "WHO T.E",
            vec![AccessPath::IndexCovered {
                index_name: "eav_exact_index".into(),
            }],
        )
        .unwrap();
        let sealed = plan.seal_with(|bytes| bytes.iter().map(|b| b.wrapping_add(1)).collect());
        assert!(!sealed.seal.is_empty());
        assert_eq!(sealed.source_query, "WHO T.E");
    }

    #[test]
    fn empty_access_paths_is_admissible_vacuously() {
        // A plan with no clauses at all has nothing to full-scan.
        let plan = UnsealedPlan::try_new("PRESENT identity", vec![]);
        assert!(plan.is_ok());
    }
}
