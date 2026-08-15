# Threat Model

> **DubSar Help** | `Security > Threat Model` | Security

## Assets

- Identity-KAKI namespace (forgery = identity theft).
- Jordan Chain audit trail (tampering = history falsification).
- Tribe Ideal vectors (manipulation = governance subversion).

## Threats

| Threat | Gate | Mitigation |
| :--- | :--- | :--- |
| Duplicate signal injection | ADAD | Temporal window de-duplication. |
| Low-authority source spoofing | ANU | Authority rank validation. |
| Mid-pipeline data corruption | MARDUK | Transformation lock + VGCA delta check. |
| Dead record reactivation | SHAMASH | Steward approval required for reincarnation. |
| KAKI nucleus forgery | KAKI crate | Lean 4 type invariants in CI. |

## See Also

- `04_gates/high_council.md`
- `18_security/audit_trail.md`
