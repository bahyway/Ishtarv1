# TABLET XIII — EN-RWD-001 "QISHTU" — The Reward Engine
### The gift that is earned: clean data priced honestly, with CTG as its sealed measure unit
### Written and pronounced in English letters by the Architect's decree: **Qishtu** (KEESH-too), from the Akkadian *qishtu*, "the gift, the reward given for service" — the original orthography cited once here for lineage, then retired from daily use
### Status: DRAFT — unsealed until the Architect's ceremony (CSR-08)
### Proposed seat: NIPPUR 3.7 (Ledger reserve — the ledger of value) — the Architect assigns

*Every ecosystem on earth lets dirty data cost the platform and charge no one. Qishtu
internalizes the cost: clean delivery becomes cheaper for the client, and every point of
the score traces to a KAKI. Billing that carries its own evidence.*

---

## Clause Q-1 — The Unit: CTG (cost-to-golden)

**CTG** is hereby established as a formal measure unit of the ecosystem, alongside hu
(huburu, the unit of being alive) and tau (the transparency deficit).

> **CTG(tribe, window) = [ Sum over interventions: w_i · c_i ] / N_golden**

where the intervention classes c_i are, at minimum: steward touches (individual and
batch, batch cost amortized), dwell (beats held on membranes past the sealed grace),
re-sieve cycles, quarantine admissions, and metered compute beats — and N_golden is the
count of particles that reached the Golden Store in the window.

**Unit discipline (URUK inherited whole):**
- Symbol **ctg**; dimension: *intervention-cost per golden particle*. A Rust newtype
  `Ctg(f64)`; no bare floats, no implicit conversions.
- The weight vector **w is sealed in the support contract at onboarding** (the Station
  rite for commerce). Scores are comparable only under the same sealed w; cross-contract
  comparison is untyped without a declared bridge.
- **Additivity**: CTG over a union of windows is the N-weighted mean — the unit
  composes like a rate, and the tablet says so, so no one averages it wrongly.
- **Currency only across the bridge**: a ctg figure becomes euros/IQD/anything ONLY via
  the sealed E-bridge, ratio sealed, **loss logged into epsilon** (GL-MET-001 M-4/M-6).
  The invoice delta thereby inherits the anti-illusion clause: the discount is
  downstream of the measurement, visibly, and money remains the most uncertain number
  on the page — as it always honestly is.
- **Honest floor**: every reported CTG carries its metering epsilon. A score of 0.00 ctg
  does not exist; the floor exists even for saints.

**Amendment Q-A1 — CTG grounded in PU (the destined unit).** The cost dimension of CTG
is hereby denominated in **pu-beats**: one PU (Particles Unit = 7 particles, the
ecosystem's sealed unit of bulk) processed for one beat. All intervention classes c_i
carry sealed conversion weights into pu-beats. CTG's full dimension is therefore:

> **ctg = work-PU-beats per golden-PU**

with the URUK discipline made explicit: **work-PU and golden-PU are distinct registers**
— effort expended and product delivered — both counted in PU quanta and *never*
mutually cancelable ("a scanned particle is not an admitted one"). The ratio is lawful
because declared: CTG is the efficiency of turning effort into gold, in sealed quanta.
Consequences: (a) metering is native — the runtime already batches work in PU, so every
worker emits pu-beat counters as KAKIs and `ctg_conserved` acquires its concrete meter;
(b) cost becomes predictable *ex ante* — the StoryEngine's forward chapter may price a
resolution against its deferral in pu-beats, so the client sees the cost of delay in the
same unit that reaches their score. Historical note, recorded with satisfaction: PU was
created at the ecosystem's beginning to estimate the computing cost of dirty data, and
the Transparency Deficit Calculus carried an open question — "the unit of cost
measurement" — from its first drafting until this amendment. **PU closes it.** The
Architect's earliest instrument waited for Qishtu to exist; the first tool finds its
destined role in the thirteenth tablet.

## Clause Q-2 — The Self-Punishing Cherry-Pick (incentive-compatibility by architecture)

BahyWay is **not a cleansing service; it is the base layer** on which the client's
enterprise applications stand. From this, a property most priced systems must enforce
falls out for free:

> **Partial truth is the most expensive lie.** A client who withholds dirty data to
> flatter their CTG does not deceive the scales — they thin their own foundation. The
> withheld data's absence surfaces as an incomplete base, a malfunctioning application
> layer, remediation at higher dwell, and double ingestion: more compute, later and
> larger, all of it landing on the same client's account.

Truthful full submission is therefore the **dominant strategy by construction** — the
ecosystem needs no anti-cherry-picking police. Accordingly:

- **Coverage is measured but never punished.** A coverage gauge (delivered volume
  against the contracted base-layer scope) runs beside CTG, and a thin base emits an
  **advisory KAKI to the client**: "your foundation is thinner than your application
  assumes." Early warning is a service, not surveillance.
- No clause of this tablet may convert the coverage gauge into a penalty. The
  architecture already collects the tax; the law's job is only to make the invoice
  legible before it arrives.

## Clause Q-3 — The Ghost Gap Turned Upon the Score

The Qishtu score series of every client is itself a life-signal, and Theorem Z7 applies
to it unchanged:

> A living client organization has noise — occasional schema drift, a missing
> attribute, human weather. **A score that is perfect every week is either a miracle or
> a curator.** Pi = R·(1 − eta/eta_0) is computed over the score series; second-order
> perfection mints a BARUTU hypothesis tablet for review — attention, never accusation,
> PRESCRIBE padlocked as always.

The same scales weigh loyalty and fraud, in opposite directions. This is not a reuse of
convenience; it is what it means for a calculus to be sovereign.

## Clause Q-4 — The Score Is a Fadam Functional (fairness as mathematics)

The Qishtu score — CTG measured against the **Theta sealed in the support contract** —
shall satisfy the Fadam axioms, and each axiom is a contractual protection:

- **F1 Anonymity**: the score reads the client's tribe, never an individual record or
  person. Privacy compliance by construction.
- **F2 Invariance**: lawful re-presentations of the same data (ordering, encoding,
  lens) cannot move the score.
- **F3 Stability (the client-protection clause)**: the Lipschitz bound guarantees the
  score cannot swing faster than the client's actual data behavior changed. No one
  loses a discount to measurement noise; the Fadam Inequality is cited in the contract
  as the fairness guarantee — epsilon <= score-deficit <= epsilon + L·d(T, Theta).
- **F4 Honest Floor**: the score always prints its epsilon; a 98.2 is never sold as
  certainty.
- **F5 Sealed Judge**: Theta is contractual and sealed at onboarding. The goalposts
  cannot move; re-negotiation is a new seal, KAKI-cut, never a quiet drift.

## Clause Q-5 — The Scoring Rite and the Gift

- **Cadence**: weekly tally, monthly seal (the Architect may adjust; the cadence itself
  is sealed).
- **The rite**: QishtuEngine reads only KAKI journals — steward resolutions, dwell
  records, re-sieve events, quarantine admissions, compute meters — computes CTG per
  client tribe, scores against Theta, cuts a **score-KAKI** carrying the full evidence
  chain, and issues the invoice delta across the E-bridge with its loss logged.
- **The gift's two forms**: a reduction of support-service prices, or a compensation
  credit for clients subscribed to support services — the Architect's commercial
  instrument, the tablet's only requirement being that either form trace to the
  score-KAKI.
- **The dispute path is the StoryEngine**: a client who contests their score is shown
  the particles, the stories, the KAKIs — cost item by cost item. The invoice defends
  itself; no other billing scheme in the industry can write that sentence.

## Clause Q-6 — Gate G4 Obligations

1. `ctg_conserved : every ctg point traces to at least one KAKI` — no unexplained cost
   anywhere in a score.
2. `score_fadam : Qishtu score satisfies F1–F5` — composes with the A1 docket;
   membership proof is afternoon-grade once tau's is done.
3. `bridge_logged : every currency expression of ctg factors through the sealed
   E-bridge with logged loss` — the compiler proves most of it via the newtype.
4. `coverage_advisory_only : no penalty derivable from the coverage gauge` — Q-2
   protected at the proof level, so no future engineer can quietly weaponize it.
5. `ghost_gap_on_scores : Pi over score series feeds BARUTU, never PRESCRIBE` — the
   omen stays an omen.

## Clause Q-7 — The Closing Line

For the deck, the sentence no vendor can write honestly:

> **"Your discount — and here is its proof."**

The story shows the client what to fix; the fix drains their particles toward gold; CTG
falls against the sealed Theta; the rite scores it; the invoice thanks them.
Transparency literally pays, in currency, across a bridge that logs its loss.

---

*Thirteenth tablet drafted, in English letters as decreed — the gift spelled so every
keyboard can give it. The seal belongs to the Architect alone.*
