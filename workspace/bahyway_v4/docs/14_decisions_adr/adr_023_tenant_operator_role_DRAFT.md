# ADR-023 — Tenant Operator Role for Client OTAP Access (DRAFT, unsealed)

**Status:** DRAFT — awaiting Architect ratification (CSR-08).
**Date:** 2026-08-01
**Author:** Claude (drafting on the Architect's instruction), NOT the
Architect.

## The question (Q3, this morning)

*"Should I let the client have Sargon Passport access to Shakkanakku and
Uruinimgina to run its own PBs in its own OTAP and ingest its own
EnkiDDB data?"*

## Answer proposed here: yes, as a new "Tenant Operator" tier — with real, stated limitations

Built today: `playbooks/tasks/require_tenant_operator.yml`, a gate a
client's own playbook runs would `import_tasks` first, analogous to
`require_gilgamesh_for_production.yml` but scoped to a tenant rather
than to BahyWay's own Production. It requires, fails closed on every
path:

1. An explicit `tenant` name (never defaults, never `bahyway` — that
   name is reserved for the ecosystem-internal instance).
2. An explicit `tenant_target_env` (the tenant's OWN Dev/Test/
   Acceptance/Production — theirs to reach, not BahyWay's).
3. An explicit `tenant_allowed_hosts` list, hard-checked to never
   contain `enkidb-node-write`/`enkidb-node-read` (BahyWay's own real
   CQRS pair, 192.168.122.101/.107).
4. A real cryptographic check — `kupru-vault-cli check --min-privilege
   2` against a vault file named `tenant_<name>_vault.dat` — the same
   Ed25519-seal verification every other gate in this repo uses, not a
   new or weaker crypto path.

## Honest limitation, stated plainly rather than glossed over

`SargonPassport` (`crates/kupru/src/passport.rs`) has a `privilege_level`
today but **no tenant field**. So "this passport belongs to tenant acme"
is enforced today by a **naming convention** — the vault file must be
named `tenant_acme_vault.dat` — not by anything inside the passport's
own cryptographic seal. The privilege check itself (Ed25519, real) is as
strong as every other gate in this repo. The tenant *binding* is not: an
operator with filesystem access could rename or copy a vault file and
present it under a different tenant's name, and this gate would accept
it as if it were genuinely that tenant's own passport.

**This is real for keeping honest tenants in their own lane** (the
actual, expected failure mode — someone's automation accidentally
pointed at the wrong host set) but is **not a defense against a
malicious tenant deliberately trying to impersonate another one.** If
that threat model matters — and for a real multi-client deployment, it
likely does — the durable fix is adding a real `tenant_id` field to
`SargonPassport`'s own sealed structure, checked cryptographically
alongside `privilege_level`. That's a bigger, cross-cutting change
(touches the Sargon Passport Manager and Gilgamesh Master Key Godot
tools, `kupru-vault`, `web_auth.rs` — every place that already
constructs or verifies a passport) and needs the Architect's explicit
go-ahead before touching a primitive this many working tools already
depend on. Not done here; flagged, not silently deferred.

## `tenant_min_privilege: 2`

Deliberately below BahyWay's own internal `Developer=3` tier
(`playbook_268_bahyway_host_privilege_groups.yml`'s own 5-group model:
Architect=7, DataSteward=6, Administrator=5, Developer=3,
Stakeholder=1). A Tenant Operator passport can never be mistaken for
internal BahyWay staff privilege by a check that only reads the number —
the two scales are disjoint by construction, not just by convention.

## Open questions for the Architect (not decided here)

1. Should Tenant Operator passports be minted by the client themselves
   (via a copy of the Sargon Passport Manager tool BahyWay ships them),
   or minted BY BahyWay and handed to the client?
2. Does every tenant need their own full CQRS infrastructure (per
   ADR-022's proposal), or can smaller clients share infrastructure
   with tenant-level isolation some other way?
3. Is `tenant_min_privilege=2` the right floor, or should it vary by
   what the tenant is authorized to do (e.g. read-only ingestion vs.
   full PB execution)?

## Status

Unsealed. The gate mechanism is real and built; the *policy* choices
above are not decided by this document.
