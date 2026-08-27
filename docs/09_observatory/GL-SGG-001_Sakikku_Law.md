# GL-SGG-001 — THE SAKIKKÛ LAW (The Law of Diagnosis)
**Wound jurisprudence of BahyWay.Ecosystem · after SA.GIG, the forty tablets, and after the field veterinarian who had no laboratory**

| Field | Value |
|---|---|
| Law ID | GL-SGG-001 |
| Epithet | Sakikkû — "symptoms"; the Babylonian Diagnostic Handbook: *if the patient shows X under Y, the disease is Z, the prognosis W, the treatment T* |
| Status | **PROPOSED — awaiting Bahaa's word (CSR-08)** |
| Physician | **Asû Engine** — deterministic diagnostician; readable reasoning under scarcity; NEVER an opaque scorer |
| Court | **Bīt Asî** — the house of the physician: Phase-A filesystem shelf, later the wound wing of EnkiDDB·7007 |
| Parents | Balāṭu doctrine (self-healing), GL-HSI-001-A3 (biography, no silent close), EN-DDB-003 Šasû (rule-tablet reader), EN-DDB-004 Eṭemmu (ghost surprisal), PB-365 two-witness verdicts, GL-NIM-001 N1 (epistemic stamps), Labīru (truth has a timestamp) |
| Deposited | 2026-08-24 · APPEND-ONLY |

---

## Inscription

> *The boy without equipment had only the seven questions and a trained eye.
> The system inherits both. Every wound is a particle; every survivor is evidence;
> the boundary of the outbreak is the etiology.*

---

## §1 — The Wound Particle (Ḫiṭītu)

Every failed playbook task-run is minted as a **Ḫiṭītu particle** with mandatory
EAV facets:

- `playbook`, `task`, `module`, `rc_class`
- `signature` — the **normalized** stderr/failure text (paths, hex, counts
  stripped), hashed; the disease name is the signature, never the raw scream
- `host` + the **fact-facets** true at failure (distro, kernel, versions —
  the Ansible facts snapshot)
- `epoch` — **the failure moment, never the ingestion moment** (Labīru clause:
  construction-era wounds remain queryable as construction-era wounds)
- `stage` (§4), `prognosis` (§5), epistemic stamp per facet (N1)

Wound *tablets* (full stderr, task YAML, facts dump, operator note) are
**documents**: their GOLDEN home is **EnkiDDB·7007** under Simtu facets —
NOT EnkiMDB. When the census later derives the hypergraph **skeleton**
(signature lattice, syndrome hyperedges, β steles), that anatomy flows to
EnkiMDB·7006 by the Masku division. Flesh in 7007, skeleton in 7006,
ancestry in NUZI.

## §2 — THE HERD DOCTRINE (the veterinarian's clause)

**The unit of diagnosis is the herd, not the patient.**

1. **The healthy witness is mandatory evidence.** For every wounded run, the
   harvester MUST also record the healthy herd: runs of the same playbook on
   comparable hosts that succeeded. A wound corpus without its survivors is
   epidemiologically blind; the contrast class is where the cause lives.
2. **Immunity is a first-class object.** `IMMUNITY(signature)` = the
   fact-facet set present in the healthy herd and absent in the wounded —
   the discriminating facets that answer the deepest question:
   *why has it NOT spread to the rest of the herd?* The boundary of the
   outbreak is the etiology.
3. **Outbreak forensics are W5H2 facets:** index case (first BIRTH of the
   signature: host + epoch), transmission vector (the shared change that
   carried it — commit, template, base image: the RIKSU edge it traveled),
   incubation (vector-arrival → symptom-onset lag). Each answer stamped
   MEASURED / DERIVED / ESTIMATED — a physician who confuses what he saw
   with what he inferred loses animals.

## §3 — The Two-Witness Cure Law

A causal edge (signature → cause → cure) is judged as in PB-365:

- **WITNESS 1 · DIAGNOSIS** — the signature matches the rule tablet.
- **WITNESS 2 · CURE** — the prescribed change turns the rerun **green**.

CONFIRMED requires both. Diagnosis without a green rerun is **DECLARED-ONLY**
(a hypothesis, never a template). A green rerun nobody can explain is
**OBSERVED-ONLY** — a ghost for Eṭemmu. No cure enters Nalbanu's brick-mold,
and no template enters Borsippa, without CONFIRMED status.

## §4 — Staging (the course, not the category)

Every signature carries a stage derived from its biography (Event KAKIs,
no silent close):

| Stage | Meaning |
|---|---|
| **INDEX** | first occurrence — one host, one event; the index case |
| **LOCALIZED** | recurring on one host |
| **EPIDEMIC** | spread across hosts — the fact-pattern (or a bad vector) is propagating |
| **RESOLVED** | later healthy runs on formerly wounded hosts; the scar remains in the ledger |

CHRONIC is not a category; it is a stage *history* (RESOLVED → INDEX → …
recurring across epochs).

## §5 — The Prognosis Triad (prescribe before refusing)

The pre-flight gate reads the stele index **before any playbook touches a
host** and answers with one of three:

- **BULṬU** — a sealed cure exists (bulṭu, remedy — sister-word of balāṭu,
  life). The gate injects the sealed template and proceeds.
- **CHRONIC** — recurring wound, no sealed cure. WARN loudly, journal,
  proceed under witness.
- **FATAL** — an approach explicitly sealed as forbidden. Only these refuse
  outright, and only behind a Puluḫtu-style two-lock.

Default is prescription. Refusal is reserved for sealed law — a hard gate on
fuzzy matches would strangle the pipeline with false positives.

## §6 — The Two Phases

- **Phase A (now, pre-database):** the **Bīt Asî shelf** — a filesystem
  court. Every wound run gzip-sealed as a crate with its sha-256 Kanīku,
  append-only ledgers beside it (wounds AND herd), crates chmod-read-only
  to all stakeholders but the Architect. The Kārum grammar, landward.
- **Phase B (when the seven waters flow):** one sail through the BeeMDM
  chain, **timestamps preserved**, GOLDEN wound tablets into EnkiDDB·7007.
  Šasû reads the sealed rule tablets as the diagnostic engine; Eṭemmu's
  surprisal flags the novel wound (−log P high → ghost → escalate to the
  Architect, never pattern-match falsely); the census derives the skeleton
  to 7006; Igigi Watch alarms on **acceleration of wound-birth rate** and
  on persistent β₁ loops (fix A exposes B exposes A — the topological
  signature of a design flaw, not a bug).

## §7 — HeptaScript Nouns (HS-EXT candidate, queued for Gula's court)

No sixth verb. Nouns only: `WOUND`, `SIGNATURE(...)`, `SYNDROME`
(a β₀ component of the wound hypergraph), `CURE`, `PROGNOSIS`
(BULṬU | CHRONIC | FATAL), `IMMUNITY(...)`, `STAGE`, and the certificate
function `PROVE cure(rerun_green)`. Canonical query:

```hepta
ORBIT ddb.tribes.wounds
  WHERE SIGNATURE MATCHES facts(host)
  PRESENT prognosis, cure, stage, IMMUNITY(signature)
  WHY    PROVE cure(rerun_green)
  WITNESS naru
```

## §8 — Law Tests

- **L-SGG-1** — no wound without its healthy herd recorded in the same harvest.
- **L-SGG-2** — no template minted from a cure that is not CONFIRMED (two witnesses).
- **L-SGG-3** — wound epochs are failure-moments; ingestion may never rewrite them.
- **L-SGG-4** — FATAL refusals only for explicitly sealed forbidden approaches; all else prescribes or warns.
- **L-SGG-5** — a signature matching nothing is a ghost: Eṭemmu escalates; the Asû never invents a diagnosis.
- **L-SGG-6** — wound flesh to 7007; skeleton to 7006; never inverted.

---

*Deposited by the scribe for the Architect's word. The system was always autobiography.
DUB.SAR 𒁾 · BahyWay.Ecosystem v4.0 · الحالة القديمة لا تُمحى أبداً*
