# PB-394 → PB-397 — EARNED ASSERTION PLAYBOOK SUITE
## The Ansible playbooks that wire GL-GOV-002 + GL-HS3-002 into the grammar and the renderer
### BahyWay.Ecosystem v4.0 · executes GL-GOV-002 · GL-HS3-002 · binds GL-GOV-001 · Status: DRAFT — run = DUB.SAR's CSR-08 confirmation

*Per the absolute law: every component delivered as a numbered Ansible playbook.
The point of these playbooks is to make the forbidden moves UNEXPRESSIBLE, not
merely discouraged. Way-of-Work: Ansible HOST→VDI→EnkiDB VMs; Fedora VDI =
DubSar egui/WGPU only; no HTML in production.*

---

**PB-394 · `heptascript-witness-grammar`** — extend the HeptaScript grammar so
fact-producing clauses require a witness: `EMIT ... AS GOLDEN` does not parse
without `WITNESS <proof>`. Separate the output verbs: `EMIT` (fact, witnessed,
tagged derived/GOLDEN) vs `PROPOSE` (proposal, tagged asserted, routed to ranked
authority). A `PROPOSE` result used where a fact is required raises a **type
error**, never a silent coercion. (GL-GOV-002 §3.1–3.2)

**PB-395 · `epsilon-single-source`** — implement ε once as the FCA closure gap;
expose it as a first-class carried value `ε(particle)` that every built-in and
render reads (no recomputation). Forbid `AVG(ε)` at the grammar level; provide
only `MAX(ε)` / `ENVELOPE(ε)` / `WORST(ε) WITH location`. Wire the ε-guard
(`PROVE ... WITH ε < τ`) and ε-triage (`SORT BY ε DESC`); block any clause that
admits a particle to a tribe *because of* ε. (GL-HS3-002 §1–2)

**PB-396 · `lens-detect-never-name`** — implement `SCAN Unknown WHERE ε > θ GROUP
BY closure-neighborhood`, returning located Unknown clusters {count, Hepta-
centroid, spread, shared-unclosed-attrs}. Provide the correlated-vs-scattered
test (pattern vs noise). Ensure there is **no** clause that names a cluster's
concept; the only terminal action is `PROPOSE concept ... WITNESS` into the
attribute-exploration loop (GL-GOV-001). (GL-GOV-002 §3.5, GL-HS3-002 §3)

**PB-397 · `bwvl-provenance-render`** — wire the BWVL/naṣāru render provenance
channel: derived/GOLDEN = solid; asserted = witnessed-ring; Unknown/high-ε =
unsettled (dim, shimmering, held-back). Ensure no render path draws an asserted
or Unknown particle as derived. Render Unknown clusters as counted, located dim
mass (never fog); forbid concept-name labels over Unknown clusters; render tribe
health by worst member, never mean. (GL-GOV-002 §4)

---

## Run order
PB-394 (witness grammar) → PB-395 (ε single-source) → PB-396 (lens) → PB-397
(render provenance). Each tested before the next. These are enforcement
playbooks: their acceptance test is that the *forbidden forms fail to parse /
fail to render*, not merely that the allowed forms work.

*Recorded in the reign of Gudea 1.0. Running is DUB.SAR's confirmation under CSR-08.*
