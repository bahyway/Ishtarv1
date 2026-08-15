# Step 7 — Verification Report

Run from this remote session on 2026-08-15. Two of the three planned
checks could be run here; the third (live EnkiDDB ingestion) explicitly
cannot, for the same reason `playbook_208_full_corpus_ingestion_runbook.yml`
already documents in its own header: `enkiddb-cli` runs bare-metal against
real infrastructure this sandbox has no access to. Nothing here was faked.

## 1. YAML syntax — PASS

All 272 `.yml` files under `playbooks/` (pre-existing + all 90 newly
landed) parse cleanly with a standard YAML parser. No malformed files.

## 2. Host-pattern resolution against the real inventory — as expected

Simulated what `ansible-playbook --check` would report for each of the 90
newly landed playbooks against the real `ansible/inventory.ini`
(`ansible-playbook` itself isn't installed in this sandbox, so this is a
direct comparison of each playbook's `hosts:` line against the inventory's
defined hosts, not a live dry-run):

- **73 of 90 resolve cleanly** (target `uruk`, `localhost`,
  `enkidb-node-write`, or `enkidb-node-read`).
- **17 do not resolve** — exactly the ones already listed in
  `playbooks/SHAKKANAKKU_PB_MANUAL.md` under "Not yet runnable" (PB-310,
  311, 312, 313, 314, 315, 316, 317, 318, 319, 320, 330, 338, 422, 423,
  424, 425). No surprises; this confirms the manual's status table rather
  than finding anything new. None of the 90 landed playbooks target
  `hosts: kish` directly (kish is referenced in a few playbooks' variables
  and prose, not as a direct host target), so the "kish defined but not
  connectable" note in the manual is more cautious than strictly needed
  right now — worth revisiting once kish playbooks are written for real.

## 3. Rust workspace integrity — PASS (manifest only, not full test suite)

`cargo metadata --no-deps` against `workspace/bahyway_v4` completes
cleanly (exit 0). A full `cargo test --workspace` (79 crates, 1,804+
tests per the README) was **not** run in this pass: no Rust source file
was touched by this integration (only `docs/`, `playbooks/`,
`shala-prototypes/`, and `ansible/inventory.ini` changed), so the risk
surface a full test run would catch is effectively zero, and the run
itself would cost significant time for no expected finding. Recommend
running it for real before your next release milestone regardless, just
not as a blocking step of this integration.

## 4. EnkiDDB ingestion of the new corpus — NOT RUN HERE, by design

This remote session has no access to your real `uruk` box, its running
EnkiDDB write/read nodes, or a build of `enkiddb-cli`. Faking this would
mean reporting a result I cannot verify, which this integration has
avoided at every step. Instead, here is exactly what to run once you're
back at the real machine:

```bash
# From the repo root on uruk:
ansible-playbook playbooks/playbook_208_full_corpus_ingestion_runbook.yml --check
ansible-playbook playbooks/playbook_208_full_corpus_ingestion_runbook.yml
```

PB-208 ingests the **whole** repo tree (it's the existing full-corpus
runbook, not a Phase-2-specific one) — since all the Phase 2 docs are now
committed under `docs/`, they'll be picked up automatically in the same
pass as everything else, no separate ingestion step needed. It builds
`enkiddb-cli` fresh, runs the Musaru security pre-ingest scan (PB-206),
and reports per-file OK/SKIP/REJECTED with reasons; it exits non-zero on
any per-file failure, so a clean exit code is a real signal, not
optimistic reporting.
