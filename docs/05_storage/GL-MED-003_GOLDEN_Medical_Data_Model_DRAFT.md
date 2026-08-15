# GL-MED-003 (candidate) — THE GOLDEN MEDICAL DATA MODEL
## Merck-as-GOLDEN-Source · Symptom↔Disease↔Image Schema · The Convergence Query
### BahyWay.Ecosystem v4.0 · binds GL-HS3-001 (grammar) · GL-VIZ-000 (BWVL) · the seven-depth cascade · Status: DRAFT — pending CSR-08 by DUB.SAR 𒁾

---

## 0 · Origin (why this model exists)

This model is drawn from lived field experience: a veterinarian carrying the
*Merck Veterinary Manual* as a daily companion. The Manual is authoritative,
comprehensive, curated — a true GOLDEN source. But it has a structural limit
that hurts most in an **outbreak**: it is organised *one disease per entry*, so
it answers "what is anthrax?" but cannot answer "I see sudden death + bleeding
from orifices + no rigor mortis across six animals — what converges on all
three, and show me the image." Correlating multiple symptoms under time
pressure, with diagnostic imagery at the junctions, is the query a book cannot
serve. **This instrument exists to serve exactly that query.**

---

## 1 · The Merck-as-GOLDEN doctrine

A GOLDEN source is authoritative, curated, and citable. The Merck Veterinary
Manual is the archetype: every disease entry, every clinical sign, every
diagnostic image is a **GOLDEN particle** with provenance. BeeMDM ETL ingests
such sources into EnkiDB (OLTP) → EnkiDW (OLAP) as GOLDEN records. The
instrument never invents; it reads what the GOLDEN source states (GL-HS3-001
four-outcome contract).

---

## 2 · The schema — three particle kinds, not one

A manual stores **diseases**. This model stores **three kinds of GOLDEN
particle and the edges between them**, because the field query is symptom-first:

- **SYMPTOM particle** — an observable clinical sign (sudden death, crepitant
  swelling, dark non-clotting blood, absence of rigor mortis, bleeding from
  orifices, fever, lameness).
- **DISEASE particle** — a diagnosis (anthrax, blackleg, malignant edema,
  lightning strike).
- **IMAGE/EVIDENCE particle** — a diagnostic image or finding attached at the
  junction (colony morphology, necropsy lesion, post-mortem sign).

**Edges (all GOLDEN-sourced, all carry confidence + citation):**
- `SYMPTOM —presents_in→ DISEASE` (weighted: how characteristic)
- `DISEASE —differentiated_from→ DISEASE` (the differential set)
- `DISEASE —shows→ IMAGE` (the picture to match in the field)

This is the inversion a book cannot do: **enter from symptoms, converge on
disease.** The manual is disease→symptom; the instrument is symptom→disease.

---

## 3 · The Convergence Query (the outbreak differential)

The primary mode. The field user drops **several observed symptoms** into the
instrument. Each symptom sends filaments to every disease it presents in. The
disease-knots **light up by how many of the observed symptoms converge on
them** — that convergence count, weighted by confidence, IS the ranked
differential diagnosis.

    ASK converge OVER symptoms {sudden_death, dark_unclotting_blood, no_rigor_mortis}
      PROVE OVER EnkiDW SCOPE domain="bovine_infectious"
      RANK diseases BY convergence_weight
      SHOW image AT each disease junction
      WITNESS result → StoryEngine

Four-outcome honesty applies per disease:
- **FACT** — all/most symptoms converge, high confidence → bright knot, ranked top.
- **WEAK** — few symptoms, low confidence → dim knot, flagged.
- **GHOST** — a sub-threshold match (one soft sign) → faint, "consider / needs lab."
- **NONE** — no convergence → dark, not shown.

The instrument shows *which symptom is doing the work* (the discriminator) and
where the differentials overlap — e.g. anthrax vs blackleg vs malignant edema
all converge on "sudden death," but "crepitant swelling" discriminates blackleg,
and "no rigor mortis / dark unclotting blood" discriminates anthrax. **The
discriminating symptom is the finding.**

---

## 4 · Images at the junctions (the field companion)

At cascade depth **D5 (concept)** and **D6 (records)**, the diagnostic
IMAGE particles attach. When the convergence lands on a disease, the user does
not just get the word — they get the **image to match against the animal in
front of them**: the crepitant swelling, the dark tarry blood, the necropsy
lesion. This is the visual presentation the book could not provide at speed.

---

## 5 · Binding to the seven-depth cascade (GL-VIZ / BWVL)

| Depth | Medical stratum |
|---|---|
| D1 Cosmic Web | whole GOLDEN corpus — all domains as density field |
| D2 Region | one domain (e.g. Bovine Infectious) |
| D3 Filament | a symptom→disease convergence chain |
| D4 Cluster | the differential set (diseases sharing symptoms) |
| D5 Knot | one disease + its facets (organism, signs, IMAGES) |
| D6 Neighborhood | the GOLDEN records grounding the facet (Merck entries, case reports) |
| D7 Particle | one KAKI record; BWVL animates by lifecycle state |

Descending = descending the data's own hierarchy. The convergence query enters
at the symptom particles and lets the filaments pick the depth-4 cluster to
resolve.

---

## 6 · Real worked example (grounded, checkable)

From the Merck Veterinary Manual + real sources (verified):
- **Sudden death** presents in: anthrax, blackleg, malignant edema, lightning
  strike (Merck: anthrax "must be differentiated from other conditions that
  cause sudden death").
- **Crepitant/emphysematous swelling of large muscles** → blackleg (Merck:
  "crepitant swellings of the large muscles suggests blackleg").
- **Dark blood that fails to clot + bleeding from orifices + absence of rigor
  mortis** → anthrax (Veterian Key / OSU extension).
- **Differential**: Merck explicitly lists blackleg ↔ malignant edema ↔ anthrax
  as mutually differentiated; necropsy alone unreliable for blackleg vs
  malignant edema → lab confirmation (the GHOST/needs-research outcome).

So dropping {sudden death, crepitant swelling} converges hard on **blackleg**
(FACT), with anthrax and malignant edema as WEAK differentials; swapping
crepitant swelling for {dark unclotting blood, no rigor mortis} swings the
convergence to **anthrax**. The instrument shows the swing — that is the triage
a book cannot do.

---

## 7 · Codex compliance & placement
- **A-1 zero new mathematics**: composes GOLDEN scoring, GL-HS3-001 grammar,
  BWVL cascade, Graph-RAG retrieval. New = the symptom↔disease↔image schema +
  the convergence-ranking query.
- **A-4 cited**: GL-HS3-001 · GL-VIZ-000 · GL-DDB-002/004 · seven-depth cascade.
- **PB**: PB-363 `convergence-query` engine; PB-364 `image-junction-attach`.

## 8 · Open seals for CSR-08
Adoption of the three-particle schema · "convergence_weight" ranking formula ·
whether IMAGE particles get their own KAKI or attach as EAV to DISEASE ·
symptom-first as the DEFAULT mode · PB-363/364 numbering.

*Recorded in the reign of Gudea 1.0. Drawn from the field. Nothing sealed until
DUB.SAR confirms under CSR-08.*
