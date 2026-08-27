use std::time::Instant;

/// Maximum total pipeline duration — sovereign 3-second guarantee
pub const PIPELINE_BUDGET_MS: u64 = 2800;

/// Stage timing record for PMPVD audit log.
#[derive(Debug, Clone)]
pub struct PipelineTiming {
    pub stage_name: String,
    pub started_ms: u64,
    pub elapsed_ms: u64,
    pub budget_ms: u64,
    pub within_budget: bool,
}

/// Full pipeline execution context — tracks per-stage timings.
#[derive(Debug)]
pub struct PipelineContext {
    pub frame_id: u64,
    pub iraq_protocol: bool,
    pub face_enabled: bool,
    pub timings: Vec<PipelineTiming>,
    t_start: Instant,
}

impl PipelineContext {
    pub fn new(frame_id: u64, iraq_protocol: bool, face_enabled: bool) -> Self {
        Self {
            frame_id,
            iraq_protocol,
            face_enabled,
            timings: Vec::new(),
            t_start: Instant::now(),
        }
    }

    pub fn record(&mut self, stage: &str, budget_ms: u64) {
        let elapsed_ms = self.t_start.elapsed().as_millis() as u64;
        self.timings.push(PipelineTiming {
            stage_name: stage.to_string(),
            started_ms: elapsed_ms,
            elapsed_ms,
            budget_ms,
            within_budget: elapsed_ms <= budget_ms,
        });
    }

    pub fn elapsed_ms(&self) -> u64 {
        self.t_start.elapsed().as_millis() as u64
    }

    pub fn within_budget(&self) -> bool {
        self.elapsed_ms() < PIPELINE_BUDGET_MS
    }

    pub fn timing_report(&self) -> String {
        self.timings
            .iter()
            .map(|t| {
                format!(
                    "  [{:4}ms] {:30} budget={}ms {}",
                    t.elapsed_ms,
                    t.stage_name,
                    t.budget_ms,
                    if t.within_budget {
                        "✓"
                    } else {
                        "⚠ EXCEEDED"
                    }
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Stage timing budgets (cumulative ms from pipeline start).
pub struct PipelineBudgets;
impl PipelineBudgets {
    pub const IR_CAPTURE: u64 = 33;
    pub const PARTICLE_BUILD: u64 = 55;
    pub const TRS_COMPUTE: u64 = 65;
    pub const AZUGA_INFER: u64 = 120;
    pub const FUZZY_INFER: u64 = 155;
    pub const FACE_DETECT: u64 = 340;
    pub const SCORE_COMPUTE: u64 = 365;
    pub const ALERT_GEN: u64 = 380;
    pub const TOTAL: u64 = 2800;
}

/// Alert message emitted at the end of each pipeline pass.
#[derive(Debug, Clone)]
pub struct NuskuAlert {
    pub frame_id: u64,
    /// 0=clear, 1=medical, 2=warning, 3=high, 4=threat
    pub alert_level: u8,
    pub score: f32,
    pub message_en: String,
    pub message_ar: String,
    pub kaki_hex: String,
    pub elapsed_ms: u64,
    pub sound_alarm: bool,
    pub vibrate: bool, // mobile haptic
}

impl NuskuAlert {
    pub fn color_hex(&self) -> &'static str {
        match self.alert_level {
            0 => "#3a9050",     // green  — clear
            1 | 2 => "#4a80c8", // blue   — medical
            3 => "#c88020",     // amber  — warning
            4 => "#d84020",     // orange — high alert
            _ => "#c82020",     // red    — threat
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pipeline_context_starts_within_budget() {
        let ctx = PipelineContext::new(1, true, false);
        assert!(ctx.within_budget());
    }

    #[test]
    fn record_creates_timing_entry() {
        let mut ctx = PipelineContext::new(1, false, false);
        ctx.record("IR_CAPTURE", PipelineBudgets::IR_CAPTURE);
        assert_eq!(ctx.timings.len(), 1);
        assert_eq!(ctx.timings[0].stage_name, "IR_CAPTURE");
    }

    #[test]
    fn timing_report_non_empty_after_stages() {
        let mut ctx = PipelineContext::new(0, false, false);
        ctx.record("stage_a", 100);
        ctx.record("stage_b", 200);
        let report = ctx.timing_report();
        assert!(report.contains("stage_a"));
        assert!(report.contains("stage_b"));
    }

    #[test]
    fn nusku_alert_color_hex_for_all_levels() {
        for level in 0u8..=5 {
            let alert = NuskuAlert {
                frame_id: 0,
                alert_level: level,
                score: 0.0,
                message_en: String::new(),
                message_ar: String::new(),
                kaki_hex: String::new(),
                elapsed_ms: 0,
                sound_alarm: false,
                vibrate: false,
            };
            assert!(alert.color_hex().starts_with('#'), "level={level}");
        }
    }
}
