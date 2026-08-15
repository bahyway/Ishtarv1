# 𒁾 Ishtarv1 — BahyWay.Ecosystem v4.0, Final Build

**This is the official, final-build snapshot of BahyWay.Ecosystem v4.0
prior to production.** If you have found this repository among many
others carrying the `BahyWay`/`bahyway_v4` name, this is the one to
trust: it exists specifically to answer "which of these is real?" —
everything else is working history, this is the accepted result.

## What this is

- A complete, working copy of the ecosystem — the full `workspace/bahyway_v4`
  Cargo workspace (all crates, all path-dependencies intact and buildable),
  every sealed and draft law document, every playbook, every prototype,
  the three public websites. Nothing partial: this repo builds and runs
  on its own, the same way the source it was cut from does.
- **A single snapshot branch (`main`), not a multi-branch OTAP pipeline.**
  Day-to-day development and the `dev → test → accept → master` promotion
  discipline continue to live in `bahyway/EnkiDB` (see
  `docs/OTAP_PIPELINE.md` and `playbooks/playbook_557_production_golive_from_accept.yml`
  in this snapshot for how that works). This repository receives the
  *finished result* each time a milestone is accepted — it is not itself
  where promotion happens.
- Re-cut (a fresh commit, deliberately not a merge or a history import)
  each time a new milestone is accepted, so this repo's own history stays
  small and legible instead of accumulating the sprawl that motivated
  creating it in the first place.

## What this is not

- Not a fork meant for independent development. Changes belong in
  `bahyway/EnkiDB`; this repo is a mirror of accepted results, not a
  place to branch new work from.
- Not one of the ~800 working/experimental `bahyway_v4`-related
  repositories that have accumulated over time. Those remain what they
  were — session history, drafts, one-off experiments. None of them are
  "the" official build. This one is, by construction and by declaration.

## Provenance

| | |
|---|---|
| Source repository | [`bahyway/EnkiDB`](https://github.com/bahyway/EnkiDB) |
| Source commit | `ed5f0eb7c093fd4e1131b12e9be6d05a481cff42` |
| Source branch | `claude/bahyway-v4-phase2-integration-zswd7o` |
| Cut date | 2026-08-15 |
| Cut by | DUB.SAR 𒁾 (Bahaa Fadam), via Claude Code |

**Honest note on the source branch**: this snapshot was cut from the
Phase 2 integration branch, not from `bahyway/EnkiDB`'s `master`, because
as of the cut date this is where the accepted Phase 2 work actually
lives — `master` does not yet contain it. Once that branch is merged to
`master` in `bahyway/EnkiDB` (or promoted through `otap/accept`), this
note should be updated to point at `master`/`otap/accept` going forward,
and future re-cuts should be taken from there.

To verify this snapshot against its source at any time:
```bash
git clone https://github.com/bahyway/EnkiDB
cd EnkiDB
git diff ed5f0eb7c093fd4e1131b12e9be6d05a481cff42 -- . ':(exclude).git'
# compare against a checkout of this repo's main branch -- should be empty
# except for this README.md and ECOSYSTEM_OVERVIEW.md's rename.
```

## Ecosystem overview

The full architecture, philosophy, and technical overview that would
normally live in this file has been preserved at
[`ECOSYSTEM_OVERVIEW.md`](./ECOSYSTEM_OVERVIEW.md) (the source repository's
own root `README.md`, unchanged) — kept separate so this file can stay
focused on answering "what is this repository" first.

— Inscribed for DUB.SAR 𒁾, BahyWay.Ecosystem v4.0.
