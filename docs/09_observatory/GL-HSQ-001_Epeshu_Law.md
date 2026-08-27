# GL-HSQ-001 — The Epēšu Law (From Tablet to Work)

**Corpus:** BahyWay.Ecosystem v4.0 — GL-series (sealed law corpus)
**Tablet:** GL-HSQ-001
**Epithet:** Epēšu — to make, to do: the Making
**Deposited:** 2026-08-22 · **Status:** SEALED · APPEND-ONLY
**Parents:** HeptaScript (Anti-SQL doctrine), EN-MDB-001 (Masku — EnkiMDB
7006), EN-DDB-003 (Šasû — machine-read sealed tablets), GL-NAM-002
(Ḫubullu — the gloss at sealing), GL-ALG-002/004 (Napharu roll, Kiṣru
formations), GL-REL-001 (Kanīku), AkkadianAOL codegen, MUMMU/G4 (Z3
design-time gate)
**Decree served:** *"The Arsenal shall be saved as GOLDEN particles for
internal use in EnkiMDB algebra_schema, selectable as snippets — with
theory, concept, theorem — as the base of new functions, procedures,
materialized views, triggers, and types."*

---

## Inscription

> *A law that can only be read is a monument; a law that can be summoned,
> quoted with its theorem, and made into work — that is a living god of
> the house. Epēšu: the tablet becomes the tool.*

---

## §1 — The GOLDEN Shelf (algebra_schema in EnkiMDB·7006)

Every member of the Algebra Arsenal is enrolled as a **GOLDEN particle**
in `algebra_schema` of EnkiMDB — GOLDEN in the sealed EAV sense: a stable
limit cycle, never mutated. Each law-particle carries, in EAV Mandatory
Attributes (ColourID lives here, never in KAKI bytes):

- `name`, `napharu_no`, `tier`, `kisru[]` (its formations, per GL-ALG-004)
- `equation` (canonical form), `gloss` (Ḫubullu plain words),
  `theorem` (the concept text), `provenance` (foreign credits)
- `po[]` (proof obligations with status WITNESSED/OPEN)
- `source_sha` (SHA-256 of the sealed tablet — the umbilical Kanīku)

**The GOLDEN Shelf Clause:** amendments enroll NEW particles beside their
elders (append-only); a shelf particle whose `source_sha` no longer
matches its tablet is a CORRUPTION event of the first rank.

## §2 — The Five Vessels (lawful names for the borrowed concepts)

Reified law lands in one of five sovereign vessels — the RDBMS notions,
renamed into the house and sharpened:

| Vessel | Akkadian | Is | RDBMS shadow |
|---|---|---|---|
| **ŠIPRU** | šipru — a work | pure function over Hepta state | function |
| **NĒPEŠU** | nēpešu — procedure text | multi-step rite with EMIT effects | stored procedure |
| **LIBITTU** | libittu — a brick | standing cast result, recast by Nalbanu on demand or schedule | materialized view |
| **MAṢṢARTU** | maṣṣartu — the watch | event-bound vigilance firing a rite | trigger |
| **ŠIKNU** | šiknu — form | sealed domain/type (closed per Simtu discipline) | type |

## §3 — The ALGEBRA Clauses (HeptaScript grammar, Anti-SQL absolute)

No foreign vocabulary enters. Two clause families are added:

**WITNESS LAW — quoting the shelf (query):**

```
WITNESS LAW GL-ALG-003
  PRESENT equation, theorem, po(*)
  ORBIT BY tier

WITNESS LAW * OF KISRU K1-MU
  PRESENT name, equation, gloss
  ORBIT BY napharu_no
```

The result is the snippet: equation + concept + theorem + PO ledger,
delivered as particles (never prose), each stamped with `source_sha`.

**REIFY LAW — the Making (instantiation):**

```
REIFY LAW GL-HYD-001 AS MAṢṢARTU leak_watch
  BINDING (eta := pipe.material.eta, R := pipe.friction, c := pipe.c)
  ON EVENT stroke.echo
  PROVE HYD-PO-1, HYD-PO-2
  EMIT verdict TO EnkiODB

REIFY LAW GL-ALG-003 AS LIBITTU mode_admissions
  BINDING (N := survey.N, T := survey.window, guard := survey.guard)
  RECAST EVERY campaign
  PROVE ALG3-PO-1, ALG3-PO-2

REIFY LAW GL-RU-001 AS SIKNU Pu
  DOMAIN 0..100 SEALED
  PROVE RU-PO(*)

SYNC LIBITTU mode_admissions        -- recast the brick now
WITNESS MASSARTU leak_watch PRESENT provenance, po(*)   -- audit the watch
```

## §4 — The Inherited-Proof Clause (the heart)

**REIFY without PROVE is refused.** Every vessel born of a law inherits
that law's proof obligations: WITNESSED POs become gate-hooks the vessel
re-runs at declared drills (Kiṣru Liveness §4); OPEN POs are carried
visibly as debts. *A snippet without its obligations is a rumor* — and
rumors do not compile.

## §5 — The Copy-Bound Clause

Reification copies; it never links live. The vessel embeds
`source_sha` and its own `kaniku`; the shelf particle remains GOLDEN and
untouched. If the source tablet is superseded, the vessel keeps working
under its sealed copy and is FLAGGED `ELDER-SOURCE` until re-reified —
old works do not silently change their law.

## §6 — The Design-Time Clause

REIFY is an act of the forge, not the field: HeptaScript hands the bound
snippet to the **AkkadianAOL code generator**, which emits pure Rust into
the owning crate; MUMMU's G4 gate (Z3) checks bindings and domains at
design time. **No runtime evaluation of equation text, ever** — shipped
binaries contain compiled vessels only. The 1-billion-particles law is
not negotiable for the sake of convenience.

## §7 — Worked Lineage (one example, end to end)

The Tirku Law sits GOLDEN on the shelf. An engineer writes the
`leak_watch` MAṢṢARTU of §3: BINDING pulls η from the pipe's Simtu-sealed
material facet; PROVE inherits HYD-PO-1/2 as drill-gates; codegen emits
`massartu_leak_watch.rs` into WPDEngine with the tablet's sha in its
header; G4 verifies domains; the watch fires on `stroke.echo`, runs the
grid-scan šipru (itself reified from the same law), and EMITs its
ESTIMATE to Enbilulu — verdicts still requiring two witnesses, because
the vessel inherited §6 of its mother-law along with her mathematics.

## §8 — Proof Obligations

- **HSQ-PO-1** — Shelf integrity: every algebra_schema particle's
  `source_sha` matches its sealed tablet (sweep).
- **HSQ-PO-2** — Refusal fidelity: REIFY without PROVE, with unbound
  symbols, or against a missing PO is refused with the reason named.
- **HSQ-PO-3** — Round-trip: a reified ŠIPRU reproduces its law's
  witnessed proof numerically (e.g. Askuppu stone values) from the
  generated code.
- **HSQ-PO-4** — Elder-source flagging: superseding a tablet flags all
  its vessels within one sweep.

## §9 — Closing

The scribes kept two rooms: the tablet-house where law slept, and the
workshop where tools were cut — and a wall between them, so that work
drifted from law and law never touched work. Epēšu removes the wall and
replaces it with a gate: the law walks into the workshop carrying its
theorem in one hand and its obligations in the other, and what is made
there is law-shaped forever.

*Summon the tablet; prove the debt; make the work.*

---

## Seal

Deposited append-only by **PB-HSQ-001**, which raises `algebra_schema`
and enrolls the named Arsenal as GOLDEN particles with their kiṣru,
equations, glosses, POs, and source hashes; SHA-256 in the GL ledger;
HSQ-PO-1..4 opened.

𒁾 DUB.SAR — sole architect, BahyWay.Ecosystem v4.0
