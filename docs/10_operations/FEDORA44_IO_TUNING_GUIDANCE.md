# Fedora Workstation 44 — I/O & Filesystem Tuning for the EnkiDB Fleet

**Architect:** DUB.SAR 𒁾 Bahaa Fadam
**Date:** 2026-07-21
**Scope:** real, technically accurate guidance for the Architect's own
Fedora Workstation 44 hardware. **None of this has been measured on that
hardware** — this sandbox has no access to it, 4 vCPUs, and no ZFS
kernel module. Every number below is either a general, well-established
fact about the mechanism in question, or explicitly marked as something
to measure once applied. Do not treat any of this as a benchmarked claim.

## The honest headline: the engine change matters far more than the filesystem

With `CachedReadNode` (this session's real change — see
`PB-221_SCALE_BENCHMARK_FINDINGS.md` and the surrogate-key follow-up),
a query touches **zero disk I/O** once the Data File is loaded into RAM.
The only I/O left on the query-serving path is:

1. The one-time bulk load at `CachedReadNode::open`/reload (two large
   sequential reads — the whole `.idx` file, the whole `.data` file).
2. The Write Node's `materialize()` pass (sequential writes) and the
   cross-host sync step (`tasks/enkiddb_cross_host_sync.yml` — tar,
   `scp`-equivalent, untar).

Filesystem/OS tuning changes the cost of *those* — reload latency and
ingest throughput — not steady-state query latency, which is now RAM-
and CPU-bound (hash/array lookups, `HeptaScript` evaluation). Tune with
that target in mind, not "make queries faster."

## ZFS

ZFS's ARC (Adaptive Replacement Cache) is a second-level, in-RAM cache
of the pool's own blocks, sitting below the OS page cache. For a
workload where the application (`CachedReadNode`) already loads the
entire dataset into its own heap on startup, ARC's main value is making
the *second and later* `open()`/reload sequential reads of the Data
Files fast even after the file has been evicted from the plain page
cache (e.g., after other processes have used the box's RAM in between) —
ZFS's own checksumming and compression (`lz4` is typically free or
net-positive on modern CPUs, since it reduces bytes actually read from
the physical device) can also shrink the real on-disk size of the entity
history store and EAV posting index, which directly shortens the one-
time bulk read.

Concretely, if you put `DATA_DIR` on a ZFS dataset:
- `zfs set recordsize=1M <pool>/<dataset>` — the Data File format's own
  reads are large and mostly sequential (this session's
  `iter_all_raw()`), so a large ZFS record size reduces per-record
  overhead versus ZFS's 128K default.
- `zfs set compression=lz4 <pool>/<dataset>` — EAV posting lists and
  entity history blobs are text/small-integer-heavy and compress well;
  lz4's decompression cost is typically far below the I/O it saves.
- `zfs set atime=off <pool>/<dataset>` — this fleet's Data Files are
  read via bulk `iter_all_raw()` and point lookups (`ReadNode`'s
  fallback path, still used elsewhere), never scanned for access-time
  bookkeeping; `atime` updates are pure write amplification here.
- Size `zfs_arc_max` deliberately if the box also runs other memory-
  hungry things (e.g. Godot/DubSar itself) — ARC will otherwise grow to
  consume most of free RAM by default, which can starve
  `CachedReadNode`'s own multi-GB in-process heap at large N. As a
  starting point: reserve `CachedReadNode`'s expected working set (see
  the RSS numbers in `PB-221_SCALE_BENCHMARK_FINDINGS.md` — roughly
  300 bytes/entity in this session's synthetic schema, scale by your
  real particle's average EAV size) plus the rest of the box's needs,
  and cap ARC to what's left.

**Real caveat**: ZFS-on-Linux is a real, well-supported, but
out-of-kernel-tree module (DKMS or `zfs-dracut` on Fedora, via the ZFS
project's own repo — not in Fedora's default repos). Confirm it's
actually installed and the pool is imported before pointing `DATA_DIR`
at it; a silent fallback to whatever the mount point resolves to
otherwise is worse than not using ZFS at all.

## If not ZFS: XFS over ext4 for this workload

Both are fine, real, mature filesystems. For this specific access
pattern (large sequential reads at load/reload, sequential appends
during `materialize()`, occasional large file replace on sync), XFS's
allocation strategy (extent-based, delayed allocation) generally handles
large sequential files with less fragmentation over repeated
rewrite-the-whole-file materialize cycles than ext4's block-group
approach — a real, well-documented XFS advantage for large-file
workloads, not something to take on faith; verify by watching
`materialize()`'s own wall-clock time across repeated runs (a fresh
Fedora install won't show this — fragmentation-driven slowdown shows up
after many materialize cycles on the same volume, which this sandbox's
one-shot benchmark runs never exercised).

Mount options worth setting on either, for this specific workload
(Data Files are never memory-mapped, always read via `File::read_to_end`/
`seek`+`read_exact`, per `enkidb-datafile`'s own "no memmap2 crate" doc
comment):
- `noatime` — same rationale as ZFS's `atime=off` above.
- `nobarrier`/`nodiratime` only if the volume is on a device with a
  battery-backed or otherwise power-loss-safe write cache (e.g. a
  server-grade NVMe with power-loss protection) — do not disable write
  barriers on a consumer NVMe/SSD without that guarantee; the durability
  tradeoff is real, not free performance.

## NVMe / block layer

- **I/O scheduler**: `none` (a.k.a. `noop`) for NVMe — the device's own
  internal queueing already does reordering far better than a kernel-
  level scheduler can for a fast NVMe; Fedora Workstation typically
  already defaults to this for NVMe block devices, worth confirming with
  `cat /sys/block/<nvme-device>/queue/scheduler` rather than assuming.
- **Queue depth**: NVMe drives expose deep hardware queues (often 32-
  128+); the default Linux NVMe driver settings are usually already
  tuned reasonably out of the box — this is a "verify, don't blindly
  change" item, via `nvme id-ctrl` / `cat /sys/block/<dev>/queue/nr_requests`.
- **`io_uring`**: real, genuinely faster async I/O than epoll-based
  reads for high-IOPS workloads, but this codebase's servers use plain
  synchronous `std::fs`/`std::net` I/O (per this whole session's own
  zero-third-party-dependency discipline) — adopting `io_uring` would
  mean either the `tokio-uring`/`io-uring` crate (a new dependency) or
  hand-rolling raw syscalls, a real, separate engineering decision, not
  something to silently bolt on. Given queries are already RAM-bound
  post-`CachedReadNode`, the I/O this would actually speed up is the
  load/reload bulk read and the Write Node's materialize — worth
  revisiting specifically if reload latency at real 1B scale (which
  this sandbox could not measure — see the PB-221 report) turns out to
  matter operationally.

## Threads, file descriptors, and the concurrency model

Both `enkidb-read-server` and `enkidb-write-server` spawn one OS thread
per TCP connection (`thread::spawn` in the accept loop) — the same
pattern every server in this fleet already uses. Real, measured in this
session (see `PB-221_SCALE_BENCHMARK_FINDINGS.md`'s worker-thread
section): 50 concurrent `QUERY` connections against `enkidb-read-server`
all returned correct results, with per-request latency ranging 6.6ms–
88ms in this sandbox's **4-vCPU** environment — consistent with OS
thread scheduling across a small core count, not a locking bottleneck
(the server holds an `RwLock`, so concurrent readers never serialize
against each other, only briefly against a reload's `Arc` swap).

For real production load on a real multi-core Fedora 44 box:
- Raise `ulimit -n` (open file descriptors) and check
  `/proc/sys/kernel/threads-max` if you expect thousands of concurrent
  connections — thread-per-connection has a real per-thread memory cost
  (default 8MB stack reservation per thread on Linux, though only the
  touched pages are actually committed) that becomes worth watching
  above a few thousand concurrent connections. Not a concern at the
  scale this session tested (tens of connections), worth measuring
  before it becomes one.
- If connection counts do grow large, a bounded worker-thread pool
  (fixed N OS threads pulling connections off a channel, rather than one
  thread per connection) is the standard next step — a real, scoped
  future change, not something to build speculatively before it's
  actually needed.

## What to actually do next

1. Apply `noatime` (or ZFS's `atime=off`) — free, safe, real gain,
   zero risk.
2. If ZFS is already the Architect's chosen filesystem (referenced
   elsewhere in this codebase's own design docs): set `recordsize=1M`
   and `compression=lz4` on the `DATA_DIR` dataset specifically.
3. Run `playbook_221_enkidb_core_deploy_and_scale_sweep.yml` at real
   100M/1B scale and measure `CachedReadNode::open`'s real wall-clock
   time on real hardware — that is the number these filesystem changes
   actually move, and it's the one piece of this whole investigation
   that could not be measured from this sandbox.
