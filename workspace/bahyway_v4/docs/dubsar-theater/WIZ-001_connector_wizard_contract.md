# WIZ-001 — Enki Database Connector Wizard: Sovereign Contract
Status: SEALED · Landed 2026-07-10 as scenes/wizard.tscn +
scripts/wizard.gd (dubsar-theater). Supersedes the retired
scripts/enkidb_wizard.gd and dubsar_proof.gd's inline wizard.

## 1. Engine Grid — SEVEN tiles, canonical labels & ports
Port numbering preserves what was already live in
enkidb_wizard.gd (EnkiDB..EnkiQDB), extended with the two
engines that script never had.

| Tile | Sovereign label | HEPT port |
|---|---|---|
| EnkiDB  | Sovereign Particle Graph (flagship)  | 7001 |
| EnkiDW  | Data Warehouse                       | 7002 |
| EnkiSDB | Search & Index                       | 7003 |
| EnkiODB | Object / Document Particles          | 7004 |
| EnkiQDB | Query Acceleration                   | 7005 |
| EnkiMDB | Metadata Store — READ-ONLY at runtime | 7006 |
| EnkiDDB | Distributed Particle Fabric           | 7007 |

FORBIDDEN words anywhere in the wizard: "SQL", "Relational",
"Multi-Model" (for EnkiMDB), port 5432, any Postgres residue.
(Still VERIFY against GL-001 if it later specifies a different
mapping — this numbering is the pragmatic in-repo reconciliation
of two conflicting drafts, not a GL-001 citation.)

## 2. Connection step
- Protocol: HEPT binary TCP, matching bin/enkidb-query-server's
  real wire format: [u32 LE query_len][UTF-8 query] -> batched
  JSON frames -> u32=0 DONE sentinel. There is NO magic-byte
  handshake in the real server (verified directly against
  bin/enkidb-query-server/src/main.rs) — an earlier WIZ-001
  draft assumed one; that assumption has been removed from the
  landed connection_tester.gd, which now does TCP-reachability
  verification only, honestly labelled as such.
- Transport encryption: crates/kupru's real AkkadianCipher
  (ChaCha20-Poly1305), SargonKdf (Argon2id), AkkadianSeal
  (Ed25519) — bridge from Godot not yet wired (GDExtension or
  local socket pending); until then secrets are NOT persisted,
  by design (scripts/akkadi_safe_bridge.gd returns UNAVAILABLE).

## 3. Credentials step
- Identity = Passport: [ Gilgamesh Passport | Sargon Passport ]
  + key unlock.
- "Remember credentials" -> AkkadiSafeEngine vault path once
  wired (Argon2id KDF, real crate: kupru::sargon_kdf).

## 4. Test & Save step
- Today: TCP reachability only. A real protocol-level probe
  requires a handshake frame to be added to
  bin/enkidb-query-server first — flagged as a genuine gap, not
  simulated.
- Save is permitted regardless of test result (informational);
  EnkiMDB refuses a Sargon (write) passport profile at the
  wizard level (gilgamesh_required: true in enki_engines.gd —
  FIXED 2026-07-27: this used to read `read_only`, which is true for
  all seven engines for an unrelated reason and refused Sargon
  everywhere, not just EnkiMDB; gilgamesh_required is scoped to the
  Nergal/ADAD law specifically).

## 5. Multi-engine federation (the wizard's real purpose)
Saved profiles form the catalog for federated W5H2 queries —
see TPL-001 (PB-158/159/160) for the query-planning side of
this, corrected the same day.
