//! Risk-ranked dispatch priority (enterprise APM's "health & criticality
//! index"). Enterprise APM ranks assets by probability x consequence
//! using static criticality tags; this derives both halves from
//! physics already computed elsewhere in this crate:
//!   - probability comes from the Gibil horizon (`horizon::Tier`) --
//!     how soon this asset is predicted to fail.
//!   - consequence comes from this asset's own N-1 contingency result
//!     (`exclusion::Verdict`) -- how many customers would be stranded
//!     if it failed right now.
//! No new physics; this module only combines two numbers this crate
//! already produces.

use crate::horizon::{Tier, MAINTENANCE_WINDOW_DAYS};

/// Higher = dispatch sooner. 0.0 for a Sound asset with no contingency
/// exposure; grows both as the predicted failure gets closer (Birqu is
/// the maximum probability weight, 1.0) and as more customers would be
/// stranded if it failed.
pub fn dispatch_priority(tier: Tier, unserved_if_lost: usize) -> f64 {
    let probability_weight = match tier {
        Tier::Birqu => 1.0,
        Tier::Horizon(days) => {
            let days = days.clamp(0.0, MAINTENANCE_WINDOW_DAYS);
            (MAINTENANCE_WINDOW_DAYS - days) / MAINTENANCE_WINDOW_DAYS
        }
        Tier::Sound => 0.0,
    };
    probability_weight * (1.0 + unserved_if_lost as f64)
}

/// Ranks a batch of (tier, unserved_if_lost) pairs, highest priority
/// first. Ties keep their original relative order (stable sort).
pub fn rank(items: &[(Tier, usize)]) -> Vec<usize> {
    let mut idx: Vec<usize> = (0..items.len()).collect();
    idx.sort_by(|&a, &b| {
        let pa = dispatch_priority(items[a].0, items[a].1);
        let pb = dispatch_priority(items[b].0, items[b].1);
        pb.partial_cmp(&pa).unwrap_or(std::cmp::Ordering::Equal)
    });
    idx
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sound_asset_with_no_exposure_has_zero_priority() {
        assert_eq!(dispatch_priority(Tier::Sound, 0), 0.0);
    }

    #[test]
    fn birqu_always_outranks_horizon_at_equal_exposure() {
        let birqu = dispatch_priority(Tier::Birqu, 5);
        let horizon = dispatch_priority(Tier::Horizon(80.0), 5);
        assert!(birqu > horizon);
    }

    #[test]
    fn closer_horizon_outranks_farther_horizon_at_equal_exposure() {
        let soon = dispatch_priority(Tier::Horizon(5.0), 3);
        let later = dispatch_priority(Tier::Horizon(85.0), 3);
        assert!(soon > later);
    }

    #[test]
    fn more_stranded_customers_outranks_fewer_at_equal_horizon() {
        let critical = dispatch_priority(Tier::Horizon(30.0), 200);
        let minor = dispatch_priority(Tier::Horizon(30.0), 1);
        assert!(critical > minor);
    }

    #[test]
    fn rank_orders_highest_priority_first() {
        let items = vec![
            (Tier::Sound, 0),           // priority 0
            (Tier::Birqu, 10),          // priority 11 -- highest
            (Tier::Horizon(45.0), 2),   // priority 1.5
        ];
        let order = rank(&items);
        assert_eq!(order, vec![1, 2, 0]);
    }
}
