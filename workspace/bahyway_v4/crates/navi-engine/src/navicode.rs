//! NaviCode — 6-stage sovereign routing command pipeline (NC1–NC6).
//!
//! Phase 1 (NC1–NC3): Graph preparation — lock origin, score by tribal resonance,
//!   tombstone dead edges. Produces the initial EdgeCostMatrix.
//! Phase 2 (NC4–NC6): Cost refinement and path finalization — friction, clustering,
//!   Dijkstra shortest path.
//!
//! The executor runs all 6 commands in strict NC1→NC6 order.
//! ExecutionContext threads mutable state through every stage with pure sequential ownership.

use crate::error::{NaviError, NaviResult};
use crate::graph::{NaviGraph, NaviNode};
use crate::particle::{NaviNodeId, NaviParticleState, NaviSignal};
use bahyway_core::TribeId;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};

// ── NaviCommand ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NaviCommand {
    SpawnOrigin,    // NC1 — lock origin; verify passable
    ResonanceSeek,  // NC2 — tribal resonance cost reduction toward destination
    RepelDead,      // NC3 — tombstone dead/jammed/restricted edges (∞ cost)
    FrictionAdjust, // NC4 — multiply costs by surface quality + speed
    TribeCluster,   // NC5 — group nodes by tribe for sector-jump fast paths
    GoldenPath,     // NC6 — Dijkstra on the EdgeCostMatrix; seal RoutePlan
}

impl NaviCommand {
    /// Fixed 6-stage pipeline — NC1 → NC6.
    pub fn pipeline() -> [NaviCommand; 6] {
        [
            NaviCommand::SpawnOrigin,
            NaviCommand::ResonanceSeek,
            NaviCommand::RepelDead,
            NaviCommand::FrictionAdjust,
            NaviCommand::TribeCluster,
            NaviCommand::GoldenPath,
        ]
    }

    pub fn phase(self) -> u8 {
        match self {
            NaviCommand::SpawnOrigin | NaviCommand::ResonanceSeek | NaviCommand::RepelDead => 1,
            NaviCommand::FrictionAdjust | NaviCommand::TribeCluster | NaviCommand::GoldenPath => 2,
        }
    }

    pub fn stage(self) -> u8 {
        match self {
            NaviCommand::SpawnOrigin => 1,
            NaviCommand::ResonanceSeek => 2,
            NaviCommand::RepelDead => 3,
            NaviCommand::FrictionAdjust => 4,
            NaviCommand::TribeCluster => 5,
            NaviCommand::GoldenPath => 6,
        }
    }
}

// ── RouteConstraints ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
pub struct RouteConstraints {
    /// Maximum acceptable risk score (0=permissive 255=strict safe).
    pub max_risk: u8,
    pub avoid_restricted: bool,
    pub prefer_quality: bool,
}

impl Default for RouteConstraints {
    fn default() -> Self {
        RouteConstraints {
            max_risk: 128,
            avoid_restricted: true,
            prefer_quality: true,
        }
    }
}

// ── RouteRequest ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct RouteRequest {
    pub origin: NaviNodeId,
    pub destination: NaviNodeId,
    pub constraints: RouteConstraints,
    pub dest_tribe: TribeId,
}

impl RouteRequest {
    pub fn new(origin: NaviNodeId, destination: NaviNodeId, dest_tribe: TribeId) -> Self {
        RouteRequest {
            origin,
            destination,
            constraints: RouteConstraints::default(),
            dest_tribe,
        }
    }
}

// ── EdgeCostMatrix ────────────────────────────────────────────────────────────

/// Mutable cost table threaded through NC1–NC5, consumed by NC6.
/// f32::INFINITY = tombstoned (dead/restricted edge — never traversed).
#[derive(Debug, Default)]
pub struct EdgeCostMatrix {
    costs: HashMap<(NaviNodeId, NaviNodeId), f32>,
}

impl EdgeCostMatrix {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, from: NaviNodeId, to: NaviNodeId, cost: f32) {
        self.costs.insert((from, to), cost);
    }

    pub fn get(&self, from: NaviNodeId, to: NaviNodeId) -> f32 {
        self.costs
            .get(&(from, to))
            .copied()
            .unwrap_or(f32::INFINITY)
    }

    pub fn tombstone(&mut self, from: NaviNodeId, to: NaviNodeId) {
        self.costs.insert((from, to), f32::INFINITY);
    }

    pub fn scale(&mut self, from: NaviNodeId, to: NaviNodeId, factor: f32) {
        let c = self.get(from, to);
        if c.is_finite() {
            self.set(from, to, c * factor);
        }
    }

    pub fn reduce(&mut self, from: NaviNodeId, to: NaviNodeId, reduction: f32) {
        let c = self.get(from, to);
        if c.is_finite() {
            self.set(from, to, (c - reduction).max(0.01));
        }
    }

    pub fn entry_count(&self) -> usize {
        self.costs.len()
    }
}

// ── TribeClusterMap ───────────────────────────────────────────────────────────

#[derive(Debug, Default)]
pub struct TribeClusterMap {
    clusters: HashMap<TribeId, Vec<NaviNodeId>>,
}

impl TribeClusterMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, tribe: TribeId, id: NaviNodeId) {
        self.clusters.entry(tribe).or_default().push(id);
    }

    pub fn nodes_for(&self, tribe: TribeId) -> &[NaviNodeId] {
        self.clusters.get(&tribe).map(Vec::as_slice).unwrap_or(&[])
    }

    pub fn tribe_count(&self) -> usize {
        self.clusters.len()
    }
}

// ── RoutePlan ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct RoutePlan {
    /// Ordered node IDs from origin to destination.
    pub waypoints: Vec<NaviNodeId>,
    pub total_cost: f32,
    /// How many distinct tribes the route crosses.
    pub tribe_hops: HashMap<TribeId, usize>,
    /// True when NC6 successfully sealed the path.
    pub is_golden: bool,
}

impl RoutePlan {
    pub fn waypoint_count(&self) -> usize {
        self.waypoints.len()
    }
    pub fn tribe_hop_count(&self) -> usize {
        self.tribe_hops.len()
    }
    pub fn is_valid(&self) -> bool {
        self.is_golden && self.waypoints.len() >= 2
    }
}

// ── ExecutionContext ──────────────────────────────────────────────────────────

pub struct ExecutionContext {
    pub request: RouteRequest,
    pub origin_locked: bool,
    pub matrix: EdgeCostMatrix,
    pub clusters: TribeClusterMap,
    pub plan: Option<RoutePlan>,
}

impl ExecutionContext {
    pub fn new(request: RouteRequest) -> Self {
        ExecutionContext {
            request,
            origin_locked: false,
            matrix: EdgeCostMatrix::new(),
            clusters: TribeClusterMap::new(),
            plan: None,
        }
    }
}

// ── NaviCodeExecutor ──────────────────────────────────────────────────────────

pub struct NaviCodeExecutor;

impl NaviCodeExecutor {
    /// Run the full NC1→NC6 pipeline and return the sealed RoutePlan.
    pub fn run_pipeline(request: RouteRequest, graph: &NaviGraph) -> NaviResult<RoutePlan> {
        if graph.node_count() == 0 {
            return Err(NaviError::EmptyGraph);
        }
        let mut ctx = ExecutionContext::new(request);
        for cmd in NaviCommand::pipeline() {
            Self::execute(&mut ctx, cmd, graph)?;
        }
        ctx.plan.ok_or(NaviError::RouteNotFound)
    }

    pub fn execute(
        ctx: &mut ExecutionContext,
        cmd: NaviCommand,
        graph: &NaviGraph,
    ) -> NaviResult<()> {
        match cmd {
            NaviCommand::SpawnOrigin => nc1_spawn_origin(ctx, graph),
            NaviCommand::ResonanceSeek => nc2_resonance_seek(ctx, graph),
            NaviCommand::RepelDead => nc3_repel_dead(ctx, graph),
            NaviCommand::FrictionAdjust => nc4_friction_adjust(ctx, graph),
            NaviCommand::TribeCluster => nc5_tribe_cluster(ctx, graph),
            NaviCommand::GoldenPath => nc6_golden_path(ctx, graph),
        }
    }
}

// ── NC1: SpawnOrigin ──────────────────────────────────────────────────────────

fn nc1_spawn_origin(ctx: &mut ExecutionContext, graph: &NaviGraph) -> NaviResult<()> {
    let origin = ctx.request.origin;
    let node = graph.node(origin).ok_or(NaviError::MapNotLoaded)?;
    if !node.is_passable() {
        return Err(NaviError::InvalidParticle(format!(
            "origin node {origin} is not passable"
        )));
    }
    // Seed the EdgeCostMatrix with effective costs from the graph
    for edge in graph.edges() {
        ctx.matrix.set(edge.from, edge.to, edge.effective_cost());
    }
    ctx.origin_locked = true;
    Ok(())
}

// ── NC2: ResonanceSeek ────────────────────────────────────────────────────────

fn nc2_resonance_seek(ctx: &mut ExecutionContext, graph: &NaviGraph) -> NaviResult<()> {
    let dest_tribe = ctx.request.dest_tribe;
    for edge in graph.edges() {
        if let Some(node) = graph.node(edge.to) {
            let bonus = resonance_bonus(node, dest_tribe);
            if bonus > 0.0 {
                ctx.matrix.reduce(edge.from, edge.to, bonus);
            }
        }
    }
    Ok(())
}

fn resonance_bonus(node: &NaviNode, dest_tribe: TribeId) -> f32 {
    if node.particle.tribe == dest_tribe {
        0.30
    } else {
        0.0
    }
}

// ── NC3: RepelDead ────────────────────────────────────────────────────────────

fn nc3_repel_dead(ctx: &mut ExecutionContext, graph: &NaviGraph) -> NaviResult<()> {
    for edge in graph.edges() {
        if let Some(node) = graph.node(edge.to) {
            let p = &node.particle;
            let dead = matches!(
                p.state,
                NaviParticleState::Dead | NaviParticleState::Restricted
            ) || matches!(p.signal, NaviSignal::Jammed);
            if dead {
                ctx.matrix.tombstone(edge.from, edge.to);
            }
        }
    }
    Ok(())
}

// ── NC4: FrictionAdjust ───────────────────────────────────────────────────────

fn nc4_friction_adjust(ctx: &mut ExecutionContext, graph: &NaviGraph) -> NaviResult<()> {
    for edge in graph.edges() {
        if let Some(node) = graph.node(edge.to) {
            let surface_factor = node.particle.surface.cost_multiplier();
            let speed_factor = if node.particle.speed_kmh == 0 {
                2.0 // stopped → double cost
            } else {
                60.0_f32 / node.particle.speed_kmh as f32
            };
            ctx.matrix
                .scale(edge.from, edge.to, surface_factor * speed_factor);
        }
    }
    Ok(())
}

// ── NC5: TribeCluster ─────────────────────────────────────────────────────────

fn nc5_tribe_cluster(ctx: &mut ExecutionContext, graph: &NaviGraph) -> NaviResult<()> {
    let mut ids = graph.all_node_ids();
    ids.sort_unstable();
    for id in ids {
        if let Some(node) = graph.node(id) {
            ctx.clusters.add(node.particle.tribe, id);
        }
    }
    Ok(())
}

// ── NC6: GoldenPath (Dijkstra) ────────────────────────────────────────────────

fn nc6_golden_path(ctx: &mut ExecutionContext, graph: &NaviGraph) -> NaviResult<()> {
    let origin = ctx.request.origin;
    let dest = ctx.request.destination;

    // dist[(node)] = best known cost from origin
    let mut dist: HashMap<NaviNodeId, f32> = HashMap::new();
    let mut prev: HashMap<NaviNodeId, NaviNodeId> = HashMap::new();
    // BinaryHeap is a max-heap; Reverse makes it a min-heap
    let mut heap: BinaryHeap<Reverse<(u32, NaviNodeId)>> = BinaryHeap::new();

    dist.insert(origin, 0.0);
    heap.push(Reverse((0, origin)));

    while let Some(Reverse((cost_u32, u))) = heap.pop() {
        let cost = cost_u32 as f32 / 1000.0;
        if u == dest {
            break;
        }
        if cost > *dist.get(&u).unwrap_or(&f32::INFINITY) {
            continue;
        }

        for &(v, _) in graph.neighbours(u) {
            let edge_cost = ctx.matrix.get(u, v);
            if !edge_cost.is_finite() {
                continue;
            }
            let new_cost = cost + edge_cost;
            if new_cost < *dist.get(&v).unwrap_or(&f32::INFINITY) {
                dist.insert(v, new_cost);
                prev.insert(v, u);
                heap.push(Reverse(((new_cost * 1000.0) as u32, v)));
            }
        }
    }

    // Reconstruct path
    if !dist.contains_key(&dest) {
        return Err(NaviError::RouteNotFound);
    }
    let mut path = Vec::new();
    let mut cur = dest;
    while cur != origin {
        path.push(cur);
        cur = *prev.get(&cur).ok_or(NaviError::RouteNotFound)?;
    }
    path.push(origin);
    path.reverse();

    // Build tribe hops
    let mut tribe_hops: HashMap<TribeId, usize> = HashMap::new();
    for &wp in &path {
        if let Some(n) = graph.node(wp) {
            *tribe_hops.entry(n.particle.tribe).or_insert(0) += 1;
        }
    }

    let total_cost = *dist.get(&dest).unwrap_or(&0.0);
    ctx.plan = Some(RoutePlan {
        waypoints: path,
        total_cost,
        tribe_hops,
        is_golden: true,
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::NaviGraph;
    use crate::navimap::seven_node_map;

    fn graph() -> NaviGraph {
        NaviGraph::from_navimap(&seven_node_map()).unwrap()
    }
    fn tribe() -> TribeId {
        TribeId::from_u16(0x0001)
    }

    // Pipeline structure
    #[test]
    fn pipeline_has_6_stages() {
        assert_eq!(NaviCommand::pipeline().len(), 6);
    }
    #[test]
    fn pipeline_starts_with_spawn() {
        assert_eq!(NaviCommand::pipeline()[0], NaviCommand::SpawnOrigin);
    }
    #[test]
    fn pipeline_ends_with_golden() {
        assert_eq!(NaviCommand::pipeline()[5], NaviCommand::GoldenPath);
    }
    #[test]
    fn phase_1_commands() {
        assert_eq!(NaviCommand::SpawnOrigin.phase(), 1);
        assert_eq!(NaviCommand::ResonanceSeek.phase(), 1);
        assert_eq!(NaviCommand::RepelDead.phase(), 1);
    }
    #[test]
    fn phase_2_commands() {
        assert_eq!(NaviCommand::FrictionAdjust.phase(), 2);
        assert_eq!(NaviCommand::TribeCluster.phase(), 2);
        assert_eq!(NaviCommand::GoldenPath.phase(), 2);
    }
    #[test]
    fn stage_numbers_correct() {
        let pipeline = NaviCommand::pipeline();
        for (i, cmd) in pipeline.iter().enumerate() {
            assert_eq!(cmd.stage(), (i + 1) as u8);
        }
    }

    // RouteConstraints
    #[test]
    fn default_constraints() {
        let c = RouteConstraints::default();
        assert!(c.avoid_restricted);
        assert!(c.prefer_quality);
        assert_eq!(c.max_risk, 128);
    }

    // EdgeCostMatrix
    #[test]
    fn matrix_set_and_get() {
        let mut m = EdgeCostMatrix::new();
        m.set(1, 2, 3.5);
        assert!((m.get(1, 2) - 3.5).abs() < 0.001);
    }
    #[test]
    fn matrix_tombstone_is_infinity() {
        let mut m = EdgeCostMatrix::new();
        m.set(1, 2, 3.5);
        m.tombstone(1, 2);
        assert!(!m.get(1, 2).is_finite());
    }
    #[test]
    fn matrix_scale_applies() {
        let mut m = EdgeCostMatrix::new();
        m.set(1, 2, 4.0);
        m.scale(1, 2, 2.0);
        assert!((m.get(1, 2) - 8.0).abs() < 0.001);
    }
    #[test]
    fn matrix_reduce_applies() {
        let mut m = EdgeCostMatrix::new();
        m.set(1, 2, 4.0);
        m.reduce(1, 2, 1.0);
        assert!((m.get(1, 2) - 3.0).abs() < 0.001);
    }
    #[test]
    fn matrix_missing_key_is_infinity() {
        let m = EdgeCostMatrix::new();
        assert!(!m.get(99, 100).is_finite());
    }
    #[test]
    fn matrix_tombstone_does_not_scale() {
        let mut m = EdgeCostMatrix::new();
        m.tombstone(1, 2);
        m.scale(1, 2, 0.5);
        assert!(!m.get(1, 2).is_finite());
    }

    // TribeClusterMap
    #[test]
    fn cluster_add_and_retrieve() {
        let mut cm = TribeClusterMap::new();
        cm.add(tribe(), 1);
        cm.add(tribe(), 2);
        assert_eq!(cm.nodes_for(tribe()).len(), 2);
    }
    #[test]
    fn cluster_missing_tribe_is_empty() {
        let cm = TribeClusterMap::new();
        assert!(cm.nodes_for(tribe()).is_empty());
    }
    #[test]
    fn cluster_tribe_count() {
        let mut cm = TribeClusterMap::new();
        cm.add(TribeId::from_u16(0x0001), 1);
        cm.add(TribeId::from_u16(0x0002), 2);
        assert_eq!(cm.tribe_count(), 2);
    }

    // RoutePlan
    #[test]
    fn route_plan_valid_when_golden_and_2_waypoints() {
        let plan = RoutePlan {
            waypoints: vec![1, 2],
            total_cost: 3.0,
            tribe_hops: HashMap::new(),
            is_golden: true,
        };
        assert!(plan.is_valid());
    }
    #[test]
    fn route_plan_invalid_if_not_golden() {
        let plan = RoutePlan {
            waypoints: vec![1, 2],
            total_cost: 3.0,
            tribe_hops: HashMap::new(),
            is_golden: false,
        };
        assert!(!plan.is_valid());
    }

    // Full pipeline
    #[test]
    fn pipeline_routes_centre_to_north() {
        let g = graph();
        let req = RouteRequest::new(1, 2, tribe()); // Centre → North
        let plan = NaviCodeExecutor::run_pipeline(req, &g).expect("route failed");
        assert!(plan.is_valid());
        assert_eq!(*plan.waypoints.first().unwrap(), 1);
        assert_eq!(*plan.waypoints.last().unwrap(), 2);
    }

    #[test]
    fn pipeline_routes_centre_to_south() {
        let g = graph();
        let req = RouteRequest::new(1, 5, tribe()); // Centre → South
        let plan = NaviCodeExecutor::run_pipeline(req, &g).expect("route failed");
        assert!(plan.is_valid());
    }

    #[test]
    fn pipeline_nonexistent_destination_errors() {
        let g = graph();
        let req = RouteRequest::new(1, 99, tribe());
        assert!(NaviCodeExecutor::run_pipeline(req, &g).is_err());
    }

    #[test]
    fn pipeline_empty_graph_errors() {
        let g = NaviGraph::new();
        let req = RouteRequest::new(1, 2, tribe());
        assert!(NaviCodeExecutor::run_pipeline(req, &g).is_err());
    }
}
