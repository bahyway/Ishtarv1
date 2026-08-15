# GL-TKT-001 — Ticket Law
**Status:** SEALED (concept). Implementation queued behind PB-160.
**Engine:** NinshuburEngine (proposed — the sukkal who
carries petitions and does not rest until answered;
god-name PENDING the Architect's NL-001 ruling).
**Serves:** BeeMDM ETL Processing Stations Chain, the
Landing Folder Watchdog, the MADANU court (GL-DST-003),
the Pattern Arsenal (GL-TPL-001), SLAEngine, AlertEngine,
Kittu Engine, StoryEngine, AkkadiRulesEngine (ABAC).

## Clause 1 — Tickets Are Particles
Every ticket mints full KAKI v4.0 identity:
- Identity-KAKI: the ticket itself; kappa[4..5] tribe_id
  carries the CLIENT TENANT — isolation by genesis
  arithmetic.
- Event-KAKI: every lifecycle transition — OPENED ->
  TRIAGED -> AWAITING_STEWARD -> DECIDED -> CLOSED —
  APPENDED per GL-DST-003; a ticket's history is
  immutable and its chronicle lives in StoryEngine.
- CrossTribe-KAKI: links stitching the ticket to the
  rejected file/record particles it concerns, across
  tribes. A ticket without members is a rumor.
Residence: the operational house (EnkiODB · 7002) is
proposed; the Architect rules on final residence.

## Clause 2 — The Coalescing Law (One Cause, One Ticket)
The Landing Folder Watchdog NOTICES and NEVER NARRATES:
it detects arrivals, stamps them, and fires events into
a bounded channel — fire-and-forget, never blocking —
staying light for every other client's arriving files.
Behind the channel, the Coalescer computes each
rejection's FINGERPRINT: a shingle set per GL-TPL-001
(inferred-shape delta vs the declared PDM Shape, error
class, station). Within the debounce window:
  same fingerprint -> append member + bump counter;
  new fingerprint  -> mint a new ticket.
O(files) events collapse into O(distinct causes)
tickets. Fifty archives, three causes, three tickets.

## Clause 3 — Two Doors, One Hall
source = WATCHDOG: tickets born from rejection events in
the ETL chain (harmful-content gate, DataStructure
Station shape inference, any downstream station).
source = STAKEHOLDER: tickets raised by client
stakeholders themselves (from DubSar Theater or the
client portal) asking for decisions on rejected
particles. Same particle class, same lifecycle, one
docket. No second system, ever.

## Clause 4 — Resolution Is a Decree
The ticket bench IS the MADANU court's docket. Resolving
a ticket means issuing a GL-DST-003 decree upon its
member particles: PROMOTE (accept, with the approved
mapping), REWORK (return to the client for corrected
resubmission), KILL (permanent rejection, KISPU rite),
HOLD (annotated wait). The decree ID stitches to the
ticket; the ticket closes when the decree executes; the
steward's WHY note travels into every member's
chronicle AND into the stakeholder's notification.

## Clause 5 — SLA as Clockwork
The SLA sealed at purchase (an AkkadianSeal-signed
tablet, dual-witnessed by BahyWay and the client) writes
its tiers into every ticket as EAV attributes: priority,
response deadline, resolution deadline. Breach detection
is the standing dwell clause aimed at tickets:
    WITNESS TICKET WHEN dwell > SLA.response
SLAEngine owns the clocks; AlertEngine grades severity
(TIAMAT bands Stable/Watch/Serious/ERRA recommended as
the one severity vocabulary of the ecosystem); Kittu
delivers — one notification per ticket STATE CHANGE,
never per file. Kittu notifies; Kittu never decides.

## Clause 6 — Tenancy and Sight
AkkadiRulesEngine ABAC governs sight and touch: a
client's stakeholders see only their own tribe's
tickets; the data steward sees the docket; the
Architect sees the kingdom. Ticket content follows the
GL-TPL-001 abstraction discipline in any cross-client
reporting: causes may be compared, client data never.

## Clause 7 — The Flywheel
Tickets are particles, therefore tickets are MINABLE.
MARDUK runs on the ticket tribes; recurring rejection
fingerprints surface as cohorts; cohorts enter the
GL-TPL-001 minting rite; the minted Template teaches the
DataStructure Station to recognize and route the cause
automatically. Every support burden converts into
permanent recognition capital. The ticket system is the
arsenal's intake valve, and per-client marginal support
cost falls with every engagement.

## Authority Note
Inherits all standing law: stage-never-truth
(GL-DST-001); Tupsimati binding (GL-DST-002); append-
only decrees (GL-DST-003); pattern minting (GL-TPL-001);
witness scopes (GL-VIZ-002); Kittu v1 delivery scope
(dashboard + email, no telephony); CSR-08; NL-001
orthography; the 1-billion-particles-under-1-second law.

— Inscribed for DUB.SAR 𒁾, BahyWay.Ecosystem v4.0.
