# GL-SEG-001 · Tablet of Segment Shape (draft, unsealed)
Proposed 2026-08-25 · DUB.SAR 𒁾
Depends on: GL-UNT-001 (RU, MLU), GL-FLD-001 §1 (amplitude is density), GL-LYF-001 (layer life),
GL-VIZ-002 §8–§9 (render grammar, the unknown is drawn), KAKI v4.0 byte layout (locked)

---

## §1 Watching is not preventing
The Layer Life Court **measures** decay; it does not stop it. Fragmentation and page splits are not cured by
observation — they are caused by **write order**. This tablet governs the order, and the instruments only report
whether the order held.

## §2 The cause, named plainly
`κ[0..3]` of a KAKI is a **uuid_hash**. If the physical clustering key is that hash, every insert lands on a random
page: maximum split rate, maximum free-space fragmentation, and an orbit scan that touches as many extents as it has
citizens. **A hash may be an identity and a lookup key; it may never be the physical order.**
The physical order is `(κ[4..5] tribe_id, κ[12..13] timestamp)` — monotonic within a tribe, so inserts append.

## §3 Shape is locality, not tidiness
The measure of a segment's shape is **not** fragmentation percent. It is the correspondence between *logical*
adjacency (citizens that orbit together) and *physical* adjacency (blocks that read together):

> `L = extents touched by one orbit scan ÷ extents strictly required`

`L = 1` is a segment whose shape matches its ontology. `L = 40` means a layer's address no longer describes where its
citizens live, and the I/O cost of every WITNESS is forty times the honest cost. `L` is the number to publish, and it
is `MEASURED`.

## §4 The field is already a zone map
The ENLIL density field over `(MLU, θ, RU)` is a **coarse index the estate already computes**. A bin maps to an extent
range; a PRESENT bounded by RU and MLU therefore needs no B-tree at all. Prefer **zone maps and min/max per extent**
over secondary B-trees on high-cardinality EAV attributes — those indexes are the principal source of split storms at
10⁹, and each one added must justify itself against the field it duplicates.

## §5 Seven types, seven write shapes — one policy each is wrong
| type | write shape | policy |
|------|-------------|--------|
| `EnkiSDB·7001` | append-heavy, short retention | log-structured; large extents; **no secondary indexes**; expire by dropping a partition, never by DELETE |
| `EnkiODB·7002` | tiny, read-mostly, governed | fully cached; fill 100%; a changed definition is a new sealed particle, not an edit |
| `EnkiQDB·7003` | small, payload immutable, time-bounded | append; partition by `wasu.disposition` deadline so expiry drops a partition |
| `EnkiDB·7004` | the 10⁹ core; read-heavy orbit scans | cluster by `(tribe, epoch)`; **fill 100%**; zone maps only; append-only |
| `EnkiDW·7005` | immutable crossings, rebuilt | columnar segments, write-once; splits impossible by construction |
| `EnkiMDB·7006` | derived masks | **rebuild, never repair** — a fragmented mask is regenerated from 7004 |
| `EnkiDDB·7007` | sealed tablets | WORM; append-only; never reorganised |

## §6 A page split is an event, not a statistic
Every split emits an **Event KAKI** on the affected tribe: epoch, extent, cause (`insert-out-of-order`,
`update-grew-row`, `index-rebalance`). Splits are therefore visible in a biography and countable per tribe, not merely
aggregated into a percentage. A tribe whose split events cluster in one epoch has a write-order fault, and the court
can name the hour.

## §7 There is no UPDATE
BahyWay has no update path — not in the Golden Store, not in the Pre-Golden zone, not anywhere. A correction is a
**new particle** citing the prior KAKI; the prior particle remains exactly as written. This is not a preference to be
tuned; it is the shape of the ecosystem, and it has three storage consequences that follow with no further argument:

1. **Rows never grow.** Fill factor is therefore **100% in every type**. The customary 80–90% headroom exists only to
   absorb in-place row growth, and where no row can grow that headroom is pure waste — it inflates every segment,
   every read and every cache line for a hazard that cannot occur here.
2. **`update-grew-row` is not a split cause; it is a breach.** If a split ever reports it, something wrote in place.
   That is a violation of the ecosystem, not a tuning problem, and the court treats it as one.
3. **Only two split causes remain possible:** `insert-out-of-order`, which §2 eliminates by clustering on
   `(tribe, timestamp)`, and `index-rebalance`, which §4 eliminates by preferring the field over B-trees.
   With both closed, a correctly ordered EnkiDB segment splits **never** — not rarely.

## §11 Nothing that cannot fit a page ever enters a row
A hyperspectral cube, a SAR granule, a point cloud: none of these is a *value*. The rule of §7 is about **in-place
mutation**, not about size, and the two must not be confused. A payload larger than a page is not a grown row — it is
an object that never belonged in the row.

**First remedy, and the preferred one: the cube is a tribe, not a particle.**
A hyperspectral acquisition is tiled at `(band, tile)` grain and each tile is a **citizen with its own KAKI**, its own
RU/MLU address, its own biography. Nothing is oversized because nothing is monolithic; the field indexes the tiles as
it indexes any crowd, and a band read is an orbit scan whose locality is measurable by §3. The estate already thinks
this way about crowds — an image is a crowd of measurements, not a single fact.

**Second remedy, where a monolith must be preserved intact** (provenance, legal custody, a vendor SAFE archive):
the payload is stored **out-of-line** as an immutable, content-addressed chunk set, and the particle's row carries only
a fixed-width descriptor:

`content_hash · byte_length · codec · geometry (bands × rows × cols, dtype) · chunk_map_ref`

The row is therefore fixed width by construction and cannot grow. Chunks are written once and never rewritten, so they
append and never split. The `chunk_map` is itself fixed-size or spilled to its own object — an unbounded inline array
is the same mistake in a smaller coat.

## §12 Chunk along the access, and measure it
Chunking is not a storage detail; it is an ontological choice. Chunk a cube along `(band, tile)` because that is how it
is read, and a band read then touches contiguous chunks. Locality applies to blobs exactly as to rows:

> `L_blob = chunks touched by one band or tile read ÷ chunks strictly required`

A correction to an image is a **new particle with a new content hash**. Because chunks are content-addressed, bands
that did not change cost nothing to keep — the new version shares them. Nothing is updated; nothing is duplicated
without reason.

## §8 Thresholds and what they mean
`L > 4` — the shape has drifted from the ontology; schedule a reclustering rite.
`splits/epoch > 0` — with §2 and §7 in force the only admissible number is zero; any split is a fault to be named, not a rate to be tolerated.
`cause = update-grew-row` — a breach: something wrote in place. Escalate; do not tune.
`inline value ≥ page size` — a breach of §11: an object was put in a row. Tile it into citizens, or move it out of line.
`L_blob > 4` — the chunking does not match the access; rechunk, do not add an index.
`free-space fragmentation > 25%` with `L` still ≈ 1 — cosmetic; do **not** reorganise, it costs I/O and buys nothing.
Reorganisation is justified by `L`, never by tidiness.

## §9 What must be published per type
Beside the particle count: `L`, splits this epoch with their causes, write amplification, free-space fragmentation,
**and the extents nobody could read** (GL-VIZ-002 §9). A segment report without its unknown bucket is not admissible.

## §10 The instruments report; the playbooks act
PB-396 measures layer life, PB-395 reads the substrate, PB-400 applies the per-type policy and re-measures.
No dashboard reorganises a segment. Reclustering is a numbered rite with a Kanīku receipt.
