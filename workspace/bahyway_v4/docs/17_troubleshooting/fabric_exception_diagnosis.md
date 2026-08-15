# Troubleshooting: Fabric Exception Diagnosis

> **DubSar Help** | `Troubleshooting > Fabric` | bahyway-fabric

## Overview

`bahyway-fabric` never silences exceptions. Every failure in
`OrchestratorResult.exceptions` has a typed `ExceptionKind`, a stage name,
an epoch, and (for record-level failures) the raw payload that triggered it.

This guide walks through each exception kind, its likely cause, and its remedy.

---

## Diagnostic Pattern

```rust
let result = fabric.run_pipeline(&id, &cursor).unwrap();

if !result.is_clean() {
    println!("SUCCESS RATE: {:.1}%", result.success_rate() * 100.0);
    for ex in &result.exceptions {
        println!(
            "[{:?}] stage={} epoch={}\n  message: {}\n  payload fields: {}",
            ex.kind, ex.stage, ex.epoch, ex.message,
            ex.payload.len()
        );
    }
}
```

---

## ExceptionKind::SchemaViolation

**Symptom:** Source produced a field not declared in its `SchemaContract`.

**Diagnosis:**
```
[SchemaViolation] stage=source-boundary epoch=42
  message: field X unknown
```

**Cause:** The external source added a new field without the connector's
schema being updated, OR the source sent data from a different schema version.

**Remedy:**
1. Identify the undeclared `attr_hash` in the payload.
2. Add it to the connector's `schema()` as an `FieldSpec::optional(...)`.
3. Bump the contract `version`.
4. Re-register the connector.

---

## ExceptionKind::MissingRequiredField

**Symptom:** A required field was absent from a record.

**Diagnosis:**
```
[MissingRequiredField] stage=source-boundary epoch=42
  message: Required field 'amount' absent
```

**Cause:** The source system sent incomplete records — common during partial
exports, system migrations, or soft-deletes that zero out fields.

**Remedy:**
- If the field is legitimately optional sometimes: change it from
  `FieldSpec::required` to `FieldSpec::optional` in the contract.
- If the source is misbehaving: fix the source extract logic or add a
  pre-extraction filter that rejects incomplete records at the source.
- Route to dead-letter queue for human review:
  ```rust
  .on_exception(ExceptionRule {
      applies_to: vec![ExceptionKind::MissingRequiredField],
      policy: ExceptionPolicy::DeadLetter { queue_id: "dlq.incomplete" },
  })
  ```

---

## ExceptionKind::QualityRejection

**Symptom:** Record's computed B11 was below the pipeline's `min_b11` threshold.

**Diagnosis:**
```
[QualityRejection] stage=hepta-score epoch=42
  message: B11=55 below quality threshold 140
```

**Cause:** The record has too many empty or low-quality fields. B11 is computed
as `(non_empty_fields / total_fields) × 240`. B11=55 means roughly 23% of
fields carry data.

**Remedies (choose based on data quality policy):**

| Option | When to use |
|---|---|
| Dead-letter to `dlq.quality` | Data is salvageable; human steward reviews |
| Lower `min_b11` threshold | Source is legitimately sparse (e.g. legacy system) |
| Add `Stage::Cleanse` before Validate | Empty-string fields may be cleanable |
| Fix the source | Source is exporting incomplete records due to a bug |

**Check quality distribution:**
```rust
// B11 lane thresholds for reference:
// GEM    ≥ 200  — ideal
// TRIBE  ≥ 140  — enterprise standard
// ACTIVE ≥ 100  — acceptable
// FUZZY  ≥  60  — monitored
// DEAD   <  60  — rejected
```

---

## ExceptionKind::DuplicateIdentity

**Symptom:** `idu-prober` detected two records with the same logical identity.

**Diagnosis:**
```
[DuplicateIdentity] stage=deduplicate epoch=42
  message: conflicting KAKI for entity erp_id=1001
```

**Cause:** The source sent the same logical entity twice — common in:
- ERP batch exports that include both current and historical records
- Polling sources where the same event appears in two consecutive extraction windows

**Remedies:**

| Option | When to use |
|---|---|
| Accept `LastWins` — keep the more recent | Source always sends the latest version |
| Route both to `dlq.duplicates` | Need human merging decision |
| Use `Stage::Aggregate { strategy: LastWins }` | Standard dedup by latest record |

---

## ExceptionKind::DeliveryFailure

**Symptom:** Target refused the batch.

**Diagnosis:**
```
[DeliveryFailure] stage=dw.deliver epoch=42
  message: connection timeout after 30s
```

**Cause:** Target system is unavailable, over capacity, or the authentication
token has expired.

**Remedy:**
1. Check target system health.
2. Configure retry policy:
   ```rust
   .on_exception(ExceptionRule {
       applies_to: vec![ExceptionKind::DeliveryFailure],
       policy: ExceptionPolicy::Retry {
           max_retries: 4,
           fallback: Box::new(ExceptionPolicy::DeadLetter { queue_id: "dlq.delivery" }),
       },
   })
   ```
3. Inspect `DeliveryReceipt.token` from successful deliveries to verify which
   batches reached the target before the failure.

---

## ExceptionKind::TransformError

**Symptom:** A custom `Stage::Transform` returned an unexpected output shape.

**Diagnosis:**
```
[TransformError] stage=transform epoch=42
  message: transform=erp.currency.normalise returned 0 fields
```

**Cause:** The transform function has a bug or its input assumptions changed
(e.g. a currency field changed from ISO code to numeric code).

**Remedy:**
1. Unit-test the transform function independently.
2. Review the input record in `ex.payload`.
3. Fix the transform or update its input expectations.
4. Bump the `transform_id` string to signal the update to dependent pipelines.

---

## ExceptionKind::ExtractionError

**Symptom:** Source connection failed before any records were extracted.

**Diagnosis:**
```
[ExtractionError] stage=extraction epoch=0
  message: HTTP 401 Unauthorized from erp.sovereign
```

**Cause:** API key rotated, endpoint changed, or network unavailable.

**Note:** When extraction fails, `records_in = 0` and no lineage chains are
built. The entire `OrchestratorResult` will have `records_in = 0, records_out = 0`.

**Remedy:**
1. Check credential rotation schedule.
2. Verify endpoint in connector configuration.
3. Check network connectivity to the source system.
4. Re-run with updated credentials.

---

## ExceptionKind::InternalFault

**Symptom:** Logic error inside the Fabric itself.

**Diagnosis:**
```
[InternalFault] stage=orchestrator epoch=42
  message: stage dispatch returned unexpected None
```

**Cause:** This exception kind should never occur in a correct production
deployment. It indicates a bug in a custom `Stage::Transform` implementation
or an internal orchestrator inconsistency.

**Remedy:**
1. File a sovereign audit event immediately.
2. Capture the full `OrchestratorResult` for debugging.
3. Isolate the pipeline and halt it with `pipeline.enabled = false`.
4. Report to the ecosystem architect with full exception payload.

---

## Lineage-Based Debugging

When an exception's payload alone is not enough, use the lineage chain of
successfully processed records to understand the data shape:

```rust
// Find what the cleanse stage produced for a passing record
if let Some(chain) = result.lineage.first() {
    for hop in &chain.hops {
        if hop.stage.0 == "cleanse" {
            println!("Cleanse input:  {:x}", hop.input_hash);
            println!("Cleanse output: {:x}", hop.output_hash);
            println!("Note: {}", hop.annotation.as_deref().unwrap_or("—"));
        }
    }
}
```

Comparing `input_hash` to `output_hash` across stages shows exactly which
stage modified the record — or confirms a stage passed data through unchanged.

---

## See Also

- `08_pipeline_alaktu/bahyway_fabric.md` — Fabric pipeline architecture
- `15_howto/connect_a_new_source.md` — Fix source connector issues
- `15_howto/declare_a_pipeline.md` — Fix pipeline declaration issues
- `crates/bahyway-fabric/MANUAL.md` — Full exception reference
