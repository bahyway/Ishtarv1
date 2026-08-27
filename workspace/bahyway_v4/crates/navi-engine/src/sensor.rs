//! SensorFeed — real-time sovereign event updates to a live NaviGraph.
//!
//! No external network calls — all updates are local, applied in-process.
//! After each SensorEvent the graph is immediately consistent for the next
//! NaviCode pipeline run.

use crate::error::{NaviError, NaviResult};
use crate::graph::NaviGraph;
use crate::particle::{NaviNodeId, SurfaceQuality};
use bahyway_core::TribeId;

// ── SensorEvent ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum SensorEvent {
    /// Congestion level update at a node (0=clear, 255=gridlock).
    Congestion { node_id: NaviNodeId, level: u8 },
    /// Node is closed — set to Dead.
    RoadClosure { node_id: NaviNodeId },
    /// Node reopens — restore to Active.
    RoadOpen { node_id: NaviNodeId },
    /// Surface quality change (road repair, rain damage, etc.).
    WeatherChange {
        node_id: NaviNodeId,
        surface: SurfaceQuality,
    },
    /// A sovereign tribe alert — restrict all nodes belonging to this tribe.
    SovereignAlert { tribe: TribeId },
    /// Speed limit or flow-speed update at a node.
    SpeedChange { node_id: NaviNodeId, speed_kmh: u8 },
}

impl SensorEvent {
    pub fn name(&self) -> &'static str {
        match self {
            SensorEvent::Congestion { .. } => "Congestion",
            SensorEvent::RoadClosure { .. } => "RoadClosure",
            SensorEvent::RoadOpen { .. } => "RoadOpen",
            SensorEvent::WeatherChange { .. } => "WeatherChange",
            SensorEvent::SovereignAlert { .. } => "SovereignAlert",
            SensorEvent::SpeedChange { .. } => "SpeedChange",
        }
    }
}

// ── SensorFeed ────────────────────────────────────────────────────────────────

pub struct SensorFeed;

impl SensorFeed {
    /// Apply a SensorEvent to the live NaviGraph.
    /// Returns `NaviError::InvalidParticle` if the target node does not exist.
    pub fn apply(graph: &mut NaviGraph, event: SensorEvent) -> NaviResult<()> {
        match event {
            SensorEvent::Congestion { node_id, level } => {
                let n = graph.node_mut(node_id).ok_or_else(|| {
                    NaviError::InvalidParticle(format!("node {node_id} not found"))
                })?;
                n.particle.congestion = level;
            }

            SensorEvent::RoadClosure { node_id } => {
                let n = graph.node_mut(node_id).ok_or_else(|| {
                    NaviError::InvalidParticle(format!("node {node_id} not found"))
                })?;
                n.particle.kill();
            }

            SensorEvent::RoadOpen { node_id } => {
                use crate::particle::NaviParticleState;
                let n = graph.node_mut(node_id).ok_or_else(|| {
                    NaviError::InvalidParticle(format!("node {node_id} not found"))
                })?;
                n.particle.state = NaviParticleState::Active;
                n.particle.signal = crate::particle::NaviSignal::Strong;
            }

            SensorEvent::WeatherChange { node_id, surface } => {
                let n = graph.node_mut(node_id).ok_or_else(|| {
                    NaviError::InvalidParticle(format!("node {node_id} not found"))
                })?;
                n.particle.surface = surface;
            }

            SensorEvent::SovereignAlert { tribe } => {
                // Restrict all nodes that belong to this tribe.
                let affected: Vec<NaviNodeId> = graph
                    .all_node_ids()
                    .into_iter()
                    .filter(|&id| {
                        graph
                            .node(id)
                            .map(|n| n.particle.tribe == tribe)
                            .unwrap_or(false)
                    })
                    .collect();
                for id in affected {
                    if let Some(n) = graph.node_mut(id) {
                        n.particle.restrict();
                    }
                }
            }

            SensorEvent::SpeedChange { node_id, speed_kmh } => {
                let n = graph.node_mut(node_id).ok_or_else(|| {
                    NaviError::InvalidParticle(format!("node {node_id} not found"))
                })?;
                n.particle.speed_kmh = speed_kmh;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::NaviGraph;
    use crate::navimap::seven_node_map;

    fn graph() -> NaviGraph {
        NaviGraph::from_navimap(&seven_node_map()).unwrap()
    }

    #[test]
    fn congestion_event_updates_node() {
        let mut g = graph();
        SensorFeed::apply(
            &mut g,
            SensorEvent::Congestion {
                node_id: 2,
                level: 200,
            },
        )
        .unwrap();
        assert_eq!(g.node(2).unwrap().particle.congestion, 200);
    }

    #[test]
    fn road_closure_kills_node() {
        let mut g = graph();
        SensorFeed::apply(&mut g, SensorEvent::RoadClosure { node_id: 3 }).unwrap();
        assert!(!g.node(3).unwrap().is_passable());
    }

    #[test]
    fn road_open_restores_node() {
        let mut g = graph();
        SensorFeed::apply(&mut g, SensorEvent::RoadClosure { node_id: 3 }).unwrap();
        SensorFeed::apply(&mut g, SensorEvent::RoadOpen { node_id: 3 }).unwrap();
        assert!(g.node(3).unwrap().is_passable());
    }

    #[test]
    fn weather_change_updates_surface() {
        let mut g = graph();
        SensorFeed::apply(
            &mut g,
            SensorEvent::WeatherChange {
                node_id: 4,
                surface: SurfaceQuality::Poor,
            },
        )
        .unwrap();
        assert_eq!(g.node(4).unwrap().particle.surface, SurfaceQuality::Poor);
    }

    #[test]
    fn sovereign_alert_restricts_all_tribe_nodes() {
        let mut g = graph();
        // All seven_node_map nodes belong to tribe 0x0001
        let tribe = TribeId::from_u16(0x0001);
        SensorFeed::apply(&mut g, SensorEvent::SovereignAlert { tribe }).unwrap();
        for id in 1u32..=7 {
            assert!(
                !g.node(id).unwrap().is_passable(),
                "node {id} must be restricted"
            );
        }
    }

    #[test]
    fn sovereign_alert_different_tribe_leaves_nodes_active() {
        let mut g = graph();
        let other_tribe = TribeId::from_u16(0x0099);
        SensorFeed::apply(&mut g, SensorEvent::SovereignAlert { tribe: other_tribe }).unwrap();
        // All nodes have tribe 0x0001, not 0x0099 → unaffected
        assert!(g.node(1).unwrap().is_passable());
    }

    #[test]
    fn speed_change_updates_node() {
        let mut g = graph();
        SensorFeed::apply(
            &mut g,
            SensorEvent::SpeedChange {
                node_id: 5,
                speed_kmh: 30,
            },
        )
        .unwrap();
        assert_eq!(g.node(5).unwrap().particle.speed_kmh, 30);
    }

    #[test]
    fn event_on_nonexistent_node_errors() {
        let mut g = graph();
        let r = SensorFeed::apply(
            &mut g,
            SensorEvent::Congestion {
                node_id: 99,
                level: 50,
            },
        );
        assert!(r.is_err());
    }

    #[test]
    fn sensor_event_names_non_empty() {
        let events = [
            SensorEvent::Congestion {
                node_id: 1,
                level: 0,
            },
            SensorEvent::RoadClosure { node_id: 1 },
            SensorEvent::RoadOpen { node_id: 1 },
            SensorEvent::WeatherChange {
                node_id: 1,
                surface: SurfaceQuality::Good,
            },
            SensorEvent::SovereignAlert {
                tribe: TribeId::from_u16(0x0001),
            },
            SensorEvent::SpeedChange {
                node_id: 1,
                speed_kmh: 60,
            },
        ];
        for e in &events {
            assert!(!e.name().is_empty());
        }
    }

    #[test]
    fn double_closure_then_reopen_is_passable() {
        let mut g = graph();
        SensorFeed::apply(&mut g, SensorEvent::RoadClosure { node_id: 2 }).unwrap();
        SensorFeed::apply(&mut g, SensorEvent::RoadClosure { node_id: 2 }).unwrap();
        SensorFeed::apply(&mut g, SensorEvent::RoadOpen { node_id: 2 }).unwrap();
        assert!(g.node(2).unwrap().is_passable());
    }
}
