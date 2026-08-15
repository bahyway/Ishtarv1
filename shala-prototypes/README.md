# Šala Prototypes — Rehearsal Reference Only

**These are not the application.** Per the Šala Design Charter (below) and
Way-of-Work rule 5: no HTML in production, ever. This directory holds 122
single-file HTML rehearsals built during Phase 2 design work — each one
rehearses an engine concept visually, in the browser, before it is sealed
in pure Rust and built for real in the Godot stage (DubSar Theater) or as
a `workspace/bahyway_v4` crate.

**This is entirely separate from `shakkanakku-web`** (the real Rust web
server, `workspace/bahyway_v4/crates/shakkanakku`, launched by
`playbooks/playbook_284_launch_shala_dashboard.yml`). Nothing here is
wired into that binary, and nothing here should be, without a deliberate
decision to promote a rehearsed concept into real code.

- `SHALA-DESIGN-CHARTER.md` / `shala_charter.css` — the standing visual law
  for all Šala rehearsal work (locked charcoal/gold palette, tribe-colour
  semantics, HeptaScript-only copy, offline-sovereign: no CDNs, no webfonts).
- `INDEX.md` — what every prototype rehearses, grouped by batch/theme,
  with its DRAFT/sealed status where known.
- One subdirectory per source batch (`batch1_...` through `batch7_...`),
  preserving provenance back to `docs/phase2-incoming/`.

To view any prototype, open its `.html` file directly in a browser — they
are self-contained, no build step, no server required.
