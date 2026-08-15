//! Email delivery via local sendmail (sovereign, no SMTP crate, no cloud).
//! Shells out to /usr/sbin/sendmail. Requires postfix/sendmail on host.
#![forbid(unsafe_code)]
use crate::{error::KittuError, AlertEvent};
use std::io::Write;
use std::process::{Command, Stdio};

/// Build the raw RFC-822 email text for an alert. Pure function, testable
/// without spawning sendmail.
pub fn build_email_text(alert: &AlertEvent, to: &str) -> String {
    let subject = format!(
        "[BahyWay SHEDU] {} Alert - {} | tribe={}",
        alert.severity.as_str(),
        alert.rule_id,
        alert.tribe_id
    );
    let body = format!(
        "BahyWay.Ecosystem v4.0 - Kittu Engine Alert\n\
         -----------------------------------------------\n\
         Severity : {}\n\
         Rule     : {}\n\
         Tribe    : {}\n\
         Time     : {}\n\
         Message  : {}\n\
         Evidence : {}\n\
         KAKI ID  : {}\n\
         -----------------------------------------------\n\
         Nisaba detected. Kittu delivered. Sovereign.\n",
        alert.severity.as_str(),
        alert.rule_id,
        alert.tribe_id,
        alert.timestamp,
        alert.message,
        alert.evidence.join(", "),
        alert.kaki_id,
    );
    format!("To: {to}\nSubject: {subject}\nMIME-Version: 1.0\nContent-Type: text/plain; charset=utf-8\n\n{body}")
}

pub fn send_alert_email(alert: &AlertEvent, to: &str) -> Result<(), KittuError> {
    let email_text = build_email_text(alert, to);
    let mut child = Command::new("/usr/sbin/sendmail")
        .args(["-t", "-oi"])
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| KittuError::Email(format!("sendmail spawn: {e}")))?;
    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(email_text.as_bytes())
            .map_err(|e| KittuError::Email(format!("sendmail stdin: {e}")))?;
    }
    let status = child
        .wait()
        .map_err(|e| KittuError::Email(format!("sendmail wait: {e}")))?;
    if !status.success() {
        return Err(KittuError::Email(format!("sendmail exited: {status}")));
    }
    eprintln!("Kittu email: alert {} sent to {}", alert.kaki_id, to);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AlertSeverity;

    #[test]
    fn builds_expected_subject_and_body() {
        let alert = AlertEvent {
            kaki_id: "kaki-1".to_string(),
            tribe_id: "shedu_security".to_string(),
            severity: AlertSeverity::Erra,
            rule_id: "R-042".to_string(),
            message: "critical drift".to_string(),
            timestamp: "2026-07-08T00:00:00Z".to_string(),
            evidence: vec!["e1".to_string(), "e2".to_string()],
        };
        let text = build_email_text(&alert, "security@bahyway.local");
        assert!(text.contains("To: security@bahyway.local"));
        assert!(text.contains("ERRA Alert - R-042"));
        assert!(text.contains("critical drift"));
        assert!(text.contains("e1, e2"));
    }
}
