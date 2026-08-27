# BahyWay.Ecosystem v4.0 — Bare-Metal `uruk` Go-Live Runbook

**Purpose:** the single "start here, in this order" checklist for taking
everything landed on `master` (as of `bahyway/EnkiDB` PR #115, merged
2026-08-15) from "correct in git" to "actually running" on the real
Fedora Workstation 44 box (`uruk`). This is a sequencing guide, not a
replacement for the detailed references it points to —
`docs/16_runbooks/DEPLOY_REFERENCE_ALL_PLAYBOOKS.md` and
`docs/08_pipeline_alaktu/OTAP_PIPELINE.md` remain the authoritative detail
for any one step.

**Honest scope note, checked before writing this:** most of the playbooks
below are, per `docs/16_runbooks/PLAYBOOK_EXECUTION_TRIAGE.md`'s own
per-row notes, *authored, YAML-checked, and unit-tested in the authoring
sandbox* — not yet run for real on `uruk`. This runbook does not claim
otherwise. Each phase below says plainly what "done" looks like so you can
tell, on your own machine, whether a step is already satisfied or still
needs running.

**Run every command below from the repo root (`~/Forge/EnkiDB`), never
from inside `playbooks/`.** `ansible.cfg` (which points `inventory =
ansible/inventory.ini`) is only auto-discovered in the *current*
directory — Ansible does not search upward for it. `cd`-ing into
`playbooks/` first hides it, so any playbook targeting a real inventory
host (not `hosts: localhost`) silently matches zero hosts and does
nothing, with no error — confirmed live: `playbook_269` (`hosts:
eriduous-vdi`) did exactly this on a first real run of this runbook.
Every command below is written `ansible-playbook playbooks/playbook_
NNN_....yml`, run from the root, for this reason.

---

## Phase 0 — Get the code onto `uruk`

```bash
# On uruk itself (ansible/inventory.ini's `uruk ansible_connection=local`
# means playbooks run FROM uruk directly, no separate control node):
cd ~/Forge/EnkiDB   # or wherever your clone lives
git fetch origin
git checkout master
git pull origin master
```

**Verify:** `git log -1 --oneline` shows `302f52c` or later (the sealing
commit) with `8cf2ad9` (PR #115's merge) in its ancestry.

---

## Phase 1 — Confirm the control node (idempotent, safe to re-run)

```bash
cd ~/Forge/EnkiDB
ansible-playbook playbooks/playbook_269_retire_eriduous_vdi_confirm_baremetal_control_node.yml
```

**What this checks:** confirms via `systemd-detect-virt` that the host
running this play reports bare metal, not a VM; leaves
`ansible/inventory.ini`'s `uruk ansible_connection=local` line as-is (it
was already correct). If this has already run once successfully on
`uruk`, re-running it is a no-op confirmation, not a repeat of any real
change. **Verify it actually ran** (not "no hosts matched") — the `PLAY
RECAP` should show `eriduous-vdi` (the alias this playbook still targets
by name — see its own header for why it wasn't renamed) with `ok=` and
no `skipping: no hosts matched` line above it.

**Also confirm host privilege groups exist** (creates the 5 Linux groups
`bahyway-architect`/`dataSteward`/`administrator`/`developer`/
`stakeholder` used by AnuGovernor's run-confirmation registry):

```bash
ansible-playbook playbooks/playbook_268_bahyway_host_privilege_groups.yml
```

This one targets `hosts: localhost`, so it runs correctly regardless of
inventory resolution — real gids get assigned and reported directly in
the play output.

---

## Phase 2 — Confirm the CQRS write/read VMs exist and are reachable

`enkidb-node-write` (192.168.122.101) and `enkidb-node-read`
(192.168.122.107) are assumed to already exist by nearly every deploy
playbook in this repo.

**Topology note (2026-08-15, Architect decision):** the shipped default
is exactly 2 nodes, 7 Podman containers each — `playbook_212`/`221`/`222`
(everything `playbook_259` chains in Phase 3) all target the same two
inventory groups (`enkidb_write`/`enkidb_read`) regardless of which
EnkiDB type they deploy; the 7 types differ by port (7001–7007) on the
same two hosts, not by host. **This is what BahyWay ships to clients.**
A client with the capacity for it may instead choose 14 VMs — a
dedicated write/read pair per EnkiDB type — but that is a deliberate,
not-yet-built scale-out variant, tracked here as a real future option,
not assumed or half-implemented today. Building it means giving each of
the 7 types its own inventory group and having `playbook_212`/`221`/`222`
target them individually instead of the shared pair — real, scoped
follow-on work, not something to bolt on silently later.

**A third node — the vault-librarian (2026-08-15, Architect request):**
`playbook_265` can now optionally create a third VM alongside write/read,
via the same mechanism, off by default:

```bash
ansible-playbook playbooks/playbook_265_anu_governor_type1_infra_cqrs_nodes.yml \
  -e cqrs_environment=dev -e create_vault_node=true \
  -e node_write_name=uruk-node-write -e node_write_ip=192.168.122.111 \
  -e node_read_name=uruk-node-read   -e node_read_ip=192.168.122.112 \
  -e node_vault_name=uruk-node-vault -e node_vault_ip=192.168.122.113
```

**Real scope boundary, checked before recommending this:** this creates
the VM and enables Podman on it — nothing more. The rest of the
continuity/backup suite this node belongs to (`playbook_310`–`320`,
`GL-OPS-001/002`) is **separately blocked today**, independent of this
runbook's own critical path:
- `playbook_318` (the real ZFS "vault body," a RAID-Z2 pool across 7×5TB
  NAS drives) needs that NAS physically attached — **not yet the case**.
- `playbook_319`/`320`/`315` all target `hosts: host_forge`, an inventory
  group that does not exist in `ansible/inventory.ini` — this suite ships
  its own separate `inventories/uruk/`/`inventories/kish/` tree instead,
  not yet reconciled with the main inventory.
- `playbook_320` additionally needs `host_vg` (a real LVM volume group
  name on `uruk`) and `kaki_runs` (a run-log path), neither supplied by
  `ansible/inventory.ini` — real, unwired prerequisites, not defaults.
- `playbook_315`/`316`/`317` need `kish` (the second physical machine),
  which per `ansible/inventory.ini`'s own notes does not exist yet.
- The whole `PB-310-320` suite's own README states it is **DRAFT,
  untested against the living machines**, pending Architect seal.

`playbook_320`'s hardcoded VM names (`enki-write`/`enki-read`/
`vault-lib`) were renamed to `uruk-node-write`/`uruk-node-read`/
`uruk-node-vault` (2026-08-15) to match what `playbook_265` actually
creates, but the gaps above mean it still cannot run against the main
inventory without real, separate follow-on wiring. Creating the vault
node now is real, useful groundwork — closing the rest of this list is
not part of this runbook's scope.

**Naming note:** this repo already uses "vault" for the Sargon/Gilgamesh
passport authentication file (`vault_check_enabled`, `vault_path` — see
`playbook_265`'s own Safety Mechanism 3). The vault-*librarian* node here
is an unrelated concept (ZFS backup continuity) that happens to share the
word. Read `node_vault_name`/`create_vault_node` and `vault_path`/
`vault_check_enabled` as two separate "vaults," not the same one.

**"Unreachable" and "doesn't exist" are different
facts** — check both explicitly, in this order:

```bash
# 1. Does libvirt already define them? (they may just be powered off)
virsh list --all --name | grep -E '^(enkidb-node-write|enkidb-node-read)$'

# 2. If they're listed but "shut off", start them first:
virsh start enkidb-node-write
virsh start enkidb-node-read

# 3. Then, once booted, confirm SSH reachability:
ssh bahyway@192.168.122.101 'echo write-node reachable'
ssh bahyway@192.168.122.107 'echo read-node reachable'
```

**If `virsh list --all` already shows both names:** you have two real
options, not one:

- **Use them as-is** — start them and skip straight to Phase 3.
- **Start fresh under new names instead**, deliberately not touching or
  mixing with the old pair's unknown prior state — e.g. to prove the
  from-scratch bring-up genuinely works, repeatably, whenever a new OTAP
  environment is needed. `playbook_265`'s node names and IPs are fully
  overridable via `-e` (Ansible's `--extra-vars` beats the `vars:`
  block's own environment-derived defaults entirely — no code change
  needed), and its duplicate-guard only checks the *names you give it*,
  so a new name pair is created cleanly without ever touching
  `enkidb-node-write`/`enkidb-node-read`:
  ```bash
  ansible-playbook playbooks/playbook_265_anu_governor_type1_infra_cqrs_nodes.yml \
    -e cqrs_environment=dev \
    -e node_write_name=uruk-node-write -e node_write_ip=192.168.122.111 \
    -e node_read_name=uruk-node-read   -e node_read_ip=192.168.122.112
  ```
  (`cqrs_environment=dev` here only controls the vault-gate tier, not the
  naming — the explicit `node_*_name`/`node_*_ip` overrides take full
  precedence over whatever that value would otherwise compute. IPs
  `.111`/`.112` are free — outside both the production `.101`/`.107` and
  the dev/test/acc `.151`-`.156` range already in use.)

  **Real gap, confirmed live, now fully automated AND unified per
  Ecosystem version (2026-08-15, Automation Pillar — no human lookup, no
  pasted URL, one shared file per version, silent end to end):** the base
  image is no longer downloaded by `playbook_265` at all. It lives at a
  shared, version-pinned location inside the Forge tree itself —
  `Infra/OSImage/v{{ fedora_release }}/<real Fedora filename>` — fetched
  there exactly **once per Ecosystem version**, then **read** (never
  re-downloaded) by every VM `playbook_265` creates: write, read, vault,
  every environment. Two-step flow:

  **Step 1 — once per Ecosystem version, run `playbook_273`:**
  ```bash
  ansible-playbook playbooks/playbook_273_fetch_os_image.yml
  ```
  It creates `Infra/OSImage/v44/` if missing, fetches Fedora's own live
  directory listing at
  `download.fedoraproject.org/.../releases/{{ fedora_release }}/Cloud/x86_64/images/`,
  finds the real qcow2 filename Fedora is serving right now (discovered at
  run time, never a filename anyone typed in), fetches the matching
  `*-CHECKSUM` file from the same directory, extracts that image's real
  published SHA256, and downloads via `ansible.builtin.get_url` with that
  checksum — a corrupted or wrong download is refused, not silently
  accepted. Idempotent: re-running it when a `.qcow2` already sits in that
  folder is a no-op report, not a re-download (pass
  `-e force_refetch=true` to force one).

  **Step 2 — every `playbook_265` run, any number of times:**
  ```bash
  ansible-playbook playbooks/playbook_265_anu_governor_type1_infra_cqrs_nodes.yml ...
  ```
  It only looks inside `Infra/OSImage/v{{ fedora_release }}/`, uses
  whatever `.qcow2` is there, and fails clearly — naming the exact
  `playbook_273` command to run — if that folder is still empty. It never
  touches the network for the base image.

  The only config either playbook reads is `ansible/vars/base_image.yml`
  — `fedora_release` (just the version number, `"44"`, deliberately not
  "latest" — silently jumping major Fedora versions mid-build is a bigger
  risk than a number bumped on purpose once a year) plus two optional
  pin-override fields, left blank by default so live auto-discovery is
  the normal path, not a fallback.

  `Infra/OSImage/` is `.gitignore`d — real qcow2s are host-local build
  output, not source, even though they live inside the Forge checkout by
  design.

  If you'd rather point at an image you already have somewhere else,
  `-e base_image_path=/home/bfadam/vdi/fedora-44-base.qcow2` still works
  (must be root-readable, since `qemu-img`/`virt-install` run as root
  under `become: true`) and skips discovery/download entirely.

  **A second real gap, caught live (2026-08-15) at the exact same root
  cause:** `ssh_pubkey_path` and `vault_path` had the identical
  `become: true`/`ansible_env.HOME` bug as the base image once did —
  both silently resolved to `/root/...`, so the cloud-init render step
  failed looking for `/root/.ssh/id_ed25519.pub` (a key that will never
  exist there; the real key is the Architect's own, under their own
  login home). Fixed the same way conceptually as PB-273's approach:
  stop asking a become-escalated fact for a value that means "the human
  who's actually running this," and read it directly from the
  controller's own invoking environment instead
  (`lookup('env', 'HOME')`, evaluated before any become elevation
  applies) — no `-e` override needed on a normal run;
  `libvirt_pool_dir`/`tls_dir` correctly keep pointing at root's home,
  since those are root-owned storage by design.

  **A third real gap, caught live (2026-08-15), same family but the
  opposite direction:** once the SSH-pubkey fix landed, the very next
  task failed the same way for the per-node TLS cert/key — but this
  time the files genuinely existed (the previous run's openssl task had
  created them at `/root/vdi/tls/*.crt`/`.key`, correctly, as root).
  The bug was reading them back: `lookup('file', ...)` is a
  **controller-side** Jinja2 lookup — it always runs as whichever user
  actually invoked `ansible-playbook`, never elevated by `become: true`,
  even under `connection: local`. `tls_dir` is root-owned, `mode 0700`
  (correctly, for private key material), so the login user's own
  `lookup('file', ...)` call couldn't even see into it. Fixed by reading
  those two files with `ansible.builtin.slurp` instead — a real module,
  which *does* run under `become` — then decoding its base64 result
  where the content gets embedded into the cloud-init YAML.
  `tls_dir`'s permissions are untouched (still root-only, still
  correct); only how the content gets read back changed. Net effect of
  all three fixes together: `ssh_pubkey_path` needed the *other* user's
  file, read as that user (`lookup('env','HOME')`); the TLS keypair
  needed root's own file, read as root (`slurp` under `become`) — same
  become/controller distinction, opposite sides of it.

  **A fourth real gap, caught live (2026-08-15) — a different class
  entirely, nothing to do with become/lookup:** `virt-install` itself
  failed — `Cannot access storage file '/root/vdi/uruk-node-write.qcow2'
  (as uid:107, gid:107): Permission denied`. libvirt's `qemu:///system`
  driver runs the actual guest process as the unprivileged `qemu`
  service account, which cannot traverse into `/root` (mode `0700`) at
  all, no matter what permissions the files themselves have — every
  directory component in a path needs search permission for that
  account. Fixed by moving `libvirt_pool_dir` off of `/root/vdi`
  entirely, onto the canonical `/var/lib/libvirt/images/bahyway` — the
  standard libvirt storage pool location, already configured by the
  libvirt/qemu packages with correct traversal permissions for the
  `qemu` account, so VM disks/seed-ISOs/cloud-init files never sit under
  any human or root home directory again.

  **Operational heads-up, not a bug:** `virt-install`'s own failure
  message hints it may have left the three domains **defined** in
  libvirt even though starting them failed (`"you can restart your
  domain by running: virsh ... start ..."`). If your next run's
  duplicate-CQRS-pair guard unexpectedly halts saying a pair "already
  exists," that's why — clean up the stray definitions first:
  ```bash
  for n in uruk-node-write uruk-node-read uruk-node-vault; do
    virsh --connect qemu:///system undefine "$n" --remove-all-storage 2>/dev/null
  done
  ```
  (safe to run even if some/all of those domains don't exist — this
  playbook's own duplicate-guard uses the identical `virsh dominfo`
  check to decide, so an empty/no-op result here means nothing to clean.)
  The stray `.qcow2`/`.iso` files left behind at the old `/root/vdi/`
  location are harmless orphans, not referenced by anything anymore —
  safe to delete for disk space, not required.

  **A fifth real gap, suspected live (2026-08-15) — `virt-install`
  itself succeeded this time (all three VMs defined and started), but
  the final `wait_for` SSH-reachability check timed out at 300s on all
  three.** Leading theory, pending confirmation: the rendered
  network-config hardcoded the interface name `eth0`, but systemd's
  predictable-network-interface-names scheme (the default on modern
  Fedora) names a virtio-net NIC something like `enp1s0`/`ens3` instead
  — `eth0` then matches nothing, cloud-init's static-IP config silently
  never applies, and the guest likely picked up whatever address the
  libvirt `default` network's own DHCP handed out instead of the
  `.111`/`.112`/`.113` this playbook waited on. **Confirm before
  assuming the fix below is the whole story:**
  ```bash
  virsh net-dhcp-leases default
  ```
  If that shows the three VMs holding *different* addresses than
  requested, the theory's confirmed. Fixed the network-config to match
  the interface by its virtio_net **driver** instead of guessing a name
  (guaranteed correct given this playbook's own
  `--network network=default,model=virtio`), with a fixed `set-name` so
  it's stable across boots.

  **This fix cannot take effect on the three VMs already created** —
  cloud-init only reads its seed ISO on first boot, and this playbook's
  own `creates:`/duplicate-guard idempotency means simply re-running it
  will skip regenerating anything that already exists. A clean teardown
  is required first — a deliberate act, not something this playbook
  does automatically (same CSR-08 sovereignty reasoning as its own
  duplicate-guard: halt and let the Architect decide, never silently
  destroy real infrastructure):
  ```bash
  for n in uruk-node-write uruk-node-read uruk-node-vault; do
    virsh --connect qemu:///system destroy "$n" 2>/dev/null
    virsh --connect qemu:///system undefine "$n" --remove-all-storage 2>/dev/null
  done
  sudo rm -f /var/lib/libvirt/images/bahyway/uruk-node-*
  ```
  (one broad glob, not four narrow ones — `--remove-all-storage` already
  deletes the `.qcow2`/`.iso` domain-referenced volumes, so a separate
  glob for those matches nothing on a fresh teardown; under `zsh`
  (`NOMATCH` on by default), a *single* unmatched glob among several on
  one command line aborts the entire line before any of them run,
  silently skipping the ones that *did* match — caught live, this is
  why one broad pattern is safer than several narrow ones. In practice
  this particular cleanup step turned out not to gate the actual fix
  below — the `ansible.builtin.copy` tasks that render `-user-data.yaml`/
  `-network-config.yaml` always diff-and-overwrite their content
  unconditionally, `creates:`-free — but it's still correct hygiene, and
  the next fix genuinely does depend on `-seed.iso` being gone.) The TLS
  keypair under `/root/vdi/tls/` is unrelated to networking and does not
  need regenerating.

  **A sixth real gap, caught live (2026-08-15) — the actual root cause
  behind both SSH-timeout runs, superseding the driver-match theory
  above as the explanation (though that fix stays; it's still correct
  on its own terms):** `cloud-localds`' real CLI is
  `cloud-localds [options] OUTPUT USERDATA [METADATA]` — the third bare
  positional argument is **meta-data**, not network-config. This
  playbook was handing the rendered `-network-config.yaml` file to
  `cloud-localds` as meta-data, so cloud-init never received real
  network configuration in **either** SSH-timeout run — the content was
  never reaching cloud-init's network stage at all, regardless of
  whether it said `eth0` or matched by driver. Fixed by passing it via
  the real `--network-config=` flag instead of a bare third argument.
  This is why the `net-dhcp-leases` table stayed empty both times: no
  networking was ever configured on the guest, static or DHCP.

  **A seventh real gap, caught live (2026-08-15), a different layer
  entirely — nothing to do with cloud-init or networking:** even with
  network-config actually reaching the guest, all three domains vanished
  from `virsh list --all` outright (not "shut off" like a normal crash —
  gone, undefined) shortly after `virt-install` reported success.
  Root cause: `base_image_path` (the shared per-version OS image,
  PB-273) is deliberately kept inside the Forge tree per the Architect's
  own explicit request, which puts it under a human login user's home
  directory — and Fedora's default home-directory permissions (`0700`)
  block the unprivileged `qemu` service account from traversing in. Each
  node's overlay disk correctly lives under the qemu-accessible
  `/var/lib/libvirt/images/bahyway/` now (the earlier fix), but it's a
  copy-on-write overlay whose **backing file** is this same
  home-directory-nested image — QEMU still has to open that backing
  file directly at boot, wherever the overlay itself sits. `virt-install`
  reports success as soon as the domain is defined and the start call is
  issued; the actual failure happens moments later when QEMU tries to
  open the backing chain and can't — consistent with no error reaching
  Ansible and the domain disappearing rather than showing "shut off."

  On top of that: Fedora ships SELinux enforcing by default, and
  confined qemu processes only permit opening files labeled
  `virt_image_t` — which anything outside the conventional
  `/var/lib/libvirt/images/` path never gets automatically. This is a
  second, independent blocker layered on the same non-standard-location
  problem; fixing the POSIX permissions doesn't fix the SELinux label,
  and vice versa.

  Both fixed in `playbook_273_fetch_os_image.yml` (not `playbook_265`)
  since PB-273 already owns this folder's lifecycle: a POSIX ACL grants
  the `qemu` account execute-only (traversal, not read or list)
  permission on every directory between the login user's home and the
  image folder — the home directory's normal privacy for every other
  account is untouched — plus `semanage fcontext`/`restorecon` to label
  the folder `virt_image_t` for SELinux (harmless no-ops if SELinux
  turns out to be permissive/disabled on this host). Because this lives
  in PB-273, simply **re-running it** (no arguments needed, no
  re-download — the image is already there, this is a pure permissions
  pass) applies the fix to the image you already fetched:
  ```bash
  ansible-playbook playbooks/playbook_273_fetch_os_image.yml
  ```
  No teardown needed before the next `playbook_265` attempt this time —
  the domains already vanished on their own, so the duplicate-guard has
  nothing to refuse.

  **An eighth and ninth real gap, both caught live (2026-08-16) via
  actual console/boot-log evidence, same underlying symptom twice in a
  row:** after all seven fixes above, `virt-install` and cloud-init both
  succeeded cleanly, but SSH still timed out. `virsh --connect
  qemu:///system console <node>` (note the explicit `--connect` — plain
  `virsh` commonly defaults to the unprivileged `qemu:///session`, a
  completely separate namespace from where these VMs actually live
  under `become: true`; every diagnostic in this runbook uses the
  explicit form for exactly this reason) showed cloud-init's own
  boot-time network report: the interface (`enp1s0`, renamed from
  `eth0` by systemd's predictable-naming scheme) was up, but held only
  a link-local IPv6 address — no static IPv4 at all, on two independent
  fresh boots with two different MAC addresses. First attempt matched
  the interface by `driver: virtio_net` with `set-name: vnet0`; the
  rename silently never took effect, so the static-address config
  (keyed to the unrenamed name) never matched the real device. Second
  attempt dropped `set-name` and kept `match: driver:` alone; same
  exact symptom persisted — this cloud-init version's renderer does not
  reliably apply an `addresses:` block keyed under a `match:`-only
  entry. Fixed by dropping `match:`/`set-name` entirely and keying the
  network-config directly on `enp1s0` — empirically confirmed identical
  across two independent boots for this exact virt-install topology
  (single `virtio-net-pci` device, fixed PCI bus/slot), not a guess.

  **RESOLVED, 2026-08-16 — first successful real bring-up.** With the
  hardcoded-`enp1s0` fix, a clean teardown/rerun produced
  `PLAY RECAP ... failed=0` with all three `wait_for` tasks `ok` and the
  full write/read/vault summary printed — `uruk-node-write` (.111),
  `uruk-node-read` (.112), `uruk-node-vault` (.113) all live, booted,
  network-configured, and SSH-reachable. Nine real, independently
  diagnosed and fixed issues across this whole chain (base image
  location and its qemu-account ACL/SELinux access, SSH-pubkey/
  vault-path home-directory resolution under `become`, TLS cert/key
  read permissions via `slurp` vs. controller-side `lookup()`, VM
  storage location moved off `/root`, `cloud-localds`' real
  `--network-config` argument syntax, and finally the interface-naming/
  matching chain above) — this is the first genuinely working,
  first-real-hardware bring-up of the CQRS write/read/vault pair.

**Only if `virsh list --all` shows neither name at all** — genuinely
nothing defined yet, a real from-scratch bring-up of the default
`enkidb-node-*` pair:

```bash
ansible-playbook playbooks/playbook_265_anu_governor_type1_infra_cqrs_nodes.yml \
  -e vault_check_enabled=true
```

This creates the VMs via libvirt/KVM from nothing, with the
duplicate-CQRS-pair guard described in its own header (refuses to create
a second `production`-environment pair if one already exists — re-run
with `-e cqrs_environment=dev` for a non-production pair instead). This
playbook is marked, in its own header, as **not yet executed for real**
(syntax-checked only, no libvirt available in the authoring sandbox) —
you are the first real run.

---

## Phase 3 — First-time 7-Types EnkiDB bring-up (one command)

This is the step that actually stands up all 7 EnkiDB database types'
containers on the 2-VM CQRS split and ingests real content into the two
types that have an automated ingestion path (EnkiDDB, EnkiMDB):

**⚠ Stakeholder prerequisite, if `enkidb_write`/`enkidb_read` point at
freshly created nodes** (e.g. right after a from-scratch `playbook_265`
bring-up): PB-259 chains `playbook_212` internally, and that step cannot
complete unattended the first time a node has ever run it — each fresh
node needs its own dedicated, read-only GitHub Deploy Key registered
before it can `git fetch` this repo, and registering one is a real,
security-sensitive action only a human with write access to this repo's
Settings can do. Confirmed live, twice: a fresh node's own SSH key gets
`Permission denied (publickey)` against `git@github.com:...` until this
is done. **This is expected, not a bug** — see `playbook_212`'s own
header comment (PB-212.11) for the full run-fail-register-rerun sequence.
Registering a node's key is a one-time cost, not a per-run one.

```bash
cd ~/Forge/EnkiDB
ansible-playbook playbooks/playbook_259_full_7types_enkidb_bootstrap.yml
```

**Do not run PB-212/221/222/208/216/213 by hand** — PB-259 sequences all
six internally, in the correct order. Full detail, including exactly what
each of the six sub-steps does and why EnkiDB core / the BeeMDM-chain
types (EnkiSDB/EnkiODB/EnkiQDB/EnkiDW) legitimately end up with zero or
synthetic rows (no automated real-content path exists for them yet — a
real, open gap, not a bug), is in `DEPLOY_REFERENCE_ALL_PLAYBOOKS.md`'s
"First-Time 7-Types EnkiDB Bring-Up" section.

**Verify:** re-running this same command is the correct idempotency
check for the deploy half (containers only recreate on a real image
change) — but note step 4/5 (ingest) **will duplicate content** on a
second full run. Day 2 onward, reach for `playbook_208` or
`playbook_216` alone instead of re-running PB-259.

---

## Phase 3b — Optional: build the assets-node image

Not required by anything else in this runbook — real, useful groundwork
whenever it's convenient. `playbook_271` builds one Podman image holding
the pinned Rust toolchain, every vendored crate source, and the
rebranded Godot engine binary, so downstream builds (including a future
Flatpak) "download once, work forever" against a fixed digest instead of
re-fetching from crates.io/GitHub every time:

```bash
cd ~/Forge/EnkiDB
ansible-playbook playbooks/playbook_271_build_bahyway_assets_node.yml
```

This is a Podman **image**, not a new VM — it builds and runs directly
on `uruk` (`hosts: eriduous-vdi`, the retired-VDI alias PB-269 confirmed
now means bare metal), not a dedicated third machine. It does not push
the image anywhere (registry choice is a separate, later decision) and
does not wire into a Flatpak manifest yet — see the playbook's own
header for exactly what it does and doesn't do.

---

## Phase 4 — Build and install AnuGovernor (the corpus-execution tool)

```bash
cd ~/Forge/EnkiDB
ansible-playbook playbooks/playbook_263_deploy_anu_governor.yml \
  -e target_env=production
```

Builds `crates/anu-governor` in place (shares the workspace's warm
`target/`), installs to `/usr/local/bin/anu-governor` (or
`anu-governor-web` for the browser-facing face — pass
`-e anu_governor_features=web`). Run it from the repo root afterward
(`anu-governor.toml`'s paths are relative to there):

```bash
cd ~/Forge/EnkiDB
anu-governor
```

From here, AnuGovernor's own Corpus tab can drive the rest of the
playbook catalog interactively — see `playbooks/ANU_GOVERNOR_PB_MANUAL.md`
for what's runnable and what isn't yet.

---

## Phase 5 — Production go-live (the step that makes `master` actually serve)

**This is the step this session's own work identified as missing and
built: reaching `master` in git and Production actually serving that
commit are two different facts.** Run this deliberately, after
confirming you're on the commit you mean:

```bash
cd ~/Forge/EnkiDB
git rev-parse HEAD   # confirm this is the commit you intend to go live with
ansible-playbook playbooks/playbook_557_production_golive_from_accept.yml \
  -e i_understand_this_is_production=true
```

**What it does, in order** (full detail in the playbook's own header):
refuses to run without the explicit flag above; verifies local `HEAD`
matches `origin/master` exactly (refuses otherwise — no going live from
an unreviewed commit by accident); re-runs `cargo test --workspace` one
more time as defense in depth; redeploys the real
`enkiddb-write-server`/`enkiddb-read-server` Podman containers via the
already-proven `playbook_212`; records a permanent, timestamped entry in
`docs/16_runbooks/NARU_AUDIT_JOURNAL.md`.

**Deliberately not chained by PB-557 — run these separately if this
go-live also needs them:**

```bash
# Only if the AnuGovernor web dashboard UI itself changed:
ansible-playbook playbooks/playbook_284_launch_shala_dashboard.yml

# Only if website content changed (bahyway.com / beemdm.com / heptascript.com):
ansible-playbook playbooks/playbook_556_deploy_bahyway_websites_nginx.yml
```

---

## Phase 6 — Optional: OTAP branch housekeeping

**Not blocking Phase 5** — `playbook_557` checks against `origin/master`
directly, not `otap/accept`. But for OTAP discipline going forward,
`otap/dev`/`otap/test`/`otap/accept` are currently behind `master` (the
Phase 2 work landed via a direct PR to `master`, bypassing the normal
promotion chain). Bring them back in sync at your convenience:

```bash
git push origin master:otap/dev
git push origin master:otap/test
git push origin master:otap/accept
```

(A plain fast-forward, since `master` already contains everything each
of them has — confirmed via `git merge-base --is-ancestor` before writing
this runbook.)

---

## Phase 7 — Optional: full-corpus audit report

If you want the CEO-facing executive report across the whole tracked
playbook set (not required by anything above):

```bash
cd ~/Forge/EnkiDB
ansible-playbook playbooks/playbook_IsimudEngine.yml -c local -v
```

Produces `ISIMUD_ENGINE_REPORT_<ts>.json/.txt` (technical) and
`EXECUTIVE_REPORT_<ts>.md/.html` (CEO-facing), plus a ledger entry in
`docs/16_runbooks/NARU_AUDIT_JOURNAL.md`. See
`docs/11_tooling/ISIMUD_ENGINE_MANUAL.md`.

---

## Quick reference — the whole sequence, minimal form

**All from the repo root — see the note at the top of this file for why.**

```bash
cd ~/Forge/EnkiDB
git checkout master && git pull origin master
ansible-playbook playbooks/playbook_269_retire_eriduous_vdi_confirm_baremetal_control_node.yml
ansible-playbook playbooks/playbook_268_bahyway_host_privilege_groups.yml
# verify enkidb-node-write/-read are reachable; if not:
# ansible-playbook playbooks/playbook_265_anu_governor_type1_infra_cqrs_nodes.yml -e vault_check_enabled=true
ansible-playbook playbooks/playbook_259_full_7types_enkidb_bootstrap.yml
ansible-playbook playbooks/playbook_263_deploy_anu_governor.yml -e target_env=production
git rev-parse HEAD  # confirm the commit
ansible-playbook playbooks/playbook_557_production_golive_from_accept.yml -e i_understand_this_is_production=true
```

Every step above is the Architect's own CSR-08 act — this runbook orders
them, it does not run any of them.
