//! UTNAPISHTIM — manifest.akk Generator
//! Sovereign assembly declaration for each client delivery

#![forbid(unsafe_code)]
use crate::ClientTopology;

pub fn generate_manifest(topo: &ClientTopology, output_dir: &str) -> String {
    format!(
        "; manifest.akk — UTNAPISHTIM 𒌓𒍣𒅁𒀭\n\
         ; Sovereign Assembly Declaration\n\
         ; Client: {} (0x{:04X})\n\
         ; BahyWay.Ecosystem v4.0 — DUB.SAR 𒁾\n\
         ; Sealed at: {}\n\
         ;\n\
         [assembly]\n\
         client_id    = {}\n\
         client_name  = {:?}\n\
         tribe_count  = {}\n\
         sealed_at    = {}\n\
         output_dir   = {:?}\n\
         ;\n\
         [sovereign]\n\
         kaki_version       = v4.0\n\
         plimpton_divisor   = 240\n\
         golden_angle_deg   = 137.507764\n\
         kur_threshold      = 0.40\n\
         colour_kur         = #1A0A2E\n\
         colour_dead        = #404040\n\
         colour_nergal_av   = #800000  ; AV engine only — never particles\n\
         ;\n\
         [deliverables]\n\
         threejs_viewer     = dubsar_viewer_{}.html\n\
         godot_project      = dubsar_pdm_{}\n\
         manifest           = manifest.akk\n",
        topo.client_name,
        topo.client_id,
        topo.sealed_at,
        topo.client_id,
        topo.client_name,
        topo.tribes.len(),
        topo.sealed_at,
        output_dir,
        topo.client_id,
        topo.client_id,
    )
}
