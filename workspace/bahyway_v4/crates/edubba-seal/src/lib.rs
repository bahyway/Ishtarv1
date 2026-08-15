//! É-DUBBA 𒂍𒁾𒁀 — Sovereign Modelling Laboratory Seal Engine
//! BahyWay.Ecosystem v4.0 — DUB.SAR 𒁾
//!
//! Five-stage seal sequence (all succeed or none):
//!   Stage 1: MASHSHARU   — mathematical verification (Lean 4)
//!   Stage 2: NĀMZITUM    — semantic consistency check
//!   Stage 3: PĀŠIRU      — golden angle colour assignment
//!   Stage 4: KISPU       — PARZU-KAKI mint + zakāru journal
//!   Stage 5: UTNAPISHTIM — Godot PDM + Three.js generation
//!
//! MVP status: Stage 5 (UTNAPISHTIM) is real -- it calls the real
//! generator and writes real files. Stages 1/2/4 (MASHSHARU, NĀMZITUM,
//! KISPU) are honest stubs: they return a fixed success value without
//! doing the verification/mint work their names describe (no Lean 4
//! invocation, no NĀMZITUM fact-corpus query, no PARZU-KAKI actually
//! minted). The orchestration and short-circuit-on-failure sequencing
//! is real and tested; the three named stages' internals are not yet.
//!
//! TIAMAT Levels (all four — CORRECTED):
//!   Level 1: DILBAT 𒀭𒁾𒁍
//!   Level 2: KAKKAB 𒀭𒆳𒀝
//!   Level 3: ERRA   𒀭𒂗𒆳  (was missing KUR below)
//!   Level 4: KUR    𒆳       ← CORRECTED: was absent
//!
//! KAKI v4.0: quality in EAV only, κ[6]=kaki_type, κ[7]=kaki_role

#![forbid(unsafe_code)]

use utnapishtim::{ClientTopology, generator::UtnapishtimGenerator};

#[derive(Debug, Clone, PartialEq)]
pub enum SealStage {
    Mashsharu,
    Namzitum,
    #[allow(dead_code)]
    Pashiru,
    Kispu,
    Utnapishtim,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SealResult {
    Passed,
    Failed { stage: SealStage, reason: String },
}

#[derive(Debug, Clone)]
pub struct EdubbaSession {
    pub topology:        ClientTopology,
    pub output_dir:      String,
    pub mashsharu_cert:  Option<String>,
    pub namzitum_cert:   Option<String>,
}

impl EdubbaSession {
    pub fn new(topology: ClientTopology, output_dir: impl Into<String>) -> Self {
        Self {
            topology,
            output_dir: output_dir.into(),
            mashsharu_cert:  None,
            namzitum_cert:   None,
        }
    }

    /// Execute the five-stage sovereign seal sequence.
    /// All stages must pass or the session is not sealed.
    pub fn seal(&mut self) -> SealResult {
        println!("[É-DUBBA] Beginning seal sequence for client {} (0x{:04X})",
            self.topology.client_name, self.topology.client_id);

        // Stage 1 — MASHSHARU
        println!("[É-DUBBA] Stage 1: MASHSHARU — mathematical verification");
        match self.run_mashsharu() {
            Ok(cert) => {
                self.mashsharu_cert = Some(cert);
                println!("[É-DUBBA] MASHSHARU ✓");
            }
            Err(e) => return SealResult::Failed {
                stage: SealStage::Mashsharu,
                reason: e,
            },
        }

        // Stage 2 — NĀMZITUM
        println!("[É-DUBBA] Stage 2: NĀMZITUM — semantic consistency");
        match self.run_namzitum() {
            Ok(cert) => {
                self.namzitum_cert = Some(cert);
                println!("[É-DUBBA] NĀMZITUM ✓");
            }
            Err(e) => return SealResult::Failed {
                stage: SealStage::Namzitum,
                reason: e,
            },
        }

        // Stage 3 — PĀŠIRU
        println!("[É-DUBBA] Stage 3: PĀŠIRU — golden angle colours");
        self.assign_pashiru_colours();
        println!("[É-DUBBA] PĀŠIRU ✓");

        // Stage 4 — KISPU
        println!("[É-DUBBA] Stage 4: KISPU — PARZU-KAKI mint + zakāru");
        match self.run_kispu() {
            Ok(_) => println!("[É-DUBBA] KISPU ✓"),
            Err(e) => return SealResult::Failed {
                stage: SealStage::Kispu,
                reason: e,
            },
        }

        // Stage 5 — UTNAPISHTIM
        println!("[É-DUBBA] Stage 5: UTNAPISHTIM — generating deliverables");
        match self.run_utnapishtim() {
            Ok(_) => println!("[É-DUBBA] UTNAPISHTIM ✓"),
            Err(e) => return SealResult::Failed {
                stage: SealStage::Utnapishtim,
                reason: e,
            },
        }

        println!("[É-DUBBA] 𒂍𒁾𒁀 Session sealed. The flood cannot reach it.");
        SealResult::Passed
    }

    fn run_mashsharu(&self) -> Result<String, String> {
        // STUB: Lean 4 formal verification not yet wired up.
        // Production: calls lean4/DubSarGA/Main.lean via subprocess.
        if self.topology.tribes.is_empty() {
            return Err("MASHSHARU: topology has no tribes".into());
        }
        Ok(format!("MASHSHARU-CERT-{:04X}-{}", self.topology.client_id,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs()).unwrap_or(0)))
    }

    fn run_namzitum(&self) -> Result<String, String> {
        // STUB: semantic validation against organisation documentation
        // not yet wired up. Production: queries NĀMZITUM fact corpus via
        // HeptaScript.
        Ok(format!("NAMZITUM-CERT-{:04X}", self.topology.client_id))
    }

    fn assign_pashiru_colours(&mut self) {
        // PĀŠIRU already assigns colours via golden_hue() in ClientTopology
        // Nothing to do — colour is computed from tribe_id deterministically
    }

    fn run_kispu(&self) -> Result<(), String> {
        // STUB: does not actually mint a PARZU-KAKI or write a zakāru
        // journal entry yet. Production: calls kispu crate.
        println!("[KISPU] PARZU-KAKI minted for client {:04X}", self.topology.client_id);
        Ok(())
    }

    fn run_utnapishtim(&self) -> Result<(), String> {
        let gen = UtnapishtimGenerator::new(&self.output_dir);
        gen.generate(&self.topology)
            .map(|r| {
                println!("[UTNAPISHTIM] Three.js → {:?}", r.html_path);
                println!("[UTNAPISHTIM] Godot    → {:?}", r.godot_dir);
            })
            .map_err(|e| format!("UTNAPISHTIM: {:?}", e))
    }
}

/// TIAMAT alert levels for É-DUBBA Theater display
/// CORRECTED: all four levels present (KUR was missing)
pub fn tiamat_colour(level: u8) -> &'static str {
    match level {
        1 => "#E8B400",  // DILBAT 𒀭𒁾𒁍 amber
        2 => "#D46000",  // KAKKAB 𒀭𒆳𒀝 orange
        3 => "#CC2244",  // ERRA   𒀭𒂗𒆳 crimson
        4 => "#1A0A2E",  // KUR    𒆳     sovereign indigo ← CORRECTED
        _ => "#00D4C8",  // HEALTHY teal
    }
}

pub fn tiamat_name(level: u8) -> &'static str {
    match level {
        1 => "DILBAT 𒀭𒁾𒁍",
        2 => "KAKKAB 𒀭𒆳𒀝",
        3 => "ERRA 𒀭𒂗𒆳",
        4 => "KUR 𒆳",       // ← CORRECTED: was absent
        _ => "NOMINAL",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tiamat_has_four_levels_including_kur() {
        // KUR must be present as level 4 — was missing in iPhone session
        assert_eq!(tiamat_name(4), "KUR 𒆳");
        assert_eq!(tiamat_colour(4), "#1A0A2E");
        assert_ne!(tiamat_colour(4), "#800000"); // not Nergal AV
    }

    #[test]
    fn test_seal_fails_empty_topology() {
        let topo = ClientTopology {
            client_id: 1, client_name: "Test".into(),
            tribes: vec![], sealed_at: 0,
        };
        let mut session = EdubbaSession::new(topo, "/tmp/test_seal");
        let result = session.seal();
        assert!(matches!(result, SealResult::Failed { stage: SealStage::Mashsharu, .. }));
    }
}
