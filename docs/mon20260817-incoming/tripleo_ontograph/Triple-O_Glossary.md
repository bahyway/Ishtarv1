# Triple-O Glossary
## Definitional Reference for Orbit-Oriented Ontology

**Ecosystem:** BahyWay.Ecosystem v4.0
**Companion to:** the Triple-O Manual, the BahyWay.Ecosystem Manual, GL-ONT-002
**Convention:** every sealed Akkadian name carries a plain-language gloss (Ḫubullu Law, NL-001).

---

### Core Ontological Terms (defined against each other)

**Triple-O (Orbit-Oriented Ontology).** The paradigm holding that data are **non-substitutable relational individuals** whose identity is adjudicated at a birth boundary, whose position is unique and immutable, and whose standing shifts as their Tribe orbits around them. Post-OOO: distinct from both OOP and Harman's Object-Oriented Ontology. See *Non-Substitution Law*.

**Non-Substitution Law (GL-ONT-002).** The founding law: a Particle is *not* a polymorphic instance of a type; its individuality is earned, witnessed, and non-substitutable. No two Particles are interchangeable, even if they share every attribute value. Substitutability — OOP polymorphism's defining virtue — is forbidden at the level of the individual. *This is the single law from which the Particle/spot distinction, the Birth Gate, and the two-ledger forensics all follow.*

**Particle.** A Record that passed the **Birth Gate** and earned a **KAKI v4.0 identity**. Owns a colophon (origin/provenance), a unique real-valued **Hepta coordinate**, and standing in a **Tribe**. Lives inside the walls (NUZI). A Particle is an *individual*, not an instance — **Particle ≠ Object**.

**Object (OOP sense).** A slot holding a value of a type, addressable and **substitutable** for any other instance of that type (Liskov). Identity is automatic (a memory address / storage slot). In Triple-O, the OOP object corresponds to a **spot** — the thing a Record *stays* if it is refused identity. Triple-O's central negative claim: **a Particle is not an Object.**

**Spot.** A location on a storage sector holding a value whose Record was **refused** at the Birth Gate — a **Non-Particle**. Has no KAKI, no colophon, no coordinate. A spot *is* the OOP object: a substitutable slot. Spots mass on SUSA-side sectors and may siege the walls from outside, but can never masquerade as Particles. The Particle/spot split is the Non-Substitution Law made visible in storage.

**Non-Particle.** A refused Record. Synonym-in-effect for the Record occupying a *spot*. Never admitted to the Ecosystem; traceable only via the **Bāb Ṭurdi log** (transport metadata + refusal reason), never by identity.

**Polymorphism (and why Triple-O objects to it).** The OOP mechanism by which many concrete instances are interchangeable through a shared type interface; behaviour dispatches on the *type*, not the individual. Triple-O does not claim polymorphism is wrong — it is correct for *behaviour dispatch over interchangeable values*. Triple-O claims it **optimizes for the wrong thing when substitutability is precisely the property that must be forbidden** — as with governed data carrying provenance, where "same type" must never mean "interchangeable."

---

### The Three Positions (quick contrast)

**OOP object** — substitutable (Liskov), relational, individuality incidental, identity automatic.
**Harman's OOO object** — withdrawn (never exhausted by relations), *non*-substitutable, but static and relationless at the core.
**Triple-O Particle** — non-substitutable *and* relational *and* orbit-situated: unique fixed position, standing that shifts with the Tribe. Keeps OOP's relationality, discards its substitutability; keeps OOO's individuality, discards its staticness.

---

### Enforcement & Structure Terms

**Ṣīt Birth Gate (GL-BRT-001).** The ingestion boundary that adjudicates each Record: PASS → Particle (KAKI minted); REFUSE → Non-Particle (spot, outer quarantine). The place where the Non-Substitution Law is *enforced*: identity is conferred at the Gate, never carried in.

**KAKI v4.0.** The 16-byte primary key, minted only at the Birth Gate on PASS. The identity that makes a Particle non-substitutable. A refused Record structurally cannot hold one.

**Hepta Space / Uniqueness Law.** The 7-dimensional space in which every Particle has a unique, immutable real-valued position. The positional face of non-substitution — no two individuals coincide.

**Šību (šb) / Relational-Standing Law (GL-AGE-001).** The measure of a Particle's shifting standing within its Tribe as the orbit widens; coordinates immutable, standing relational. The clause that makes Triple-O individuals *relational and dynamic*, unlike Harman's static withdrawn objects.

**Labīru (GL-LBR-001).** The origin doctrine: every born Particle carries an origin colophon that individuates it. Provenance is a face of non-substitution.

**NUZI.** The inward/archival provenance store — home of the born, queried by KAKI identity.

**Bāb Ṭurdi log.** The Gate of the Turned-Away's ledger — refusal records for Non-Particles (transport metadata + reason, no identity), read by *meaning*, never joined to NUZI.

**Sum-type verdict.** The non-polymorphic construction `Born(Particle) | Refused(RefusalRecord)` with disjoint key-spaces, used instead of subtype polymorphism precisely so the born and refused cannot be made substitutable. The engineering signature of Triple-O.

**HeptaScript.** The Anti-SQL sovereign query language; its five operations (ORBIT, EMIT, PROVE, SYNC, WITNESS) witness individuals by attribute, never type-substitute.

---

### The Falsifiable Claim (glossary entry)

**"Polymorphic Records cannot express the Birth Gate."** The testable consequence of the Non-Substitution Law: modelling the born/refused split with subtype polymorphism forces (via Liskov) the two to be mutually substitutable wherever a `Record` is expected, collapsing the distinction. Only a non-polymorphic sum-type with disjoint key-spaces holds the boundary. This is Triple-O's proof that its ontology has engineering teeth.

---

*Glossary maintained under NL-001 / Ḫubullu Law. Sealed by: ______________________ (DUB.SAR 𒁾, CSR-08)*
