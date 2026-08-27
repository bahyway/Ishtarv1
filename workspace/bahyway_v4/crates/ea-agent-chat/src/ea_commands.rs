//! ea_commands.rs — EaAgent quick commands for instant math results.
#![forbid(unsafe_code)]

use ea_agent_algebra::solver::Equation;
use ea_agent_algebra::AlgebraSolver;
use ea_agent_core::{constants::HEPTA_WEIGHTS, HeptaVector};

#[derive(Debug, Clone, PartialEq)]
pub enum EaCommand {
    Pauli,
    Jordan,
    Harmony,
    Solve(String),
    B11(String),
    Hepta,
    Laws,
    Help,
    Status,
    Clear,
    /// `!enkiddb <query>` — real network I/O, handled specially by
    /// `EaChatEngine::chat` (see its match arm) rather than by
    /// `handle_ea_command` below, the same way TamuzAI's `QuickCommand::
    /// Enkiddb` is intercepted before its own generic dispatch: this
    /// function stays pure/no-I/O for every other command.
    Enkiddb(String),
    Unknown(String),
}

pub fn parse_ea_command(msg: &str) -> Option<EaCommand> {
    let msg = msg.trim();
    if !msg.starts_with('!') {
        return None;
    }
    let parts: Vec<&str> = msg[1..].splitn(2, ' ').collect();
    let cmd = parts[0].to_lowercase();
    let arg = parts.get(1).map(|s| s.trim().to_string());

    Some(match cmd.as_str() {
        "pauli" => EaCommand::Pauli,
        "jordan" => EaCommand::Jordan,
        "harmony" => EaCommand::Harmony,
        "solve" => EaCommand::Solve(arg.unwrap_or_default()),
        "b11" => EaCommand::B11(arg.unwrap_or_default()),
        "hepta" => EaCommand::Hepta,
        "laws" => EaCommand::Laws,
        "help" => EaCommand::Help,
        "status" => EaCommand::Status,
        "clear" => EaCommand::Clear,
        "enkiddb" => EaCommand::Enkiddb(arg.unwrap_or_default()),
        other => EaCommand::Unknown(other.to_string()),
    })
}

pub fn handle_ea_command(cmd: &EaCommand) -> String {
    match cmd {
        EaCommand::Pauli   => PAULI_RESPONSE.to_string(),
        EaCommand::Jordan  => JORDAN_RESPONSE.to_string(),
        EaCommand::Harmony => HARMONY_RESPONSE.to_string(),
        EaCommand::Hepta   => HEPTA_RESPONSE.to_string(),
        EaCommand::Laws    => LAWS_RESPONSE.to_string(),
        EaCommand::Help    => HELP_RESPONSE.to_string(),
        EaCommand::Status  => "Check the model status indicator in the panel header.".to_string(),
        EaCommand::Clear   => "🧹 Conversation cleared.".to_string(),
        EaCommand::Solve(expr) => solve_expression(expr),
        EaCommand::B11(vals)   => compute_b11(vals),
        // Real network I/O (enkiddb_rag_client::search) doesn't belong in
        // this pure, no-I/O function -- EaChatEngine::chat() intercepts
        // EaCommand::Enkiddb before it ever reaches here, the same way it
        // already special-cases Clear. This arm only exists so the match
        // stays exhaustive if handle_ea_command is ever called directly.
        EaCommand::Enkiddb(_)  => "!enkiddb must be run through EaChatEngine::chat(), which performs the real EnkiDDB network call.".to_string(),
        EaCommand::Unknown(s)  => format!("Unknown command: !{s}\nType !help for available commands."),
    }
}

fn solve_expression(expr: &str) -> String {
    if expr.is_empty() {
        return "Usage: !solve <equation>\nExamples:\n  !solve x^2 - 5x + 6\n  !solve 2x + 4\n  !solve x^3 - 6x^2 + 11x - 6".to_string();
    }

    let solver = AlgebraSolver::new();
    let expr_lower = expr.to_lowercase().replace(' ', "");

    // Try to parse common forms
    if let Some(result) = try_parse_quadratic(&expr_lower, &solver) {
        return format_solver_result(expr, &result);
    }
    if let Some(result) = try_parse_linear(&expr_lower, &solver) {
        return format_solver_result(expr, &result);
    }

    format!("𒂗𒆠 EaAgent cannot parse: \"{expr}\"\n\nSupported forms:\n  !solve ax + b      (linear)\n  !solve ax^2+bx+c   (quadratic)\n  !solve ax^3+bx^2+cx+d (cubic)\n\nFor complex expressions, ask EaAgent in natural language.")
}

fn try_parse_linear(
    expr: &str,
    solver: &AlgebraSolver,
) -> Option<ea_agent_algebra::solver::SolverResult> {
    // Simple: "2x+4" or "x-3"
    let expr = expr.replace("=0", "").replace('=', "");
    if expr.contains("x^2") || expr.contains("x^3") {
        return None;
    }
    if !expr.contains('x') {
        return None;
    }

    // Extract coefficient of x and constant
    let (a, b) = parse_linear_coeffs(&expr)?;
    Some(solver.solve(&Equation::Linear { a, b }))
}

fn try_parse_quadratic(
    expr: &str,
    solver: &AlgebraSolver,
) -> Option<ea_agent_algebra::solver::SolverResult> {
    if !expr.contains("x^2") && !expr.contains("x²") {
        return None;
    }
    if expr.contains("x^3") {
        return None;
    }

    // Try standard form ax^2+bx+c
    let expr = expr
        .replace("=0", "")
        .replace('=', "")
        .replace("x²", "x^2")
        .replace("x2", "x^2");

    // Simple parser for common forms
    let (a, b, c) = parse_quadratic_coeffs(&expr)?;
    Some(solver.solve(&Equation::Quadratic { a, b, c }))
}

fn parse_linear_coeffs(expr: &str) -> Option<(f64, f64)> {
    // Handle: "2x+3", "x-1", "-x+2", "3x"
    let expr = expr.replace("+-", "-");
    if let Some(pos) = expr.find('x') {
        let a_str = &expr[..pos];
        let a = match a_str {
            "" | "+" => 1.0,
            "-" => -1.0,
            s => s.parse::<f64>().ok()?,
        };
        let rest = &expr[pos + 1..];
        let b = if rest.is_empty() {
            0.0
        } else {
            rest.parse::<f64>().ok()?
        };
        Some((a, b))
    } else {
        None
    }
}

fn parse_quadratic_coeffs(expr: &str) -> Option<(f64, f64, f64)> {
    // Very basic: detect x^2 coefficient, x coefficient, constant
    let expr = expr.replace("+-", "-").replace("-+", "-");
    let x2_pos = expr.find("x^2")?;
    let a_str = &expr[..x2_pos];
    let a = match a_str.trim_matches('+') {
        "" | "+" => 1.0,
        "-" => -1.0,
        s => s.parse::<f64>().ok()?,
    };

    let rest = &expr[x2_pos + 3..];
    if rest.is_empty() {
        return Some((a, 0.0, 0.0));
    }

    // Look for x term and constant
    if let Some(xpos) = rest.find('x') {
        let b_str = &rest[..xpos];
        let b = match b_str.trim_matches('+') {
            "" | "+" => 1.0,
            "-" => -1.0,
            s => s.parse::<f64>().ok()?,
        };
        let c_str = &rest[xpos + 1..];
        let c = if c_str.is_empty() {
            0.0
        } else {
            c_str.parse::<f64>().ok()?
        };
        Some((a, b, c))
    } else {
        let c = rest.parse::<f64>().ok()?;
        Some((a, 0.0, c))
    }
}

fn format_solver_result(expr: &str, r: &ea_agent_algebra::solver::SolverResult) -> String {
    let mut out = format!("𒂗𒆠 EaAgent solves: {expr}\n\n");
    out.push_str("Chain of Thought (Logos):\n");
    for step in &r.steps {
        out.push_str(&format!("  {step}\n"));
    }
    out.push_str(&format!("\nLaTeX: {}\n", r.latex));
    if !r.roots.is_empty() {
        out.push_str("\nRoots: ");
        out.push_str(
            &r.roots
                .iter()
                .map(|x| format!("x={x:.6}"))
                .collect::<Vec<_>>()
                .join(", "),
        );
    }
    if !r.complex.is_empty() {
        out.push_str("\nComplex roots: ");
        out.push_str(
            &r.complex
                .iter()
                .map(|(r, i)| format!("{r:.4}+{i:.4}i"))
                .collect::<Vec<_>>()
                .join(", "),
        );
    }
    out
}

fn compute_b11(vals: &str) -> String {
    if vals.is_empty() {
        return "Usage: !b11 <7 values>\nExample: !b11 0.9 0.8 0.7 0.85 0.75 0.6 0.95".to_string();
    }
    let nums: Vec<f64> = vals
        .split_whitespace()
        .filter_map(|s| s.parse::<f64>().ok())
        .collect();
    if nums.len() != 7 {
        return format!(
            "Need exactly 7 values (D1..D7), got {}.\nExample: !b11 0.9 0.8 0.7 0.85 0.75 0.6 0.95",
            nums.len()
        );
    }
    let mut d = [0.0f64; 7];
    d.copy_from_slice(&nums);
    let v = HeptaVector::new(d);
    let q = v.quality_score();
    let b11 = v.b11();
    let lane = match b11 {
        200..=255 => "● GEM (Golden Record)",
        140..=199 => "● TRIBE",
        100..=139 => "● ACTIVE",
        60..=99 => "● FUZZY",
        _ => "● DEAD",
    };

    let mut out = "𒂗𒆠 H(P) Computation\n\n".to_string();
    out.push_str("H(P) = 1 / (1 + √ Σᵢwᵢ(Pᵢ−Tᵢ)²)\n\n");
    for (i, (di, wi)) in d.iter().zip(HEPTA_WEIGHTS.iter()).enumerate() {
        out.push_str(&format!(
            "  D{}: {:.4} × w={:.2} → contribution={:.6}\n",
            i + 1,
            di,
            wi,
            wi * (di - 1.0).powi(2)
        ));
    }
    out.push_str(&format!(
        "\nH(P) = {q:.6}\nB11  = round({q:.6} × 240) = {b11}\nLane = {lane}\n"
    ));
    out
}

const PAULI_RESPONSE: &str = "\
𒂗𒆠 Pauli Exclusion Principle in BahyWay
═══════════════════════════════════════

\"No two particles may occupy the same
 (Identity-KAKI, Orbit-Level, QuantumState) triplet simultaneously.\"
                    — The High Council of Data

Three Quantum Numbers:
  λ (Eigenvalue)    — Identity-KAKI stability from H(P)
  n (Principal)     — Position in Jordan Chain (orbit depth)
  s (Spin/Status)   — Dead(−1) | Stewardship(0) | Active(+1)

Three Energy Shells:
  Active      s=+1  Conduction Band  → Golden Record candidate
  Stewardship s=0   Forbidden Gap    → awaiting Data Steward
  Dead        s=-1  Valence Band     → archived, mathematical barrier

Detection: !pauli check uses Manhattan distance in 7D Hepta space
Threshold: PAULI_EXCLUSION_THRESHOLD = 0.05
Severity:  Critical < threshold/4 | High < threshold/2 | Warning < threshold

Use !solve to verify exclusion coordinates algebraically.";

const JORDAN_RESPONSE: &str = "\
𒂗𒆠 Jordan Normal Form in BahyWay (Enlil Algebra)
══════════════════════════════════════════════════

Each Tribe = independent Jordan Block in the adjacency matrix.
Spectral Radius ρ(A) determines stability:

  ρ ≥ 0.85  → SOVEREIGN  ✅ Tribe is stable
  ρ ∈ [0.50, 0.85) → MARGINAL ⚠ Monitor decay
  ρ < 0.50  → CRITICAL   🔴 Collapse imminent
  ρ = 0     → VOID       ⚫ Tribe has collapsed

Jordan Chain:
  Each Orbit = nilpotent chain
  Old events fall off naturally (no separate cleanup job)
  EnkiDB = Full Jordan Form (operational)
  EnkiDW = Diagonalized Matrix (analytical)

Block-diagonal property:
  Tribe A has ZERO cross-talk with Tribe B → infinite horizontal scaling

Try: !solve (spectral radius) to compute exact eigenvalues.";

const HARMONY_RESPONSE: &str = "\
𒂗𒆠 Tribe Harmony Coefficient
══════════════════════════════

τ = (∏ᵢ₌₁⁷ (1 − σᵢ²))^(1/7)

Where σᵢ² = variance of dimension i across all tribe particles.

τ = 1.0 → perfect harmony (all particles identical in all dimensions)
τ = 0.0 → complete disorder

ADR-004 target: GEM_RATE_TARGET = 35.4%
  At least 35.4% of particles must have B11 ≥ 200 (GEM lane)

Harmony feeds into:
  JordanAnalyzer  → spectral radius
  CollapsePredictor → trend slope
  OracleEngine    → KAKI-stamped decisions

Use !b11 to compute individual particle quality.";

const HEPTA_RESPONSE: &str = "\
𒂗𒆠 The 7-Dimensional Hepta Basis (Ibn Wahshiyya)
══════════════════════════════════════════════════

D1  Identity   w=0.30  uuid_hash   — κ[0..3]
D2  Belonging  w=0.20  tribe_id    — κ[4..5]
D3  Domain     w=0.15  RED byte    — EAV color_rgb (NOT in KAKI)
D4  Quality    w=0.15  GREEN byte  — EAV color_rgb (NOT in KAKI)
D5  Freshness  w=0.10  BLUE byte   — EAV color_rgb (NOT in KAKI)
D6  Temporal   w=0.05  timestamp   — κ[12..13]
D7  Integrity  w=0.05  checksum    — κ[14..15]

H(P) = 1 / (1 + √ Σᵢwᵢ(Pᵢ−Tᵢ)²)
B11  = round(H(P) × 240)    ← QUALITY_DIVISOR=240 (Plimpton 322)

⚠ ColorID (RGB) = EAV attribute — NOT in the 16 KAKI bytes

Use !b11 <d1 d2 d3 d4 d5 d6 d7> to compute B11 for any vector.";

const LAWS_RESPONSE: &str = "\
𒂗𒆠 The 10 Sovereign Laws — Mathematical Perspective
═════════════════════════════════════════════════════

§1  No external database — EnkiDB is the only storage
§2  No UPDATE/DELETE — append-only (Axiom 8: particle permanence)
§3  No assessments in KAKI nucleus — structural facts only (§2.4)
§4  CrossTribe state on PROBE — never stored (§8.3)
§5  B11 = round(H(P) × 240) — never manually assigned
§6  Every decision = KAKI-stamped .akk file (audit by architecture)
§7  No unwrap() in production code
§8  No logic in main.rs
§9  ColorID (RGB) in EAV — never in KAKI bytes
§10 -Way suffix reserved for .way security policy files only

EaAgent enforces §3, §4, §5, §9 algebraically.
Every EaAgent decision is stamped per §6.";

const HELP_RESPONSE: &str = "\
𒂗𒆠 EaAgent Quick Commands
══════════════════════════
!pauli         — Pauli Exclusion Principle explanation
!jordan        — Jordan Normal Form & tribe stability
!harmony       — Tribe Harmony Coefficient formula
!hepta         — 7D Hepta basis (Ibn Wahshiyya dimensions)
!laws          — 10 Sovereign Laws (mathematical perspective)
!solve <expr>  — Exact algebraic solver (linear/quadratic/cubic)
!b11 <7 vals>  — Compute B11 from Hepta vector values
!enkiddb <phrase> — Real SEARCH: against EnkiDDB's Read Node (RAG)
!status        — DeepSeek-Math model status
!clear         — Clear conversation
!help          — This help message

Examples:
  !solve x^2 - 5x + 6
  !b11 0.9 0.8 0.7 0.85 0.75 0.6 0.95
  !solve 2x + 8
  !enkiddb HeptaScript ingestion

For complex questions, ask naturally:
  'What happens when two particles share the same D3 domain?'
  'Explain why the spectral radius of this tribe is declining'";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_pauli_command() {
        assert_eq!(parse_ea_command("!pauli"), Some(EaCommand::Pauli));
    }
    #[test]
    fn parse_solve_command() {
        assert_eq!(
            parse_ea_command("!solve x^2-5x+6"),
            Some(EaCommand::Solve("x^2-5x+6".into()))
        );
    }
    #[test]
    fn parse_b11_command() {
        let cmd = parse_ea_command("!b11 0.9 0.8 0.7 0.85 0.75 0.6 0.95");
        assert!(matches!(cmd, Some(EaCommand::B11(_))));
    }
    #[test]
    fn parse_non_command() {
        assert_eq!(parse_ea_command("What is Pauli?"), None);
    }
    #[test]
    fn parse_enkiddb_command() {
        assert_eq!(
            parse_ea_command("!enkiddb spectral radius"),
            Some(EaCommand::Enkiddb("spectral radius".to_string()))
        );
    }
    #[test]
    fn handle_pauli_contains_quantum() {
        let r = handle_ea_command(&EaCommand::Pauli);
        assert!(r.contains("Quantum"));
        assert!(r.contains("KAKI"));
    }
    #[test]
    fn handle_jordan_contains_spectral() {
        let r = handle_ea_command(&EaCommand::Jordan);
        assert!(r.contains("Spectral"));
        assert!(r.contains("0.85"));
    }
    #[test]
    fn b11_correct_computation() {
        let r = handle_ea_command(&EaCommand::B11("1.0 1.0 1.0 1.0 1.0 1.0 1.0".into()));
        assert!(r.contains("240"));
        assert!(r.contains("GEM"));
    }
    #[test]
    fn b11_wrong_count_returns_error() {
        let r = handle_ea_command(&EaCommand::B11("0.9 0.8".into()));
        assert!(r.contains("7 values"));
    }
    #[test]
    fn solve_linear_correct() {
        let r = handle_ea_command(&EaCommand::Solve("2x+4".into()));
        assert!(r.contains("-2") || r.contains("x="));
    }
    #[test]
    fn help_contains_all_commands() {
        let r = handle_ea_command(&EaCommand::Help);
        assert!(r.contains("!pauli"));
        assert!(r.contains("!jordan"));
        assert!(r.contains("!solve"));
        assert!(r.contains("!b11"));
    }
}
