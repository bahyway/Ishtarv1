# kinetic-engine — Sovereign 7D Physics Engine
𒈨𒄖𒊕𒀀𒉈𒌓𒌷 *ME · GU · SAG · A · IZI · UD · URU — The Seven Sovereign Dimensions*

> **Layer 5 · Operational Engines** | Vec7D | Euler Integration | Fuzzy Health | Plimpton 322 | Zero external deps

---

## W5H2 Manual

### WHO — 𒀭 Who Uses This Crate

| Persona | Role |
|---|---|
| **dmw-engine** | Runs kinetic analysis on SQL query particles (SalesOrder correlated subqueries) |
| **fuzzy-engine** | Imports `ScoringEngine` and `HealthClassification` as foundation |
| **vault-engine** (future) | Feeds `PhysicalForce` (Gray-Rot %) into the accumulator |
| **najaf-engine** | Uses `ScoringEngine::sovereign()` (τ = 0.99) for grave-level precision |
| **ammas-engine** | Shares the kinetic equation df/dt = I_phys + I_mem + I_learn (different implementation level) |
| **AkkadianAOL compiler** (`aaol`) | Compiles `.akk` particle equations targeting the Rust backend via `SovereignAccumulator` |
| **hepta-score** | Uses `Vec7D` as the canonical 7D score vector type |

---

### WHAT — 𒁾 What This Tablet Contains

`kinetic-engine` is the **executable mathematical substrate** of BahyWay.Ecosystem.  
It implements the sovereign 7D vector space and the discrete kinetic force integrator
that drives particle evolution through the Heptagon analytical levels.

#### The Five Modules

| Module | Types | Purpose |
|---|---|---|
| `vec7d` | `Vec7D` | 7D vector with full algebra — Add/Sub/Mul/Neg/Dot/Magnitude/Alignment/Sigma/HealthScore |
| `force` | `HeptaDimension`, `ForceGenerator`, `PhysicalForce`, `MemoryForce`, `LearningForce`, `ConstantForce` | The three BahyWay force terms (I_phys / I_mem / I_learn) |
| `accumulator` | `KineticParticle`, `SovereignAccumulator` | Millington Euler integrator in 7D space |
| `fuzzy` | `MembershipFunction`, `FuzzyRule`, `ScoringEngine`, `HealthClassification` | Triangular MF + weighted scoring engine |
| `plimpton` | `PlimptonAnchors`, `PlimptonTrigger`, `TriggerAction` | Babylonian Plimpton 322 boundary triggers |

#### The Kinetic Equation

```
df/dt = I_phys + I_mem + I_learn

I_phys  — vault-engine Gray-Rot (URU) + IO friction (A)
I_mem   — EnkiDB cardinality staleness (GU) + plan drift (UD)
I_learn — Oracle Egg rewrites (ME) + sargability (SAG) + CPU reduction (IZI)
```

#### The 7 Heptagon Dimensions

| Field | Sumerian | Analytical Meaning | Primary Force |
|---|---|---|---|
| `me`  | ME 𒈨  | Intent / Set Theory | I_learn (oracle_improvement) |
| `gu`  | GU 𒄖  | Mass / Cardinality  | I_mem (staleness) |
| `sag` | SAG 𒊕 | Gravity / Selectivity | I_learn (sargability_gain) |
| `a`   | A 𒀀   | Viscosity / IO Friction | I_phys (io_friction) |
| `izi` | IZI 𒉈 | Heat / CPU Burn | I_learn (cpu_reduction) |
| `ud`  | UD 𒌓  | Volatility / Cache | I_mem (plan_drift) |
| `uru` | URU 𒌷 | Strength / Index Health | I_phys (gray_rot_pct) |

---

### WHEN — 𒌓 When Is This Invoked

```
One Heptagon analysis cycle:

1. Particle enters with 7D position from KAKI-identified entity
   │
   ▼
2. SovereignAccumulator::integrate() per cycle
   │  F = I_phys + I_mem + I_learn
   │  a = F * inverse_mass
   │  v += a * dt
   │  v *= damping^dt      ← health decay
   │  x += v * dt
   │
   ▼
3. ScoringEngine::classify() after each cycle
   │  Sovereign (τ ≥ 0.70)  → write to EnkiDB, issue .akk sovereign seal
   │  Warning   (τ ≥ 0.42)  → queue for next analysis cycle
   │  Critical  (τ < 0.42)  → route to repair via dominant_failure()
   │
   ▼
4. dominant_failure() → repair routing
   │  URU → index_rebuild.akk
   │  GU  → stats_refresh.akk
   │  ME  → cartesian_rewrite.akk
   │  A   → loop_to_hash.akk
```

---

### WHERE — 𒆳 Architectural Position

```
Layer 5: Operational Engines

kinetic-engine (Vec7D · Forces · Accumulator · Fuzzy · Plimpton)
    │
    │  imported by
    ├──► dmw-engine       (SQL query particle analysis)
    ├──► fuzzy-engine     (ScoringEngine foundation)
    ├──► hepta-score      (Vec7D as canonical score vector)
    ├──► najaf-engine     (sovereign τ=0.99 threshold)
    └──► vault-engine     (PhysicalForce from Gray-Rot reports)

    ▲  imports from
    └── (zero deps — pure std Rust)
```

**Distinguished from similar crates:**
- `akkadi-ir/kinetic.rs` — IR quality force *kinds* for the AkkadianAOL compiler (Layer 9); different abstraction
- `ammas-engine/kinetic.rs` — AMMAS differential equation (J_phys + J_mem + J_learn) at Layer 4.5; shares the equation name but operates on a different substrate

---

### WHY — 𒀊 Why This Exists

**The problem in v3.5:**  
SQL query analysis used ad-hoc thresholds scattered across 23 crates — `if io_friction > 0.80 { ... }` with no shared mathematical model, no trajectory prediction, and no routing logic.

**The sovereign solution:**  
One mathematical substrate shared by all engines. Every particle follows the same integration loop. The kinetic equation is testable, reproducible, and can be compiled to any AkkadianAOL backend.

**Why Millington's physics engine?**  
Millington's *Game Physics Engine Development* Chapter 2 provides a rigorously tested Euler integrator for mass-damping-force systems. Applied to 7D space, it gives particle health a physically meaningful trajectory — not just a static score.

**Why Plimpton 322?**  
The 3,700-year-old Babylonian tablet provides exact rational boundary values — no floating-point approximation artifacts at tier transitions. Row 11 (3,4,5) gives the canonical health tier at spread = 9/25 = 0.36 exactly.

**Why zero external dependencies?**  
kinetic-engine is imported by every engine at Layer 5. Any external dependency would propagate to all of them. Pure `std` ensures zero transitive dependency risk.

---

### HOW — 𒅗 How It Works

#### Basic kinetic integration

```rust
use kinetic_engine::{
    KineticParticle, SovereignAccumulator,
    PhysicalForce, LearningForce, Vec7D,
    ScoringEngine, HealthClassification,
};

// 1. Create a particle (cardinality = mass)
let raw = Vec7D::new(0.3, 0.2, 0.4, 0.1, 0.3, 0.2, 0.1);
let mut particle = KineticParticle::new(raw, 6069.0, 0.95);

// 2. Diagnose entry classification
let engine = ScoringEngine::dmw_default();
assert_eq!(engine.classify(&particle.position), HealthClassification::Critical);

// 3. Apply forces: df/dt = I_phys + I_mem + I_learn
let mut acc = SovereignAccumulator::default_cycle();
acc.add_force(Box::new(PhysicalForce::new(0.10, 0.20)));  // I_phys
acc.add_force(Box::new(LearningForce::full()));             // I_learn

// 4. Integrate 5 Heptagon cycles
acc.integrate_cycles(&mut particle, 5);

// 5. Check health trajectory
let dh_dt = acc.health_velocity(&particle);
assert!(dh_dt >= 0.0, "trajectory must be improving after Oracle");
```

#### Dead zone detection (Self-Immune)

```rust
// Particle is still Sovereign but trajectory is declining
let slightly_degraded = Vec7D::new(0.85, 0.85, 0.85, 0.85, 0.85, 0.85, 0.85);
let particle = KineticParticle::new(slightly_degraded, 1.0, 0.95);

assert!(engine.is_sovereign(&particle.position)); // still above τ=0.70

let dh_dt = acc.health_velocity(&particle);
if dh_dt < 0.0 {
    // Self-Immune trigger: sovereign NOW but declining → schedule maintenance
}
```

#### Plimpton 322 boundary detection

```rust
use kinetic_engine::{PlimptonTrigger, PlimptonAnchors};

let triggers = PlimptonTrigger::sovereign_set(); // 7 triggers
let anchors  = PlimptonAnchors::sovereign();

// Row 11 (3,4,5) — canonical health level
// spread = 0.36 exactly — no floating-point drift
assert_eq!(anchors.ud, (3.0_f64 / 5.0).powi(2));
```

---

### HOW MUCH — 𒀸 Sovereign Metrics

| Metric | Value |
|---|---|
| Source files | 6 (vec7d, force, accumulator, fuzzy, plimpton, lib) |
| Integration test file | 1 (tests/integration.rs — 10 scenarios) |
| Lines of Rust | ~1,200 |
| Unit tests | 60+ (across all modules) |
| Integration test scenarios | **10** |
| Vec7D dimensions | **7** (ME, GU, SAG, A, IZI, UD, URU) |
| Force types | **4** (Physical, Memory, Learning, Constant) |
| Health classifications | **3** (Sovereign, Warning, Critical) |
| Threshold presets | **4** (Relaxed 0.50 / Standard 0.70 / Strict 0.90 / Sovereign 0.99) |
| Plimpton triggers | **7** (rows 1,3,5,7,9,11,13) |
| External dependencies | **0** — pure `std` Rust |

---

## Sovereign Constraints

- `#![forbid(unsafe_code)]`
- `Vec7D::HEALTHY_ORBIT = Vec7D::UNIT` — healthy orbit is the unit cube corner `(1,1,1,1,1,1,1)`
- Sovereign-sealed particles (`inverse_mass = 0.0`) are completely immovable — no force can change them
- `PlimptonAnchors::sovereign().ud` = `(3/5)² = 0.36` exactly — Row 11 is the canonical health tier
- Mass encodes **cardinality** (row count), NOT schema width — the Particle Unit invariant
- Plimpton spreads are strictly decreasing across rows (Row 1 > Row 3 > … > Row 13)

---

## Files

```
crates/kinetic-engine/
├── Cargo.toml                 (zero external deps)
├── README.md                  (this file)
└── src/
│   ├── lib.rs                 — module declarations + flat re-exports
│   ├── vec7d.rs               — Vec7D: 7D vector, all algebra, ZERO/UNIT/HEALTHY_ORBIT
│   ├── force.rs               — HeptaDimension, ForceGenerator trait, PhysicalForce,
│   │                            MemoryForce, LearningForce, ConstantForce
│   ├── accumulator.rs         — KineticParticle, SovereignAccumulator (Euler integrator)
│   ├── fuzzy.rs               — MembershipFunction, FuzzyRule, ScoringEngine,
│   │                            HealthClassification (3 states, 4 threshold presets)
│   └── plimpton.rs            — PlimptonAnchors, PlimptonTrigger, TriggerAction
└── tests/
    └── integration.rs         — 10 end-to-end scenarios (DMW real cases)
```

## Quick Start

```bash
# From the workspace root:
cargo test -p kinetic-engine
cargo test -p kinetic-engine --test integration
```
