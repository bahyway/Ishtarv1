# ADR-005 — Enterprise Data Fabric as Sovereign Layer 8

> **DubSar Help** | `ADR > 005` | Architecture Decisions

<!--META
doc_kaki:
  concept_hash:   "0x00000000"
  tribe:          "DT-14"
  concept_type:   "0x02"
  epoch:          "2026-01-01"
  concept_depth:  200
  riksu_count:    1
  snapshot_epoch: "2026-06-06"

concept:          "Enterprise Data Fabric"
summary:          "Enterprise Data Fabric is the sovereign Layer 8 integration mesh — no third-party data substrates permitted."
sovereign_laws:   []

riksu_bindings:
  - target: "adr_008_ooo_foundation_kaki_roles_forbidden_operations.md"
    concept: "OOO Layer 8"
    type: "CHILD"

orbit_tags:       ["OOO Mathematical Foundation"]
rag_keywords:     ["ORBIT", "PROBE", "sovereign fabric", "data mesh", "Layer 8"]
-->

## Status: Accepted

## Context

BahyWay.Ecosystem v4.0 includes powerful physics engines (AMMAS, VGCA,
Hepta-Score), a sovereign identity system (KAKI), and a complete storage
stack (EnkiDB). However, the bridge between external enterprise data sources
and the sovereign ecosystem was ad-hoc — each domain engine (NajafEngine,
NuskuEngine, WPDEngine) implemented its own extraction and routing logic.

This produced exactly the conditions shown in enterprise spaghetti diagrams:
- No shared schema enforcement across sources
- No common lineage model
- Exceptions handled differently in each engine (some silenced entirely)
- Adding a new source required modifying existing engine code

The ecosystem lacked a **unified entry layer** that could accept data from
any of the eight common enterprise source types and route it to any of the
seven common target types with consistent traceability and quality enforcement.

## Decision

A new crate, `bahyway-fabric`, is introduced at Layer 8 as the sovereign
Enterprise Data Fabric. It provides:

1. **`SchemaContract`** — typed field declarations enforced at every source and
   target boundary. A connector that produces undeclared fields, or omits
   required ones, cannot silently pass data downstream.

2. **`SourceConnector` / `TargetConnector` traits** — the only legal attachment
   point for external systems. Adding a new source or target requires only
   implementing the relevant trait. No existing pipeline, stage, or orchestrator
   code is modified.

3. **`PipelineDeclaration`** — a named, versioned struct declaring source,
   ordered stages, targets, and exception policies. Pipelines are data, not
   code — they can be registered, replaced, and versioned independently.

4. **`LineageChain`** — an append-only, per-record audit trail. Every stage
   records its FNV-1a input hash, output hash, quality before, and quality
   after. The chain answers "where does this data come from?" with cryptographic
   certainty about what each stage received and produced.

5. **`FabricException`** — a typed exception enum with seven variants. Exceptions
   carry source, stage, message, payload, and epoch. They cannot be silenced —
   every failure is returned in `OrchestratorResult.exceptions` and must be
   explicitly handled by the caller.

6. **`FabricOrchestrator`** — the single coordinator. It holds registries of
   sources, targets, and pipelines, and executes `run_pipeline` with full
   lineage and exception collection.

## Consequences

**Positive:**
- Enterprise spaghetti is structurally impossible — the connector trait and
  schema contract enforce boundaries at compile time and runtime.
- Any new source or target can be added in minutes with zero impact on existing
  pipelines.
- Lineage chains provide a complete audit trail for data stewardship, compliance,
  and debugging without any additional instrumentation.
- Exception handling is uniform across all pipelines — no more per-engine
  exception silencing.

**Constraints introduced:**
- All data entering EnkiDB from external systems must flow through
  `bahyway-fabric`. Direct engine-to-storage writes bypass lineage and are
  not permitted for externally-sourced data.
- `SchemaContract` must be declared and kept current as source schemas evolve.
  Stale contracts will reject valid records until updated.
- `PipelineDeclaration` versions must be bumped when stages or routing change —
  in-place modification of a registered pipeline is not permitted during a
  pipeline run.

**Neutral:**
- Built-in adapters are stubs. Production deployments must wire real I/O in
  `extract()` and `deliver()` implementations. The Fabric never changes when
  doing so.
- `bahyway-fabric` has no I/O of its own — all I/O lives in connector
  implementations. This is by design: the Fabric governs, connectors act.

## Relationship to Previous ADRs

| ADR | Interaction |
|---|---|
| ADR-001 (No External DB) | Fabric stores nothing itself — all persistence goes through `adad-gate` → `enkidb-journal` |
| ADR-003 (KAKI Sovereignty) | Every record accepted by the Fabric is immediately minted a `IdentityKaki` via `AdadGate` |
| ADR-004 (BeeMDM 4-lane pipeline) | `bahyway-fabric` is the upstream entry point that feeds BeeMDM — it is the "before the pipeline" layer |
