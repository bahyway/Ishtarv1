# GL-SEC-002 · Attack Surface as Particles (draft, unsealed)
Proposed 2026-08-24 · DUB.SAR 𒁾 · die: **Nergal Cubic** (held/refused matter → Irkalla)
Depends on: GL-AGE-001 (two-witness), GL-UNT-001 §3 (disclosure boundary), EN-DDB-004 (Eṭemmu), Nisaba (signed Alert Event KAKIs)

---

## §1 Nergal AV is not a file scanner
Nergal AV does not hunt signatures in bytes. It particleizes the **attack surface of a host you own**:
every listening socket, every router forward, every UPnP mapping, every account without a second factor,
every outbound phone-home, every unattended container, every backup older than its promise.
Each is a citizen with a KAKI. The die renders them as **cubes stacked in threat sectors**.

## §2 Seven threat sectors (hepta)
`EXPOSURE` reachable surface · `IDENTITY` accounts and factors · `TRANSPORT` relays and TLS ·
`FIRMWARE` versions and updates · `EGRESS` what the host sends outward · `PERSISTENCE` what restarts itself ·
`CUSTODY` receipts, snapshots and off-site copies.

## §3 Held and refused
A finding is **HELD** when the posture violates the sealed policy but service continues (the court can still work).
A finding is **REFUSED** when the posture is disallowed outright; the offending surface is closed by the remediation rite.
Refused matter descends to **Irkalla** and stays on the ledger — Irkalla is a record, never a delete.

## §4 Two witnesses before a verdict
No finding fires on a single probe. Two **independent** probes — a local socket enumeration and an
outside-in reachability check from a different network — must agree within one epoch (GL-AGE-001).
One probe alone yields **suspicion**, and suspicion is displayed, never actioned.

## §5 Own-host only
Nergal AV probes only hosts the operator owns and has declared in its charter. There is no discovery of
third-party hosts, no scanning of neighbours, no reachability testing against addresses not in the charter.
A probe outside the charter is itself a REFUSED finding against the operator.

## §6 Epistemic classes carry through
`MEASURED` a socket was observed listening · `DERIVED` a rule implies reachability ·
`ESTIMATED` version implies a known-fixed defect · `ADVISED` the court recommends closure.
No ADVISED finding may be rendered with the weight of a MEASURED one.

## §7 The verdict is a rite, never a silent fix
Every closure is a numbered playbook. Nergal AV proposes; PB-3xx executes; the Kanīku receipt records
what changed, when, and which clause required it. Nothing is remediated silently.
