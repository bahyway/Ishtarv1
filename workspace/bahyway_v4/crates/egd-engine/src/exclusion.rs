//! Exclusion simulation: before prescribing that a part be taken out of
//! service, prove the remaining grid still serves all load.
//! in_service=false on a COPY; residual re-sweep; any node whose
//! residual jumps to its full injection magnitude is unserved ->
//! exclusion refused.

use crate::gibil::Gibil;
use crate::phasor::Phasor;
use fdd_core::{residuals, FddError, Network, Quantity};

#[derive(Debug, Clone, PartialEq)]
pub enum Verdict {
    /// Safe to exclude — prescription may be issued.
    Proven,
    /// These node ids would lose service.
    Refused(Vec<u32>),
}

pub fn simulate_exclusion(
    net: &Network<Phasor>,
    phi: &[Phasor],
    edge_idx: usize,
    tol: f64,
) -> Result<Verdict, FddError> {
    let mut copy = net.clone();
    copy.edges[edge_idx].in_service = false;
    let r = residuals(&copy, phi, &Gibil)?;
    let unserved: Vec<u32> = copy
        .nodes
        .iter()
        .enumerate()
        .filter(|(i, n)| {
            let inj = n.injection.magnitude();
            inj > tol && (r[*i] - inj).abs() < tol
        })
        .map(|(_, n)| n.id)
        .collect();
    Ok(if unserved.is_empty() { Verdict::Proven } else { Verdict::Refused(unserved) })
}

/// N-1 contingency analysis (enterprise APM territory, ETAP/PowerFactory):
/// sweep `simulate_exclusion` over EVERY in-service edge, one at a time,
/// to find which single failures would island load -- a near-free
/// generalization of the PROVE-before-PRESCRIBE simulator, since it's
/// the same function called once per edge instead of once on request.
#[derive(Debug, Clone, PartialEq)]
pub struct ContingencyResult {
    pub edge_idx: usize,
    pub verdict: Verdict,
}

pub fn n_minus_1_sweep(
    net: &Network<Phasor>,
    phi: &[Phasor],
    tol: f64,
) -> Result<Vec<ContingencyResult>, FddError> {
    net.edges
        .iter()
        .enumerate()
        .filter(|(_, e)| e.in_service)
        .map(|(i, _)| Ok(ContingencyResult { edge_idx: i, verdict: simulate_exclusion(net, phi, i, tol)? }))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Edge, Node};

    /// Three-bus radial feeder: 0 -> 1 -> 2. Losing edge (0,1) strands
    /// both downstream nodes; losing edge (1,2) strands only node 2.
    fn radial_feeder() -> (Network<Phasor>, Vec<Phasor>) {
        let y = Phasor::new(10.0, -5.0);
        let v = vec![Phasor::new(1.0, 0.0), Phasor::new(0.97, -0.01), Phasor::new(0.94, -0.02)];
        let i01 = y.mul(v[0].sub(v[1]));
        let i12 = y.mul(v[1].sub(v[2]));
        let net = Network {
            nodes: vec![
                Node { id: 0, injection: Phasor::zero().sub(i01) },
                Node { id: 1, injection: i01.sub(i12) },
                Node { id: 2, injection: i12 },
            ],
            edges: vec![
                Edge { from: 0, to: 1, coeff: y, in_service: true },
                Edge { from: 1, to: 2, coeff: y, in_service: true },
            ],
        };
        (net, v)
    }

    #[test]
    fn n_minus_1_sweep_covers_every_in_service_edge() {
        let (net, v) = radial_feeder();
        let results = n_minus_1_sweep(&net, &v, 1e-9).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].edge_idx, 0);
        assert_eq!(results[1].edge_idx, 1);
    }

    #[test]
    fn losing_the_upstream_edge_strands_more_customers() {
        let (net, v) = radial_feeder();
        let results = n_minus_1_sweep(&net, &v, 1e-9).unwrap();
        let upstream = match &results[0].verdict {
            Verdict::Refused(nodes) => nodes.len(),
            Verdict::Proven => 0,
        };
        let downstream = match &results[1].verdict {
            Verdict::Refused(nodes) => nodes.len(),
            Verdict::Proven => 0,
        };
        assert!(upstream >= downstream, "losing the feeder head should strand at least as many customers");
    }

    #[test]
    fn out_of_service_edges_are_skipped_not_resimulated() {
        let (mut net, v) = radial_feeder();
        net.edges[1].in_service = false;
        let results = n_minus_1_sweep(&net, &v, 1e-9).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].edge_idx, 0);
    }
}
