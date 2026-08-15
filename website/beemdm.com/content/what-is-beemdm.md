# What is BeeMDM?

**Master Data, Sealed as Truth.**

BeeMDM is the flagship application of the BahyWay.Ecosystem: a sovereign Master Data Management system that ingests swarms of conflicting source records and masters them into sealed golden records — with lineage, quality, and proof carried inside every particle. Pure Rust. Anti-SQL. Sovereign core, no vendor lock-in.

## The problem

Every organization holds the same customer, product, or asset in many systems, described many contradictory ways. Classic MDM tools bolt matching and lineage onto relational databases as an afterthought — so the golden record is an opinion, not a proof.

BeeMDM makes mastering a property of the data itself:

- Every source record becomes a **particle** with identity, tribe, and time inside its 16-byte KAKI key.
- Quality, state, and colour live in **EAV Mandatory Attributes** — never guessed, always carried.
- Every match, merge, and survivorship decision is an immutable **event particle** — the chronicle is the audit.

## Who is BeeMDM for?

- **MDM teams** deduplicating and mastering entities across many sources.
- **Data stewards** who must defend every golden record with evidence.
- **Architects** who refuse to rent their master data layer from a vendor.

## How do you use it?

### Ingest {#ingest}

Sources enter the nine-station ETL Processing Chain (S0–S8) into EnkiSDB. Ingestion is defined as sealed `.akk` tablets — orchestrations, not scripts.

### Master {#master}

Records flow through the seven sovereign databases, each with one duty:

- **EnkiSDB** — Stage. Where every record first lands and is pre-scanned.
- **EnkiODB** — Operate. Active, validated particles; state changes are new inserts, never mutations.
- **EnkiQDB** — Quarantine. A permanent, append-only home for conflicts and fuzzy/unknown records — never deleted, never silently dropped.
- **EnkiDB** — the Golden Store. One golden record per real-world entity, sealed as the platform's final, permanent destination.
- **EnkiDW** — Warehouse. Receives retired EnkiODB particles for full-scale ETL and analytics.
- **EnkiMDB** and **EnkiDDB** — Metadata and Documents. The ecosystem's own crates, playbooks, and documents, held under the same law as the master data they describe.

Duplicates are matched, survivors chosen, conflicts held in EnkiQDB's permanent quarantine until resolved. Advisory analysis may flag anomalies — but advisors never hold blocking power; the deterministic core decides and the Architect ratifies.

### Prove {#prove}

There is no SQL. Five sovereign operations govern all access — ORBIT, EMIT, PROVE, SYNC, WITNESS — written in HeptaScript. A golden record answers PROVE with its full lineage back to every source, and WITNESS with the chronicle of every decision that shaped it.

### See

The DubSar Theater renders tribes as constellations and mastering as motion: swarms of source particles converging into golden cells, at any scale, powered by the Elu Index Stack.

## Get started

- **Quick start** — coming with the Gudea 1.0 era release.
- **Concepts** — Triple-O, Hepta Space, and the Patterns Arsenal: see [What is BahyWay?](https://www.bahyway.com)

---

*BeeMDM — a BahyWay.Ecosystem application. One scribe, one seal, one truth.*
