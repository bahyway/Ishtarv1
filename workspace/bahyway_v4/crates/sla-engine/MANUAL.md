# sla-engine — SLA Governance Engine

Configurable fuzzy-logic compliance scoring for all BahyWay Enterprise App topologies. Administrators select applicable requirements through the HTML GUI; the evaluator produces B11 scores on the 0–240 QUALITY_DIVISOR scale.

## Architecture

```
Stakeholder (Admin / Data Steward)
        │
        ▼
SlaGuiRenderer::render_selection_html()   ← browser form
        │  (POST: req_{id}_enabled, req_{id}_level per requirement)
        ▼
ComplianceState { HashMap<u16, MaturityLevel> }
        │
        ▼
SlaEvaluator::evaluate(profile, state, epoch) ← fuzzy scoring
        │
        ▼
SlaReport { domain_scores, overall_b11, go_live_ready, priority_actions }
        │
        ▼
SlaGuiRenderer::render_dashboard_html(report)  ← compliance dashboard
```

## Topology Profiles

Each BahyWay application has a pre-built `SlaProfile` selected via `AppTopology`:

| Topology              | Target B11 | Primary Domains                          |
|-----------------------|-----------|------------------------------------------|
| `NajafEngine`         | 200 (83%) | GDPR Privacy, Najaf Privacy              |
| `WpdPipeline`         | 180 (75%) | WPD Infrastructure, Network Security     |
| `WebGateway`          | 192 (80%) | Network Security, Operational            |
| `EnkiDb`              | 192 (80%) | Encryption at Rest, Key Management       |
| `EnterpriseAll`       | 180 (75%) | All 8 domains                            |

Administrators can add/remove individual requirements from any profile via `SlaProfile::add(id)` / `SlaProfile::remove(id)`.

## Compliance Domains

| Domain                    | Icon | Default Weight | Description                       |
|---------------------------|------|----------------|-----------------------------------|
| `GdprPrivacy`             | 🔏   | 240            | GDPR + data subject rights        |
| `EncryptionAtRest`        | 🔐   | 240            | PII fields, LUKS, vault keys      |
| `KeyManagement`           | 🗝   | 200            | Rotation, erased-hash persistence |
| `NetworkSecurity`         | 🌐   | 200            | TLS 1.3, rate limiting, WAF       |
| `OperationalSecurity`     | 🛡   | 180            | IRP, pen test, audit logs         |
| `NajafPrivacy`            | 🕌   | 220            | PII sidecar, Islamic sovereignty  |
| `WpdInfrastructure`       | 🏗   | 200            | NIS2, ENKWAL, threat model        |
| `GovernanceAssurance`     | 📋   | 180            | STRIDE, ISO27001, SOC2, BCP       |

## Maturity Levels (Fuzzy Membership)

| Level           | Fuzzy | B11 contribution | Meaning                              |
|-----------------|-------|-----------------|--------------------------------------|
| `NotApplicable` | 1.0   | excluded        | Requirement excluded from scoring    |
| `NotStarted`    | 0.0   | 0               | Nothing done                         |
| `Planned`       | 0.2   | 20% credit      | Architecture decision made           |
| `InProgress`    | 0.5   | 50% credit      | Partial implementation               |
| `Implemented`   | 0.8   | 80% credit      | Complete but not verified            |
| `Verified`      | 1.0   | 100% credit     | Implemented + independently audited  |

## Scoring Formula

```
req_b11  = round(fuzzy_score × req.weight / QUALITY_DIVISOR × QUALITY_DIVISOR)
domain_b11 = round(weighted_avg(req_b11) × domain_weight / 240)
overall_b11 = weighted_avg(domain_b11)   // weights = profile.domain_weight(domain)
```

All scores are on the B11 scale (0–240, `QUALITY_DIVISOR = 240`, ADR-001).

## Compliance Thresholds

| Threshold              | B11  | %   | Meaning                                      |
|------------------------|------|-----|----------------------------------------------|
| `MANDATORY_THRESHOLD`  | 192  | 80% | Mandatory reqs must reach this or block go-live |
| `GO_LIVE_THRESHOLD`    | 180  | 75% | Overall score must reach this for go-live    |
| Sovereign              | 200  | 83% | GEM — highest compliance tier                |

## Compliance Status

| Status            | B11 Range  | Go-live | CSS class         |
|-------------------|-----------|---------|-------------------|
| `Sovereign`       | ≥ 200     | ✅      | `sovereign`       |
| `Compliant`       | 180–199   | ✅      | `compliant`       |
| `Partial`         | 120–179   | ⚠      | `partial`         |
| `ActionRequired`  | < 120     | ❌      | `action-required` |
| `MandatoryGap`    | any       | ❌ BLOCK | `mandatory-gap`  |

`MandatoryGap` overrides the score — any mandatory requirement below 192 blocks go-live regardless of overall score.

## Priority Action Levels

| Priority      | Meaning                       |
|---------------|-------------------------------|
| `P0Blocking`  | Mandatory gap — blocks go-live |
| `P1Required`  | Mandatory req not at threshold |
| `P2Soon`      | High-weight req (≥200), 90 days |
| `P3Roadmap`   | Recommended, 12-month plan    |

## Usage Example

```rust
use sla_engine::{SlaProfile, AppTopology, ComplianceState, MaturityLevel, SlaEvaluator, SlaGuiRenderer};

// 1. Pick a topology
let profile = SlaProfile::for_topology(AppTopology::NajafEngine);

// 2. Admin fills in the state (from form POST)
let mut state = ComplianceState::new();
state.set(1001, MaturityLevel::Implemented);   // DPIA
state.set(1004, MaturityLevel::Verified);       // Erasure mechanism
state.set(2001, MaturityLevel::Implemented);   // PII field encryption
// ...

// 3. Evaluate
let report = SlaEvaluator::evaluate(&profile, &state, now_epoch_u32);

// 4. Render dashboard
let html = SlaGuiRenderer::render_dashboard_html(&report);

// 5. Render selection form for next update
let form_html = SlaGuiRenderer::render_selection_html(&profile, "/sla/evaluate");
```

## Requirement Catalog

Requirements are defined in `src/requirements.rs` as a static slice (`ALL_REQUIREMENTS`). Each has:

- `id: u16` — stable numeric identifier
- `domain: ComplianceDomain`
- `name: &'static str`
- `description: &'static str`
- `weight: u8` — 0–240, relative importance within the domain
- `mandatory: bool` — if true, must reach MANDATORY_THRESHOLD (192)
- `akk_policy: Option<&'static str>` — WAYv4.0 policy reference
- `gap_action: &'static str` — remediation action text

Key requirement IDs:

| ID   | Name                          | Domain              | Mandatory |
|------|-------------------------------|---------------------|-----------|
| 1001 | DPIA                          | GDPR Privacy        | ✅        |
| 1004 | Erasure mechanism             | GDPR Privacy        | ✅        |
| 2001 | PII field encryption          | Encryption at Rest  | ✅        |
| 2002 | LUKS volume encryption        | Encryption at Rest  | ✅        |
| 2004 | PiiVault master key protection | Encryption at Rest | ✅        |
| 2101 | Key rotation procedure        | Key Management      | ✅        |
| 2102 | Erased-hash persistence       | Key Management      |           |
| 3001 | hepta-sec-web boundary guard  | Network Security    | ✅        |
| 3003 | TLS 1.3 everywhere            | Network Security    | ✅        |
| 5001 | PII sidecar                   | Najaf Privacy       | ✅        |
| 6001 | NIS2 24h incident reporting   | WPD Infrastructure  | ✅        |

## HTML Generation

`SlaGuiRenderer` produces self-contained HTML with inline CSS. No JavaScript framework required. No external crate dependencies.

**Selection form** (`render_selection_html`):
- POST fields: `req_{id}_enabled` (checkbox) + `req_{id}_level` (u8 select)
- Grouped by domain with icons
- Mandatory requirements marked with `*`
- Inline CSS for all maturity level colours

**Dashboard** (`render_dashboard_html`):
- Go-live readiness banner (green / red)
- Overall score (large percentage + B11 + progress bar)
- Per-domain score cards (progress bar + status badge + top gap actions)
- Priority action table (P0→P3, sorted)
- Per-domain requirement detail table (maturity badge + B11 + remediation action)
- Footer with generated epoch + ADR citation

## WAYv4.0 Policy Integration

The SLA engine is governed by `policies/sla_governance_protocol.akk`:

- `GoLiveApproval` policy — enforces B11 ≥ 180 and no mandatory gaps
- `MandatoryGapDetected` alarm — notifies approvers when P0 gaps appear
- `SlaApiFirewall` — protects the GUI endpoints via hepta-sec-web
- `SlaReportFreshness` guard — warns if report is older than 24 hours

## Constants

| Constant              | Value | Description                          |
|-----------------------|-------|--------------------------------------|
| `QUALITY_DIVISOR`     | 240   | B11 scale maximum (ADR-001)          |
| `GO_LIVE_THRESHOLD`   | 180   | 75% — minimum for go-live            |
| `MANDATORY_THRESHOLD` | 192   | 80% — mandatory requirement minimum  |

## Dependencies

- `bahyway-core` — `BahyWayError`, common types
- `hepta-sec-policy` — optional policy rule integration
- `hepta-sec-firewall` — optional firewall verdict integration
- No external crates. No unsafe code.

## See Also

- `crates/pii-vault/MANUAL.md` — encrypted PII fields (req 2001, 5001)
- `crates/hepta-sec-web/MANUAL.md` — HTTP boundary guard (req 3001)
- `crates/hepta-sec-policy/MANUAL.md` — rate limiting (req 3002)
- `policies/sla_governance_protocol.akk` — WAYv4.0 governance rules
