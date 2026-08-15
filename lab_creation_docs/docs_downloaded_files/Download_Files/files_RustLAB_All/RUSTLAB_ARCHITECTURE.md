# RustLab Workspace Architecture Guide

## 🏗️ Complete Directory Structure

```
RustLab/
├── Cargo.toml                     # Workspace root
├── README.md
├── .gitignore
│
├── shared/                        # 📚 Core Library
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── domain.rs              # SovereignIdentity, Ethnicity, QualityTier
│       ├── utils.rs               # Helper functions
│       └── error.rs               # BahyWayError
│
├── akkadian_dsl/                  # 🔍 Query Language (v3.4)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── parser/
│       │   ├── mod.rs
│       │   ├── seek.rs            # SEEK queries
│       │   ├── find.rs            # FIND queries
│       │   └── traverse.rs        # TRAVERSE queries
│       ├── executor/
│       │   ├── mod.rs
│       │   └── sql_compiler.rs    # Akkadian → SQL
│       └── optimizer/
│           └── mod.rs             # Query optimization
│
├── fuzzy_engine/                  # 🎯 Fuzzy Logic Engine
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── rules/
│       │   ├── mod.rs
│       │   ├── tribal_rules.rs    # Tribal validation rules
│       │   └── quality_rules.rs   # Quality assessment rules
│       └── inference/
│           ├── mod.rs
│           └── fuzzy_inference.rs # Inference engine
│
├── score_engine/                  # 📊 Quality Scoring
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── calculators/
│       │   ├── mod.rs
│       │   ├── iso25012.rs        # ISO-25012 scoring
│       │   └── composite.rs       # Composite scoring
│       └── validators/
│           └── mod.rs             # Data validators
│
├── ontoway/                       # 🎨 Knowledge Graph Editor
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs                # Entry point
│   │   ├── api/
│   │   │   ├── mod.rs
│   │   │   ├── routes.rs          # REST endpoints
│   │   │   └── websocket.rs       # Real-time updates
│   │   ├── graphql/
│   │   │   ├── mod.rs
│   │   │   ├── schema.rs          # GraphQL schema
│   │   │   └── resolvers.rs       # Query/Mutation resolvers
│   │   ├── domain/
│   │   │   ├── mod.rs
│   │   │   ├── graph.rs           # Graph models
│   │   │   └── query.rs           # Query models
│   │   ├── services/
│   │   │   ├── mod.rs
│   │   │   ├── graph_service.rs   # Graph operations
│   │   │   └── query_service.rs   # Query execution
│   │   └── repository/
│   │       └── pg_repository.rs   # PostgreSQL + AGE
│   └── frontend/                  # React/TypeScript UI
│       ├── src/
│       ├── package.json
│       └── tsconfig.json
│
├── tribeway/                      # 🌍 Tribal Visualization
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   ├── api/
│   │   │   ├── mod.rs
│   │   │   └── routes.rs          # Tribal API endpoints
│   │   ├── domain/
│   │   │   ├── mod.rs
│   │   │   ├── tribe.rs           # Tribe models
│   │   │   └── validation.rs      # Name validation
│   │   ├── services/
│   │   │   ├── mod.rs
│   │   │   ├── tribal_service.rs  # Validation logic
│   │   │   └── color_service.rs   # Color assignment
│   │   ├── repository/
│   │   │   └── tribal_repo.rs     # Database access
│   │   └── visualization/
│   │       ├── mod.rs
│   │       ├── renderer.rs        # wgpu renderer
│   │       └── layout.rs          # 3D layout engine
│   └── frontend/                  # 3D Visualization UI
│       └── src/
│
├── najafway/                      # 🕌 Cemetery Management
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   ├── api/
│   │   │   └── routes.rs          # Cemetery API
│   │   ├── domain/
│   │   │   ├── mod.rs
│   │   │   └── cemetery.rs        # Cemetery models
│   │   ├── services/
│   │   │   ├── mod.rs
│   │   │   ├── ingest_service.rs  # CSV ingestion
│   │   │   └── query_service.rs   # Data queries
│   │   ├── repository/
│   │   │   └── cemetery_repo.rs
│   │   └── stress_test/
│   │       ├── mod.rs
│   │       ├── generator.rs       # Data generator
│   │       └── benchmarks.rs      # Performance tests
│   └── data/                      # CSV files
│       └── najaf_cemetery_*.csv
│
├── particlesway/                  # 💎 Gem Activation
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── domain/
│       │   ├── mod.rs
│       │   ├── particle.rs        # Particle models
│       │   └── activation.rs      # Activation rules
│       ├── services/
│       │   ├── mod.rs
│       │   └── activation_service.rs
│       └── activation/
│           ├── mod.rs
│           ├── detector.rs        # Quality detector
│           └── processor.rs       # Activation processor
│
├── bdbway_extension/              # 🔧 PostgreSQL Extension (existing)
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
│
├── scripts/                       # 🛠️ Utility Scripts
│   ├── import_all_najaf_batches.sh
│   ├── najaf_data_generator.py
│   ├── najafway_api_server.py
│   └── performance_report.sh
│
└── docs/                          # 📖 Documentation
    ├── ARCHITECTURE.md
    ├── API.md
    ├── DEPLOYMENT.md
    └── CONTRIBUTING.md
```

## 🔄 Dependency Flow

```
                    ┌──────────────┐
                    │    shared    │
                    │ (Core Types) │
                    └──────┬───────┘
                           │
          ┌────────────────┼────────────────┐
          ▼                ▼                ▼
    ┌──────────┐    ┌─────────────┐  ┌──────────────┐
    │ akkadian │    │fuzzy_engine │  │score_engine  │
    │   _dsl   │    │             │  │              │
    └────┬─────┘    └──────┬──────┘  └──────┬───────┘
         │                 │                 │
         └────────┬────────┴────────┬────────┘
                  ▼                 ▼
         ┌────────────────┐  ┌──────────────┐
         │    ontoway     │  │  tribeway    │
         │  (KG Editor)   │  │(Tribal Viz)  │
         └────────────────┘  └──────────────┘
                  │                 │
                  └────────┬────────┘
                           ▼
                  ┌────────────────┐
                  │   najafway     │
                  │ (Cemetery Mgmt)│
                  └────────────────┘
                           │
                           ▼
                  ┌────────────────┐
                  │ particlesway   │
                  │ (Gem Activation)│
                  └────────────────┘
```

## 🎯 How Each App Uses Shared Resources

### **OntoWay** (Knowledge Graph Editor)
```rust
use shared::{SovereignIdentity, Position, BahyWayError};
use akkadian_dsl::parse_query;
use fuzzy_engine::FuzzyValidator;

// Execute Akkadian query on graph
let query = parse_query("SEEK nodes WHERE quality >= 200")?;
let results = graph_service.execute(query).await?;
```

### **TribeWay** (Tribal Visualization)
```rust
use shared::{TribalNode, Ethnicity, QualityTier};
use fuzzy_engine::validate_tribal_name;
use score_engine::calculate_quality;

// Validate tribal identity
let identity = validate_tribal_name("محمد علي الدليمي")?;
let quality = calculate_quality(&identity)?;
let color = assign_tribal_color(&identity);
```

### **NajafWay** (Cemetery Management)
```rust
use shared::{SovereignIdentity, Position};
use akkadian_dsl::parse_query;
use score_engine::ISO25012Scorer;

// Import with quality scoring
for record in csv_reader {
    let quality = scorer.calculate(&record)?;
    let identity = SovereignIdentity::new(
        record.uuid,
        record.tribe_id,
        tribal_color,
        quality,
        75
    );
    repository.insert(identity, record).await?;
}
```

### **ParticlesWay** (Gem Activation)
```rust
use shared::{QualityTier, SovereignIdentity};
use fuzzy_engine::ActivationRules;

// Detect sovereign gems
let gems = repository
    .find_by_quality(QualityTier::Sovereign)
    .await?;

for gem in gems {
    if activation_rules.should_activate(&gem) {
        activate_gem(gem).await?;
    }
}
```

## 🚀 Build & Run Commands

### Build Everything
```bash
cargo build --workspace
```

### Build Specific App
```bash
cargo build -p ontoway
cargo build -p tribeway --release
```

### Run Tests
```bash
# All tests
cargo test --workspace

# Specific crate
cargo test -p fuzzy_engine
cargo test -p akkadian_dsl
```

### Run Applications
```bash
# OntoWay (default port 8080)
cargo run -p ontoway

# TribeWay (default port 8081)
cargo run -p tribeway --release

# NajafWay (default port 8082)
cargo run -p najafway

# ParticlesWay
cargo run -p particlesway
```

### Run with Environment Variables
```bash
DATABASE_URL=postgresql://akkad@localhost/bdbway \
cargo run -p ontoway
```

## 🧪 Testing Strategy

### Unit Tests
```bash
# Test shared library
cargo test -p shared

# Test Akkadian parser
cargo test -p akkadian_dsl --lib
```

### Integration Tests
```bash
# Test OntoWay with database
cargo test -p ontoway --test integration
```

### Stress Tests
```bash
# NajafWay stress test
cargo run -p najafway --release -- stress-test --records 1000000
```

### Benchmarks
```bash
cargo bench -p najafway
```

## 📦 Publishing Crates

### Publish Core Libraries
```bash
cargo publish -p shared
cargo publish -p akkadian_dsl
cargo publish -p fuzzy_engine
cargo publish -p score_engine
```

### Create Docker Images
```bash
docker build -f ontoway/Dockerfile -t bahyway/ontoway:latest .
docker build -f tribeway/Dockerfile -t bahyway/tribeway:latest .
```

## 🔧 Development Workflow

1. **Make changes to shared**
   ```bash
   cd shared
   # Edit code
   cargo test
   ```

2. **Changes automatically propagate**
   ```bash
   # Other crates see changes immediately
   cargo build -p ontoway  # Uses updated shared
   ```

3. **Add new dependency to workspace**
   ```toml
   # Edit root Cargo.toml
   [workspace.dependencies]
   new-crate = "1.0"
   ```

4. **Use in any crate**
   ```toml
   # In ontoway/Cargo.toml
   [dependencies]
   new-crate = { workspace = true }
   ```

## ✅ Advantages of This Structure

1. ✅ **Shared Code** - Single source of truth
2. ✅ **Fast Builds** - Incremental compilation
3. ✅ **Consistent Versions** - Workspace dependencies
4. ✅ **Easy Testing** - Test all or specific crates
5. ✅ **Clear Architecture** - Layered dependencies
6. ✅ **Independent Deployment** - Each app separate
7. ✅ **Type Safety** - Shared types prevent mismatches
8. ✅ **Code Reuse** - Maximum reusability

## 🎯 Next Steps

1. Run setup script: `bash setup_rustlab_workspace.sh`
2. Build workspace: `cargo build --workspace`
3. Implement each crate incrementally
4. Start with `shared` → `akkadian_dsl` → apps
