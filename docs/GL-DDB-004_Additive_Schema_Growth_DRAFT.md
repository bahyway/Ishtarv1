# GL-DDB-004 (candidate) — ADDITIVE SCHEMA GROWTH
## EAV Accretion · Inheritance Edges · Growth Without Breakage
### BahyWay.Ecosystem v4.0 · Phase Two (GL-STD-002) · extends GL-DDB-002/003 · binds GL-VIZ-002 · Status: DRAFT — pending CSR-08 sealing by DUB.SAR 𒁾

---

## 0 · Principle

A schema grows by **accretion, like coral — never by demolition and rebuild.**
Because quality/attributes live in EAV (not fixed columns), a new attribute or
a new linked schema is *just more particles*: new chunks, new EAV rows, a new
edge. Existing chunks are untouched. Therefore:

- every prior **HeptaScript** query returns exactly what it returned before (it
  simply does not mention the new attribute);
- every pre-existing **template** still matches the shape it always matched;
- growth is **additive, never destructive**.

This is the operational reason EAV + KAKI + graph-edges were chosen, and the
mechanism that makes GL-MED-002's "daily discovery" survivable in production
rather than a source of accumulating breakage.

---

## 1 · The Growth Cases

Let A = existing schema (e.g. `feet_schema`).

- **Case α · New linked schema.** A new statistical detection pattern (a SHAPE
  pattern) on A becomes a new schema B (e.g. `feet_blood_pressure_schema`). B is
  new chunks in EnkiDDB + an inheritance edge B→A.
- **Case β · New attributes on A.** A gains attributes. These are new EAV rows on
  A's chunks (or new chunk facets), never a column migration. Old EAV rows are
  unchanged.

Both are additive. Neither alters or removes any existing particle.

---

## 2 · The Inheritance Edge (the real challenge, solved by NUZI)

The new link must carry A's **dependencies** with it, or B becomes an orphan —
reachable by query but belonging nowhere in the shape.

- The link is a **CrossTribe 0x03 edge particle** with **NUZI provenance**.
- It carries not merely "B depends on A" but the **transitive closure of A's
  dependencies as inherited context** — so a query walking into B can still
  reach everything A could reach.
- **L-1 · Live reference, not frozen copy.** The inheritance edge points at A's
  *current* dependency set. If A is later amended (a GL-VIZ-002 Verdict 2), B's
  inherited context updates with it — or B is flagged for re-verification. A
  frozen inherited copy that silently diverges from A is forbidden (the
  schema-level form of the "beautiful lie"; cf. GL-MED-002 M-1 refinement date).

---

## 3 · Discovery → Verdict → Schema → Edge (one continuous rite)

Adding B is not a separate manual step; it is the natural consequence of the
Shape Verdict (GL-VIZ-002):

    new SHAPE pattern appears on A
      → discovered shape does not match A's template
      → VERDICT rite classifies it:
          Verdict 1 (new sub-shape)   → mint new template B  → mint edge B→A
          Verdict 3 (hidden dependency) → complete A's definition → mint edge
          Verdict 2 (metric defect)   → amend A in place (Case β), no new schema
      → minting the schema ALSO mints the inheritance edge (§2), atomically.

Discovery, verdict, new schema, and inheritance edge are one rite — the edge is
never forgotten because it is minted in the same sealed transaction as the
schema it links.

---

## 4 · Guarantees (what never breaks)

- **G-1 · Query stability.** No pre-existing HeptaScript query changes behavior
  from additive growth. New attributes are opt-in: a query sees them only if it
  names them.
- **G-2 · Template stability.** No pre-existing template breaks; B is a new
  template, A's template is unchanged (unless a deliberate Verdict 2 amendment,
  which is versioned + scarred).
- **G-3 · Dependency integrity.** Every new chunk belongs somewhere: the
  inheritance edge (L-1) guarantees no orphan schemas.
- **G-4 · Reversibility of knowledge.** Because nothing is destroyed, the state
  of the shape at any past date is reconstructable (Point-in-Time Totality) —
  the schema remembers every stage of its growth (StoryEngine scar).

---

## 5 · Codex Compliance & Placement
- **A-1 zero new mathematics**: composes EAV model (GL-DDB-002), CrossTribe/NUZI
  edges, GL-VIZ-002 verdict, Point-in-Time Totality, append-only sealing. New is
  the *growth discipline* (accretion + inheritance edge + live reference).
- **A-4 members cited**: GL-DDB-002 · GL-DDB-003 · GL-VIZ-002 · GL-MED-002 ·
  NUZI · Point-in-Time Totality.
- **PB**: PB-349 `additive-growth` — mints new-schema + inheritance edge
  atomically; enforces L-1 live-reference; runs G-1..G-4 regression checks.

## 6 · Open seals for CSR-08
GL-DDB-004 adoption · sovereign name for the inheritance-edge relation
(candidate: **rēdû**, Akk. "to accompany / lead along") · whether L-1
re-verification on A-amendment is automatic or Steward-gated · PB-349 numbering.

*Recorded in the reign of Gudea 1.0, Phase Two. Nothing herein is sealed until
DUB.SAR confirms under CSR-08.*
