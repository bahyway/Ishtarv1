//! .akk-sim — Sovereign Crystalline Simulation format.
//!
//! A .akk-sim file is the offline-replay format produced by the Quantum Freeze
//! protocol. It captures the last known state of the tribe topology so the
//! Dubsar IDE can replay the visualization without a live node.
//!
//! Format (human-readable text, subset of .akk syntax):
//!
//! ```text
//! simulation <name> {
//!   frozen_at_ms: <u64>
//!   reason:       "<freeze reason>"
//!   node_id:      <u8>
//!   b11_at_freeze: <u8>
//!   harmony:      <f32>
//!
//!   particle <tribe_id> { b11: <u8>  ring: <INNER|MID|OUTER>  x: <f32>  y: <f32>  z: <f32> }
//!   arc tribe_<a> <-> tribe_<b> { strength: <f32>  state: <state>  opacity: <f32> }
//! }
//! ```

/// A particle snapshot within a .akk-sim file.
#[derive(Debug, Clone)]
pub struct SimParticle {
    pub tribe_id: u16,
    pub b11:      u8,
    pub ring:     SimRing,
    pub x:        f32,
    pub y:        f32,
    pub z:        f32,
}

/// Orbital ring in the simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimRing { Inner, Mid, Outer }

impl SimRing {
    pub fn label(self) -> &'static str {
        match self { Self::Inner => "INNER", Self::Mid => "MID", Self::Outer => "OUTER" }
    }

    pub fn from_b11(b11: u8) -> Self {
        if      b11 >= 200 { Self::Inner }
        else if b11 >= 100 { Self::Mid }
        else               { Self::Outer }
    }
}

/// An arc snapshot within a .akk-sim file.
#[derive(Debug, Clone)]
pub struct SimArc {
    pub tribe_a:  u16,
    pub tribe_b:  u16,
    pub strength: f32,
    pub state:    String,
    pub opacity:  f32,
}

/// A complete crystalline simulation program.
#[derive(Debug, Clone)]
pub struct SimulationProgram {
    pub name:          String,
    pub frozen_at_ms:  u64,
    pub reason:        String,
    pub node_id:       u8,
    pub b11_at_freeze: u8,
    pub harmony:       f32,
    pub particles:     Vec<SimParticle>,
    pub arcs:          Vec<SimArc>,
}

impl SimulationProgram {
    /// Create an empty simulation (no particles, no arcs).
    pub fn new(name: impl Into<String>, frozen_at_ms: u64, reason: impl Into<String>,
               node_id: u8, b11_at_freeze: u8, harmony: f32) -> Self {
        Self {
            name: name.into(), frozen_at_ms, reason: reason.into(),
            node_id, b11_at_freeze, harmony,
            particles: Vec::new(), arcs: Vec::new(),
        }
    }

    pub fn add_particle(&mut self, p: SimParticle) { self.particles.push(p); }
    pub fn add_arc(&mut self, a: SimArc)           { self.arcs.push(a); }

    /// Serialize to .akk-sim source text.
    pub fn to_source(&self) -> String {
        let mut s = String::with_capacity(512);
        s.push_str(&format!("simulation {} {{\n", self.name));
        s.push_str(&format!("  frozen_at_ms:  {}\n", self.frozen_at_ms));
        s.push_str(&format!("  reason:        \"{}\"\n", self.reason));
        s.push_str(&format!("  node_id:       {}\n", self.node_id));
        s.push_str(&format!("  b11_at_freeze: {}\n", self.b11_at_freeze));
        s.push_str(&format!("  harmony:       {:.4}\n\n", self.harmony));
        for p in &self.particles {
            s.push_str(&format!(
                "  particle {} {{ b11: {}  ring: {}  x: {:.4}  y: {:.4}  z: {:.4} }}\n",
                p.tribe_id, p.b11, p.ring.label(), p.x, p.y, p.z
            ));
        }
        if !self.particles.is_empty() { s.push('\n'); }
        for a in &self.arcs {
            s.push_str(&format!(
                "  arc tribe_{} <-> tribe_{} {{ strength: {:.4}  state: {}  opacity: {:.4} }}\n",
                a.tribe_a, a.tribe_b, a.strength, a.state, a.opacity
            ));
        }
        s.push_str("}\n");
        s
    }

    /// Parse a .akk-sim source string (header fields only — arcs/particles skipped).
    ///
    /// Returns `Err(String)` if the required header fields are missing.
    pub fn parse_header(src: &str) -> Result<SimulationProgram, String> {
        let mut name        = None::<String>;
        let mut frozen_at   = None::<u64>;
        let mut reason      = None::<String>;
        let mut node_id     = None::<u8>;
        let mut b11_freeze  = None::<u8>;
        let mut harmony     = None::<f32>;

        for line in src.lines() {
            let line = line.trim();
            if line.starts_with("simulation ") && line.ends_with('{') {
                let n = line.strip_prefix("simulation ").unwrap()
                            .trim_end_matches('{').trim();
                name = Some(n.to_string());
            } else if let Some(v) = line.strip_prefix("frozen_at_ms:") {
                frozen_at = v.trim().parse().ok();
            } else if let Some(v) = line.strip_prefix("reason:") {
                reason = Some(v.trim().trim_matches('"').to_string());
            } else if let Some(v) = line.strip_prefix("node_id:") {
                node_id = v.trim().parse().ok();
            } else if let Some(v) = line.strip_prefix("b11_at_freeze:") {
                b11_freeze = v.trim().parse().ok();
            } else if let Some(v) = line.strip_prefix("harmony:") {
                harmony = v.trim().parse().ok();
            }
        }

        Ok(SimulationProgram::new(
            name.ok_or("missing simulation name")?,
            frozen_at.ok_or("missing frozen_at_ms")?,
            reason.ok_or("missing reason")?,
            node_id.ok_or("missing node_id")?,
            b11_freeze.ok_or("missing b11_at_freeze")?,
            harmony.ok_or("missing harmony")?,
        ))
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_sim() -> SimulationProgram {
        let mut sim = SimulationProgram::new(
            "freeze_node0_9000", 9_000, "LAST_NODE_FAILING", 0, 45, 0.72
        );
        sim.add_particle(SimParticle { tribe_id: 1, b11: 210, ring: SimRing::Inner,
                                       x: 1.0, y: 0.0, z: 0.1 });
        sim.add_arc(SimArc { tribe_a: 1, tribe_b: 2, strength: 0.8,
                             state: "FROZEN".into(), opacity: 0.64 });
        sim
    }

    #[test]
    fn serialise_roundtrip_header() {
        let sim = make_sim();
        let src = sim.to_source();
        let parsed = SimulationProgram::parse_header(&src).unwrap();
        assert_eq!(parsed.name, "freeze_node0_9000");
        assert_eq!(parsed.frozen_at_ms, 9_000);
        assert_eq!(parsed.node_id, 0);
        assert_eq!(parsed.b11_at_freeze, 45);
        assert!((parsed.harmony - 0.72).abs() < 1e-3);
    }

    #[test]
    fn source_contains_particle_and_arc() {
        let src = make_sim().to_source();
        assert!(src.contains("particle 1"));
        assert!(src.contains("ring: INNER"));
        assert!(src.contains("arc tribe_1 <-> tribe_2"));
        assert!(src.contains("FROZEN"));
    }

    #[test]
    fn source_starts_with_simulation() {
        let src = make_sim().to_source();
        assert!(src.starts_with("simulation freeze_node0_9000 {"));
    }

    #[test]
    fn source_ends_with_closing_brace() {
        let src = make_sim().to_source();
        assert!(src.ends_with("}\n"));
    }

    #[test]
    fn sim_ring_from_b11() {
        assert_eq!(SimRing::from_b11(240), SimRing::Inner);
        assert_eq!(SimRing::from_b11(150), SimRing::Mid);
        assert_eq!(SimRing::from_b11(50),  SimRing::Outer);
    }

    #[test]
    fn parse_header_error_on_missing_field() {
        let incomplete = "simulation test_sim {\n  frozen_at_ms: 1000\n}\n";
        assert!(SimulationProgram::parse_header(incomplete).is_err());
    }

    #[test]
    fn empty_sim_serialises_cleanly() {
        let sim = SimulationProgram::new("empty_freeze", 0, "MANUAL_TRIGGER", 1, 180, 1.0);
        let src = sim.to_source();
        assert!(src.contains("simulation empty_freeze"));
        assert!(src.contains("MANUAL_TRIGGER"));
    }
}
