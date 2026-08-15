# ŠALA DESIGN CHARTER — standing law for all UI work in this repo

> Place this file in the Šala repo root (or paste its contents into `CLAUDE.md`).
> Every agent, every session, every tab: read this before touching any HTML/CSS/JS.
> The charter travels with the codebase, not with the conversation.

## 1. Identity

Šala is the prototype workbench of **BahyWay.Ecosystem v4.0**, built by the Architect
(DUB.SAR 𒁾). It is *rehearsal scaffolding* — Way-of-Work rule 5 holds: no HTML in
production, ever. Šala's job is to rehearse engine concepts visually before they are
sealed in pure Rust. Its look must match the sovereign identity of the ecosystem:
Mesopotamian, inscribed, dark, deliberate. **Never** default-bootstrap, never flat
utility-orange, never unstyled navy.

## 2. Surfaces — locked charcoal charter (from `theme.rs`, do not invent)

| Token          | Hex       | Use                          |
|----------------|-----------|------------------------------|
| `--bg-base`    | `#2B2D31` | main background              |
| `--bg-panel`   | `#232527` | panel surfaces               |
| `--bg-widget`  | `#1E2022` | inputs, code blocks          |
| `--bg-deep`    | `#18191B` | orbit stage, previews        |
| `--bg-hover`   | `#383A3E` | hover states                 |
| `--bg-selected`| `#404349` | active tabs, selection       |
| `--stroke`     | `#3D3F43` | all borders                  |

## 3. Accents — tribe colours (meaning-bearing, never decorative)

| Token       | Hex       | Meaning                                        |
|-------------|-----------|------------------------------------------------|
| `--gold`    | `#D6A44C` | GOLDEN state · seals · active element · search hit |
| `--teal`    | `#4FB8A8` | pass · info · File A · Architect's cursor      |
| `--orange`  | `#D98E3E` | FUZZY state · warning · File B                 |
| `--crimson` | `#C34A3F` | DEAD state · conflict · gate refusal           |
| `--purple`  | `#9B7FC7` | operators, rare accents only                   |

Ink: `--ink #E8E6E1`, `--muted #9A9C9F`, `--dim #6E7074`.

**Colour law:** a colour states a fact. Gold means golden/active/sealed. Crimson means
refused/dead/conflict. If a colour carries no meaning, use a surface or ink token instead.

## 4. Type — three roles, never fewer

- **Display / titles:** serif inscription voice —
  `"Iowan Old Style","Palatino Linotype","Book Antiqua",Georgia,serif`.
  Used for the Šala wordmark, tab-panel titles, verdicts. With restraint.
- **Body / UI:** system sans — `-apple-system,"Segoe UI","Noto Sans",Ubuntu,sans-serif`.
- **Data / code / identifiers:** `"JetBrains Mono","Fira Code",ui-monospace,Consolas,monospace`.
  All playbook names, counts, KAKI bytes, hashes, HeptaScript.

Everything is offline-sovereign: **no webfont CDNs, no external requests, ever.**

## 5. Structure rules

- Tabs are gates: inactive = quiet (`--muted` on transparent, `--stroke` border);
  active = gold text, gold top border, fused into `--bg-base`. Tab index in mono
  (`Tab 5`) before the name.
- Panels: `--bg-panel`, 1px `--stroke`, radius 10px. Inputs: `--bg-widget`.
- The orbit stage sits on `--bg-deep` inside a bordered, rounded frame — the stage
  is a *place*, not leftover void.
- Cubes are witnesses, not decoration: idle `#6E7074` dim · matched `#D6A44C` gold ·
  selected `#4FB8A8` teal · blocked `#C34A3F` crimson. An idle catalogue must read
  quiet; a search must visibly *light* its matches.
- Search results: mono 12.5px rows, hairline dividers `rgba(61,63,67,.45)`,
  hover = `--bg-hover` with a small left-indent shift. Playbook name bold ink;
  metadata `--dim`; match-in-title gold, match-in-body muted.
- State verdicts always map to the EAV trichotomy: GOLDEN / FUZZY / DEAD →
  gold / orange / crimson pill (`.state-golden` etc. in `shala_charter.css`).

## 6. Language in the UI

- HeptaScript is Anti-SQL: any query text shown uses
  `PRESENT · ORBIT · EMIT · PROVE · SYNC · WITNESS` — **never** SELECT/FROM/JOIN.
- Naming obeys NL-001: gods → engines, cities → structures. (CompareEngine: lawful
  descriptive class. Kish: city, structures only. Ishtaran: reserved god-name for
  the comparison engine if ever elevated.)
- Copy is plain and active: "Load a config to begin", "Search", "Seal". No filler.

## 7. Quality floor (silent, always)

- `:focus-visible` gold outline on every interactive element.
- `prefers-reduced-motion` respected.
- Dark scrollbars (`--bg-deep` track, `--bg-selected` thumb).
- Responsive to ~980px minimum; panels stack, stage stays framed.

## 8. How to apply to the current Šala

1. Copy `shala_charter.css` into the static assets directory.
2. Link it **after** existing styles: `<link rel="stylesheet" href="shala_charter.css">`.
3. Update the cube renderer to the cube colour law in §5 (renderer-side, not CSS).
4. New tabs (Shala6 · CompareEngine and onward) are built *from* these tokens,
   not restyled afterward.
