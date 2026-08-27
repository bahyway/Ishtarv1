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
  `docs/08_pipeline_alaktu/OTAP_PIPELINE.md` and
  `playbooks/playbook_557_production_golive_from_accept.yml` in this
  snapshot for how that works). This repository receives the *finished
  result* each time a milestone is accepted — it is not itself where
  promotion happens.
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
| Source commit | `4d1a07f5fa23833c633b0f25aa0033647cef7bbe` |
| Source branch | `master` |
| Source tag | [`gudea-v4.0-sealed`](https://github.com/bahyway/EnkiDB/releases/tag/gudea-v4.0-sealed) |
| Cut date | 2026-08-27 |
| Cut by | DUB.SAR 𒁾 (Bahaa Fadam), via Claude Code |
| Previous cut | `69e5deae478418085922344be0efb8921f2a0e7e` (2026-08-15) |

**What changed since the previous cut:** the build phase of BahyWay.Ecosystem
v4.0 closed for real. The Palû Crossing (law `GL-PAL-001`, coordinator
`IsimudEngine`) sealed the **Gudea** reign (v4.0) at `2026-08-27T12:37:44Z`
across all seven sovereign databases and opened the **Zagesi** reign (v4.1)
for new work — proof in `palu/COMPLETION-STELE-4.0.tsv` and
`palu/PALU-STELE-4.0-to-4.1.md`, both carried in this cut. Since the previous
cut: `enkidb-con-engine` gained **CSR-08 Architect Sovereignty**, the eighth
Connection Security Rule, append-only-honest by design (`Create`/`Supersede`/
`Retire`, never a literal modify/delete — see `docs/18_security/CONENGINE_CSR.md`);
`enkidb-ingest::kispu` landed the four-way atomic commit binding the Event
KAKI, the NATIRU orbital-range index, the zakāru audit journal, and the
Orbital position into one all-or-nothing write; `pdm-shape-admission` shipped
as GL-EAV-001 Layer 2's first real implementation; 17 DRAFT law tablets
(`GL-NSR-001`, `GL-LBR-001`, `GL-NJF-001`, `GL-DST-004`, `GL-VSL-001`,
`GL-SHP-001`, `GL-ISM-001`, `HS-EXT-003`, and nine more) were sealed with
their matching crates built and tested; the E-004/E-005 performance gates
were closed for real against the production Read Node path; and two real
bugs found live on bare-metal hardware (`playbook_687`'s stale-store reset,
`playbook_688`'s SELinux/traversal-permission chain) were diagnosed and
fixed. The workspace grew from 79 to **261 crates**. Full workspace
`cargo check --workspace` verified clean before this cut (1m31s, exit 0);
a full `cargo test --workspace` run across all 261 crates was not re-run
for this specific cut — the individual crates landed this era each carry
their own passing `cargo test -p <crate>` counts documented in their own
tablets and `docs/18_security/CONENGINE_CSR.md`.

**Honest note on the source branch**: this cut is taken from `master`
directly — unlike the previous cut (taken from a Phase 2 integration branch
because the accepted work hadn't reached `master` yet at that time), the
Gudea-era build phase is sealed on `master` itself, so this and future
re-cuts should keep tracking `master` going forward.

To verify this snapshot against its source at any time:
```bash
git clone https://github.com/bahyway/EnkiDB
cd EnkiDB
git diff 4d1a07f5fa23833c633b0f25aa0033647cef7bbe -- . ':(exclude).git'
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
