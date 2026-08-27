//! GL-LSN-001 §7 — The NĀRU journal. Append-only JSON Lines.
//! No silent close: every wound receives a verdict line naming its
//! witnesses, its signature, and its cause.

use crate::Verdict;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct JournalEntry {
    pub verdict: Verdict,
    pub signature_key: String,
    pub witnesses: Vec<String>,
    pub cause: String,
}

impl JournalEntry {
    pub fn to_json_line(&self) -> String {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let witnesses = self
            .witnesses
            .iter()
            .map(|w| format!("\"{}\"", esc(w)))
            .collect::<Vec<_>>()
            .join(",");
        let detail = match &self.verdict {
            Verdict::RejectedForeign(c)
            | Verdict::RejectedTumor(c)
            | Verdict::StagedUruk(c)
            | Verdict::PhantomPredicted(c)
            | Verdict::PhantomRecovered(c) => c.clone(),
            _ => String::new(),
        };
        format!(
            "{{\"ts\":{},\"verdict\":\"{}\",\"signature\":\"{}\",\"witnesses\":[{}],\"cause\":\"{}\",\"detail\":\"{}\"}}",
            ts,
            self.verdict.as_str(),
            esc(&self.signature_key),
            witnesses,
            esc(&self.cause),
            esc(&detail)
        )
    }
}

/// Minimal JSON string escaping (quotes, backslash, control chars).
fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

pub fn append(journal_path: &Path, entry: &JournalEntry) -> std::io::Result<()> {
    if let Some(parent) = journal_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut f = OpenOptions::new().create(true).append(true).open(journal_path)?;
    writeln!(f, "{}", entry.to_json_line())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_line_escapes_and_is_wellformed() {
        let e = JournalEntry {
            verdict: Verdict::RejectedForeign("tissue \"SELECT\"\nline".into()),
            signature_key: "P[a]|O[b]|T[c]".into(),
            witnesses: vec!["nanshe".into(), "karanu".into()],
            cause: "immune clause".into(),
        };
        let line = e.to_json_line();
        assert!(line.starts_with('{') && line.ends_with('}'));
        assert!(line.contains("\\\"SELECT\\\""));
        assert!(line.contains("\\n"));
        assert!(!line.contains('\n'));
    }

    #[test]
    fn append_is_append_only() {
        let dir = std::env::temp_dir().join("gula_naru_test");
        let path = dir.join("naru.jsonl");
        let _ = std::fs::remove_file(&path);
        let e = JournalEntry {
            verdict: Verdict::Observed,
            signature_key: "k".into(),
            witnesses: vec![],
            cause: "witnessed".into(),
        };
        append(&path, &e).unwrap();
        append(&path, &e).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert_eq!(text.lines().count(), 2);
    }
}
