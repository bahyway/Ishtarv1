# Troubleshooting — SHAMASH Zombie Detection

> **DubSar Help** | `Troubleshoot > Zombie` | Troubleshooting

## Symptom

A signal arrives for an Identity-KAKI that is in the Dead State. The SHAMASH
Gate log shows `EreshkigalViolation` (unauthorized reincarnation).

## Cause

A replaced sensor or re-activated entity is emitting signals, but its
Identity-KAKI is still marked Dead in EnkiDB.

## Fix

1. `nabu audit <kaki>` — confirm the particle is in the Dead State and review
   the SHAMASH judgment that archived it.
2. If this is a **Zombie** (the signal is a mistake): reject the signal at the
   ADAD Gate by adding the Identity-KAKI to the blocklist.
3. If this is a **Reincarnation** (the sensor was genuinely replaced): the Data
   Steward approves the reincarnation. A new Events-KAKI lineage begins; the
   Dead particle remains archived.

## See Also

- `04_gates/shamash_gate.md`
- `04_gates/pauli_exclusion.md`
