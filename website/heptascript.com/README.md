# heptascript.com — site source

Deliberately thin satellite: one screen, the language as the hero, funneling to
bahyway.com for full docs. GitHub Pages deployable (CNAME "www.heptascript.com").

Signature: the inscribed query tablet. The code sample is illustrative HeptaScript
v1.1 — update it to canonical syntax when the language reference is sealed.

## 2026-07-28 correction

The tablet's result line originally read "→ 1 000 000 000 particles · 0.41 s ·
sealed 𒁾" — presented as an achieved result, not clearly marked illustrative the
way the query syntax itself is. This repo's own `docs/PB-221_SCALE_BENCHMARK_
FINDINGS.md` (section 4) explicitly states that returning a literal billion rows
over a network in under a second isn't achievable by any system — network/
serialization physics, not a HeptaScript limitation — and was never what was
actually measured. What *was* measured and is real: LIMIT-bounded queries hold
flat latency (10–44ms) regardless of corpus size, confirmed at 1M and 10M
particles. Changed the result line to "→ flat latency, any corpus size · sealed
𒁾" to match the honest, actually-measured claim.

`style.css` here is the real, complete stylesheet for this site (shared design
tokens plus the tablet/dim-card components this page actually needs) — an
earlier pass had to reconstruct these rules from scratch because this file
hadn't been delivered yet.
