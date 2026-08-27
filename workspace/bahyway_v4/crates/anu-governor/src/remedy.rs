//! Remedy engine — turns a matched knowledge-base rule into a numbered
//! fix playbook on disk. The Architect runs it; running it is CSR-08.

use crate::config::RemedyRule;
use crate::model::ErrorEvent;
use anyhow::{Context, Result};
use std::path::PathBuf;

pub fn write_fix_playbook(fix_dir: &str, rule: &RemedyRule, ev: &ErrorEvent) -> Result<PathBuf> {
    std::fs::create_dir_all(fix_dir).context("cannot create fix dir")?;
    let stem = ev
        .playbook
        .split(['/', '\\'])
        .next_back()
        .unwrap_or(&ev.playbook)
        .trim_end_matches(".yml")
        .trim_end_matches(".yaml");
    let name = format!(
        "{}_fix_{}.yml",
        stem,
        rule.id.to_lowercase().replace('-', "")
    );
    let path = PathBuf::from(fix_dir).join(name);

    let body = rule
        .fix_template
        .replace("{playbook}", &ev.playbook)
        .replace("{task}", &ev.task)
        .replace("{message}", &ev.message);

    let header = format!(
        "# ── AnuGovernor generated fix ─────────────────────────────\n\
         # rule      : {}\n\
         # playbook  : {}\n\
         # task      : {}\n\
         # diagnosis : {}\n\
         # solution  : {}\n\
         # law       : running this playbook is the Architect's CSR-08 act\n\
         # ──────────────────────────────────────────────────────────\n",
        rule.id, ev.playbook, ev.task, rule.diagnosis, rule.solution
    );

    std::fs::write(&path, format!("{header}{body}")).context("cannot write fix playbook")?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Severity;

    #[test]
    fn writes_numbered_fix_with_correctly_glued_filename() {
        let rule = RemedyRule {
            id: "R-041".into(),
            pattern: "already bound".into(),
            diagnosis: "orphan holds port".into(),
            solution: "stop orphan; guard pre-start".into(),
            fix_template: "- hosts: enkidb_read\n  tasks:\n    - name: fix for {task}\n".into(),
        };
        let ev = ErrorEvent {
            pb_index: 126,
            playbook: "127_enlil_hotindex_deploy.yml".into(),
            task: "start enlil-hotindex.service".into(),
            error_type: "ServiceStartFailure".into(),
            message: "port 7101 already bound".into(),
            severity: Severity::Major,
            remedy_id: Some("R-041".into()),
        };
        let dir = std::env::temp_dir().join("shk_fix_test");
        let p = write_fix_playbook(dir.to_str().unwrap(), &rule, &ev).unwrap();
        let text = std::fs::read_to_string(&p).unwrap();
        assert_eq!(
            p.file_name().unwrap().to_str().unwrap(),
            "127_enlil_hotindex_deploy_fix_r041.yml"
        );
        assert!(text.contains("fix for start enlil-hotindex.service"));
        assert!(text.contains("R-041"));
    }
}
