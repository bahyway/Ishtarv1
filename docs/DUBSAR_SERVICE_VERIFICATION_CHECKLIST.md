# DubSar Service Verification Checklist — BahyWay.Ecosystem v4.0

2026-07-27. Companion to `docs/ISIMUD_ENGINE_MANUAL.md` — that report tells
you whether the *playbooks* passed. This is the next step: walking each
real *service* by hand, in and out of DubSar, on `eriduous-vdi`. Written
after fixing two real bugs found doing exactly this walkthrough:

- `theater_3d.gd`'s left/right 3D panels were unclickable (raw
  screen-coordinate forwarding onto a tilted 3D quad — fixed with a real
  raycast, commit `b5d4c61`).
- The wizard's Credentials step refused a Sargon Passport against **every**
  engine, not just EnkiMDB (`read_only` field reuse bug — fixed with a
  dedicated `gilgamesh_required` field, commit `1bd8018`).

Both were found by actually doing the walkthrough below, not by reading
code in isolation — expect to find more this way; report back what you see
and treat this doc as living, not final.

## 0. Before you start

Confirm IsimudEngine's most recent report is healthy (`docs/SHEDU/EXECUTIVE_REPORT_LATEST.html`
or the matching `.txt`). A run with `-e launch_guis=false` (the default,
since commit `f7572d2`) will not have opened any DubSar window itself —
everything below is the manual half.

## 1. The seven EnkiDB engines

Real ports, confirmed against each server's own `bind_addr()` / the
deploying playbook, not assumed:

| Engine | Read port | Write port | Deployed by |
|---|---|---|---|
| EnkiDB  | 7001 | 7011 | PB-221 |
| EnkiDW  | 7002 | 7012 | PB-222 |
| EnkiSDB | 7003 | 7013 (shared) | PB-222 |
| EnkiODB | 7004 | 7013 (shared) | PB-222 |
| EnkiQDB | 7005 | 7013 (shared) | PB-222 |
| EnkiDDB | 7102 | 7101 | PB-192 |
| EnkiMDB | 7202 | 7201 | PB-192 |

Write nodes live on `enkidb-node-write` (192.168.122.101); Read nodes on
`enkidb-node-read` (192.168.122.107) — see `ansible/inventory.ini`.

### Outside DubSar (fastest signal — do this first per engine)

```bash
# Port reachability, from eriduous-vdi:
nc -zv 192.168.122.107 7001   # …7002/7003/7004/7005/7102/7202 for the rest
nc -zv 192.168.122.101 7011   # …7012/7013/7101/7201 for the rest

# Container status + real materialized data on disk (EnkiDDB/EnkiMDB only,
# already automated — don't hand-check what a playbook already checks):
cd ~/Forge/EnkiDB/playbooks
ansible-playbook playbook_203_enkiddb_enkimdb_health_check_and_backup.yml -i ../ansible/inventory.ini -v

# Cross-host flush/sync + real QUERY/SEARCH content check (EnkiDDB):
ansible-playbook playbook_213_cross_host_flush_sync_verify.yml -i ../ansible/inventory.ini -v
```

EnkiDB/EnkiDW/EnkiSDB/EnkiODB/EnkiQDB don't have an equivalent PB-203-style
health-check playbook yet — `nc -zv` on their ports plus `podman ps`
(`delegate_to` the right node) is the honest current state. Worth a future
playbook if you want it automated the same way; flag it and I'll build one
rather than you doing it by hand every time.

### Inside DubSar

- **Ctrl+D** (Dashboard Theater) — one tab per engine, each backed by a
  real schema-agnostic probe query. Fastest single view of all 7 at once;
  each tab's "READ NODE" pill just means it's hitting the Read port above.
- **Ctrl+W** (EnkiDB 7-Types Connector Wizard) — pick an engine on Page 1,
  fill Host/Port (pre-filled from Page 1's choice) on Page 2, present a
  Passport on Page 3, Test Connection on Page 4. A **Sargon Passport**
  (e.g. PB-261's `isimud-bootstrap`) now works against all six
  non-EnkiMDB engines (post `1bd8018`); EnkiMDB still requires a
  **Gilgamesh Passport** by law (Nergal/ADAD) — that's expected, not a bug.
- **Ctrl+E** (HeptaScript Editor) — real W5H2 queries against EnkiDDB/EnkiMDB
  specifically, with a live execution log showing real row content.
- **Ctrl+G** (Grid & Orbit 3D Theater) — HeptaScript editor + ORBIT/GRID
  view for EnkiDB/EnkiDDB/EnkiMDB, same three targets as `theater_3d.gd`'s
  `TARGETS` dict. Note the 50,000-particle BIGRING backdrop may not render
  under software/remote rendering (GPUParticles3D needs compute-shader
  support) — that's a rendering-capability gap, not a functional one; the
  editor/query/results panels are what actually prove the service works.
- **Ctrl+K** (Graph Explorer) — real `link.target`/`link.description`
  particles from EnkiDDB.
- **Ctrl+F** (Document Explorer) — EnkiDDB documents, tree + content +
  Related Links.
- **Ctrl+O** (Onion Layers 3D Tower) — static 25-layer/163-crate dataset,
  not a live service check (no EnkiDB engine behind it) — skip for this
  checklist's purposes.

## 2. DubSar PDM IDE — Client documents + SLA Layer

Separate app from DubSar Theater, own login/vault
(`workspace/bahyway_v4/godot/dubsar-pdm/`). Launch:

```bash
cd ~/Forge/EnkiDB/playbooks
ansible-playbook playbook_230_build_and_launch_dubsar_pdm.yml -i "localhost," -c local -v
```

What to actually check once it's open: client document listing loads, a
document opens with real content (not placeholder text), and the SLA Layer
configuration panel shows real, editable fields rather than a static
summary. If anything here looks like the Credentials-page-vs-popup
duplication you flagged in DubSar Theater's wizard, say so specifically —
that's exactly the kind of "looks right in code, wrong in practice" gap
this checklist exists to catch.

## 3. Passport tooling

```bash
cd ~/Forge/EnkiDB/playbooks
ansible-playbook playbook_227_build_and_launch_kupru_tools.yml -i "localhost," -c local -v
```

- **Sargon Passport Manager** — open your existing vault (the salt-bug fix
  from PB-261's investigation, commit `de015f0`, means this should now
  actually reopen across a restart — worth confirming live once, since
  before that fix it never could).
- **Gilgamesh Master Key Manager** — task #130 on the standing list is
  still open ("verify Gilgamesh Master Key tool live on eriduous-vdi") —
  this is the one still-untested piece needed before EnkiMDB's write path
  can be exercised through the wizard at all.

## 4. Suggested order

1. `nc -zv` all 14 ports (7 read + 7 write) — fastest possible "is anything
   obviously down" signal before touching any GUI.
2. PB-203 + PB-213 (already-automated EnkiDDB/EnkiMDB checks).
3. Ctrl+D dashboard — visual confirmation across all 7 at once.
4. Ctrl+W wizard — one non-EnkiMDB engine end-to-end with your Sargon
   Passport (confirms the `1bd8018` fix for real).
5. Sargon Passport Manager — confirm the vault reopens.
6. DubSar PDM — client documents + SLA Layer.
7. Gilgamesh Master Key Manager — closes task #130, unblocks EnkiMDB's
   write side + a Gilgamesh-passport wizard run.

Report back what you see at each step the same way you have been —
real terminal output / screenshots — and I'll fix whatever's actually
broken, not just what looks broken.
