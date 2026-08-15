# 𒁾 TABLETS IX & X — CONTINUITY AND THE TWO STREAMS
### GL-OPS-001 "LAHMU–LAHAMU" — The Ledger Continuity Law (HA/DR for the EnkiDB Seven)
### GL-OPS-002 "URUK–KISH" — The Two Streams Doctrine (experimental / conservative)
### Playbook suite: PB-310 … PB-317 (numbering begins at 310 by the Architect's instruction, clear of all prior ranges)
### Status: DRAFT — unsealed until the Architect's ceremony (CSR-08)
### Proposed seats: GL-OPS-001 → NIPPUR 3.6 (Ledger — continuity of the ledger) · GL-OPS-002 → NIPPUR 7.6 (Sovereignty — release governance) — the Architect assigns

*Lahmu and Lahamu: the primordial twins, born as a pair before the ordered world.
Uruk: the city that invented writing and kept inventing. Kish: where kingship sat.
Continuity is a pair; releases are a throne.*

---

# TABLET IX — GL-OPS-001 "LAHMU–LAHAMU": THE LEDGER CONTINUITY LAW

## Clause L-1 — The Ledger Is the Truth; Replication Is a River

HA/DR in BahyWay is not a subsystem; it is the four river-laws pointed at ourselves.
The write node's append-only KAKI ledger is the sole source of truth, and its
replication to a standby twin is a **PALGU stream** like any other:

- **ADANNU** — replication lag *is* the watermark; the standby's settled frontier is
  displayed, never hidden. A failover before the watermark is a declared-loss event,
  printed in the promotion KAKI, never a silent one.
- **ARKÛ** — late ledger segments are lawful late arrivals: absorbed in adannu order on
  the standby, with projections re-derived and the ledger untouched.
- **MĪLU** — if the standby falls behind, the three-rung ladder applies: buffer within a
  proven bound, shed segments to the NUZI vault for replay, throttle the shipping cadence.
  **A dropped segment does not exist at any rung.**

The primary/standby pair is registered as the **Lahmu–Lahamu pattern** (NL-001-A2
petition: practices name patterns) — twins born together, one awake, one listening.

## Clause L-2 — Read Nodes Are Disposable by Doctrine

The read node (192.168.122.107) holds only projections, and ARKÛ already sealed the
principle: *projections move; the ledger never does.* Therefore the read node has **no
backup and needs none** — it has a rebuild rite (PB-312) that replays projections from
the ledger, with Gate G4's replay-determinism proof guaranteeing the rebuilt node is not
similar to the lost one but **equal** to it.

## Clause L-3 — Snapshots Are Sealed Artifacts

Point-in-time recovery = signed snapshot + ledger replay from the snapshot's watermark.
Every snapshot is Ed25519-signed under the same discipline as the Zagesi release
manifest; an unsigned snapshot is not a backup, it is a rumor. Snapshots and shed
segments live in the **NUZI vault** — and NUZI, the inward archival name, hereby earns
its off-host meaning: the vault's first home may be an encrypted external disk, but the
law requires it **off the host box**, because a vault inside the burning house is
furniture.

## Clause L-4 — The Seven Types, Tiered

| Port | Type | Role | Protection tier | RPO target |
|------|------|------|-----------------|------------|
| 7001 | EnkiSDB | source/staging | rebuildable from upstream | snapshot only |
| 7002 | EnkiODB | operational | ledger shipping | ≤ watermark lag |
| 7003 | EnkiQDB | **quarantine** — harmful, corrupt, or unknown-state particles held for judgment | sealed snapshots + NUZI vault (**evidence-grade**) | snapshot cadence + chain of custody |
| 7004 | EnkiDB | core graph ledger | **Lahmu–Lahamu continuous** | ≤ watermark lag |
| 7005 | EnkiDW | warehouse | rebuild from GOLDEN | snapshot cadence |
| 7006 | EnkiMDB | master data | **Lahmu–Lahamu continuous** | ≤ watermark lag |
| 7007 | EnkiDDB | **document database** — the queryable Graph RAG corpus | documents shipped & sealed (primary content); RAG indexes/embeddings rebuilt by rite | documents ≤ watermark lag · indexes none needed |

The tier column is the honest architecture: not everything deserves continuous
replication, and pretending otherwise is how small fleets drown. The ledger-bearing
types (7004, 7006, the 7002 stream, and **EnkiDDB's document corpus** — for a document,
like a ledger entry, is primary content no replay can regenerate) get the twins; the
projection types — including **EnkiDDB's RAG indexes and embeddings, which are pure
derivations of the corpus** — get rites; the staging type gets snapshots and a shrug. **The quarantine (7003) gets
neither a shrug nor a rebuild rite** — its contents are exactly the particles that
were refused or suspected at the gate, which means they are *not derivable from the
main ledger by replay*: lose the quarantine and you lose the evidence, the PARZU
investigation trail, and the ability to ever judge what knocked at the walls.
Quarantined particles are therefore sealed with the same dignity as the ledger itself —
signed snapshots to the NUZI vault on every admission batch, custody recorded as KAKI
lineage — because *the accused deserve a preserved record no less than the acquitted*,
and because tomorrow's BĀRÛTU calibration is mined from precisely this archive of what
once went wrong.

## Clause L-5 — Failover Is a Ceremony, Not a Reflex

Read-node failover is automatic. **Write-node promotion is a scripted ceremony**
(PB-313) requiring the Architect's seal: on a fleet of one host and a handful of VMs,
CSR-08 is the split-brain protection — no quorum protocol beats a single sovereign with
a checklist. The promotion KAKI records: watermark at promotion, declared loss (if any),
the operator, and the seal. The demoted twin, on return, rejoins as standby and replays
forward; it never resumes primacy by its own opinion.

## Clause L-6 — The Backup Muster (PIQITTU applied to ourselves)

**A backup that has never been restored is a ghost backup** — it passes every payday and
fails every muster. Therefore restore drills (PB-314) are scheduled, numbered, and
recorded as muster-KAKIs; a snapshot's liveness η is its restore history, and a vault
full of never-tested seals earns the same silent-perfect suspicion as a spaceman on the
payroll. The instrument that guards the nation's rations guards our own.

## Clause L-7 — The Conservation of Particles (the No-Loss Guarantee)

The Architect's principle, sealed as an invariant: **once data becomes a particle, it
cannot be lost — only located.** The reasoning is a chain of laws already sealed, now
stated as one:

1. **Total custody.** Every particle that passes the gate lands in **at least one** of
   the seven EnkiDB types. The seven form a *partition of custody* with no gap: the
   conforming enter the ledgers (7004/7006) and streams (7002); documents enter the
   corpus (7007); projections derive (7003→rites, 7005); the staged wait (7001); and —
   the keystone — even the **refused and the suspect are kept**, in quarantine (7003)
   and the NUZI vault, per PALGU P-4. There exists no ingest path whose terminus is
   nowhere.
2. **No silent subtraction.** MĪLU forbids drops at every rung; ARKÛ absorbs the late;
   KAKI is append-only, so no later event can un-write an earlier one. The particle
   population inside the walls is *monotone non-decreasing in the record*, even when it
   decreases in the world — for even a deletion request is itself a particle.
3. **Replay totality.** Because every particle has a persistent home and every
   projection is a deterministic derivation (ADANNU A-4, proven at Gate G4), **the state
   of the ecosystem at any adannu t is reconstructible**: take the seals at or before t,
   replay the ledger forward to t's watermark, re-derive the projections. Not restored
   approximately — *re-derived equally*.

**Theorem (Point-in-Time Totality).** Conservation (1–2) + determinism (3) ⟹ for every
past instant t inside the walls, there exists a rite that yields the ecosystem's state
at t, exactly. *Data loss is thereby not a risk to be minimized but an event the
architecture renders inexpressible — like the illusion unit, it does not type-check.*

**The two honest conditions** (stated so the theorem can be defended, not just admired):
- *The wall condition.* Conservation begins **at the gate**. The vulnerable interval is
  before and at ingest — the network hop, the write buffer not yet shipped. This is why
  the Lahmu twin listens continuously and the MĪLU buffer bound is a *proven* parameter:
  the theorem's jurisdiction starts where the KAKI is cut, and the engineering's job is
  to make that boundary as thin as physics allows.
- *The vault condition.* Conservation is a **logical** guarantee; physics can still burn
  a disk. The theorem therefore holds *conditional on Tablet IX being operated*: seals
  off-host (L-3), tiers honored (L-4), musters passed (L-6). Conservation is the law;
  Continuity is its enforcement; neither suffices alone, and together they are the
  guarantee the Architect stated.

**Gate G4 obligation (added to the docket):**
`conservation : ∀ ingest_path p, terminates_in_persistent_home p` — every path through
the gate provably ends in one of the seven custodies — together with
`pit_totality : ∀ t ≤ watermark, reconstructible (state t)`. The first is an exhaustive
case proof over the ingest graph; the second composes it with the existing
replay-determinism certificate. When both are green in Lean4, "we never lose data" stops
being a promise on a slide and becomes a checked property of the walls.

## Clause L-8 — The Incorruptibility Chain (the No-Lie Guarantee)

Conservation (L-7) guarantees a particle's **existence**; this clause guarantees its
**fidelity**. Loss is a particle vanishing; corruption is a particle *lying about
itself* — a bit-flipped KAKI is perfectly conserved and perfectly wrong. The two are
different laws, and each requires its own defense. The chain has four links, and its
governing rule is: **no single layer is ever the only witness.**

**Link 1 — The particle testifies about itself.** Every KAKI carries a content hash,
chained to its predecessor, cut **at the gate** — as early after arrival as physics
allows. Integrity is thereby detectable *by the ledger itself*, on any filesystem, on
any disk, forever — the same move as the Honest Floor: do not trust the substrate's
honesty; make honesty structural. Every read and every replay verifies the chain; a
replay that does not verify is not a replay, it is an accusation.

**Link 2 — The substrate as second witness.** ZFS on the vault body checksums every
block, and the monthly scrub is the **muster of the bits** — silent corruption has one
month, at most, to hide. Btrfs guards the Forge the same way. The ledger disks (raw
LVM, XFS) are deliberately *not* trusted for data integrity — Link 1 covers them — but
their honest write semantics (`cache=none`) guarantee that what was acknowledged was
written, which is the wall condition's half of this clause.

**Link 3 — Redundancy as the healer.** *A checksum without a second copy is a death
certificate; a second copy without a checksum is a rumor.* Detection only diagnoses;
healing requires a verified twin — RAID-Z2 parity on the vault, the Lahamu replica of
the ledger stream, a signature-verified seal. On the host's single NVMe the chain can
only detect; the cure always comes from the vault or the twin, which is one more reason
neither may share the ledger's power cord.

**Link 4 — The corrupt are quarantined, not deleted.** When the chain catches a lying
copy, the response is a **custody transition plus a healing rite**, never an erasure:
the corrupt copy passes to EnkiQDB — the quarantine defined for exactly this, "corrupt
or unknown-state particles" — with a PARZU KAKI recording where and how it lied (which
disk, which block, which pattern: tomorrow's BĀRÛTU omen about our own hardware), while
the true particle is re-derived from a verified copy and takes its place. Corruption in
BahyWay is therefore not a loss event and not even an integrity *failure*: it is an
evidence-generating event. Even the lie is kept, because the lie is data about the
walls.

**The honest gap (declared, per the Fadam Floor).** The host's RAM is not ECC: a bit
may flip in the interval between arrival and the cutting of the hash — before Link 1
can witness. The gap is made thin by hashing at the gate and can be made thinner by ECC
hardware in the Kish era, but it cannot be made zero, and the clause says so: the
residual interval is a declared contribution to ε, not a silence. **L-7 and L-8
together are the full guarantee the Architect stated this morning: nothing entering the
walls is ever lost, and nothing inside them can lie about itself for long.**

**Gate G4 obligations (added to the docket):**
`chain_integrity : ∀ k, verify (hash_chain k)` on every read and replay path — an
invariant, not a scheduled job; `heal_totality : ∀ corrupt_copy, ∃ verified_twin,
rederivable corrupt_copy` conditional on the vault condition of L-7; and
`quarantine_custody : ∀ corrupt_copy, transitions_to EnkiQDB corrupt_copy` — the
exhaustive proof that no detection path ends in deletion.

---

# TABLET X — GL-OPS-002 "URUK–KISH": THE TWO STREAMS DOCTRINE


## Clause K-1 — The Two Cities

Upon the ecosystem passing its tests on the bare-metal Fedora 44 host, it forks into two
streams by the upstream-first pattern:

- **The URUK stream** — experimental, rolling, allowed to break. New engines, unsealed
  laws, prototype tabs, BĀRÛTU's freshest omens. The city that invented writing keeps
  inventing.
- **The KISH stream** — conservative, sealed, boring on purpose. It receives **nothing**
  except promotions: crates that passed the PB-153 four-stage gate and Gate G4, bundled
  into an Ed25519-signed Zagesi-class manifest. Fixes enter Kish only as backported
  numbered playbooks. Where kingship sits, surprises do not.

## Clause K-2 — Clone by Playbook, Never by Image

Both streams are materialized **exclusively from Ansible inventories**
(`inventories/uruk`, `inventories/kish`) run from the host per Way of Work 5. Disk-image
cloning is forbidden for stream creation: an image proves a machine once existed; a
playbook proves it can exist again. Cattle, not pets — and every Kish rebuild from
nothing doubles as a passed DR drill, which is Clause K-4's gift.

## Clause K-3 — The Promotion Pipeline

Uruk → Kish promotion (PB-316) is the release liturgy: candidate set frozen in Uruk →
four-stage gate (PB-153) → Gate G4 obligations green → Zagesi manifest signed → Kish
inventory pinned to the manifest → Kish rebuilt from playbooks → smoke rites →
promotion KAKI sealed. A change that cannot survive this pipeline was never conservative;
it was merely untested optimism wearing a tie.

## Clause K-4 — The Marriage of the Tablets

The day Kish lives on a second box, one architecture answers both of this morning's
questions at once: Kish **is** the DR second leg — the off-host home of the NUZI vault
and the standing Lahamu twin for the ledger-bearing types; Kish **is** the standing
restore-drill target, mustered on every rebuild; and Kish **is** the demonstration
environment that professors and investors see, which Uruk's experiments can never break.
One host is a workshop; two streams on two boxes are an institution.

---

# THE PLAYBOOK SUITE — PB-310 … PB-317

All playbooks run from the Fedora 44 host per Way of Work 5 (Ansible host→VDI; no manual
commands anywhere in this suite; no HTML production dashboards — all watch surfaces are
NuskuEngine on the sovereign monitoring node). Each playbook ends by writing its own
run-KAKI.

**PB-310 — Lahmu–Lahamu Ledger Shipping.** Provision the standby twin for the
ledger-bearing types (7004, 7006; shard streams for 7002/7007); establish the KAKI
shipping stream with ADANNU watermark reporting into NuskuEngine; configure the MĪLU
ladder bounds (buffer size, NUZI shed path, throttle signal); verify by comparing ledger
heads and watermark lag against sealed tolerance. *Gate:* shipped-segment hash chain
verified end to end.

**PB-311 — Snapshot Seals & the NUZI Vault.** Create the vault target (encrypted,
off-host — external disk now, Kish box later); schedule per-type snapshot cadence from
the L-4 tier table, with **EnkiQDB quarantine batches sealed on every admission** and
their custody lineage recorded; Ed25519-sign every snapshot with the release key; prune
by sealed retention — **quarantine seals exempt from pruning while any PARZU
investigation remains open**; register every seal as a KAKI. *Gate:* signature
verification of the newest and oldest retained seals, including the oldest open-custody
quarantine seal.

**PB-312 — Read-Node Rebuild Rite.** Tear down and re-derive the read node's projections
entirely from the ledger (optionally from snapshot + replay); verify projection equality
against pre-recorded checksums, exercising the Gate G4 replay-determinism guarantee in
anger. *Gate:* checksum equality — not similarity — or the rite fails loudly.

**PB-313 — Promotion Ceremony (Write-Node Failover).** The scripted ceremony: freeze
shipping, record watermark, present declared-loss statement, require the Architect's
seal input (CSR-08), promote Lahamu, repoint streams, demote-and-rejoin the returned
twin as standby. *Gate:* promotion KAKI complete with watermark, loss, operator, seal;
split-brain check (old primary fenced) verified before streams repoint.

**PB-314 — The Backup Muster.** Scheduled restore drill: select seals by age and tier,
restore into a scratch VM, run the smoke rites, record muster-KAKI with restore duration
(the measured RTO) and outcome; flag any seal whose restore history is empty as
**ghost-backup: silent-perfect** on the Nusku watch. *Gate:* every tier mustered within
its sealed interval.

**PB-315 — The Two Inventories.** Author `inventories/uruk` and `inventories/kish` with
shared roles and stream-specific variable sets (versions pinned in Kish, rolling in
Uruk); prove both streams build from bare definitions; forbid image-derived hosts by
assertion tasks. *Gate:* clean build of each stream from playbooks alone.

**PB-316 — The Kish Promotion Pipeline.** Implement Clause K-3 end to end: freeze,
four-stage gate, G4 check, Zagesi manifest signing, Kish pin, rebuild, smoke rites,
promotion KAKI. *Gate:* a deliberately broken candidate is rejected at the correct stage
(the pipeline is tested by being refused, not only by being obeyed).

**PB-317 — Game Day (the Full Muster).** The annual-or-better exercise: simulate loss of
the write node mid-stream; execute PB-313; rebuild the read node by PB-312; restore one
tier from the vault by PB-314's rite; measure RTO/RPO against the sealed targets; file
the game-day KAKI with every measured number and every surprise. *Gate:* the ecosystem
serves reads and accepts writes at ceremony's end, and the report hides nothing —
Truth Before Beauty applies to our own outages most of all.

---

*Ninth and tenth tablets drafted, suite numbered 310–317 clear of all prior claims.
Implementation waits, per the governing law, until the standing playbooks close — but
the twins are named, the cities are chosen, and the vault knows it must leave the house.
The seal belongs to the Architect alone. 𒁾*
