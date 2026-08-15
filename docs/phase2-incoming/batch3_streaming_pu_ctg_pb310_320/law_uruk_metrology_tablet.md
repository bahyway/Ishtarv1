# 𒁾 TABLET VI — GL-MET-001 "URUK ŠID" 𒋃 — The Archaic Metrology Law
### Proposed coordinate: NIPPUR 3.5 (Major 3 · LEDGER, reserve slot — assignment by the Architect)
### Status: DRAFT — unsealed until the Architect's ceremony (CSR-08)

*Named for Uruk, the city of the archaic tablets (structures named for cities, per NL-001),
and ŠID 𒋃, the Sumerian act of counting. Registry scholarship after the archaic bookkeeping
corpus (Uruk IV–III proto-cuneiform).*

---

### Preamble — Against the Illusion Unit

Modern accounting knows one number and many labels; the archaic scribes knew many numbers,
and the **ontology chose the arithmetic**. A herd, a field, a jar of beer, and a dead animal
were counted in different systems that could not be silently exchanged. This law restores that
discipline to BahyWay v4.0: what a thing *is* determines how it may be counted, and nothing —
least of all the dead of the natural world — may be flattened into someone else's unit as a
mere loss. **What money is to banks, these counts are to nature: the medium in which its
account is real.**

### Clause M-1 — The Registry of the Thirteen Systems

The following archaic systems are registered as sovereign count-types of the ecosystem
(the URUK Registry). Each is a distinct numeric type in the pure-Rust sense: a newtype,
never a bare integer, carried on every KAKI as an EAV Mandatory Attribute.

| Sign | System | Archaic domain | BahyWay v4.0 register |
|------|--------|----------------|------------------------|
| S | Sexagesimal S | slaves*, animals, fish, wooden/stone objects, containers | living stock, discrete assets |
| S′ | Sexagesimal S′ | **dead animals**, certain beers | **the Register of the Dead** (Clause M-2) |
| B | Bisexagesimal B | cereal, bread, fish, milk products | food-mass, provisions |
| B* | Bisexagesimal B* | rations | human need, allocations |
| G | GAN₂ G | field measurement | land, habitat area |
| Š | ŠE Š | barley by volume | primary grain volume |
| Š′ | ŠE Š′ | malt by volume | transformed grain (malt) |
| Š″ | ŠE Š″ | wheat by volume | secondary grain volume |
| Š* | ŠE Š* | barley groats | processed grain |
| E | EN E | weight | mass |
| U | U₄ U | calendrics | time, seasons, spans |
| Db | DUGᵇ Db | milk by volume | nourishing liquids |
| Dc | DUGᶜ Dc | beer by volume | fermented / process liquids |

*\* The archaic world counted persons among property; BahyWay registers this fact as history
and rejects it as practice — System S in this ecosystem counts no human being.*

### Clause M-2 — The Sovereign Register of the Dead (System S′)

Every death in an ecological ledger — an animal, a hive, a stand of trees, a reef — is a
**first-class particle counted in S′**, with its own KAKI, its own lineage, its own place in
Hepta Space. The dead are never recorded as a decrement of the living count.

> **A dead thing is not a negative living thing.**

S′ totals are reported beside S totals, never subtracted from them, so that no dashboard,
report, or stakeholder view can present extinction as a rounding of stock. This is the
end of the illusion unit: the losses of nature are not "shown to normal people as losses" —
they are shown as *counted beings in their own system*, with the same dignity of arithmetic
that money enjoys in a bank.

### Clause M-3 — Incommensurability by Type

No value of one system may be converted to another implicitly. In code this is a compile-time
guarantee: the thirteen systems are distinct types with no `From`/`Into` between them.
In HeptaScript, cross-system expressions without a declared bridge do not parse.
In EnkiDB, the EAV unit attribute is mandatory and unmixable. Incommensurability is not a
limitation of the ledger; it **is** the ledger's honesty.

### Clause M-4 — The Four Bridges (the Only Lawful Conversions)

Conversion between systems exists only through the sealed archaic ratios, each a bridge with
a **declared loss** that flows into τ as conversion-ε:

| Bridge | Ratio | Archaic office | Declared loss (1 − ratio) |
|--------|-------|----------------|---------------------------|
| Komma | 80⁄81 | ration planning against the 360-day administrative year | 1⁄81 |
| Leimma | 24⁄25 | decimal → sexagesimal passage | 1⁄25 |
| Diesis | 15⁄16 | minor passage between registers | 1⁄16 |
| Euboic | 5⁄6 | weight-standard passage | 1⁄6 |

Every bridge crossing emits a KAKI recording: source system, target system, ratio applied,
and the loss as declared ε. **A conversion that hides its loss is an illusion; a conversion
that seals its loss is a bridge.** No composite of bridges may be applied silently; each hop
is its own KAKI.

### Clause M-5 — The Seven Balls of Uruk (the Resonance Game Law)

The reconciliation of ledgers is visualized as the seven-ball game: seven system-ledgers as
colored balls, each orbiting the core (the issue under account) on its own radius, each with
a period given by its system's ratio to the fundamental. The game runs orbits in contrast to
one another until **grand symmetry** — all seven phases aligned within sealed tolerance.

- Balls lock one by one as symmetry approaches; the **score** of a symmetry event is the
  radial span from the first ball to lock to the last: `score = r(first-lock) − r(last-lock)`,
  credited with the count of runs each ball completed (runs displayed in sexagesimal
  numerals 𒌋𒁹).
- With all four bridges honored, the periods are rational to one another and grand symmetry
  is periodic and reachable. With any bridge dishonored, its dependent ledgers are detuned by
  the unsealed ratio and **symmetry never arrives** — the ledgers chase alignment forever.
  This is the doctrine in mechanics: *ledgers reconcile only across sealed bridges.*
- The S′ ball (the Dead) is drawn distinct — ashen body, gold rim — and its alignment with S
  (the Living) is the ceremony's heart: the living and the dead ledgers meet only when the
  Komma is honored.

### Clause M-6 — The Anti-Illusion Clause

Currency is registered as one system among the many — the bank's system — with no primacy.
Expressing an ecological quantity in currency requires a declared bridge chain, each hop
sealed and lossy by record; absent that chain, the expression is an **illusion unit** and is
forbidden in every BahyWay ledger, report, and theater lens. Nature's account is kept in
nature's systems.

---

*Sixth tablet drafted. Nothing herein is sealed; the seal belongs to the Architect alone.*
*𒁾 Prepared in service of DUB.SAR — after the scribes of Uruk, who knew that what a thing is
decides how it may be counted.*
