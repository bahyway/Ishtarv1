//! fdd-core — the domain-neutral flow-defect skeleton.
//! DETECT -> PROVE -> PREDICT -> PRESCRIBE, with the physics injected
//! by each domain calculus (Gibil for electricity, ...).
//!
//! HONEST SOURCE NOTE: this is new machinery, not an extraction from
//! WPDEngine. The real `wpd-engine` crate is a spectral/remote-sensing
//! classifier (BaghdadSector heptagram + 12-band VNIR/SWIR/TIR
//! signature matching), and the real Enbilulu Calculus
//! (`bahyway_algebra::enbilulu`) is a 5-factor weighted score feeding
//! TIAMAT bands -- neither is a network-potential/conservation-residual
//! model. `fdd-core` is a fresh abstraction for domains where the
//! physics genuinely IS a conservation law solved over a graph
//! (Kirchhoff's current law for a grid); it does not touch, and is not
//! derived from, WPDEngine's sealed source. See SPEC-FDD-001.md.

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FddError {
    /// `phi.len()` must equal `net.nodes.len()`.
    PotentialLengthMismatch { expected: usize, got: usize },
    /// An edge referenced a node index outside `net.nodes`.
    NodeIndexOutOfRange { edge_index: usize, node_index: u32 },
}

/// Domain scalar: water uses f64, electricity a complex phasor.
/// Anything with a magnitude can be a residual.
pub trait Quantity: Copy + Default {
    fn zero() -> Self;
    fn add(self, o: Self) -> Self;
    fn sub(self, o: Self) -> Self;
    fn magnitude(self) -> f64;
}

impl Quantity for f64 {
    fn zero() -> Self {
        0.0
    }
    fn add(self, o: Self) -> Self {
        self + o
    }
    fn sub(self, o: Self) -> Self {
        self - o
    }
    fn magnitude(self) -> f64 {
        self.abs()
    }
}

#[derive(Clone)]
pub struct Node<Q: Quantity> {
    pub id: u32,
    /// External injection (demand/generation): water draw or injected current.
    pub injection: Q,
}

#[derive(Clone)]
pub struct Edge<Q: Quantity> {
    pub from: u32,
    pub to: u32,
    /// Transfer coefficient: hydraulic conductance or complex admittance.
    /// Interpreted by the domain.
    pub coeff: Q,
    /// DYNAMIC TOPOLOGY: breaker/valve state is a state variable.
    /// Out-of-service edges carry nothing.
    pub in_service: bool,
}

#[derive(Clone, Default)]
pub struct Network<Q: Quantity> {
    pub nodes: Vec<Node<Q>>,
    pub edges: Vec<Edge<Q>>,
}

impl<Q: Quantity> Network<Q> {
    fn validate(&self, phi: &[Q]) -> Result<(), FddError> {
        if phi.len() != self.nodes.len() {
            return Err(FddError::PotentialLengthMismatch { expected: self.nodes.len(), got: phi.len() });
        }
        for (i, e) in self.edges.iter().enumerate() {
            if e.from as usize >= self.nodes.len() {
                return Err(FddError::NodeIndexOutOfRange { edge_index: i, node_index: e.from });
            }
            if e.to as usize >= self.nodes.len() {
                return Err(FddError::NodeIndexOutOfRange { edge_index: i, node_index: e.to });
            }
        }
        Ok(())
    }
}

/// The domain calculus: given the potential field, compute edge flow.
/// Water: head-loss law. Electricity: Ohm/KCL.
pub trait Potential<Q: Quantity> {
    fn edge_flow(&self, edge: &Edge<Q>, phi_from: Q, phi_to: Q) -> Q;
}

/// Conservation residual at every node: sum of flows minus injection.
/// |residual| beyond tolerance = candidate defect.
pub fn residuals<Q: Quantity, P: Potential<Q>>(
    net: &Network<Q>,
    phi: &[Q],
    calculus: &P,
) -> Result<Vec<f64>, FddError> {
    net.validate(phi)?;
    let mut acc: Vec<Q> = vec![Q::zero(); net.nodes.len()];
    for e in net.edges.iter().filter(|e| e.in_service) {
        let f = calculus.edge_flow(e, phi[e.from as usize], phi[e.to as usize]);
        acc[e.from as usize] = acc[e.from as usize].sub(f);
        acc[e.to as usize] = acc[e.to as usize].add(f);
    }
    Ok(net
        .nodes
        .iter()
        .enumerate()
        .map(|(i, n)| acc[i].sub(n.injection).magnitude())
        .collect())
}

/// Baru record: one node's deviation, ready for downstream
/// systematic-vs-anomalous separation.
#[derive(Clone, Copy, Debug)]
pub struct BaruResidual {
    pub node_id: u32,
    pub kappa: f64,
    pub ts_millis: u64,
}

/// Alert emission — domain names its class (Milu, Birqu).
pub trait AlertSink {
    fn emit(&mut self, class: &str, node_id: u32, kappa: f64, ts_millis: u64);
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Identity;
    impl Potential<f64> for Identity {
        fn edge_flow(&self, edge: &Edge<f64>, phi_from: f64, phi_to: f64) -> f64 {
            edge.coeff * (phi_from - phi_to)
        }
    }

    #[test]
    fn mismatched_potential_length_is_a_typed_error() {
        let net = Network::<f64> {
            nodes: vec![Node { id: 0, injection: 0.0 }, Node { id: 1, injection: 0.0 }],
            edges: vec![],
        };
        let err = residuals(&net, &[1.0], &Identity).unwrap_err();
        assert_eq!(err, FddError::PotentialLengthMismatch { expected: 2, got: 1 });
    }

    #[test]
    fn out_of_range_edge_is_a_typed_error() {
        let net = Network::<f64> {
            nodes: vec![Node { id: 0, injection: 0.0 }],
            edges: vec![Edge { from: 0, to: 5, coeff: 1.0, in_service: true }],
        };
        let err = residuals(&net, &[0.0], &Identity).unwrap_err();
        assert_eq!(err, FddError::NodeIndexOutOfRange { edge_index: 0, node_index: 5 });
    }

    #[test]
    fn balanced_network_has_near_zero_residual() {
        let net = Network::<f64> {
            nodes: vec![Node { id: 0, injection: -1.0 }, Node { id: 1, injection: 1.0 }],
            edges: vec![Edge { from: 0, to: 1, coeff: 1.0, in_service: true }],
        };
        let r = residuals(&net, &[1.0, 0.0], &Identity).unwrap();
        assert!(r.iter().all(|k| *k < 1e-9));
    }
}
