//! The sealed report — the run rendered as an official document.
//! Markdown body, Ed25519 signature (AkkadianSeal) over the body bytes,
//! signature block appended after signing so verification is exact.

use crate::model::{ErrorEvent, PbStatus, Severity};
use anyhow::{Context, Result};
use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};
use std::path::{Path, PathBuf};

pub struct RunSummary<'a> {
    pub started: chrono::DateTime<chrono::Utc>,
    pub finished: chrono::DateTime<chrono::Utc>,
    pub playbooks: &'a [String],
    pub statuses: &'a [PbStatus],
    pub events: &'a [ErrorEvent],
    pub parameters: &'a [(String, String)],
    pub fixes_generated: &'a [PathBuf],
}

/// Mints (or loads) the Ed25519 key that seals every report.
///
/// Security note: this is a floor, not the destination. The ecosystem's
/// real secret custody is the Sargon Passport Manager vault
/// (`crates/kupru`'s Argon2id + Ed25519 + ChaCha20-Poly1305), not a bare
/// key file on disk — see `docs/PLAYBOOK_EXECUTION_TRIAGE.md`'s "Secrets"
/// section. Until Shakkanakku has a scriptable way to mint this key inside
/// that vault instead of next to itself, owner-only file permissions are
/// the best this function can do on its own, and this key should be
/// treated as sensitive exactly like `akkadian_seal.key` — never committed,
/// never uploaded, rotated immediately if it ever is.
fn load_or_create_key(path: &Path) -> Result<SigningKey> {
    if path.exists() {
        let bytes = std::fs::read(path).context("cannot read seal key")?;
        let arr: [u8; 32] = bytes
            .as_slice()
            .try_into()
            .map_err(|_| anyhow::anyhow!("seal key must be 32 bytes"))?;
        Ok(SigningKey::from_bytes(&arr))
    } else {
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent).context("cannot create seal key directory")?;
        }
        let key = SigningKey::generate(&mut rand_core::OsRng);
        std::fs::write(path, key.to_bytes()).context("cannot write seal key")?;
        harden_key_permissions(path)?;
        Ok(key)
    }
}

#[cfg(unix)]
fn harden_key_permissions(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(path)
        .context("cannot stat freshly-written seal key")?
        .permissions();
    perms.set_mode(0o600);
    std::fs::set_permissions(path, perms).context("cannot restrict seal key permissions")?;
    Ok(())
}

#[cfg(not(unix))]
fn harden_key_permissions(_path: &Path) -> Result<()> {
    Ok(())
}

pub fn generate(report_dir: &str, seal_key: &str, run: &RunSummary) -> Result<PathBuf> {
    std::fs::create_dir_all(report_dir).context("cannot create report dir")?;

    let ok = run.statuses.iter().filter(|s| **s == PbStatus::Ok).count();
    let warned = run.statuses.iter().filter(|s| **s == PbStatus::Warned).count();
    let failed = run.statuses.iter().filter(|s| **s == PbStatus::Failed).count();
    let skipped = run.statuses.iter().filter(|s| **s == PbStatus::Skipped).count();
    let dur = run.finished - run.started;

    let mut md = String::new();
    md.push_str("# Shakkanakku — Infrastructure Deployment Report\n\n");
    md.push_str(&format!(
        "**Run:** {} → {} UTC  \n**Duration:** {}m {}s  \n**Corpus:** {} playbooks\n\n",
        run.started.format("%Y-%m-%d %H:%M:%S"),
        run.finished.format("%H:%M:%S"),
        dur.num_minutes(),
        dur.num_seconds() % 60,
        run.playbooks.len()
    ));

    md.push_str("## Summary\n\n");
    md.push_str(&format!(
        "| Outcome | Count |\n|---|---|\n| ✓ Succeeded | {ok} |\n| ⚠ Warnings | {warned} |\n| ✖ Failed | {failed} |\n| » Skipped by Architect | {skipped} |\n\n"
    ));

    md.push_str("## Parameter manifest\n\n");
    if run.parameters.is_empty() {
        md.push_str("_none_\n\n");
    } else {
        md.push_str("| Key | Value |\n|---|---|\n");
        for (k, v) in run.parameters {
            md.push_str(&format!("| {k} | {v} |\n"));
        }
        md.push('\n');
    }

    md.push_str("## Per-playbook outcomes\n\n| # | Playbook | Outcome |\n|---|---|---|\n");
    for (i, pb) in run.playbooks.iter().enumerate() {
        let st = run.statuses.get(i).copied().unwrap_or(PbStatus::Pending);
        md.push_str(&format!("| {} | {} | {} |\n", i + 1, pb, st.icon()));
    }
    md.push('\n');

    md.push_str("## Errors, warnings & remedies\n\n");
    if run.events.is_empty() {
        md.push_str("_A clean run — no events._\n\n");
    } else {
        for ev in run.events {
            md.push_str(&format!(
                "### {} — {}\n\n- **Playbook:** {}\n- **Task:** {}\n- **Type:** {}\n- **Message:** {}\n",
                ev.severity, ev.playbook, ev.playbook, ev.task, ev.error_type, ev.message
            ));
            match (&ev.remedy_id, ev.severity) {
                (Some(id), _) => md.push_str(&format!("- **Remedy:** rule {id} matched — fix playbook generated\n\n")),
                (None, Severity::Major) => md.push_str("- **Remedy:** none in knowledge base — escalated to Architect\n\n"),
                (None, Severity::Warning) => md.push_str("- **Remedy:** proceeded by law\n\n"),
            }
        }
    }

    if !run.fixes_generated.is_empty() {
        md.push_str("## Generated fix playbooks\n\n");
        for f in run.fixes_generated {
            md.push_str(&format!("- `{}`\n", f.display()));
        }
        md.push('\n');
    }

    // ---- seal: sign the body, then append the seal block ----
    let key = load_or_create_key(Path::new(seal_key))?;
    let sig = key.sign(md.as_bytes());
    let vk: VerifyingKey = key.verifying_key();

    let sealed = format!(
        "{md}---\n\n## AkkadianSeal 𒁾\n\n\
         Signature (Ed25519) over the report body above this rule line.\n\n\
         - **Public key:** `{}`\n- **Signature:** `{}`\n\n\
         _Verify: strip this section at the `---` rule, verify body bytes against the signature._\n",
        hex::encode(vk.to_bytes()),
        hex::encode(sig.to_bytes())
    );

    let ts = run.started.format("%Y%m%d_%H%M%S");
    let path = PathBuf::from(report_dir).join(format!("shakkanakku_report_{ts}.md"));
    std::fs::write(&path, sealed).context("cannot write report")?;
    Ok(path)
}

/// Renders a sealed report's markdown as a self-contained styled HTML
/// document — a real conversion, not a `<pre>` dump, but deliberately
/// scoped to exactly the markdown subset `generate` above ever emits
/// (h1-h3, `|`-tables, `**bold**`, `` `code` ``, `- ` bullets, and
/// whole-line `_italic_`) rather than pulling in a general markdown crate.
/// General inline `_italic_` pairing is deliberately NOT implemented: this
/// report's tables are full of underscores (playbook/param names like
/// `enkidb_write`), and naively pairing every `_` would mangle them.
pub fn to_html(markdown: &str, title: &str) -> String {
    let mut body = String::new();
    let mut in_ul = false;
    let mut table_rows: Vec<Vec<String>> = Vec::new();

    fn flush_table(body: &mut String, rows: &mut Vec<Vec<String>>) {
        if rows.is_empty() {
            return;
        }
        body.push_str("<table>\n<thead><tr>");
        for c in &rows[0] {
            body.push_str(&format!("<th>{}</th>", inline_html(c)));
        }
        body.push_str("</tr></thead>\n<tbody>\n");
        for r in rows.iter().skip(1) {
            body.push_str("<tr>");
            for c in r {
                body.push_str(&format!("<td>{}</td>", inline_html(c)));
            }
            body.push_str("</tr>\n");
        }
        body.push_str("</tbody>\n</table>\n");
        rows.clear();
    }

    for line in markdown.lines() {
        let t = line.trim_end();
        if t.trim_start().starts_with('|') {
            let cells: Vec<String> = t.trim().trim_matches('|').split('|').map(|c| c.trim().to_string()).collect();
            let is_sep = cells.iter().all(|c| !c.is_empty() && c.chars().all(|ch| ch == '-'));
            if !is_sep {
                table_rows.push(cells);
            }
            continue;
        }
        flush_table(&mut body, &mut table_rows);

        let trimmed = t.trim();
        if trimmed.is_empty() {
            if in_ul {
                body.push_str("</ul>\n");
                in_ul = false;
            }
        } else if trimmed == "---" {
            body.push_str("<hr>\n");
        } else if let Some(h) = trimmed.strip_prefix("### ") {
            body.push_str(&format!("<h3>{}</h3>\n", inline_html(h)));
        } else if let Some(h) = trimmed.strip_prefix("## ") {
            body.push_str(&format!("<h2>{}</h2>\n", inline_html(h)));
        } else if let Some(h) = trimmed.strip_prefix("# ") {
            body.push_str(&format!("<h1>{}</h1>\n", inline_html(h)));
        } else if let Some(item) = trimmed.strip_prefix("- ") {
            if !in_ul {
                body.push_str("<ul>\n");
                in_ul = true;
            }
            body.push_str(&format!("<li>{}</li>\n", inline_html(item)));
        } else {
            if in_ul {
                body.push_str("</ul>\n");
                in_ul = false;
            }
            if trimmed.len() > 1 && trimmed.starts_with('_') && trimmed.ends_with('_') {
                body.push_str(&format!("<p><em>{}</em></p>\n", inline_html(&trimmed[1..trimmed.len() - 1])));
            } else {
                body.push_str(&format!("<p>{}</p>\n", inline_html(trimmed)));
            }
        }
    }
    if in_ul {
        body.push_str("</ul>\n");
    }
    flush_table(&mut body, &mut table_rows);

    format!(
        "<!doctype html>\n<html><head><meta charset=\"utf-8\">\n<title>{}</title>\n<style>{}</style>\n</head>\n<body>\n<article>\n{}</article>\n</body></html>\n",
        html_escape(title),
        REPORT_CSS,
        body
    )
}

const REPORT_CSS: &str = r#"
body { background:#0B1626; color:#F2E8D5; font-family: ui-monospace, "Cascadia Code", Menlo, Consolas, monospace; }
article { max-width: 860px; margin: 2em auto; padding: 0 1.5em; line-height: 1.5; }
h1,h2,h3 { color:#F2E8D5; }
h1 { border-bottom: 2px solid #E8A33D; padding-bottom: .3em; }
h2 { border-bottom: 1px solid #23324a; padding-bottom: .2em; margin-top: 1.6em; }
table { border-collapse: collapse; width: 100%; margin: 1em 0; }
th, td { border: 1px solid #23324a; padding: 6px 10px; text-align: left; }
th { background: #101F35; color: #E8A33D; }
code { background: #101F35; padding: 1px 5px; border-radius: 3px; }
strong { color: #E8A33D; }
hr { border: none; border-top: 1px solid #23324a; margin: 2em 0; }
@media print {
  body { background: #fff; color: #111; }
  h1,h2,h3,strong { color: #111; }
  h1 { border-color: #999; }
  h2 { border-color: #ccc; }
  table, th, td, hr { border-color: #ccc; }
  th { background: #f2f2f2; color: #111; }
  code { background: #f2f2f2; }
}
"#;

fn inline_html(s: &str) -> String {
    let escaped = html_escape(s);
    let escaped = replace_paired(&escaped, "**", "<strong>", "</strong>");
    replace_paired(&escaped, "`", "<code>", "</code>")
}

fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(c),
        }
    }
    out
}

/// Alternates `open`/`close` at each occurrence of `delim` — correct for
/// well-formed pairs (which is all `generate` ever writes for `**`/`` ` ``),
/// not a general parser.
fn replace_paired(s: &str, delim: &str, open: &str, close: &str) -> String {
    let mut out = String::new();
    let mut rest = s;
    let mut inside = false;
    while let Some(idx) = rest.find(delim) {
        out.push_str(&rest[..idx]);
        out.push_str(if inside { close } else { open });
        inside = !inside;
        rest = &rest[idx + delim.len()..];
    }
    out.push_str(rest);
    out
}

/// Verify a sealed report file (used by tests and by auditors).
pub fn verify(path: &Path) -> Result<bool> {
    let text = std::fs::read_to_string(path)?;
    let idx = text.rfind("---\n\n## AkkadianSeal").context("no seal block")?;
    let body = &text[..idx];
    let get = |label: &str| -> Option<String> {
        text[idx..]
            .lines()
            .find(|l| l.contains(label))
            .and_then(|l| l.split('`').nth(1))
            .map(|s| s.to_string())
    };
    let pk = hex::decode(get("**Public key:**").context("no pubkey")?)?;
    let sg = hex::decode(get("**Signature:**").context("no sig")?)?;
    let vk = VerifyingKey::from_bytes(pk.as_slice().try_into()?)?;
    let sig = ed25519_dalek::Signature::from_bytes(sg.as_slice().try_into()?);
    Ok(vk.verify(body.as_bytes(), &sig).is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_is_sealed_and_verifiable() {
        let dir = std::env::temp_dir().join("shk_rep_test");
        let keyp = dir.join("seal.key");
        std::fs::create_dir_all(&dir).unwrap();
        let playbooks = vec!["01_a.yml".to_string(), "02_b.yml".to_string()];
        let statuses = vec![PbStatus::Ok, PbStatus::Warned];
        let events = vec![ErrorEvent {
            pb_index: 1,
            playbook: "02_b.yml".into(),
            task: "template".into(),
            error_type: "DeprecatedParam".into(),
            message: "deprecated".into(),
            severity: Severity::Warning,
            remedy_id: None,
        }];
        let run = RunSummary {
            started: chrono::Utc::now(),
            finished: chrono::Utc::now(),
            playbooks: &playbooks,
            statuses: &statuses,
            events: &events,
            parameters: &[("env".into(), "test".into())],
            fixes_generated: &[],
        };
        let p = generate(dir.to_str().unwrap(), keyp.to_str().unwrap(), &run).unwrap();
        assert!(verify(&p).unwrap(), "seal must verify");
        // tampering must break the seal
        let mut t = std::fs::read_to_string(&p).unwrap();
        t = t.replacen("Succeeded | 1", "Succeeded | 2", 1);
        std::fs::write(&p, t).unwrap();
        assert!(!verify(&p).unwrap(), "tampered report must fail verification");
    }

    #[test]
    fn seal_key_parent_directory_is_created_if_missing() {
        // Regression: load_or_create_key used to assume its parent
        // directory already existed (unlike fix_dir/report_dir/
        // chronicle_dir, which all get create_dir_all elsewhere in this
        // crate) -- a fresh checkout with a gitignored, not-yet-created
        // `secrets/` directory would fail here on the very first run.
        let base = std::env::temp_dir().join("shk_rep_nested_dir_test");
        let _ = std::fs::remove_dir_all(&base);
        let keyp = base.join("secrets").join("shakkanakku_seal.key");
        assert!(!keyp.parent().unwrap().exists());
        let key = load_or_create_key(&keyp).unwrap();
        assert!(keyp.exists());
        // loading it back must reproduce the same key
        let reloaded = load_or_create_key(&keyp).unwrap();
        assert_eq!(key.to_bytes(), reloaded.to_bytes());
    }

    #[test]
    fn to_html_renders_structure_without_mangling_underscored_filenames() {
        let md = "# Title\n\n## Summary\n\n| Outcome | Count |\n|---|---|\n| \u{2713} Succeeded | 1 |\n\n\
                   ## Per-playbook outcomes\n\n| # | Playbook | Outcome |\n|---|---|---|\n\
                   | 1 | playbook_93_geo_engine_reconciled.yml | \u{2713} |\n\n\
                   ## Errors\n\n_A clean run — no events._\n\n\
                   - `enkidb_write_fix_r041.yml`\n";
        let html = to_html(md, "Test Report");
        assert!(html.contains("<h1>Title</h1>"));
        assert!(html.contains("<h2>Summary</h2>"));
        assert!(html.contains("<table>"));
        // The whole-line italic case must render...
        assert!(html.contains("<em>A clean run — no events.</em>"));
        // ...but underscores inside ordinary table/list content must NOT
        // be treated as italic delimiters and mangled.
        assert!(html.contains("playbook_93_geo_engine_reconciled.yml"));
        assert!(html.contains("<code>enkidb_write_fix_r041.yml</code>"));
        assert!(!html.contains("<em>93_geo_engine_reconciled.yml"));
    }

    #[cfg(unix)]
    #[test]
    fn seal_key_is_written_owner_only() {
        use std::os::unix::fs::PermissionsExt;
        let dir = std::env::temp_dir().join("shk_rep_perm_test");
        let keyp = dir.join("seal.key");
        std::fs::create_dir_all(&dir).unwrap();
        let _ = std::fs::remove_file(&keyp);
        let _ = load_or_create_key(&keyp).unwrap();
        let mode = std::fs::metadata(&keyp).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "seal key must be owner-read/write only");
    }
}
