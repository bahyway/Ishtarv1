# GL-STW-001 · Steward Court Law · Pre-Golden and Golden Store (draft, unsealed)
Proposed 2026-08-25 · DUB.SAR 𒁾
Depends on: EN-QDB-001 (quarantine facets), GL-AGE-001 (two-witness), GL-SEC-002 §7 (propose, never execute),
GL-UNT-001 §2 (MLU), EN-MDB-001 (Masku), Kanīku receipts

---

## §1 The boundary
The seven types are divided by one line that did not exist before this tablet:

**Pre-Golden (admission)** — `EnkiSDB·7001` · `EnkiODB·7002` · `EnkiQDB·7003`
**Golden Store (citizenship)** — `EnkiDB·7004` · `EnkiDW·7005` · `EnkiMDB·7006` · `EnkiDDB·7007`

A particle in the Pre-Golden zone is a *candidate*. A particle in the Golden Store is a *citizen*, and everything
downstream — facts, masks, doctrine — may rest on it. **Crossing that line is the gravest act a steward performs.**

## §2 The steward proposes; a playbook executes
No console, tab or dashboard may mutate a particle. A steward act produces a **decree**: a signed proposal naming
the subject, the act, the sealed clause relied upon, and the witnesses. PB-386 validates and executes it, and a
Kanīku receipt records what changed. A UI that writes directly to a store is a breach of this tablet.

## §3 The transition matrix
| act | from | to | witnesses | note |
|-----|------|----|-----------|------|
| `MARK_SUSPICIOUS` | any | 7003 | **1** | protective; a single steward may always quarantine |
| `DEFINE` | 7001 | 7002 | 2 | a candidate becomes a definition (factor, leaf, unit, tribe) |
| `RELEASE` | 7003 | 7001 | 2 | quarantine cleared; re-enters admission, never the Golden Store directly |
| `PROMOTE` | 7002 · 7003 | 7004 | **2** + sealed clause | the Golden crossing |
| `DERIVE` | 7004 | 7005 · 7006 | 2 | a citizen yields crossings or masks |
| `CODIFY` | 7005 · 7006 | 7007 | 2 | a pattern becomes doctrine |
| `DEMOTE` | any Golden | 7003 | **2** + reason clause | gravest; everything resting on it is re-examined |
| any other pair | — | — | — | **refused**; skipping a stage is never a steward's discretion |

## §4 Quarantine is asymmetric on purpose
One witness may quarantine; two are required to release, promote or demote. Protection is cheap, admission is dear.
A steward who suspects need not persuade anyone; a steward who admits must.

## §5 A new membrane layer is a decree too
Creating an ORBIT relation between two particles — a new **Membrane Layer** — names the factor leaf and the MLU it
crosses. The leaf must already exist in `EnkiODB·7002`; a membrane may not invent its own factor. Until two epochs
witness it, the relation is `DERIVED` and may not be used in a verdict (GL-AGE-001).

## §6 Every act is a biography entry
`STEWARD-DECREE`, `PROMOTED`, `DEMOTED`, `QUARANTINED`, `MEMBRANE-OPENED` are Event KAKIs on the subject particle.
No silent close: a decree that is withdrawn is recorded as withdrawn, not erased.

## §8 Quarantine lands in a cloned tribe, never a bin
A particle marked suspicious does not fall into a shared quarantine bucket. The court **mints a tribe in
`EnkiQDB·7003`** for that source tribe, and the new tribe carries a **clone of the source schema** so the deviation
remains comparable to its origin. The clone declares its lineage:

| lineage | schema cloned from | shape of the work |
|---------|--------------------|-------------------|
| `ETL`    | `EnkiSDB·7001` — the arrival schema | batch: the particle arrived in a consignment and is judged against how it landed |
| `STREAM` | `EnkiODB·7002` — the definition schema | event: the particle arrived alone and is judged against what the factor means |

The minted tribe is **provisional** until the witness rule of §3 is met, then it stands. Its members carry the seven
mandatory facets of EN-QDB-001, and its shape (β₀, β₁, RU layers) is inherited from the source so that a change in
shape after quarantine is itself evidence.

## §7 What a steward may never do
Mutate a quarantined payload · promote directly from 7001 to the Golden Store · create a membrane on an
undefined factor · act as both witnesses · dispose of a quarantined particle whose deadline is under legal hold ·
delete anything (Irkalla records; NUZI archives; nothing is erased).
