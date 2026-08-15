# AkkadiRulesEngine — Compiled Reference (Recovery Edition)
**BahyWay.Ecosystem v4.0 · SHEDU security sector · assembled 2026-08-05**

**Provenance note:** no standalone AkkadiRulesEngine document previously
existed. This reference compiles every sealed statement about the engine
from four sources: [A] the v3.5 BAHYWAYOTAP catalog session (2026-03,
"AkkadiRulesWay, member #15"), [B] BC-SEC-001 UrNammu Engine (2026-07),
[C] HS-EXT-002 HeptaScript v1.2 spec (2026-07), [D] this era's laws
(GL-TKT-001, GL-MDM-001, GL-TPL-001). v3.5 fossils are corrected to v4.0
law and marked. If a lost original resurfaces, it rules.

---

## 1. Identity and Charter  [A, corrected]

AkkadiRulesEngine is the ABAC policy engine of the SHEDU triad
(AkkadiSafeEngine · AkkadiRulesEngine · AkkadiCipherEngine), standing on
UrNammu's hardware trust. Its charter: **AkkadianAOL policy declarations
→ compiled → Rust enforcement functions** running at every API gateway,
every service boundary, every credential access, every EnkiDB query.

*The Hammurabi Connection:* the Code of Hammurabi was the first written
legal code — carved in Akkadian cuneiform, ~1754 BCE. AkkadiRulesEngine
is the digital Hammurabi Code. Your laws. Your stack. Carved in Rust.

**Key insight: rules are code, not config.** No runtime interpreter —
policies compile to zero-cost Rust (order-of-magnitude figures from the
v3.5 catalog: ~0.002 ms per decision vs ~2–5 ms for interpreted cloud
policy engines). Crate scale (v3.5 estimate): ~4,000 lines compiler
extension + ~1,500 lines enforcement runtime.

## 2. The Three Rule Classes  [A, verbatim syntax]

**2.1 ACCESS POLICY (ABAC)**

    policy EnkiDB_Access {
        allow DUB_SAR    { actions [Read, Write, Admin] }
        allow API_Client { actions [Read] where domain != Sovereign }
        deny  *          { where mfa == false }
    }

**2.2 FIREWALL RULES**

    firewall BahyWay_Edge {
        rule RateGuard   { limit 100/min       action Throttle }
        rule SQLiGuard   { match sqli_patterns  action BlackBox }
        rule PromptGuard { match prompt_inject  action BlackBox }
    }

**2.3 DATA GOVERNANCE RULES**

    rule DataSovereignty {
        on      EnkiDB.export
        where   domain == Sovereign
        action  Deny
        log     AuditTrail
    }

## 3. Compilation Path  [A, corrected to v4.0]

    AkkadianAOL source (.akk)
        │  AkkadianAOL Compiler (AAOL Core — bidirectional)
        ▼
    Rust enforcement functions   // GENERATED — DO NOT EDIT
        │  compiled into BahyWay services
        ▼
    zero-cost policy enforcement (pure compiled Rust)

Generated-function shape (v3.5 example, corrected names): deny-first
evaluation, explicit `Decision::Allow / Decision::Deny{reason}` — e.g.
`enforce_enkidb_access(identity, req)` checks MFA deny, then role
allows in declaration order, falling through to deny.

## 4. Subjects, Attributes, Decisions  [B, C, D]

- **Subjects** are sovereign stakeholder roles: DubSar, TabletWriter,
  DataSteward, Client — the HS-EXT-002 `STAKEHOLDER(role,…)` noun in
  the WHO slot maps directly onto them, and maps to the Θ dimension of
  the Transparency Deficit Calculus. [C]
- **Attributes** come from EAV Mandatory Attributes, tribe_id (κ[4..5]),
  and request context (action, domain, MFA state). Never from repurposed
  KAKI bytes — the layout is locked. [C]
- **Decisions are BLOCKING.** The governance layering law: WHO/STAKEHOLDER
  → AkkadiRulesEngine ABAC (blocking); WHY/UNDER SLA → planner budget
  with breaches WITNESSed (advisory record). AkkadiRulesEngine is the
  only layer with the authority to refuse. [C]

## 5. Integration Contracts  [B, D]

| Consumer | Contract |
|---|---|
| UrNammu Engine | usbguard port/peripheral decisions are ABAC-driven, keyed on tribe_id/role; attestation events land in EnkiSDB as KAKI particles [B] |
| AkkadiCipherEngine | key release gated on TPM attestation of known-good boot state [B] |
| AkkadiSafeEngine | vault unlock requires a passing UrNammu attestation event [B] |
| Ticket system (GL-TKT-001 §6) | tenancy and sight: a client's stakeholders see only their own tribe's tickets; the steward sees the docket; the Architect sees the kingdom [D] |
| BeeMDM (GL-MDM-001 §4) | DMBOK-derived unification policies live as rules here — enforcement is policy execution, not persuasion [D] |
| Arsenal (GL-TPL-001 §4) | the abstraction/IP boundary on templates is policed at export [D] |
| HeptaScript | every STAKEHOLDER-filtered query passes through enforcement before ENLIL answers [C] |

## 6. The Transparency Invariant  [A v3.5, restated under v4.0 law]

Every ecosystem behavior must be traceable to a sealed .akk rule
artifact that existed before the behavior began, whose compile status
is recorded (v4.0: as an EAV Mandatory Attribute — the v3.5 GREEN-byte
form is a fossil per the locked KAKI layout), whose CRC/seal is valid,
and whose governance link is chronicled in StoryEngine. If a behavior
cannot be traced to a sealed rule, it is a BUG — not a feature gap.
Consequences: a client auditor can read every active rule governing
their tenant in human-readable Akkadian form without a developer
present; a regulator gets the complete evidence chain (intent event,
rule content, timestamp, compiler result, actor); the Architect
returning after a week can diff behavior by diffing sealed rules.

## 7. v4.0 Corrections Applied

-Way suffix fully deprecated (AkkadiRulesWay → AkkadiRulesEngine, sealed
in the BC-SEC-001 session); EnkiWay → EnkiDB family; RGB/GREEN score
bytes do not exist in v4.0 — all compile/quality state lives exclusively
in EAV; rules-as-particles inherit the append-only law (GL-DST-003): a
policy change is a new sealed version with lineage, never an edit.

## 8. Open Items for the Architect

(a) Formalize this reference as BC-SEC-002 (Word, BC-house style) —
one session's work on request. (b) Rule whether policy artifacts get
their own kaki_type ordinal in the GL-STY-001 event/type registry.
(c) The PollutionWay-era policy script (7-section .akk: namespace,
constants, flow rules, alarms, firewall, write policy, cipher
directives) stands as the fullest historical example file — worth
re-pressing as the v4.0 policy template when the gate opens.

— Compiled for DUB.SAR 𒁾, BahyWay.Ecosystem v4.0.
