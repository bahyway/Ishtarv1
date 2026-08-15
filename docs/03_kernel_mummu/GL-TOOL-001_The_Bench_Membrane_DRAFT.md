# GL-TOOL-001 (candidate) — THE BENCH MEMBRANE
## The universal law for every external tool: it may compute and witness, on the bench — it may never cleanse, and never ship
### BahyWay.Ecosystem v4.0 · binds GL-GOV-001/002/003 · GL-HS3-002 · CSR-08 · the pure-Rust sovereignty commitment · Status: DRAFT — pending CSR-08 by DUB.SAR 𒁾

---

## 0 · Why this law exists

The ecosystem repeatedly meets powerful external tools — Wolfram's hypergraph
algorithm, Z3, Julia, QUTE, and a hundred not yet encountered. Deciding each one
case-by-case is slow and inconsistent, and inconsistency is where the discipline
leaks. This law is the single **membrane** every tool is held against, so the
decision is the same every time and made by the same test.

The test emerged from real cases and sorted all of them correctly:
- **Wolfram** (hypergraph rewrite) → simulation workshop, never the showroom → **passes.**
- **Z3** (SMT proofs) → design-time, never in the shipped binary → **passes.**
- **Julia** (scientific computing) → computes & validates the math, decides no
  truth, delivered to no one → **passes.**
- **QUTE** (trained neural ensemble) → its *purpose* fits, but its *mechanism*
  would ship a black box into the sovereign core → **refused from the core**
  (its pattern re-implemented deterministically instead).

One test sorted four tools. That test is this law.

---

## 1 · The Law

**Any external tool may be used only as a *bench instrument*: it may COMPUTE (run
the mathematics, the simulation, the analysis) and it may WITNESS (serve as a
reference the sovereign implementation is validated against). It may NEVER
CLEANSE (decide what data is true, clean, kept, or discarded — that is a ruling
reserved to the ranked authority) and it may NEVER SHIP (enter the sovereign
binary delivered to a client). Computes-and-witnesses passes the membrane;
cleanses-or-ships is stopped at it.**

---

## 2 · The two verbs that pass, the two that are stopped

**PASSES — a bench instrument may:**
- **COMPUTE** — run the FCA, the TDA/persistent homology, the gravity-field
  dynamics, the simulation, the hard math. Produce results *for the bench*.
- **WITNESS** — serve as the reference oracle the pure-Rust implementation is
  checked against. *The tool proves the Rust is correct; it never hands the Rust
  the answer.* (Rust computes independently; the tool confirms.)

**STOPPED — a bench instrument may never:**
- **CLEANSE** — decide which data is clean/true/kept/dropped, fill a gap, assert
  an edge, name an Unknown cluster. Every such decision is a *ruling* and belongs
  to the ranked authority with a witness (GL-GOV-001). A tool may *reveal* where a
  shape is dirty or holed; it may never *decide* to change it. (Same rule as the
  ε-lens: detects, never names.)
- **SHIP** — enter the sovereign runtime delivered to a client. The shipped body
  is pure Rust (DubSar Theater / WGPU). External tools live at research-time and
  design-time, on the development host, and are discarded from anything shipped —
  exactly as Z3 is downloaded once, used at Gate G4, and never in the binary.

---

## 3 · The single discriminating test

For any tool, ask exactly two questions:

1. **Does it decide anything about truth?** (cleanse) — if yes, it is stopped
   from that role; the decision returns to the ranked authority.
2. **Does it ship inside the sovereign binary?** — if yes, it is stopped from the
   core; either re-implement its pattern in pure Rust, or keep it on the bench.

**A tool passes only if BOTH answers are no** — it computes and witnesses on the
bench, decides no truth, and ships to no one. Every other case is stopped at the
membrane, in whole or in the offending part.

---

## 4 · Why this is the same refusal as every other law

The Bench Membrane is not a new principle — it is the ecosystem's one refusal,
applied to tools:
- **GOLDEN ⟹ verified fact** — a tool's output is not fact until proven.
- **GL-GOV-001** — a tool may propose (reveal structure); only the authority
  admits (cleanse/decide), with a witness.
- **GL-GOV-002** — a tool may not assert more than it proves; it computes and
  witnesses, it does not rule.
- **GL-GOV-003** — a tool's simulation may not reach the stakeholder unsigned.
- **Sovereignty** — the shipped body is pure Rust; no tool ships inside it.
The membrane simply names, once, the boundary all of these draw around tools.

---

## 5 · The pattern-not-tool corollary

When a tool's *capability* is wanted but the tool itself is stopped (it cleanses
or ships), the move is **port the pattern, refuse the tool**: re-implement the
capability deterministically in sovereign pure Rust, and use the original tool
only as the bench WITNESS that the Rust version is correct. (QUTE's
uncertainty-monitoring pattern → the deterministic two-witness + ε monitor;
Niagara's GPU-particle technique → native WGPU compute; Julia's math → Rust
validated against a Julia reference.) The capability enters; the tool does not.

---

## 6 · Honest scope (what this guarantees, what it doesn't)
- **Guaranteed:** no external tool ships in the sovereign binary; no tool's output
  is treated as an admitted truth without the ranked authority. These are enforced
  by the build (nothing but Rust in the shipped artifact) and by governance
  (cleansing is an authority ruling).
- **Not guaranteed:** the law cannot stop a developer from *believing* a bench
  tool's output and acting on it informally. What it guarantees is that such a
  belief has no *sanctioned path* into truth or into the shipped system without
  passing the authority and the Rust re-implementation. The bench is where belief
  is formed; the membrane is where it must earn its crossing. This limit is
  stated, not hidden.
- **This is a sovereignty commitment, not a law of nature.** Many fine systems
  embed Julia or other runtimes happily. The membrane holds because DUB.SAR
  sealed pure-Rust sovereignty as foundational. Should that commitment ever be
  revisited, it must be revisited *deliberately and on the record* (CSR-08) — not
  dissolved quietly by a convenient tool slipping across the membrane.

## 7 · Codex compliance & placement
- **A-1 zero new mathematics:** composes the existing refusals (GOLDEN scope,
  GL-GOV-001/002/003, Z3 design-time precedent, pure-Rust sovereignty) into one
  named boundary for tools. New = the *membrane* framing and the two-verb test.
- **PB:** PB-401 (below) — the tool-intake checklist wired into the build/CI.

## 8 · Open seals for CSR-08
The Bench Membrane · the COMPUTE/WITNESS-pass, CLEANSE/SHIP-stop rule · the
two-question discriminating test · the pattern-not-tool corollary · the
honest-scope limits (belief on the bench; sovereignty as a revisable-on-record
commitment) · PB-401.

*Recorded in the reign of Gudea 1.0. A tool is welcome at the bench and forbidden
in the body. Let it compute for us and bear witness to our proofs — and let it
neither decide what is true nor travel to the client in our name. What we ship,
we wrote; what we claim, we proved; and every tool that helped us stays behind at
the bench when the work goes out the door. Nothing sealed until DUB.SAR confirms
under CSR-08.*
