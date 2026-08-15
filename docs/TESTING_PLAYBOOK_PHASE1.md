# BahyWay.Ecosystem v4.0 — Phase 1 Testing Playbook

> 50 Compressed Files · 10 Million Particles · HeptaScript < 1 sec
> Run this playbook top-to-bottom in order. Record PASS / FAIL / STUB for every test.

---

## Prerequisites

```bash
# 1. Make sure you are on the correct branch
git checkout claude/babyway-ecosystem-new-crates-meerhp
git pull origin claude/babyway-ecosystem-new-crates-meerhp

# 2. Confirm workspace compiles cleanly
cd /home/user/EnkiDB/workspace/bahyway_v4
cargo build --workspace 2>&1 | grep -E "^error" | head -20
# Expected: no output (zero errors)

# 3. Run the full test suite baseline
cargo test --workspace 2>&1 | tail -5
# Expected: all tests pass, 0 failed
```

---

## BLOCK A — Archive Decompression (NARAMSIN Stage 0)

Run all archive tests first — these validate that the 50-file corpus is correctly
classified before any particle data is loaded.

### A-001 — Run unit tests for naramsin-archive

```bash
cargo test -p naramsin-archive -- --nocapture
```

**Expected output:**
```
test tests::empty_input_returns_error ... ok
test tests::truncated_zip_returns_truncated ... ok
test tests::valid_store_zip_decompresses ... ok
test tests::recursion_depth_exceeded ... ok
test tests::decompress_valid_gzip_tar ... ok
test result: ok. 9 passed; 0 failed
```

Record: PASS / FAIL

---

### A-002 — ZIP STORE extraction (ashnan_test_zip.zip)

```bash
cargo run --example naramsin_extract -- ashnan_test_zip.zip
```

If no example exists yet, use this one-liner test binary (add to naramsin-archive/examples/):

```rust
// naramsin-archive/examples/naramsin_extract.rs
fn main() {
    let path = std::env::args().nth(1).expect("usage: naramsin_extract <file>");
    let data = std::fs::read(&path).expect("cannot read file");
    match naramsin_archive::decompress(&data, 0) {
        Ok(files) => {
            println!("OK: {} file(s) extracted", files.len());
            for f in &files { println!("  -> {} ({} bytes)", f.name, f.data.len()); }
        }
        Err(e) => println!("ERR: {e}"),
    }
}
```

```bash
# Build and run
cargo run -p naramsin-archive --example naramsin_extract -- /path/to/ashnan_test_zip.zip
```

**Expected:** `OK: N file(s) extracted` with filenames listed  
Record: PASS / FAIL

---

### A-003 — ZIP DEFLATE extraction (ashnan_test_zip.zip compressed entries)

Same command as A-002. If the zip contains DEFLATE-compressed entries they should
also extract correctly.

**Expected:** all entries listed, no `DeflateError`  
Record: PASS / FAIL

---

### A-004 — Nested ZIP (ashnan_test_nested.zip)

```bash
cargo run -p naramsin-archive --example naramsin_extract -- /path/to/ashnan_test_nested.zip
```

**Expected:** outer zip extracted, inner zip recursively extracted (depth ≤ 4)  
Record: PASS / FAIL

---

### A-005 — Corrupt ZIP (ashnan_test_CORRUPT.zip)

```bash
cargo run -p naramsin-archive --example naramsin_extract -- /path/to/ashnan_test_CORRUPT.zip
```

**Expected:** `ERR: NARAMSIN NRM_TRUNCATED: archive truncated mid-transfer`  
Record: PASS / FAIL  
**CRITICAL** — if this returns OK or partial data, stop and report immediately.

---

### A-006 — tar.gz extraction (ashnan_test_targz.tar)

```bash
cargo run -p naramsin-archive --example naramsin_extract -- /path/to/ashnan_test_targz.tar
```

**Expected:** `OK: N file(s) extracted` — gzip unwrap + tar reader  
Record: PASS / FAIL

---

### A-007 — tar.bz2 (ashnan_test_tarbz2.tar)

```bash
cargo run -p naramsin-archive --example naramsin_extract -- /path/to/ashnan_test_tarbz2.tar
```

**Expected:** `ERR: NARAMSIN: unsupported format: bzip2 sovereign module pending`  
This is EXPECTED behavior — bzip2 is a Phase 2 stub.  
Record: STUB (expected)

---

### A-008 — tar.xz (ashnan_test_tarxz.tar)

```bash
cargo run -p naramsin-archive --example naramsin_extract -- /path/to/ashnan_test_tarxz.tar
```

**Expected:** `ERR: NARAMSIN: unsupported format: xz sovereign module pending`  
Record: STUB (expected)

---

### A-009 — 7z (ashnan_test_7z.7z)

```bash
cargo run -p naramsin-archive --example naramsin_extract -- /path/to/ashnan_test_7z.7z
```

**Expected:** `ERR: NARAMSIN: unsupported format: 7z sovereign module pending`  
Record: STUB (expected)

---

### A-010 — Zip-bomb depth guard

Create a synthetic test (or use naramsin-archive unit test):

```bash
cargo test -p naramsin-archive recursion_depth -- --nocapture
```

**Expected:** `MaxRecursionDepth(5)` returned, no stack overflow  
Record: PASS / FAIL

---

### A-BLOCK Summary

| Test | Description | Result |
|------|-------------|--------|
| A-001 | Unit tests | |
| A-002 | ZIP STORE | |
| A-003 | ZIP DEFLATE | |
| A-004 | Nested ZIP | |
| A-005 | CORRUPT ZIP | |
| A-006 | tar.gz | |
| A-007 | tar.bz2 | STUB expected |
| A-008 | tar.xz | STUB expected |
| A-009 | 7z | STUB expected |
| A-010 | Zip-bomb guard | |

**Gate:** A-001 through A-006 and A-010 must be PASS before proceeding to Block B.

---

## BLOCK B — CRC Integrity (bahyway-crc)

### B-001 — CRC-32 standard vector

```bash
cargo test -p bahyway-crc -- --nocapture
```

**Expected:**
```
test known_vector_crc32_123456789 ... ok   # "123456789" → 0xCBF43926
test known_vector_123456789 ... ok          # CRC-16 → 0x29B1
test result: ok. 7 passed; 0 failed
```

Record: PASS / FAIL

---

### B-002 — ZIP CRC-32 validation end-to-end

Extract any valid ZIP file via NARAMSIN and verify it does NOT produce a CRC error.
If a valid file's CRC-32 check fails → critical bug, stop testing.

Record: PASS / FAIL

---

## BLOCK C — Session Registry

### C-001 — Parse enkidb-sessions.toml

```bash
cargo test -p enkidb-session-registry -- --nocapture
```

**Expected:** 6 tests pass  
Record: PASS / FAIL

### C-002 — Manual parse check

Create a quick test file `test_sessions.toml`:

```toml
[[session]]
id = "test-write"
host = "127.0.0.1"
port = 7001
role = "WRITE"
node_type = "EnkiDB"
tribe_ids = [1, 2]
label = "Test Write Node"
enabled = true

[[session]]
id = "test-read"
host = "127.0.0.1"
port = 7002
role = "READ"
node_type = "EnkiDW"
tribe_ids = []
label = "Test Read Node"
enabled = false
```

Add a quick integration test or check that `from_toml()` parses it cleanly.  
Record: PASS / FAIL

---

## BLOCK D — ConEngine 7 CSR Rules

### D-001 — Unit tests

```bash
cargo test -p enkidb-con-engine -- --nocapture
```

**Expected:** 6 tests pass  
```
test tests::csr01_rejects_invalid_passport ... ok
test tests::csr02_rejects_client_for_write ... ok
test tests::csr07_rejects_cross_tribe ... ok
test tests::journal_verify_all_passes_after_operations ... ok
test tests::role_ordering ... ok
test tests::naru_entry_serialize_verify ... ok
```

Record: PASS / FAIL

### D-002 — CSR rule sequence verification

Confirm that all 7 rules fire in order for a valid DubSar cross-tribe write:
- CSR-01 passport valid ✓
- CSR-02 DubSar ≥ TabletWriter ✓
- CSR-03 NĀRU journal entry written ✓
- CSR-04 credential valid ✓
- CSR-05 DubSar exempt from Gilgamesh block ✓
- CSR-06 KIBRATU emitted (stub, no-op) ✓
- CSR-07 DubSar cross-tribe exempt ✓

This is covered by the unit test. Check output matches.  
Record: PASS / FAIL

---

## BLOCK E — HeptaScript Engine (10M Particles)

### E-001 — HeptaScript unit tests

```bash
cargo test -p heptascript -- --nocapture 2>&1 | tail -5
```

**Expected:** 164 passed, 0 failed  
Record: PASS / FAIL

### E-002 — ŠUMU-UKIN routing tests

```bash
cargo test -p heptascript sumuukin -- --nocapture
```

**Expected:** 4 tests pass  
Record: PASS / FAIL

### E-003 — NatiruIndex + EavExactIndex unit tests

```bash
cargo test -p enkidb-indexes -- --nocapture
```

**Expected:** 54 passed, 0 failed  
Record: PASS / FAIL

### E-004 — HeptaScript performance: 10M particles

This requires an EnkiDB instance with 10M particles loaded.  
Construct a benchmark query using AnchorStrategy::SurrogateTime:

```
NODE tribe = "1"
ANCHOR SurrogateTime
ORBITAL start = 0 end = 999999
FILTER_ORDER OrbitalRange EavAttr
ABORT_SCAN 5000000
LIMIT 1000
```

**Acceptance criteria:**
- Query returns in < 1 second wall-clock time
- `StreamStats.aborted == false` (did not hit ABORT_SCAN limit)
- `StreamStats.matched` > 0

Record: TIME=___ms  PASS / FAIL

### E-005 — ABORT_SCAN safety valve

Run a FullScan query with ABORT_SCAN set low:

```
NODE tribe = "1"
ANCHOR FullScan
ABORT_SCAN 100
LIMIT 1000000
```

**Expected:** `StreamStats.aborted == true`, returns in < 100ms  
Record: PASS / FAIL

---

## BLOCK F — Full Corpus Run (50 Files)

### F-001 — Batch extract all 50 files

```bash
for f in /path/to/test-corpus/*.{zip,tar,7z}; do
    echo -n "$f: "
    cargo run -p naramsin-archive --example naramsin_extract -- "$f" 2>&1 | head -1
done
```

**Expected classification:**
- Valid ZIP → `OK: N file(s)`
- Valid tar.gz → `OK: N file(s)`
- Corrupt files → `ERR: NARAMSIN NRM_TRUNCATED`
- Malicious/path-traversal → `OK` but filenames sanitised (no `../`)
- bzip2/xz/7z → `ERR: unsupported format ... pending`
- Unknown format → `ERR: NARAMSIN NRM_UNKNOWN_FORMAT`

Record count: OK=___ TRUNCATED=___ STUB=___ UNKNOWN=___ ERROR(unexpected)=___

**Gate:** `ERROR(unexpected)` count must be 0.

---

## Phase 1 Pass / Fail Gate

All of the following must be true to declare Phase 1 PASSED and proceed to Phase 2:

- [ ] A-001 through A-006, A-010: all PASS
- [ ] B-001, B-002: all PASS
- [ ] C-001, C-002: all PASS
- [ ] D-001, D-002: all PASS
- [ ] E-001 through E-003: all PASS
- [ ] E-004: HeptaScript query time < 1000ms at 10M particles
- [ ] E-005: ABORT_SCAN fires correctly
- [ ] F-001: zero unexpected errors across 50 files

---

## Failure Response Protocol

| Symptom | Action |
|---------|--------|
| CORRUPT.zip returns OK or partial data | STOP — NRM_TRUNCATED guard broken |
| CRC-32 mismatch on valid ZIP | STOP — bahyway-crc crc32() bug |
| HeptaScript > 1 sec at 10M particles | Tune BUCKET_ORBITALS, check ANCHOR strategy |
| NatiruIndex returns empty for valid range | Check epoch_orbital alignment |
| CSR-01 passes invalid passport | STOP — security breach, do not proceed |
| Any `unsafe` code warning | STOP — §0.3 violation |
| Unexpected panic | Capture stack trace, report with file/line |

---

## Notes Section (fill in during testing)

```
Date:
Environment:
Rust version (rustc --version):
Total files in corpus:

Block A notes:

Block B notes:

Block C notes:

Block D notes:

Block E timing:
  - E-004 query time: ___ms
  - Particles matched: ___
  - Particles evaluated: ___

Block F summary:
  OK=___ TRUNCATED=___ STUB=___ UNKNOWN=___ UNEXPECTED_ERROR=___

Phase 1 verdict: PASS / FAIL
Ready for Phase 2 (100M): YES / NO
```
