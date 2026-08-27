# ADR-024 — Seven Types Internal RBAC: Host Groups, VM Access, Per-Type Rules (DRAFT, unsealed)

**Status:** DRAFT — awaiting Architect ratification (CSR-08). Not a sealed
decision — correct or replace freely rather than treating this as settled.
**Date:** 2026-08-21
**Author:** Claude (drafting on the Architect's instruction), NOT the
Architect — same convention as ADR-022/023's own DRAFT status.

## The question, verbatim

*"We need to consider creating users groups on Fedora Workstation44 Host,
to be used with authorities levels on the connecting to VMs and from the
2 Nodes VMs to each 7 Podman containers (for 7Types EnkiDB Databases),
having in mind that some Databases can be considered as 'Read-Only' for
almost all stakeholders except the Architect..."* — followed by a real,
specific per-Type access spec (below).

## What already exists — this is not starting from zero

1. **The 5 host groups already exist as a playbook.** `playbook_268_
   bahyway_host_privilege_groups.yml` creates `bahyway-architect` (7),
   `bahyway-datasteward` (6), `bahyway-administrator` (5),
   `bahyway-developer` (3), `bahyway-stakeholder` (1) on the bare-metal
   host — idempotent, real `ansible.builtin.group` tasks, confirmed
   working. It deliberately stops there: no real Fedora user account is
   assigned to any group (a personnel decision, CSR-08, the Architect's
   to make per-person), and it deliberately does NOT touch any Sargon/
   Gilgamesh passport — **group membership and a cryptographic passport
   are two separate identity signals by design**, both readable
   independently by AnuGovernor's run-confirmation registry so a
   mismatch between them is a visible fact, not silently merged away.
2. **The privilege scale is already real and used elsewhere.**
   `kupru::IshtarLayer` (`crates/kupru/src/passport.rs`) already defines
   `privilege_level: 7` for an Architect (Gilgamesh-minted) passport and
   `privilege_level: 1` for a gardener (Sargon-minted) passport. PB-268's
   5 groups map onto that same 1-7 scale, not a new one.
3. **What is still completely missing** (confirmed live this session,
   reading the actual server code): the EnkiDDB/EnkiMDB write/read
   servers' wire protocol (`QUERY`/`SEARCH`/`FLUSH`/ingest) has **zero
   authentication of any kind**. DubSar Theater's Connector Wizard now
   checks `SessionIdentity.privilege_level` before opening (this
   session's own earlier fix) — but that check lives entirely inside the
   Godot client process. Anyone who can reach the TCP port at all —
   any script, any other machine on that network segment, not just
   Theater — can `FLUSH` or ingest with no passport check whatsoever.
   Host-group membership and passport privilege_level currently answer
   "who is allowed," but nothing on the server side ever asks the
   question.

This ADR is about closing that gap, and formalizing the per-Type rules
the Architect just specified, precisely.

## The per-Type access matrix, as specified

| Type | Direct human/stakeholder access | Who, if direct | Real path for everyone else |
|---|---|---|---|
| **EnkiMDB** | Read-only | All roles may read; only Architect (7) may write | — |
| **EnkiDDB** | Not direct at all | — | Write only through the HeptaScript Notebook and/or the BeeMDM ETL Pipeline. A future Confluence-like interface will let stakeholders work with their own EnkiDDB documentation (named as planned, not built — separate, larger project, out of scope here) |
| **EnkiQDB** | Direct, gated | Architect (7), DataSteward (6), Administrator (5) only | — |
| **EnkiODB** | Direct, three named paths | Any role, via a user GUI | Or via Streaming Endpoints, or via the BeeMDM ETL Pipeline |
| **EnkiSDB** | Never direct, by anyone | — | Only through BahyWay.Ecosystem internal services/processes (e.g. the BeeMDM ETL Pipeline itself) |
| EnkiDW, EnkiDB core | **Not yet specified** | — | Open question below |

## Two enforcement layers this ADR proposes — neither built yet

### Layer A — host → VM (SSH)

Today, any Fedora account with SSH keys distributed to `uruk-node-write`/
`read`/`vault` can reach all three equally; PB-268's groups exist but
nothing at the SSH layer reads them yet. Two honest options, not decided
here:

1. **Group-gated SSH** — `sshd_config`'s `Match Group` blocks + per-group
   `AuthorizedKeysFile` on each VM, so e.g. `bahyway-stakeholder` cannot
   SSH to any VM at all (their access, if any, is only ever through a
   GUI/HTTP layer — Rimush, DubSar Theater, a future Connector Wizard
   endpoint — never a raw shell).
2. **Uniform SSH, gated higher up** — every BahyWay account with a key
   can reach all three VMs at the transport layer (matching PB-268's own
   "these are separate identity signals" philosophy: SSH proves *a*
   trusted operator connected, not *which* role), and all real
   enforcement happens at the wire-protocol layer (Layer B) and at the
   Godot client layer (already partly done for the Connector Wizard).

Option 1 is the stronger guarantee (a compromised stakeholder key can't
even reach a shell to try). Option 2 is simpler to operate and matches
the existing philosophy more closely. **Not decided here.**

### Layer B — VM → each of the 7 Podman containers (wire protocol)

The real, missing piece from last turn's finding: an `AUTH <token>`
frame, sent once per TCP connection, checked server-side before any
`QUERY`/`SEARCH`/`FLUSH`/ingest is honored. Concretely:

- The token would be a short-lived, signed artifact minted from an
  already-verified `SargonPassport`/Gilgamesh passport (the seal
  verification already exists in `kupru`; only the "turn a verified
  passport into a token the wire protocol can carry" step is new).
- Each server checks the token's `privilege_level` against **that
  Type's own rule** from the matrix above — not one global threshold,
  since EnkiMDB/EnkiQDB/EnkiSDB each need a different rule.
- **EnkiSDB is a special case, and probably shouldn't be a token check
  at all.** "Only through BahyWay.Ecosystem internal services" reads as
  a stronger guarantee than "checks a token that identifies you as an
  internal service" — a token can be copied; a port that is simply never
  reachable from outside `localhost`/the write node's own process
  cannot be reached at all. Recommend: EnkiSDB's write/read servers bind
  `127.0.0.1` only (or a firewall rule scoped to the write node itself),
  the same `BIND_ADDR` pattern `enkiddb-read-server`/`rimush-server`
  already use for a different reason — not an app-level token gate.
  **Recommendation, not decided here.**
- EnkiDDB's rule ("writable only through HeptaScript Notebook and/or
  BeeMDM ETL Pipeline") is really "no direct human `FLUSH`/ingest at
  all, regardless of privilege_level" — closer to EnkiSDB's shape than
  to EnkiMDB/EnkiQDB's role-threshold shape. Concretely this likely
  means: `FLUSH`/ingest commands are only accepted from a request that
  also carries a "coming from the Notebook/ETL pipeline" marker, not
  from an arbitrary passport-holding human client at all — a second,
  narrower kind of gate than "privilege_level >= N."

## Scope question this ADR does not answer

Only EnkiDDB and EnkiMDB have real write/read server binaries with
content today (per PB-259's own header: the other 5 have containers only,
no automated real-content path yet). Should Layer B's `AUTH` frame be
designed and built against EnkiDDB/EnkiMDB first (where it can actually
be exercised against real data), with the design generic enough to
retrofit onto the other 5 as their servers get built — or should the
protocol be finalized once, against all 7's eventual shape, before
touching any of them? **Not decided here.**

## Open questions for the Architect (not decided here)

1. Layer A: group-gated SSH (Option 1) or uniform SSH with all
   enforcement higher up (Option 2)?
2. EnkiSDB/EnkiDDB: network-level exclusion (bind/firewall) as
   recommended above, or should there still be an app-level token check
   as defense in depth?
3. EnkiDW and EnkiDB core: what access rule, if any — do they need one
   at all yet, given neither has a real content-ingestion path today?
4. Scope/sequencing: EnkiDDB/EnkiMDB first, or design once for all 7?
5. Does the planned Confluence-like stakeholder interface for EnkiDDB
   documentation belong in this ADR's scope at all, or is it a fully
   separate project this ADR should only reference, not design?

## Status

Unsealed. The per-Type matrix above is written as precisely as the
Architect's own words allow; the two enforcement-layer questions and the
5 open questions are genuinely undecided and need a direct instruction,
not a guess, before any of this is built against live infrastructure.
