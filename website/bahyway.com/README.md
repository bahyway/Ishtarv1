# bahyway.com — site source

The umbrella home of the BahyWay.Ecosystem. Same architecture as beemdm.com:
markdown truth source in content/, thin static renderer, GitHub Pages deployable
(push repo → Settings → Pages → branch main → add CNAME file "www.bahyway.com").

Signature: the orbital system — seven shells, one golden core. Endgame: replace
the hero SVG with the WASM DubSar viewer (post-PB-160).

## 2026-07-28 correction pass

Four claims in the delivered `index.html` had drifted from what's actually
built or actually measured, and were fixed before landing:

1. **"No third-party dependencies"** — restated as "a deliberately minimal
   dependency footprint... infrastructure only, never imported architecture."
   Same fix as beemdm.com and docs/WHAT_IS_BAHYWAY.md — false on its face
   given this ecosystem runs a forked Godot engine, tokio, serde, z3.
2. **The EnkiDB port table was factually wrong**, not just simplified. The
   original tile listed EnkiSDB:7001, EnkiODB:7002, EnkiQDB:7003, EnkiDB:7004,
   EnkiDW:7005, EnkiMDB:7006 (and omitted EnkiDDB entirely). The real,
   verified mapping (`docs/components/ENKIDB_7_TYPES.md`) is EnkiDB:7001
   (Golden Store), EnkiDW:7002, EnkiSDB:7003, EnkiODB:7004, EnkiQDB:7005,
   EnkiMDB:7006, EnkiDDB:7007. Every port number in the original tile was
   attached to the wrong database. Rebuilt the SVG with the correct 7-row
   table and moved the "golden store" marker to the actual EnkiDB row.
3. **"ENLIL ring"** — ENLIL already names the ecosystem's Total Algebra
   Content (GeoLaw-05), a real, previously-found naming collision; the
   index stack itself was deliberately renamed to the **Anu Index Stack**.
   Fixed in the "Orbit truth" verb card.
4. **"→ 10⁹ particles < 1 s"** — this repo's own `docs/PB-221_SCALE_
   BENCHMARK_FINDINGS.md` (section 4, "What 'show it in <1s' honestly
   means at 100M–1B") explicitly states that returning a literal billion
   rows over a network in under a second is not achievable by any system
   — network/serialization physics, not a HeptaScript limitation — and
   was never what was actually measured. What *was* measured and is real:
   LIMIT-bounded queries hold flat latency (10–44ms) regardless of corpus
   size, confirmed at 1M and 10M particles, once via headless benchmark
   and once end-to-end over a real TCP connection. Replaced the bare
   billion-particle timing claim with that honest framing in the
   HeptaScript tile.
5. **DubSar Theater tile described the aspirational egui/WGPU rewrite**
   as current fact (tags "egui, WGPU"). The Theater that actually exists
   and runs today is Godot-hosted — GDScript scenes plus Rust GDExtension
   bridges, inside a from-source rebranded Godot 4.3 engine build
   (PB-262). Same fix as `docs/WHAT_IS_BAHYWAY.md`. Updated the tile's
   caption and tags; the egui/WGPU version remains the Theater's stated
   next era, not its present one.

An earlier `bahyway-index-preview.html` draft (linking a `bahyway-style.css`
that doesn't match this site's actual `style.css` filename) was not carried
into this pass, same reasoning as beemdm.com's dropped preview file.

## Docs page added

`content/what-is-bahyway.md` and `docs/what-is-bahyway.html` landed in a
later pass, matching beemdm.com's markdown-source-plus-rendered-twin
pattern. Both had the exact same three drifted claims as `index.html` above
(no-third-party-dependencies, ENLIL vs the Anu Index Stack, and the
aspirational egui/WGPU Theater description) plus the same oversimplified
linear `EnkiSDB → ... → EnkiDDB` pipeline arrow chain already fixed in the
top-level `docs/WHAT_IS_BAHYWAY.md` and in beemdm.com's docs page — all
fixed identically here before landing, since it's the same source content
rendered into this site's page anatomy. Anchor IDs (`#problem`, `#concepts`,
`#how`, `#arsenal`, `#sages`, `#sovereign`) verified to match between the
markdown headings and the rendered HTML's `docs-nav` sidebar.
