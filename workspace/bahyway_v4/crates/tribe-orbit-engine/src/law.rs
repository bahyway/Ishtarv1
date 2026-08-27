//! The 7 Tribal Laws — governing particle state transitions.
//!
//! Priority order (ascending — L3 has lowest priority, L6 highest):
//!   P(L3) < P(L7) < P(L1) < P(L2) < P(L5) < P(L4) < P(L6)
//!
//! When multiple laws fire simultaneously, the highest-priority law wins.
//! Law 7 has the second-lowest priority — it only fires when density saturates
//! because all higher-priority laws have already had a chance to act.
//!
//! DUB.SAR 𒁾 — BahyWay.Ecosystem v4.0 | Pure Rust

use crate::constants::LAW_PRIORITY_ORDER;

/// One of the 7 Tribal Laws (1-based index).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TribeLaw {
    L1,
    L2,
    L3,
    L4,
    L5,
    L6,
    L7,
}

impl TribeLaw {
    /// All laws in declaration order (L1..L7).
    pub const ALL: [Self; 7] = [
        Self::L1,
        Self::L2,
        Self::L3,
        Self::L4,
        Self::L5,
        Self::L6,
        Self::L7,
    ];

    /// 0-based priority index — lower = fires last (higher priority overrides lower).
    ///
    /// Sovereign order: P(L3)=0 < P(L7)=1 < P(L1)=2 < P(L2)=3 < P(L5)=4 < P(L4)=5 < P(L6)=6
    pub fn priority(self) -> usize {
        // LAW_PRIORITY_ORDER[priority_rank] = law_index (0-based)
        // Invert: find position of this law's 0-based index in priority order
        let law_idx = self as usize; // L1=0, L2=1, ... L7=6
        LAW_PRIORITY_ORDER
            .iter()
            .position(|&idx| idx == law_idx)
            .unwrap_or(law_idx)
    }

    /// Returns `true` if `self` has higher priority than `other`.
    pub fn overrides(self, other: TribeLaw) -> bool {
        self.priority() > other.priority()
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::L1 => "L1-Identity",
            Self::L2 => "L2-Quality",
            Self::L3 => "L3-Scope",
            Self::L4 => "L4-Orbit",
            Self::L5 => "L5-Transition",
            Self::L6 => "L6-Sovereignty",
            Self::L7 => "L7-Overflow",
        }
    }
}

/// Resolve which law wins when a set of laws fire simultaneously.
///
/// Returns the highest-priority law from the set.
/// Returns `None` if the set is empty.
pub fn resolve_conflict(laws: &[TribeLaw]) -> Option<TribeLaw> {
    laws.iter().copied().max_by_key(|l| l.priority())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn l6_has_highest_priority() {
        for &law in &TribeLaw::ALL {
            if law == TribeLaw::L6 {
                continue;
            }
            assert!(
                TribeLaw::L6.overrides(law),
                "L6 must override {}",
                law.label()
            );
        }
    }

    #[test]
    fn l3_has_lowest_priority() {
        for &law in &TribeLaw::ALL {
            if law == TribeLaw::L3 {
                continue;
            }
            assert!(
                law.overrides(TribeLaw::L3),
                "{} must override L3",
                law.label()
            );
        }
    }

    #[test]
    fn l7_has_second_lowest_priority() {
        // L7 priority < L1, L2, L4, L5, L6; only > L3
        assert!(TribeLaw::L7.overrides(TribeLaw::L3));
        assert!(TribeLaw::L1.overrides(TribeLaw::L7));
        assert!(TribeLaw::L6.overrides(TribeLaw::L7));
    }

    #[test]
    fn priority_order_sovereign() {
        // Verify the complete sovereign ordering
        let ordered = [
            TribeLaw::L3,
            TribeLaw::L7,
            TribeLaw::L1,
            TribeLaw::L2,
            TribeLaw::L5,
            TribeLaw::L4,
            TribeLaw::L6,
        ];
        for i in 0..ordered.len() - 1 {
            assert!(
                ordered[i + 1].overrides(ordered[i]),
                "{} must override {}",
                ordered[i + 1].label(),
                ordered[i].label()
            );
        }
    }

    #[test]
    fn resolve_conflict_returns_highest() {
        let laws = [TribeLaw::L3, TribeLaw::L1, TribeLaw::L7];
        assert_eq!(resolve_conflict(&laws), Some(TribeLaw::L1));
    }

    #[test]
    fn resolve_conflict_empty_returns_none() {
        assert_eq!(resolve_conflict(&[]), None);
    }

    #[test]
    fn resolve_conflict_single_law_returns_it() {
        assert_eq!(resolve_conflict(&[TribeLaw::L5]), Some(TribeLaw::L5));
    }

    #[test]
    fn all_laws_have_distinct_priorities() {
        let priorities: Vec<usize> = TribeLaw::ALL.iter().map(|l| l.priority()).collect();
        let mut deduped = priorities.clone();
        deduped.sort_unstable();
        deduped.dedup();
        assert_eq!(deduped.len(), 7, "all 7 laws must have distinct priorities");
    }
}
