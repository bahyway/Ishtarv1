# SPEC-AKD-001 — AcadEngine and the Three Domains

Status: SEALED (design), landed by PB-254. Renumbered from an
ideation-only session's "PB-180 REV-2" draft — see PB-254's own
header comment for the three corrections applied before landing
(renumbering, host/invocation, paths) and the Architect's
recorded rulings R1-R3.

## Engine and rulings
AcadEngine: sovereign static Academy generator. Descriptive-
acronym engine class (egd-engine/wpd-engine precedent); "Acad"
= Academy — not a god name, not a city name, no NL-001
conflict. HTML5 confirmed as the lawful medium of outward
publication (Architect ruling R2). Production dashboards
remain egui/WGPU — unaffected.

## Purpose
The rendering layer for the Web Arsenal: the engine that lets
www.bahyway.com, www.heptascript.com, and www.beemdm.com
display the parts of EnkiDDB/EnkiMDB's existing documentation
meant for the outside world, and turn that into a real Academy
for learning BahyWay.Ecosystem's deep knowledge — not a
one-off page generator.

## One store, three faces
Source of truth (post-gate, see Honest Limit below): EnkiDDB
(7007). Every lecture is a particle (KAKI identity, EaAgent
generate-once model) with address sector/category/era and a
set of TARGET DOMAINS:

| Domain              | Face                     | Default sectors |
|---------------------|--------------------------|------------------|
| www.bahyway.com     | ecosystem + full Academy | ALL              |
| www.heptascript.com | the language's home      | DUBSAR           |
| www.beemdm.com      | commercial product face  | ADAD             |

Routing law: bahyway.com receives everything; heptascript.com
and beemdm.com receive their default sectors plus any lecture
that explicitly declares them. One lecture, one KAKI, N
renderings.

## Content model
- W5H2 body (Who What When Where Why How HowMuch)
- Blocks: Prose | MathML (native, zero JS) | Code (build-time
  highlighted, sovereign lexers post-gate) | Video (self-hosted
  MP4/WebM: Theater captures, Nisaba plates) | Reference
  (sealed tablets PH/SPEC/GL/PB + external literature)

## Rendering laws
1. Static HTML5 only; zero runtime JS dependencies.
2. Every lecture claim cites its sealed tablet or external
   reference — no unversioned claims.
3. Chronicle discipline: superseded lectures are never deleted,
   only marked with their succeeding era.
4. Cross-domain canonicals: heptascript.com and beemdm.com
   pages declare their bahyway.com canonical URL, keeping one
   scholarly identity across the three faces.

## Honest limit (post-gate work, deliberately deferred)
This spec and PB-254 land the content model, the per-domain
HTML5 renderer, and the routing/canonical laws — proven by
real cargo tests (routing_law_holds, mirror_declares_canonical).
NOT yet wired: a live EnkiDDB feed for lecture particles, the
real TemplateEngine (.tmpl) replacing the string-builder
renderer, sovereign per-language lexers for build-time code
highlighting, and actual SUSA outward publication to the three
live domains. Each is real, separate follow-up work.
