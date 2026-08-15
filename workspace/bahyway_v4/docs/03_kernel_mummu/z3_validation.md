# Z3 SMT Constraint Validation

> **DubSar Help** | `Kernel > Z3` | Mathematical Brain

## Purpose

Z3 is used alongside Lean 4 for constraint satisfaction problems that are
easier to express as satisfiability queries than as formal proofs — particularly
threshold validation for the four Pauli Exclusion gates.

## Mechanism

- ADAD temporal windows, ANU authority ranks, MARDUK noise limits, and
  SHAMASH judgment scores are validated as Z3 constraints.
- Z3 queries run in the pre-commit hook; a threshold that creates an
  unsatisfiable exclusion rule is rejected before it reaches CI.

## See Also

- `03_kernel_mummu/lean4_verification.md`
- `04_gates/high_council.md`
