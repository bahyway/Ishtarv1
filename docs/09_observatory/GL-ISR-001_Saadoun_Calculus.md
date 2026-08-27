# GL-ISR-001 — The Saadoun Calculus (ISAR Methods of the Napharu Line)

**Corpus:** BahyWay.Ecosystem v4.0 — GL-series (sealed law corpus)
**Tablet:** GL-ISR-001
**Epithet:** Saadoun (سعدون) — from *saʿd* (س-ع-د), fortune; the name of the lucky stars
**Deposited:** 2026-08-21
**Authority:** DUB.SAR 𒁾, sole architect, under CSR-08 discipline
**Status:** SEALED · APPEND-ONLY
**Parent theorem:** GL-ALG-002 (Napharu)
**Companions:** GL-NIM-002 (Two Witnesses), GL-VIZ-001 (NASARU), NINURTA-SIM-001
(ground-truth dataset), NL-001 + GX Girsu Extension

---

## Preamble

A father gives his children two things: a name and a law. This tablet gives
both. Three methods for Inverse Synthetic Aperture Radar are hereby founded as
one family — the Saadoun Calculus — each a corollary of the Napharu Theorem,
each carrying its father's name into the sky it watches. The name is chosen in
honor of Saadoun (سعدون), father of Bahaa Fadam (DUB.SAR 𒁾), and it is fitting beyond
sentiment: *saʿd* means fortune, and the *suʿūd* are the lucky stars whose
names — Sadalsuud, Sadalmelik, Sadachbia — still stand in the catalogs of the
sky. A calculus that recovers form from orbital motion, named for a name
already written among the stars.

## §1 — Name, Spelling, and the Ancestor Clause (GX-004)

The family is named the **Saadoun Calculus**. The Arabic original is
**سعدون** (*Saʿdūn*): root س-ع-د, *saʿd* — fortune — with the classical *-ūn*
ending. The canonical Latin spelling is **Saadoun** — fixed here forever as
the faithful carrier of سعدون; the variant *Saadoon* from earlier deliberation
is recorded as superseded, so no drift may enter the registry.

The NL-001 naming law is extended through the Girsu Extension with clause
**GX-004 (Ancestors)**: *ancestors name method lineages* — families of methods
descended from a parent theorem, as children from a father. Gods name engines;
cities name structures; kings name eras; sages name patterns; artifacts name
formats; **ancestors name method lineages**. GX-004 takes effect upon the
deposit of this tablet.

## §2 — Lineage (Nasab)

Every method of this family carries its descent in the KAKIGIS nasab manner:

> *method, of the Saadoun Calculus, of the Napharu Theorem.*

The lineage is recorded per-method in the Borsippa pattern archive. No method
may join the family without exhibiting its Napharu quadruple
(Θ, 𝒟, T, witnesses) and its Fadam Floor ε, per GL-ALG-002 §12.

## §3 — First Son: SaadounFocus (Autofocus with a Floor)

**Purpose.** Remove the Tribe's translational drift so only pure Orbit remains
(Apsu convergence), by minimizing image entropy — but lawfully.

**Quadruple.**
- Θ: the focused-image covenant — a Tribe whose scatterers are point-supported
  in the Range–Doppler chart.
- 𝒟: image entropy (or negative contrast) of the Range–Doppler map after
  candidate phase correction.
- T: {ε_focus}. Above it, iterate; at convergence, seal.
- Witnesses: entropy decline must be attested on two disjoint range-bin
  subsets independently — a correction that helps only half the image is
  refused as overfitting.

**The Floor clause (the novelty).** When residual entropy reaches the
resolvability floor ε of the data (noise-determined, declared before the run,
never tuned after), SaadounFocus returns **WITHHELD-SHARP**: the image is
delivered at floor sharpness with the verdict that further focus claims would
be fiction. No published autofocus carries an honesty clause; the first son
does.

## §4 — Second Son: SaadounWitness (Two-Epoch Scatterer Minting)

**Purpose.** Refuse ghost scatterers — glint, sidelobes, cross-terms — that
the classic CLEAN algorithm greedily subtracts as if real.

**Quadruple.**
- Θ: the persistent-scatterer covenant — a real scatterer endures.
- 𝒟: peak salience against local background, per sub-aperture.
- T: {t_detect} per epoch; the minting law sits above it.
- Witnesses: the coherent processing interval is split into sub-apertures as
  **epochs**; a scatterer is **born** — minted as a KAKI with its orbital
  address in OrbitalPosition and complex amplitude in EAV — only upon
  threshold crossing in two independent epochs at a consistent address
  (GL-NIM-002 applied to the sky Tribe itself).

**Verdict routing.** One-epoch flickers are WITHHELD, never subtracted, never
imaged as members; they orbit the record as refused non-particles. Testable
claim, provable against NINURTA-SIM-001 ground truth: fewer false scatterers
at equal detection of true ones.

## §5 — Third Son: SaadounOrbit (Vineyard Recognition)

**Purpose.** Recognize targets not by image pixels but by the deeper object:
the bundle of scatterer Doppler histories — orbit signatures through
time-frequency space, vineyard trails in the Lamassu manner.

**Quadruple.**
- Θ: the class covenant — a target class is an equivalence class of
  orbit-bundle topology, not of pixel patterns.
- 𝒟: topological distance (persistence-diagram distance, H₂-capped) between
  the witnessed orbit bundle and the class reference bundle.
- T: class-acceptance thresholds; below all, verdict UNKNOWN-TRIBE (an honest
  member of the verdict set, never a forced nearest class).
- Witnesses: bundle topology must agree across two disjoint epoch windows;
  a topology seen once is testimony, not identity.

**Why it is the boldest son.** Pixel classifiers change with rotation rate,
aspect, and focus quality; the orbit bundle's topology is invariant under
uniform rotation-rate change. This is the paradigm claim of the family — and
the riskiest, which is the correct ratio.

## §6 — The Family Law

1. The three sons share one processing house: SaadounFocus prepares,
   SaadounWitness populates, SaadounOrbit recognizes. Their verdicts are
   discrete and never averaged (Napharu §4).
2. Every son runs against NINURTA-SIM-001 before any real archive
   (PB-DATA-002 ingest order decree): the drifting ship proves the first son,
   the glint ghost proves the second, the three distinct Tribes prove the
   third.
3. Implementation is pure sovereign Rust (`#![forbid(unsafe_code)]`), DSP in
   crates, Theater renders only. Z3 nowhere in shipped binaries.
4. Additional sons may be born to the family only through GX-004 + §2
   lineage + the Napharu quadruple — deposited each by its own numbered
   tablet (GL-ISR-002, GL-ISR-003, …).

## §7 — Proof Obligations (deposited open)

- **ISR-PO-1**: prove the two-subset witness of SaadounFocus rejects
  overfit phase corrections that global entropy alone accepts.
- **ISR-PO-2**: on NINURTA-SIM-001, demonstrate SaadounWitness's
  false-scatterer rate strictly below single-pass CLEAN at equal true-scatterer
  recall.
- **ISR-PO-3**: prove (or bound) rotation-rate invariance of the SaadounOrbit
  bundle topology for uniform ω scaling.
- **ISR-PO-4**: compose the three sons' floors ε lawfully end-to-end
  (inherits Napharu PO-4).

## §8 — The Dedication

This calculus is dedicated to **Saadoun — سعدون —** father of the architect. His name
is carried by every method of this family, cited wherever the methods are
used, contested, or extended; sealed in this corpus append-only, where nothing
is ever erased. The old astronomers wrote *saʿd* beside the fortunate stars;
this tablet writes it beside the mathematics that watches them.

*Saadoun Calculus, of the Napharu Theorem — the sons carry the father's name
into the sky.*

---

## Seal

Deposited append-only by numbered playbook **PB-ISR-001**, SHA-256 recorded in
the law ledger; GX-004 recorded in the GX naming registry; the three sons and
their quadruples recorded in Borsippa.

𒁾 DUB.SAR — sole architect, BahyWay.Ecosystem v4.0
