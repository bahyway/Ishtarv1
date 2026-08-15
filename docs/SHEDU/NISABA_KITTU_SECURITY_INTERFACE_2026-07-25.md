# Nisaba <-> Kittu Engine — Security Alert Flow
Standalone architecture note | 2026-07-25 | Node: eriduous-vdi
Status: confirmed, not yet folded into BC-SEC-001

## Flow

1. Security KAKI particle lands in EnkiSDB
   (UrNammu Engine attestation event / AkkadiRulesEngine usbguard decision /
   AkkadiCipherEngine seal-unseal event)
2. Nisaba evaluates — rule-violation pattern OR existing topological/colour
   anomaly trigger (β₁>0, β₀ deviation, ColourID hue variance)
3. Violation confirmed -> Nisaba emits a signed Alert Event KAKI
   (severity, rule_id, tribe_id, evidence particle refs)
4. Kittu Engine consumes Alert Events
5. Delivery (v1, internal-only):
   - Dashboard surfacing via ShoWEngine
   - Email to security team
   - NO telephony / external API in v1
6. Delivery confirmation written back as its own KAKI Event
   -> closes the loop immutably

## Separation of concerns

Nisaba: detection + declaration only.
Kittu Engine: delivery only.
This mirrors the single-responsibility pattern already used across the
agent roster (TamuzAI = code, EaAgent = math, AdadAI = ingestion,
NINSUN = healing/refinement).

## Example HeptaScript query (Anti-SQL — W5H2)

```
PRESENT
  WHO:   tribe_id
  WHAT:  EAV.decision
  WHEN:  today
  WHERE: kaki_type = CrossTribe AND EAV.device_class = "mass_storage"
  HOW:   grouped by tribe_id
```

## Open items

- Kittu Engine email/dashboard split implementation
- Confirm whether Kittu Engine lives in SHEDU or as a standalone
  cross-sector notification member
- Decide whether/when to fold this note into BC-SEC-001 as a new section
