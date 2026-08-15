//! FUZZY/DEAD orbital drift pattern detection.
#![forbid(unsafe_code)]
use crate::KakiSummary;

#[derive(Debug, Clone)]
pub struct DriftEvent {
    pub kaki_id: String,
    pub tribe_id: String,
    pub pattern_code: String,
    pub days_in_state: u32,
}

/// Detect sustained FUZZY (>=7 days) or DEAD without DEAD_CRITICAL review.
pub fn detect_drift(batch: &[KakiSummary]) -> Vec<DriftEvent> {
    batch
        .iter()
        .filter_map(|k| {
            let pattern = match k.eav_state.as_str() {
                "FUZZY" if k.days_in_state >= 7 => Some("FUZZY_SUSTAINED_7DAY"),
                "DEAD" if k.days_in_state >= 1 => Some("DEAD_UNREVIEWED"),
                _ => None,
            };
            pattern.map(|p| DriftEvent {
                kaki_id: k.kaki_id.clone(),
                tribe_id: k.tribe_id.clone(),
                pattern_code: p.to_string(),
                days_in_state: k.days_in_state,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn summary(state: &str, days: u32) -> KakiSummary {
        KakiSummary {
            kaki_id: "k".into(),
            eav_state: state.into(),
            orbital_state: "N/A".into(),
            tribe_id: "t".into(),
            days_in_state: days,
        }
    }

    #[test]
    fn fuzzy_under_7_days_is_not_drift() {
        assert!(detect_drift(&[summary("FUZZY", 6)]).is_empty());
    }

    #[test]
    fn fuzzy_7_days_is_drift() {
        let events = detect_drift(&[summary("FUZZY", 7)]);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].pattern_code, "FUZZY_SUSTAINED_7DAY");
    }
}
