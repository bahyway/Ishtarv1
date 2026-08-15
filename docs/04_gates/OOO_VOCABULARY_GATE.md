# OOO Vocabulary Gate

**Crate:** `crates/musaru-security` (`src/vocab_gate.rs`). **CLI:**
`bin/ooo-vocab-gate`. **Playbook:** `playbooks/playbook_225_ooo_vocabulary_gate.yml`.

Status: ✅ VERIFIED — every rule below is a real check in the source above,
each backed by a passing test, and the whole tool has been run twice
against real content: the live `godot/dubsar-theater/` tree (clean) and
an uploaded prototype that had adopted real foreign-database vocabulary
(caught, 15/15 real hits, zero false positives).

**Scope note, confirmed by actually running it against this file**:
`.md` is a gated extension (§2c), and this document's entire purpose is
to *list* the forbidden vocabulary — so scanning it (or any of the
`docs/14_decisions_adr/` ADRs that explain the same law) reports dozens
of "violations" that are really just this doc doing its job. That is
expected, not a bug: PB-225's default target is `godot/dubsar-theater/`
specifically to avoid this — reference documentation about the law is
never in scope by default, only UI/prototype surfaces that might
accidentally *adopt* the vocabulary rather than discuss it.

## 1. The law this enforces

Stated directly by the Architect: BahyWay's Seven EnkiDB Types are
**Triple-O (Orbit-Oriented Ontology)** — never SQL, never relational,
never a foreign database's product name or port, "because they ARE NOT
OOO and will NEVER BE." Before this tool existed, that law lived only as
a comment in `enki_engines.gd`'s header. This is the promotion of that
comment into something that actually runs and actually fails a build.

**Division of labor with the other SQL gate**: `heptascript::operations
::Operation::parse()` already rejects SQL keywords (`SELECT`, `INSERT`,
`JOIN`, ...) — but only when they appear as the leading verb of real,
*parsed* HeptaScript query text. It never sees a Godot button label, an
adopted HTML prototype, or a paragraph of documentation. This gate covers
exactly that other surface — UI copy, design prototypes, docs — and
deliberately does **not** duplicate SQL-verb checking (see §3).

## 2. What it checks

### 2a. Forbidden terms (`FORBIDDEN_TERMS`, case-insensitive substring match)

| Term | Why |
|---|---|
| `SQL` | the paradigm word itself |
| `NoSQL` | still a foreign-paradigm label, not BahyWay's own vocabulary |
| `Relational` | the relational model this ecosystem's EAV/Particle model deliberately isn't |
| `Multi-Model` | explicitly named as forbidden in `enki_engines.gd`'s own header, specifically because it was once used to describe EnkiMDB |
| `PostgreSQL`, `Postgres` | foreign product |
| `MySQL`, `MariaDB` | foreign product |
| `MongoDB` | foreign product |
| `Cassandra` | foreign product |
| `Redis` | foreign product |
| `SQLite` | foreign product |
| `Oracle Database` | foreign product |
| `SQL Server` | foreign product |
| `DynamoDB` | foreign product |
| `CouchDB` | foreign product |
| `Neo4j` | foreign product |
| `Elasticsearch` | foreign product |

### 2b. Forbidden ports (`FORBIDDEN_PORTS`, matched as a standalone digit run — see §4)

| Port | Product |
|---|---|
| 5432 | PostgreSQL |
| 3306 | MySQL/MariaDB |
| 27017 | MongoDB |
| 9042 | Cassandra |
| 6379 | Redis |
| 1433 | SQL Server |
| 1521 | Oracle |
| 5984 | CouchDB |
| 9200 | Elasticsearch |
| 7687 | Neo4j Bolt |

### 2c. Gated file extensions (`GATED_EXTENSIONS`)

`.gd`, `.tscn`, `.html`, `.htm`, `.md` — UI/design surfaces. Deliberately
excludes `.rs`: real Rust/HeptaScript source is `Operation::parse()`'s
job, not this scanner's (see §3 and §5).

## 3. What it deliberately does NOT check, and why

**Bare SQL verbs** (`SELECT`, `INSERT`, `DELETE`, `GROUP BY`, `ORDER BY`,
`JOIN`, ...) are **not** in `FORBIDDEN_TERMS`. A first version of this
gate included them — run against the real repo, every hit was ordinary
English or GDScript (`_select_database()`, `"select a node"`,
`database_selected`), not one real SQL violation. `Operation::parse()`
already owns SQL-verb rejection precisely, as an exact-token match
against real parsed HeptaScript source; substring-scanning prose for the
same words produces only noise. (Separately, and for an unrelated
reason — the Architect wanting *no* SQL-flavored word at all, not just
no false SQL clauses — every real "Select"/"Selected" occurrence in the
DubSar UI was renamed to "Choose"/"Chosen" by hand; see
`docs/00_codex/BAHYWAY_ECOSYSTEM_V4_GLOSSARY.md`'s session log or the git history
around 2026-07-22 for that pass.)

## 4. How a match is decided

- **Terms**: case-insensitive substring match against the line.
- **Ports**: matched only as a standalone digit run — `contains_port_token`
  checks the character immediately before and after the match isn't
  itself a digit. This is why a real HEPT port like `70012`
  (hypothetically) would never false-positive against `1521`'s trailing
  digits, and why `enki_engines.gd`'s own real ports (`7001`..`7005`,
  `7102`, `7202`) never collide with anything in the forbidden-port list.

## 5. Exemptions — law statements aren't violations of themselves

A line is skipped if it (or one of the **2 lines before it**, the
`EXEMPT_WINDOW`) contains one of these phrases (case-insensitive):

```
anti-sql, sqlforbidden, forbidden vocabulary, no use of sql,
sql is forbidden, never sql, not sql, not in sql, sql-free
```

**Why a window, not just the current line**: `enki_engines.gd`'s own
header wraps its law statement across two comment lines — *"Forbidden
vocabulary anywhere in this project: SQL, Relational, 5432,"* then, on
the next line, *"Multi-Model (for EnkiMDB), or any foreign database's
port/protocol."* The exempt phrase and the term it's exempting land on
different lines. A single-line-only check flagged the law statement
itself as a violation of the law it states — a real bug, caught by
actually running the gate, fixed by widening the check to a 2-line
lookback window (`exempt_window_does_not_suppress_a_real_violation_
several_lines_later` proves the window doesn't over-suppress a real hit
several lines later).

This exemption is deliberately coarse (a whole line/window, not a
sub-string carve-out): the threat model here is a wholesale mockup — a
whole card, row, or paragraph — carrying real foreign vocabulary, not
one word hidden inside an already-compliant sentence about forbidding it.

## 6. Running it

CLI directly:
```
cargo run -p ooo-vocab-gate -- <path> [path...]
```
Prints every violation as `file:line: forbidden vocabulary 'X' -- context`,
then a summary line and `GATE: PASS` or `GATE: FAIL`. Exits non-zero on
any violation.

Ansible (PB-225), scans the DubSar Godot tree by default:
```
ansible-playbook playbooks/playbook_225_ooo_vocabulary_gate.yml
```
Or a specific staged path (e.g. before adopting an uploaded prototype's
layout):
```
ansible-playbook playbooks/playbook_225_ooo_vocabulary_gate.yml \
  -e 'vocab_gate_targets=["/path/to/staged_prototype.html"]'
```

## 7. Real verification runs

- **The live DubSar tree**: `godot/dubsar-theater/` — 43 files, 0 dirty,
  0 violations. Zero false positives on real, legitimate content
  (checkbox/dropdown/tree API calls, real HEPT ports, etc.).
- **A real uploaded prototype**: labelled its database cards "Relational
  SQL Engine" (port 5432), "Object Document Store" (port 27017), "Multi-
  Model Database" (port 9042), and used 6379 (Redis) for a fifth card.
  The gate caught all 15 real violations, each with exact file:line and
  surrounding context — `SQL`, `Relational`, `Multi-Model`, and all four
  foreign ports, none missed, none spuriously duplicated beyond what the
  line genuinely contained.

## 8. See also

- `crates/musaru-security/src/zip_scan.rs` — the pre-existing sibling
  scanner this module's structure (byte/string pattern list, `scan`/
  `scan_all`, pure pattern matching, no third-party dependency) is
  modeled on.
- `crates/heptascript/src/operations.rs` — `Operation::parse()`'s
  `OperationError::SqlForbidden`, the query-*language*-level SQL gate
  this one is the non-redundant complement to.
- `docs/09_observatory/HEPTASCRIPT_GLOSSARY.md` — the Anti-SQL Law entry,
  the ecosystem-wide statement this gate is one concrete enforcement of.
