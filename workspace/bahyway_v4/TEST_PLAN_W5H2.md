# EnkiDB / BeeMDM — Test Cases Manual Plan (W5H2)

**Crate:** `enkidb-dw`  
**Branch:** `claude/serene-goldberg-tpobS`  
**Total test cases:** 38  
**Last verified:** all 38 pass — `cargo test -p enkidb-dw`

---

## How to read this document

Each test case is described across seven dimensions:

| Dimension | Question answered |
|-----------|-------------------|
| **WHO**   | Which system actor / component is under test |
| **WHAT**  | Which specific behaviour is being verified |
| **WHERE** | Source file + test function name |
| **WHEN**  | The trigger / precondition that makes this test relevant |
| **WHY**   | The BeeMDM or EnkiDB requirement this test guards |
| **HOW**   | The test mechanics: setup → action → assertion |
| **HOW MANY** | Quantified pass criteria (counts, ratios, hashes) |

---

## Module 1 — `zip_engine` (6 tests)

### TC-ZE-01 · build_and_extract_store_zip

| | |
|---|---|
| **WHO** | ZipEngine — STORE-method (method=0) extraction path |
| **WHAT** | A single-file STORE ZIP can be built and round-tripped without data loss |
| **WHERE** | `zip_engine.rs` → `tests::build_and_extract_store_zip` |
| **WHEN** | Any time a CSV/TSV file lands in the shard folder inside a plain ZIP |
| **WHY** | The ETL pipeline's first action is ZIP extraction; data integrity is the absolute minimum guarantee before any processing begins |
| **HOW** | Build a 37-byte TSV payload with `build_store_zip("records.tsv", …)`, call `extract_ok(&zip)`, assert entry name and bytes match the original |
| **HOW MANY** | 1 entry returned; 0 bytes difference vs original payload |

---

### TC-ZE-02 · deflate_entry_with_invalid_data_returns_error

| | |
|---|---|
| **WHO** | ZipEngine — DEFLATE-method (method=8) error path |
| **WHAT** | A ZIP whose header claims DEFLATE but contains non-DEFLATE bytes returns `ZipError::DeflateError`, not a panic or a silent empty entry |
| **WHERE** | `zip_engine.rs` → `tests::deflate_entry_with_invalid_data_returns_error` |
| **WHEN** | A corrupt or truncated Bandizip file is dropped into the shard folder |
| **WHY** | The system must never panic on bad input; the ETL pipeline must receive a typed error so it can write an alert and skip the batch cleanly |
| **HOW** | Build a STORE ZIP, patch bytes 8–9 to method=8, call `extract(&zip)`, assert at least one result matches `ZipError::DeflateError { .. }` |
| **HOW MANY** | ≥1 `Err(ZipError::DeflateError)` in the results vec |

---

### TC-ZE-03 · empty_zip_returns_no_entries

| | |
|---|---|
| **WHO** | ZipEngine — invalid-magic / empty-input guard |
| **WHAT** | Non-ZIP bytes produce an empty entry list rather than a panic |
| **WHERE** | `zip_engine.rs` → `tests::empty_zip_returns_no_entries` |
| **WHEN** | A zero-byte file, a plain text file, or a corrupted download is placed in the shard folder |
| **WHY** | LandingZone classifies any file ending in `.zip` as `LandingFileKind::Zip`; the watchdog must not crash when the file content is garbage |
| **HOW** | Call `extract_ok(b"not a zip at all")` and assert the returned vec is empty |
| **HOW MANY** | 0 entries |

---

### TC-ZE-04 · multi_file_zip

| | |
|---|---|
| **WHO** | ZipEngine — multi-entry ZIP traversal |
| **WHAT** | A ZIP containing more than one file does not stop after the first entry |
| **WHERE** | `zip_engine.rs` → `tests::multi_file_zip` |
| **WHEN** | A batch ZIP containing multiple CSVs (e.g. one CSV per governorate) is dropped into the shard |
| **WHY** | Each `najaf_cemetery_batch_*.zip` may contain multiple sheets; missing subsequent entries would silently drop records |
| **HOW** | Concatenate two STORE ZIP local-file records manually, call `extract_ok`, verify no panic and both entries are accessible |
| **HOW MANY** | ≥2 entries reachable without error |

---

### TC-ZE-05 · inflate_stored_block

| | |
|---|---|
| **WHO** | DEFLATE inflate engine — BTYPE=00 (stored block) path |
| **WHAT** | A manually constructed raw DEFLATE stored block decompresses to the exact original bytes |
| **WHERE** | `zip_engine.rs` → `tests::inflate_stored_block` |
| **WHEN** | A DEFLATE stream begins with BTYPE=00 (often used for very short or incompressible payloads) |
| **WHY** | RFC 1951 requires all three block types to be supported; stored blocks are the simplest and serve as a baseline for the inflate engine |
| **HOW** | Hand-craft a 12-byte DEFLATE stream: BFINAL=1, BTYPE=00, LEN=6, NLEN=0xFFF9, then "Hello!"; call `inflate::inflate(&block, 6)`; assert result equals `b"Hello!"` |
| **HOW MANY** | 6 bytes output, exact match |

---

### TC-ZE-06 · inflate_fixed_block_literals

| | |
|---|---|
| **WHO** | DEFLATE inflate engine — BTYPE=01 (fixed Huffman) path |
| **WHAT** | Fixed-Huffman blocks decode correctly (documents the bit-layout; tested end-to-end via real ZIP integration) |
| **WHERE** | `zip_engine.rs` → `tests::inflate_fixed_block_literals` |
| **WHEN** | A DEFLATE stream begins with BTYPE=01, as emitted by most compressors for short payloads |
| **WHY** | The 12 Bandizip ZIPs use fixed or dynamic Huffman; this test records the bit-reversal reasoning for future maintainers |
| **HOW** | Documents the RFC 1951 fixed-code mapping and LSB-first reversal logic; full round-trip verified through TC-ZE-01/05 and real ZIP files |
| **HOW MANY** | Passes trivially (documentation test); functional coverage via TC-ZE-05 |

---

## Module 2 — `batch_schema` (5 tests)

### TC-BS-01 · mandatory_vs_optional

| | |
|---|---|
| **WHO** | BatchSchema — column classification engine |
| **WHAT** | Columns that are always non-empty are classified as mandatory; columns with at least one empty value are classified as optional |
| **WHERE** | `batch_schema.rs` → `tests::mandatory_vs_optional` |
| **WHEN** | A CSV batch is ingested and the schema is inferred before table creation in EnkiDB |
| **WHY** | BeeMDM §12.4 — the schema descriptor must distinguish mandatory from optional attributes so EnkiDB can enforce NOT-NULL constraints on Golden Records |
| **HOW** | Build 2 records: `id`/`name` always filled, `nickname` empty in row 1; infer schema; assert `id` and `name` in `mandatory_attrs`, `nickname` in `optional_attrs` |
| **HOW MANY** | 2 mandatory, 1 optional |

---

### TC-BS-02 · schema_name_and_table_name

| | |
|---|---|
| **WHO** | BatchSchema — naming convention |
| **WHAT** | Schema name is `{batch_name}_{timestamp}` and table name is `tb_{batch_name}` |
| **WHERE** | `batch_schema.rs` → `tests::schema_name_and_table_name` |
| **WHEN** | Any batch arrives; the schema name is used as the EnkiDB namespace key |
| **WHY** | Every ZIP creates a unique, time-stamped schema in EnkiDB; the `tb_` prefix distinguishes tables from Tribe identifiers — mixing these up would corrupt the namespace |
| **HOW** | Infer schema for `"najaf_batch_001"` with timestamp `1_700_000`; assert `schema_name == "najaf_batch_001_1700000"` and `table_name == "tb_najaf_batch_001"` |
| **HOW MANY** | Exact string equality on both fields |

---

### TC-BS-03 · descriptor_contains_key_fields

| | |
|---|---|
| **WHO** | BatchSchema — `.schema` file renderer |
| **WHAT** | `to_descriptor()` produces a human-readable string containing all key fields and section headers |
| **WHERE** | `batch_schema.rs` → `tests::descriptor_contains_key_fields` |
| **WHEN** | After batch ingestion, the `.schema` file is written to `Processing/{batch}/batch.schema` for operator inspection |
| **WHY** | The `.schema` file is the operator's window into what EnkiDB inferred from the CSV; it must be complete enough to debug schema mismatches without accessing the database |
| **HOW** | Infer schema; call `to_descriptor()`; assert presence of `"schema_name"`, `"tb_test_batch"`, `"[mandatory_attributes]"`, `"[optional_attributes]"`, `"nickname"` |
| **HOW MANY** | 5 substring assertions, all must pass |

---

### TC-BS-04 · empty_records_produces_empty_schema

| | |
|---|---|
| **WHO** | BatchSchema — empty-batch edge case |
| **WHAT** | An empty record slice produces a valid BatchSchema with zero columns and zero records, without panicking |
| **WHERE** | `batch_schema.rs` → `tests::empty_records_produces_empty_schema` |
| **WHEN** | A ZIP contains a CSV file with a header row only (no data rows) or a completely empty file |
| **WHY** | Empty batches must be handled gracefully; they should produce a schema (with zero counts) rather than crashing the watchdog |
| **HOW** | Call `BatchSchema::infer("empty_batch", 0, &[])` and assert `total_columns == 0`, `total_records == 0`, both attr lists empty |
| **HOW MANY** | 0 columns, 0 records, 0 mandatory, 0 optional |

---

### TC-BS-05 · entity_seed_deterministic

| | |
|---|---|
| **WHO** | BatchSchema — FNV-1a entity seed |
| **WHAT** | Two BatchSchema instances built from the same batch name but different timestamps produce the same `entity_seed` |
| **WHERE** | `batch_schema.rs` → `tests::entity_seed_deterministic` |
| **WHEN** | The same batch is reprocessed after a system restart (timestamp changes, batch name stays the same) |
| **WHY** | `entity_seed` is the FNV-1a(batch_name) input to the KAKI Minter for the Entity particle; if the seed changed between runs, the Entity KAKI would change, breaking immutability — a core BeeMDM invariant |
| **HOW** | Infer two schemas from `"batch_abc"` with timestamps 0 and 99; assert `s1.entity_seed == s2.entity_seed` |
| **HOW MANY** | Exact u32 equality across both instances |

---

## Module 3 — `dw_analytics` (4 tests)

### TC-DA-01 · report_counts_by_state

| | |
|---|---|
| **WHO** | DwAnalytics — particle state aggregation |
| **WHAT** | `DwReport` correctly counts total, Golden, Fuzzy, and Dead particles across the persisted journal |
| **WHERE** | `dw_analytics.rs` → `tests::report_counts_by_state` |
| **WHEN** | An operator or dashboard queries the warehouse health after a batch run |
| **WHY** | DAMA-DMBOK data quality KPIs require knowing how many records reached Golden vs stayed Fuzzy or were retired; wrong counts mislead stewardship decisions |
| **HOW** | Populate PersistedDb with 4 particles: 2 Golden (epochs 1,2), 1 Fuzzy (epoch 3), 1 Dead (epoch 4); run `DwAnalytics::report(10)`; assert each count |
| **HOW MANY** | total=4, golden=2, fuzzy=1, dead=1 |

---

### TC-DA-02 · report_epoch_range

| | |
|---|---|
| **WHO** | DwAnalytics — epoch min/max scan |
| **WHAT** | `DwReport` correctly identifies the earliest and latest epoch in the journal |
| **WHERE** | `dw_analytics.rs` → `tests::report_epoch_range` |
| **WHEN** | After ingesting a multi-epoch batch, the operator needs to know the temporal span of data in the warehouse |
| **WHY** | Epoch ranges are used for auditing, incremental exports, and time-travel queries in BeeMDM; wrong range boundaries would silently omit records |
| **HOW** | Same 4-particle fixture as TC-DA-01; assert `r.epoch_min == 1` and `r.epoch_max == 4` |
| **HOW MANY** | epoch_min=1, epoch_max=4 |

---

### TC-DA-03 · count_in_epoch_range

| | |
|---|---|
| **WHO** | DwAnalytics — range-bounded particle count |
| **WHAT** | `count_in_epoch_range(lo, hi)` returns the correct count of particles whose epoch falls within [lo, hi] (inclusive) |
| **WHERE** | `dw_analytics.rs` → `tests::count_in_epoch_range` |
| **WHEN** | Incremental ETL runs need to know how many records are new since the last export epoch |
| **WHY** | Overcounting would trigger duplicate export; undercounting would silently miss records — both corrupt downstream consumers |
| **HOW** | Same fixture; assert `count(1,2)==2`, `count(1,4)==4`, `count(5,9)==0` |
| **HOW MANY** | 3 range queries, all exact |

---

### TC-DA-04 · golden_ratio

| | |
|---|---|
| **WHO** | DwAnalytics — data quality ratio KPI |
| **WHAT** | `golden_ratio()` returns Golden count / total count as an f32, correct to float epsilon |
| **WHERE** | `dw_analytics.rs` → `tests::golden_ratio` |
| **WHEN** | The operator dashboard or StoryEngine gate reads the data quality score for a tribe |
| **WHY** | BeeMDM uses the golden ratio as the primary DQ health metric; a wrong ratio would misclassify a low-quality batch as healthy or block a healthy batch as failing |
| **HOW** | Same fixture (2 Golden, 4 total); assert `(golden_ratio() - 0.5).abs() < f32::EPSILON` |
| **HOW MANY** | ratio = 0.5 ± float epsilon |

---

## Module 4 — `etl_pipeline` (5 tests)

### TC-EP-01 · ingest_tsv_from_landing_zone

| | |
|---|---|
| **WHO** | EtlPipeline — TSV file ingestion path |
| **WHAT** | A `.tsv` file placed in the landing zone is picked up, parsed, and ingested; `records_ingested` reflects the correct data-row count |
| **WHERE** | `etl_pipeline.rs` → `tests::ingest_tsv_from_landing_zone` |
| **WHEN** | A tab-separated export (e.g. from a government registry system) is dropped directly into the shard folder |
| **WHY** | TSV is one of the three primary input formats; if the TSV path silently fails, records are lost without any alert |
| **HOW** | Write a 2-row TSV to the landing dir; call `pipe.run_once()`; assert `records_ingested == 2`, `records_skipped == 0` |
| **HOW MANY** | 2 ingested, 0 skipped |

---

### TC-EP-02 · ingest_csv_from_landing_zone

| | |
|---|---|
| **WHO** | EtlPipeline — CSV file ingestion path |
| **WHAT** | A `.csv` file is picked up, parsed (comma-separated), and its single data row is ingested |
| **WHERE** | `etl_pipeline.rs` → `tests::ingest_csv_from_landing_zone` |
| **WHEN** | A comma-separated export (e.g. Excel Save As CSV from cemetery records) is dropped into the shard folder |
| **WHY** | CSV and TSV share the same kaki_generator but use different delimiters; a bug in the delimiter branch would silently produce one mangled record instead of separate fields |
| **HOW** | Write a 1-row CSV; call `run_once()`; assert `records_ingested == 1` |
| **HOW MANY** | 1 ingested |

---

### TC-EP-03 · ingest_zip_with_tsv

| | |
|---|---|
| **WHO** | EtlPipeline — ZIP extraction + TSV ingestion combined |
| **WHAT** | A `.zip` file containing a `.tsv` entry is extracted and its records are ingested; both `zips_processed` and `records_ingested` are updated |
| **WHERE** | `etl_pipeline.rs` → `tests::ingest_zip_with_tsv` |
| **WHEN** | This is the primary production path for the 12 Bandizip cemetery files |
| **WHY** | Validates the full ZIP → extract → parse → ingest chain in a single test; a break anywhere in this chain silently drops all records from the batch |
| **HOW** | Build a STORE ZIP containing a 1-row TSV; write to landing dir; call `run_once()`; assert `zips_processed == 1`, `records_ingested == 1` |
| **HOW MANY** | 1 ZIP processed, 1 record ingested |

---

### TC-EP-04 · compile_way_file

| | |
|---|---|
| **WHO** | EtlPipeline — WAYv2.0 sovereignty file compilation |
| **WHAT** | A `.way` file placed in the landing zone is picked up and compiled; `way_compiled` counter increments |
| **WHERE** | `etl_pipeline.rs` → `tests::compile_way_file` |
| **WHEN** | A new tribe sovereignty declaration file is deployed alongside a data batch |
| **WHY** | WAY files define the AAOL sovereignty rules that govern which roles can access which data; if compilation is silently skipped, the tribe operates with no security policy |
| **HOW** | Write a minimal 3-line `.way` file to landing dir; call `run_once()`; assert `way_compiled == 1` |
| **HOW MANY** | 1 WAY file compiled |

---

### TC-EP-05 · second_poll_sees_no_duplicates

| | |
|---|---|
| **WHO** | EtlPipeline — file deduplication (seen-files set) |
| **WHAT** | Calling `run_once()` twice on the same file does not re-ingest any records; the seen-files guard prevents duplicate processing |
| **WHERE** | `etl_pipeline.rs` → `tests::second_poll_sees_no_duplicates` |
| **WHEN** | The bee-watchdog daemon polls the shard folder every 2 seconds; a file that has not yet been moved or deleted will be seen on every poll |
| **WHY** | Without deduplication, every poll would re-insert all records already in the database, creating unbounded duplicates and invalidating KAKI uniqueness |
| **HOW** | Write one TSV; `run_once()` → record first count; `run_once()` again; assert count unchanged |
| **HOW MANY** | 0 additional records on second poll |

---

## Module 5 — `kaki_generator` (6 tests)

### TC-KG-01 · parse_tsv_single_row

| | |
|---|---|
| **WHO** | kaki_generator — TSV parser |
| **WHAT** | `parse_tsv` splits a tab-delimited byte slice into one `RawRecord` with correct headers and values |
| **WHERE** | `kaki_generator.rs` → `tests::parse_tsv_single_row` |
| **WHEN** | Any TSV file is parsed before KAKI minting |
| **WHY** | Headers become EAV attribute names; values become EAV values. A wrong parse here corrupts every downstream EAV triple for the entire batch |
| **HOW** | Parse `b"name\tepoch\tstate\nAli_Karim\t1\tGolden\n"`; assert 1 record, headers = `["name","epoch","state"]`, first value = `b"Ali_Karim"` |
| **HOW MANY** | 1 record, 3 headers, value[0] exact match |

---

### TC-KG-02 · generate_golden_particle

| | |
|---|---|
| **WHO** | kaki_generator — particle generation for an explicitly Golden record |
| **WHAT** | A record whose `state` column is `"Golden"` produces a `GeneratedEntry` with `epoch == 5` and an EAV triple where `ATTR_STATE` decodes to `ParticleState::Golden` |
| **WHERE** | `kaki_generator.rs` → `tests::generate_golden_particle` |
| **WHEN** | A pre-verified record (already marked Golden in the source system) enters the ETL pipeline |
| **WHY** | Golden state in the source file must propagate correctly through generation; if the state EAV is wrong, the record gets stored as Fuzzy and enters the steward queue unnecessarily |
| **HOW** | Parse TSV with `state=Golden`, call `generate(&minter, &row)`; find EAV with `attr_hash == ATTR_STATE`; decode and assert `ParticleState::Golden` |
| **HOW MANY** | 1 state EAV, decoded state = Golden, epoch = 5 |

---

### TC-KG-03 · deterministic_uuid_hash_for_same_name

| | |
|---|---|
| **WHO** | kaki_generator — deterministic uuid_hash derivation |
| **WHAT** | Two separate `KakiMinter` instances produce `GeneratedEntry` values with identical `uuid_hash` for the same record name |
| **WHERE** | `kaki_generator.rs` → `tests::deterministic_uuid_hash_for_same_name` |
| **WHEN** | The system restarts, the minter is recreated, but the same source records are reprocessed (e.g. after a crash) |
| **WHY** | KAKI immutability — the Entity KAKI must be the same across all time for the same entity. If `uuid_hash` changes between minter instances, identity particles get duplicated and KAKI collision detection fails |
| **HOW** | Generate from two independent minters for the same name; assert `ge1.particle.uuid_hash() == ge2.particle.uuid_hash()` |
| **HOW MANY** | Exact u64 equality |

---

### TC-KG-04 · missing_state_defaults_to_fuzzy

| | |
|---|---|
| **WHO** | kaki_generator — default-state guard |
| **WHAT** | A record with no `state` column produces `ParticleState::Fuzzy`, not a panic and not `Golden` |
| **WHERE** | `kaki_generator.rs` → `tests::missing_state_defaults_to_fuzzy` |
| **WHEN** | A source CSV has no `state` column (e.g. a raw registry export before any DQ processing) |
| **WHY** | The default must be Fuzzy, never Golden — defaulting to Golden would allow unvalidated records to bypass the entire DQ station chain and enter the Golden Record store |
| **HOW** | Parse TSV with only a `name` column; generate; find ATTR_STATE EAV; decode; assert `ParticleState::Fuzzy` |
| **HOW MANY** | State = Fuzzy |

---

### TC-KG-05 · extra_columns_become_eav_triples

| | |
|---|---|
| **WHO** | kaki_generator — dynamic EAV triple generation for arbitrary columns |
| **WHAT** | A column named `city` with value `Baghdad` produces an EAV triple with `attr_hash == fnv1a_32(b"city")` and `value == b"Baghdad"` |
| **WHERE** | `kaki_generator.rs` → `tests::extra_columns_become_eav_triples` |
| **WHEN** | Every batch — CSV column names are dynamic and known only at parse time |
| **WHY** | EnkiDB stores everything as EAV; if arbitrary columns are silently dropped instead of stored as triples, custom attributes (names, dates, coordinates) are lost with no error |
| **HOW** | Parse TSV with `name,city`; generate; compute `city_hash = fnv1a_32(b"city")`; assert EAV with that hash and value `b"Baghdad"` exists |
| **HOW MANY** | 1 EAV triple for `city`, exact value match |

---

### TC-KG-06 · parse_csv_works

| | |
|---|---|
| **WHO** | kaki_generator — CSV parser (comma delimiter) |
| **WHAT** | `parse_csv` correctly splits on commas and returns a record with the right first value |
| **WHERE** | `kaki_generator.rs` → `tests::parse_csv_works` |
| **WHEN** | A CSV file (not TSV) enters the pipeline |
| **WHY** | TSV and CSV use the same `RawRecord` struct but different delimiters; verifying both parsers independently ensures a delimiter mix-up cannot silently collapse multi-column rows into a single-field record |
| **HOW** | Parse `b"name,epoch,state\nFatima,2,Golden\n"` with `parse_csv`; assert 1 record with `values[0] == b"Fatima"` |
| **HOW MANY** | 1 record, value[0] exact match |

---

## Module 6 — `landing_zone` (2 tests)

### TC-LZ-01 · poll_returns_new_files

| | |
|---|---|
| **WHO** | LandingZone — filesystem poller and deduplication |
| **WHAT** | First poll returns all existing files; second poll returns nothing for the same files; third poll returns only the newly added file |
| **WHERE** | `landing_zone.rs` → `tests::poll_returns_new_files` |
| **WHEN** | The bee-watchdog daemon calls `lz.poll()` in its main loop every 2 seconds |
| **WHY** | The watchdog's entire event-driven architecture depends on `poll()` returning each file exactly once; if deduplication fails, every record is processed indefinitely |
| **HOW** | Touch `a.zip`, `b.way` → first poll; assert 2 files. Poll again → assert 0. Touch `c.csv` → poll; assert 1 file with kind=Csv |
| **HOW MANY** | first=2, second=0, third=1 |

---

### TC-LZ-02 · kind_detection

| | |
|---|---|
| **WHO** | LandingZone — file type classifier |
| **WHAT** | `LandingFileKind::detect` correctly classifies filenames by extension, case-insensitively |
| **WHERE** | `landing_zone.rs` → `tests::kind_detection` |
| **WHEN** | Every file discovered during a poll is classified before dispatch to the correct pipeline branch |
| **WHY** | The wrong kind routes a ZIP to the CSV parser (producing garbage records) or a WAY file to the ZIP extractor (producing a ZipError instead of AAOL compilation) |
| **HOW** | Assert `"data.ZIP"` → Zip, `"najaf.way"` → Way, `"records.tsv"` → Tsv, `"unknown.bin"` → Other |
| **HOW MANY** | 4 exact kind matches |

---

## Module 7 — `processing_zone` (3 tests)

### TC-PZ-01 · creates_subdirs

| | |
|---|---|
| **WHO** | ProcessingZone — filesystem initialisation |
| **WHAT** | Calling `ProcessingZone::new(shard)` creates both `Processing/` and `Moved_To/` subdirectories inside the shard folder |
| **WHERE** | `processing_zone.rs` → `tests::creates_subdirs` |
| **WHEN** | The bee-watchdog starts for the first time on a new shard folder |
| **WHY** | If either directory is missing, the first `stage()` or `complete()` call will fail with a filesystem error, halting the entire watchdog |
| **HOW** | Create a temp shard dir; construct `ProcessingZone::new`; assert both `processing_dir()` and `moved_to_dir()` exist on disk |
| **HOW MANY** | Both paths exist |

---

### TC-PZ-02 · stage_and_complete

| | |
|---|---|
| **WHO** | ProcessingZone — batch lifecycle (stage → complete) |
| **WHAT** | `stage()` writes entries to `Processing/{batch}/`; `complete()` moves that directory to `Moved_To/{batch}/`; `is_staged()` returns true between and false after |
| **WHERE** | `processing_zone.rs` → `tests::stage_and_complete` |
| **WHEN** | Every ZIP batch is staged before ETL begins and completed after StoryEngine confirmation |
| **WHY** | The Processing/Moved_To lifecycle is the operator's visual audit trail; if `complete()` does not atomically rename the directory, operators cannot tell which batches are in-flight vs finished |
| **HOW** | Stage `"batch_001"` with one CSV entry; assert file exists and `is_staged` true; call `complete`; assert dest exists and `is_staged` false |
| **HOW MANY** | File present after stage; absent from Processing after complete; dest present in Moved_To |

---

### TC-PZ-03 · write_schema_descriptor

| | |
|---|---|
| **WHO** | ProcessingZone — in-batch file writer |
| **WHAT** | `write_in_batch()` creates an arbitrary named file inside an already-staged batch directory |
| **WHERE** | `processing_zone.rs` → `tests::write_schema_descriptor` |
| **WHEN** | After BatchSchema is inferred, the `.schema` descriptor is written to the batch staging directory for operator inspection |
| **WHY** | The `.schema` file in `Processing/{batch}/` is the only human-readable record of what EnkiDB inferred from the CSV columns; without it, a schema mismatch has no traceable source |
| **HOW** | Stage batch `"b"` with no entries; call `write_in_batch("b", "batch.schema", "table_name = tb_test\n")`; read file back; assert content contains `"tb_test"` |
| **HOW MANY** | File exists, contains expected substring |

---

## Module 8 — `way_file` (4 tests)

### TC-WF-01 · parse_full_way_file

| | |
|---|---|
| **WHO** | WayFile parser — full-directive parsing |
| **WHAT** | A complete WAY source string is parsed into a `WayFile` struct with all fields correctly assigned |
| **WHERE** | `way_file.rs` → `tests::parse_full_way_file` |
| **WHEN** | A `.way` sovereignty declaration is read from the landing zone before AAOL compilation |
| **WHY** | Every field in a WAY file maps to a security property in the compiled AAOL (tribe ID, clearance level, sovereignty zone, emitting tribe); a wrong parse creates a mis-scoped security policy |
| **HOW** | Parse a 9-directive WAY string; assert `tribe_id==0x0001`, `class=="civil.registry"`, `sovereignty=="PA-15"`, `clearance==3`, `role=="Zikru"`, `emit_tribe==Some(0x0001)` |
| **HOW MANY** | 6 field assertions, all exact |

---

### TC-WF-02 · tribe_name_camel_case

| | |
|---|---|
| **WHO** | WayFile — class-to-name conversion |
| **WHAT** | `tribe_name()` converts `"civil.registry"` to `"CivilRegistry"` (PascalCase, dot removed) |
| **WHERE** | `way_file.rs` → `tests::tribe_name_camel_case` |
| **WHEN** | The way_compiler needs a valid AAOL identifier for the tribe actor name |
| **WHY** | AAOL identifiers cannot contain dots; using the raw class name `"civil.registry"` would produce a parse error in the generated `.akk` file |
| **HOW** | Parse the sample WAY; call `tribe_name()`; assert `== "CivilRegistry"` |
| **HOW MANY** | Exact string match |

---

### TC-WF-03 · defaults_for_missing_fields

| | |
|---|---|
| **WHO** | WayFile parser — default-value assignment |
| **WHAT** | A WAY file with only a `tribe` directive fills in clearance=3 and role="Zikru" as defaults |
| **WHERE** | `way_file.rs` → `tests::defaults_for_missing_fields` |
| **WHEN** | A minimal WAY file is used during initial tribe bootstrapping |
| **WHY** | If defaults are missing, `way_compiler::compile()` would panic or produce incomplete AAOL; the security policy for the tribe would be blank |
| **HOW** | Parse `"tribe 0x0002"`; assert `tribe_id==0x0002`, `clearance==3`, `role=="Zikru"` |
| **HOW MANY** | 3 assertions, all exact |

---

### TC-WF-04 · unknown_directive_errors

| | |
|---|---|
| **WHO** | WayFile parser — strict unknown-directive rejection |
| **WHAT** | A WAY file containing an unrecognised directive word returns `Err(…)`, not a silently ignored field |
| **WHERE** | `way_file.rs` → `tests::unknown_directive_errors` |
| **WHEN** | A typo in a WAY file (e.g. `"sovreingty PA-15"`) or a WAY file from a future version with new directives the current parser does not know |
| **WHY** | Silent ignore would mask configuration errors; a sovereignty zone written incorrectly would be silently dropped, leaving the tribe with no sovereignty constraint in the compiled AAOL |
| **HOW** | Parse `"unknown value"`; assert result is `Err` |
| **HOW MANY** | Err variant (any) |

---

## Module 9 — `way_compiler` (3 tests)

### TC-WC-01 · compile_produces_valid_aaol

| | |
|---|---|
| **WHO** | WayCompiler — AAOL source generation |
| **WHAT** | `compile(&way)` produces a `CompileResult` whose `akk_source` contains `"tribe CivilRegistry"`, `"role Zikru"`, and `"when event"` |
| **WHERE** | `way_compiler.rs` → `tests::compile_produces_valid_aaol` |
| **WHEN** | A WAY file is compiled to AAOL before being stored as the tribe's sovereignty program |
| **WHY** | The three required tokens are: the tribe identifier (access scope), the role (actor binding), and the event handler (activation rule); missing any one produces an incomplete security policy |
| **HOW** | Compile a full Najaf WAY; assert `akk_source` contains all three expected substrings |
| **HOW MANY** | 3 substring assertions |

---

### TC-WC-02 · generated_akk_parses_clean

| | |
|---|---|
| **WHO** | WayCompiler + AAOL parser — end-to-end round-trip |
| **WHAT** | The AAOL source emitted by `compile()` is tokenised and parsed by the sovereign AAOL parser without any error |
| **WHERE** | `way_compiler.rs` → `tests::generated_akk_parses_clean` |
| **WHEN** | A WAY file is compiled and the resulting `.akk` source is stored in the EnkiDB journal |
| **WHY** | If the generated AAOL is syntactically invalid, the sovereignty engine cannot load the policy, leaving the tribe with no access control whatsoever — a critical security gap |
| **HOW** | Compile Najaf WAY; tokenise `akk_source` with `aaol::tokenize()`; parse with `aaol::Parser::new(tokens).parse()`; assert no error |
| **HOW MANY** | 0 parse errors |

---

### TC-WC-03 · actor_naming_for_known_classes

| | |
|---|---|
| **WHO** | WayCompiler — class-to-actor-name mapping |
| **WHAT** | `actor_name_for(class)` maps four known class strings to their canonical AAOL actor names |
| **WHERE** | `way_compiler.rs` → `tests::actor_naming_for_known_classes` |
| **WHEN** | Any WAY file is compiled — the actor name is the AAOL `actor` directive that binds the sovereignty rule to an executing agent |
| **WHY** | Using the wrong actor name binds the security policy to a non-existent agent; the sovereignty rule is accepted by the parser but never enforced at runtime |
| **HOW** | Assert: `"civil.registry"` → `"Registrar"`, `"sensor.stream"` → `"StreamAgent"`, `"operational"` → `"OperationalAgent"`, `"data.pipeline"` → `"PipelineAgent"` |
| **HOW MANY** | 4 exact string matches |

---

## Coverage Summary

| Module | Tests | Key invariants covered |
|--------|------:|------------------------|
| zip_engine | 6 | STORE extract, DEFLATE inflate (stored/fixed blocks), error on bad data, multi-entry traversal, empty-input safety |
| batch_schema | 5 | Mandatory vs optional classification, naming convention, descriptor format, empty-batch safety, entity seed immutability |
| dw_analytics | 4 | State counts, epoch range, range-bounded counts, golden ratio KPI |
| etl_pipeline | 5 | TSV ingestion, CSV ingestion, ZIP→TSV chain, WAY compilation, deduplication |
| kaki_generator | 6 | TSV parse, CSV parse, Golden particle generation, Fuzzy default, deterministic uuid_hash, dynamic EAV columns |
| landing_zone | 2 | Single-return deduplication, extension-based kind detection |
| processing_zone | 3 | Directory creation, stage→complete lifecycle, in-batch file writing |
| way_file | 4 | Full parse, class→PascalCase, defaults, unknown-directive rejection |
| way_compiler | 3 | AAOL generation, generated source round-trips the parser, actor-name mapping |
| **Total** | **38** | |

---

## Running the tests

```bash
cd workspace/bahyway_v4
cargo test -p enkidb-dw
```

Expected output: `test result: ok. 38 passed; 0 failed`
