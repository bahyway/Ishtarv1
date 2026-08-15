# PB-221 — HeptaScript Scale Sweep: Real Findings

**Architect:** DUB.SAR 𒁾 Bahaa Fadam
**Date:** 2026-07-21
**Ask:** "I want to see this time my HeptaScript Query running 1M Particles in <1Sec and then 10M in <1Sec .. 100M in <1Sec till 1Billion Particles... otherwise we need to discover why it is not up to the assumed score."

This is that discovery, done honestly: what was actually measured, what was
actually broken, what was fixed, and what still needs the real
infrastructure to prove at full scale.

---

## 1. What was true before this pass

Point/needle lookups (`WHERE <attr> = <value>`, one matching entity out
of N) were already fast and flat at every scale tested — proof the
underlying index design (`enkidb-indexes::EavExactIndex`, `enkidb-
datafile`'s O(log n) binary-search Data Files) works exactly as designed:

| N | needle query (1 match) |
|---|---|
| 1,000,000 | 30–62 µs |
| 10,000,000 | 38–107 µs |

Flat, not growing with N. This part of the three months' work was already
real and already correct.

## 2. What was broken

A **broad** query — one matching a large fraction of the corpus, the
realistic "browse a category" shape — was nowhere near `<1s`, and got
*worse* than linearly as N grew:

| N | matches | unbounded query time |
|---|---|---|
| 1,000,000 | 200,000 | 1.95s |
| 10,000,000 | 2,000,000 | 26.4s |

**Root cause, found by reading the actual code, not guessing:**
`ReadNode::query_parsed` (`crates/enkidb-readnode/src/readnode.rs`) uses
the index correctly to narrow candidates to exactly the matching posting
list, then fetches **every one of those candidates' full history blob
from disk, individually**, before `execute_over_histories` (which
already has its own `HOW_MUCH LIMIT` early-exit optimization) ever runs.
For a query matching 200K–2M rows, that is 200K–2M individual O(log n)
disk reads paid up front — real I/O cost that no query-planning
cleverness removes, because the code fetched everything before the LIMIT
had any chance to apply.

A second, independent bug was found while writing tests for the fix:
`execute_over_histories` itself (`crates/heptascript/src/engine.rs`)
applied its `HOW_MUCH LIMIT` early-exit **before** the `HOW BY <attr>`
sort. Combining a sort with a limit — e.g. "top 3 by rank" — silently
returned an arbitrary 3-row subset sorted among itself, not the true
top-3 of the full match set. This is a correctness bug, not just a
performance one; it would have shipped a wrong answer to any HOW+LIMIT
query, at any scale.

## 3. What was fixed

- **`crates/enkidb-readnode/src/readnode.rs`** — new `safe_early_limit()`:
  when a query's *entire* filter is one indexed equality condition (no
  extra WHERE, no WHY, no HOW sort, verb is ORBIT), the posting list
  itself already *is* the complete, correct answer set, so it's safe to
  truncate to the LIMIT before the expensive per-row fetch loop runs, not
  after. Any other query shape is left untouched — same correct, if
  slower, behavior as before.
- **`crates/heptascript/src/engine.rs`** — the early-exit `.take(n)` now
  only applies when nothing downstream needs to rank the full candidate
  set (`query.how.is_none()` and not `HOW_MUCH TOP`), fixing the
  HOW+LIMIT correctness bug.
- 4 new tests in `enkidb-readnode` proving: the fetch truncates correctly,
  it's a no-op when fewer rows exist than the limit, it correctly
  *declines* to truncate when a HOW sort is present (and the true top-N
  comes back right), and it correctly declines when a second WHERE
  condition is present (no undercounting). All 194 existing heptascript
  tests and 15 enkidb-readnode tests still pass.

**Result, identical data, identical query, only the fix applied:**

| N | matches | before | after | speedup |
|---|---|---|---|---|
| 1,000,000 | 200,000 (LIMIT 1000) | 1.95s | 10ms | ~195× |
| 10,000,000 | 2,000,000 (LIMIT 1000) | 26.4s | 30ms | ~880× |

Confirmed twice: once via `enkidb-readnode`'s own `scale_benchmark`
example (headless, `cargo run --release`), and again end-to-end over a
**real TCP connection** to a live `enkidb-write-server`/`enkidb-read-
server` pair running in this sandbox (SEED:1000000 → sync → QUERY):
44ms for the LIMIT-1000 query, 3.75ms for the needle lookup, both at
1M particles, both real client-perceived latency including the network
round trip.

## 4. What "show it in <1s" honestly means at 100M–1B

No visualization — DubSar's or anyone else's — can render or transfer
literally a billion individual rows across a network in under a second.
That's network and serialization physics (a JSON row is realistically
50–100 bytes; a billion of them is 50–100GB, which cannot cross even a
10Gbps link in under 5 seconds, before any rendering happens), not a
HeptaScript limitation — no real database does this either. The correct
and honest target, which is what was actually fixed and measured above,
is: a `HOW_MUCH LIMIT`-bounded page renders fast regardless of how many
rows *would* match, and a point/selective lookup stays flat regardless of
corpus size. That is what DubSar's Grid & Orbit view (`theater_3d.gd`,
new "EnkiDB (7001)" target) now actually does.

## 5. What was deployed

The core EnkiDB (type 7001) operational particle store had **no**
deployed server at all before this pass — only the old, never-deployed
`enkidb-query-server` (a different, incompatible wire protocol,
confirmed dead in an earlier session's investigation). This pass built
and deployed a real CQRS pair:

- `enkidb-write-server` (new `bin/` crate, port 7011) — `SEED:<n>`
  (generates n synthetic particles, the same generator
  `scale_benchmark.rs` uses, then materializes) and `FLUSH`.
- `enkidb-read-server` (new `bin/` crate, port 7001 — the port every
  existing client already expected) — `QUERY:<heptascript>`, same
  verb-aware JSON wire shape as `enkiddb-read-server`/`enkimdb-read-
  server`.

Both were live-tested in this sandbox end to end (seed → sync → query
over real TCP, see §3). `theater_3d.gd` (DubSar's Grid + Orbit view) now
has a real "EnkiDB (7001)" target alongside EnkiDDB/EnkiMDB, with
`HOW_MUCH LIMIT`-bounded default/preset queries.

## 6. What still needs the Architect's real infrastructure

This sandbox's disk allowance (~12GB free at the time of this work) does
not fit a materialized 100M or 1B particle store. **1M and 10M were run
for real, here, with real numbers** (§3). 100M and 1B need real hardware.

`playbooks/playbook_221_enkidb_core_deploy_and_scale_sweep.yml` is the
real, executable playbook for that: it deploys the EnkiDB CQRS pair onto
the real 2-VM split (`enkidb-node-write`/`enkidb-node-read`, the same
topology PB-212 already established for EnkiDDB/EnkiMDB), then runs the
full 1M/10M/100M/1B sweep for real, using PB-213's own cross-host
sync mechanism (not a direct write→read SSH, which the fleet's own
inventory documents as not existing), printing every N's real SEED time,
LIMIT-1000 query time, and needle-lookup time together as the actual
evidence. Run it with:

```
ansible-playbook playbooks/playbook_221_enkidb_core_deploy_and_scale_sweep.yml
```

Override `enkidb_seed_sizes` (e.g. `-e '{"enkidb_seed_sizes": [1000000, 10000000]}'`)
to skip 100M/1B on a volume not sized for them yet.

## 7. Honest bottom line

The three months of indexing work behind the point-lookup path was
already real and already correct — flat at any N, proven again here. The
"show a broad result set in under a second" path had a genuine,
now-fixed, now-tested bug that made it 195–880× slower than it needed to
be, plus an independent correctness bug in HOW+LIMIT combinations that's
also fixed. With both fixed, 1M and 10M are proven in this sandbox with
real numbers; 100M and 1B are one real playbook run away, on hardware
this sandbox doesn't have.
