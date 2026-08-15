# How To: Connect a New Data Source

> **DubSar Help** | `How-To > Sources` | bahyway-fabric

## When to use this guide

You have a new data system — a second ERP vendor, an IoT sensor feed, a
government data API — and you want it to flow into BahyWay.Ecosystem with full
lineage, schema enforcement, and quality scoring.

**Time to complete:** 15–30 minutes for a stub; 1–4 hours for production I/O.

---

## Step 1: Define the SchemaContract

Declare which fields your source produces and which are required:

```rust
use bahyway_fabric::contract::{FieldSpec, FieldType, SchemaContract};

fn my_source_schema() -> SchemaContract {
    SchemaContract::new("my_system.record", 1, vec![
        // Required fields — records missing these will surface as FabricException
        FieldSpec::required("record_id",    0xA001, FieldType::Integer),
        FieldSpec::required("timestamp",    0xA002, FieldType::Timestamp),
        FieldSpec::required("value",        0xA003, FieldType::Decimal),
        // Optional fields — absence is allowed
        FieldSpec::optional("description",  0xA004, FieldType::Text),
        FieldSpec::optional("region_code",  0xA005, FieldType::Text),
    ])
}
```

**Attr hash convention:** Choose a unique `u32` prefix range for your system
(e.g. `0xA000..0xAFFF` for "my system"). Do not reuse hashes from other
systems' contracts.

---

## Step 2: Implement SourceConnector

```rust
use bahyway_fabric::connector::{
    DataBatch, ExtractionCursor, SourceConnector, SourceId,
};
use bahyway_fabric::exception::FabricException;
use bahyway_fabric::contract::SchemaContract;

pub struct MySystemConnector {
    endpoint: &'static str,
    api_key:  String,
}

impl SourceConnector for MySystemConnector {
    fn source_id(&self) -> SourceId {
        SourceId("my_system.sovereign")  // unique, stable, lowercase.dotted
    }

    fn display_name(&self) -> &'static str {
        "My System — Production"
    }

    fn schema(&self) -> SchemaContract {
        my_source_schema()
    }

    fn extract(&self, cursor: &ExtractionCursor) -> Result<DataBatch, FabricException> {
        // 1. Connect to your system using self.endpoint + self.api_key
        // 2. Fetch records since cursor.last_epoch
        // 3. Convert each record to Vec<(u32, Vec<u8>)> — attr_hash to raw bytes
        // 4. Return DataBatch

        // Example production pattern:
        // let raw = http_get(self.endpoint, cursor.last_epoch, &self.api_key)
        //     .map_err(|e| FabricException::extraction_error(
        //         self.source_id(), e.to_string(), cursor.last_epoch,
        //     ))?;
        // let records = raw.iter().map(|r| serialize_record(r)).collect();
        // Ok(DataBatch::new(self.source_id(), records, current_epoch()))

        // Stub — replace with real I/O:
        Ok(DataBatch::new(self.source_id(), vec![], cursor.last_epoch))
    }
}
```

**Record serialisation convention:**
- Each field → `(attr_hash, value_as_bytes)` pair
- Integers: `i64::to_le_bytes()`
- Decimals: represent as scaled integer bytes or ASCII decimal string
- Text: UTF-8 bytes
- Timestamps: Unix seconds as `u64::to_le_bytes()`
- Absent/null optional fields: simply omit the pair from the record

---

## Step 3: Register the connector

```rust
let mut fabric = FabricOrchestrator::new();
fabric.register_source(Box::new(MySystemConnector {
    endpoint: "https://my-system.internal/api/v2",
    api_key:  std::env::var("MY_SYSTEM_API_KEY").unwrap(),
}));
```

The connector is now live. No existing pipeline or connector is affected.

---

## Step 4: Declare a pipeline that uses it

```rust
use bahyway_fabric::pipeline::{PipelineDeclaration, Stage};

let pipeline = PipelineDeclaration::builder("my_system → dw", "my_system.sovereign")
    .description("Ingest My System records, validate quality, load to Data Warehouse")
    .stage(Stage::Cleanse)
    .stage(Stage::Validate { min_b11: 100 })  // ACTIVE lane minimum
    .stage(Stage::Deduplicate)
    .target("dw.central")
    .build();

fabric.register_pipeline(pipeline);
```

---

## Step 5: Run and inspect

```rust
use bahyway_fabric::connector::ExtractionCursor;
use bahyway_fabric::pipeline::PipelineId;

let result = fabric.run_pipeline(
    &PipelineId("my_system → dw"),
    &ExtractionCursor::default(),
).unwrap();

println!("Records in:  {}", result.records_in);
println!("Records out: {}", result.records_out);
println!("Exceptions:  {}", result.exceptions.len());
for ex in &result.exceptions {
    println!("  [{:?}] {}", ex.kind, ex.message);
}
for chain in &result.lineage {
    println!("{}", chain.report());
}
```

---

## Checklist

- [ ] `attr_hash` values are unique within your system and do not collide with
      existing adapters in `adapters.rs`
- [ ] `source_id()` returns a stable, unique, lowercase dotted identifier
- [ ] `extract()` returns `FabricException::extraction_error()` on I/O failure
      (never panics, never silences errors)
- [ ] Required fields in `schema()` match what the source guarantees — not what
      it sometimes produces
- [ ] The connector is registered before any pipeline that references its
      `SourceId` is run

## See Also

- `15_howto/declare_a_pipeline.md` — Write a full pipeline declaration
- `17_troubleshooting/fabric_exception_diagnosis.md` — Diagnose extraction errors
- `crates/bahyway-fabric/MANUAL.md` — Full module reference
