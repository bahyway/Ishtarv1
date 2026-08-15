# Lean 4 Verification Engine

> **DubSar Help** | `Kernel > Lean4` | Mathematical Brain

## Purpose

The Lean 4 prover is the Mathematical Brain (Mummu) of BahyWay. Every
algebraic identity, Pauli Exclusion rule, and KAKI uniqueness constraint is
expressed as a Lean 4 theorem before it is implemented in Rust.

## Mechanism

- Theorems PA-1 through PA-16 have corresponding Lean 4 proofs.
- The Pauli Exclusion gates (ADAD, ANU, MARDUK, SHAMASH) are formalised as
  type-level constraints in Lean 4.
- Proof obligations are checked in CI before any Rust crate is published.

## Sovereign Constraints

No runtime behaviour that is not covered by a Lean 4 proof may be merged into
the main branch.

## See Also

- `03_kernel_mummu/z3_validation.md`
- `01_mathematics/particles_algebra.md`
