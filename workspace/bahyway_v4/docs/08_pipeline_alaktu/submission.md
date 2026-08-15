# Submission Pipeline

> **DubSar Help** | `Pipeline > Submit` | CI/CD

## Purpose

The submission stage is the entry point of the ALAKTU pipeline. A particle or
.akk file is submitted, validated by the four Pauli Gates, and queued for the
verification stage.

## Steps

1. Signal arrives at ADAD Gate (temporal de-duplication).
2. Signal passes ANU Gate (authority check).
3. Signal passes MARDUK Gate (structure / noise check).
4. Signal passes SHAMASH Gate (state / archive check).
5. Particle is staged in the hot plane with `state = Stewardship`.
6. Pipeline notifies the verification stage.

## See Also

- `08_pipeline_alaktu/verification.md`
- `04_gates/high_council.md`
