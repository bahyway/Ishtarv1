#![forbid(unsafe_code)]
//! The 8 sovereign anomaly codes (SL-ANOMALY-001) and the AnomalyEvent struct.

use enkidb_kaki::Kaki;

/// Eight anomaly codes detectable by StewardLens without writing to EnkiDB.
///
/// Each code maps to a TiamatLevel in `StewardAlert::severity`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum AnomalyCode {
    /// SA-001: Nash equilibrium breaking — anarchy_score > 0.85.
    NashBreaking = 0x01,
    /// SA-002: Pattern confidence collapse — canonical pattern drops below 0.60.
    PatternConfidenceDrop = 0x02,
    /// SA-003: Synthetic particle flood — ENKI-GENESIS output rate > 10× baseline.
    SyntheticFlood = 0x03,
    /// SA-004: TIAMAT escalation — pattern severity jumped ≥ 2 levels in 1 orbital.
    TiamatEscalation = 0x04,
    /// SA-005: AICouncil split — no 75% consensus after 3 deliberation rounds.
    AiCouncilDeadlock = 0x05,
    /// SA-006: Šàtamu override anomaly — SatamuRecord chain break or forged signature.
    SatamuChainBreak = 0x06,
    /// SA-007: KAKI collision — two particles share the same 16 bytes (impossible
    ///   in normal operation; indicates storage corruption or attack).
    KakiCollision = 0x07,
    /// SA-008: Orbital clock drift — node BASE orbital counter diverged > 5 ticks
    ///   from cluster consensus.
    OrbitalClockDrift = 0x08,
}

impl AnomalyCode {
    pub fn as_str(self) -> &'static str {
        match self {
            AnomalyCode::NashBreaking => "SA-001:NashBreaking",
            AnomalyCode::PatternConfidenceDrop => "SA-002:PatternConfidenceDrop",
            AnomalyCode::SyntheticFlood => "SA-003:SyntheticFlood",
            AnomalyCode::TiamatEscalation => "SA-004:TiamatEscalation",
            AnomalyCode::AiCouncilDeadlock => "SA-005:AiCouncilDeadlock",
            AnomalyCode::SatamuChainBreak => "SA-006:SatamuChainBreak",
            AnomalyCode::KakiCollision => "SA-007:KakiCollision",
            AnomalyCode::OrbitalClockDrift => "SA-008:OrbitalClockDrift",
        }
    }

    /// Default TiamatLevel (encoded as u8) for this anomaly code.
    /// GREEN=0, DILBAT=1, KAKKAB=2, NERGAL=3, MAROON=4.
    pub fn default_tiamat_level(self) -> u8 {
        match self {
            AnomalyCode::NashBreaking => 3,          // NERGAL
            AnomalyCode::PatternConfidenceDrop => 2, // KAKKAB
            AnomalyCode::SyntheticFlood => 2,        // KAKKAB
            AnomalyCode::TiamatEscalation => 3,      // NERGAL
            AnomalyCode::AiCouncilDeadlock => 2,     // KAKKAB
            AnomalyCode::SatamuChainBreak => 4,      // MAROON — chain break is critical
            AnomalyCode::KakiCollision => 4,         // MAROON — impossible in normal ops
            AnomalyCode::OrbitalClockDrift => 1,     // DILBAT
        }
    }
}

impl core::fmt::Display for AnomalyCode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A detected anomaly event — emitted by StewardLens, consumed by `StewardAlert`.
///
/// StewardLens is read-only: it detects and reports, never modifies.
#[derive(Clone, Debug)]
pub struct AnomalyEvent {
    /// The anomaly classification.
    pub code: AnomalyCode,
    /// The KAKI of the particle or pattern that triggered this anomaly, if known.
    pub subject_kaki: Option<Kaki>,
    /// BASE orbital counter at detection time.
    pub detected_at_orbital: u64,
    /// Human-readable detail string (ASCII, ≤ 128 bytes).
    pub detail: &'static str,
    /// Optional numeric measurement that triggered the threshold.
    pub measurement: Option<f64>,
}

impl AnomalyEvent {
    pub fn new(
        code: AnomalyCode,
        subject_kaki: Option<Kaki>,
        detected_at_orbital: u64,
        detail: &'static str,
    ) -> Self {
        Self {
            code,
            subject_kaki,
            detected_at_orbital,
            detail,
            measurement: None,
        }
    }

    pub fn with_measurement(mut self, m: f64) -> Self {
        self.measurement = Some(m);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_codes_have_str() {
        let codes = [
            AnomalyCode::NashBreaking,
            AnomalyCode::PatternConfidenceDrop,
            AnomalyCode::SyntheticFlood,
            AnomalyCode::TiamatEscalation,
            AnomalyCode::AiCouncilDeadlock,
            AnomalyCode::SatamuChainBreak,
            AnomalyCode::KakiCollision,
            AnomalyCode::OrbitalClockDrift,
        ];
        for c in &codes {
            let s = c.as_str();
            assert!(s.starts_with("SA-0"), "unexpected format: {s}");
        }
    }

    #[test]
    fn satamu_and_collision_are_maroon() {
        assert_eq!(AnomalyCode::SatamuChainBreak.default_tiamat_level(), 4);
        assert_eq!(AnomalyCode::KakiCollision.default_tiamat_level(), 4);
    }

    #[test]
    fn anomaly_event_builds() {
        let ev = AnomalyEvent::new(
            AnomalyCode::NashBreaking,
            None,
            1_000,
            "equilibrium_score > 0.85",
        )
        .with_measurement(0.92);
        assert_eq!(ev.code, AnomalyCode::NashBreaking);
        assert_eq!(ev.measurement, Some(0.92));
        assert_eq!(ev.detected_at_orbital, 1_000);
    }
}
