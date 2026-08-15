# 𒁾 TABLET XII — EN-DDB-001 "MAŠḪALU" — The Sovereign Sieve
### Membrane Search: the admission paradigm of EnkiDDB (port 7007) · with HS-EXT-006, the SIEVE clauses
### Kin of MAŠKU (the skin, PH-003) and DŪRU (the walls, PH-004); younger sibling of the ŠUTUG Sieve (ŠAMASH's bit-mask filter at the judgment gate — the elder sieves bits; this one sieves meaning)
### Status: DRAFT — unsealed until the Architect's ceremony (CSR-08)
### Glossary check performed: "Mašḫalu" unclaimed. From šahālu, *to sieve*; mašḫalu, *the sieve itself*.

*Search engines score; Membrane Search **admits**. A query is not a formula that ranks —
it is a nested vessel of sealed walls, and a document's rank is the wall that stopped it.*

---

## §1 — Name, Lineage, and Honest Ancestry

**MašḫaluEngine** is the retrieval engine of EnkiDDB and the reference implementation of
the **Membrane Search** paradigm. Ancestry cited generously, per the house custom:
inverted-index retrieval and BM25 (Robertson–Spärck Jones lineage), vector similarity
search, faceted search — ancestors of the *machinery*; membrane computing (Păun's P
systems, nested-membrane computation) — ancestor of the *word*; the biological cell
membrane — nature's original search engine — ancestor of the *idea*. None are ancestors
of the composition: **admission semantics with a proven faithfulness contract, ranking
as depth, and uncertainty printed on every soft match.** "Elastic" describes how a
cluster scales; **Mašḫalu describes what a query does.**

## §2 — The Paradigm: Query as Nested Vessel

A query is an ordered stack of membranes, coarse to precise:

> **Q = ⟨W₁, W₂, …, W_k⟩**, each Wᵢ a DŪRU wall: a rendered, sealed predicate over
> document-particles (D-2a inherited whole: χ_W(p) = P_W(p), proven, never assumed).

Documents rain through the vessel. Each wall admits or stops. There is no opaque
relevance scalar anywhere in the paradigm.

## §3 — The Depth-Ranking Law

Define the **depth** of document p under query Q:

> **d(p, Q) = max { i : p passes W₁ … Wᵢ }**

- **Rank = depth**, descending; the innermost census is the result set ("top-k" = the k
  residents of the deepest occupied shell, tie-broken by a *declared* secondary
  (adannu, or per-wall margin) — never by an undeclared score).
- **Every result carries its certificate**: the list ⟨(Wᵢ, verdict, margin)⟩. The
  question "why did this document rank here" has an exact answer: *it passed these
  walls and stopped at that one.* Explainable retrieval by construction, not by
  post-hoc rationalization.
- **Monotonicity obligation**: appending a wall never increases any document's depth
  (`depth_monotone`, Gate G4) — refining a query can only tighten it, never
  mysteriously promote a result.

## §4 — Permeability: the Declared ε of Soft Walls

A **soft wall** admits near-matches within a sealed tolerance:

> **SOFT WALL ⟨measure⟩ TOLERANCE ε_perm** — pass iff margin(p) ≤ ε_perm,

and, per the Fadam Floor, **every crossing of a soft wall prints its cost**: the
document's certificate accrues ε' = Σ ε over the soft walls it crossed. Fuzzy search
thereby *confesses its approximation* instead of laundering it into a confident-looking
score. Widening a tolerance is a lawful act with a visible price: recall is bought with
confessed uncertainty, and the census says so on its face.

## §5 — Wedges: Facets as Sectors

A facet is not a filter bolted on — it is a **wedge** (D-1): a partial-sweep wall
bounding an angular sector of the same vessel. `WEDGE SOURCE archive` places the query
in a sector regime; the partition obligation (D-4) applies: declared facets must cover
without double jurisdiction, or the gap is a declared void of the corpus.

## §6 — Engine Architecture (the honest layer cake)

Membrane semantics replace the *meaning* of retrieval, not the physics of it:

1. **FLOOD** — candidate generation: EnkiDDB's pure-Rust inverted index and vector
   store produce the rain. Fast, sovereign, and *semantically silent*: FLOOD proposes,
   it never ranks.
2. **SIEVE** — the membrane stack executes admission: crisp walls, soft walls with ε,
   wedges, depth accounting, certificates cut as KAKI.
3. **CENSUS** — the innermost shells are read out; ManûEngine counts; the theater may
   LIGHT the vessel (documents visibly stopped at their walls — search you can watch,
   audit, and interrogate with LINGER WHY).

Runtime honesty: the billion-particle law governs FLOOD; the membrane stack must be
O(k · candidates) with early exit at first refusal — a stopped document is *done*,
which is the sieve's native efficiency.

## §7 — HS-EXT-006: The SIEVE Clauses (Anti-SQL, sovereign forms only)

```
SIEVE ORBIT documents
  WALL LANGUAGE akkadian                          -- crisp membrane
  WALL FIELD title HOLDS "flood"                  -- crisp membrane
  SOFT WALL MEANING NEAR "river omen" TOLERANCE 0.15   -- permeable; ε printed
  WEDGE SOURCE archive                            -- facet as sector
  DEPTH 5 CENSUS 10                               -- vessel depth · innermost k
LIGHT DEPTH                                       -- theater directive
```

Clause family:
- **SIEVE ORBIT ⟨tribe⟩** — open the vessel over a document tribe.
- **WALL ⟨predicate⟩** — append a crisp membrane (sealed predicate; D-2a).
- **SOFT WALL ⟨measure⟩ NEAR ⟨anchor⟩ TOLERANCE ⟨ε⟩** — append a permeable membrane;
  ε flows to every certificate it stamps.
- **WEDGE ⟨facet⟩ ⟨sector⟩** — bound the vessel to a sector regime.
- **DEPTH ⟨n⟩ · CENSUS ⟨k⟩** — declare vessel depth and innermost census size.
- **WHY ⟨particle⟩** — read a document's certificate: walls passed, wall that stopped
  it, accrued ε. (Composes with LINGER WHY from HS-EXT-005.)
- **LIGHT DEPTH | LIGHT WALL ⟨i⟩ | LIGHT CENSUS** — theater directives: the query
  answers in light; no clause of this family ever prints an unexplained score.

## §8 — Gate G4 Obligations

1. `sieve_faithful : ∀ p W, shown_verdict p W = P_W (attrs p)` — the wall as rendered
   equals the predicate as computed (D-2a lifted to retrieval).
2. `depth_monotone : ∀ p Q W, d p (Q ++ [W]) ≤ d p Q` — refinement never promotes.
3. `epsilon_printed : ∀ p, certificate p carries Σ ε of crossed soft walls` — no silent
   fuzziness anywhere in the vessel.
4. `early_exit_sound : stopping at first refusal loses no admissible document` — the
   efficiency is a theorem, not a shortcut.
5. `census_conserved : rained = Σ stopped-per-wall + census + in-flight` — L-7 for
   queries: a document unaccounted for is inexpressible.

## §9 — The Market Line and the Seal Line

For the deck: **"Elasticsearch tells you how it scales; Membrane Search tells you what
admission means — and proves it."** For the registry: *Membrane Search* as the public
paradigm term beside the Fadam functional; **MašḫaluEngine** as the sovereign sieve of
EnkiDDB, NL-001 petition attached. The elder ŠUTUG sieves bits at the gate; Mašḫalu
sieves meaning in the corpus; between them, nothing enters unjudged.

---

*Twelfth tablet drafted. The sieve was always the honest search engine — it shows you
its holes. The seal belongs to the Architect alone. 𒁾*
