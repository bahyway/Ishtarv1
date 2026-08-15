//! Cell magic handlers for the DubSar kernel.
//!
//! Supported magics:
//!   %%dubsar sim   — run simulation step
//!   %%dubsar stats — print fragmentation statistics
//!   %%dubsar proj  — change 7D projection (pass JSON as body)
//!   %%glsl         — compile and display a GLSL compute shader (no-op in SW mode)

use serde_json::Value;

use crate::particle::ParticleCloud;
use crate::projection::Projection7D3D;

pub enum MagicResult {
    /// Displayable image as base64 PNG
    Image(String),
    /// Plain text output
    Text(String),
    /// JSON data for analysis cells
    Json(Value),
    /// Not a magic command — caller should do normal execution
    NotAMagic,
}

pub struct MagicHandler {
    pub cloud: Option<ParticleCloud>,
    pub projection: Projection7D3D,
}

impl MagicHandler {
    pub fn new() -> Self {
        Self {
            cloud: None,
            projection: Projection7D3D::default(),
        }
    }

    pub fn handle(&mut self, code: &str) -> MagicResult {
        let trimmed = code.trim();

        if let Some(rest) = trimmed.strip_prefix("%%dubsar") {
            return self.handle_dubsar(rest.trim());
        }

        if trimmed.starts_with("%%glsl") {
            return MagicResult::Text(
                "%%glsl: GLSL shader accepted. GPU path disabled in Hyper-V VM (no passthrough).\n\
                 Enable vulkan-backend feature on bare-metal Fedora 44 to execute shaders."
                    .to_string(),
            );
        }

        MagicResult::NotAMagic
    }

    fn handle_dubsar(&mut self, args: &str) -> MagicResult {
        let (cmd, body) = args
            .split_once('\n')
            .unwrap_or((args, ""));

        match cmd.trim() {
            "init" | "init-storage" => {
                let count: usize = body.trim().parse().unwrap_or(100_000);
                self.cloud = Some(ParticleCloud::new_storage_fragmentation(count));
                MagicResult::Text(format!(
                    "Initialised storage fragmentation cloud: {} particles (Tribe A + B + free)",
                    count
                ))
            }

            "step" => {
                let steps: u32 = body.trim().parse().unwrap_or(1);
                if let Some(ref mut cloud) = self.cloud {
                    for _ in 0..steps {
                        cloud.step(0.016);
                    }
                    let stats = cloud.fragmentation_stats();
                    MagicResult::Json(serde_json::to_value(stats).unwrap())
                } else {
                    MagicResult::Text(
                        "No simulation active. Run: %%dubsar init-storage <count>".to_string(),
                    )
                }
            }

            "stats" => {
                if let Some(ref cloud) = self.cloud {
                    let stats = cloud.fragmentation_stats();
                    let json = serde_json::to_value(&stats).unwrap();
                    MagicResult::Json(json)
                } else {
                    MagicResult::Text("No simulation active.".to_string())
                }
            }

            "proj" => {
                // Parse JSON projection matrix from cell body
                match serde_json::from_str::<Projection7D3D>(body.trim()) {
                    Ok(proj) => {
                        self.projection = proj;
                        MagicResult::Text("Projection updated.".to_string())
                    }
                    Err(e) => MagicResult::Text(format!(
                        "Invalid projection JSON: {e}\n\nExpected format:\n{}",
                        serde_json::to_string_pretty(&Projection7D3D::default()).unwrap()
                    )),
                }
            }

            "proj-phase-space" => {
                self.projection = Projection7D3D::phase_space();
                MagicResult::Text(
                    "Projection set to phase-space view (pos.x vs mom.x vs scalar).".to_string(),
                )
            }

            "render" => {
                if let Some(ref cloud) = self.cloud {
                    use crate::renderer::SoftwareRenderer;
                    let points = cloud.project(&self.projection);
                    let r = SoftwareRenderer::new(800, 600).unwrap();
                    let frame = r.render_frame(&points, cloud.step_count()).unwrap();
                    let b64 = SoftwareRenderer::to_base64_png(&frame).unwrap();
                    MagicResult::Image(b64)
                } else {
                    MagicResult::Text("No simulation active.".to_string())
                }
            }

            "help" | "" => MagicResult::Text(
                "DubSar cell magics:\n\
                 %%dubsar init-storage <N>   — init N particles (storage fragmentation)\n\
                 %%dubsar step [N]           — advance N steps (default 1)\n\
                 %%dubsar stats              — print fragmentation statistics as JSON\n\
                 %%dubsar render             — render current state to PNG in output cell\n\
                 %%dubsar proj               — set 7D projection (JSON body)\n\
                 %%dubsar proj-phase-space   — switch to phase-space view\n\
                 %%glsl                      — accept GLSL (GPU path, bare-metal only)"
                    .to_string(),
            ),

            other => MagicResult::Text(format!(
                "Unknown %%dubsar command: '{}'. Try: %%dubsar help",
                other
            )),
        }
    }
}
