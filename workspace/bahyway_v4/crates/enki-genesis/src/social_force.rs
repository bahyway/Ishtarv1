#![forbid(unsafe_code)]
//! Social Force model — 2D indoor pedestrian dynamics.
//!
//! Each agent has a goal and is attracted toward it while repelling from
//! neighbours and walls.  Coordinates are in metres (f64).

/// Simulation parameters (GEN-PARAMS-001, canonical values).
#[derive(Clone, Debug)]
pub struct SocialForceParams {
    pub attraction_coeff: f64,     // 2.0 — goal attraction strength
    pub repulsion_coeff: f64,      // 5.0 — inter-agent repulsion
    pub wall_repulsion_coeff: f64, // 10.0 — wall repulsion
    pub neighbor_radius: f64,      // 2.0m — interaction radius
    pub wall_radius: f64,          // 1.0m — wall influence radius
    pub time_step: f64,            // 0.01s
    pub damping: f64,              // 0.95 — velocity damping per step
}

impl Default for SocialForceParams {
    fn default() -> Self {
        Self {
            attraction_coeff: 2.0,
            repulsion_coeff: 5.0,
            wall_repulsion_coeff: 10.0,
            neighbor_radius: 2.0,
            wall_radius: 1.0,
            time_step: 0.01,
            damping: 0.95,
        }
    }
}

/// A 2D pedestrian agent in the Social Force simulation.
#[derive(Clone, Debug)]
pub struct Agent2D {
    /// Current position [x, y] in metres.
    pub pos: [f64; 2],
    /// Current velocity [vx, vy] in m/s.
    pub vel: [f64; 2],
    /// Goal position [x, y] in metres.
    pub goal: [f64; 2],
}

impl Agent2D {
    pub fn new(pos: [f64; 2], goal: [f64; 2]) -> Self {
        Self {
            pos,
            vel: [0.0, 0.0],
            goal,
        }
    }
}

/// Advance all agents one time step using the Social Force model.
///
/// Wall is modelled as a rectangle [0, width] × [0, height].
pub fn step(agents: &mut [Agent2D], params: &SocialForceParams, room_width: f64, room_height: f64) {
    let n = agents.len();
    // Compute forces before applying (read positions, then write velocities)
    let mut forces: Vec<[f64; 2]> = vec![[0.0, 0.0]; n];

    for i in 0..n {
        let pos_i = agents[i].pos;
        let goal_i = agents[i].goal;

        // 1. Goal attraction force
        let dx_g = goal_i[0] - pos_i[0];
        let dy_g = goal_i[1] - pos_i[1];
        let d_g = (dx_g * dx_g + dy_g * dy_g).sqrt().max(1e-6);
        forces[i][0] += params.attraction_coeff * dx_g / d_g;
        forces[i][1] += params.attraction_coeff * dy_g / d_g;

        // 2. Inter-agent repulsion
        for (j, agent_j) in agents.iter().enumerate().take(n) {
            if i == j {
                continue;
            }
            let pos_j = agent_j.pos;
            let dx = pos_i[0] - pos_j[0];
            let dy = pos_i[1] - pos_j[1];
            let d = (dx * dx + dy * dy).sqrt().max(1e-6);
            if d < params.neighbor_radius {
                let f = params.repulsion_coeff * (params.neighbor_radius - d) / d;
                forces[i][0] += f * dx;
                forces[i][1] += f * dy;
            }
        }

        // 3. Wall repulsion (4 walls)
        // Left wall (x=0)
        if pos_i[0] < params.wall_radius {
            forces[i][0] += params.wall_repulsion_coeff * (params.wall_radius - pos_i[0]);
        }
        // Right wall (x=room_width)
        if pos_i[0] > room_width - params.wall_radius {
            forces[i][0] -=
                params.wall_repulsion_coeff * (params.wall_radius - (room_width - pos_i[0]));
        }
        // Bottom wall (y=0)
        if pos_i[1] < params.wall_radius {
            forces[i][1] += params.wall_repulsion_coeff * (params.wall_radius - pos_i[1]);
        }
        // Top wall (y=room_height)
        if pos_i[1] > room_height - params.wall_radius {
            forces[i][1] -=
                params.wall_repulsion_coeff * (params.wall_radius - (room_height - pos_i[1]));
        }
    }

    // Apply forces to velocities and positions
    for i in 0..n {
        agents[i].vel[0] = (agents[i].vel[0] + forces[i][0] * params.time_step) * params.damping;
        agents[i].vel[1] = (agents[i].vel[1] + forces[i][1] * params.time_step) * params.damping;
        agents[i].pos[0] += agents[i].vel[0] * params.time_step;
        agents[i].pos[1] += agents[i].vel[1] * params.time_step;
        // Clamp to room bounds
        agents[i].pos[0] = agents[i].pos[0].clamp(0.0, room_width);
        agents[i].pos[1] = agents[i].pos[1].clamp(0.0, room_height);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agents_move_toward_goal() {
        let mut agents = vec![Agent2D::new([0.5, 5.0], [9.5, 5.0])];
        let params = SocialForceParams::default();
        let x_start = agents[0].pos[0];
        for _ in 0..50 {
            step(&mut agents, &params, 10.0, 10.0);
        }
        assert!(
            agents[0].pos[0] > x_start,
            "agent should move toward goal (positive x)"
        );
    }

    #[test]
    fn agents_stay_in_bounds() {
        let mut agents = vec![
            Agent2D::new([0.1, 0.1], [9.9, 9.9]),
            Agent2D::new([9.9, 9.9], [0.1, 0.1]),
        ];
        let params = SocialForceParams::default();
        for _ in 0..1_000 {
            step(&mut agents, &params, 10.0, 10.0);
        }
        for a in &agents {
            assert!((0.0..=10.0).contains(&a.pos[0]));
            assert!((0.0..=10.0).contains(&a.pos[1]));
        }
    }
}
