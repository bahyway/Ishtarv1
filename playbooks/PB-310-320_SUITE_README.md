# 𒁾 BahyWay v4.0 — Playbook Suite PB-310 … PB-320
## Continuity, Storage & Two Streams — implementation of Tablets IX & X (GL-OPS-001/002)
### Status: DRAFT — READY FOR THE ARCHITECT'S SEAL (see SEAL_REGISTER.md)
### Sealing authority: CSR-08 — the Architect alone. Nothing herein is sealed by its author.

**Honest status (Fadam Floor):** written against the documented topology
(host = Fedora 44 MSI Prestige; write node 192.168.122.101; read node 192.168.122.107;
vault librarian VM; NAS vault body 7×5TB), but **untested against the living machines**.
Per PB-153 doctrine: review → dry-run (`--check`) → live run on Uruk → then seal.

## Run order (first bring-up)
1. PB-319 host storage layout (subvolumes, nodatacow, LVM pool)
2. PB-318 vault body (ZFS pool, 7 datasets + 2 annexes, scrub timer)
3. PB-320 VM disk provisioning (ledger raw LVs, cache policies)
4. PB-315 two inventories validation (uruk/kish)
5. PB-310 ledger shipping (Lahmu–Lahamu)
6. PB-311 snapshot seals & vault
7. PB-312 read-node rebuild rite (prove it once, before you need it)
8. PB-314 backup muster (first muster before first trust)
9. PB-316 Kish promotion pipeline
10. PB-313 promotion ceremony (rehearse on Uruk scratch first)
11. PB-317 game day (the full muster)

## Conventions
- Every playbook ends by appending a **run-KAKI** (hash-chained JSON line) to
  `{{ kaki_runs }}` — Clause L-8 Link 1 applied to our own operations.
- All variables live in `inventories/*/group_vars/all.yml`; playbooks carry no secrets.
- `ansible-playbook -i inventories/uruk pbNNN_*.yml --check` before any live run.
- Native EnkiDB stream shipping is stubbed with rsync+checksum until the
  `enkidb-ship` binary exists; the TODO markers say so where they apply.

*The suite is the law made runnable; the seal remains the Architect's. 𒁾*
