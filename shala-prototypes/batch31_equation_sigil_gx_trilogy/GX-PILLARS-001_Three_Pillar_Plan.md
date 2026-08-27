# GX-PILLARS-001 — STEP-BY-STEP PLAN OF USE · THE THREE PILLARS
**BahyWay.Ecosystem v4.0 · Automation · Visualization/Simulation · Triple-O**
**Compiled at the Turning · 2026-08-23 · stages cross-referenced to GX-COMMENCEMENT-001**

> Each pillar is given as a numbered operating procedure: what to run, in
> what order, what gate ends each step, and which stage of the Turning
> (I The Ground · II The Courts · III The Mercies) it belongs to.

---

## PILLAR 1 — AUTOMATION (CSR-08: the playbook IS the word)

**Principle.** Nothing is done by hand. Every act is a numbered playbook run
from the bare-metal Fedora host toward the KVM VMs; every run journals to
NĀRU; every playbook pairs with a Šala tab. A step is finished when its
gates are green, never when it feels finished.

**Step 1 · Arm the host (Stage I).**
Verify toolchain and truth of the ground: `cargo --version`, Ansible
present, `BAHYWAY_ROOT` exported. Gate: preflight tasks of any playbook
pass without assertion failures.

**Step 2 · Burn the backlog to green (Stage I — the Trinh gate).**
Run PB-88 … PB-356 in numeric order on the host. For every run: COMPILE
gate (cargo build) and RUNTIME gate (cargo test) are blocking; a red gate
stops the sequence until fixed by a *new* numbered playbook (never a manual
edit). Gate: the NĀRU journal shows one green act per tablet; BLK-1…BLK-5
retired.

**Step 3 · Raise the Gula chain (Stage I).**
Run PB-357 → 358 → 359 → 360 → 361 → 362 → 363 strictly in order (each
refuses without its predecessor). Gate: seven green acts; `engines/gula`
tests all pass.

**Step 4 · Fuel the Harvest (Stage I).**
Annotate sealed tablets with machine-readable witness lines
(`ḪUBULLU:: engine= particle= orbit= tribe= gloss="…"`), one per gloss,
tablet by tablet. Gate: PB-358's read-only harvest count > 0 and rising;
first organic wounds appear in NĀRU (*speed the witnesses, never the
verdict* — thresholds untouched).

**Step 5 · Automate the streaming spine (Stage II).**
Wire real sensor exports (WPD telemetry first) into Uṣurtu's
`push_tile → step → drain` loop as a scheduled playbook (systemd timer
provisioned by Ansible, never cron-by-hand). Gate: `finished()` true on
every scheduled window; zero dropped tiles; SUMMONED/SUSPECT verdicts
landing in NĀRU.

**Step 6 · Automate the exhibits (Stage II→III).**
Nightly playbook renders Uṣurtu SVG sheets and Kanīku receipt bundles into
a dated `exhibits/` directory; PROVISIONAL watermark logic left untouched —
the press law is not configurable. Gate: sheet per polygon per night,
watermark correct against `finished()`.

**Step 7 · Kish crossings by hand of the Architect only (all stages).**
Any promotion URUK→KISH, any seal, any naming: a dedicated playbook that
Bahaa runs personally. Gate: the act line in NĀRU carries the seal note.

---

## PILLAR 2 — VISUALIZATION / SIMULATION (truth before beauty)

**Principle.** Every claim gets a stage before it gets a stakeholder. All
tabs carry honesty banners (REHEARSAL vs live), COMPILE+RUNTIME self-test
gates, the full Naṣāru suite, and the Membrane Motion Doctrine. Rehearsal
seed is always 3600. The Šurinnu stele is a controller, never décor.

**Step 1 · Rehearse in Šala (Stage I).**
Open each of the seven Turning tabs (Lišānu Court → Zaqīqu Veil → Apsû
Mirror → Uttu Loom → Puluḫtu Gate → Agû Crown → Uṣurtu) and walk the full
scripted arc, including the refusals — they are the demonstration. Gate:
both header gates green on first frame in every tab.

**Step 2 · Bind tabs to live NĀRU (Stage II).**
Replace scripted event feeds with journal tails; honesty banner flips from
REHEARSAL to LIVE only when data provenance is real. Gate: a verdict in
NĀRU appears in the tab's journal panel within one refresh.

**Step 3 · Stand up the standing dashboards (Stage II).**
Agû Wave per domain (Monitoring first: Storage IO / resource sectors,
tribes riding, libbu pulse mapped to the domain's worst bound). Nergal
Cubic for held/refused matter. EnkiDB Golden Store Dashboard v11 remains
the investor demo. Gate: libbu bpm demonstrably tracks the bound during a
staged incident drill.

**Step 4 · Simulate before ground (Stage II).**
For each product claim, run the rehearsal twin first (as the twin tests
caught the PB-361 parameterization that would never certify). Rule: any
scripted demo must be numerically replicated headless before it is shown.
Gate: replication script passes; demo end-state matches.

**Step 5 · Exhibit to the world (Stage III).**
Uṣurtu sheets to field teams; Agû Crown to operations rooms; DubSar Theater
lenses for engineering review; the sigil (mark/lockup/tablet) on every
external document. Gate: every external artifact carries its honesty line
("a bound, never a certainty" / "suspicion, never confirmation").

---

## PILLAR 3 — TRIPLE-O (the ontology practiced daily)

**Principle.** Model first, code second. Every new thing is answered on
three faces before it exists: what is its Particle, what Orbit binds it,
what Tribe owns it. Laws are sealed with Ḫubullu glosses; names wait in
the queue; contested matters go to the Fadam functional.

**Step 1 · Face decomposition (every new object, all stages).**
Write the signature: P[particle-face] | O[orbit-face] | T[tribe-face].
Check Hepta uniqueness (no duplicate positions). Gate: signature admitted
to the corpus without collision.

**Step 2 · Gloss at sealing (GL-NAM-002, all stages).**
Every sealed tablet receives its plain-language Ḫubullu gloss AND its
machine-readable witness line — one act, two forms. Gate: harvest count
increments; a human who is not Bahaa can read the gloss and understand.

**Step 3 · Let the courts breathe (Stage I→II).**
Wounds trigger only by three glosses + two engines; phantoms only by
surrounded gaps; tissue only by three independent engines; certificates
only through both locks. Never lower a threshold to go faster —
*speed the witnesses, never the verdict.* Gate: every TRIGGERED/MINTED/
RECOVERED/TISSUE/CERTIFIED line in NĀRU names its witnesses.

**Step 4 · Adjudicate the contested (all stages).**
Conflicting witnesses, ambiguous genomes, gloss disputes → the Fadam
functional; the five contested-sky instruments (Δt_decree, T, r\*, ζ, f)
where dynamics are in dispute. Gate: a written verdict, journaled.

**Step 5 · Seal, name, and cross (Bahaa only, all stages).**
Work the naming queue (Gula · Zaqīqu · Apsû-Mirror · Uttu · Uṣurtu · the
Fadam Field Condition · era-name Nebuchadnezzar). Each seal = a Kish
crossing playbook. Gate: NL-001 respected — gods/engines, cities/
structures, kings/eras.

**Step 6 · Register the paradigm (Stage III).**
Zenodo DOI whitepaper led by the technical claims (pure-Rust CQRS graph
lineage; persistent-homology DQ monitoring; τ under the EU AI Act; the
Fadam Field Condition with its Five Refusals) — Mesopotamian philosophy
second, as the naming system it is. Then Benelux trademark; then NWO/WBSO
with a university co-applicant. Gate: a DOI exists; τ is citable.

---

## THE THREE PILLARS IN ONE LINE EACH

**Automation:** the playbook is the word, the gate is the truth, the
journal is the memory.
**Visualization/Simulation:** every claim rehearses before it testifies,
and every screen confesses what it does not know.
**Triple-O:** three faces before code, a gloss before a seal, a witness
before a verdict — and the word, always, is Bahaa's. 𒁾
