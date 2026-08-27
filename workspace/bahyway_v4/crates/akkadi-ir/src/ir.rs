//! 𒁾 AkkIr — The Complete Intermediate Representation

use crate::errors::IrError;
use crate::node::{AkkNode, EmitNode, ParticleNode, PipelineNode, TribeNode};
use crate::node_id::NodeId;
use crate::span::Span;
use std::collections::HashMap;

/// The complete IR for one `.akk` program.
#[derive(Debug, Clone)]
pub struct AkkIr {
    nodes: Vec<AkkNode>,
    by_id: HashMap<NodeId, usize>,
    by_name: HashMap<String, usize>,
    pub source: String,
    pub ir_version: String,
}

impl AkkIr {
    pub fn nodes(&self) -> &[AkkNode] {
        &self.nodes
    }
    pub fn len(&self) -> usize {
        self.nodes.len()
    }
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn get_by_id(&self, id: NodeId) -> Option<&AkkNode> {
        self.by_id.get(&id).map(|&i| &self.nodes[i])
    }

    pub fn get_by_name(&self, name: &str) -> Option<&AkkNode> {
        self.by_name.get(name).map(|&i| &self.nodes[i])
    }

    pub fn count_kind(&self, keyword: &str) -> usize {
        self.nodes
            .iter()
            .filter(|n| n.kind_keyword() == keyword)
            .count()
    }

    pub fn particles(&self) -> impl Iterator<Item = &crate::node::ParticleNode> {
        self.nodes.iter().filter_map(|n| {
            if let AkkNode::Particle(p) = n {
                Some(p)
            } else {
                None
            }
        })
    }

    pub fn tribes(&self) -> impl Iterator<Item = &crate::node::TribeNode> {
        self.nodes.iter().filter_map(|n| {
            if let AkkNode::Tribe(t) = n {
                Some(t)
            } else {
                None
            }
        })
    }

    pub fn rules(&self) -> impl Iterator<Item = &crate::node::RuleNode> {
        self.nodes.iter().filter_map(|n| {
            if let AkkNode::Rule(r) = n {
                Some(r)
            } else {
                None
            }
        })
    }

    pub fn emits(&self) -> impl Iterator<Item = &crate::node::EmitNode> {
        self.nodes.iter().filter_map(|n| {
            if let AkkNode::Emit(e) = n {
                Some(e)
            } else {
                None
            }
        })
    }

    pub fn is_generative(&self) -> bool {
        self.nodes.iter().any(|n| n.is_generative())
    }

    pub fn summary(&self) -> String {
        format!(
            "AkkIr[{}] particles={} tribes={} rules={} equations={} flows={} observes={} emits={}",
            self.source,
            self.count_kind("PARTICLE"),
            self.count_kind("TRIBE"),
            self.count_kind("RULE"),
            self.count_kind("EQUATION"),
            self.count_kind("FLOW"),
            self.count_kind("OBSERVE"),
            self.count_kind("EMIT"),
        )
    }
}

// ── IrBuilder ─────────────────────────────────────────────────────────────────

pub struct IrBuilder {
    nodes: Vec<AkkNode>,
    by_id: HashMap<NodeId, usize>,
    by_name: HashMap<String, usize>,
    source: String,
}

impl IrBuilder {
    pub fn new(source: &str) -> Self {
        Self {
            nodes: Vec::new(),
            by_id: HashMap::new(),
            by_name: HashMap::new(),
            source: source.to_string(),
        }
    }

    pub fn add_node(&mut self, node: AkkNode) -> Result<NodeId, IrError> {
        let name = node.name().to_string();
        let id = node.id();
        if self.by_name.contains_key(&name) {
            return Err(IrError::DuplicateNode {
                name,
                span: node.span().clone(),
            });
        }
        let idx = self.nodes.len();
        self.by_id.insert(id, idx);
        self.by_name.insert(name, idx);
        self.nodes.push(node);
        Ok(id)
    }

    pub fn upsert_node(&mut self, node: AkkNode) -> NodeId {
        let name = node.name().to_string();
        let id = node.id();
        if let Some(&existing_idx) = self.by_name.get(&name) {
            let old_id = self.nodes[existing_idx].id();
            self.by_id.remove(&old_id);
            self.by_id.insert(id, existing_idx);
            self.nodes[existing_idx] = node;
        } else {
            let idx = self.nodes.len();
            self.by_id.insert(id, idx);
            self.by_name.insert(name, idx);
            self.nodes.push(node);
        }
        id
    }

    pub fn build(self) -> Result<AkkIr, IrError> {
        if self.nodes.is_empty() {
            return Err(IrError::EmptyIr);
        }
        Ok(AkkIr {
            nodes: self.nodes,
            by_id: self.by_id,
            by_name: self.by_name,
            source: self.source,
            ir_version: crate::IR_VERSION.to_string(),
        })
    }

    pub fn build_unchecked(self) -> AkkIr {
        AkkIr {
            nodes: self.nodes,
            by_id: self.by_id,
            by_name: self.by_name,
            source: self.source,
            ir_version: crate::IR_VERSION.to_string(),
        }
    }

    // ── convenience helpers ───────────────────────────────────────────────────

    pub fn particle(&mut self, name: &str, span: Span) -> Result<NodeId, IrError> {
        self.add_node(AkkNode::Particle(ParticleNode::new(name, span)))
    }

    pub fn tribe_arabic_mdm(&mut self, name: &str, span: Span) -> Result<NodeId, IrError> {
        self.add_node(AkkNode::Tribe(TribeNode::sovereign_arabic_mdm(name, span)))
    }

    pub fn rule(&mut self, name: &str, span: Span) -> Result<NodeId, IrError> {
        self.add_node(AkkNode::Rule(crate::node::RuleNode::new(name, span)))
    }

    pub fn emit_self_heal(
        &mut self,
        name: &str,
        template: &str,
        span: Span,
    ) -> Result<NodeId, IrError> {
        self.add_node(AkkNode::Emit(EmitNode::self_heal(name, template, span)))
    }

    pub fn pipeline(mut self, name: &str, version: &str) -> Result<AkkIr, IrError> {
        if self.nodes.is_empty() {
            return Err(IrError::EmptyIr);
        }
        let span = Span::generated();
        let inner = self.nodes.drain(..).collect::<Vec<_>>();
        let mut pip = PipelineNode::new(name, version, span);
        for node in inner {
            pip = pip.push_node(node);
        }
        let mut final_b = IrBuilder::new(&self.source);
        final_b.add_node(AkkNode::Pipeline(pip))?;
        final_b.build()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;
    use crate::span::Span;

    fn sp() -> Span {
        Span::generated()
    }

    #[test]
    fn empty_ir_returns_error() {
        assert!(matches!(IrBuilder::new("t").build(), Err(IrError::EmptyIr)));
    }

    #[test]
    fn add_particle_then_build() {
        let mut b = IrBuilder::new("t");
        b.particle("citizen", sp()).unwrap();
        let ir = b.build().unwrap();
        assert_eq!(ir.len(), 1);
        assert_eq!(ir.count_kind("PARTICLE"), 1);
    }

    #[test]
    fn duplicate_returns_error() {
        let mut b = IrBuilder::new("t");
        b.particle("citizen", sp()).unwrap();
        assert!(matches!(
            b.particle("citizen", sp()),
            Err(IrError::DuplicateNode { .. })
        ));
    }

    #[test]
    fn upsert_replaces() {
        let mut b = IrBuilder::new("t");
        b.particle("citizen", sp()).unwrap();
        b.upsert_node(AkkNode::Particle(ParticleNode::new("citizen", sp())));
        assert_eq!(b.build().unwrap().len(), 1);
    }

    #[test]
    fn get_by_name() {
        let mut b = IrBuilder::new("t");
        b.particle("citizen", sp()).unwrap();
        b.tribe_arabic_mdm("IraqiMDM", sp()).unwrap();
        let ir = b.build().unwrap();
        assert!(ir.get_by_name("citizen").is_some());
        assert!(ir.get_by_name("none").is_none());
    }

    #[test]
    fn is_generative_with_emit() {
        let mut b = IrBuilder::new("t");
        b.emit_self_heal("heal", "tmpl", sp()).unwrap();
        assert!(b.build().unwrap().is_generative());
    }

    #[test]
    fn is_not_generative_without_emit() {
        let mut b = IrBuilder::new("t");
        b.particle("citizen", sp()).unwrap();
        assert!(!b.build().unwrap().is_generative());
    }

    #[test]
    fn summary_contains_source() {
        let mut b = IrBuilder::new("citizen.akk");
        b.particle("citizen", sp()).unwrap();
        assert!(b.build().unwrap().summary().contains("citizen.akk"));
    }
}
