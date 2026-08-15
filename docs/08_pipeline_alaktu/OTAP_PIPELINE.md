# EnkiDDB OTAP Pipeline

**Sealed:** 2026-07-12
**Scope:** the EnkiDDB (Tigris) rebuild — EAV documentation structure, HeptaScript
query language, the ENLIL 7-index stack (target: >1 billion particles, <1s
query latency), and the AI Agents that sit on top of it.

## Why branches, not a second repository

EnkiDDB is a crate (`crates/enkiddb`) inside the `bahyway/EnkiDB` workspace,
with path-dependencies on a dozen+ sibling crates (`enkidb-kaki`,
`enkidb-particles`, `enkidb-journal`, `enkidb-readnode`, `heptascript`,
`adapa-recall`, `bahyway-core`, `akkvalue`, `enkidb-ingest`, ...). A second
repository containing only EnkiDDB's own files would not build — Rust
path-dependencies need the actual sibling directories present, and this
ecosystem has no private crate registry to depend on instead (that would
also violate the sovereign/offline/pure-Rust rule the whole project runs
on). So the four OTAP stages are **branches of the one repository**, each
carrying the full workspace, not four partial repos that can drift apart.

## The four stages

| OTAP letter | Branch | What it represents | Where it runs |
|---|---|---|---|
| **O**ntwikkeling (Dev) | `otap/dev` | Active rebuild work lands here first. | Built/tested locally or on `eriduous-vdi`; never deployed. |
| **T**est | `otap/test` | Promoted once `cargo test --workspace` is green on `otap/dev`. | Built + full test suite run on `eriduous-vdi`. Still never deployed to the real CQRS VMs. |
| **A**cceptatie (Accept) | `otap/accept` | Promoted once Test passes AND a real smoke test has run — e.g. `enkiddb-ingest` against a real doc tree, materializing a Tigris generation, `RagIndex` search returning real hits. | Built on `eriduous-vdi`; smoke-tested against `enkidb-node-write`/`enkidb-node-read` in a non-serving capacity (no systemd unit started yet, no production traffic). |
| **P**roductie (Production) | `master` | The default branch. Promoted only on the Architect's explicit sign-off — and being on `master` is not itself "live," see the correction below. | Actually goes live via `playbook_557_production_golive_from_accept.yml`, which redeploys the real `enkiddb-write-server`/`enkiddb-read-server` rootless Podman containers (via `playbook_212`) on `enkidb-node-write` (write path) and `enkidb-node-read` (read path). |

`master` is not a fifth branch invented for this pipeline — it's the
existing default branch, already the ecosystem's production-equivalent
(per `scripts/consolidate-branches.sh`'s own convention of merging
validated `claude/*` work into it). OTAP's "P" stage is simply that same
branch, not a duplicate concept.

## Promotion mechanism

Promotion = `scripts/otap-promote.sh <from> <to>` (see that script for the
exact gates). In short:

- `otap/dev` → `otap/test` and `otap/test` → `otap/accept`: automatic once
  `cargo test --workspace` passes on the source branch — these are low-risk,
  reversible, nothing is deployed.
- `otap/accept` → `master`: the script refuses to run without an explicit
  `--i-understand-this-is-production` flag, and even then only fast-forwards
  (never force-pushes). **Correction, 2026-08-15**: this git-level promotion
  does *not* itself redeploy anything — `master` reaching a new commit and
  Production actually serving that commit are two different facts (branch
  position vs. runtime state). The script's own closing message names the
  missing step; `playbook_557_production_golive_from_accept.yml` is that
  step — a separate, explicitly-confirmed act that verifies HEAD matches
  `origin/master`, re-runs the test gate, then redeploys the real
  `enkiddb-write-server`/`enkiddb-read-server` Podman containers (via the
  proven `playbook_212`) and witnesses the go-live in
  `docs/16_runbooks/NARU_AUDIT_JOURNAL.md`. Run it deliberately, after promotion,
  not as an automatic consequence of the git merge.

**Branch reconciliation, 2026-08-15**: `otap/dev`/`test`/`accept` had gone
orphaned from `master` — a `git filter-repo` history rewrite (documented in
`playbook_281`) gave every commit on `master` a new hash after these
branches were created, leaving them with zero common ancestry
(`git merge-base` returned nothing) even though their content was the same
work under old hashes. `scripts/otap-promote.sh`'s fast-forward-only merge
cannot bridge disjoint histories, so all three were reset (force-pushed) to
match `master` exactly, re-anchoring them to real history. This was a
one-time repair, not a routine operation — ordinary promotions after this
point use the script normally.

Day-to-day rebuild work does **not** happen directly on `otap/dev` — this
session's actual development continues on its designated working branch
(currently `claude/iphone-playbooks-crosstribe-eval-nbc5sw`), and gets
promoted into `otap/dev` explicitly, same as any other promotion. This
keeps the working branch's own commit history intact and makes "what's
actually in the Dev stage right now" an unambiguous, separate question
from "what's the session currently iterating on."

## Definition of done for the scope this pipeline carries

Per the Architect's own framing, "Production-level, full working" means
all of the following hold on `master`, deployed and verified on the real
2-node CQRS VMs:

1. **EAV documentation structure** — the numbered `docs/` taxonomy
   (`00_codex` .. `20_meta_engine`, `99_index`) fully populated and
   ingested, `meta.collection` tags mirroring it 1:1 (landed: PB-179).
2. **HeptaScript query language** — full syntax coverage, documented
   (`docs/08_pipeline_alaktu/HEPTASCRIPT_QUERY_LANGUAGE.md`, still pending as of
   this pipeline's creation).
3. **ENLIL 7-index stack** — verified real index count and behavior
   (blocked as of this pipeline's creation on the Architect confirming the
   correct list of 7 indexes — `BTreeRange` is confirmed dead code, not
   one of the 7).
4. **>1 billion particles, <1s query** — a real, repeatable benchmark
   proving this on the actual index stack once (3) is resolved, not a
   projection.
5. **AI Agents** — the agents that query EnkiDDB, most concretely
   `enkidullm-codex` (the still-unbuilt bridge letting TamuzAI/EnkiduLLM
   query EnkiDDB/RagIndex), currently the single largest honestly-scoped
   gap in this list.

This is a multi-stage program, not a single promotion — stated here
plainly so "reaches Production" has a checkable meaning rather than being
a vague endpoint.
