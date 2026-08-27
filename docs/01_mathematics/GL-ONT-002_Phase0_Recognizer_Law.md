# GL-ONT-002 — Phase 0 Recognizer Law
## Pure Rust, deterministic, offline-forever — no external model dependency in production

**Ecosystem:** BahyWay.Ecosystem v4.0
**Domain:** GL (Global Law) — Ontology / Recognition / Sovereignty
**Status:** SEALED-CONCEPT (per CSR-08 chat confirmation, 2026-08-15)
**Author:** DUB.SAR 𒁾
**Related tablets:** GL-ONT-001 (OntoGraph — Rite I Reading, Z3 at Gate G4 precedent), GL-NAV-001 (fastembed-rs/ONNX offline-embedding precedent), GL-DB-001 (No False Authority), CSR-08 (Architect Sovereignty).

---

## 1 · Purpose

This tablet formalizes a standing law the Architect has stated directly,
in these exact terms: **"NO External Model, Just Pure Rust and any once
Download use for ever law."** It governs every recognizer, classifier, or
NLP/NER-style component that ships in production anywhere in this
ecosystem — starting with the Phase 0 recognizer that reads raw,
unformatted documents and discovers their structure (the instrument
imagined this session as "Nasaru Instrument Phase 0").

The law is not "no ML, ever." It is: **production never depends on a live
external model call.** Everything the recognizer needs to run must either
be pure Rust logic, or a resource downloaded once and used forever
offline. ML/NLP tooling remains genuinely useful — at design time, as a
comparison instrument — under the same discipline this ecosystem already
applies to Z3.

---

## 2 · Definitions

**Recognizer** — any component that reads raw input (a document, a field,
a particle) and assigns it a structural or semantic classification: a
concept, a category, a taxonomy slot. Phase 0's recognizer specifically
discovers document structure from unformatted input, feeding the
Nebuchadnezzar/OntoGraph pipeline (`GL-ONT-001`).

**Pure-Rust deterministic recognizer** — the only form of recognizer
permitted to run in the shipped, production binary. Given the same input,
it produces the same output, with no network call, no live inference
endpoint, and no dependency that requires an internet connection to
function once installed.

**Once-Download-Use-Forever resource** — a file (lexicon, embedding table,
ONNX model, tokenizer vocabulary) fetched exactly once, cached locally,
and thereafter read only from local disk. This is not a loophole in "pure
Rust" — it is the same offline discipline `GL-NAV-001` already sealed for
Nabû's SSE: *"embed the sentence (fastembed-rs, offline)... tiles cached
locally, never fetched live in the sovereign path"* (§1.1, §6). A
once-downloaded ONNX model read through a pure-Rust inference crate is
lawful under this tablet on the same terms fastembed-rs's cached tiles
already are.

**Design-time-only ML/NLP comparison harness** — a separate, non-shipped
tool that runs an external ML/NLP model (local or hosted, any license) side
by side with the deterministic recognizer, during development only, to
find gaps between what the deterministic recognizer catches and what a
heavier model catches. It never ships. It never runs in the production
binary. It never gates a runtime decision.

---

## 3 · The Law

**Clause 3.1 — Production is pure Rust, deterministic, offline-capable.**
The shipped recognizer contains no code path that calls a live external
model — no hosted inference API, no runtime download, no network
dependency for its core function. It may read Once-Download-Use-Forever
resources already present on local disk.

**Clause 3.2 — The harness never ships.** Any ML/NLP model used to probe
the recognizer's gaps runs only in the authoring/design environment. It is
never linked into the production binary, never invoked from a production
code path, and never a dependency the shipped crate requires to build or
run. This mirrors the Z3 discipline already sealed at Gate G4
(`GL-ONT-001` §5, item 4: *"Z3 composite proof — only at Gate G4, only
when [a pattern] is promoted to a sealed tablet. Never at runtime, never
in the shipped binary"*) — Z3 proves; here, an ML/NLP model probes. Same
boundary, same reason: a heavier, harder-to-audit tool earns a voice at
design time, never a vote at runtime.

**Clause 3.3 — A gap is motivation, not a verdict.** If the design-time
harness finds a real classification the deterministic recognizer misses,
that gap is evidence the pure-Rust algorithm needs a new rule, a new
Once-Download-Use-Forever resource, or a better heuristic — never
evidence that the ML model should be promoted into production instead.
The deterministic recognizer is the thing that ships; the harness's job is
making it better, not replacing it.

**Clause 3.4 — Reproducibility is the test.** A recognizer decision that
cannot be reproduced offline, deterministically, from the same input, on a
disconnected machine, has failed this law regardless of its accuracy.
Accuracy earns a rule change under Clause 3.3; it never earns an exception
to Clause 3.1.

---

## 4 · Why this law, stated plainly

Every other sovereignty guarantee in this ecosystem — pure-Rust, no
external database engine, no cloud dependency, offline-first bare-metal
deployment — is undermined the moment a recognizer's correctness depends
on a live call to someone else's model, someone else's uptime, someone
else's changing weights. A recognizer that silently degrades or changes
behavior because an external API updated its model is not sovereign, no
matter how accurate it is today. `GL-DB-001`'s No False Authority law
already forbids claiming certainty the system does not have; this tablet
extends that same honesty to the recognizer's own dependency graph — it
must be honest about running entirely on what the Architect's own hardware
holds.

---

## 5 · Applying this to Phase 0

The Phase 0 recognizer (raw, unformatted documents → discovered structure,
feeding OntoGraph's Rite I Reading) is the first component this law binds,
by name, at landing:

1. **Design-time:** run the deterministic recognizer against a real corpus
   of unformatted documents, and — separately, never in the same binary —
   run a design-time ML/NLP harness (candidate: a local embedding model
   plus a pure-Rust nearest-neighbour crate, matching `GL-NAV-001` §7's
   candidate substrate) against the same corpus.
2. **Compare.** Where they agree, no action. Where the harness finds
   structure the deterministic recognizer misses, record the gap as a
   real, cited finding — not folded silently into either tool's output.
3. **Enhance the deterministic recognizer**, per Clause 3.3 — a new
   pattern rule, a new lexicon entry, a new Once-Download-Use-Forever
   resource — until the gap closes or is explicitly logged as accepted
   scope, never patched over by routing production through the harness's
   model instead.
4. **Ship only the pure-Rust recognizer.** The harness and any model
   weights it used stay out of the production build entirely.

---

## 6 · Playbooks

- **PB-325** (reserved) — Phase 0 recognizer crate scaffold + design-time
  comparison harness wiring, mirroring PB-322's OntoGraph scaffold
  pattern (`GL-ONT-001` §9).
- **PB-326** (reserved) — first real corpus comparison run + gap report,
  logging every deterministic-recognizer miss found by the harness under
  Clause 3.3, before any rule changes are made.

## 7 · Seal

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
