# Events-KAKI

> **DubSar Help** | `κ_ev` | Identity

## Purpose

The Events-KAKI is the KAKI variant that represents a state-change event in a
particle's Orbit. It is the "Orbit tail" — the entry point of the Jordan Chain.

## Mechanism

- Each new event appends to the nilpotent chain without modifying prior events.
- The ADAD Gate (temporal exclusion) de-duplicates rapid-fire events within the
  configured `ADAD_BREATH_MS` window before an Events-KAKI is minted.
- Old events fall off the chain naturally via nilpotency — no archive job.

## Sovereign Constraints

The Events-KAKI is always subordinate to its Identity-KAKI. It cannot exist
without a parent nucleus.

## See Also

- `02_identity/kaki_triad.md`
- `04_gates/adad_gate.md`
- `01_mathematics/jnf.md`
