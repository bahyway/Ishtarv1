//! EGDEngine — Electricity Grid Defect Engine.
//! Gibil Calculus on the fdd-core skeleton: residual sweep + Birqu
//! emission, Gibil horizon (via trend-core), physical asset registry,
//! and PROVE-before-PRESCRIBE exclusion simulation.

pub mod assets;
pub mod dispatch;
pub mod exclusion;
pub mod gibil;
pub mod horizon;
pub mod phasor;
pub mod state_estimate;

pub use fdd_core::{AlertSink, BaruResidual, Edge, FddError, Network, Node};
pub use gibil::Gibil;
pub use phasor::Phasor;

/// One Gibil sweep: residuals + Birqu emission beyond tolerance.
pub fn sweep(
    net: &Network<Phasor>,
    phi: &[Phasor],
    tol: f64,
    ts_millis: u64,
    sink: &mut impl AlertSink,
) -> Result<Vec<BaruResidual>, FddError> {
    let residuals = fdd_core::residuals(net, phi, &Gibil)?;
    Ok(residuals
        .into_iter()
        .enumerate()
        .map(|(i, kappa)| {
            let r = BaruResidual {
                node_id: net.nodes[i].id,
                kappa,
                ts_millis,
            };
            if kappa > tol {
                sink.emit(gibil::ALERT_CLASS, r.node_id, kappa, ts_millis);
            }
            r
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use fdd_core::Quantity;

    struct Collect(Vec<(u32, f64)>);
    impl AlertSink for Collect {
        fn emit(&mut self, _c: &str, n: u32, k: f64, _t: u64) {
            self.0.push((n, k));
        }
    }

    /// Two-bus line: source feeds a load through admittance Y.
    /// Consistent injections -> near-zero residuals -> no Birqu. Then
    /// steal current at the load (loss/fraud) -> Birqu.
    #[test]
    fn gibil_detects_missing_current() {
        let y = Phasor::new(10.0, -5.0);
        let v = [Phasor::new(1.0, 0.0), Phasor::new(0.95, -0.02)];
        let i_line = y.mul(v[0].sub(v[1]));

        let mut net = Network {
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

        let mut sink = Collect(vec![]);
        let r = sweep(&net, &v, 1e-9, 0, &mut sink).unwrap();
        assert!(r.iter().all(|b| b.kappa < 1e-9));
        assert!(sink.0.is_empty());

        // Defect: 10% of the current vanishes at the load.
        net.nodes[1].injection = i_line.mul(Phasor::new(0.9, 0.0));
        let mut sink2 = Collect(vec![]);
        sweep(&net, &v, 1e-9, 1, &mut sink2).unwrap();
        assert_eq!(sink2.0.len(), 1);
        assert_eq!(sink2.0[0].0, 1); // Birqu strikes node 1
    }

    /// Dynamic topology: open the breaker -> the island's residual
    /// equals its full injection (as physics says).
    #[test]
    fn breaker_state_is_a_state_variable() {
        let y = Phasor::new(10.0, -5.0);
        let v = [Phasor::new(1.0, 0.0), Phasor::new(0.95, -0.02)];
        let i_line = y.mul(v[0].sub(v[1]));
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
                in_service: false,
            }],
        };
        let mut sink = Collect(vec![]);
        let r = sweep(&net, &v, 1e-9, 0, &mut sink).unwrap();
        assert!(r[1].kappa > 0.0);
    }

    #[test]
    fn mismatched_phi_length_is_a_typed_error_not_a_panic() {
        let net = Network::<Phasor> {
            nodes: vec![
                Node {
                    id: 0,
                    injection: Phasor::zero(),
                },
                Node {
                    id: 1,
                    injection: Phasor::zero(),
                },
            ],
            edges: vec![],
        };
        let mut sink = Collect(vec![]);
        let err = sweep(&net, &[Phasor::zero()], 1e-9, 0, &mut sink).unwrap_err();
        assert_eq!(
            err,
            FddError::PotentialLengthMismatch {
                expected: 2,
                got: 1
            }
        );
    }
}
