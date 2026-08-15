# akkadi-cli — Sovereign Command-Line Interface
𒀭𒆳𒁺 *Akkadi* — "the language of Akkad / the sovereign tongue"

> **Binary · bin/akkadi-cli** | AkkadiNotebook | KAKI Inspector | Tribe Compass

---

## W5H2 Manual

### WHO — 𒀭 Who Uses This Binary

| Persona | Daily use |
|---|---|
| **Data Steward** | Inspects KAKI PKs, checks tribe quality lanes, monitors pipeline stations |
| **System Architect** | Views system status (7 services, 2624+ tests), compares sovereign vs legacy |
| **AkkadianAOL Developer** | Runs `.akk` cells in the sovereign notebook, inspects IR output |
| **DevOps / SRE** | Checks BeeMDM pipeline station health, reads flow statistics |
| **New stakeholder** | `akkadi-cli vocab` — searches the Akkadi sovereign vocabulary |

---

### WHAT — 𒁾 What This Tablet Contains

`akkadi-cli` is the **unified developer interface** to BahyWay.Ecosystem v4.0.  
It is a single binary that exposes seven command groups:

#### Command Groups

| Command | Description |
|---|---|
| `status [--format json]` | 7-service system health dashboard |
| `kaki inspect <hex>` | Decode all 16 KAKI PK bytes with sovereign meaning |
| `kaki generate --domain <n> --quality <n>` | Generate a KAKI PK stub |
| `pipeline [--format json]` | BeeMDM 4-lane × 8-station flow statistics |
| `tribe [--dim <name>]` | Ibn Wahshiyya 7D tribe compass (orbit radii, quality scores) |
| `compare --tribe <id> [--limit <n>]` | Sovereign vs legacy speed comparison (avg ~425×) |
| `notebook run <file>` | Execute an `.akk` sovereign notebook |
| `notebook new <title>` | Create a new notebook with metadata |
| `vocab search <term>` | Search the Akkadi sovereign vocabulary |
| `vocab list` | List all vocabulary entries |

#### Output Formats

```
--format table      (default) — aligned ASCII table
--format json       — machine-parseable JSON
--format cuneiform  — cuneiform Unicode glyphs
```

#### Sovereign Notebook

The notebook subsystem (`AkkadiNotebook` / `AkkadiKernel`) supports 10 cell kinds:

| Cell kind | Description |
|---|---|
| `AkkadianAol` | Execute AkkadianAOL `.akk` source — parsed by aaol crate |
| `Rust` | Rust code reference (display only) |
| `Hepta` | 7D quality score computation |
| `Kaki` | KAKI PK inspection |
| `Pipeline` | Pipeline station query |
| `Tribe` | Tribe compass query |
| `Sql` | SQL query (read-only, sovereign gateway) |
| `Json` | JSON data display |
| `Markdown` | Rich text documentation |
| `AkkValue` | Typed AkkValue display |

Notebooks are saved as JSON (`.akk.nb`) with UUID cell IDs, execution state, and timestamps.

---

### WHEN — 𒌓 When Is This Invoked

```
Development lifecycle                    Production operations
──────────────────                      ──────────────────────
$ akkadi-cli status                     $ akkadi-cli pipeline --format json
  (health check before coding)           (monitoring / alerting integration)

$ akkadi-cli kaki inspect b0b1b2…       $ akkadi-cli tribe --dim Accuracy
  (debug a KAKI PK from logs)            (quality dashboard)

$ akkadi-cli notebook run my.akk.nb     $ akkadi-cli compare --tribe 3 --limit 100
  (prototype a sovereign algorithm)      (performance regression test)

$ akkadi-cli vocab search "seal"
  (discover Akkadi language terms)
```

---

### WHERE — 𒆳 Architectural Position

```
bin/akkadi-cli
    │
    │  reads from (library APIs, not network)
    ├──► crates/aaol          (AkkadianAOL parsing for notebook cells)
    ├──► crates/akkadi        (vocabulary lookup)
    ├──► crates/akkadi-ir     (IR inspection)
    ├──► client/EnkiClient    (HTTP → EnkiDB server, if running)
    │
    │  displays
    └──► OutputFormat { Table | Json | Cuneiform }
         ├── Tab-aligned ASCII tables
         ├── serde_json pretty-print
         └── Unicode cuneiform glyphs (𒁾 𒆳 𒀭)
```

`akkadi-cli` is a **development tool** — it does not run in production pipelines.  
It communicates with a running `bin/bahyway-server` via HTTP when one is available,  
and operates in offline mode (showing static sovereign data) otherwise.

---

### WHY — 𒀊 Why This Exists

**The problem:**  
Understanding BahyWay.Ecosystem without tooling requires reading 70+ crate source files.  
A data steward should be able to inspect a KAKI PK without understanding Rust.

**The sovereign solution:**  
One binary, seven command groups, three output formats.  
The KAKI inspector alone saves hours of manual byte decoding.

**Why a notebook instead of a REPL?**  
A REPL state is ephemeral. A notebook (`.akk.nb`) is a **sovereign document** — it carries  
UUID cell IDs, execution timestamps, and MUDÛ scores. It can be committed to the repository  
and replayed deterministically.

**Why cuneiform output format?**  
When presenting BahyWay.Ecosystem to stakeholders unfamiliar with English, cuneiform  
glyphs provide immediate cultural and intellectual context. The glyph `𒁾` means "clay tablet"  
— the same metaphor used for `.akk` files.

---

### HOW — 𒅗 How It Works

#### KAKI PK decode (16 bytes)

```
$ akkadi-cli kaki inspect b0b1b2b3b4b5b6 07 0801 03 b9 05 aabbcc

B0–B6  BLAKE3 geometric address   b0b1b2b3b4b5b6
B7     Sequence byte               07 (seq=7)
B8–B9  AkkadiSeal bytes            08 01
B10    Domain byte                 03 = Water
B11    Quality (÷240.0)            b9 = 185 → TribeMember lane
B12    HijriPeriod                 05 = Dhul-Hijjah
B13–15 TribeRGB ⊕ Seal Ghost      aa bb cc
```

#### Pipeline station health

```
$ akkadi-cli pipeline

Station          Lane         In/s   Out/s   Avg ms
──────────────── ──────────── ────── ─────── ───────
KAKI Validation  Gem          1,240   1,240      0.3
VGCA Cleansing   TribeMember  3,890   3,802      1.2
Semantic Check   Active       8,240   8,100      0.8
EnkiDB Write     Fuzzy        2,100   1,950      4.1
…
```

#### Sovereign vs legacy comparison

```
$ akkadi-cli compare --tribe 3

Operation          Sovereign    Legacy      Speedup
────────────────── ──────────── ─────────── ───────
KAKI PK lookup        0.3 ms     127.0 ms    423×
EAV triple insert     0.8 ms     342.0 ms    427×
Quality gate eval     0.1 ms      51.0 ms    510×
…
Average speedup: 425×
```

---

### HOW MUCH — 𒀸 Sovereign Metrics

| Metric | Value |
|---|---|
| Source files | 20 |
| Lines of Rust | ~2,000 |
| Command groups | 7 |
| Cell types (notebook) | 10 |
| Output formats | 3 (Table / JSON / Cuneiform) |
| Average sovereign speedup | **~425×** vs legacy SQL |
| External dependencies | clap-style arg parsing, ureq (HTTP), serde_json, uuid, dirs, toml |

---

## Sovereign Constraints

- `#![forbid(unsafe_code)]`
- KAKI `QUALITY_DIVISOR = 240.0` — all quality % computed as `b11 / 240.0 * 100` (ADR-001)
- Notebook cell IDs are **UUID v4** — guaranteed globally unique
- `EnkiClient::health()` has a 5-second timeout; CLI degrades gracefully to offline mode
- "Way" suffix absent from all command names, types, and output labels (ADR-002)

---

## Files

```
bin/akkadi-cli/
├── Cargo.toml
└── src/
    ├── main.rs                    — argument dispatch, print_usage, flag_format, flag_value
    ├── lib.rs                     — crate root
    ├── config.rs                  — AkkConfig (loaded from ~/.akkadi/config.toml)
    ├── client/
    │   ├── mod.rs
    │   └── enki.rs                — EnkiClient (HTTP · health check)
    ├── commands/
    │   ├── mod.rs
    │   ├── status.rs              — system_status()  7 services
    │   ├── kaki.rs                — inspect_kaki(), generate_kaki()
    │   ├── pipeline.rs            — pipeline_stats()  BeeMDM 4-lane × 8-station
    │   ├── tribe.rs               — list_tribes(), show_tribe()  Ibn Wahshiyya 7D
    │   └── compare.rs             — sovereign_vs_legacy()  avg ~425×
    ├── notebook/
    │   ├── mod.rs
    │   ├── cell.rs                — CellKind (10 variants), CellState, NotebookCell
    │   ├── kernel.rs              — AkkadiKernel::execute() via aaol Parser
    │   ├── renderer.rs            — render_cell(), render_notebook_header()
    │   └── session.rs             — AkkadiNotebook save/load (serde_json)
    └── output/
        ├── mod.rs
        └── format.rs              — OutputFormat { Table | Json | Cuneiform }
```

## Quick Start

```bash
# From the workspace root:
cargo run -p akkadi-cli -- status
cargo run -p akkadi-cli -- kaki inspect b0b1b2b3b4b5b600080103b90500aabb
cargo run -p akkadi-cli -- pipeline --format json
cargo run -p akkadi-cli -- tribe --dim Accuracy
cargo run -p akkadi-cli -- vocab search seal
cargo run -p akkadi-cli -- notebook new "My Analysis"
```
