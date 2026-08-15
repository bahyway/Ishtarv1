# Verification Stage

> **DubSar Help** | `Pipeline > Verify` | CI/CD

## Purpose

The verification stage applies Lean 4 proof obligations and Z3 constraint
checks to the staged particle before it is released to the Active Orbit.

## Steps

1. Lean 4 prover checks algebraic invariants (§ references from Particles Algebra).
2. Z3 validates threshold constraints for ADAD, ANU, MARDUK, SHAMASH.
3. If all checks pass: particle state transitions `Stewardship → Active`.
4. If any check fails: particle remains in Stewardship; Steward is notified.

## See Also

- `08_pipeline_alaktu/submission.md`
- `08_pipeline_alaktu/release.md`
- `03_kernel_mummu/lean4_verification.md`
