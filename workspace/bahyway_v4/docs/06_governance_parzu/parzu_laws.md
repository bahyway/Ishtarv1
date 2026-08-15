# PARZU Governance Laws

> **DubSar Help** | `PARZU` | Governance

## Purpose

PARZU is the governance layer that enforces the .akk law files at runtime.
Every Tribe has a PARZU ruleset that defines which particle states are valid,
which state transitions are permitted, and what thresholds trigger a Stewardship
Gap referral.

## Mechanism

- .akk files declare the PARZU rules for a Tribe in the Akkadian Scripting
  Language.
- At runtime, the PARZU engine evaluates each particle transition against its
  Tribe's .akk rules.
- Rule violations route the particle to the appropriate Pauli Gate for
  exclusion or escalation.

## See Also

- `07_file_formats/akk_format.md`
- `04_gates/high_council.md`
