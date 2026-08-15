# BahyWay.Ecosystem v4.0 — Sovereign Algebra Glossary

> *𒁾 DUB.SAR — The Clay Tablet of Mathematical Foundations*

**Author**: Bahaa Fadam — BahyWay Sovereign Ecosystem
**Version**: 4.0.0 | **Date**: 2026-06-05
**Purpose**: Reference knowledge for all algebraic concepts before and during
Rust implementation. Each entry defines the concept, states its sovereign action
in BahyWay, and maps its escalation — what it depends on (↑) and what depends
on it (↓).

---

## How to Read This Glossary

Each entry follows this structure:

```
## [Concept Name]
Layer: which OOO layer it belongs to
Crate: where it is or will be implemented

Definition     — what it is mathematically (pure mathematics, no BahyWay)
BahyWay Action — what it does operationally in the sovereign ecosystem
Escalation ↑   — concepts this one is built ON (must exist first)
Escalation ↓   — concepts that are built ON THIS (enabled by this)
Sovereign Rule — the invariant that must never be violated
```

**Reading order for implementation:** follow the escalation ↑ arrows bottom-up.
Build what has no ↑ dependencies first. Never implement a concept before its
↑ dependencies are implemented and tested.

---

## Part I — Foundational Mathematics (Layer 2: Simplicial + JNF)

---

### Field

**Layer**: 2 — Simplicial + JNF (implicit foundation)
**Crate**: `bahyway-algebra`

**Definition**: A set F with two operations (+ and ×) satisfying commutativity,
associativity, distributivity, identity elements, and inverses. The real
numbers ℝ and rational numbers ℚ are fields. A field is the minimal structure
needed to do linear algebra.

**BahyWay Action**: The quality scores in VGCA-Σ live in ℝ (or its fixed-point
approximation). The KAKI hash computations live in ℤ/2^n (a finite field). Every
algebraic structure in BahyWay is built over one of these two fields.

**Escalation ↑**: None — this is a foundational primitive.

**Escalation ↓**: Vector Space, Algebra, Ring

**Sovereign Rule**: The fixed-point field for B11 uses divisor 240 exactly.
The field ℤ/240 is the sovereign scoring field — never ℤ/255.

---

### Vector Space

**Layer**: 2 — Simplicial + JNF
**Crate**: `bahyway-algebra`

**Definition**: A set V over a field F with addition and scalar multiplication,
satisfying 8 axioms (closure, associativity, commutativity, identity, inverse,
scalar distributivity, field distributivity, compatibility). Elements of V are
called vectors.

**BahyWay Action**: The 7D VGCA-Σ quality space is a vector space V = ℝ⁷.
Every particle's Hepta-Score is a vector in this space. The DomainCentroid is
a special vector (the origin of the "healthy" region).

**Escalation ↑**: Field

**Escalation ↓**: Inner Product Space, Manifold, Clifford Algebra, VGCA-Σ

**Sovereign Rule**: The 7D quality vector encodes assessments only — it is never
stored in KAKI bytes (Structural-Facts-Only Rule §2.4).

---

### Inner Product Space

**Layer**: 2 — Simplicial + JNF
**Crate**: `bahyway-algebra`

**Definition**: A vector space V equipped with an inner product
⟨·,·⟩ : V × V → F satisfying conjugate symmetry, linearity, and
positive-definiteness. Gives a notion of angle and length.

**BahyWay Action**: The Hepta-Score formula H(P) = 1/(1 + √Σwᵢ(Pᵢ−Tᵢ)²)
is the weighted inner product distance from particle P to tribe ideal T.
The weights wᵢ make the inner product non-Euclidean — each quality dimension
contributes differently to the sovereign distance.

**Escalation ↑**: Vector Space

**Escalation ↓**: Riemannian Manifold, DGA, H(P) formula

**Sovereign Rule**: The inner product weights wᵢ are tribe-specific and
calibrated from GEM-lane particles only (DomainCentroid calibration).

---

### Algebra (over a Field)

**Layer**: 4 — TOP Algebra
**Crate**: `bahyway-algebra`

**Definition**: A vector space A over field F equipped with a bilinear
multiplication A × A → A. The multiplication may or may not be commutative,
associative, or have an identity. Examples: matrix algebra (associative),
octonions (non-associative), Clifford algebra (associative).

**BahyWay Action**: TOP Algebra = Tribe Algebra ⊕ Orbits Calculus ⊕ Particles
Algebra is a composite algebra over ℝ. Each sub-algebra has its own
multiplication rules. The composition is the legal operation set for every
sovereign computation in the system.

**Escalation ↑**: Vector Space, Field

**Escalation ↓**: Clifford Algebra, Octonions, Tribe Algebra, Particles Algebra

**Sovereign Rule**: Operations not defined in the TOP Algebra multiplication
table are Forbidden Operations (ADR-008, Decision 7).

---

### Simplex

**Layer**: 2 — Simplicial + JNF
**Crate**: `bahyway-algebra`

**Definition**: A simplex is the simplest possible geometric shape in n
dimensions. A 0-simplex is a point. A 1-simplex is a line segment. A
2-simplex is a triangle. A 3-simplex is a tetrahedron. An n-simplex is the
convex hull of n+1 affinely independent points.

**BahyWay Action**: A KAKI particle is a 0-simplex (a sovereign point in
quality space). An Event-Kaki is a 1-simplex (a directed edge — a state
transition from one quality position to another). A CrossTribe-Kaki is a
2-simplex (a triangular face connecting two particles and their relation).

**Escalation ↑**: Vector Space

**Escalation ↓**: Simplicial Complex, KAKI physical types, VGCA-Δ

**Sovereign Rule**: A 0-simplex (Identity-Kaki) is never modified — it is a
point that does not move. Only its surrounding complex grows.

---

### Simplicial Complex

**Layer**: 2 — Simplicial + JNF
**Crate**: `bahyway-algebra`

**Definition**: A collection K of simplices closed under the face relation:
if σ ∈ K and τ is a face of σ, then τ ∈ K. A simplicial complex is a
topological space built by gluing simplices together.

**BahyWay Action**: A tribe is a simplicial complex. Its particles (0-simplices)
are its vertices. Its Event-Kakis (1-simplices) are its edges. Its CrossTribe
relations (2-simplices) are its faces. The VGCA-Δ geometric cleansing gate
ensures that every Event-Kaki appended to the Journal is a valid edge — that
the Journal is a topologically sound simplicial complex, not a broken mesh.

**Escalation ↑**: Simplex

**Escalation ↓**: JNF (applied to the complex's transition operators),
VGCA-Δ (validity gate for edges), Graph Algebra (graph is a 1-complex)

**Sovereign Rule**: A Journal that fails VGCA-Δ validation contains an invalid
simplex. The invalid edge is rejected and logged as a DEAD-lane Event-Kaki.
The complex remains sound.

---

### Jordan Normal Form (JNF)

**Layer**: 2 — Simplicial + JNF
**Crate**: `bahyway-algebra`

**Definition**: Every square matrix A over an algebraically closed field is
similar to a matrix J in Jordan normal form: J = P⁻¹AP, where J consists of
Jordan blocks along the diagonal. Each Jordan block Jₖ(λ) is a k×k matrix
with eigenvalue λ on the diagonal and 1s on the superdiagonal.

**BahyWay Action**: A particle's Journal over time can be represented as a
linear operator J acting on its quality state space S. JNF decomposes J into:
- **Jordan blocks of size 1** (diagonal): independently evolving EAV attributes
  (the common case — 99% of particles)
- **Jordan blocks of size > 1** (off-diagonal entries): causally coupled EAV
  attributes (e.g., credit_score driving transaction_limit)

The Hepta-Score H(P) is the PCA projection (the diagonal/size-1 case of JNF).
Full JNF is invoked only when an AAOL covariance declaration couples attributes.

**Escalation ↑**: Algebra, Vector Space, Simplicial Complex

**Escalation ↓**: Hepta-Score H(P), AMMAS kinetic equation, Snapshot Index
(each Jordan block = one independently snapshotable attribute stream)

**Sovereign Rule**: JNF decomposition uses only sovereign-computed eigenvalues
(from the particle's own Journal data). No external matrix library.
QUALITY_DIVISOR = 240 always.

---

### Eigenvalue / Eigenvector

**Layer**: 2 — Simplicial + JNF
**Crate**: `bahyway-algebra`

**Definition**: For a linear operator A, a non-zero vector v is an eigenvector
with eigenvalue λ if Av = λv. The eigenvalue measures how much the operator
stretches or shrinks v without changing its direction.

**BahyWay Action**: The 7 eigenvalues of a particle's state-transition operator
(in the diagonal JNF case) are the 7 quality drift rates — one per Hepta
dimension. A large eigenvalue on the "freshness" dimension means the particle's
freshness is decaying rapidly. A zero eigenvalue means that dimension is stable.
The Hepta-Score aggregates these 7 drift rates into one scalar B11.

**Escalation ↑**: JNF, Inner Product Space

**Escalation ↓**: Hepta-Score, VGCA-Σ 7D vector, DomainCentroid calibration

**Sovereign Rule**: Eigenvalues are always real for the symmetric quality
operators used in BahyWay (symmetric = quality dimensions are self-adjoint).
Complex eigenvalues signal a malformed quality operator — a validation error.

---

## Part II — Geometric Algebra (Layer 5: DGA + Spinors)

---

### Clifford Algebra Cl(n)

**Layer**: 5 — DGA + Spinors
**Crate**: `bahyway-algebra` (to be extended)

**Definition**: The Clifford algebra Cl(n) over ℝ is generated by n basis
vectors e₁, e₂, ..., eₙ subject to the relation eᵢeⱼ + eⱼeᵢ = 2δᵢⱼ.
Elements of Cl(n) have grades: grade 0 (scalars), grade 1 (vectors),
grade 2 (bivectors), ..., grade n (pseudoscalar). Dimension of Cl(n) = 2ⁿ.

**BahyWay Action**: Cl(7) is the natural algebra for the 7D VGCA-Σ quality
space. Its grade structure maps directly to KAKI physical types:
- Grade 0 scalar → Identity-Kaki quality scores
- Grade 1 vector → Event-Kaki state transitions (directed changes)
- Grade 2 bivector → CrossTribe-Kaki relations (oriented areas between particles)

The Clifford product eᵢeⱼ = eᵢ∧eⱼ + g(eᵢ,eⱼ) encodes both the geometric
relationship between two quality dimensions AND their inner product in one
operation.

**Escalation ↑**: Algebra, Inner Product Space, Vector Space

**Escalation ↓**: Spinor, Rotor, Bivector, Octonions (Cl(7) contains octonions),
KAKI grade mapping

**Sovereign Rule**: Cl(7) is used for quality space geometry only.
KAKI byte computations remain in the discrete field ℤ/2^32.

---

### Bivector

**Layer**: 5 — DGA + Spinors
**Crate**: `bahyway-algebra`

**Definition**: A grade-2 element of a Clifford algebra. The bivector eᵢ∧eⱼ
represents an oriented area element in the plane spanned by eᵢ and eⱼ.
Bivectors are antisymmetric: eᵢ∧eⱼ = −eⱼ∧eᵢ.

**BahyWay Action**: A CrossTribe-Kaki is geometrically a bivector — it
represents the oriented area swept between two particles' quality trajectories.
The sign of the bivector encodes whether the CrossTribe relation is "forward"
(particle A influences B) or "reverse" (B influences A). The magnitude
encodes how much quality space is enclosed by the relation.

When two Event-Kakis eₐ and e_b compose: eₐ · e_b = eₐ∧e_b + g(eₐ,e_b).
The bivector part eₐ∧e_b is the non-commutative residue — the geometric
evidence that order matters.

**Escalation ↑**: Clifford Algebra Cl(n)

**Escalation ↓**: Spinor, CrossTribe-Kaki geometry, StoryEngine divergence
computation

**Sovereign Rule**: Bivector magnitude is stored in EAV attributes.
It is never stored in CrossTribe-Kaki KAKI bytes (Structural-Facts-Only Rule).

---

### Spinor

**Layer**: 5 — DGA + Spinors
**Crate**: `bahyway-algebra`

**Definition**: A spinor is an element of the **even-grade subalgebra** of
Cl(n): the subspace spanned by grades 0, 2, 4, ... A spinor returns to its
original state after a 720° rotation, acquiring a factor of −1 after 360°.
In Cl(3): spinors = quaternions. In Cl(7): spinors are related to Spin(7).

**BahyWay Action**: Spinors govern **rotations in quality space** — how a
particle's quality vector transforms when it undergoes state transitions.
The even-grade structure means the combination of Identity-Kaki (grade 0)
and CrossTribe-Kaki (grade 2) operations forms the spinor sub-algebra.

In NaviEngine: a path traversal of the heptagram is a spinor. Clockwise
and counterclockwise traversals have spinor phases +1 and −1 respectively.
Two routes with identical cost but opposite spinor phase are geometrically
inequivalent — chiral routing distinguishes them.

**Escalation ↑**: Clifford Algebra, Bivector

**Escalation ↓**: Rotor, Chiral Routing (NaviEngine), StoryEngine spinor
divergence, G₂ / Spin(7) symmetry

**Sovereign Rule**: A spinor phase of −1 after one full circuit of the
heptagram is not an error — it is the correct spinorial behaviour. The
routing algorithm must not normalise this away.

---

### Rotor

**Layer**: 5 — DGA + Spinors
**Crate**: `bahyway-algebra`

**Definition**: A rotor R is a unit spinor (RR̃ = 1, where R̃ is the reverse)
that implements rotations via the sandwich product: v' = RvR⁻¹. Every
rotation in n dimensions can be expressed as a rotor. Two rotors compose
by multiplication: R₁R₂ implements two successive rotations.

**BahyWay Action**: A quality state transition in VGCA-Σ space is a rotor
acting on the particle's quality vector. The StoryEngine, when replaying a
Journal, composes all Event-Kaki rotors to arrive at the projected quality
state: v_final = (Rₙ···R₂R₁) v_birth (Rₙ···R₂R₁)⁻¹.

This is more efficient than replaying each transition individually when
snapshots are available — the snapshot records the composed rotor up to
that point, and subsequent replay only composes the delta rotors.

**Escalation ↑**: Spinor

**Escalation ↓**: StoryEngine projection (rotor composition), Snapshot
(stores composed rotor), DGA geodesic computation

**Sovereign Rule**: Rotor composition is non-commutative. R₁R₂ ≠ R₂R₁
in general. The Journal's append-only order is what makes the rotor
composition deterministic and auditable.

---

### Octonions 𝕆

**Layer**: 4 — TOP Algebra (via Cl(7))
**Crate**: `bahyway-algebra`

**Definition**: The octonions are an 8-dimensional non-associative division
algebra over ℝ with basis {1, e₁, e₂, e₃, e₄, e₅, e₆, e₇}. They extend
quaternions as ℝ ⊂ ℂ ⊂ ℍ ⊂ 𝕆. Octonions are non-associative:
(ab)c ≠ a(bc) in general. Their automorphism group is the exceptional Lie
group G₂.

**BahyWay Action**: The 7 imaginary units of 𝕆 correspond to the 7 dimensions
of VGCA-Σ quality space. The octonion multiplication table encodes the
interaction rules between quality dimensions — which dimensions reinforce
each other (positive product) and which oppose each other (negative product).

The non-associativity of octonions is the algebraic expression of what
BahyWay observes empirically: quality improvement in dimension A followed
by improvement in dimension B does not always equal B then A. The order
of quality interventions matters.

**Escalation ↑**: Clifford Algebra Cl(7) (octonions are embedded in Cl(7)),
Algebra

**Escalation ↓**: G₂ symmetry group (the automorphisms of 𝕆), Tribe Algebra
(tribe interactions governed by octonionic product structure)

**Sovereign Rule**: The 7 quality dimensions are not interchangeable. Their
interaction is governed by the octonion multiplication table, not by a
symmetric inner product. Two dimensions that are octonionically orthogonal
cannot substitute for each other in quality scoring.

---

### Manifold

**Layer**: 5 — DGA
**Crate**: `vgca-engine`

**Definition**: A manifold M is a topological space that locally resembles
ℝⁿ. Every point in M has a neighbourhood homeomorphic to an open subset of
ℝⁿ. A Riemannian manifold additionally has a metric tensor g that defines
distances and angles.

**BahyWay Action**: The VGCA-Σ 7D quality space is a Riemannian manifold.
The DomainCentroid defines the "healthy region" of this manifold. Particles
orbit on this manifold — their quality trajectories are smooth curves
(geodesics or deviations from geodesics). The curvature of the manifold
near the DomainCentroid tells you how tightly quality dimensions are
correlated in a healthy tribe.

**Escalation ↑**: Inner Product Space, Vector Space

**Escalation ↓**: Geodesic, Riemannian Curvature, Covariant Derivative,
DomainCentroid, ColorID fraud detection

**Sovereign Rule**: The quality manifold is calibrated from GEM-lane particles
only. FUZZY and DEAD particles are not used in curvature computation — they
would distort the "healthy region" definition.

---

### Geodesic

**Layer**: 5 — DGA
**Crate**: `vgca-engine`

**Definition**: A geodesic is the shortest path between two points on a
manifold — the generalisation of a straight line to curved space. On a
sphere, geodesics are great circles. Formally: a curve γ(t) is a geodesic
if its covariant acceleration is zero: ∇_γ'(γ') = 0.

**BahyWay Action**: The optimal quality improvement path for a particle is
the geodesic from its current quality position to the DomainCentroid. A
particle that degrades does not follow a geodesic — it deviates. The
magnitude of deviation from the geodesic is the early fraud/defect signal
in the ColorID diagnostic system.

In NaviEngine: the optimal route between two NaviNodes is the geodesic on
the routing cost manifold — the path that minimises total effective cost.
Dijkstra's algorithm approximates this geodesic on the discrete graph.

**Escalation ↑**: Manifold, Covariant Derivative

**Escalation ↓**: ColorID fraud detection (geodesic deviation signal),
NaviEngine routing (geodesic approximation), DGA orbit trajectory

**Sovereign Rule**: Geodesic deviation increasing monotonically over
multiple epochs is an AlertEngine trigger — not a single-point check.
One deviation is noise. Monotonic increase is a signal.

---

### Covariant Derivative

**Layer**: 5 — DGA
**Crate**: `vgca-engine`

**Definition**: The covariant derivative ∇_X Y measures how a vector field Y
changes along the direction X on a manifold, correcting for the manifold's
curvature. On flat space (ℝⁿ) it reduces to the ordinary directional
derivative. On curved space it includes connection coefficients (Christoffel
symbols) that encode the curvature.

**BahyWay Action**: The quality drift velocity of a particle is γ'(t) — its
tangent vector. The covariant derivative ∇_γ'(γ') is the quality drift
acceleration — how fast the drift is changing. A particle with
∇_γ'(γ') ≈ 0 is in stable orbit (quality changing at constant rate).
A particle with ∇_γ'(γ') growing is accelerating toward degradation —
the DGA early-warning signal before B11 crosses the lane boundary.

**Escalation ↑**: Manifold, Geodesic, Riemannian Curvature

**Escalation ↓**: DGA fraud/defect early warning, AMMAS quality prediction,
Spinor parallel transport

**Sovereign Rule**: Covariant derivative is computed over a rolling window
of sovereign epochs — not instantaneously. Single-epoch acceleration is
noise. Multi-epoch trend is signal.

---

### Riemannian Curvature

**Layer**: 5 — DGA
**Crate**: `vgca-engine`

**Definition**: The Riemann curvature tensor R(X,Y)Z measures how much a
vector Z rotates when parallel-transported around an infinitesimal loop
defined by vector fields X and Y. On a flat manifold R = 0. On a sphere
R ≠ 0. Scalar curvature K is the trace of R.

**BahyWay Action**: The curvature of the quality manifold near the
DomainCentroid measures how tightly the tribe's quality dimensions correlate.
High curvature = quality dimensions are tightly coupled (a tribe where
freshness and accuracy always degrade together).
Low curvature = quality dimensions are independent.

A particle whose local curvature diverges from the tribe's mean curvature
is an outlier — its quality dimensions are interacting differently from
its peers. This is the deepest anomaly signal in the system: not just
"this particle is far from the centroid" but "this particle's quality
geometry is structurally different from its tribe."

**Escalation ↑**: Manifold, Covariant Derivative

**Escalation ↓**: ColorID 7D fraud diagnostic (curvature mismatch),
DomainCentroid self-calibration, AMMAS quality physics

**Sovereign Rule**: Curvature is computed from the GEM-lane particle
population only. It is a tribe property, not a particle property.

---

## Part III — Modular Forms (Layer 5: Viazovska)

---

### Modular Form

**Layer**: 5 — DGA + Viazovska
**Crate**: `heptascript`

**Definition**: A modular form of weight k and level N is a holomorphic
function f : ℍ → ℂ on the upper half-plane ℍ that transforms as
f(γτ) = (cτ+d)ᵏ f(τ) for all γ = [[a,b],[c,d]] in the congruence
subgroup Γ₀(N) ⊆ SL(2,ℤ). The Fourier expansion is f(τ) = Σ a(n)qⁿ
where q = e^(2πiτ).

**BahyWay Action**: The ModularNaviIndex uses the theta series (a specific
modular form) to build a compact routing signature for NaviMaps. The
Fourier coefficients a(n) count the number of NaviNode pairs at effective
routing cost n. Two NaviMaps with identical Fourier coefficients are
routing-equivalent — they have identical optimal path structures.

**Escalation ↑**: Complex Analysis (implicit), Algebra

**Escalation ↓**: Theta Series, Eisenstein Series E₂, ModularNaviIndex,
E₂FourierWeights (sacred sector weight derivation)

**Sovereign Rule**: The level N = 7 is fixed by the heptagram's 7-fold
symmetry. It is not a tuning parameter.

---

### Theta Series

**Layer**: 5 — DGA + Viazovska
**Crate**: `heptascript`

**Definition**: The theta series of a lattice Λ is θ_Λ(τ) = Σ_{v∈Λ} q^(|v|²/2)
where q = e^(2πiτ). Its Fourier coefficients a(n) count the number of
lattice vectors with squared norm 2n.

**BahyWay Action**: The routing cost histogram of a NaviMap is its theta
series. a(n) = number of directed edges with effective cost n. The spectral
peak cost argmax_n a(n) is the NaviMap's "natural frequency" — the most
common routing cost. For the heptagram: spoke cost (400m) = rim cost (400m)
→ single bucket a(4) = 24 → delta-function spectrum → maximum routing
resonance, identical to E8 lattice sphere-packing optimality.

**Escalation ↑**: Modular Form

**Escalation ↓**: ModularNaviIndex (Stage 1), spectral_peak_cost(),
is_equivalent() via CRC-16 of non-zero coefficients

**Sovereign Rule**: The theta series is computed from actual edge costs in
the NaviMap at construction time — not approximated or pre-loaded.

---

### Eisenstein Series E₂

**Layer**: 5 — DGA + Viazovska
**Crate**: `heptascript`

**Definition**: The Eisenstein series of weight 2 is:
E₂(τ) = 1 − 24 Σ_{n≥1} σ₁(n) qⁿ where σ₁(n) = Σ_{d|n} d (sum of divisors).
E₂ is a quasi-modular form (not a true modular form due to weight-2
anomaly) but is the building block of all weight-2 modular forms.

**BahyWay Action**: The E₂FourierWeights type evaluates E₂ at τ = k/7
for k = 0..6, producing 7 mathematically derived weights. These replace
the hand-chosen NajafSector weights (0.85, 0.88, 0.90...) with values
that are provably optimal in the sphere-packing sense. The dominant value
C(0) ≈ 1.62 confirms the heptagram's central hub. Outer sectors C(2)
and C(5) are the most elevated — these correspond to the Awliya and
Ulamaa zones in the NajafEngine sacred geography.

**Escalation ↑**: Modular Form, Theta Series

**Escalation ↓**: E₂FourierWeights (Stage 2 sacred weights),
NajafSector weight calibration, HeptaChordType sovereign derivation

**Sovereign Rule**: The E₂ weights are computed once at crate design time
and stored as constants. They are not recomputed at runtime. Their values
are the sovereignty proof that the system was mathematically designed,
not hand-tuned.

---

## Part IV — Composite Sovereign Algebra (Layer 4: TOP Algebra)

---

### Particles Algebra

**Layer**: 4 — TOP Algebra
**Crate**: `enkidb-kaki`, `story-engine`

**Definition**: The Particles Algebra P is a monoid (P, ·, e) where:
- Elements are KAKI particles and their Event-Kaki Journals
- The operation · is Journal append (Event-Kaki concatenation)
- The identity element e is the empty Journal (particle at birth)
- The monoid is free (no cancellation) and append-only

**BahyWay Action**: Every particle's entire lifecycle is expressed in the
Particles Algebra. Birth = identity element. Each Event-Kaki appended =
one monoid multiplication. StoryEngine::project = the evaluation
homomorphism from the Journal monoid to the current quality state space.

The monoid is the algebraic proof that DELETE does not exist: a monoid
has no inverse operation. You can append; you cannot un-append.

**Escalation ↑**: Algebra, Simplex (particles as 0-simplices)

**Escalation ↓**: StoryEngine, Journal, Snapshot (records monoid prefix),
Rotor composition (each Event-Kaki is a rotor in the spinor representation)

**Sovereign Rule**: The Journal monoid is append-only. No inverse. No
modification. No deletion. This is algebraically enforced, not
policy-enforced.

---

### Orbits Calculus

**Layer**: 4 — TOP Algebra
**Crate**: `tribe-orbit-engine`, `vgca-engine`

**Definition**: The Orbits Calculus is the calculus of particle trajectories
in the 7D VGCA-Σ quality manifold. It defines:
- **Orbit composition**: γ₁ ∘ γ₂ (concatenating two trajectory segments)
- **Orbit projection**: γ(t) at a given sovereign epoch t
- **Orbit distance**: d(γ₁, γ₂) between two orbit trajectories
- **Orbit convergence**: whether γ(t) → DomainCentroid as t → ∞

**BahyWay Action**: Every particle traces an orbit in quality space. The
quality lane assignments (GEM / TRIBE / ACTIVE / FUZZY / DEAD) are regions
of the quality manifold partitioned by B11 thresholds. A particle whose
orbit converges to the GEM region is healthy. A particle whose orbit
diverges radially is degrading. The AlertEngine monitors orbit divergence.

**Escalation ↑**: Manifold, Geodesic, DGA, Particles Algebra

**Escalation ↓**: ColorID 7D diagnostics, AMMAS Markov quality prediction,
DomainCentroid calibration, Lane assignment

**Sovereign Rule**: Orbit convergence is computed over a minimum of 3
sovereign epochs. A single data point is not an orbit — it is a position.

---

### Tribe Algebra

**Layer**: 4 — TOP Algebra
**Crate**: `bahyway-core`, `enkidb-kaki`

**Definition**: The Tribe Algebra T is a partial algebra on the set of
tribes. Tribe composition T₁ ⊗ T₂ is defined only when the tribes satisfy
the CrossTribe compatibility condition (same domain family or explicit
CrossTribe-Kaki registration). The DomainCentroid is the identity element
of each tribe under the quality scoring operation.

**BahyWay Action**: Tribe membership is encoded in κ[4..5] of every KAKI.
Tribe composition governs which CrossTribe-Kakis are legal. The Tribe Algebra
defines the boundaries of the sovereign field — a query that crosses tribe
boundaries without a CrossTribe-Kaki is algebraically illegal, not merely
unauthorised.

**Escalation ↑**: Algebra, Simplicial Complex (tribes as complexes)

**Escalation ↓**: CrossTribe-Kaki, IDU Probing Rule, Partition routing
(κ[4..5] as partition key), istar domain isolation

**Sovereign Rule**: Tribe composition is partial — not all tribes can
compose. An attempt to compose incompatible tribes is a Forbidden Operation
(ADR-008 #11).

---

### DomainCentroid

**Layer**: 4 — TOP Algebra + DGA
**Crate**: `vgca-engine`

**Definition**: The DomainCentroid C_T is the centroid of all GEM-lane
particles in tribe T in the 7D VGCA-Σ quality space:
C_T = (1/|GEM_T|) Σ_{p ∈ GEM_T} v(p)
where v(p) is the quality vector of particle p. It is the "sovereign ideal"
against which all particle quality is measured.

**BahyWay Action**: The DomainCentroid is self-calibrating — it updates
automatically as GEM-lane particles are added or degrade. The H(P)
Hepta-Score formula uses C_T as the target T. All VGCA-Σ distances are
measured relative to C_T. A tribe whose DomainCentroid is drifting
(C_T(t+1) ≠ C_T(t)) is experiencing systemic quality drift — an
AlertEngine tribe-level signal.

**Escalation ↑**: Orbits Calculus, Manifold, Tribe Algebra

**Escalation ↓**: H(P) Hepta-Score, VGCA-Σ distance, Riemannian curvature
calibration, Markov transition matrix calibration

**Sovereign Rule**: The DomainCentroid is calibrated from GEM-lane only.
FUZZY, DEAD, or SUSPENDED particles never contribute to C_T.

---

## Part V — ADR-009 Additions (Layers 6, 7, 8)

---

### Directed Graph G = (V, E, w)

**Layer**: 6 — Graph Algebra
**Crate**: `graph-engine` (not yet built)

**Definition**: A directed graph consists of a vertex set V, a directed edge
set E ⊆ V×V, and a weight function w: E → ℝ⁺. A path from u to v is a
sequence of edges u→v₁→...→v. A cycle is a path that returns to its
starting vertex.

**BahyWay Action**: The transaction network of a financial tribe is a
directed graph where vertices are KAKI Identity-Kakis (accounts) and edges
are Event-Kakis (transactions) with weight = transaction amount or effective
VGCA-Σ cost. The Graph Algebra detects ring structures (cycles), identifies
hubs (PageRank), and finds routing nodes (betweenness centrality).

**Escalation ↑**: Simplicial Complex (graph = 1-complex), Tribe Algebra

**Escalation ↓**: PageRank, Betweenness Centrality, SCC, AML detection,
NaviEngine routing graph

**Sovereign Rule**: Graph vertices are KAKI particles — they carry sovereign
identity. Edges without KAKI identity are not permitted in the sovereign graph.

---

### PageRank

**Layer**: 6 — Graph Algebra
**Crate**: `graph-engine` (not yet built)

**Definition**: PageRank r ∈ ℝ^|V| is the dominant eigenvector of the
column-stochastic adjacency matrix A: r = (1−d)/|V| + d·Aᵀr where d=0.85
is the damping factor. Converges by Perron-Frobenius theorem for strongly
connected graphs.

**BahyWay Action**: Particles with r_i >> tribe_mean are flow hubs — in
a financial transaction graph, these are the central nodes of a laundering
ring. PageRank reveals hubs invisible to per-particle VGCA scoring because
it operates on the network topology, not on individual particle quality.

**Escalation ↑**: Directed Graph, Eigenvalue (PageRank = dominant eigenvector)

**Escalation ↓**: AML hub detection, AlertEngine network-level trigger

**Sovereign Rule**: Damping factor d = 0.85 (sovereign constant). Alert
threshold = tribe_mean + 3σ_tribe. Both are configurable per tribe via
HeptaScript but cannot be set to 0 or 1.

---

### Betweenness Centrality

**Layer**: 6 — Graph Algebra
**Crate**: `graph-engine` (not yet built)

**Definition**: CB(v) = Σ_{s≠v≠t} σ_st(v)/σ_st where σ_st = total shortest
paths from s to t and σ_st(v) = those paths passing through v. Measures
how often a vertex acts as a bridge.

**BahyWay Action**: High betweenness centrality in a transaction graph =
the vertex is a routing node for many funds flows. In AML context: structuring
schemes (smurfing) use a small number of routing accounts to aggregate
and disperse funds. These accounts have high betweenness but may have low
individual transaction amounts — invisible to threshold-based detection
but visible to betweenness centrality.

**Escalation ↑**: Directed Graph, PageRank

**Escalation ↓**: AML structuring detection, AlertEngine

**Sovereign Rule**: Betweenness is computed on the full tribe transaction
graph, not on sub-graphs. Restricting to sub-graphs allows adversaries to
stay below detection by keeping ring components small.

---

### Strongly Connected Component (SCC)

**Layer**: 6 — Graph Algebra
**Crate**: `graph-engine` (not yet built)

**Definition**: A SCC is a maximal subgraph of a directed graph where every
vertex can reach every other vertex. Computed by Tarjan's algorithm in
O(|V| + |E|).

**BahyWay Action**: A SCC among financial particles = circular money flow =
layering. Any SCC of size ≥ 3 in a transaction graph is a mandatory
AlertEngine trigger. Even if all individual B11 scores are healthy, the
circular topology is the fraud pattern — funds are moving in a loop to
create the appearance of legitimate business activity.

**Escalation ↑**: Directed Graph

**Escalation ↓**: AML ring detection (highest priority alert), Cycle detection

**Sovereign Rule**: SCC detection runs on every new CrossTribe-Kaki
registration. Registering a CrossTribe-Kaki that would complete a cycle
in the transaction graph triggers AlertEngine before the Kaki is committed.

---

### Shannon Entropy H(X)

**Layer**: 7 — Information Theory
**Crate**: `vgca-engine` (extension)

**Definition**: H(X) = −Σᵢ p(xᵢ) log₂ p(xᵢ) measured in bits. H(X) = 0
when the outcome is certain (one value has probability 1). H(X) is maximum
when all outcomes are equally likely. For n outcomes: 0 ≤ H(X) ≤ log₂ n.

**BahyWay Action**: Applied to the value distribution of each EAV attribute
across a particle's Journal. Low H = attribute carries rich information
(values vary meaningfully). High H = attribute is informationally sparse
(values near-uniform or systematically null — typical of fraud where real
activity is hidden behind defaults).

Alert: H(attribute) > H_tribe_mean + 2σ_tribe = particle's attribute
distribution is anomalous for its tribe.

**Escalation ↑**: Particles Algebra (distribution over Journal events),
DomainCentroid (tribe reference distribution)

**Escalation ↓**: KL Divergence, Benford's Law, Cross-entropy anomaly score,
EAV fraud detection

**Sovereign Rule**: Entropy is computed per EAV attribute, not over the full
EAV vector. Different attributes have different natural entropy profiles —
a uniform distribution is normal for some attributes and suspicious for others.

---

### Kullback-Leibler Divergence D_KL(P ‖ Q)

**Layer**: 7 — Information Theory
**Crate**: `vgca-engine` (extension)

**Definition**: D_KL(P‖Q) = Σᵢ P(xᵢ) log(P(xᵢ)/Q(xᵢ)) measured in nats.
D_KL ≥ 0 always (Gibbs inequality). D_KL = 0 iff P = Q. D_KL is
asymmetric: D_KL(P‖Q) ≠ D_KL(Q‖P).

**BahyWay Action**: D_KL(particle_distribution ‖ DomainCentroid_distribution)
measures how much a particle's EAV value distribution diverges from its
tribe's reference. A particle with D_KL >> 0 is informationally foreign to
its tribe — it may have been populated from a different source, structured
to avoid pattern detection, or corrupted.

Benford's Law check: D_KL(observed_first_digit ‖ Benford_distribution)
detects financial amount structuring. Benford's law states that in natural
data, leading digits follow log₁₀(1 + 1/d). Structured amounts violate this.

**Escalation ↑**: Shannon Entropy, DomainCentroid

**Escalation ↓**: EAV fraud detection, Benford's Law, Cross-entropy score,
Information Theory anomaly alert

**Sovereign Rule**: Smoothing constant ε = 1×10⁻¹⁰ is applied to Q before
computing D_KL to prevent division by zero on zero-probability bins. This
constant is fixed — not tunable per tribe.

---

### Markov Chain

**Layer**: 8 — Stochastic Processes
**Crate**: `ammas-engine` (extension)

**Definition**: A discrete-time Markov chain is a sequence of random
variables X₀, X₁, X₂, ... where P(X_{n+1}=j | X_n=i, X_{n-1}, ...) =
P(X_{n+1}=j | X_n=i) = P_ij (the Markov property — memoryless). The
transition matrix P is row-stochastic: Σⱼ P_ij = 1 for all i.

**BahyWay Action**: The quality lane system (GEM / TRIBE / ACTIVE / FUZZY /
DEAD) is a Markov chain with 5 states. The transition matrix P is calibrated
per tribe from the historical Event-Kaki Journal. DEAD is an absorbing state
(P_DEAD,DEAD = 1). The chain answers: given a particle is currently FUZZY,
what is the probability it reaches DEAD in the next 10 epochs?

This transforms AMMAS from an observer (current B11 score) into a predictor
(future lane distribution probability).

**Escalation ↑**: Particles Algebra (Journal provides transition observations),
Orbits Calculus (lanes are orbit regions)

**Escalation ↓**: Steady-State Distribution, Mean First Passage Time,
χ² Anomaly Detection, behaviour-engine internal threat detection,
Optimal Snapshot Interval derivation

**Sovereign Rule**: The transition matrix P is calibrated from GEM-lane and
TRIBE-lane particles only for the "healthy" entries. FUZZY→DEAD transitions
are calibrated from the full population. This prevents the matrix from
being corrupted by an already-degraded population.

---

### Steady-State Distribution π

**Layer**: 8 — Stochastic Processes
**Crate**: `ammas-engine` (extension)

**Definition**: The steady-state distribution π of a Markov chain is the
row vector satisfying πP = π and Σᵢ πᵢ = 1. It gives the long-run
proportion of time spent in each state, independent of initial state.

**BahyWay Action**: π_GEM is the long-run proportion of GEM-lane particles
in a tribe. If π_GEM < GEM_RATE_TARGET (35.4%, ADR-004), the tribe's quality
physics is degrading — the Markov dynamics have shifted and corrective
intervention is required before the steady state drifts further.

The steady-state distribution is a tribe health dashboard metric: it shows
where the tribe is heading, not just where it is today.

**Escalation ↑**: Markov Chain

**Escalation ↓**: Tribe health dashboard, GEM rate monitoring (ADR-004),
AlertEngine tribe-level trigger

**Sovereign Rule**: π is recomputed when the transition matrix P is updated
(after each calibration epoch). It is never stored as a particle attribute
— it is a derived tribe property.

---

### Mean First Passage Time M_ij

**Layer**: 8 — Stochastic Processes
**Crate**: `ammas-engine` (extension)

**Definition**: The mean first passage time M_ij is the expected number of
steps to reach state j starting from state i:
M_ij = 1/P_ij + Σ_{k≠j} (P_ik/P_ij) M_kj

**BahyWay Action**: M_FUZZY→DEAD gives the expected epochs before a degrading
particle reaches the DEAD lane. This is the sovereign time-to-dead prediction.

Critical application: **optimal snapshot interval derivation**. The snapshot
interval for a tribe should be no larger than M_ACTIVE→{FUZZY∪DEAD} / 2 —
this guarantees a snapshot exists before the expected first degradation event,
so power-outage restart never replays more than half the degradation window.

**Escalation ↑**: Markov Chain, Steady-State Distribution

**Escalation ↓**: Optimal SnapshotSchedule derivation (ADR-007),
Particle time-to-dead prediction, Stewardship prioritisation

**Sovereign Rule**: M_ij is used as an upper bound on the snapshot interval,
not as a guarantee. The actual snapshot is taken on schedule regardless of
whether a degradation event occurred.

---

## Part VI — Escalation Map (Build Order)

The following sequence is the correct Rust implementation order.
Never implement a row before all its ↑ dependencies are tested.

**RULE (added 2026-07-11, after a re-verification pass found the "Crate"
column below had drifted from reality — see `algebra-arsenal` crate):
no row may be marked ✓ without a `Verified` citation of
`crate::module::item` and a passing test name. If you cannot cite a test,
the row is not ✓, no matter how confident the claim sounds. This table
is checked against real code, not asserted from memory.**

Every concept marked ✓ below is reachable in one place regardless of
which crate actually implements it: `algebra-arsenal` (path
`crates/algebra-arsenal`) re-exports all of them under
glossary-matching module names and has its own passing test per
concept. That crate is the front door — if something claimed here is
ever doubted again, `cargo test -p algebra-arsenal` is the one-command
answer.

```
Priority  Concept                    Layer  Crate (real)        Verified (2026-07-11)
────────  ─────────────────────────  ─────  ───────────────────  ──────────────────────────────────────
1         Field                      2      bahyway-field ✓      Field trait + RealField, 7 tests, bahyway-field/src/lib.rs.
                                                                  NOTE: Z/240 is a ring, not a field (240 is
                                                                  composite -> zero divisors) -- corrected from
                                                                  the original v4.0.0 claim. See Zmod240 + its
                                                                  own tests proving non-invertibility of 2.
2         Vector Space               2      kinetic-engine ✓     Vec7D: Add/Sub/Mul<f64>, kinetic-engine/src/vec7d.rs.
                                                                  Real R^7, not a type alias.
3         Inner Product Space        2      hepta-score ✓        HeptaVector::health_score/weighted_distance,
                                                                  hepta-score/src/domain.rs + equation.rs.
                                                                  This is the actual weighted H(P) formula --
                                                                  it was never missing, it was in the wrong
                                                                  Crate column.
4         Algebra                    4      bahyway-algebra ✓    clifford.rs (Multivector), lie.rs (su(7),
                                                                  Casimir), 103 tests total in this crate.
5         Simplex                    2      najaf-engine ✓       (partial) boundary_n exists in
                                                                  bahyway-algebra/src/topology.rs (dimension-
                                                                  tagged boundary operator on raw vertex
                                                                  lists, no Simplex type). Full membership
                                                                  test (barycentric) is in
                                                                  najaf-engine/src/topology.rs.
6         Simplicial Complex         2      najaf-engine ✓       SimplicialComplex + is_particle_in_complex
                                                                  (barycentric 7D membership) +
                                                                  reconstruct_ghost (Gaussian elimination),
                                                                  najaf-engine/src/topology.rs, 8 tests.
                                                                  Built for cemetery-plot navigation --
                                                                  a real domain application, not a stub.
7         Eigenvalue / Eigenvector   2      ea-agent-algebra ✓   SovereignMatrix::eigenvalues_2x2 (exact,
                                                                  including complex), spectral_radius (power
                                                                  iteration), ea-agent-algebra/src/matrix.rs,
                                                                  tested against a known +-i case.
8         Jordan Normal Form         2      ea-agent-algebra ✓   (symmetric case only, exact) jacobi_eigen
                                                                  in ea-agent-algebra/src/jnf.rs -- the
                                                                  classical Jacobi rotation eigendecomposition.
                                                                  For symmetric matrices JNF *is* full
                                                                  diagonalization (spectral theorem: every
                                                                  Jordan block has size 1), so this is complete
                                                                  for that case, not partial. 6 tests, including
                                                                  Av=lambda*v verified directly and eigenvector
                                                                  orthonormality. jordan::JordanAnalyzer (tribe
                                                                  stability via spectral_radius) is unchanged,
                                                                  real, and separate. NOT covered, deliberately:
                                                                  general defective (non-symmetric) n x n JNF
                                                                  with generalized eigenvectors -- genuinely
                                                                  absent, not being claimed here.
9         Manifold                   5      vgca-engine ✓        RiemannianManifold over a general metric
                                                                  tensor field g_ij(x), vgca-engine/src/
                                                                  riemannian.rs. Verified against BOTH BahyWay's
                                                                  actual flat Nabu metric (Christoffel symbols
                                                                  == 0) AND an independent known-analytic case
                                                                  (round 2-sphere, Gaussian curvature K=1/r^2,
                                                                  matched numerically) so the general machinery
                                                                  is proven correct, not just self-consistent
                                                                  on the trivial flat case. 5 tests.
10        DomainCentroid             4      vgca-engine ✓             not re-verified this pass -- carried over from v4.0.0.
11        Orbits Calculus            4      tribe-orbit-engine ✓      not re-verified this pass -- carried over from v4.0.0.
12        Particles Algebra          4      enkidb-kaki ✓             not re-verified this pass -- carried over from v4.0.0.
13        Tribe Algebra              4      bahyway-core ✓            not re-verified this pass -- carried over from v4.0.0.
14        Geodesic                   5      vgca-engine ✓        RiemannianManifold::geodesic (RK4 integration
                                                                  of the geodesic ODE), same file/tests as
                                                                  Manifold above -- flat-metric geodesics proven
                                                                  to be exact straight lines.
15        Covariant Derivative       5      vgca-engine ✓        RiemannianManifold::covariant_derivative,
                                                                  same file. Tested: a constant vector field's
                                                                  covariant derivative vanishes on BahyWay's
                                                                  flat metric, as it must.
16        Riemannian Curvature       5      vgca-engine ✓        RiemannianManifold::{riemann_tensor (private),
                                                                  ricci_tensor, ricci_scalar,
                                                                  gaussian_curvature_2d}, same file. The sphere
                                                                  check above is the load-bearing test -- it is
                                                                  what makes this ✓ rather than "returns zero
                                                                  and calls it done."
17        Clifford Algebra Cl(7)     5      bahyway-algebra ✓    clifford.rs: full Cl(7,0), 128 basis blades
                                                                  (2^7), geometric + wedge products with correct
                                                                  sign via transposition counting. Was wrongly
                                                                  marked "(to extend)" -- it was already complete.
                                                                  5 tests including e_i^2=+1 and wedge
                                                                  antisymmetry.
18        Bivector                   5      bahyway-algebra ✓    Multivector::grade(2) -- directly exercised by
                                                                  clifford.rs's fact_composition_produces_bivector
                                                                  test. Was wrongly marked "(to extend)."
19        Spinor                     5      bahyway-algebra (partial)  Rotor (below) is a genuine single-plane
                                                                  spinor (even-graded element, half-angle
                                                                  double-cover form R=cos(th/2)-sin(th/2)B) but
                                                                  only in one bivector plane at a time -- the
                                                                  general multi-plane spinor (arbitrary even
                                                                  subalgebra element of Cl(7)) does not exist.
20        Rotor                      5      bahyway-algebra ✓    rotor.rs: Rotor + RotorJournal, 7 tests
                                                                  (identity, unit norm, angle round-trip,
                                                                  R.R^-1=identity, journal composition, anomaly
                                                                  detection, angle-band sectoring). Confirmed
                                                                  and cited properly this pass (was
                                                                  unverified-✓ last pass).
21        Octonions                  4→5    bahyway-algebra ✓    octonion.rs: built via the general
                                                                  Cayley-Dickson doubling construction
                                                                  (R->C->H->O), not a hand-typed multiplication
                                                                  table. 6 tests, including the two deep,
                                                                  independently-checkable properties: norm
                                                                  multiplicativity |xy|=|x||y| and loss of
                                                                  associativity at the octonion level (while H
                                                                  one level down stays associative).
22        Modular Form               5      heptascript ✓ (Stage 1+2)    not re-verified this pass.
23        Theta Series               5      heptascript ✓                not re-verified this pass.
24        Eisenstein Series E₂       5      heptascript ✓                not re-verified this pass.
25        Directed Graph             6      graph-engine ✓       New crate. DirectedGraph adjacency structure,
                                                                  graph-engine/src/lib.rs.
26        PageRank                   6      graph-engine ✓       Power iteration, damping d=0.85 (Sovereign
                                                                  Rule). 2 tests: sums to 1.0, ranks a
                                                                  higher-inlink node above lower.
27        Betweenness Centrality     6      graph-engine ✓       Brandes' algorithm. 1 test: bridge node on
                                                                  the only shortest path scores positive,
                                                                  endpoints score zero.
28        SCC                       6      graph-engine ✓        Tarjan's algorithm + scc_alerts(): SCC of
                                                                  size >=3 emits a Critical/Fraud alert through
                                                                  the real alert-engine (Sovereign Rule,
                                                                  AML ring detection) -- same "consume, don't
                                                                  duplicate" pattern as Enbilulu/Milu. 3 tests,
                                                                  including a negative case (size-2 cycle must
                                                                  NOT alert).
29        Shannon Entropy H(X)      7      vgca-engine ✓         Broader than previously stated: TWO real
                                                                  implementations, not one -- byte entropy
                                                                  (bfv::compute_bfv) AND character/text entropy
                                                                  (fsv::shannon_entropy). Corrected from
                                                                  "(partial - byte only)."
30        KL Divergence             7      vgca-engine ✓         kl_divergence.rs: D_KL(P||Q), fixed smoothing
                                                                  epsilon=1e-10 (Sovereign Rule). 6 tests,
                                                                  including Gibbs' inequality (non-negativity)
                                                                  and asymmetry D_KL(P||Q) != D_KL(Q||P).
31        Markov Chain              8      ammas-engine ✓        MarkovChain, ammas-engine/src/markov.rs.
32        Steady-State π            8      ammas-engine ✓        Power iteration on pi^T. Verified against the
                                                                  closed-form 2-state analytic result
                                                                  pi=[b/(a+b), a/(a+b)].
33        Mean First Passage Time   8      ammas-engine ✓        Fundamental-matrix method. Verified against
                                                                  the closed-form 2-state analytic result
                                                                  m_12=1/a, m_21=1/b (a special,
                                                                  independently-checkable property of 2-state
                                                                  chains). 5 tests total for items 31-33.
34        Enbilulu Calculus (Φ_Enbi, TIAMAT bands, Enbi horizon,
          Terru diagnosis, Milu alerts)
                                    4      bahyway-algebra ✓            enbilulu.rs, 14 tests. Added 2026-07-10.
                                                                         wpd-engine consumes it (junction.rs),
                                                                         does not re-derive it.
```

**Legend:** ✓ = exists, cited · (partial) = real but narrower than the concept's
full mathematical scope · (to build/extend) = not yet · "not re-verified
this pass" = carried over from the 2026-06-05 version unchanged; may still
be accurate, but wasn't re-checked against code in this pass, so treat it
with the same caution that produced this rewrite in the first place.

**2026-07-11 build-out pass (PB-165):** items 8, 9, 14, 15, 16, 17, 18,
20, 21, 25, 26, 27, 28, 29, 30, 31, 32, 33 were re-verified and/or built
for real this pass, closing every gap that was concretely, checkably
absent. Full workspace: `cargo test --workspace` = 3368 passed, 0 failed
(up from 3334 before this pass -- the 34-test difference is exactly the
new coverage added: 6 octonion + 6 KL divergence + 5 riemannian + 5
markov + 6 graph-engine + 6 jnf). Remaining honest gaps, stated plainly,
not smoothed over:
  - General (non-symmetric, defective) Jordan Normal Form with
    generalized eigenvectors. Does not exist. Symmetric case (which
    covers every documented BahyWay use of JNF so far) is complete.
  - General multi-plane Spinor (arbitrary even-subalgebra element of
    Cl(7)). Only the single-plane Rotor simplification exists.
  - Items 10-13, 22-24 were not re-checked this pass (no evidence found
    either way) and are carried over from 2026-06-05 unchanged.

---

*𒁾 DUB.SAR — BahyWay.Ecosystem v4.0 | Algebra Glossary v1.0 (2026-06-05),
re-verified against source 2026-07-11 after `algebra-arsenal` +
`bahyway-field` landed. See those crates' own tests as the ground truth
for Part VI going forward.*
