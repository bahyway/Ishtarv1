#!/bin/bash
# ============================================================
# RustLab Workspace Setup Script
# Creates the complete BahyWay Ecosystem structure
# ============================================================

set -e

WORKSPACE_ROOT="/workspace"
cd "$WORKSPACE_ROOT"

echo "🚀 Creating RustLab Workspace Structure..."
echo "============================================================"

# ============================================================
# 1. CORE LIBRARIES
# ============================================================

echo "📦 Creating Core Libraries..."

# 1.1 Shared Library (common types, utilities)
cargo new shared --lib
cat > shared/Cargo.toml << 'EOF'
[package]
name = "shared"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { workspace = true }
serde_json = { workspace = true }
uuid = { workspace = true }
thiserror = { workspace = true }
EOF

# 1.2 Akkadian DSL (Query Language)
cargo new akkadian_dsl --lib
cat > akkadian_dsl/Cargo.toml << 'EOF'
[package]
name = "akkadian_dsl"
version = "3.4.0"
edition = "2021"
description = "Akkadian v3.4 - Fuzzy Logic Query Language"

[dependencies]
shared = { path = "../shared" }
nom = "7"
serde = { workspace = true }
thiserror = { workspace = true }
petgraph = { workspace = true }
EOF

# 1.3 Fuzzy Logic Engine
cargo new fuzzy_engine --lib
cat > fuzzy_engine/Cargo.toml << 'EOF'
[package]
name = "fuzzy_engine"
version = "1.0.0"
edition = "2021"
description = "Fuzzy Logic Rules Engine for BahyWay"

[dependencies]
shared = { path = "../shared" }
serde = { workspace = true }
thiserror = { workspace = true }
EOF

# 1.4 Score Engine
cargo new score_engine --lib
cat > score_engine/Cargo.toml << 'EOF'
[package]
name = "score_engine"
version = "1.0.0"
edition = "2021"
description = "Quality Scoring Engine for BahyWay"

[dependencies]
shared = { path = "../shared" }
fuzzy_engine = { path = "../fuzzy_engine" }
serde = { workspace = true }
EOF

# ============================================================
# 2. APPLICATIONS
# ============================================================

echo "🎨 Creating Applications..."

# 2.1 OntoWay (Knowledge Graph Editor)
cargo new ontoway --bin
cat > ontoway/Cargo.toml << 'EOF'
[package]
name = "ontoway"
version = "1.0.0"
edition = "2021"
description = "OntoWay - Visual Knowledge Graph Editor"

[dependencies]
shared = { path = "../shared" }
akkadian_dsl = { path = "../akkadian_dsl" }
fuzzy_engine = { path = "../fuzzy_engine" }

# Web framework
axum = { workspace = true }
tower = { workspace = true }
tower-http = { workspace = true }
tokio = { workspace = true }

# GraphQL
async-graphql = { workspace = true }
async-graphql-axum = { workspace = true }

# Database
tokio-postgres = { workspace = true }
deadpool-postgres = { workspace = true }

# Graph algorithms
petgraph = { workspace = true }

# Serialization
serde = { workspace = true }
serde_json = { workspace = true }

# Logging
tracing = { workspace = true }
tracing-subscriber = { workspace = true }

# Error handling
anyhow = { workspace = true }
thiserror = { workspace = true }
EOF

# 2.2 TribeWay (Tribal Visualization)
cargo new tribeway --bin
cat > tribeway/Cargo.toml << 'EOF'
[package]
name = "tribeway"
version = "1.0.0"
edition = "2021"
description = "TribeWay - Sovereign Tribal Identity & Visualization"

[dependencies]
shared = { path = "../shared" }
fuzzy_engine = { path = "../fuzzy_engine" }
score_engine = { path = "../score_engine" }

# Web framework
axum = { workspace = true }
tower = { workspace = true }
tower-http = { workspace = true }
tokio = { workspace = true }

# Database
tokio-postgres = { workspace = true }
sqlx = { workspace = true }

# 3D Graphics
wgpu = { workspace = true }
winit = { workspace = true }

# Serialization
serde = { workspace = true }
serde_json = { workspace = true }

# Logging
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
EOF

# 2.3 NajafWay (Cemetery Management & Stress Testing)
cargo new najafway --bin
cat > najafway/Cargo.toml << 'EOF'
[package]
name = "najafway"
version = "1.0.0"
edition = "2021"
description = "NajafWay - Cemetery Management & Stress Testing"

[dependencies]
shared = { path = "../shared" }
akkadian_dsl = { path = "../akkadian_dsl" }
fuzzy_engine = { path = "../fuzzy_engine" }
score_engine = { path = "../score_engine" }

# Web framework
axum = { workspace = true }
tokio = { workspace = true }

# Database
tokio-postgres = { workspace = true }
sqlx = { workspace = true }

# CSV processing
csv = "1.3"

# Serialization
serde = { workspace = true }
serde_json = { workspace = true }

# Performance testing
criterion = { workspace = true }

# Logging
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
EOF

# 2.4 ParticlesWay (Sovereign Gem Activation)
cargo new particlesway --bin
cat > particlesway/Cargo.toml << 'EOF'
[package]
name = "particlesway"
version = "1.0.0"
edition = "2021"
description = "ParticlesWay - Sovereign Gem Activation Engine"

[dependencies]
shared = { path = "../shared" }
fuzzy_engine = { path = "../fuzzy_engine" }
score_engine = { path = "../score_engine" }

# Database
tokio-postgres = { workspace = true }

# Async runtime
tokio = { workspace = true }
async-trait = { workspace = true }

# Serialization
serde = { workspace = true }
serde_json = { workspace = true }

# Logging
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
EOF

# ============================================================
# 3. CREATE DIRECTORY STRUCTURE
# ============================================================

echo "📁 Creating Directory Structure..."

# Shared library structure
mkdir -p shared/src/{domain,utils}

# Akkadian DSL structure
mkdir -p akkadian_dsl/src/{parser,executor,optimizer}

# Fuzzy Engine structure
mkdir -p fuzzy_engine/src/{rules,inference}

# Score Engine structure
mkdir -p score_engine/src/{calculators,validators}

# OntoWay structure
mkdir -p ontoway/src/{api,domain,services,repository,graphql}
mkdir -p ontoway/frontend/{src,public}

# TribeWay structure
mkdir -p tribeway/src/{api,domain,services,repository,visualization}
mkdir -p tribeway/frontend/{src,public}

# NajafWay structure
mkdir -p najafway/src/{api,domain,services,repository,stress_test}
mkdir -p najafway/data

# ParticlesWay structure
mkdir -p particlesway/src/{domain,services,activation}

# ============================================================
# 4. CREATE SHARED TYPES
# ============================================================

echo "📝 Creating Shared Types..."

cat > shared/src/lib.rs << 'RUST'
//! Shared types and utilities for BahyWay Ecosystem

pub mod domain;
pub mod utils;
pub mod error;

// Re-export commonly used types
pub use domain::*;
pub use error::BahyWayError;
pub use uuid::Uuid;

pub type Result<T> = std::result::Result<T, BahyWayError>;
RUST

cat > shared/src/domain.rs << 'RUST'
//! Core domain models shared across BahyWay applications

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// BDBWay 16-byte Sovereign Identity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SovereignIdentity {
    /// Raw 16-byte identity
    pub bytes: [u8; 16],
}

impl SovereignIdentity {
    pub fn new(
        uuid: Uuid,
        tribe_id: u32,
        red: u8,
        green: u8,
        blue: u8,
    ) -> Self {
        let mut bytes = [0u8; 16];
        
        // [0-7]: UUID partial
        bytes[0..8].copy_from_slice(&uuid.as_bytes()[0..8]);
        
        // [8-11]: Tribe ID
        bytes[8..12].copy_from_slice(&tribe_id.to_be_bytes());
        
        // [12]: Red (Tribal Color)
        bytes[12] = red;
        
        // [13]: Green (Quality)
        bytes[13] = green;
        
        // [14]: Blue (Temporal)
        bytes[14] = blue;
        
        // [15]: Flags
        bytes[15] = 0b00000000;
        
        Self { bytes }
    }
    
    pub fn uuid_partial(&self) -> [u8; 8] {
        let mut uuid = [0u8; 8];
        uuid.copy_from_slice(&self.bytes[0..8]);
        uuid
    }
    
    pub fn tribe_id(&self) -> u32 {
        u32::from_be_bytes([
            self.bytes[8],
            self.bytes[9],
            self.bytes[10],
            self.bytes[11],
        ])
    }
    
    pub fn tribal_color(&self) -> u8 {
        self.bytes[12]
    }
    
    pub fn quality(&self) -> u8 {
        self.bytes[13]
    }
    
    pub fn temporal(&self) -> u8 {
        self.bytes[14]
    }
    
    pub fn flags(&self) -> u8 {
        self.bytes[15]
    }
}

/// Ethnicity classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Ethnicity {
    Arab,
    Kurdish,
    Turkmen,
    Assyrian,
    Yazidi,
    Mandaean,
    Shabak,
    Mixed,
    Unknown,
}

/// Quality tier classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum QualityTier {
    Sovereign = 200,    // 200-255
    Active = 140,       // 140-199
    Poor = 100,         // 100-139
    NonActive = 0,      // 0-99
}

impl QualityTier {
    pub fn from_score(score: u8) -> Self {
        match score {
            200..=255 => QualityTier::Sovereign,
            140..=199 => QualityTier::Active,
            100..=139 => QualityTier::Poor,
            _ => QualityTier::NonActive,
        }
    }
}

/// Spatial position
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Position {
    pub lon: f32,
    pub lat: f32,
    pub elevation: f32,
}

/// Tribal node for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TribalNode {
    pub id: String,
    pub name: String,
    pub name_ar: String,
    pub ethnicity: Ethnicity,
    pub color: u8,
    pub quality: u8,
    pub position: Position,
    pub population: Option<u32>,
    pub data_count: u32,
}
RUST

cat > shared/src/error.rs << 'RUST'
//! Error types for BahyWay Ecosystem

use thiserror::Error;

#[derive(Error, Debug)]
pub enum BahyWayError {
    #[error("Database error: {0}")]
    Database(#[from] tokio_postgres::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("UUID error: {0}")]
    Uuid(#[from] uuid::Error),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}
RUST

# ============================================================
# 5. CREATE ROOT CARGO.TOML
# ============================================================

echo "📦 Creating Root Cargo.toml..."

cat > Cargo.toml << 'EOF'
[workspace]
resolver = "2"

members = [
    "bdbway_extension",
    "shared",
    "akkadian_dsl",
    "fuzzy_engine",
    "score_engine",
    "ontoway",
    "tribeway",
    "najafway",
    "particlesway",
]

[workspace.dependencies]
# Database
tokio-postgres = { version = "0.7", features = ["with-uuid-1", "with-serde_json-1"] }
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-native-tls", "uuid", "json"] }
deadpool-postgres = "0.12"

# Async runtime
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# UUID
uuid = { version = "1", features = ["v4", "serde"] }

# Web framework
axum = { version = "0.7", features = ["macros", "ws"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace"] }

# GraphQL
async-graphql = { version = "7", features = ["uuid", "chrono"] }
async-graphql-axum = "7"

# 3D Graphics
wgpu = "0.18"
winit = "0.29"

# Graph algorithms
petgraph = "0.6"

# Error handling
thiserror = "1"
anyhow = "1"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Testing
criterion = "0.5"

[profile.release]
opt-level = 3
lto = true
codegen-units = 1

[profile.dev]
opt-level = 0
debug = true
EOF

# ============================================================
# 6. CREATE README
# ============================================================

cat > README.md << 'EOF'
# RustLab - BahyWay Ecosystem

Monorepo for all BahyWay Rust applications and libraries.

## Architecture

```
RustLab/
├── Core Libraries
│   ├── shared/           - Common types and utilities
│   ├── akkadian_dsl/     - Akkadian v3.4 query language
│   ├── fuzzy_engine/     - Fuzzy logic rules engine
│   └── score_engine/     - Quality scoring engine
│
├── Applications
│   ├── ontoway/          - Knowledge Graph Editor
│   ├── tribeway/         - Tribal Visualization
│   ├── najafway/         - Cemetery Management
│   └── particlesway/     - Sovereign Gem Activation
│
└── Extensions
    └── bdbway_extension/ - PostgreSQL extension (existing)
```

## Build All

```bash
cargo build --workspace
```

## Build Specific App

```bash
cargo build -p ontoway
cargo build -p tribeway
cargo build -p najafway
```

## Run Tests

```bash
cargo test --workspace
```

## Run Application

```bash
cargo run -p ontoway
cargo run -p tribeway --release
```

## Shared Dependencies

All projects share dependencies via workspace configuration.
Update dependencies in root `Cargo.toml` `[workspace.dependencies]`.
EOF

# ============================================================
# 7. INITIALIZE GIT
# ============================================================

echo "🔧 Initializing Git..."

cat > .gitignore << 'EOF'
/target
**/*.rs.bk
*.pdb
.DS_Store
.env
.vscode
.idea
EOF

# ============================================================
# DONE
# ============================================================

echo ""
echo "============================================================"
echo "✅ RustLab Workspace Created Successfully!"
echo "============================================================"
echo ""
echo "📦 Created Crates:"
echo "  - shared (library)"
echo "  - akkadian_dsl (library)"
echo "  - fuzzy_engine (library)"
echo "  - score_engine (library)"
echo "  - ontoway (application)"
echo "  - tribeway (application)"
echo "  - najafway (application)"
echo "  - particlesway (application)"
echo ""
echo "🚀 Next Steps:"
echo "  1. cd /workspace"
echo "  2. cargo build --workspace"
echo "  3. cargo test --workspace"
echo "  4. cargo run -p ontoway"
echo ""
echo "============================================================"
