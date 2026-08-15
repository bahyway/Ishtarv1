# 𒀭𒁾𒊭 DubSar IDE — EnkiDB Extension for VSCodium / VS Code

> **DubSar** (𒀭𒁾𒊭) — "The Scribe" in Sumerian.
> The sovereign IDE for the BahyWay.Ecosystem v4.0.
>
> دوبسار — الكاتب بالسومرية — بيئة التطوير السيادية لنظام BahyWay

---

## What Is This?

**DubSar IDE** is the official VSCodium/VS Code extension for the
[EnkiDB](https://github.com/bahyway/enkidb) sovereign data ecosystem.

It ships as a Codium extension today and is on the roadmap to become a
**standalone IDE** (DubSar Desktop) — a fully sovereign development environment
built on the same Rust + WASM foundation as the rest of BahyWay.Ecosystem v4.0.

---

## Features

### Included Now (v0.x — Extension Phase)

| Feature | Status | Description |
|---------|--------|-------------|
| **EnkiDB Manifold Dark** theme | ✅ | Deep navy `#060610` background · golden cuneiform accents `#FFD700` · cyan data-type highlights `#00FFFF` |
| **HeptaScript** syntax highlighting | ✅ | `.hpta` files — orbit queries, KAKI expressions, fuzzy rules |
| **Akkadian DSL** highlighting | ✅ | `.akkadi` and `.aaol` files |
| **KAKI Inspector** | 🚧 | Hover over a 16-byte hex literal → decoded KAKI fields in tooltip |
| **Cargo.toml crate lenses** | ✅ | Via `rust-analyzer` companion (recommended) |
| **Orbital quality badge** | 🔜 | Status bar shows B11 score for the active HeptaScript query |

### Roadmap — DubSar Desktop (v1.0)

| Feature | Target |
|---------|--------|
| Native WASM kernel (Rust → WASM) | Q4 2026 |
| Embedded `enkidb-engine` query runner | Q4 2026 |
| Orbital particle visualizer panel | Q1 2027 |
| StoryEngine time-travel diff view | Q1 2027 |
| BeeMDM pipeline inspector | Q2 2027 |
| HeptaScript compiler + REPL | Q2 2027 |
| Arabic RTL editor first-class | Q3 2027 |
| Eridu OS integration | 2027+ |

---

## Installation

### From Local VSIX (current method)

```bash
# Build the extension package
cd workspace/bahyway_v4/crates/dubsar-ide
npm install
npx vsce package

# Install in VSCodium
codium --install-extension dubsar-ide-*.vsix

# Install in VS Code
code --install-extension dubsar-ide-*.vsix
```

### From Marketplace (coming soon)

> The extension will be published to the Open VSX Registry
> (the sovereign, open-source Marketplace for VSCodium).

---

## Applying the EnkiDB Manifold Dark Theme

1. Open the Command Palette: `Ctrl+Shift+P` (Linux/Windows) or `Cmd+Shift+P` (macOS)
2. Type: **Preferences: Color Theme**
3. Select: **EnkiDB Manifold Dark**

Or set it directly in `settings.json`:

```json
{
  "workbench.colorTheme": "EnkiDB Manifold Dark"
}
```

---

## Colour Palette — The Sovereign Spectrum

| Role | Hex | Usage |
|------|-----|-------|
| Background | `#060610` | Editor background (deep Sumerian night) |
| Accent Gold | `#FFD700` | Keywords · cuneiform symbols · headings |
| GEM Cyan | `#00FFFF` | Data types · GEM-lane identifiers |
| GEM Green | `#00E5A0` | Strings · passing tests |
| TRIBE Blue | `#4DB8FF` | Functions · TRIBE-lane constructs |
| ACTIVE Yellow | `#FFD700` | Numbers · constants · ACTIVE-lane |
| FUZZY Orange | `#FF8C42` | Warnings · FUZZY-lane indicators |
| DEAD Grey | `#888888` | Comments · dead code · DEAD-lane |
| Arabic Cream | `#E8D5A3` | Arabic text passages (RTL support) |

---

## HeptaScript Language Support

HeptaScript (`.hpta`) is the sovereign query language of EnkiDB.
DubSar IDE provides:

- **Syntax highlighting** — ORBIT, KAKI, TRIBE, AT EPOCH keywords
- **Bracket matching** — nested orbit expressions
- **Snippet library** — common query patterns

**Example HeptaScript file** (recognised by the extension):

```heptascript
-- Query: find all GEM particles in the Najaf tribe at a historical epoch
ORBIT particle FROM tribe("najaf")
  WHERE quality(B11) >= GEM           -- B11 ≥ 200
  AT epoch(2026-01-01)
  PROJECT StoryEngine.state_at(epoch)
  RETURN particle.kaki, quality.lane, geo.h3_index
```

---

## Recommended Companion Extensions

Install these alongside DubSar IDE for the full sovereign development experience:

| Extension | Purpose |
|-----------|---------|
| `rust-analyzer` | Full Rust language server (IntelliSense, type hints, refactoring) |
| `Even Better TOML` | Cargo.toml editing with schema validation |
| `CodeLLDB` | Native debugger for Rust binaries |
| `Error Lens` | Inline compiler errors (pairs well with Manifold Dark colours) |

---

## Extension Structure

```
crates/dubsar-ide/
├── Cargo.toml              ← Rust library (native DubSar logic)
├── src/
│   ├── lib.rs              ← DubSarIde struct — orchestrates all panels
│   └── ide.rs              ← Panel definitions: HeptaScript, Particles, ETL, Story
├── themes/
│   └── enkidb-manifold-dark.json   ← VSCode theme JSON
├── syntaxes/
│   ├── heptascript.tmLanguage.json ← HeptaScript TextMate grammar
│   └── akkadian.tmLanguage.json    ← Akkadian DSL grammar
├── package.json            ← Extension manifest (contributes, activationEvents)
├── README.md               ← This file
└── CHANGELOG.md
```

---

## Architecture — Road to DubSar Desktop

```
Today (Extension):
  VSCodium / VS Code
  └── dubsar-ide extension
      ├── EnkiDB Manifold Dark theme
      ├── HeptaScript syntax
      └── KAKI hover inspector

v1.0 (DubSar Desktop — Rust + WASM):
  eridu-runtime (Layer 10)
  └── dubsar-ide (Layer 11)
      ├── dubsar-visualizer  ← Orbital particle Canvas 2D
      ├── enkidb-engine      ← Embedded query runner (no server needed)
      ├── heptascript REPL   ← Live query evaluation
      └── story-engine panel ← Time-travel diff view
```

The desktop version will be built on the same Rust codebase as the rest of
BahyWay.Ecosystem — compiled to native binary via `eridu-runtime`, with a WASM
fallback for web deployment.

---

## The DubSar Philosophy

> *DubSar — The Scribe — was the keeper of sovereign records in ancient Sumer.*
> *Every tablet he wrote was immutable. Every seal he applied was permanent.*
> *The IDE is his modern incarnation: a tool that helps you write sovereign data.*

The DubSar IDE enforces the same principles as EnkiDB itself:

- **No hidden state** — every editor action is traceable
- **Identity first** — KAKI awareness built into the language server
- **Quality visible** — orbital scores shown inline, not hidden in logs
- **Bilingual by design** — Arabic RTL support is not an add-on, it is a requirement

---

## Sovereign Constants Recognised by the IDE

The extension highlights violations of sovereign constants as warnings:

| Constant | Value | If Violated |
|----------|-------|-------------|
| `QUALITY_DIVISOR` | `240` | Warning: "Non-sovereign divisor detected" |
| `TAU_R7` | `11` | Warning: "Rule 7 threshold tampered" |
| `RESONANCE_RADIUS` | `0.15` | Warning: "SPH kernel deviated" |
| `PARTICLES_PER_TRIBE` | `7` | Warning: "Ibn Wahshiyya dimension count wrong" |

---

## 17 Forbidden Operations — IDE Enforcement

DubSar IDE detects and flags the 17 Forbidden Operations (ADR-008) at edit time:

```
F01  DELETE on Identity particles            → Error lint
F02  UPDATE on Event-KAKI fields             → Error lint
F03  KAKI nucleus mutation                   → Error lint
F04  B11 stored in KAKI nucleus              → Error lint
F05  Quality lane stored in κ               → Error lint
F09  External database calls                 → Warning lint
F12  QUALITY_DIVISOR ≠ 240                  → Warning lint
F13  TAU_R7 ≠ 11                            → Warning lint
```

---

## Contributing

This extension is part of **BahyWay.Ecosystem v4.0** — a proprietary sovereign
data system. Contributions require agreement to the sovereign development protocol:

1. All code must be pure Rust (where applicable) or TypeScript (extension manifest only)
2. No `unsafe {}` in logic crates
3. No third-party IO runtime dependencies
4. All new HeptaScript keywords must be approved via ADR process
5. Arabic support must be tested before every release

---

## Licence

Proprietary — BahyWay Sovereign Ecosystem
© 2026 Bahaa Fadam

See `LICENSE` in the repository root.

---

## Links

| Resource | URL |
|----------|-----|
| Repository | https://github.com/bahyway/enkidb |
| BahyWay Website | https://www.bahyway.com |
| HeptaScript Docs | https://www.heptascript.com |
| Open VSX Registry | *(coming soon)* |

---

*𒀭𒁾𒊭 DubSar — The Scribe writes in sovereign ink.*
*دوبسار — الكاتب يكتب بحبر سيادي.*
