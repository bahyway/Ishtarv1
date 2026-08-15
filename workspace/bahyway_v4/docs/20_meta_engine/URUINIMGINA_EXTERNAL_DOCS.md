# Running Uruinimgina Document Intake From Outside the Official EnkiDB/docs Repo

## The short answer

**This is already the pipeline's default mode, not a special case.** Uruinimgina
(`crates/anu-governor/src/docpulse.rs`, wired into Shakkanakku's own "Uruinimgina"
GUI tab, `crates/anu-governor/src/app.rs::show_uruinimgina_tab`) takes a `repo`
field the app's own hint text sets to `~/Forge/bahyway_v4_docs` — a personal,
external, non-`bahyway/EnkiDB` repository. Landing a promoted document into the
official repo is a **separate, opt-in, sixth stage** ("official repo landing")
that is skipped entirely whenever its `official repo` field is left empty. You
do not need the official repo at all to mint documents into EnkiDDB.

## The two repos Uruinimgina ever touches, and how they differ

| Field | What it is | Required? |
|---|---|---|
| `repo` (`DocPulseCfg::repo_path`) | ANY local git repo holding the documents you're promoting — can be a scratch repo you made just to hold "All PBs Roadmap"/the Manual/the Glossary, can be your Fedora W44 authoring tree, anything. **This is where the real KAKI-minting reads from.** | Required — the "▶ Reform & Pulse" button stays disabled until this is non-empty. |
| `official repo` (`DocPulseCfg::official_repo_path`) | A separate local clone of `bahyway/EnkiDB` specifically. Only used for the OPTIONAL final "promote a copy into the shared repo" stage. | Optional — empty string (the field's own hint text: "empty = skip landing stage") means Uruinimgina never looks at or touches the official repo at all. |

These are asserted to be different repos in the code itself: `official_repo_path`'s
own doc comment says the `repo_path` above "is NEVER assumed to be that repo."

## Step by step: promote external documents into EnkiDDB, without the official repo

1. **Create (or reuse) any local git repo** to hold the documents — it does not
   need to be `bahyway/EnkiDB`, does not need to be on GitHub at all if you never
   configure a remote push, and can be a folder you made specifically for this:
   ```
   mkdir -p ~/Forge/bahyway_v4_docs && cd ~/Forge/bahyway_v4_docs
   git init
   git checkout -b devVM
   git commit --allow-empty -m "seed"
   git checkout -b main
   git checkout devVM
   ```
   Two real, hard requirements in the code, not stylistic suggestions:
   - **The repo must be checked out on a branch literally named `devVM`** when
     Uruinimgina runs — `run()`'s very first git check does
     `rev-parse --abbrev-ref HEAD` and aborts immediately (`WrongBranch`) if it
     isn't. Any other branch name, including `main` or `master`, fails closed.
   - **A `main` branch must already exist** in that same repo, because stage 3
     ("Pulse devVM → main") does `push origin devVM` → `checkout main` →
     `pull origin main` → `merge devVM --no-edit` → `push origin main` →
     `checkout devVM` — real git operations, not simulated. If you don't want
     anything pushed anywhere yet, point `origin` at a throwaway bare repo, or
     expect stage 3 to fail at `push origin devVM` and use the halt-and-Retry/
     Abort control the GUI already gives you (every stage after Reform halts
     on failure and waits for the Architect, per CSR-08 — it never silently
     presses on).

2. **Drop your documents into that repo** and commit them normally (or let
   Uruinimgina's own Commit stage do it — an empty/clean working tree at
   pulse time is a logged warning, not a failure, so you can pre-commit or
   let the tool commit for you).

3. **Open Shakkanakku's Uruinimgina tab** and fill in the toolbar fields:
   - `repo` → the path from step 1 (e.g. `~/Forge/bahyway_v4_docs`)
   - `message` → the commit message for this pulse
   - `archive` → a quarantine directory OUTSIDE that repo (a hard guard: the
     run aborts with `ArchiveInsideRepo` if `archive` is inside `repo`)
   - `EnkiDDB root` → where the real Tigris generation materializes locally
     (`DocPulseCfg::enkiddb_output_root` — this is the same
     `enkiddb::materialize_version` this session's own
     `preparing_bare_metal_pbs_run.rs` test calls directly)
   - **Leave `official repo` empty.** That alone skips stage 6 entirely — you
     never need to touch `bahyway/EnkiDB` for this.
   - Click **"▶ Reform & Pulse"**.

## Which EnkiDDB tribe a pulse mints into (added 2026-08-01)

`DocPulseCfg::docs_tribe_id` controls this — `enkiddb::DOCS_TRIBE_ID` by
default, the sealed BahyWay.Ecosystem v4.0 documentation corpus. If the
source repo is a genuinely different corpus — e.g. the Architect's own
personal daily-work repo (ideas, discussions, drafts, alongside complete
write-ups, never the same repo as `bahyway/EnkiDB`) — mint it under
`enkiddb::ARCHITECT_DOCS_TRIBE_ID` instead, so it stays real, queryable
EnkiDDB content (`WHERE tribe=...` in HeptaScript, or any `RagIndex`
search) without ever mixing with the sealed corpus.

Via `uruinimgina-cli`'s TOML config (`bin/uruinimgina_cli.rs`), set:
```toml
docs_tribe = "architect_docs"   # or "docs" (default) -- any other value is a hard error
```
The GUI's Uruinimgina tab does not expose this yet (still hardcoded to
`"docs"`) — use `uruinimgina-cli` for an `architect_docs` pulse.

4. **What actually happens, stage by stage** (`docpulse::STAGE_NAMES`):
   1. *Reform (quarantine)* — fences `.gitignore` (`target/`, `node_modules/`,
      etc.), moves any file over `limit MB` (default 90) OUT of the repo into
      `archive`, with a `MANIFEST.txt` (nothing is ever deleted, ADR-006).
   2. *Commit* — commits whatever's dirty with your `message`.
   3. *Blob audit* — scans outgoing history for any object over 95 MB; a
      HALT here (Architect decides Retry/Abort), never silently skipped —
      per CSR-08, this is a governance gate, not a warning.
   4. *Pulse devVM→main* — the git dance described in step 1 above.
   5. *EnkiDDB mint + manifest* — **this is the real mint.** Every changed
      `.md`/`.akk`/`.svg`/`.png`/`.toml`/`.hepta` file in the commit gets a
      real Identity-Kaki via `enkiddb::WriteNode::ingest_document_from_path`
      (same call `enkiddb-ingest`'s bulk CLI uses), journaled, and
      materialized into a real Tigris generation under `EnkiDDB root` via
      `enkiddb::materialize_version` — plus a durable JSONL audit copy and a
      `doc_kaki_registry.jsonl` that makes a document promoted again under
      the same path a real `supersede_document` (ADR-014 Decision 2), not an
      orphaned duplicate mint.
   6. *Official repo landing (gated, opt-in)* — a no-op, logged as "not
      configured — skipped", whenever `official repo` is empty. This is the
      ONLY stage that ever touches `bahyway/EnkiDB`.

5. **Result:** your external documents are now real, KAKI-sealed EnkiDDB
   particles, queryable via `enkiddb::rag::RagIndex` or any Read Node opened
   against `EnkiDDB root`, entirely without ever creating a commit, branch, or
   PR against `bahyway/EnkiDB`.

## If you DO eventually want a copy landed in the official repo

Fill in `official repo` (a separate local clone of `bahyway/EnkiDB`), `subdir`
(e.g. `docs/bahyway-v4`), and `branch` — the code refuses `main`/`master` here
by design (`UnsafeTargetBranch`): landing always happens on a named review
branch, and a human-reviewed PR off that branch is the only path into the
official repo's trunk. The `push` checkbox is off by default; leaving it off
stages + commits locally on that branch and stops, so nothing reaches the
shared repo until the Architect explicitly turns it on for that run.

## Applying this to the 3 documents already ingested this session

`docs/20_meta_engine/ALL_PBS_ROADMAP.md`, `docs/20_meta_engine/
BAHYWAY_ECOSYSTEM_MANUAL_V4.md`, and `docs/00_codex/BAHYWAY_ECOSYSTEM_V4_GLOSSARY.md`
were minted directly via `WriteNode::ingest_document_categorized` in
`crates/enkiddb/tests/preparing_bare_metal_pbs_run.rs` (a real, passing test,
not a simulation) because they already live inside this checkout of
`bahyway/EnkiDB` — Uruinimgina's full git-pulse ceremony (branches, remotes,
blob audit) isn't needed when the source documents are already in the repo
being minted from. The steps above are for the case this task actually asked
about: documents that live somewhere else entirely.

## When the blob audit keeps HALTing on the same oversized blobs

Stage 3 (blob audit) checks the OUTGOING git history, not just the
current working tree — quarantining a large file (stage 1) stops it
from being committed *again*, but if it's already sitting in a PRIOR
commit in that repo's history, the audit finds it there every single
pulse, forcing the same Retry/Abort decision every time. Retry
overrides the halt for that one run; it does not remove the blob from
history, so the next pulse hits the identical halt again.

The real fix is removing the blob from history once, with
[`git filter-repo`](https://github.com/newren/git-filter-repo) (the
tool git itself recommends over the older `filter-branch`/BFG for this).
Run this **in the external docs repo itself** (e.g. `~/Forge/
bahyway_v4`), not in `bahyway/EnkiDB`:

```bash
# From inside the external docs repo (e.g. ~/Forge/bahyway_v4):
# 1. See exactly what's oversized and where, before removing anything
git rev-list --objects --all \
  | git cat-file --batch-check='%(objecttype) %(objectname) %(objectsize) %(rest)' \
  | awk '$3 > 90*1024*1024 {print}'

# 2. Strip specific paths from ALL history (adjust globs to match what
#    stage 3 actually reported -- e.g. the Godot binary and build
#    artifacts under a target/ or deps/ directory)
git filter-repo --path-glob '*Godot_v4*.x86_64' --invert-paths
git filter-repo --path-glob '*/target/debug/deps/*' --invert-paths

# 3. filter-repo already expires the reflog and runs gc -- verify the
#    blobs are actually gone before trusting it
git rev-list --objects --all \
  | git cat-file --batch-check='%(objecttype) %(objectname) %(objectsize) %(rest)' \
  | awk '$3 > 90*1024*1024 {print}'
# should print nothing now
```

`git filter-repo` rewrites every commit that touched those paths (new
hashes for all of them) — safe for a personal/authoring repo like this
one where you're the only committer, but never do this on a shared repo
with other collaborators' clones already based on the old history
(that's exactly why `bahyway/EnkiDB` itself uses ordinary PRs, never
history rewrites). After this, the blob audit should pass clean on the
next pulse without needing a Retry override.
