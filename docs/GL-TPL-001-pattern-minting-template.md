# GL-TPL-001 — Pattern Minting & Template Law
**Status:** SEALED (concept). Implementation queued behind PB-160.
**Engines:** CompareEngine (similarity, ratified);
TemplateEngine (mold-maker — god-name PENDING the
Architect's ruling; Kulla proposed). AAOL Core generates
the .tmpl artifact from the pattern's .akk tablet.
**Arsenal residence:** EnkiMDB · 7006.
**Surfaces:** MARDUK stage + MADANU court (DubSar Theater).

## Clause 1 — Story Featurization (Shingles)
A particle's behavior is matched as a SET, derived from
its StoryEngine chronicle: transition bigrams
(INGEST->STAGE, STAGE->TRANSFORM, ...), anomaly tokens
(REWORK@station, DWELL>Nd@station), and terminal tokens
(PROVE_FAIL, KISPU_RITE). A pattern Template's canonical
shingle set is the agreed set of its founding cohort.
Jaccard similarity J(A,B) = |A∩B| / |A∪B| is computed
between cohort and Template shingle sets.

## Clause 2 — The Two-Witness Verdict
A match requires two independent witnesses:
  (a) Jaccard on story shingles — symbolic story shape;
  (b) Nabû-metric distance to the Template particle's
      Hepta coordinates — continuous position.
Both above threshold  -> CONFIDENT MATCH (auto-label).
Witnesses disagree    -> FUZZY MATCH (steward reviews
                         in the MADANU court).
Both below threshold  -> NOVEL PATTERN CANDIDATE.
Thresholds are EAV attributes of each Template — tunable
per pattern, never hard-coded.

## Clause 3 — The Scale Clause (MinHash)
At the billion-particle law, exact Jaccard per particle
is unlawful arithmetic. CompareEngine carries MinHash
signatures — a fixed small array of hash minima per
shingle set, pure Rust, zero external crates — giving
O(1) approximate Jaccard per comparison. The signature
is small enough to live as an EAV attribute on every
particle; the arsenal search is thereby a signature
scan, sovereign and sub-second.

## Clause 4 — The Abstraction Clause (IP boundary)
A Template captures SHAPE ONLY: shingle sets, MinHash
signature, Hepta centroid and tolerance ellipsoid,
thresholds, the HeptaScript recognition clause, and a
human description. It NEVER contains client values,
identifiers, records, or reconstructable fragments
thereof. Provenance records the engagement abstractly
(domain class, date, steward), never the client's data.
The arsenal is thereby the Architect's own portable
intellectual property, lawfully reusable across clients.

## Clause 5 — The Minting Rite
DETECT: CompareEngine reports a cohort matching no
        Template above threshold.
PROVE:  support statistics — cohort size, internal
        coherence, separation from existing Templates.
        No pattern is minted from a handful of outliers.
JUDGE:  the steward names, describes, and APPROVES the
        pattern in a MADANU-style rite (CSR-08). An
        unapproved pattern is a rumor, not a law.
MOLD:   TemplateEngine emits the .akk tablet; AAOL Core
        generates the .tmpl artifact; AkkadianSeal
        (Ed25519) is affixed.
Naming of behavioral patterns follows NL-001 clause 4
(sages name patterns) once the Architect seals the
behavioral-sage registry; descriptive names serve until.

## Clause 6 — Template as Particle (the Keystone)
Per the Hepta Space Uniqueness Law, Templates are
particles. Every .tmpl receives full KAKI v4.0 identity:
kappa[0..3] uuid_hash, kappa[4..5] arsenal tribe_id,
kappa[6] kaki_type=Template, kappa[12..13] timestamp,
kappa[14..15] CRC-16/CCITT. It registers in EnkiMDB ·
7006 with EAV Mandatory Attributes: version, release era
(king-named per NL-001), data-structure reference,
approved flag + approver, created/approved dates,
abstracted provenance, thresholds, seal. Templates are
therefore fully HeptaScript-queryable:

    PRESENT TEMPLATE
      WHAT  kind     = Pattern
      WHO   approved = DUB.SAR
      WHEN  release  = Zagesi
      HOW   version  >= 1.2
    ORBIT ArsenalShelf

Recognition round-trip: the same .tmpl loaded in DubSar
Theater identifies the same pattern in any later client
engagement — the arsenal audits and extends itself with
its own instruments.

## Authority Note
Inherits all standing law: stage-never-truth (GL-DST-001);
witness scopes (GL-VIZ-002); Tupsimati binding
(GL-DST-002); append-only decrees (GL-DST-003); CSR-08;
NL-001 orthography; the 1-billion-particles-under-
1-second runtime law.

— Inscribed for DUB.SAR 𒁾, BahyWay.Ecosystem v4.0.
