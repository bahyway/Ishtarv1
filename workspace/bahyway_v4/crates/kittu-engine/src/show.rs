//! ShoWEngine dashboard delivery — writes alert JSON to drop directory.
//! DubSar Theater reads from this directory on its dashboard panel.
#![forbid(unsafe_code)]
use crate::{error::KittuError, AlertEvent};
use std::fs;
use std::path::PathBuf;

pub fn write_show_alert(alert: &AlertEvent, show_dir: &str) -> Result<(), KittuError> {
    let short_id: String = alert.kaki_id.chars().take(8).collect();
    let filename = format!("alert_{}_{}.json", alert.severity.as_str(), short_id);
    let path = PathBuf::from(show_dir).join(&filename);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| KittuError::Io(e.to_string()))?;
    }
    let json =
        serde_json::to_string_pretty(alert).map_err(|e| KittuError::Serialise(e.to_string()))?;
    fs::write(&path, json).map_err(|e| KittuError::Io(e.to_string()))?;
    eprintln!("Kittu ShoWEngine: alert written -> {}", path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AlertSeverity;

    fn sample_alert() -> AlertEvent {
        AlertEvent {
            kaki_id: "0123456789abcdef".to_string(),
            tribe_id: "shedu_security".to_string(),
            severity: AlertSeverity::Critical,
            rule_id: "R-001".to_string(),
            message: "test".to_string(),
            timestamp: "2026-07-08T00:00:00Z".to_string(),
            evidence: vec!["ev-1".to_string()],
        }
    }

    #[test]
    fn writes_json_file_with_expected_name() {
        let tmp = std::env::temp_dir().join(format!("kittu_show_test_{}", std::process::id()));
        write_show_alert(&sample_alert(), tmp.to_str().unwrap()).unwrap();
        let expected = tmp.join("alert_CRITICAL_01234567.json");
        assert!(expected.exists());
        let contents = std::fs::read_to_string(&expected).unwrap();
        assert!(contents.contains("shedu_security"));
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
