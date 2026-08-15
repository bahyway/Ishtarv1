# GL-DOC-001 — Single Glossary Law
**Status:** SEALED (concept). Registry discipline applies now.
**Scope:** every named thing in the ecosystem — engines,
structures, eras, laws, patterns, calculi, agents, artifacts.

## The Problem It Solves
The pantheon grew faster than the code. For a solo Architect,
every named thing is a lookup cost, and the risk is not
running out of names but holding too many in memory. The cure
is NOT a cleverer naming scheme (version-numbered names add a
second axis and make the load worse) — it is ONE PLACE to
look any name up.

## Clause 1 — One Flat Glossary
There is a single canonical glossary at
CAT-001-glossary.md. Every named thing has exactly ONE line:
    <name> → <role> → <governing law/PB> → <status>
Example:
    Nabu → knowledge-graph engine → GL-NAV-001 → sealed-concept
    EnkiDW → warehouse projection (port 7005) → GL-DDB-001 → sealed-concept
    Shakkanakku → automation/report governor → (report law) → sealed-concept
One line, four fields, no prose. The glossary is a lookup
table, not a document to read.

## Clause 2 — Names Stay Single and Stable
A named thing keeps ONE stable name for its whole life.
Version is NEVER bolted onto the name (no NabuEngine.v3).
The version axis already exists twice and is reused:
- ERAS (kings, NL-001) carry release version — "NabuEngine
  as it stands in the Uruinimagina era."
- PLAYBOOKS (PB-nnn) carry change sequence and lineage.
Adding a third version axis to the name is forbidden as
duplicative and load-increasing.

## Clause 3 — Register Before First Use
A named thing is added to the glossary the moment it is
named — before it appears in prose, a prototype, or code.
An unregistered name is not yet real. Status field tracks
its life: proposed → sealed-concept → implemented → retired.
Retired names keep their line (status: retired); names are
never reused (mirrors the GX registry rule, PB-195).

## Clause 4 — The Ruling Ledger Lives Here Too
Unratified names (the "ruling ledger") are glossary lines
with status: proposed. Ratification flips the status; it does
not create a new line. The ledger and the glossary are the
same table at different statuses — one place, one lookup.

## Clause 5 — The Moratorium Clause
This law is the naming moratorium made concrete: prefer
reducing the count of named things to formatting them better.
No new named engine is registered while an existing one
remains status: proposed or sealed-concept without progress —
a name earns its keep by being USED (implemented), not by
being coined. The glossary makes the backlog visible so the
Architect can see when naming has outrun building.

## Authority Note
Foundational documentation law. Complements GL-STY-001
(sedimentation), the GX registry (PB-195), NL-001 (naming
orthography and eras). CSR-08; the 1-billion-particles law.

— Inscribed for DUB.SAR 𒁾, BahyWay.Ecosystem v4.0.
