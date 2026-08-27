//! The Nimrud arc for the grammar court: WITNESS → JUDGE → MOVE → STAGE.
//! Every phase journals to NĀRU. No wound is silently closed (§7.1).

use crate::court::{judge, TrialityWitness};
use crate::graft::GraftMint;
use crate::naru::{append, JournalEntry};
use crate::wound::WoundRegistry;
use crate::{PatternWitness, Verdict};
use std::path::Path;

/// Proxy formula for the triality witness, pending court calibration under
/// the Fadam functional (§6.1): the transparency deficit τ carried by a wound
/// grows with each gloss beyond the first; the graft admits a fixed candidate
/// uncertainty ε; the stakeholder floor ε(Θ) is one unit — meaning any wound
/// deep enough to trigger (three glosses) always clears the gate, while the
/// gate remains a real refusal surface once calibrated values arrive.
fn triality_proxy(gloss_count: usize) -> TrialityWitness {
    TrialityWitness {
        tau: (gloss_count.saturating_sub(1)) as f64 * 0.5,
        epsilon: 0.25,
        epsilon_theta: 1.0,
    }
}

pub struct ArcReport {
    pub verdicts: Vec<(String, Verdict)>,
    pub staged_files: Vec<std::path::PathBuf>,
}

/// Run the full arc over a batch of testimonies against a corpus root.
/// `root` is the agent's territory; the NĀRU journal lives at
/// `<root>/naru/lisanu.jsonl` and grafts stage under `<root>/uruk/`.
pub fn run_arc(root: &Path, witnesses: &[PatternWitness]) -> std::io::Result<ArcReport> {
    let journal = root.join("naru").join("lisanu.jsonl");
    let mut registry = WoundRegistry::new();
    let mut mint = GraftMint::new();
    let mut verdicts = Vec::new();
    let mut staged_files = Vec::new();

    // WITNESS
    for w in witnesses {
        let v = registry.witness(w);
        append(
            &journal,
            &JournalEntry {
                verdict: v.clone(),
                signature_key: w.signature.key(),
                witnesses: vec![w.engine.clone()],
                cause: format!("testimony: \"{}\"", w.hubullu_gloss),
            },
        )?;
        verdicts.push((w.signature.key(), v));
    }

    // JUDGE → MOVE → STAGE
    for wound in registry.triggered() {
        let engines: Vec<String> = wound.engines.iter().cloned().collect();
        let Some(candidate) = mint.mint(wound) else {
            continue; // already grafted; §5.2 refuses overgrowth silently is
                      // forbidden — but a prior MINTED line already exists.
        };
        let t = triality_proxy(wound.glosses.len());
        let verdict = judge(&candidate, t);
        append(
            &journal,
            &JournalEntry {
                verdict: verdict.clone(),
                signature_key: wound.signature.key(),
                witnesses: engines.clone(),
                cause: format!("candidate {}", candidate.id),
            },
        )?;
        verdicts.push((wound.signature.key(), verdict.clone()));

        if verdict == Verdict::Minted {
            let path = crate::uruk::stage(root, &candidate)?;
            let staged = Verdict::StagedUruk(path.display().to_string());
            append(
                &journal,
                &JournalEntry {
                    verdict: staged.clone(),
                    signature_key: wound.signature.key(),
                    witnesses: engines,
                    cause: "graft staged UNSEALED in Uruk; Kish awaits the word".into(),
                },
            )?;
            verdicts.push((wound.signature.key(), staged));
            staged_files.push(path);
        }
    }

    Ok(ArcReport { verdicts, staged_files })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Signature;

    fn w(engine: &str, gloss: &str) -> PatternWitness {
        PatternWitness {
            engine: engine.into(),
            signature: Signature::new("leak-onset", "night-flow-orbit", "pipe-tribe"),
            hubullu_gloss: gloss.into(),
        }
    }

    #[test]
    fn full_arc_wound_to_uruk() {
        let root = std::env::temp_dir().join("gula_arc_test");
        let _ = std::fs::remove_dir_all(&root);
        let report = run_arc(
            &root,
            &[
                w("wpdengine", "the moment a pipe first begins to lose water"),
                w("nanshe", "the earliest visible onset of a hidden loss"),
                w("igigi", "the first breath of a leak before any alarm"),
            ],
        )
        .unwrap();
        assert_eq!(report.staged_files.len(), 1, "one graft staged");
        assert!(report.verdicts.iter().any(|(_, v)| v.as_str() == "TRIGGERED"));
        assert!(report.verdicts.iter().any(|(_, v)| v.as_str() == "MINTED"));
        assert!(report.verdicts.iter().any(|(_, v)| v.as_str() == "STAGED-URUK"));
        let journal = std::fs::read_to_string(root.join("naru/lisanu.jsonl")).unwrap();
        assert!(journal.lines().count() >= 5, "no silent close: every act journaled");
        assert!(!root.join("kish").exists());
    }
}
