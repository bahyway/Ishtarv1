// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  shulman-engine · src/lib.rs
//  𒀭𒂞𒈪 Shulmanu — sovereign alert and report generator
//  Produces: alert struct, Arabic/English summary, KAKI audit trail
//  Pure Rust — no serde, no chrono
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

use nusku_score::{FinalVerdict, VerdictClass};

// ── SovereignAlert ────────────────────────────────────────────────

/// Complete alert record emitted at the end of each pipeline pass.
///
/// Feeds the Alert Log panel in DubSar visualizer and the PMPVD audit trail.
#[derive(Debug, Clone)]
pub struct SovereignAlert {
    /// Unique alert ID — "NW-{epoch_sec}-{kaki_prefix_8}"
    pub alert_id: String,
    /// Unix epoch milliseconds (from FinalVerdict — no SystemTime call here)
    pub timestamp_ms: u64,
    /// HH:MM:SS UTC for Alert Log display
    pub time_display: String,
    pub verdict: String, // "Clear" | "MedicalAlert" | "Warning" | "HighAlert" | "ThreatConfirmed"
    pub score: f32,
    pub action_en: &'static str,
    pub action_ar: &'static str,
    pub kaki_hex: String,
    pub trs: f32,
    pub med_score: f32,
    pub processing_ms: u32,
    /// Whether the alarm should sound — cached from should_sound_alarm()
    pub sound_alarm: bool,
}

// ── Alert generation ──────────────────────────────────────────────

/// Build a `SovereignAlert` from a completed `FinalVerdict`.
pub fn generate_alert(verdict: &FinalVerdict) -> SovereignAlert {
    let alarm = should_sound_alarm(verdict);
    let epoch_sec = verdict.timestamp_ms / 1000;
    let alert_id = format!("NW-{}-{}", epoch_sec, &verdict.audit_kaki_hex[..8]);
    let time_display = format_utc_hms(verdict.timestamp_ms);

    SovereignAlert {
        alert_id,
        timestamp_ms: verdict.timestamp_ms,
        time_display,
        verdict: verdict_name(&verdict.verdict).to_string(),
        score: verdict.final_score,
        action_en: verdict.alert_english,
        action_ar: verdict.alert_arabic,
        kaki_hex: verdict.audit_kaki_hex.clone(),
        trs: verdict.trs,
        med_score: verdict.med_score,
        processing_ms: verdict.processing_ms,
        sound_alarm: alarm,
    }
}

/// True when the verdict requires an audible/haptic alarm at the checkpoint.
pub fn should_sound_alarm(verdict: &FinalVerdict) -> bool {
    matches!(
        verdict.verdict,
        VerdictClass::HighAlert | VerdictClass::ThreatConfirmed
    )
}

// ── Helpers ───────────────────────────────────────────────────────

/// Format a Unix epoch millisecond timestamp as HH:MM:SS UTC (no chrono dep).
pub fn format_utc_hms(timestamp_ms: u64) -> String {
    let secs = (timestamp_ms / 1000) % 86_400;
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    format!("{h:02}:{m:02}:{s:02}")
}

fn verdict_name(v: &VerdictClass) -> &'static str {
    match v {
        VerdictClass::Clear => "Clear",
        VerdictClass::MedicalAlert => "MedicalAlert",
        VerdictClass::Warning => "Warning",
        VerdictClass::HighAlert => "HighAlert",
        VerdictClass::ThreatConfirmed => "ThreatConfirmed",
    }
}

// ── Tests ─────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;
    use nusku_score::{ScoreEngine, ScoreInputs};

    fn make_verdict(trs: f32, med: f32, fuzz: f32) -> FinalVerdict {
        let inputs = ScoreInputs {
            trs,
            med_score: med,
            fuzzy_score: fuzz,
            face_score: 0.0,
            face_enabled: false,
        };
        ScoreEngine::compute(&inputs, [0u8; 16], 42, 1_624_842_031_000)
    }

    #[test]
    fn generate_alert_clear_verdict() {
        let verdict = make_verdict(0.0, 0.0, 0.0);
        let alert = generate_alert(&verdict);
        assert_eq!(alert.verdict, "Clear");
        assert!(!alert.sound_alarm);
        assert!(alert.alert_id.starts_with("NW-"));
        assert_eq!(alert.processing_ms, 42);
    }

    #[test]
    fn generate_alert_threat_confirmed() {
        let verdict = make_verdict(1.0, 1.0, 1.0);
        let alert = generate_alert(&verdict);
        assert_eq!(alert.verdict, "ThreatConfirmed");
        assert!(alert.sound_alarm);
        assert!(!alert.action_en.is_empty());
        assert!(!alert.action_ar.is_empty());
    }

    #[test]
    fn should_sound_alarm_only_for_high_and_threat() {
        assert!(!should_sound_alarm(&make_verdict(0.0, 0.0, 0.0))); // Clear
        assert!(!should_sound_alarm(&make_verdict(0.3, 0.0, 0.0))); // MedicalAlert / Warning
                                                                    // TRS=0.85 with face_disabled → trs_w=0.60 → score=0.51 → Warning (no alarm)
        assert!(!should_sound_alarm(&make_verdict(0.85, 0.0, 0.0)));
        // All max → ThreatConfirmed
        assert!(should_sound_alarm(&make_verdict(1.0, 1.0, 1.0)));
    }

    #[test]
    fn format_utc_hms_midnight() {
        assert_eq!(format_utc_hms(0), "00:00:00");
    }

    #[test]
    fn format_utc_hms_known_time() {
        // 1_624_842_031 s % 86400 = 3631 s → 01:00:31 UTC (2021-06-28T01:00:31Z)
        let ms = 1_624_842_031_000u64;
        assert_eq!(format_utc_hms(ms), "01:00:31");
    }

    #[test]
    fn alert_id_format_contains_kaki_prefix() {
        let verdict = make_verdict(0.0, 0.0, 0.0);
        let alert = generate_alert(&verdict);
        // alert_id = "NW-{epoch_sec}-{kaki_prefix_8}"
        let parts: Vec<&str> = alert.alert_id.splitn(3, '-').collect();
        assert_eq!(parts[0], "NW");
        assert_eq!(parts[2].len(), 8, "kaki prefix should be 8 hex chars");
    }

    #[test]
    fn kaki_hex_length_is_32() {
        let verdict = make_verdict(0.5, 0.3, 0.2);
        let alert = generate_alert(&verdict);
        assert_eq!(alert.kaki_hex.len(), 32);
    }

    #[test]
    fn score_matches_verdict_score() {
        let verdict = make_verdict(0.5, 0.3, 0.2);
        let alert = generate_alert(&verdict);
        assert!((alert.score - verdict.final_score).abs() < 1e-5);
    }
}
