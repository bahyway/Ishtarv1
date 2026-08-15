# TPL-001 — Template Execution Plans & Journal Projection Cache
Status: SEALED DESIGN · Implementation queued behind testing gate

## A. Template Plans (design-time intelligence, runtime determinism)
1. A Template is a HeptaScript query compiled ONCE at design time:
   parse -> planner -> (optional) MUMMU/Z3 satisfiability
   proof -> AkkadianSeal (Ed25519, real: crates/kupru) over the
   plan bytes.
2. Registry lives in EnkiMDB. VERIFY before implementation: the
   real crates/enkimdb catalogs BahyWay's own artifacts
   (crates/playbooks scanned from the filesystem) — a different
   purpose than a general sealed-query-plan registry. Either
   extend enkimdb's scope deliberately or pick a different home;
   don't assume the fit without a design decision.
3. Admission law: a plan is sealable only if every access path is
   index-covered by real enkidb-indexes structures (confirmed
   real: EavExactIndex, HeptaShellIndex, NatiruIndex,
   RadixSplineIndex, SurrogateMap). A plan requiring raw journal
   scan is REJECTED at seal time.
4. Z3 placement law upheld: proofs at design time only; no solver
   in any shipped binary.

## B. Journal Projection Cache (derived EAV segments, read node)
1. Source of truth: append-only journal on enkidb-node-write.
   Projections are DERIVED EAV segment files on enkidb-node-read.
2. Projector applies events post-commit, in commit order. Real
   enkidb-indexes structures are built over *those segments* —
   never over the journal.
3. Segment header: {journal_watermark_offset, built_at, seal}.
   Divergence from journal => segment condemned + rebuilt.
   Journal always wins.
4. Freshness is provable: PROVE FRESH(watermark, delta_t_max)
   where delta_t_max comes from the client SLA.
5. Per-particle history — CONFIRMED GAP, not a verify point:
   crates/enkidb-journal has NO prev_event back-pointer today
   (checked directly, 2026-07-10). Full history walk is
   currently O(N), a full journal scan. Adding the back-pointer
   at projection time (deriving it without touching the locked
   journal format) is real, unstarted implementation work and
   the prerequisite for this section's O(k) claim.
6. The ONLY lawful full journal scan is the offline projector
   rebuild (maintenance mode). A runtime plan that would
   degrade to a journal scan should be refused at seal time,
   not silently executed.

## C. Federation note
Sealed Template plans may span engines: the fan-out and
KAKI-surrogate join are part of the sealed plan. See PB-159
(corrected) for what "spanning engines" actually means today.
<!-- BEGIN PB-159 §D -->
## D. Journal Sharing Reality (corrected 2026-07-10)
1. CONFIRMED (not assumed): StoryEngine does NOT persist in a
   separate EnkiODB journal. crates/story-engine/src/
   story_engine.rs imports enkidb_journal::Journal and
   enkidb_kaki::{IdentityKaki, EventKaki} directly — the same
   shared journal every other real engine (EnkiDB included)
   reads and writes. There is one journal, one watermark. No
   cross-engine skew scenario exists in the current
   architecture.
2. The federation law below (min-watermark freshness,
   FUZZY-on-skew) is kept SEALED as forward design, dormant
   until it applies: if a future architectural decision does
   split an engine (e.g. StoryEngine) onto its own journal,
   this is the law that governs the resulting join. It is not
   active law today because its precondition (two journals)
   does not hold.
3. Symmetry law (dormant): every federated engine follows the
   same CQRS shape (write=journal, read=projected segments).
   TPL-001 §A-§C apply per engine with no per-engine exceptions
   — once there is more than one engine to federate.
4. Join law: cross-engine joins occur ONLY on the KAKI surrogate
   pair (uuid_hash, tribe_id) — this part is unconditionally
   true today too, single-journal or not, since it's just the
   lawful identity comparison rule.
5. Min-watermark law (dormant): a federated result's freshness
   would be MIN(watermark_i) over all participating engines,
   the PROVE FRESH(., delta_t_max) predicate evaluating against
   that minimum. Inapplicable while there is one watermark.
6. Skew-window law (dormant): a joined row whose components
   would straddle watermark skew is returned FUZZY, never
   GOLDEN. Inapplicable today for the same reason.
7. Before this section is ever activated: re-verify against
   CAT-001 (or whatever the current architecture doc is) that
   an engine split has actually happened — don't reactivate
   §D.5/§D.6 from a recollection again without checking source.
<!-- END PB-159 §D -->
<!-- BEGIN PB-160 §E -->
## E. Federated State-Scope Law

NOTE: GOLDEN/FUZZY/DEAD is PROPOSED vocabulary (see PB-155
corrected) — this section describes the law that would apply
once/if that vocabulary is adopted, not a restatement of
something already real elsewhere.

1. Canonical states ONLY, if adopted: GOLDEN / FUZZY / DEAD.
   Render colours (GREEN, BLUE, RED, GRAY, GOLD...) are
   unrepresentable in any state predicate (HS-EXT-003's
   ColourAsState parse error). Truth Before Beauty: the
   cube/orbit colour is derived FROM the state, never the
   reverse.
2. Explicit scope at seal time: a Template that spans >=2
   engines MUST declare its state scope, e.g.
   WHAT STATE(GOLDEN, FUZZY). No default, no inference — a
   federated Template without an explicit state scope is
   REFUSED at seal time. Forgetting is a loud compile error,
   never a silently skewed result.
3. DEAD-inclusive Templates are audit/forensic class and
   witnessed on every execution, once a witness mechanism
   exists (NĀRU-equivalent).
3b. Health law: "healthy" is not a state — it is the
   continuous EAV attribute H that drives BIGRING orbit
   radius and ColourID B11 = round(H(P) x 240). The lawful
   predicate is a threshold, not a state word:
     WHAT STATE(GOLDEN, FUZZY) HEALTH(>= h_min)
   h_min should load from a governance record (e.g. a client
   SLA), never a hand-picked literal in the query — mirroring
   the eps_geo rule from HS-EXT-002.
4. Fuzzy-reason distinction: if/when §D's dormant skew law is
   ever activated, FUZZY can arise from two causes (data
   quality vs. projector skew) — PRESENT may expose a
   fuzzy_reason field so callers can tell them apart. Until
   §D activates, only the data-quality cause applies.
<!-- END PB-160 §E -->
