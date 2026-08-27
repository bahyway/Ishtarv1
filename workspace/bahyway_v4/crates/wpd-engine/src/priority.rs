// ── Repair priority and scheduling ────────────────────────────────

use crate::defect::DefectEvent;
use crate::sector::BaghdadSector;
use crate::segment::PipelineSegment;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum RepairPriority {
    Routine = 1,
    Planned = 2,
    Urgent = 3,
    Emergency = 4,
}

impl RepairPriority {
    pub fn from_severity(severity: u8) -> Self {
        match severity {
            90..=100 => RepairPriority::Emergency,
            70..=89 => RepairPriority::Urgent,
            40..=69 => RepairPriority::Planned,
            _ => RepairPriority::Routine,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            RepairPriority::Routine => "Routine",
            RepairPriority::Planned => "Planned",
            RepairPriority::Urgent => "Urgent",
            RepairPriority::Emergency => "Emergency",
        }
    }
}

// ── Repair job ────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct RepairJob {
    pub segment_id: String,
    pub sector: BaghdadSector,
    pub priority: RepairPriority,
    pub defect_severity: u8,
    pub risk_score: f32,
    pub population_weight: f32,
    pub composite_score: f32,
    pub description: String,
}

// ── Scheduler ─────────────────────────────────────────────────────

pub struct RepairScheduler;

impl RepairScheduler {
    /// Build a repair job from a segment and its defect event.
    /// composite = severity/100×0.50 + risk_score×0.30 + population_weight×0.20
    pub fn schedule(segment: &PipelineSegment, event: &DefectEvent) -> RepairJob {
        let severity = event.defect_class.severity();
        let priority = RepairPriority::from_severity(severity);
        let risk_score = segment.risk_score();
        let population_weight = segment.sector.density_index();
        let composite_score =
            (severity as f32 / 100.0 * 0.50 + risk_score * 0.30 + population_weight * 0.20)
                .clamp(0.0, 1.0);
        RepairJob {
            segment_id: segment.id.clone(),
            sector: segment.sector,
            priority,
            defect_severity: severity,
            risk_score,
            population_weight,
            composite_score,
            description: format!(
                "{} — {} ({}) composite={:.3}",
                segment.id,
                event.defect_class.name(),
                priority.name(),
                composite_score
            ),
        }
    }

    /// Sort jobs by composite_score descending (highest urgency first).
    pub fn sort(mut jobs: Vec<RepairJob>) -> Vec<RepairJob> {
        jobs.sort_by(|a, b| {
            b.composite_score
                .partial_cmp(&a.composite_score)
                .unwrap_or(core::cmp::Ordering::Equal)
        });
        jobs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn priority_from_severity_boundaries() {
        assert_eq!(
            RepairPriority::from_severity(100),
            RepairPriority::Emergency
        );
        assert_eq!(RepairPriority::from_severity(90), RepairPriority::Emergency);
        assert_eq!(RepairPriority::from_severity(89), RepairPriority::Urgent);
        assert_eq!(RepairPriority::from_severity(70), RepairPriority::Urgent);
        assert_eq!(RepairPriority::from_severity(69), RepairPriority::Planned);
        assert_eq!(RepairPriority::from_severity(40), RepairPriority::Planned);
        assert_eq!(RepairPriority::from_severity(39), RepairPriority::Routine);
        assert_eq!(RepairPriority::from_severity(0), RepairPriority::Routine);
    }

    #[test]
    fn priority_ordering() {
        assert!(RepairPriority::Emergency > RepairPriority::Urgent);
        assert!(RepairPriority::Urgent > RepairPriority::Planned);
        assert!(RepairPriority::Planned > RepairPriority::Routine);
    }

    #[test]
    fn sort_descending() {
        let make = |score: f32| RepairJob {
            segment_id: "X".into(),
            sector: BaghdadSector::SadrCity,
            priority: RepairPriority::Routine,
            defect_severity: 0,
            risk_score: 0.0,
            population_weight: 0.0,
            composite_score: score,
            description: String::new(),
        };
        let jobs = vec![make(0.30), make(0.90), make(0.60)];
        let sorted = RepairScheduler::sort(jobs);
        assert!(sorted[0].composite_score >= sorted[1].composite_score);
        assert!(sorted[1].composite_score >= sorted[2].composite_score);
    }
}
