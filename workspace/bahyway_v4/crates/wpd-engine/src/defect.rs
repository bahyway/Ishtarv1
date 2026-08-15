// ── Defect classification ─────────────────────────────────────────

use crate::error::WpdResult;
use crate::kaki::derive_wpd_kaki;
use nusku_engine::KakiPK;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DefectClass {
    ActiveLeak,
    Corrosion,
    Crack,
    JointFailure,
    Tuberculation,
    RootIntrusion,
    Subsidence,
    Healthy,
}

impl DefectClass {
    /// Normalised severity score 0–100 for scheduling and priority.
    pub fn severity(self) -> u8 {
        match self {
            DefectClass::ActiveLeak    => 100,
            DefectClass::Crack         =>  85,
            DefectClass::JointFailure  =>  80,
            DefectClass::Subsidence    =>  75,
            DefectClass::Corrosion     =>  65,
            DefectClass::RootIntrusion =>  50,
            DefectClass::Tuberculation =>  35,
            DefectClass::Healthy       =>   0,
        }
    }

    /// Map a composite defect score [0.0, 1.0] to a defect class.
    pub fn from_score(score: f32) -> Self {
        let s = (score.clamp(0.0, 1.0) * 100.0) as u8;
        match s {
            90..=100 => DefectClass::ActiveLeak,
            75..=89  => DefectClass::Crack,
            60..=74  => DefectClass::JointFailure,
            45..=59  => DefectClass::Corrosion,
            30..=44  => DefectClass::Tuberculation,
            10..=29  => DefectClass::RootIntrusion,
            _        => DefectClass::Healthy,
        }
    }

    pub fn requires_emergency_shutdown(self) -> bool {
        matches!(self, DefectClass::ActiveLeak | DefectClass::Crack)
    }

    pub fn name(self) -> &'static str {
        match self {
            DefectClass::ActiveLeak    => "Active Leak",
            DefectClass::Corrosion     => "Corrosion",
            DefectClass::Crack         => "Crack",
            DefectClass::JointFailure  => "Joint Failure",
            DefectClass::Tuberculation => "Tuberculation",
            DefectClass::RootIntrusion => "Root Intrusion",
            DefectClass::Subsidence    => "Subsidence",
            DefectClass::Healthy       => "Healthy",
        }
    }
}

// ── Detection method ──────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectionMethod {
    SpectralScan,
    VisualInspection,
    PressureTest,
    AcousticSensor,
}

impl DetectionMethod {
    pub fn name(self) -> &'static str {
        match self {
            DetectionMethod::SpectralScan     => "Spectral Scan",
            DetectionMethod::VisualInspection => "Visual Inspection",
            DetectionMethod::PressureTest     => "Pressure Test",
            DetectionMethod::AcousticSensor   => "Acoustic Sensor",
        }
    }
}

// ── Defect event ──────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct DefectEvent {
    pub kaki:                     KakiPK,
    pub segment_id:               String,
    pub defect_class:             DefectClass,
    pub detection_method:         DetectionMethod,
    pub composite_score:          f32,
    pub confidence:               f32,
    pub timestamp_ms:             u64,
    pub description:              String,
    pub estimated_water_loss_lpd: Option<f32>,
    pub recommended_action:       String,
}

// ── Classifier ───────────────────────────────────────────────────

pub struct DefectClassifier;

impl DefectClassifier {
    pub fn classify(
        segment_id:      impl Into<String>,
        composite_score: f32,
        confidence:      f32,
        timestamp_ms:    u64,
        detection_method: DetectionMethod,
    ) -> WpdResult<DefectEvent> {
        let sid          = segment_id.into();
        let defect_class = DefectClass::from_score(composite_score);
        let kaki         = derive_wpd_kaki(&sid, composite_score, confidence, timestamp_ms);
        let estimated_water_loss_lpd = if matches!(defect_class, DefectClass::ActiveLeak) {
            // rough linear estimate: 5000 L/day at score=1.0
            Some(composite_score * 5_000.0)
        } else {
            None
        };
        let recommended_action = Self::recommended_action(defect_class).to_string();
        let description = format!(
            "{} detected via {} (score={:.2}, conf={:.2})",
            defect_class.name(), detection_method.name(), composite_score, confidence
        );
        Ok(DefectEvent {
            kaki,
            segment_id: sid,
            defect_class,
            detection_method,
            composite_score,
            confidence,
            timestamp_ms,
            description,
            estimated_water_loss_lpd,
            recommended_action,
        })
    }

    pub fn recommended_action(class: DefectClass) -> &'static str {
        match class {
            DefectClass::ActiveLeak    => "Emergency shutdown and immediate repair",
            DefectClass::Crack         => "Emergency inspection and repair within 24h",
            DefectClass::JointFailure  => "Urgent repair scheduled within 72h",
            DefectClass::Subsidence    => "Urgent soil stabilisation and pipe realignment",
            DefectClass::Corrosion     => "Planned repair within 30 days",
            DefectClass::Tuberculation => "Planned cleaning and lining within 90 days",
            DefectClass::RootIntrusion => "Routine root clearing within 6 months",
            DefectClass::Healthy       => "No action required",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_score_boundaries() {
        assert_eq!(DefectClass::from_score(0.90), DefectClass::ActiveLeak);
        assert_eq!(DefectClass::from_score(0.75), DefectClass::Crack);
        assert_eq!(DefectClass::from_score(0.60), DefectClass::JointFailure);
        assert_eq!(DefectClass::from_score(0.45), DefectClass::Corrosion);
        assert_eq!(DefectClass::from_score(0.30), DefectClass::Tuberculation);
        assert_eq!(DefectClass::from_score(0.10), DefectClass::RootIntrusion);
        assert_eq!(DefectClass::from_score(0.05), DefectClass::Healthy);
    }

    #[test]
    fn active_leak_requires_shutdown() {
        assert!(DefectClass::ActiveLeak.requires_emergency_shutdown());
        assert!(DefectClass::Crack.requires_emergency_shutdown());
        assert!(!DefectClass::Corrosion.requires_emergency_shutdown());
    }

    #[test]
    fn severity_ordering() {
        assert!(DefectClass::ActiveLeak.severity() > DefectClass::Corrosion.severity());
        assert!(DefectClass::Corrosion.severity()  > DefectClass::Healthy.severity());
    }

    #[test]
    fn classifier_produces_kaki() {
        let ev = DefectClassifier::classify(
            "BGH-SC-001", 0.80, 0.90, 1_717_000_000_000,
            DetectionMethod::SpectralScan,
        )
        .unwrap();
        assert_ne!(ev.kaki, [0u8; 16]);
        assert_eq!(ev.defect_class, DefectClass::Crack);
    }

    #[test]
    fn active_leak_estimates_water_loss() {
        let ev = DefectClassifier::classify(
            "BGH-KZ-001", 0.95, 0.85, 1_717_000_000_000,
            DetectionMethod::SpectralScan,
        )
        .unwrap();
        assert!(ev.estimated_water_loss_lpd.is_some());
        assert!(ev.estimated_water_loss_lpd.unwrap() > 0.0);
    }

    #[test]
    fn healthy_segment_no_water_loss() {
        let ev = DefectClassifier::classify(
            "BGH-GZ-001", 0.05, 0.95, 1_717_000_000_000,
            DetectionMethod::SpectralScan,
        )
        .unwrap();
        assert!(ev.estimated_water_loss_lpd.is_none());
    }
}
