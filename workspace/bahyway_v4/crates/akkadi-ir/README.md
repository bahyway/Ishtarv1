# akkadi-ir — AkkadianAOL Sovereign Intermediate Representation
𒁾 *ṭuppu* — "the clay tablet that outlives every scribe"

> **Layer 9 · Languages** | Zero external dependencies | Pure Rust

---

## W5H2 Manual

### WHO — 𒀭 Who Uses This Crate

| Persona | Role |
|---|---|
| **AkkadianAOL Compiler** (`crates/aaol`) | Produces an `AkkIr` tree after parsing `.akk` source |
| **Code-Generation Backends** | Consume `AkkIr` via the `AkkBackend` trait → emit Rust / Python / JSON / PS / XML |
| **IR Optimisers** (future) | Walk and transform `AkkNode` variants before codegen |
| **DubSar IDE** (`crates/dubsar-ide`) | Inspects `AkkIr` for live semantic highlighting and quality overlays |
| **Quality Validators** | Run `ValidateQuality` walker to enforce KAKI B11 ≥ 59 on every node |

---

### WHAT — 𒁾 What This Tablet Contains

`akkadi-ir` is the **target-agnostic Intermediate Representation** for AkkadianAOL programs.  
It decouples *language semantics* from *code generation targets* so that one `.akk` source compiles  
to five targets without any backend knowing about Akkadian grammar.

**9 node variants:**

| Variant | Cuneiform concept | Purpose |
|---|---|---|
| `Particle` | 𒀭 — divine particle | Named value / atom |
| `Tribe` | 𒆳 — land / people | Grouped namespace |
| `Rule` | 𒋾 — tablet of law | `IF … THEN` policy |
| `Equation` | 𒀀 — water flow | Mathematical relation |
| `Flow` | 𒌓 — path of sun | Data pipeline edge |
| `Observe` | 𒀲 — eye | Read / subscribe |
| `Emit` | 𒁀 — to speak | Write / publish |
| `Guard` | 𒂗 — gate | Access-control predicate |
| `Pipeline` | 𒌓𒌓 — twin sun | Ordered sequence of nodes |

**Supporting systems:**

- `NodeId` — FNV-1a 64-bit deterministic hash (content-derived, mirrors KAKI principle)
- `QualityLane` — Gem / TribeMember / Active / Fuzzy / Dead (ADR-001)
- `HeptaDim` — 7-dimensional quality vector (Accuracy / Completeness / Consistency / Validity / Uniqueness / Timeliness / Integrity)
- `KineticForce` — `df/dt = I_phys + I_mem + I_learn` (Physical + Memory + Learning)
- `AkkWalker` trait — visitor pattern for IR traversal
- `AkkBackend` trait — code emission interface

---

### WHEN — 𒌓 When Is This Invoked

```
.akk source file
      │
      ▼
  [aaol lexer]  →  tokens
      │
      ▼
  [aaol parser]  →  AkkFile AST
      │
      ▼
  [aaol semantic]  →  validated AkkFile
      │
      ▼
  ┌─────────────┐
  │  akkadi-ir  │  ◄── YOU ARE HERE
  │  IrBuilder  │
  └─────────────┘
      │
      ├──► RustHintBackend   → .rs hints
      ├──► AkkGenBackend     → .akk (self-generative)
      ├──► DebugBackend      → diagnostic text
      └──► (future backends: Python, JSON, PS, XML)
```

Invoked once per compilation unit, after semantic analysis, before any backend emits output.

---

### WHERE — 𒆳 Architectural Position

```
Workspace Layer Map
───────────────────
Layer 0   bahyway-core, bahyway-crc, bahyway-algebra
Layer 1   enkidb-kaki, enkidb-vector-id
Layer 2   enkidb-block … enkidb-odb   (storage substrate)
Layer 3   enkidb-indexes, enkidb-dictionary
Layer 4   enkidb-engine, enkidb-query
Layer 4.5 vgca-engine, tribe-orbit-engine, shedu-engine …
Layer 5   story-engine, fuzzy-engine, hepta-score …
Layer 6   idu-prober, idu-batching
Layer 7   template-engine, diagnosis-engine …
Layer 8   adad-gate … permanent-storage  (pipeline stations)
Layer 9   aaol → akkadi-ir ← ← ← ← ← ← (THIS CRATE)
              └─► akkadi, heptascript
Layer 9.1 kupru, akkvalue, istar
Layer 10  eridu-runtime, eridu-scheduler, eridu-supervisor
Layer 11  dubsar-ide, dubsar-visualizer
Layer 12  bahyway-web
```

`akkadi-ir` sits **between the language layer (aaol) and all code-generation targets**.  
Nothing above Layer 9 may import `akkadi-ir` directly; they receive `AkkIr` via the compiler API.

---

### WHY — 𒀊 Why This Exists

**Design decision 1 — Homoiconicity**  
AkkadianAOL programs are data. An `.akk` file describing a tribe *is* a tribe. The IR makes this  
physical: `AkkNode::Tribe { … }` carries the same semantic weight as a running tribe instance.

**Design decision 2 — Separation of concerns**  
Without an IR, each backend would re-implement parsing. With `akkadi-ir`, adding a new target  
requires implementing one trait (`AkkBackend`) — the entire language is already parsed.

**Design decision 3 — Quality enforcement at IR level**  
`ValidateQuality` walker rejects nodes with `quality < 59` (Dead lane) before any byte is emitted.  
Quality is a first-class compile-time constraint, not a runtime check.

**Design decision 4 — Zero external deps**  
`akkadi-ir` has no Cargo dependencies. It must be auditable by inspection alone.  
The entire trust chain of the compiler starts here.

---

### HOW — 𒅗 How It Works

#### Building an IR tree

```rust
use akkadi_ir::{IrBuilder, AkkNode, ParticleNode, QualityLane};

let mut builder = IrBuilder::new("my_program");

let id = builder.add_node(AkkNode::Particle(ParticleNode {
    id:      NodeId::from_content(b"my_particle"),
    name:    "pm2_5".into(),
    quality: 200,  // Gem lane
    span:    Span::generated(),
    value:   Some("15.0".into()),
}));

let ir = builder.build();  // AkkIr
```

#### Walking the IR

```rust
use akkadi_ir::walker::{AkkWalker, NodeCounter};

let mut counter = NodeCounter::default();
ir.walk(&mut counter);
println!("Nodes: {}", counter.count);
```

#### Emitting via a backend

```rust
use akkadi_ir::backend::{AkkBackend, RustHintBackend};

let mut backend = RustHintBackend::new();
backend.emit_ir(&ir);
println!("{}", backend.output());
```

#### Quality lanes (ADR-001)

| Lane | B11 range | Meaning |
|---|---|---|
| `Gem` | 200–240 | Sovereign certified |
| `TribeMember` | 140–199 | Production ready |
| `Active` | 100–139 | Operational |
| `Fuzzy` | 59–99 | Needs improvement |
| `Dead` | 0–58 | Blocked at gate |

`QUALITY_DIVISOR = 240.0` — never 255 (ADR-001).

---

### HOW MUCH — 𒀸 Sovereign Metrics

| Metric | Value |
|---|---|
| Source files | 10 |
| Lines of Rust | ~1,600 |
| External dependencies | **0** |
| IR node variants | 9 |
| Walker implementations | 4 (CollectNames, ValidateQuality, NodeCounter, FindGenerative) |
| Backend implementations | 3 (Debug, AkkGen, RustHint) |
| Unit tests | 13 (8 in ir.rs, 5 in walker.rs) |
| Crate version | `4.0.2` |

---

## Sovereign Constraints

- `#![forbid(unsafe_code)]` — no unsafe Rust
- `#![allow(missing_docs)]` — suppressed to avoid 385 warnings on re-exported items
- "Way" suffix removed from all type names per ADR-002 (`FlowSource::Enki`, not `EnkiWay`)
- `IR_VERSION = "akkadi-ir-v4"` — version string in protocol metadata
- `NodeId` is content-derived (FNV-1a) — the same content always produces the same ID

---

## Files

```
crates/akkadi-ir/
├── Cargo.toml
└── src/
    ├── lib.rs        — crate root, re-exports, IR_VERSION
    ├── node.rs       — AkkNode enum (9 variants) + all node structs
    ├── node_id.rs    — NodeId (FNV-1a 64-bit), NodeIdBuilder
    ├── ir.rs         — AkkIr container, IrBuilder
    ├── walker.rs     — AkkWalker trait + 4 implementations
    ├── backend.rs    — AkkBackend trait + 3 implementations
    ├── quality.rs    — QualityLane, HeptaDim, HEPTA_DIMS
    ├── kinetic.rs    — ForceKind, KineticForce
    ├── span.rs       — Span (source location)
    └── errors.rs     — IrError (10 variants)
```
