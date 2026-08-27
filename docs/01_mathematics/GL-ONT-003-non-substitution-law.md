# GL-ONT-003 — The Non-Substitution Law
## Particles Are Not Objects: Individuality Is Non-Polymorphic

**Ecosystem:** BahyWay.Ecosystem v4.0 — Triple-O foundational ontology
**Series note:** GL-ONT-001 is the *OntoGraph Unified Pattern Law*. This tablet, GL-ONT-003, is the deeper foundation beneath it: the ontological reason the OntoGraph's nodes are individuals and not instances.
**Sits beside:** the Hepta Space Uniqueness Law, GL-BRT-001 (Birth Gate), GL-LBR-001 (labīru origin), GL-AGE-001 (Šību relational standing)
**Status:** DRAFT — awaiting Architect seal (CSR-08)

**Landing note (2026-08-21):** this tablet arrived in the Mon20260817 delivery bearing the ID GL-ONT-002 and was renumbered to GL-ONT-003 on landing — GL-ONT-002 is already sealed in this repo as the *Phase 0 Recognizer Law* (`docs/01_mathematics/GL-ONT-002_Phase0_Recognizer_Law_DRAFT.md`), an unrelated document. See `docs/mon20260817-incoming/README.md` for the original, unmodified draft and the full collision record.

---

## 1. The Law (stated first)

> **A Particle is not a polymorphic instance of a type. Its individuality is earned, witnessed, and non-substitutable. No two Particles are interchangeable — not even two that share every attribute value — because each carries a distinct origin colophon and occupies a unique position in Hepta Space. Substitutability, the defining virtue of OOP polymorphism, is forbidden at the level of the individual. Type membership is one witnessed attribute among seven dimensions, never the ground of identity.**

Corollary (the visible consequence): a **Record that fails the Birth Gate never becomes a Particle**. It remains a **spot** — a location on a storage sector holding a value — which is precisely the OOP object: a slot addressable by type, substitutable for any other of its type. The Particle/spot split *is* the Non-Substitution Law made visible.

## 2. The Objection to OOP Polymorphism

OOP polymorphism holds that *many concrete things are interchangeable through a shared type interface*. The Liskov Substitution Principle makes this explicit and mandatory: any instance of a type must be swappable for any other instance of that type without the caller noticing. **Substitutability is the point.** Dispatch is on the type, not the individual; the individual is a fungible carrier of a type's behavior.

Triple-O objects to this at the root when the thing being modelled is a **witnessed individual with provenance**. For data under governance, "these two records are of the same type" must *never* imply "these two records are interchangeable" — because one may have passed the Gate and one not; one may carry a clean colophon and one be a ghost; one may hold irreplaceable lineage and one be a probe. To make them substitutable through a shared `Record` interface is to erase exactly the distinction that governance exists to preserve.

The Non-Substitution Law does not say *polymorphism is wrong*. Polymorphism is excellent for **behaviour dispatch over interchangeable values**. It says: **polymorphism optimizes for the wrong thing when substitutability is the very property that must be forbidden.** Identity-bearing, provenance-carrying individuals are that case.

## 3. The Precise Inversion (OOP ↔ Triple-O)

| | OOP object | Triple-O Particle |
|---|---|---|
| **Identity source** | cheap, automatic — memory slot / address at allocation | earned, adjudicated — KAKI minted only after the Birth Gate |
| **Where meaning lives** | in the shared *type* (the class) | in the *individual's* witnessed standing across 7 dimensions |
| **Substitutability** | mandatory (Liskov) — the defining virtue | forbidden — the defining prohibition |
| **Birth** | born into a *type*; individuality optional | born into *individuality*; type is one attribute |
| **Position** | a slot, reusable, addressable | a unique real-valued Hepta coordinate no other shares |
| **Relation to others** | dispatch target of a shared interface | orbit-situated; standing shifts as the Tribe moves (Šību) |

OOP: *born into a type, individuality optional.* Triple-O: *born into individuality, type membership is just one witnessed attribute.*

## 4. Distinct From Both OOP and Harman's OOO

Triple-O is **post-OOO**, and the Non-Substitution Law is where the three positions separate cleanly:

- **OOP** — objects are *substitutable* (interchangeable via type). Relational (they call each other), but individuality is incidental.
- **Harman's Object-Oriented Ontology (OOO)** — objects are *withdrawn*: never fully accessible or exhausted by their relations. This already breaks OOP's substitutability at the metaphysical level — a real object exceeds any relation to it. But OOO's objects are **static and relationless at the core**; withdrawnness is a standing condition, not a lived trajectory.
- **Triple-O** — Particles are **relational individuals**: their uniqueness is a *fixed* Hepta coordinate (Uniqueness Law), yet their *standing* shifts continuously as the Tribe orbits around them (the Šību insight — coordinates immutable, standing relational). Triple-O keeps OOP's **relationality** but discards its **substitutability**; it keeps OOO's **individuality** but discards its **staticness**.

The resulting position — **individual, relational, non-substitutable, orbit-situated** — is what "Particles ≠ Objects" claims. It is a genuine ontological stance, citable and distinct, not a rebranding of either predecessor.

## 5. The Birth Gate as Enforcement

The Non-Substitution Law is not a wish; it is enforced at a specific place. The **Ṣīt Birth Gate (GL-BRT-001)** is the mechanism: it adjudicates each arriving Record and mints a KAKI *only* on pass. Because identity is conferred at the Gate and never carried in, a refused Record cannot acquire individuality by masquerade. Polymorphism, had the system been built on it, would have let a refused Record pass as a Particle through the shared `Record` interface — substitutability would have defeated the Gate at exactly the boundary where born and refused must not be interchangeable.

**The falsifiable engineering consequence:** *a system built on polymorphic Records cannot express the Birth Gate.* The type interface makes the refused and the born substitutable precisely where governance forbids it. This is testable: attempt to model the two-branch split with subtype polymorphism and the Liskov constraint will force born and refused to be mutually substitutable, collapsing the distinction. Triple-O's sum-type verdict (`Born(Particle) | Refused(RefusalRecord)`) with disjoint key-spaces is the non-polymorphic construction that holds.

## 6. Relationship to Sealed Laws

- **Hepta Space Uniqueness Law** — the *positional* face of non-substitution: no two Particles share a coordinate. GL-ONT-003 is the *ontological* face: no two are interchangeable even beyond position.
- **GL-BRT-001 (Birth Gate)** — the *enforcement*: identity earned, not assigned.
- **GL-LBR-001 (labīru)** — the *provenance*: each Particle carries an origin that individuates it.
- **GL-AGE-001 (Šību)** — the *relational* clause: standing shifts while coordinates hold, which is why Triple-O individuals are not Harman's static withdrawn objects.

## 7. For the Academic Record

Cite as the founding non-substitution stance of Orbit-Oriented Ontology, distinct from OOP (substitutable instances) and from Harman's OOO (withdrawn static objects). The clean claim: *Triple-O models data as non-substitutable relational individuals whose identity is adjudicated at a birth boundary; substitutability is forbidden by construction, and this is provably inexpressible under subtype polymorphism.* Suitable as the ontological foundation preceding the technical claims (pure-Rust CQRS graph store, persistent-homology quality monitoring, Transparency Deficit Calculus).

## 8. Seal

```
Sealed by: ______________________  (DUB.SAR 𒁾, CSR-08)
```
