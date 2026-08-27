# GL-LIT-001 · Corpus Registration 002 — The Stochastic Calculus Shelf
## Spieksma «Background Notes, Stochastic Processes» (Leiden) + Spreij «A Crash Course in Stochastic Calculus» (UvA)

**Court:** Book Court (GL-LIT-001) · structure-only pointers, no prose reproduced
**Status:** All concepts minted **FUZZY** (born-FUZZY law) · witness_2 pending per concept
**Author of registration:** DUB.SAR 𒁾 · Šala rehearsal: shala_stochastic_shelf.html

---

## 1. Corpus BN — Spieksma, Background Notes (lapis)

| Node | Pointer | Structure (short standard form) |
|---|---|---|
| σ-algebra · π-system · d-system | BN §1, §3 | Dynkin: d-system ⊇ π-system ⇒ ⊇ σ(π) |
| Path-space σ-algebra | BN §2 | 𝓔^T = σ(cylinders); only countably many time-instants observable; C[0,∞) ∉ 𝓔^T |
| Measurable maps · right-continuity | BN §3 | right-continuous ⇒ B(ℝ₊)/𝓔-measurable |
| Multivariate normal | BN §4 | X = a + BZ; Σ = B diag(σ²) Bᵀ; char. fn e^{i(θ,a)−θᵀΓθ/2} |
| Progressive · stopping times · F_τ | BN §5 | non-progressive example; hitting time τ_x is a stopping time |
| Convergence of measures | BN §6 | Portmanteau; a.s. ⇒ P ⇒ D; Itô–Nisio |
| Conditional expectation | BN §7 | Kolmogorov via Radon–Nikodym; tower; Doob–Dynkin E(X\|Y)=h(Y) |
| Discrete hedging (gambling) | BN §7, L7.6 | Y − EY = Σ αⱼηⱼ, α predictable — discrete representation |
| Uniform integrability | BN §8 | UI ⇔ L¹-bounded + uniform smallness; {E(X\|𝓐)} is UI |
| Augmentation · usual conditions | BN §9 | 𝓖_t = σ(F_{t+}, 𝓝); supermartingale survives augmentation |
| Functional analysis | BN §10 | Hahn–Banach · Riesz representation · Banach–Steinhaus |

## 2. Corpus CR — Spreij, Crash Course (copper)

| Node | Pointer | Structure |
|---|---|---|
| Filtered space · usual conditions | CR §2 | right-continuous 𝔽 + null sets; adapted/progressive; modification vs indistinguishable |
| Brownian motion | CR §3 | W₀=0, indep. N(0,t−s) increments; Donsker; paths nowhere diff.; Σ\|ΔW\| → ∞ but Σ(ΔW)² →ᴾ t |
| Martingales · Doob–Meyer | CR §4 | X = M + A, A predictable increasing; ⟨N⟩; Poisson: [N] ≠ ⟨N⟩ |
| Stochastic integral | CR §5 | isometry ‖I(X)‖ = ‖X‖_M; ⟨I(X),N⟩ = ∫X d⟨M,N⟩; chain rule |
| Local martingales | CR §6 | localizing stopping times T_n ↑ ∞ |
| Itô rule | CR §7 | df(X) = f′dX + ½f″d⟨X⟩; Doléans 𝓔(M) = e^{M−½⟨M⟩}; Lévy: ⟨M⟩_t = t ⇒ M = W; product rule |
| Representation theorem | CR §8 | Brownian M ⇒ M_t = ∫₀ᵗ Y dW |
| Girsanov | CR §9 | Z = 𝓔(L), Novikov; W̃ = W − ⟨W,L⟩ Brownian under P̃ |
| SDE strong/weak | CR §10 | Lipschitz ⇒ strong uniqueness; Picard iteration; Tanaka dX = sgn(X)dW weak-not-strong; **OU: dX = θ(μ−X)dt + σdW** |
| Feynman–Kac | CR §11 | v_t + 𝓐v = kv − g, v(T,·)=f ⇒ v = E[f(X_T)e^{−∫k}+…] |
| Black–Scholes | CR §12 | martingale measure + representation ⇒ hedge; BS formula |

## 3. Unification candidates (gold — shared context, Cross-Corpus Clause)

- **USUAL CONDITIONS**: BN §9 ≙ CR §2 — same closure, two registers (construction vs use).
- **PROGRESSIVE MEASURABILITY**: BN §5 ≙ CR §2 — pathology vs criterion (adapted + right-cont ⇒ progressive).
- **STOPPING & LOCALIZATION**: BN §5 ≙ CR §6 — τ_x measurable ↔ T_n localizing.
- **REPRESENTATION**: BN L7.6 (discrete, predictable bets) ≙ CR §8 (Brownian, ∫Y dW) — one theorem, two clocks. *Strongest mint candidate on the shelf.*
- **CONDITIONING → MARTINGALE**: BN §7 feeds CR §4 (tower property is the martingale property's engine).

## 4. BahyWay bindings (Optional-EAV harvest; spine untouched)

- CR §10 **OU** → GL-NSR-001 MARKASU (the mooring model, verbatim).
- BN §5 **hitting time τ_x** → GL-ALG-003 Abūbu first-passage T\*.
- CR §3/§4 **quadratic variation** → huburu (ḫb) calibration: ⟨W⟩_t = t is the noise-unit's meter-bar.
- CR §11 **Feynman–Kac** → NinurtaEngine truth tier (PDE ↔ expectation bridge).
- CR §9 **Girsanov** → measure-change as decree: the Madanu analogy is registered as *metaphor only*, not law.

## 5. Two-witness standing

Every node above: **witness_1 = the book's statement (pointer)**. witness_2 must be a Lean4/Z3 proof or a property-tested Rust kernel. **BN and CR agreeing is one witness twice** — the shelf's own §3 nodes demonstrate the trap. The Šala rehearsal runs *rehearsal* witnesses (seeded simulations: Σ(ΔW)² → T; Itô correction = t) that gate nothing; GOLDEN only via the ladder.
