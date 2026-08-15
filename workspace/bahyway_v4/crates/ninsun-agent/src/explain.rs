//! DeepSeek-Coder-6.7B offline inference stub for anomaly explanation.
//! Model location: ~/models/ on eriduous-vdi (downloaded once via wget).
//! Full llama.cpp / candle integration deferred — returns a template for now.
#![forbid(unsafe_code)]
use crate::drift::DriftEvent;

pub fn offline_explain(drift: &DriftEvent) -> String {
    // TODO: call DeepSeek-Coder-6.7B via candle or llama.cpp binding
    // Model: ~/models/deepseek-coder-6.7b-instruct.Q4_K_M.gguf
    format!(
        "NINSUN (offline): particle {} in tribe {} shows pattern {} \
         for {} day(s). Suggest reviewing sensor calibration or data \
         pipeline for sustained state without resolution.",
        drift.kaki_id, drift.tribe_id, drift.pattern_code, drift.days_in_state
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explanation_includes_kaki_id_and_pattern() {
        let drift = DriftEvent {
            kaki_id: "kaki-42".into(),
            tribe_id: "tribe-1".into(),
            pattern_code: "FUZZY_SUSTAINED_7DAY".into(),
            days_in_state: 9,
        };
        let text = offline_explain(&drift);
        assert!(text.contains("kaki-42"));
        assert!(text.contains("FUZZY_SUSTAINED_7DAY"));
        assert!(text.contains('9'));
    }
}
