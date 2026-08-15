# HeptaScript Spatial Index: E7 Lattice Design
## Based on Maryna Viazovska's Modular Form Theory

---

## Background

Maryna Viazovska (Fields Medal 2022) proved that the **E8 lattice** gives the optimal sphere
packing in dimension 8, using quasi-modular Eisenstein series (modular forms). Her method uses
**theta functions** and **linear programming bounds** to construct a "magic function" that proves
no packing can beat E8 in 8D.

EnkiDB operates in **7-dimensional space**: `position[3] + momentum[3] + scalar[1]`.
The 7D analogue is the **E7 lattice** — optimal sphere packing in dimension 7.

---

## E7 Lattice Properties (Relevant to Indexing)

| Property | Value | EnkiDB Implication |
|---|---|---|
| Dimension | 7 | Matches HeptaScript particle space |
| Kissing number | 126 | Max index node fanout = 126 |
| Minimal vectors | 126 | 126 natural query directions |
| Packing density | π³/105 ≈ 0.2958 | Theoretical space efficiency bound |
| Symmetry group | W(E7), order 2903040 | Rich query symmetries to exploit |
| Gram matrix | Known, rational entries | Exact integer arithmetic possible |

---

## Four Design Principles from Viazovska → HeptaScript

### 1. E7 Voronoi Cells as Index Partitions

Instead of axis-aligned B-tree or R-tree boxes, partition 7D query space using the
**Voronoi cells of the E7 lattice**. Each cell is a polytope equidistant between
lattice points — the natural "bucket" for spatially local particles.

```
Traditional R-tree:  axis-aligned bounding boxes  (suboptimal in 7D)
E7 index:            E7 Voronoi polytopes          (provably optimal in 7D)
```

**Implementation note:** E7 Voronoi cells are the dual of the E7 lattice's Delaunay
triangulation. The nearest-lattice-point problem (quantisation) can be solved in O(7²) = O(49)
operations using the known E7 reduction algorithm.

---

### 2. Theta Series as Cardinality Estimator

Viazovska's proof uses the **theta function** of a lattice L:

```
Θ_L(q) = Σ_{n=0}^{∞} r_L(n) · q^n
```

where `r_L(n)` = number of lattice points at squared distance `n` from the origin.

In EnkiDB, this becomes a **shell histogram** — the query planner knows exactly how
many particles lie within each distance shell before executing a range query:

```rust
// HeptaScript query planner cardinality estimate
fn estimate_range_count(center: &Particle7D, radius: f32, index: &E7Index) -> u64 {
    let n = (radius / index.lattice_scale).powi(2) as usize;
    index.theta_series[..=n].iter().sum()  // sum r_L(0..n)
}
```

The E7 theta series begins: `1 + 126q + 756q² + 2072q³ + ...`
(126 = kissing number, confirming the first shell has exactly 126 neighbors)

---

### 3. Modular Symmetry → Position ↔ Momentum Query Duality

Modular forms satisfy `f(-1/τ) = τ^k · f(τ)` — a duality under inversion.

In HeptaScript's 7D space, this maps to **position-momentum duality**:
- A range query in `position[0..2]` subspace has a **known equivalent cost**
  in `momentum[0..2]` subspace
- The query optimizer can choose the cheaper subspace automatically
- Projection switching (`%%dubsar proj-phase-space`) exploits this duality

```
position query  ←→  momentum query  (dual via modular inversion)
spatial locality ←→  spectral locality
```

---

### 4. Linear Programming Bound → Query Cost Proof

Viazovska's LP bound proves E8 packing is optimal by constructing a function `f` where:
- `f(0) = 1`
- `f(x) ≤ 0` for `|x| ≥ 1`
- `f̂(0) = f(0)` (Poisson summation constraint)

In HeptaScript, the same LP framework gives a **provable lower bound on query cost**:
no 7D spatial index can answer a k-nearest-neighbor query in fewer comparisons than
the E7 kissing-number bound implies. This lets us certify that the E7 index is optimal.

---

## HeptaScript Magic Cell

```
%%dubsar query-lattice
{
  "basis": "E7",
  "metric": "theta",
  "fanout": 126,
  "cardinality_estimator": "theta_series",
  "projection_duality": true
}
```

---

## Phased Implementation Plan

### Phase 1 — Theta Series Estimator (Low effort, high value)
- Pre-compute E7 theta series coefficients up to n=1000
- Use as cardinality estimator in `%%dubsar stats`
- File: `dubsar-kernel/src/index/theta.rs`

### Phase 2 — E7 Nearest-Lattice-Point (Core indexing)
- Implement E7 lattice quantisation (Agrell-Vardy algorithm, O(49) ops)
- Replace linear scan in `ParticleCloud::project()` with E7 bucket lookup
- File: `dubsar-kernel/src/index/e7_lattice.rs`

### Phase 3 — Voronoi Cell Range Queries
- Build Voronoi adjacency graph for E7 (126 neighbors per node)
- Implement `range_query(center, radius)` via BFS on Voronoi graph
- File: `dubsar-kernel/src/index/voronoi.rs`

### Phase 4 — Position-Momentum Duality in Query Planner
- Detect query subspace (position vs momentum dims)
- Auto-switch projection using modular duality cost model
- File: `dubsar-kernel/src/magics.rs` (extend `%%dubsar step` and `%%dubsar stats`)

---

## Key References

- Viazovska, M. (2017). *The sphere packing problem in dimension 8.*
  Annals of Mathematics 185(3), 991–1015. arXiv:1603.04246

- Cohn, H. & Kumar, A. (2009). *Optimality and uniqueness of the Leech lattice among lattices.*
  Annals of Mathematics 170(3), 1003–1050.

- Conway, J.H. & Sloane, N.J.A. (1999). *Sphere Packings, Lattices and Groups.*
  Chapter 7: The E7 and E8 lattices.

- Agrell, E. & Vardy, A. (2000). *Closest point search in lattices.*
  IEEE Trans. Information Theory 48(8), 2201–2214.

---

## Connection to EnkiDB Storage Fragmentation Model

The **two-tribe storage fragmentation simulation** maps naturally to E7 geometry:

- **Tribe A (contiguous, low scalar)** → dense E7 sub-lattice region (low fragmentation index)
- **Tribe B (scattered, high scalar)** → sparse E7 region (high fragmentation index)
- **Free particles (chaotic)** → lattice quantisation error (off-lattice noise)

The scalar dimension (index 6 in the 7D vector) encodes deviation from the E7 lattice —
a particle with `scalar ≈ 0` is near an E7 lattice point (well-packed, contiguous storage),
while `scalar ≈ 1` means maximum fragmentation (far from any lattice point).

This gives a **geometric definition of storage fragmentation**:
```
fragmentation_index = dist(particle, nearest_E7_point) / E7_packing_radius
```
