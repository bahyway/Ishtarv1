//! First-slice state estimator: EXACT linear solve, not weighted least
//! squares. Every other function in this crate takes `phi` (nodal
//! voltage phasors) as a caller-supplied input -- nothing computes it
//! from topology and measurements. That's the gap this module fills,
//! for the case it's honestly scoped to.
//!
//! Gibil's KCL model is linear in V (`I = Y * (Vf - Vt)`), so unlike
//! full nonlinear AC power flow, solving for unknown nodal voltages
//! from known ones is standard linear nodal analysis: build the bus
//! admittance matrix Y from the network topology, then solve
//! `Y * V = I_inject` for the unknown entries of V by Gaussian
//! elimination. `fdd_core::Node::injection` is a required field (not
//! `Option<Q>`), so every node not given a known voltage is treated as
//! a known-injection node by construction -- the system is always
//! exactly determined (as many equations as unknowns), never
//! over/under-determined.
//!
//! HONEST SCOPE, matching docs/07_file_formats/GL-EGD-001.md: this is NOT the
//! weighted-least-squares, bad-data-rejecting state estimator real
//! SCADA/EMS systems run against REDUNDANT telemetry (more
//! measurements than unknowns, with King-plot-style systematic-vs-
//! anomalous separation to reject a broken meter). That estimator is
//! real, separate, harder future work (enrichment #3). This module
//! only solves the fully-determined case: exactly one known value
//! (voltage or injection) per node.

use crate::phasor::Phasor;
use fdd_core::{Network, Quantity};

#[derive(Debug, Clone, PartialEq)]
pub enum StateEstimateError {
    /// The reduced admittance matrix was singular -- e.g. some unknown
    /// nodes are disconnected from every known-voltage node (no
    /// reference/slack in that component), so their voltage is
    /// undetermined by the given topology and measurements.
    SingularSystem,
}

/// Builds the n x n bus admittance matrix from the network's
/// in-service edges: for each edge (i, j, y), Y[i][i] += y,
/// Y[j][j] += y, Y[i][j] -= y, Y[j][i] -= y.
fn bus_admittance_matrix(net: &Network<Phasor>) -> Vec<Vec<Phasor>> {
    let n = net.nodes.len();
    let mut y = vec![vec![Phasor::zero(); n]; n];
    for e in net.edges.iter().filter(|e| e.in_service) {
        let (i, j) = (e.from as usize, e.to as usize);
        y[i][i] = y[i][i].add(e.coeff);
        y[j][j] = y[j][j].add(e.coeff);
        y[i][j] = y[i][j].sub(e.coeff);
        y[j][i] = y[j][i].sub(e.coeff);
    }
    y
}

/// Solves for every node's voltage phasor, given known (measured)
/// voltages at `known_voltage` node indices and known injections
/// (`net.nodes[i].injection`) at every other node.
pub fn solve_voltages(
    net: &Network<Phasor>,
    known_voltage: &[(usize, Phasor)],
) -> Result<Vec<Phasor>, StateEstimateError> {
    let n = net.nodes.len();
    let y = bus_admittance_matrix(net);

    let mut is_known = vec![false; n];
    let mut v = vec![Phasor::zero(); n];
    for &(idx, val) in known_voltage {
        is_known[idx] = true;
        v[idx] = val;
    }

    let unknown_idx: Vec<usize> = (0..n).filter(|&i| !is_known[i]).collect();
    let m = unknown_idx.len();
    if m == 0 {
        return Ok(v);
    }

    // Reduced system: fdd_core::residuals accumulates
    // acc[i] = -(Y_bus * V)[i] (edge flow leaving `from` is
    // subtracted, arriving at `to` is added -- the opposite sign from
    // the standard Y_bus*V convention), and the true state satisfies
    // acc[i] = injection[i]. So the system to solve is
    // Y_bus * V = -injection, i.e. for each unknown row i:
    //   sum_{j unknown} Y[i][j] * V[j] = -injection[i] - sum_{j known} Y[i][j] * V[j]
    let mut a = vec![vec![Phasor::zero(); m]; m];
    let mut b = vec![Phasor::zero(); m];

    for (row, &i) in unknown_idx.iter().enumerate() {
        let mut rhs = Phasor::zero().sub(net.nodes[i].injection);
        for j in 0..n {
            if is_known[j] {
                rhs = rhs.sub(y[i][j].mul(v[j]));
            }
        }
        b[row] = rhs;
        for (col, &j) in unknown_idx.iter().enumerate() {
            a[row][col] = y[i][j];
        }
    }

    let solved = gauss_solve(&mut a, &mut b)?;
    for (row, &i) in unknown_idx.iter().enumerate() {
        v[i] = solved[row];
    }
    Ok(v)
}

/// Gaussian elimination with partial pivoting (by magnitude) over
/// Phasor entries. Consumes `a`/`b` as scratch space; returns the
/// solution vector in place of `b`.
fn gauss_solve(a: &mut [Vec<Phasor>], b: &mut [Phasor]) -> Result<Vec<Phasor>, StateEstimateError> {
    let n = b.len();
    for col in 0..n {
        let pivot_row = (col..n)
            .max_by(|&r1, &r2| {
                a[r1][col]
                    .magnitude()
                    .partial_cmp(&a[r2][col].magnitude())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap();
        if a[pivot_row][col].magnitude() < 1e-9 {
            return Err(StateEstimateError::SingularSystem);
        }
        a.swap(col, pivot_row);
        b.swap(col, pivot_row);

        let pivot = a[col][col];
        for cell in &mut a[col][col..n] {
            *cell = cell.div(pivot);
        }
        b[col] = b[col].div(pivot);

        for row in 0..n {
            if row == col {
                continue;
            }
            let factor = a[row][col];
            if factor.magnitude() < 1e-15 {
                continue;
            }
            // k reads a[col][k] (the pivot row) while writing a[row][k] (a
            // different row) -- same class as riemannian.rs's tensor-
            // contraction loops fixed earlier: two different rows of the
            // same Vec<Vec<_>> can't both be borrowed through one safe
            // iterator, so the explicit index is kept.
            #[allow(clippy::needless_range_loop)]
            for k in col..n {
                a[row][k] = a[row][k].sub(factor.mul(a[col][k]));
            }
            b[row] = b[row].sub(factor.mul(b[col]));
        }
    }
    Ok(b.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gibil::Gibil;
    use crate::{Edge, Node};
    use fdd_core::residuals;

    #[test]
    fn two_bus_line_recovers_the_known_downstream_voltage() {
        // Ground truth: pick V directly, derive the line current and
        // the resulting injections -- exactly the fixture the other
        // egd-engine tests already use.
        let y = Phasor::new(10.0, -5.0);
        let v_true = [Phasor::new(1.0, 0.0), Phasor::new(0.95, -0.02)];
        let i_line = y.mul(v_true[0].sub(v_true[1]));

        let net = Network {
            nodes: vec![
                Node {
                    id: 0,
                    injection: Phasor::zero().sub(i_line),
                },
                Node {
                    id: 1,
                    injection: i_line,
                },
            ],
            edges: vec![Edge {
                from: 0,
                to: 1,
                coeff: y,
                in_service: true,
            }],
        };

        // Only node 0's voltage is "measured"; node 1's is unknown.
        let solved = solve_voltages(&net, &[(0, v_true[0])]).unwrap();
        assert!((solved[1].re - v_true[1].re).abs() < 1e-9);
        assert!((solved[1].im - v_true[1].im).abs() < 1e-9);
    }

    #[test]
    fn three_bus_radial_feeder_round_trips_through_the_residual_sweep() {
        // A realistic shape: only the feeder head (node 0) is
        // measured; nodes 1 and 2's voltages are unknown and must be
        // estimated from topology + known injections. If the solve is
        // correct, feeding the ESTIMATED voltages back into the
        // already-tested KCL residual sweep must show zero residual
        // everywhere -- self-consistency, not a hand-derived answer.
        let y = Phasor::new(8.0, -3.0);
        let v_true = vec![
            Phasor::new(1.0, 0.0),
            Phasor::new(0.97, -0.01),
            Phasor::new(0.94, -0.02),
        ];
        let i01 = y.mul(v_true[0].sub(v_true[1]));
        let i12 = y.mul(v_true[1].sub(v_true[2]));

        let net = Network {
            nodes: vec![
                Node {
                    id: 0,
                    injection: Phasor::zero().sub(i01),
                },
                Node {
                    id: 1,
                    injection: i01.sub(i12),
                },
                Node {
                    id: 2,
                    injection: i12,
                },
            ],
            edges: vec![
                Edge {
                    from: 0,
                    to: 1,
                    coeff: y,
                    in_service: true,
                },
                Edge {
                    from: 1,
                    to: 2,
                    coeff: y,
                    in_service: true,
                },
            ],
        };

        let estimated = solve_voltages(&net, &[(0, v_true[0])]).unwrap();
        let r = residuals(&net, &estimated, &Gibil).unwrap();
        assert!(
            r.iter().all(|kappa| *kappa < 1e-6),
            "residuals should vanish for a correctly estimated state: {r:?}"
        );
    }

    #[test]
    fn disconnected_unmeasured_component_is_a_singular_system() {
        // Node 1 has no edge to node 0 (the only measured node) -- its
        // voltage is genuinely undetermined by this topology.
        let y = Phasor::new(5.0, -1.0);
        let net = Network {
            nodes: vec![
                Node {
                    id: 0,
                    injection: Phasor::zero(),
                },
                Node {
                    id: 1,
                    injection: Phasor::new(1.0, 0.0),
                },
            ],
            edges: vec![Edge {
                from: 0,
                to: 0,
                coeff: y,
                in_service: false,
            }],
        };
        let err = solve_voltages(&net, &[(0, Phasor::new(1.0, 0.0))]).unwrap_err();
        assert_eq!(err, StateEstimateError::SingularSystem);
    }

    #[test]
    fn every_voltage_known_returns_immediately() {
        let net = Network::<Phasor> {
            nodes: vec![Node {
                id: 0,
                injection: Phasor::zero(),
            }],
            edges: vec![],
        };
        let v = solve_voltages(&net, &[(0, Phasor::new(1.0, 0.0))]).unwrap();
        assert_eq!(v, vec![Phasor::new(1.0, 0.0)]);
    }
}
