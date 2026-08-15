//! 𒁾 AkkNode — The Sovereign IR Node Tree
//!
//! Every `.akk` construct maps to one of these node variants.
//! Physical homoiconicity: `ParticleNode` and `RuleNode` are both
//! `AkkNode` variants — same species, different behavior.

use crate::node_id::NodeId;
use crate::span::Span;
use crate::quality::{QualityLane, HeptaDim};
use crate::kinetic::KineticForce;

// ── Top-level AkkNode ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum AkkNode {
    Particle(ParticleNode),
    Tribe(TribeNode),
    Rule(RuleNode),
    Equation(EquationNode),
    Flow(FlowNode),
    Observe(ObserveNode),
    Emit(EmitNode),
    Guard(GuardNode),
    Pipeline(PipelineNode),
}

impl AkkNode {
    pub fn id(&self) -> NodeId {
        match self {
            Self::Particle(n) => n.id,
            Self::Tribe(n)    => n.id,
            Self::Rule(n)     => n.id,
            Self::Equation(n) => n.id,
            Self::Flow(n)     => n.id,
            Self::Observe(n)  => n.id,
            Self::Emit(n)     => n.id,
            Self::Guard(n)    => n.id,
            Self::Pipeline(n) => n.id,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Particle(n) => &n.name,
            Self::Tribe(n)    => &n.name,
            Self::Rule(n)     => &n.name,
            Self::Equation(n) => &n.name,
            Self::Flow(n)     => &n.name,
            Self::Observe(n)  => &n.name,
            Self::Emit(n)     => &n.name,
            Self::Guard(n)    => &n.name,
            Self::Pipeline(n) => &n.name,
        }
    }

    pub fn kind_keyword(&self) -> &'static str {
        match self {
            Self::Particle(_) => "PARTICLE",
            Self::Tribe(_)    => "TRIBE",
            Self::Rule(_)     => "RULE",
            Self::Equation(_) => "EQUATION",
            Self::Flow(_)     => "FLOW",
            Self::Observe(_)  => "OBSERVE",
            Self::Emit(_)     => "EMIT",
            Self::Guard(_)    => "GUARD",
            Self::Pipeline(_) => "PIPELINE",
        }
    }

    pub fn span(&self) -> &Span {
        match self {
            Self::Particle(n) => &n.span,
            Self::Tribe(n)    => &n.span,
            Self::Rule(n)     => &n.span,
            Self::Equation(n) => &n.span,
            Self::Flow(n)     => &n.span,
            Self::Observe(n)  => &n.span,
            Self::Emit(n)     => &n.span,
            Self::Guard(n)    => &n.span,
            Self::Pipeline(n) => &n.span,
        }
    }

    pub fn is_generative(&self) -> bool {
        matches!(self, Self::Emit(_) | Self::Equation(_))
    }

    pub fn child_count(&self) -> usize {
        match self {
            Self::Particle(n) => n.fields.len(),
            Self::Tribe(n)    => n.weights.len(),
            Self::Rule(n)     => n.conditions.len() + n.actions.len(),
            Self::Equation(n) => n.terms.len(),
            Self::Flow(n)     => n.stages.len(),
            Self::Observe(n)  => n.filters.len(),
            Self::Emit(n)     => n.context.len(),
            Self::Guard(n)    => 1 + n.body.len(),
            Self::Pipeline(n) => n.nodes.len(),
        }
    }
}

impl std::fmt::Display for AkkNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} ({})", self.kind_keyword(), self.name(), self.id())
    }
}

// ── PARTICLE ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ParticleNode {
    pub id:        NodeId,
    pub name:      String,
    pub fields:    Vec<FieldDecl>,
    pub tribe_ref: Option<String>,
    pub doc:       Option<String>,
    pub span:      Span,
}

impl ParticleNode {
    pub fn new(name: &str, span: Span) -> Self {
        Self { id: NodeId::from_parts(&["PARTICLE", name]), name: name.to_string(),
               fields: vec![], tribe_ref: None, doc: None, span }
    }
    pub fn with_field(mut self, f: FieldDecl)  -> Self { self.fields.push(f); self }
    pub fn with_tribe(mut self, t: &str)        -> Self { self.tribe_ref = Some(t.to_string()); self }
    pub fn with_doc(mut self, d: &str)          -> Self { self.doc = Some(d.to_string()); self }
}

#[derive(Debug, Clone)]
pub struct FieldDecl {
    pub name:        String,
    pub field_type:  String,
    pub constraints: Vec<QualityConstraint>,
    pub is_kaki:     bool,
    pub nullable:    bool,
}

impl FieldDecl {
    pub fn new(name: &str, field_type: &str) -> Self {
        Self { name: name.to_string(), field_type: field_type.to_string(),
               constraints: vec![], is_kaki: false, nullable: false }
    }
    pub fn as_kaki(mut self) -> Self { self.is_kaki = true; self }
    pub fn nullable(mut self) -> Self { self.nullable = true; self }
    pub fn with_constraint(mut self, c: QualityConstraint) -> Self { self.constraints.push(c); self }
}

impl std::fmt::Display for FieldDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.field_type)
    }
}

#[derive(Debug, Clone)]
pub struct QualityConstraint {
    pub dimension: HeptaDim,
    pub min_value: f32,
    pub validator: Option<String>,
}

impl QualityConstraint {
    pub fn threshold(dim: HeptaDim, min: f32) -> Self {
        Self { dimension: dim, min_value: min, validator: None }
    }
    pub fn validator(dim: HeptaDim, name: &str) -> Self {
        Self { dimension: dim, min_value: 0.0, validator: Some(name.to_string()) }
    }
}

// ── TRIBE ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct TribeNode {
    pub id:      NodeId,
    pub name:    String,
    pub weights: WeightProfile,
    pub ideal:   [f32; 7],
    pub doc:     Option<String>,
    pub span:    Span,
}

impl TribeNode {
    pub fn sovereign_arabic_mdm(name: &str, span: Span) -> Self {
        Self { id: NodeId::from_parts(&["TRIBE", name]), name: name.to_string(),
               weights: WeightProfile::sovereign_arabic_mdm(), ideal: [1.0; 7], doc: None, span }
    }
    pub fn equal_weights(name: &str, span: Span) -> Self {
        Self { id: NodeId::from_parts(&["TRIBE", name]), name: name.to_string(),
               weights: WeightProfile::equal(), ideal: [1.0; 7], doc: None, span }
    }
}

#[derive(Debug, Clone)]
pub struct WeightProfile {
    pub name:    String,
    pub weights: [f32; 7],
}

impl WeightProfile {
    pub fn sovereign_arabic_mdm() -> Self {
        Self { name: "sovereign_arabic_mdm".into(), weights: [0.30, 0.20, 0.15, 0.15, 0.10, 0.05, 0.05] }
    }
    pub fn equal() -> Self {
        Self { name: "equal".into(), weights: [1.0/7.0; 7] }
    }
    pub fn is_valid(&self) -> bool { (self.weights.iter().sum::<f32>() - 1.0).abs() < 0.001 }
    pub fn len(&self) -> usize { self.weights.len() }
    pub fn is_empty(&self) -> bool { false }
}

// ── RULE ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct RuleNode {
    pub id:         NodeId,
    pub name:       String,
    pub conditions: Vec<RuleCondition>,
    pub actions:    Vec<RuleAction>,
    pub doc:        Option<String>,
    pub priority:   u32,
    pub span:       Span,
}

impl RuleNode {
    pub fn new(name: &str, span: Span) -> Self {
        Self { id: NodeId::from_parts(&["RULE", name]), name: name.to_string(),
               conditions: vec![], actions: vec![], doc: None, priority: 100, span }
    }
    pub fn when(mut self, c: RuleCondition) -> Self { self.conditions.push(c); self }
    pub fn then(mut self, a: RuleAction)    -> Self { self.actions.push(a);    self }
}

#[derive(Debug, Clone)]
pub struct RuleCondition {
    pub lhs: String,
    pub op:  ConditionOp,
    pub rhs: ConditionRhs,
}

#[derive(Debug, Clone)]
pub enum ConditionRhs {
    Float(f32),
    Str(String),
    Int(i64),
    Lane(QualityLane),
    Dim(HeptaDim),
    FieldRef(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConditionOp { Gte, Lte, Gt, Lt, Eq, Neq }

impl ConditionOp {
    pub fn akk_syntax(&self) -> &'static str {
        match self {
            Self::Gte => ">=", Self::Lte => "<=",
            Self::Gt  => ">",  Self::Lt  => "<",
            Self::Eq  => "=",  Self::Neq => "!=",
        }
    }
}

#[derive(Debug, Clone)]
pub enum RuleAction {
    SetLane(QualityLane),
    /// Emit a Story audit event
    EmitStory { label: String },
    EmitAkk   { template: String, params: Vec<(String, String)> },
    SetOrbit  { lane: QualityLane },
    SendToSteward { reason: String },
    /// Escalate to Zero security
    EscalateZero  { reason: String },
    SetField  { field: String, value: String },
    ReScore,
    GotoStage { stage: String },
}

// ── EQUATION ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct EquationNode {
    pub id:         NodeId,
    pub name:       String,
    pub terms:      Vec<KineticTerm>,
    pub body:       EquationBody,
    pub generative: bool,
    pub doc:        Option<String>,
    pub span:       Span,
}

impl EquationNode {
    pub fn new(name: &str, span: Span) -> Self {
        Self { id: NodeId::from_parts(&["EQUATION", name]), name: name.to_string(),
               terms: vec![], body: EquationBody::Identity, generative: false, doc: None, span }
    }
    pub fn generative(mut self) -> Self { self.generative = true; self }
}

#[derive(Debug, Clone)]
pub struct KineticTerm {
    pub force: KineticForce,
    pub coeff: f32,
    pub label: String,
}

#[derive(Debug, Clone)]
pub enum EquationBody {
    Identity,
    KineticSum,
    HealthScore { weights: [f32; 7] },
    HealthDerivative,
    Custom(String),
}

// ── FLOW ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct FlowNode {
    pub id:     NodeId,
    pub name:   String,
    pub source: FlowSource,
    pub stages: Vec<FlowStage>,
    pub sink:   FlowSink,
    pub doc:    Option<String>,
    pub span:   Span,
}

impl FlowNode {
    pub fn new(name: &str, source: FlowSource, sink: FlowSink, span: Span) -> Self {
        Self { id: NodeId::from_parts(&["FLOW", name]), name: name.to_string(),
               source, stages: vec![], sink, doc: None, span }
    }
    pub fn pipe(mut self, stage: FlowStage) -> Self { self.stages.push(stage); self }
}

/// Where particles enter a FLOW.
#[derive(Debug, Clone)]
pub enum FlowSource {
    /// From Enki EAV store (query by lane/tribe)
    Enki    { lane: Option<QualityLane>, tribe: Option<String> },
    /// From Vault file ingestion (ZIP/CSV/XLSX)
    Vault   { path: String },
    /// From another flow's output
    FlowRef { flow: String },
    /// Literal particle set (for testing)
    Literal { count: usize },
}

#[derive(Debug, Clone)]
pub enum FlowStage {
    Cleanse   { function: String },
    Compare   { function: String },
    ApplyRule { rule: String },
    Score,
    Gate      { min_health: f32 },
    Steward,
    Story     { label: String },
    Custom    { function: String },
}

/// Where particles exit a FLOW.
#[derive(Debug, Clone)]
pub enum FlowSink {
    /// Write to Enki EAV store (upsert by KAKI)
    Enki,
    /// Write to Vault persistence
    Vault   { label: String },
    FlowRef { flow: String },
    Discard,
}

// ── OBSERVE ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ObserveNode {
    pub id:      NodeId,
    pub name:    String,
    pub from:    ObserveFrom,
    pub filters: Vec<ObserveFilter>,
    pub sort:    Option<ObserveSort>,
    pub limit:   Option<usize>,
    pub doc:     Option<String>,
    pub span:    Span,
}

impl ObserveNode {
    pub fn new(name: &str, from: ObserveFrom, span: Span) -> Self {
        Self { id: NodeId::from_parts(&["OBSERVE", name]), name: name.to_string(),
               from, filters: vec![], sort: None, limit: None, doc: None, span }
    }
}

#[derive(Debug, Clone)]
pub enum ObserveFrom {
    Tribe      { name: String },
    Lane       { lane: QualityLane },
    DeadGrid   { tribe: Option<String> },
    Neighbors  { kaki: String, k: usize },
    KakiPrefix { prefix: String },
}

#[derive(Debug, Clone)]
pub struct ObserveFilter {
    pub property: ObserveProperty,
    pub op:       ConditionOp,
    pub value:    f32,
}

#[derive(Debug, Clone, Copy)]
pub enum ObserveProperty {
    Health,
    B11,
    OrbitalRadius,
    Dimension(HeptaDim),
    HealthDerivative,
}

#[derive(Debug, Clone)]
pub struct ObserveSort {
    pub property:   ObserveProperty,
    pub descending: bool,
}

// ── EMIT ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct EmitNode {
    pub id:       NodeId,
    pub name:     String,
    pub target:   EmitTarget,
    pub template: Option<String>,
    pub context:  Vec<EmitContext>,
    pub persist:  bool,
    pub doc:      Option<String>,
    pub span:     Span,
}

impl EmitNode {
    pub fn self_heal(name: &str, template: &str, span: Span) -> Self {
        Self { id: NodeId::from_parts(&["EMIT", name]), name: name.to_string(),
               target: EmitTarget::SelfHeal, template: Some(template.to_string()),
               context: vec![], persist: true, doc: None, span }
    }
}

#[derive(Debug, Clone)]
pub enum EmitTarget {
    SelfHeal,
    SelfAlert,
    NewEquation,
    UpdateRule { rule_name: String },
    StoryEvent,
    StorageLog,
    Webhook { url: String },
    Report  { format: String },
}

#[derive(Debug, Clone)]
pub struct EmitContext {
    pub key:   String,
    pub value: String,
}

// ── GUARD ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct GuardNode {
    pub id:        NodeId,
    pub name:      String,
    pub condition: RuleCondition,
    pub body:      Vec<AkkNode>,
    pub span:      Span,
}

// ── PIPELINE ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct PipelineNode {
    pub id:      NodeId,
    pub name:    String,
    pub nodes:   Vec<AkkNode>,
    pub version: String,
    pub doc:     Option<String>,
    pub span:    Span,
}

impl PipelineNode {
    pub fn new(name: &str, version: &str, span: Span) -> Self {
        Self { id: NodeId::from_parts(&["PIPELINE", name, version]), name: name.to_string(),
               nodes: vec![], version: version.to_string(), doc: None, span }
    }
    pub fn add(mut self, node: AkkNode) -> Self { self.nodes.push(node); self }
}
