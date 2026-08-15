# Troubleshooting — ADAD Temporal Collision

> **DubSar Help** | `Troubleshoot > ADAD` | Troubleshooting

## Symptom

Particles are being queued in the Temporal Buffer but not promoted to Active.
The ADAD Gate log shows repeated `NabuViolation` (temporal overlap) errors.

## Cause

Two or more sources are emitting signals for the same Identity-KAKI faster than
the `ADAD_BREATH_MS` window allows.

## Fix

1. Check the source emission rate — reduce polling frequency on the sensor.
2. If the collision is expected (e.g., two legitimate simultaneous readings),
   increase `ADAD_BREATH_MS` in the Tribe's .akk file.
3. If sources are duplicated (data feed misconfiguration), disable the
   duplicate source at the ANU Gate level by lowering its authority rank.

## See Also

- `04_gates/adad_gate.md`
- `04_gates/anu_gate.md`
