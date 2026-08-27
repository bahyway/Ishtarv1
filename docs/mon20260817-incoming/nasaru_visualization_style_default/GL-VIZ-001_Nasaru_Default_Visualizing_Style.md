# GL-VIZ-001 — The Nasaru Default Visualizing Style
## One Interaction Grammar for Every Šala Prototype

**Ecosystem:** BahyWay.Ecosystem v4.0 — visualization standard
**Consumes:** GL-STY-001 (StoryWay), GL-BRT-001 (Birth Gate / Two-Branch), GL-ONT-002 (Non-Substitution), GL-NSR-001 (Nasaru τ)
**Reference implementation:** `nasaru_style.js` (shared core) + `shala_nasaru_style_reference.html` (template court)
**Status:** SEALED — CSR-08 confirmed by the Architect

---

## 1. The Law

> **Every Šala prototype MUST present the four canonical interactions through the single shared core `nasaru_style.js`. No court reimplements them; drift is forbidden. The four are: (1) scroll-zoom, (2) 3D drag-rotate membrane, (3) left-click bounce-in-place with StoryEngine, (4) right-click context menu branching by birth-status.**

## 2. The Four Canonical Interactions

1. **Scroll-zoom (sky↔ground).** The wheel zooms the camera between a high overview and ground level, clamped to a sky-to-ground range (~0.55–4.0). This is the same altitude range the land-from-sky descent traverses.
2. **3D moving membrane.** Dragging rotates and tilts the field (yaw + pitch). An optional **ṬĀLUKU** auto-orbit slowly rotates the scene when idle. Drag and click are discriminated by a **3-pixel threshold** — movement beyond 3px is a camera drag, movement under it is a selection click.
3. **Left-click bounce-in-place → StoryEngine.** Clicking a **born Particle** lifts it off the membrane on a gold stem, damped-bounces it in place, and opens its **StoryEngine journal** (GL-STY-001) — birth, KISPU commits, colophon renewals. The stem connects the lifted Particle to its resting position so the selection is unambiguous.
4. **Right-click context menu, branching by birth-status.** This is where the Two-Branch Law (GL-BRT-001) is enforced at the interaction level:
   - a **born Particle** offers *Open StoryEngine journal* and *Add to cohort* ("born · individual · queryable by identity");
   - a **refused Non-Particle** offers *only* *Show Refusal Record* ("no KAKI · no StoryEngine (never born) · Bāb Ṭurdi only").
   A refused Non-Particle is never dignified with a StoryEngine, on either click — it was never born, so it has no story to lift into view.

**Optional:** a **land-from-sky** cinematic descent (`api.landFromSky`), starting high and easing down to a target — used by navigation and focus courts.

## 3. Why One Core

Reimplementing these per court invites drift — a bounce that behaves differently here, a menu that leaks a story to a refused record there. A single verified core makes the interaction grammar **sovereign and uniform**: fix or improve it once, every court inherits it. It is the visualization counterpart of "every fix is one numbered playbook."

## 4. The Boundary Guarantee

The core enforces, by construction, that:
- a refused Non-Particle never receives a StoryEngine (left-click ignores it; right-click gives only a Refusal Record) — GL-BRT-001 upheld in the UI;
- each Particle is an individual with its own journal, never a substitutable instance — GL-ONT-002 upheld in the UI;
- colours come from the caller (Nasaru τ), never invented by the core — GL-NSR-001 respected.

## 5. Production Note

This is the **Šala prototype** grammar (HTML). Production visualization is sovereign **egui/WGPU on the Fedora bare-metal host**; the four interactions are the specification the production visualizer implements natively. HTML courts are rehearsals of that specification.

## 6. Seal

```
Sealed by: DUB.SAR 𒁾  (Bahaa Fadam) — CSR-08 CONFIRMED
One core, four interactions, every court. Drift is forbidden.
```
