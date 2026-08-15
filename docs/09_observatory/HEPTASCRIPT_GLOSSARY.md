# HeptaScript — Glossary

Terms an engineer needs to read `crates/heptascript`, `bahyway-z3`
(Gate G4), and `compare-tribe-schema` without cross-referencing five
other files first. HeptaScript-specific terms first, then the shared
BahyWay primitives it's built from. Companion to
`docs/08_pipeline_alaktu/HEPTASCRIPT_QUERY_LANGUAGE.md` (the Manual) — that
document is the how-to-write-a-query reference with a circuit-structure
snippet per clause; this one is the what-does-this-word-mean reference.

## Historical Background — Why Seven

In ancient Mesopotamia, the number seven (Sumerian *imin*; the
Akkadian numeral is *sebe/sebet* — **Sebettu**/**Sebitti**, "the
Seven," is the proper name of a divine group, not the numeral itself)
represented totality and cosmic completeness, and recurs across
ritual, architecture, and myth as a marker of finality or absolute
power. HeptaScript's seven dimensions are not a decorative borrowing
of that number — they are, specifically, a **judgment rendered on
every particle**, and that is exactly the myth this section traces.

**Gudea's seven-day dedication — the release-era warrant.** Gudea of
Lagash's Cylinder B describes the seven-day dedication festival of the
Eninnu (Ningirsu's temple), during which social hierarchies were
ritually inverted — the slave beside the master, the maid equal to her
mistress. Since this ecosystem's founding release is *Gudea 1.0*, this
is not incidental trivia: it is a release codename attested in a
primary source celebrating a completed build, sealed with a seven.

**The Sebettu — two aspects of the same Seven, not two teams.** The
Sebettu appear in the Erra Epic as fierce warrior gods, given by Anu
to march at Erra's side — destructive but unambiguously divine. A
related but distinct tradition, the Utukkū Lemnūtu incantations,
names the "evil Seven," offspring of Anu and the earth, blamed for
plagues and eclipses (their attacks on the moon god are the mythic
account of a lunar eclipse). These are two attested *aspects* of one
Seven, not a matched pair of opposing armies — later popular retellings
that split them into "Seven Evil Demons" versus a symmetrical "Seven
Beneficent Gods" invent a symmetry the primary sources do not have.
The seven protective clay figurines Neo-Assyrian exorcists (āšipu)
buried at doorways are chiefly the seven **apkallū** — the sages, shown
in fish-cloak and bird form — not a separate beneficent team.

**The Anunnaki judges — the myth-to-architecture link.** In Inanna's
Descent to the Netherworld, Inanna passes through seven gates, forced
to surrender a piece of her royal regalia at each one, arriving naked
and powerless before Ereshkigal — and it is the **seven Anunnaki
judges** of the underworld who then fix their gaze on her and pronounce
judgment. Seven gates, then seven judges: since HeptaScript's seven
dimensions are precisely a judgment rendered on every particle, this
is the direct mythic warrant for the architecture, not a loose analogy.

**The apkallū and the scribe.** The Seven Sages (apkallū) are credited
in Sumerian myth with bringing writing and the arts of civilization to
humanity before the Flood — making them the mythological ancestors of
the scribal art itself, and so of the DUB.SAR (𒁾) who seals these
documents.

**Astronomy, and where the trail runs out.** The seven visible moving
celestial bodies (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn)
were tracked as the seven gods directing human fate, and Mesopotamia
genuinely contributes both this astronomy and the sanctity of
7/14/21/28 as significant days of the lunar month. It is fair to call
this an **ancestor** of the modern seven-day week; it overclaims to
call it a *direct* ancestor — the planetary week as inherited today
crystallized centuries later, in Hellenistic astrology. Likewise, most
ziggurats had two or three tiers; a full seven stages is essentially
attested only for the exceptional, late Etemenanki of Babylon (partly
via reconstruction from the Esagila tablet), and "each tier a planet"
is a Herodotus-era and modern interpretive gloss, not a claim
Mesopotamian sources themselves make. Both are worth naming as real
threads without dressing them as more settled than they are.

## HeptaScript-specific

**W5H2** — the naming convention for HeptaScript's seven core v1
clauses: **W**HO, **W**HAT, **W**HEN, **W**HERE, **W**HY, **H**OW,
**H**OW MUCH. Every clause added since (v2.0's ten routing/intelligence
clauses, v2.1's MEASURE/GRAVITY) extends this same single `HeptaQuery`
AST rather than inventing a second grammar.

**Triple-O (Orbit-Oriented Ontology)** — 🔒 LAW, sealed `PH-001`. The
foundational design stance HeptaScript enforces at the language level:
every entity is a Particle, every change is an orbital event, history is
never overwritten. There is no SELECT/INSERT/UPDATE/DELETE.

**The Five Sovereign Operations** — the complete, deliberately closed
verb set (`operations::Operation`, `Operation::ALL.len() == 5`, asserted
by its own test): **ORBIT** (read/scan, the default), **EMIT** (birth
retrieval), **PROVE** (point lookup with full per-epoch history), **SYNC**
(order-independent state fingerprint), **WITNESS** (content digest). All
five stay strictly read-only. New aggregate functionality (MEASURE,
GRAVITY) is added as *clauses* that combine with any of the five, never
as a sixth verb — see Anti-SQL Law below for why that boundary is
enforced.

**Anti-SQL Law** — the Architect's standing rule, stated directly: *"it
will [be] no use of SQL Methods in my HeptaScript Query Language or any
other Dependencies of third parties technologies."* `Operation::parse()`
explicitly rejects `SELECT`/`INSERT`/`UPDATE`/`DELETE`/`FROM`/`WHERE`
(as a verb)/`JOIN`/`GROUP`/`ORDER`/`TABLE`/`VIEW` with a dedicated
`OperationError::SqlForbidden`. MEASURE/GRAVITY are this law's answer to
"how do I aggregate, then" — COUNT/SUM/AVG/GROUP BY equivalents built
from this ecosystem's own algebra and hash-index primitives, never SQL,
never a bundled third-party aggregation engine.

**QueryPlan** — the v2.0 routing/governance directives (`NODE`,
`ACROSS`, `TIER`, `STATE`, `NASH`, `PATTERN`, `LINEAGE`, `GATE`,
`SATAMU`, `ORBITAL`) copied out of a parsed `HeptaQuery` into one struct,
handed back alongside `QueryResult` for an upstream dispatcher to act on.
See the Manual §5 for which of these are 🧩 PARTIAL (parsed, carried,
not yet acted on) vs ✅ VERIFIED.

**MatchedEntity** — one entity that survived WHERE/WHY filtering, with
its WHAT-projected `(attr, value)` pairs (and, for PROVE, its full
per-epoch `history`). The unit `QueryResult.matched` is a `Vec` of.

### v2.1 — Anti-SQL aggregate clauses (2026-07-22)

**MEASURE** — the clause computing one aggregate value, either over the
whole matched set (no GRAVITY present) or once per GRAVITY group. Three
forms: `DENSE`, `FLUX <attr>`, `ROTOR_MEAN <attr>`.

**DENSE** — MEASURE's COUNT equivalent: a real, single-pass O(N) tally
of matched particles. Named to distinguish it from the *wrong* way to
compute a count in this codebase's own algebra — see Wedge Product
Collapse below for exactly why that shortcut was rejected.

**FLUX** — MEASURE's SUM equivalent: a component-wise `Multivector::add`
fold, degenerated to ordinary numeric addition since the EAV store only
ever holds scalar `AkkValue`s (no multivector-typed attribute exists to
sum component-wise in practice).

**ROTOR_MEAN** — MEASURE's AVG equivalent, for angle-valued attributes:
the closed-form circular mean `atan2(Σsin θ, Σcos θ)`, one pass. A plain
arithmetic mean is *wrong* for angles near the ±π wraparound (two
orientations just past ±π are geometrically almost identical, but their
arithmetic mean lands near 0 — the opposite direction); ROTOR_MEAN gets
this right because it treats the attribute as a point on a circle, not a
line.

**GRAVITY** — the clause partitioning the matched set into groups before
MEASURE runs per group — HeptaScript's GROUP BY equivalent. Requires an
explicit `BAND <width>` or `MAX_GROUPS <n>` — the parser refuses to parse
GRAVITY without one (`maybe_parse_gravity`), the concrete, enforced
answer to GROUP BY's real unbounded-memory risk.

**BAND** — a GRAVITY mode: numeric bucket partition, `floor(value /
width)`. Inherently bounded by `(attribute range / width)`, independent
of the attribute's value cardinality — safe at any N with no separate
cap needed.

**MAX_GROUPS** — a GRAVITY mode: exact-attribute-value partition, hard
capped at `n` distinct groups. Once the cap is reached, every further
new distinct value folds into a single `GravityKey::Overflow` sentinel
bucket instead of growing group memory further — memory stays
O(max_groups) regardless of the attribute's true cardinality.

**GravityKey::Overflow** — the sentinel group every particle whose
distinct GRAVITY-attribute value arrived *after* `MAX_GROUPS` was already
reached gets folded into, rather than being dropped or allowed to grow
group memory unbounded. `GravityResult.capped` reports whether this
bucket was ever used during a given run.

**GravityKey::Missing** — the sentinel group for a particle that lacks
the GRAVITY attribute entirely. Exists so group counts always sum to the
true total matched — no particle is ever silently excluded from the
partition just for lacking the grouped-on attribute.

**MAX_REASONABLE_GROUPS** — the constant (100,000) Gate G4 checks a
query's `MAX_GROUPS` value against. A `MAX_GROUPS` past this ceiling is
technically present (satisfying the parser's mandatory-cap rule) but too
large to bound a result set anyone can actually consume — the same
"present but too large to help" failure mode is why this check exists
separately from the parser's own presence-only check.

**Wedge Product Collapse** — the concrete, tested reason MEASURE DENSE
is a real tally rather than a wedge-product magnitude. `bahyway-algebra`'s
own test `wedge_of_dependent_is_zero` proves wedging more vectors than
the algebra's dimension (`Cl(7)`, 7 basis vectors) collapses the result
to zero — so a "count via wedge magnitude" shortcut silently returns 0
for any real Tribe beyond 7 particles. Discovered and rejected during
this feature's design, before any code shipped.

**"The index determines which; the algebra determines what"** — the
design law this whole feature (and the CompareEngine dedup station
below) was built against, stated by the Architect and confirmed true
against every real risk found while designing it: *the index determines
which particles a query ever touches; the algebra determines what gets
computed once that set is already bounded.* Concretely: WHO/WHERE/WHY
and GRAVITY's cap decide the candidate/group set size; DENSE/FLUX/
ROTOR_MEAN only ever compute over a set already bounded that way. Never
let an algebraic/exclusion computation substitute for an index at scale.

## Gate G4 (`bahyway-z3`)

**Gate G4** — the dev-time-only Z3-backed HeptaScript query validator
(`bahyway-z3` crate). Sealed in `BC-ENV-001_Enbilulu_Calculus`: *"Z3 is
design-time only... never present in the shipped EnbiluluEngine
binary."* No shipped sovereign server binary may ever depend on it.
Originally checked only WHERE/WHY scalar constraint satisfiability
(catching e.g. `age > 10 AND age < 5`); extended 2026-07-22 to also flag
an unreasonably large GRAVITY `MAX_GROUPS`.

**ValidationReport** — Gate G4's output: a `SatOutcome`
(`Satisfiable`/`Unsatisfiable`/`Unknown`), the `contradiction_sources`
that caused an UNSAT verdict, and (new) `gravity_warnings` — independent
of `outcome`, since a query can be perfectly satisfiable and still carry
an unsafe GRAVITY cap.

**Unsat core** — Z3's mechanism for reporting *which* asserted
constraints participated in an unsatisfiability proof. Gate G4 uses it
to name which WHERE/WHY conditions contradict each other, rather than
just reporting "impossible" with no explanation.

## Pauli Exclusion / CompareEngine

**Pauli Exclusion Principle** (as realized here) — `bahyway-algebra::
enlil`'s rule that no two particles may share the same `QuantumCoord`
(Identity-KAKI prefix, orbit_position, state) triple simultaneously. Pure
per-pair logic (`pauli_check`), correct at any N — the O(N²) risk lives
entirely in how a caller drives it across a candidate set, never in the
function itself.

**QuantumCoord** — the three-field key `pauli_check` tests for equality:
`kaki_prefix: [u8; 8]`, `orbit_position: u32`, `state: QuantumState`.
Derives `Eq`+`Hash`, which is exactly what makes a `HashMap`/`HashSet`
index over a set of `QuantumCoord`s the O(1)-per-lookup equivalent of
`pauli_check`'s own O(N) linear scan.

**CompareEngine** — the BeeMDM ETL pipeline's comparison station family
(`compare-tribe-schema` crate). Historically schema/template-shape
comparison only (`compare_versions`, `SchemaDiff`); extended 2026-07-22
with a particle-level station.

**`pauli_dedup` / `dedup_tribes`** — the new hash-indexed Tribe-vs-Tribe
Pauli-duplicate dedup station. Compares a previous Tribe Orbit snapshot
against a current one and flags every particle whose `QuantumCoord`
collides with one already in the previous snapshot. Builds a `HashMap`
index over `previous` once (O(N)), then probes it once per `current`
particle (O(1) amortized): **O(N+M) total, never O(N×M)** — the same
"hash join, not nested-loop join" principle behind this workspace's own
EAV exact-match index and GRAVITY's MAX_GROUPS cap.

## Wire protocol (v2.1 addendum)

**Aggregate trailer** — the byte sequence the seven `*-read-server`
binaries append to the end of a binary QUERY response frame when a
query carries MEASURE and/or GRAVITY: `[u8 aggregate_tag]` (0=none,
1=measured, 2=grouped) followed by the encoded `MeasureValue` or
`GravityResult`. Appended *after* the existing verb trailer (sync
fingerprint / witness digest), since an aggregate can co-occur with any
of the five sovereign verbs, not just SYNC/WITNESS.

**MeasureValue** — the wire/Rust encoding of one MEASURE result:
`Dense(u64)`, `Flux(f64)`, or `RotorMean(f64)`. `Dense` is encoded as a
64-bit count (not the 32-bit row counts used elsewhere in the same wire
format) specifically so it never wraps at large Tribe scale.

**GravityResult** — the wire/Rust encoding of one GRAVITY result: the
list of `GravityGroup`s (key, count, per-group `MeasureValue`) plus a
`capped: bool` reporting whether `MAX_GROUPS` was actually reached
during that run.

## Foundational (shared across the Ecosystem — brief; full definitions in `docs/00_codex/BAHYWAY_ECOSYSTEM_V4_GLOSSARY.md`)

**KAKI / Identity-Kaki / Event-Kaki** — the 16-byte sovereign primary key
format every particle is identified by; an Identity-Kaki is the "birth
certificate," an Event-Kaki is one immutable state-transition record.

**EAV / Particle** — Entity-Attribute-Value. What WHERE/WHAT actually
read and write against: which entity (an Identity-Kaki), which attribute
(a dotted namespaced string like `meta.title`), what value (an
`AkkValue` — the same scalar-only type MEASURE FLUX/ROTOR_MEAN read).

**Journal** — the append-only Write-Ahead Log every `HeptaQuery`
ultimately scans (directly, on a Write Node; via materialized Data Files,
on a Read Node).

**Tribe** — a named grouping of particles sharing a schema/source
lineage. `WHO <Tribe>.<Var>` names one (though see the Manual §3/WHO for
the honest caveat that Tribe/Var binding isn't enforced at runtime
today — a query only ever runs against one Journal already).

**BIGRING** — a federation of Tribes/clients this ecosystem's `ACROSS`
clause and `SumuUkinContext::route()` mechanism name; see the Manual §7
for which "multiple Tribes/BIGRINGs" mechanisms are real today.

**Clifford Algebra `Cl(7)` / Multivector** — `bahyway-algebra::clifford`'s
128-blade (`BLADES = 128`, `DIMS = 7`) real geometric algebra
implementation. FLUX's `Multivector::add` fold and the Wedge Product
Collapse entry above both come from this module.

**Rotor** — `bahyway-algebra::rotor`'s simplified single bivector-plane
rotor type (`{ scalar, bivector }`). ROTOR_MEAN's one-pass closed-form
circular mean is correct specifically *because* this `Rotor` is
single-plane — a general multi-plane rotor would need an iterative
Fréchet-mean instead.
