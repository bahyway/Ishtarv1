//! shape-pattern-court — GL-DST-004 (The Shape & Pattern Court): the Bell
//! Veto. While MARKASU or RIGMU is active, or the nabalkutu register is
//! non-empty, approval and release are PROHIBITED, regardless of steward
//! signature. PB-334. Pure Rust, zero dependencies.
#![forbid(unsafe_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BettiSignature {
    pub b0: u32,
    pub b1: u32,
}

/// Curve Loss (topology harm): any decrease in b0 or b1 counts as loss
/// magnitude; a gain is never counted as negative loss.
pub fn betti_diff(before: BettiSignature, after: BettiSignature) -> i64 {
    let b0_loss = (before.b0 as i64 - after.b0 as i64).max(0);
    let b1_loss = (before.b1 as i64 - after.b1 as i64).max(0);
    b0_loss + b1_loss
}

/// Census Loss (totality harm): particles overlooked.
pub fn census_diff(before_count: usize, after_count: usize) -> i64 {
    (before_count as i64 - after_count as i64).max(0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BellState {
    Quiet,
    MarkasuActive,
    RigmuActive,
}

/// The nabalkutu register (§4): particles whose state or position flaps.
/// A shape with a non-empty register is *sub judice* -- still before the
/// court.
#[derive(Debug, Default)]
pub struct NabalkutuRegister {
    flapping: Vec<u64>,
}

impl NabalkutuRegister {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.flapping.is_empty()
    }

    pub fn register(&mut self, id: u64) {
        if !self.flapping.contains(&id) {
            self.flapping.push(id);
        }
    }

    pub fn clear(&mut self, id: u64) {
        self.flapping.retain(|&x| x != id);
    }
}

#[derive(Debug, Clone)]
pub struct LossExplanation {
    pub w5h2_text: String,
}

#[derive(Debug, Clone, Copy)]
pub struct ApprovalRequest<'a> {
    pub betti_before: BettiSignature,
    pub betti_after: BettiSignature,
    pub census_before: usize,
    pub census_after: usize,
    pub explanation: Option<&'a LossExplanation>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalVerdict {
    Approved,
    VetoedBell,
    VetoedUnexplainedLoss,
}

/// THE BELL VETO (§5) — the single path to approval. Not warned,
/// disabled: this function never returns a partial or overridable
/// "approved with warning" state.
pub fn evaluate_approval(
    bell: BellState,
    nabalkutu: &NabalkutuRegister,
    request: &ApprovalRequest,
) -> ApprovalVerdict {
    if bell != BellState::Quiet || !nabalkutu.is_empty() {
        return ApprovalVerdict::VetoedBell;
    }
    let curve_loss = betti_diff(request.betti_before, request.betti_after);
    let census_loss = census_diff(request.census_before, request.census_after);
    if (curve_loss > 0 || census_loss > 0) && request.explanation.is_none() {
        return ApprovalVerdict::VetoedUnexplainedLoss;
    }
    ApprovalVerdict::Approved
}

/// Override path — never silent: the veto yields ONLY to a Madanu decree
/// bearing the Architect's CSR-08 seal. There is no administrative
/// override, no steward escalation, no quiet path.
pub fn override_veto(architect_seal_csr08: bool) -> Result<(), &'static str> {
    if architect_seal_csr08 {
        Ok(())
    } else {
        Err("no administrative override exists; only a Madanu decree bearing the Architect's CSR-08 seal")
    }
}

/// §6 — Release Gate to KAKIv4.0: Gate G4 passage.
pub fn release_gate(approval: ApprovalVerdict, concord_mint_state: bool) -> bool {
    approval == ApprovalVerdict::Approved && concord_mint_state
}

#[cfg(test)]
mod tests {
    use super::*;

    // L13 — betti_diff and census_diff correctly measure loss (never
    // negative, gains don't offset losses on other axes).
    #[test]
    fn l13_loss_measurement() {
        let before = BettiSignature { b0: 1, b1: 2 };
        let after_loss = BettiSignature { b0: 1, b1: 0 };
        assert_eq!(betti_diff(before, after_loss), 2);

        let after_gain = BettiSignature { b0: 1, b1: 3 };
        assert_eq!(betti_diff(before, after_gain), 0, "a gain is never negative loss");

        assert_eq!(census_diff(100, 97), 3);
        assert_eq!(census_diff(100, 105), 0, "more particles is not census loss");
    }

    // L14 — bell veto: approval blocked while bells active or the
    // nabalkutu register is non-empty, regardless of loss state.
    #[test]
    fn l14_bell_veto_blocks_regardless_of_loss() {
        let sig = BettiSignature { b0: 1, b1: 1 };
        let request = ApprovalRequest {
            betti_before: sig,
            betti_after: sig,
            census_before: 10,
            census_after: 10,
            explanation: None,
        };
        let empty = NabalkutuRegister::new();
        assert_eq!(
            evaluate_approval(BellState::MarkasuActive, &empty, &request),
            ApprovalVerdict::VetoedBell
        );
        assert_eq!(
            evaluate_approval(BellState::RigmuActive, &empty, &request),
            ApprovalVerdict::VetoedBell
        );

        let mut sub_judice = NabalkutuRegister::new();
        sub_judice.register(7);
        assert_eq!(
            evaluate_approval(BellState::Quiet, &sub_judice, &request),
            ApprovalVerdict::VetoedBell,
            "non-empty nabalkutu register vetoes even with quiet bells"
        );
    }

    // L15 — loss-explanation clause: unexplained loss vetoes even with
    // quiet bells; an attached explanation lets it pass.
    #[test]
    fn l15_loss_explanation_clause() {
        let empty = NabalkutuRegister::new();
        let lossy = ApprovalRequest {
            betti_before: BettiSignature { b0: 1, b1: 2 },
            betti_after: BettiSignature { b0: 1, b1: 1 },
            census_before: 10,
            census_after: 10,
            explanation: None,
        };
        assert_eq!(
            evaluate_approval(BellState::Quiet, &empty, &lossy),
            ApprovalVerdict::VetoedUnexplainedLoss
        );

        let explanation = LossExplanation { w5h2_text: "true-duplicate removal via ShalaEngine".into() };
        let explained = ApprovalRequest { explanation: Some(&explanation), ..lossy };
        assert_eq!(evaluate_approval(BellState::Quiet, &empty, &explained), ApprovalVerdict::Approved);
    }

    // L16 — override only via signed decree; no administrative override.
    #[test]
    fn l16_override_requires_signed_decree() {
        assert!(override_veto(false).is_err());
        assert!(override_veto(true).is_ok());
    }

    #[test]
    fn release_gate_requires_approval_and_concord() {
        assert!(release_gate(ApprovalVerdict::Approved, true));
        assert!(!release_gate(ApprovalVerdict::Approved, false));
        assert!(!release_gate(ApprovalVerdict::VetoedBell, true));
    }
}
