# Audit Trail

> **DubSar Help** | `Security > Audit Trail` | Security

## Purpose

Every particle state transition is KAKI-stamped and written to EnkiDW in
append-only fashion. The audit trail is the Jordan Chain itself.

## What Is Recorded

- Events-KAKI of each transition.
- Timestamp (D6 byte of KAKI).
- Source authority rank (ANU Gate decision).
- Gate decisions (ADAD/ANU/MARDUK/SHAMASH pass/fail).
- VGCA delta that MARDUK evaluated.
- HPS value before and after the transition.
- .akk rule firing that triggered the transition.

## Queryable via

`nabu audit <kaki>` returns the full Jordan Chain for any particle.

## See Also

- `05_storage/enkidw.md`
- `04_gates/high_council.md`
