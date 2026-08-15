//! 𒁾 AkkWalker — Sovereign IR Tree Traversal

use crate::ir::AkkIr;
use crate::node::AkkNode;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WalkAction { Continue, Skip, Stop }

/// Visitor trait for walking the AkkIr tree.
pub trait AkkWalker {
    fn visit(&mut self, node: &AkkNode) -> WalkAction;

    fn walk(&mut self, ir: &AkkIr) {
        for node in ir.nodes() {
            match self.visit(node) {
                WalkAction::Stop     => return,
                WalkAction::Skip     => continue,
                WalkAction::Continue => {
                    match node {
                        AkkNode::Guard(g)    => {
                            for child in &g.body {
                                if self.visit(child) == WalkAction::Stop { return; }
                            }
                        }
                        AkkNode::Pipeline(p) => {
                            for child in &p.nodes {
                                if self.visit(child) == WalkAction::Stop { return; }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

// ── Built-in walkers ──────────────────────────────────────────────────────────

pub struct CollectNames { pub kind: &'static str, pub names: Vec<String> }

impl CollectNames {
    pub fn of_kind(kind: &'static str) -> Self { Self { kind, names: vec![] } }
}

impl AkkWalker for CollectNames {
    fn visit(&mut self, node: &AkkNode) -> WalkAction {
        if node.kind_keyword() == self.kind { self.names.push(node.name().to_string()); }
        WalkAction::Continue
    }
}

pub struct ValidateQuality { pub errors: Vec<String> }

impl ValidateQuality {
    pub fn new() -> Self { Self { errors: vec![] } }
    pub fn is_valid(&self) -> bool { self.errors.is_empty() }
}

impl Default for ValidateQuality { fn default() -> Self { Self::new() } }

impl AkkWalker for ValidateQuality {
    fn visit(&mut self, node: &AkkNode) -> WalkAction {
        if let AkkNode::Particle(p) = node {
            for field in &p.fields {
                for c in &field.constraints {
                    if !(0.0..=1.0).contains(&c.min_value) {
                        self.errors.push(format!("PARTICLE {}: field {} constraint {:.3} out of [0,1]",
                            p.name, field.name, c.min_value));
                    }
                }
            }
        }
        if let AkkNode::Tribe(t) = node {
            let sum: f32 = t.weights.weights.iter().sum();
            if (sum - 1.0).abs() > 0.001 {
                self.errors.push(format!("TRIBE {}: weights sum to {sum:.4}, not 1.0", t.name));
            }
        }
        WalkAction::Continue
    }
}

pub struct NodeCounter { pub counts: std::collections::HashMap<&'static str, usize> }

impl NodeCounter {
    pub fn new() -> Self { Self { counts: std::collections::HashMap::new() } }
    pub fn get(&self, kind: &str) -> usize { self.counts.get(kind).copied().unwrap_or(0) }
    pub fn total(&self) -> usize { self.counts.values().sum() }
}

impl Default for NodeCounter { fn default() -> Self { Self::new() } }

impl AkkWalker for NodeCounter {
    fn visit(&mut self, node: &AkkNode) -> WalkAction {
        *self.counts.entry(node.kind_keyword()).or_insert(0) += 1;
        WalkAction::Continue
    }
}

pub struct FindGenerative { pub found: Vec<String> }

impl FindGenerative {
    pub fn new() -> Self { Self { found: vec![] } }
    pub fn any_found(&self) -> bool { !self.found.is_empty() }
}

impl Default for FindGenerative { fn default() -> Self { Self::new() } }

impl AkkWalker for FindGenerative {
    fn visit(&mut self, node: &AkkNode) -> WalkAction {
        if node.is_generative() { self.found.push(node.name().to_string()); }
        WalkAction::Continue
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::IrBuilder;
    use crate::span::Span;

    fn make_ir() -> AkkIr {
        let mut b = IrBuilder::new("test.akk");
        b.particle("citizen",  Span::generated()).unwrap();
        b.particle("building", Span::generated()).unwrap();
        b.tribe_arabic_mdm("IraqiMDM", Span::generated()).unwrap();
        b.rule("promote_gem",  Span::generated()).unwrap();
        b.emit_self_heal("heal_nulls", "fix_null_template", Span::generated()).unwrap();
        b.build().unwrap()
    }

    #[test]
    fn counter_counts_all_kinds() {
        let ir = make_ir();
        let mut c = NodeCounter::new();
        c.walk(&ir);
        assert_eq!(c.get("PARTICLE"), 2);
        assert_eq!(c.get("TRIBE"),    1);
        assert_eq!(c.get("RULE"),     1);
        assert_eq!(c.get("EMIT"),     1);
        assert_eq!(c.total(),         5);
    }

    #[test]
    fn collect_names_finds_particles() {
        let ir = make_ir();
        let mut col = CollectNames::of_kind("PARTICLE");
        col.walk(&ir);
        assert_eq!(col.names.len(), 2);
        assert!(col.names.contains(&"citizen".to_string()));
    }

    #[test]
    fn find_generative_finds_emit() {
        let ir = make_ir();
        let mut f = FindGenerative::new();
        f.walk(&ir);
        assert!(f.any_found());
        assert!(f.found.contains(&"heal_nulls".to_string()));
    }

    #[test]
    fn validate_quality_passes_clean_ir() {
        let ir = make_ir();
        let mut v = ValidateQuality::new();
        v.walk(&ir);
        assert!(v.is_valid());
    }

    #[test]
    fn stop_halts_traversal() {
        struct StopFirst { count: usize }
        impl AkkWalker for StopFirst {
            fn visit(&mut self, _: &AkkNode) -> WalkAction { self.count += 1; WalkAction::Stop }
        }
        let ir = make_ir();
        let mut w = StopFirst { count: 0 };
        w.walk(&ir);
        assert_eq!(w.count, 1);
    }
}
