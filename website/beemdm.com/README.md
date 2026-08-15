# beemdm.com — site source

Static website for BeeMDM, built on the same architecture Rerun uses for rerun.io,
translated into sovereign form.

## How Rerun does it (decoded)

- Docs **content** lives as markdown inside the main code repo (`rerun/docs/content/`).
- A separate renderer repo (`rerun-io/landing`, private, Next.js on Vercel) reads that
  markdown at a pinned commit and renders the website.
- Result: one truth source for docs, versioned with the code.

## How this repo does it (sovereign form)

- `content/` — markdown is the truth source (`what-is-beemdm.md`). Later this folder
  moves into the BahyWay monorepo, exactly like Rerun's `docs/content/`.
- `index.html`, `docs/`, `style.css` — the thin static renderer. No framework, no build
  step, no Vercel. Prototype-tier by law (HTML serves humans reading; production
  visualization remains DubSar's).
- Every rendered docs page links to its markdown source — the Rerun "view source" pattern.

## Deploy free on GitHub Pages

1. Push this folder to a repo (e.g. `bahyway/beemdm.com`).
2. Settings → Pages → Source: deploy from branch → `main` → `/ (root)`.
3. Add a `CNAME` file containing `www.beemdm.com` and point the domain's DNS
   (CNAME record) at `<username>.github.io`.

No server, no cost, HTTPS included.

## Upgrade path (when wanted, not required)

Replace the hand-rendered HTML pages with **Zola** (pure-Rust static site generator):
templates render `content/*.md` automatically, so editing markdown is the only
authoring act — full Rerun-parity (markdown in, site out) with zero JavaScript
and a single Rust binary. The stylesheet and page anatomy here carry over as the
Zola templates unchanged.

## Design tokens

- Palette: Ishtar Gate — lapis `#0B1626 / #14263F / #24405F`, honey `#E8A33D`,
  wax `#F2E8D5`.
- Type: Marcellus (display), IBM Plex Sans (body), IBM Plex Mono (data).
- Signature: the swarm-to-comb hero — source particles drifting into hexagonal
  cells that seal gold. Many records in, one golden record out.
- Honors `prefers-reduced-motion`; responsive to mobile.

## 2026-07-28 correction pass

Three claims had drifted from what's actually built and were corrected across
`index.html`, `content/what-is-beemdm.md`, and `docs/what-is-beemdm.html`:

1. **"No third-party stack"** — false on its face (this ecosystem runs a forked
   Godot engine, tokio, serde, z3, and more). Restated as "sovereign core, no
   vendor lock-in" — the real, defensible claim.
2. **The seven-database flow was a single linear arrow chain** — EnkiDW actually
   receives *retired* EnkiODB particles (not sequential-after-EnkiDB), and
   EnkiMDB/EnkiDDB hold the ecosystem's own metadata and documents, not the
   mastered business data. Rewritten as each database's real duty.
3. **"ENLIL index"** — ENLIL already names the ecosystem's Total Algebra Content
   (GeoLaw-05); the index stack was deliberately renamed to the **Anu Index
   Stack** specifically to resolve that collision. Updated throughout.

An earlier `beemdm-index-preview.html` draft (which linked a `beemdm-style.css`
that was later renamed to `style.css`) was not carried into this pass — `index.html`
is the single current entry point; keeping a second, differently-linked copy around
would just reintroduce that broken-link risk.

Google Fonts is still loaded from `fonts.googleapis.com` — a real third-party
network request, left as-is for now since fixing it means vendoring the actual
`.woff2` binaries, not just editing prose. Noted inline in `index.html`.

## Endgame

When the DubSar WASM/WebGPU build exists (post-PB-160), the hero SVG is replaced by
the live viewer running a sealed demo tribe — the Rerun `/viewer` move: the product
as the homepage.
