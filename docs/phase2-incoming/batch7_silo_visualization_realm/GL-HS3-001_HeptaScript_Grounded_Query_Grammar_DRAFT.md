# GL-HS3-001 (candidate) — HEPTASCRIPT PHASE 3 · THE GROUNDED QUERY GRAMMAR
## ASK / PROVE / GHOST / WITNESS · The Four-Outcome Honesty Contract
### BahyWay.Ecosystem v4.0 · Phase Three · binds GL-VIZ-001/002 · GL-DDB-002/004 · Status: DRAFT — pending CSR-08 sealing by DUB.SAR 𒁾

---

## 0 · Principle

Phase Three gives HeptaScript the power to answer a stakeholder's question at
runtime **by checking the GOLDEN store, never by inventing.** The agent (the
Bārû, via the Têrtu Engine) proposes a connection; HeptaScript *proves or
disproves it against grounded data*; the visualization renders **only what the
store supports**. This is the market differentiator (per the 2026 KG landscape):
not a bigger biomedical graph, but a **grounded, runtime, honest** instrument
where the difference between a fact, a weak signal, a ghost, and nothing is drawn
as law.

Two stores, two roles (both GOLDEN-scored):
- **EnkiDB (OLTP)** — the transactional golden store; live records, point facts.
- **EnkiDW (OLAP)** — the analytical golden warehouse; aggregates, cohorts,
  cross-domain rollups. Runtime pattern queries run primarily here; EnkiDB
  grounds individual claims.

The honesty coupling (GL-VIZ-001 D-1) becomes a **runtime rule**: the agent
proposes; the GOLDEN store proves; HeptaScript is the language of the proving.

---

## 1 · The Query Verbs

HeptaScript remains Anti-SQL (no SELECT/JOIN/WHERE). Phase-3 adds four verbs:

- **ASK** — pose a candidate connection as a question.
  `ASK connection BETWEEN "Clostridium perfringens" AND "diabetes mellitus"`
- **PROVE** — test the candidate against GOLDEN; returns an outcome + evidence.
  `PROVE OVER EnkiDW SCOPE domain="infection" WITH evidence`
- **GHOST** — probe for sub-threshold / lightly-shaded signal (Eṭemmu surprisal).
  `GHOST threshold ε<0.30 SURPRISAL -log P(claim | corpus)`
- **WITNESS** — seal the result (fact/none/weak/ghost) as a queryable Event
  particle, so the answer itself becomes grounded, auditable history.
  `WITNESS result → StoryEngine`

Composed runtime reading:
```
ASK connection BETWEEN A AND B
  PROVE OVER EnkiDW SCOPE ... WITH evidence
  GHOST IF NOT PROVEN
  WITNESS result → StoryEngine
```

---

## 2 · The Four-Outcome Contract (the core; the part the market lacks)

Every ASK resolves to exactly one of four outcomes. Each has a required render
and a required honesty rule. The agent MAY NOT return anything else — no
"plausible-sounding" fifth option, no ungrounded assertion.

| Outcome | GOLDEN condition | Render (BIGRING) | Honesty rule |
|---|---|---|---|
| **FACT** | ≥1 GOLDEN record supports it, high confidence | solid GOLDEN edge, cited | must show the supporting record IDs |
| **NONE** | store searched, nothing found | **no edge** + explicit "no connection found in store" | absence stated, never hidden |
| **WEAK** | few / low-confidence records | thin faint edge, labelled weak + count | never dressed up as strong |
| **GHOST** | sub-threshold signal (Eṭemmu surprisal above noise but below fact) | dim dashed edge, "possible pattern — needs research" | must be labelled hypothesis, never fact |

**The Ghost rule (Eṭemmu, from EN-DDB-004):** a GHOST is computed as surprisal
`−log P(claim | sealed corpus)` — a whisper in the GOLDEN store that is present
but below confidence. It is rendered *as a ghost* (dashed, dim) and carries the
label "needs research." Promoting a ghost to a fact without new grounding is a
sealed violation.

**Accuracy guarantee (structural, not aspirational):** the instrument is
"near-100% not fake" **because the agent can only draw what PROVE finds in
GOLDEN.** Ground truth = the store. Gaps are shown as NONE or GHOST, never
filled with invention. This is why it can be trusted where a raw GNN (which
emits structurally-plausible-but-fake edges) cannot.

---

## 3 · The Runtime Flow (HeptaScript ⇄ Visualization, one loop)

    stakeholder question
      → Bārû forms ASK (candidate connection)
      → PROVE over EnkiDW (OLAP) / EnkiDB (OLTP), gather evidence + confidence
      → classify: FACT / NONE / WEAK / GHOST
      → BIGRING renders the outcome (solid / none / thin / dashed)
      → WITNESS seals the result as an Event particle (auditable)
      → stakeholder inspects evidence; may request GHOST → research queue

The BIGRING is **dynamic**: it draws itself per question, orbits reconfiguring
around what the store actually holds — not a fixed pre-baked map. The
visualization and the grammar are inseparable: the grammar's four outcomes ARE
the visualization's four render modes.

---

## 4 · Why Both, Never One Without the Other
- Grammar without visualization = an answer no stakeholder can see or trust.
- Visualization without the grammar = a pretty layout that can lie (the failure
  mode already rejected).
- Together = a **grounded reasoning instrument**: the question is HeptaScript,
  the truth is GOLDEN, the honest answer is the BIGRING.

## 5 · Codex Compliance & Placement
- **A-1 zero new mathematics**: composes GOLDEN scoring, Eṭemmu surprisal
  (EN-DDB-004), Graph-RAG retrieval (GL-DDB-002), D-1 coupling, BIGRING render.
  New is the *runtime grammar + four-outcome contract*.
- **A-4 members cited**: GL-VIZ-001/002 · GL-DDB-002/004 · EN-DDB-004 (Eṭemmu) ·
  EnkiDB/EnkiDW · Têrtu Engine (Bārû).
- **PB**: PB-360 `heptascript-ask-prove` engine; PB-361 `ghost-scorer` runtime;
  PB-362 `bigring-runtime-render` (four-outcome).

## 6 · Open seals for CSR-08
GL-HS3-001 adoption · the four verb names (ASK/PROVE/GHOST/WITNESS) as canonical
· the FACT/NONE/WEAK/GHOST thresholds · whether GHOST auto-files to a research
queue · Têrtu Engine name for the Bārû's engine · PB-360–362 numbering.

*Recorded in the reign of Gudea 1.0, Phase Three. Nothing herein is sealed until
DUB.SAR confirms under CSR-08.*
