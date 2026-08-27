//! GL-LSN-001 §2.2, §3.1 — Uruk-only staging.
//! Candidates are written UNSEALED under uruk/. There is deliberately no
//! function in this crate that writes, moves, or copies into kish/ —
//! the Kish crossing exists only as a numbered playbook run by Bahaa (CSR-08).

use crate::graft::DialectCandidate;
use std::fs;
use std::path::{Path, PathBuf};

/// STAGE phase: write the unsealed dialect tablet under `<root>/uruk/`.
pub fn stage(root: &Path, c: &DialectCandidate) -> std::io::Result<PathBuf> {
    let uruk = root.join("uruk");
    fs::create_dir_all(&uruk)?;
    let file = uruk.join(format!("{}.dialect.akk.md", safe_name(&c.id)));
    let mut body = String::new();
    body.push_str("# UNSEALED DIALECT CANDIDATE — URUK STREAM\n");
    body.push_str("# Status: UNSEALED. Kish crossing requires Bahaa's playbook (GL-LSN-001 §3.1).\n\n");
    body.push_str(&format!("id: {}\n", c.id));
    body.push_str("ancestry (sealed only):\n");
    for a in &c.sealed_ancestry {
        body.push_str(&format!("  - {}\n", a));
    }
    body.push_str("\n## Productions (AkkadianAOL sub-dialect)\n\n```\n");
    for p in &c.productions {
        body.push_str(p);
        body.push('\n');
    }
    body.push_str("```\n\n## Ḫubullu gloss (GL-NAM-002)\n\n");
    body.push_str(&c.hubullu_gloss);
    body.push_str("\n\n## The wound this graft heals\n\n");
    for g in &c.wound_glosses {
        body.push_str(&format!("  - \"{}\"\n", g));
    }
    fs::write(&file, body)?;
    Ok(file)
}

fn safe_name(id: &str) -> String {
    id.chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' { c } else { '_' })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graft::DialectCandidate;

    fn cand() -> DialectCandidate {
        DialectCandidate {
            id: "LSN-CAND-P[a]|O[b]|T[c]".into(),
            productions: vec!["dialect ::= ORBIT b PRESENT a".into()],
            hubullu_gloss: "plain words".into(),
            sealed_ancestry: vec!["GL-ALG-002".into()],
            wound_glosses: vec!["x".into()],
        }
    }

    #[test]
    fn stages_into_uruk_and_never_touches_kish() {
        let root = std::env::temp_dir().join("gula_uruk_test");
        let _ = std::fs::remove_dir_all(&root);
        let path = stage(&root, &cand()).unwrap();
        assert!(path.starts_with(root.join("uruk")));
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("UNSEALED"));
        assert!(
            !root.join("kish").exists(),
            "the agent must possess no write path to kish/ (§2.2)"
        );
    }
}
