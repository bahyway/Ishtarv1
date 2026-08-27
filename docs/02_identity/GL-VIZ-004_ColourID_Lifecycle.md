# GL-VIZ-004 (candidate) — THE COLOURID LIFECYCLE
## Root colour = lineage · shade-degree = per-particle fingerprint · birth shade immutable · GOLDEN transition bounded
### BahyWay.Ecosystem v4.0 · binds GL-VIZ-003 (Particle Monism) · GL-KAKI-002 (three KAKI types) · v4.0 EAV/KAKI law · Status: SEALED-CONCEPT (per CSR-08 chat confirmation, 2026-08-15)

---

## 0 · Why colour is load-bearing in BWVL

At billion-particle density, the Bārû cannot open each particle's KAKI to tell
one from another. **Colour does the visual work KAKI cannot do at scale:** it
lets the eye read lineage (by hue-band) and individuality (by shade-degree)
instantly, across a billion particles, without a single identity lookup.

---

## 1 · ColourID lives in EAV, NEVER in KAKI

The KAKI v4.0 identity is immutable (GL-KAKI-002). The particle's *displayed*
ColourID **changes** as its state converts through ETL stations. A changing
value cannot live in immutable identity. Therefore:

**ColourID is a Mandatory EAV attribute, registered on Entity = Tribe across the
seven EnkiDB database types.** It is in EAV *because* it is mutable — this keeps
KAKI clean and immutable, consistent with the sealed v4.0 law that
colour/state/quality live only in EAV, never in KAKI bytes.

---

## 2 · Two colour values, different permanence

**(a) Birth Tribe Root Shade — IMMUTABLE.**
Assigned at first ingest (BeeMDM ETL), together with the Identity-KAKI. Example:
a light shade-degree of blue-ish green. It **never changes for the life of the
particle.** It is not stored in a KAKI byte; it is immutable because **no
Event-KAKI ever overwrites the birth event** — the first entry in the StoryEngine
Journal is permanent, so origin/tribe is always recoverable.

**(b) Displayed / current ColourID — MUTABLE (EAV on Tribe).**
What shows on the particle's surface *now*. It changes with each state
registration as the particle moves through the ETL station chain (blue / green /
maroon / red / purple = processing states). Every change is witnessed by an
Event-KAKI (GL-KAKI-002 §3).

---

## 3 · Root-hue = lineage · shade-degree = individual (two-level)

- **Broad domain** sets the **base hue** (e.g. Bacteriology, Symptoms, Organisms).
- **Sub-family / root** narrows the hue-band (e.g. Clostridial vs Bacillus vs
  symptom-class).
- **Each particle gets its OWN distinct shade-degree (a unique ColourID RGB)
  WITHIN its root's colour band.** No two particles share the exact
  shade-degree.

Consequence: **colour is a per-particle visual fingerprint that stays within the
family band.** Hue-band reveals which tribe/lineage a particle belongs to;
exact shade-degree distinguishes it from its siblings — visually, at
billion-scale, without reading KAKI. (KAKI + EAV remain the *authoritative*
distinguisher when certainty is required; the shade-degree is the *visual* one.)

---

## 4 · The GOLDEN transition is BOUNDED

When a particle reaches the golden store (EnkiDB → EnkiDW), its state ColourID
shifts to GOLDEN. This shift is **gentle and bounded:**

- It may make the particle **paler or slightly more yellowish**.
- It **MUST NOT** darken it to **brown or black**.
- It **does not overwrite or corrupt the Birth Root Shade** — the birth
  hue-family survives. A particle born light blue-green reads, when GOLDEN, as a
  paler/warmer blue-green — still recognisably its tribe.

GOLDEN is a lightening/warming *overlay*, never a destructive recolour.

## 5 · Aging / Decay (Steward-governed, root preserved)

GOLDEN state is stable. It changes only when a particle is **Aged / Decayed**,
and only through a governed event: **new data appends to the same GOLDEN
record**, firing an **alert trigger**; then a **Data Steward marks** the old
version "OLD" / "Aged" / "Decay." At that point the *content in use* changes —
**but the Birth Root Shade colour is preserved.** Aging is a witnessed Steward
decision (an Event-KAKI), never automatic state-drift, and never erases lineage
colour.

---

## 6 · The read for BWVL
- **Surface (now):** current ColourID (state-colour in ETL; paler-GOLDEN in the
  store).
- **Right-click / StoryEngine:** the immutable **Birth Root Shade** → origin &
  tribe, shown in the BIGRING view.
- **Federation scale:** read tribe-families by colour-band; descend to
  individuals by shade-degree.

## 7 · Codex compliance & placement
- **A-1 zero new mathematics:** composes EAV, KAKI immutability, Event-KAKI
  witnessing, Particle Monism. New = the ColourID lifecycle rules.
- **A-4 cited:** GL-VIZ-003 · GL-KAKI-002 · GL-VIZ-005 · v4.0 EAV/KAKI law.
- **PB:** PB-369 `colourid-eav-tribe`; PB-370 `golden-transition-bounded`.

## 8 · Open seals for CSR-08
ColourID as Mandatory EAV on Entity=Tribe (never KAKI) · immutable Birth Root
Shade · unique per-particle shade-degree · bounded GOLDEN transition (paler/
yellowish, never brown/black) · Steward-governed aging preserving root · PB-369/370.

*Recorded in the reign of Gudea 1.0. A particle wears its state on its face and
its birth in its bones; the bones do not change. Sealed under CSR-08 (chat confirmation), 2026-08-15.*

---

## APPENDED — Zoom-as-Necessity clause (per GL-VIZ-006 capstone)
Colour is what makes the **FIELD** readable at billion-scale (the bird's-eye):
tribe by hue, individual by shade-degree, state by brightness — read by looking,
without clicking. The **descent** is what makes the single **PARTICLE**
reachable. Both are required and inseparable: colour without descent = a
readable wall you cannot enter; descent without colour = a reachable fog you
cannot read at altitude. Sealed jointly with GL-VIZ-006.

## 9 · Seal

```
Sealed by: DUB.SAR 𒁾 (Bahaa Fadam), via explicit chat confirmation (CSR-08)
Date:      2026-08-15
AkkadianSeal (Ed25519): PENDING — no real signing infrastructure wired
                        yet (no Sargon/Gilgamesh passport ceremony run
                        against this tablet). The chat confirmation above
                        is the Architect's real CSR-08 act; the
                        cryptographic seal is separate, real follow-on
                        work, not fabricated here.
```
