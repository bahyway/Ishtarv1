# How To: Declare a Pipeline

> **DubSar Help** | `How-To > Pipelines` | bahyway-fabric

## When to use this guide

You have a source and one or more targets already registered, and you want to
create a data flow between them — with quality enforcement, lineage tracking,
and structured exception handling — without writing any routing code.

**Time to complete:** 5–15 minutes.

---

## Anatomy of a PipelineDeclaration

```rust
PipelineDeclaration::builder("pipeline-id", "source-id")
    .version(1)                           // bump when stages or routing change
    .description("Human description")     // shown in Dubsar pipeline map
    .stage(Stage::Cleanse)                // ordered — runs left to right
    .stage(Stage::Validate { min_b11: 140 })
    .stage(Stage::Deduplicate)
    .stage(Stage::Enrich { ruleset: "my.ruleset.v1" })
    .target("dw.central")                 // one or more targets
    .target("notify.sovereign")
    .on_exception(ExceptionRule { ... })  // optional — default: RejectRecord
    .build()
```

---

## Choosing the Right Stages

### Stage::Cleanse
Use when: the source may produce trailing whitespace, empty strings, or
mixed-encoding values. Always put Cleanse first.

```rust
.stage(Stage::Cleanse)
```

### Stage::Validate { min_b11 }
Use when: you need a quality gate. Records below `min_b11` are rejected.

| Lane | min_b11 | When to use |
|---|---|---|
| GEM | 200 | Master data, canonical records only |
| TRIBE | 140 | Standard enterprise data (recommended default) |
| ACTIVE | 100 | Noisy sources where some data loss is acceptable |
| FUZZY | 60 | Analytics-only, quality issues expected |

```rust
.stage(Stage::Validate { min_b11: 140 })  // TRIBE lane — enterprise default
```

### Stage::Deduplicate
Use when: the source may produce the same logical record more than once
(common in ERP batch exports, email inbox polling, and legacy system dumps).

```rust
.stage(Stage::Deduplicate)
// → delegates to idu-prober cross-tribe identity resolution
```

### Stage::Enrich { ruleset }
Use when: records need lookup-based augmentation — adding a country name from
a code, computing a derived score, or appending an Arabic verdict via
ShulmanEngine.

```rust
.stage(Stage::Enrich { ruleset: "erp.enrichment.v1" })
// → ruleset identifier resolved by shulman-engine / story-engine at runtime
```

### Stage::Aggregate { key_attr_hash, strategy }
Use when: you need batch-level reduction before delivery.

```rust
.stage(Stage::Aggregate {
    key_attr_hash: 0x1001,                    // group by this field
    strategy: AggregationStrategy::Sum,       // Sum | Count | LastWins | Max | Min
})
```

### Stage::Transform { transform_id }
Use when: you have a registered custom transformation function.

```rust
.stage(Stage::Transform { transform_id: "erp.currency.normalise" })
// → transform_id resolved by the runtime transform registry
```

---

## Exception Policies

### Default (no .on_exception call)
Records that fail are rejected; the batch continues.

### Dead-letter queue
```rust
.on_exception(ExceptionRule {
    applies_to: vec![ExceptionKind::QualityRejection, ExceptionKind::MissingRequiredField],
    policy: ExceptionPolicy::DeadLetter { queue_id: "dlq.quality" },
})
```

### Halt on critical failure
```rust
.on_exception(ExceptionRule {
    applies_to: vec![ExceptionKind::InternalFault],
    policy: ExceptionPolicy::HaltPipeline,
})
```

### Retry with fallback
```rust
.on_exception(ExceptionRule {
    applies_to: vec![ExceptionKind::DeliveryFailure],
    policy: ExceptionPolicy::Retry {
        max_retries: 3,
        fallback: Box::new(ExceptionPolicy::DeadLetter { queue_id: "dlq.delivery" }),
    },
})
```

---

## Versioning Pipelines

When a pipeline's stages or routing must change, **do not modify the existing
declaration in place**. Register a new version:

```rust
// Old registration (v1) remains unchanged during transition
let v2 = PipelineDeclaration::builder("erp.invoices → dw", "erp.sovereign")
    .version(2)                            // always bump version
    .stage(Stage::Cleanse)
    .stage(Stage::Validate { min_b11: 140 })
    .stage(Stage::Enrich { ruleset: "erp.enrichment.v2" })  // updated ruleset
    .stage(Stage::Deduplicate)
    .target("dw.central")
    .build();

fabric.register_pipeline(v2);             // replaces v1 atomically
```

EriduScheduler will use the new declaration on the next scheduled run.
In-flight runs using v1 complete normally.

---

## Complete Example: ERP to Data Warehouse

```rust
use bahyway_fabric::prelude::*;

let mut fabric = FabricOrchestrator::new();
fabric.register_source(Box::new(ErpConnector));
fabric.register_target(Box::new(DataWarehouseTarget));
fabric.register_target(Box::new(NotificationTarget));

let pipeline = PipelineDeclaration::builder("erp.all → dw+notify", "erp.sovereign")
    .version(1)
    .description("Full ERP ingest: cleanse → quality gate → dedup → enrich → DW + notify")
    .stage(Stage::Cleanse)
    .stage(Stage::Validate { min_b11: 140 })
    .stage(Stage::Deduplicate)
    .stage(Stage::Enrich { ruleset: "erp.sovereign.v1" })
    .target("dw.central")
    .target("notify.sovereign")
    .on_exception(ExceptionRule {
        applies_to: vec![ExceptionKind::QualityRejection],
        policy: ExceptionPolicy::DeadLetter { queue_id: "dlq.erp.quality" },
    })
    .on_exception(ExceptionRule {
        applies_to: vec![ExceptionKind::DeliveryFailure],
        policy: ExceptionPolicy::Retry {
            max_retries: 3,
            fallback: Box::new(ExceptionPolicy::DeadLetter { queue_id: "dlq.erp.delivery" }),
        },
    })
    .build();

fabric.register_pipeline(pipeline);
let result = fabric.run_pipeline(
    &PipelineId("erp.all → dw+notify"),
    &ExtractionCursor::default(),
).unwrap();
assert!(result.success_rate() > 0.95);
```

---

## Checklist

- [ ] `pipeline-id` is unique across all registered pipelines
- [ ] `source-id` matches a registered `SourceConnector`
- [ ] All `target-id`s match registered `TargetConnector`s
- [ ] `Stage::Cleanse` is first (if used)
- [ ] `Stage::Validate` `min_b11` matches the quality lane appropriate for the data
- [ ] `version` is bumped whenever stages or targets change
- [ ] Exception policies are declared for all expected failure kinds

## See Also

- `15_howto/connect_a_new_source.md` — Register a SourceConnector
- `17_troubleshooting/fabric_exception_diagnosis.md` — Debug pipeline failures
- `crates/bahyway-fabric/MANUAL.md` — Full stage and policy reference
