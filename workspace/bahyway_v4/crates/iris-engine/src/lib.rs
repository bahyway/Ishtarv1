// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  iris-engine · src/lib.rs
//  Sovereign IR camera hardware abstraction
//  Supports: FLIR Lepton, Optris PI, SEEK Thermal, simulated
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

pub use nusku_engine::BodyType;

/// Raw thermal frame — 16-zone float array (°C)
#[derive(Debug, Clone)]
pub struct IrisFrame {
    pub frame_id:     u64,
    pub temperatures: [f32; 16],  // one per body zone (Head=0 … RShin=15)
    pub body_type:    BodyType,
    pub width_px:     u32,
    pub height_px:    u32,
}

/// IR camera backend abstraction
pub trait IrisCamera: Send + Sync {
    fn capture_frame(&self) -> Result<IrisFrame, String>;
    fn camera_id(&self) -> &str;
}

/// Simulated camera for development / testing
pub struct SimulatedCamera {
    pub scenario:    SimScenario,
    frame_counter:   std::sync::atomic::AtomicU64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimScenario { Normal, Threat, Fever, Stress }

impl SimulatedCamera {
    pub fn new(scenario: SimScenario) -> Self {
        Self {
            scenario,
            frame_counter: std::sync::atomic::AtomicU64::new(0),
        }
    }
}

impl IrisCamera for SimulatedCamera {
    fn capture_frame(&self) -> Result<IrisFrame, String> {
        use std::sync::atomic::Ordering;
        let id = self.frame_counter.fetch_add(1, Ordering::Relaxed);

        // Sovereign baseline: Head, Neck, Chest, Abdomen,
        //   LShoulderZ, RShoulderZ, LUpperArm, RUpperArm,
        //   LForearm, RForearm, LHip, RHip,
        //   LThigh, RThigh, LShin, RShin
        let baseline: [f32; 16] = [
            36.8, 36.4, 36.6, 36.4,
            35.8, 35.8, 34.5, 34.5,
            33.8, 33.8, 35.6, 35.6,
            35.0, 35.0, 33.5, 33.5,
        ];
        let mut temps = baseline;
        let t = id as f32 / 60.0;

        match self.scenario {
            SimScenario::Normal => { /* natural variation only */ }

            SimScenario::Threat => {
                // Torso cold shadow + arm hot spots (concealed object signature)
                temps[2] -= 2.0 + (t * 0.7).sin() * 0.3; // Chest
                temps[3] -= 2.3;                           // Abdomen
                temps[4] -= 1.5; temps[5] -= 1.5;         // Shoulders
                temps[6] += 1.6; temps[7] += 1.0;         // Arms
                temps[0] += 1.0; temps[1] += 0.8;         // Head/neck stress
            }

            SimScenario::Fever => {
                for i in 0..16 { temps[i] += 1.8 + (t * 0.5).sin() * 0.2; }
                temps[0] += 0.5; // Head highest
            }

            SimScenario::Stress => {
                temps[0] += 1.4; temps[1] += 1.1; // Hot head
                temps[8] -= 0.6; temps[9] -= 0.6; // Cold hands (LForearm / RForearm)
            }
        }

        // Per-frame micro-variation
        for temp in temps.iter_mut() {
            *temp += (id as f32 * 0.1 + *temp).sin() * 0.15;
        }

        Ok(IrisFrame {
            frame_id:     id,
            temperatures: temps,
            body_type:    BodyType::Man,
            width_px:     80,
            height_px:    60,
        })
    }

    fn camera_id(&self) -> &str { "sim-001" }
}

// ── Tests ─────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_frame_within_physiological_range() {
        let cam = SimulatedCamera::new(SimScenario::Normal);
        let frame = cam.capture_frame().unwrap();
        for &t in &frame.temperatures {
            assert!(t > 30.0 && t < 40.0, "temperature {t} out of range");
        }
    }

    #[test]
    fn threat_frame_has_cold_torso_shadow() {
        let cam = SimulatedCamera::new(SimScenario::Threat);
        let frame = cam.capture_frame().unwrap();
        // Chest (index 2) and Abdomen (index 3) must be below baseline
        assert!(frame.temperatures[2] < 36.6, "chest should be cold");
        assert!(frame.temperatures[3] < 36.4, "abdomen should be cold");
    }

    #[test]
    fn fever_frame_elevates_all_zones() {
        let cam = SimulatedCamera::new(SimScenario::Fever);
        let frame = cam.capture_frame().unwrap();
        // All zones elevated; head (index 0) highest
        for &t in &frame.temperatures {
            assert!(t > 35.0, "fever should raise all zones");
        }
        assert!(frame.temperatures[0] > frame.temperatures[2]);
    }

    #[test]
    fn stress_frame_has_hot_head_cold_hands() {
        let cam = SimulatedCamera::new(SimScenario::Stress);
        let frame = cam.capture_frame().unwrap();
        assert!(frame.temperatures[0] > 36.8, "head should be hot in stress");
        assert!(frame.temperatures[8] < 33.8, "left forearm should be cold");
    }

    #[test]
    fn frame_counter_increments() {
        let cam = SimulatedCamera::new(SimScenario::Normal);
        let f0 = cam.capture_frame().unwrap();
        let f1 = cam.capture_frame().unwrap();
        assert_eq!(f1.frame_id, f0.frame_id + 1);
    }

    #[test]
    fn camera_id_is_stable() {
        let cam = SimulatedCamera::new(SimScenario::Normal);
        assert_eq!(cam.camera_id(), "sim-001");
    }
}
