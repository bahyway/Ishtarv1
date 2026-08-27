# How To: Generate and Use a Sovereign Passport

> **DubSar Help** | `How-To > Identity` | kupru · sargon-passport-manager · gilgamesh-master-key · dubsar-theater

## When to use this guide

You want to give yourself, a stakeholder, or a client a real, cryptographically
sealed credential that unlocks DubSar Theater IDE (or, going forward, any
other BahyWay.Ecosystem v4.0 app wired the same way) — not a typed username
and password, but an Ed25519-signed **Sovereign Passport**.

**Time to complete:** 5 minutes to mint and import your first Passport;
15–30 minutes for a full Architect Key ceremony (Gilgamesh) if you're setting
up recovery for a root key for the first time.

---

## The four things that get generated, and which one you actually need

This is the single most common point of confusion, so it comes first. Three
different tools produce JSON files that all live in similar-looking folders,
but only ONE of them is what DubSar accepts.

| Artifact | Made by | What it is | Ever import it into DubSar? |
|---|---|---|---|
| **Root signing key** | Gilgamesh Step 1/3 | The private Ed25519 key. Held in memory only — **never saved to disk as a whole**. | Never (it isn't a file) |
| **Shamir share** (`architect-key-SHARE-N-of-M.json`) | Gilgamesh Step 2 | One fragment of the root key. Any M of N shares reconstruct it; fewer reveal nothing. | **No.** DubSar will reject it — it has no `quppu` field, because it isn't a Passport at all. |
| **Sovereign Passport** (`architect-PASSPORT-*.json` or `sargon-PASSPORT-*.json`) | Gilgamesh Step 4 **or** Sargon's "Generate & Save to Vault" + Export | A self-contained, Ed25519-sealed credential: `{"quppu":..., "kupru":..., "naru":..., "istar":...}`. | **Yes — this is the only file DubSar's "Import Passport…" accepts.** |
| **Ledger entry** | Gilgamesh Step 5 (automatic) | Your own private record of which Passport went to which client, and when. Never leaves Gilgamesh. | Never (it isn't exported for this purpose) |

If you're ever unsure which file you're holding, open it in a text editor —
a real Passport always starts with `{"quppu":{"passport_id":...`. A share
file does not.

---

## Concepts: gardener vs. architect

There are two kinds of Passport, matching DubSar's own `PASSPORT` dropdown
(`EnkiEngines.PASSPORT_TYPES`):

- **Sargon Passport** ("gardener") — minted by **Sargon Passport Manager**.
  Everyday, lower-privilege credential (`level 1` in the tool's own vault
  list). This is what you'd hand to most stakeholders or client instances.
- **Gilgamesh Passport** ("architect") — minted by **Gilgamesh Master Key**,
  from a root key you either just generated or reconstructed from Shamir
  shares. Privilege level 7 — the highest level the ceremony issues. For
  the Architect, or anyone who genuinely needs full authority.

Both kinds carry the same real cryptography (Ed25519 seal, Argon2id-derived
storage keys, ChaCha20-Poly1305 encryption) — the difference is only the
privilege level baked into the Passport at mint time.

---

## Step 1: Launch the tool you need

```bash
cd /home/bahyway/Forge/EnkiDB

# Sargon Passport Manager (gardener passports) or Gilgamesh Master Key
# (architect passports + the Shamir ceremony) -- pick one with -e tool=
ansible-playbook playbooks/playbook_227_build_and_launch_kupru_tools.yml
ansible-playbook playbooks/playbook_227_build_and_launch_kupru_tools.yml -e tool=gilgamesh

# DubSar Theater IDE itself
ansible-playbook playbooks/playbook_226_launch_dubsar_godot_ide.yml
```

Both playbooks build `crates/kupru-gdext` (the shared Rust↔Godot bridge every
one of these tools uses) and copy it into the right project automatically —
you don't need to build anything by hand. Add `-e skip_build=true` on a
second run if nothing in `crates/kupru*` has changed since, to skip the
rebuild.

---

## Step 2a: Mint a gardener Passport (Sargon Passport Manager)

1. First run: set a **master vault passphrase** at the unlock prompt.
   Write it down — there is no recovery path if you forget it (that's what
   Gilgamesh's Shamir ceremony is for, on the *architect* side, not this one).
2. Fill in:
   - **Identity Phrase** — needs **≥ 3 consonants**. Vowels (a/e/i/o/u),
     spaces, and hyphens don't count, so a short name like `nabu` alone
     (only 2: n, b) will be rejected. Use something like `sargon-kish` or
     `dubsar-nabu`.
   - **Realm** — a tenant/client identifier, e.g. `bahyway`.
   - **Subject Label** — your own note for who this is (e.g. `stakeholder-01`,
     a client name). Shown in the vault list; never stored inside the
     Passport itself.
3. Click **"▶ Generate & Save to Vault"**. A new entry appears in the list.
4. Select it, click **"Verify Selected"** to confirm the seal round-trips.
5. Select it, click **"Export JSON…"** — this is the file to import into
   DubSar.

## Step 2b: Mint an architect Passport (Gilgamesh Master Key)

Gilgamesh is a 5-step pager (Previous/Next at the bottom):

1. **Generate** — makes a brand-new root key, held in memory only. Do this
   on an offline machine for a real ceremony. (Or skip to Reconstruct if
   you're recovering an existing key from shares instead.)

   This step shows a **"VERIFYING KEY (public half)"** field with its own
   **Copy** button — this trips people up, so to be direct about it: that
   field is *not* a Passport, and copying it does **not** give you
   anything to log into DubSar with. It's the public half of the
   Ed25519 root keypair — safe to share freely, because it can't sign
   anything on its own. The Copy button exists so you can distribute or
   note down this public value for the ecosystem's own verifiers to check
   future Passport signatures against (the same idea as copying an SSH
   *public* key into `authorized_keys` — routine to copy and share,
   nothing like a login credential). The root key itself "never decrypts
   or unlocks anything directly — it only ever mints a fresh, normal,
   time-boxed `SargonPassport`" (see `ARCHITECT_KEY_CEREMONY.md`'s own
   wording) — that minting happens later, at Step 4 below, and *that*
   export is the only thing DubSar's login will ever accept.
2. **Split** — breaks the key into M-of-N Shamir shares and prompts you to
   save each one to a **different** location/device, one save dialog per
   share. Full ceremony detail, including how many shares to make and where
   to actually put them: `crates/kupru/ARCHITECT_KEY_CEREMONY.md`. Clicking
   **Next** past this step with a freshly-generated key still unsplit pops
   a confirmation ("You're proceeding without splitting this root key into
   shares...") rather than silently letting you through — a soft check,
   not a hard block, so a quick practice run through the whole pager still
   only takes one click each. It won't reappear once every share has
   actually been saved, and it never fires after Reconstruct (a
   reconstructed key was already split in an earlier ceremony).
3. **Reconstruct** — break-glass only, for recovering a key from shares
   made in an *earlier* ceremony session — not something you need right
   after Generate in the same session; Mint (Step 4) can use the key
   Generate just put in memory directly. Gather at least the threshold
   number of share files (double-click each in the "Add Share File…"
   dialog, or select + press its Open button — a single click alone only
   highlights it), then Reconstruct.
4. **Mint** — same Identity Phrase / Realm / Subject Label rules as Sargon,
   plus an optional **Client Label** (recorded in your Ledger, not inside
   the Passport). Click **"▶ Issue Architect Passport"**, then
   **"Export Passport JSON…"** — again, this is the file DubSar wants.
   **Next is genuinely disabled** on this step until you export — unlike
   Split's dismissable confirmation, this one's a hard gate: the minted
   passport is the entire deliverable of the ceremony, held only in this
   session's memory until written to disk, so there's no legitimate
   reason to leave this step with a real one unsaved. Minting a second
   passport for a different client re-locks Next until *that* one is
   exported too.
5. **Ledger** — read-only. Every Passport minted in this session gets logged
   here automatically (client label, date/time, realm, expiry) once you set
   a Ledger passphrase, so you can answer "which key did I give client X,
   and when" later. Has its own separate passphrase from everything else.
   "Finish" on this step closes the tool (with a confirmation, since
   anything still only in memory would be lost).

---

## Step 3: Import the Passport into DubSar

On DubSar's login screen:

1. Click **"Import Passport…"**.
2. Pick the exported Passport JSON (**not** a share file — see the table
   above). It's seal-verified immediately; a forged or expired file is
   rejected right here, before anything else is asked.
3. Enter an **Identity** label and a **new** passphrase of your choosing —
   this passphrase is local to DubSar and does not need to match whatever
   passphrase you used in Sargon/Gilgamesh.
4. Confirm. The Passport is now stored, ChaCha20-Poly1305-encrypted, in
   `user://dubsar_login_vault.dat`, keyed to that specific identity +
   Passport type so two different identities never share a derived key
   even if they pick the same passphrase.

## Step 4: Log in

Select the matching **PASSPORT** type (Gilgamesh or Sargon), type the same
**Identity** and passphrase you just registered, click **"▶ Enter DubSar
IDE"**. This decrypts the stored Passport and re-runs the same seal/expiry
check the standalone tools use — wrong passphrase, no matching entry, or a
broken/expired seal all fail with a specific message, never a silent accept.

---

## Renewing an aging Passport

Every Passport tracks its own lifetime; once 80% of it has elapsed,
`renewal_due()` flips true (shown as `[RENEW DUE]` in Sargon's vault list
and Gilgamesh's Ledger). Renewal always **reissues** rather than extends —
there is no "extend" operation by design. In Sargon: select the entry,
re-enter the same Identity Phrase and Subject Label used originally, click
"Renew Selected". The keypair stays the same; only the Passport's validity
window resets.

---

## Troubleshooting

**"LEMNĪ (invalid input): LEMNĪ: < 3 consonants"** (or the friendlier
translated version) — your Identity Phrase doesn't have enough real
consonants. See the rule under Step 2a above.

**"Reconstruction failed... DŪRU: N shares provided, but this split
requires M"** — you gathered fewer shares than the threshold chosen in
Split. Add more, from wherever they were distributed.

**A file is rejected on import with "missing field `quppu`"** — you picked
a Shamir share file, not an exported Passport. See the artifact table at
the top of this guide.

**A file is rejected on import with "missing field `created_at`"** — you
picked Gilgamesh's exported **Ledger** JSON (Step 5's "Export Full Ledger
JSON…"), not the Passport itself. The Ledger is your own private record —
client labels, mint dates, expiry — with none of a real Passport's
`quppu`/`kupru`/`naru`/`istar` structure. The file you want is the one
from Step 4's "Export Passport JSON…" (default name
`architect-PASSPORT-<realm>.json`); the Ledger export now defaults to
`gilgamesh-ledger-record-<date>.json` specifically so the two are harder
to mix up by filename alone.

**Wrong passphrase / no matching entry on login** — either the passphrase
typed doesn't match what was set during Import, or that identity + Passport
type combination was never imported. Use "Import Passport…" first.

**A tool window opens but shows nothing (blank grey)** — this was a real bug
in early versions of PB-226/PB-227 (a stale or missing Godot import cache
meant the KupruBridge GDExtension never got discovered). Both playbooks now
prime the cache unconditionally on every launch; re-running either playbook
resolves it. If it still happens, check `/tmp/dubsar_godot_launch.log` (or
`/tmp/kupru_tool_launch.log`) and its `.import` sibling for the real error.

---

## Honest limits

- **Privilege level is recorded, not yet enforced.** A successful DubSar
  login stores `identity`/`realm`/`privilege_level`/`passport_id` in
  `SessionIdentity` (`dubsar-theater/scripts/session_identity.gd`) for the
  rest of the session, but nothing in DubSar Theater currently reads that
  to gate or restrict any feature. A gardener and an architect Passport get
  the same Theater today.
- **Sargon and Gilgamesh are deliberately offline-first.** Neither talks to
  a live EnkiDB — the ceremony is designed to work on a disconnected
  machine. Nothing you mint or import here is journaled into EnkiODB's
  real event log (`EventCause` in `crates/enkidb-journal`) yet; that would
  need new `EventCause` variants and an optional, best-effort network path
  from these two tools, which is real, separate future work.
- **Other future UrOS/BahyWay.Ecosystem v4.0 apps don't get this for
  free.** Godot's `user://` storage is per-project, so a new Godot app
  wanting the same Import-Passport pattern needs its own copy of
  `kupru.gdextension` + `bin/libkupru_gdext.so` and its own launch playbook
  extended the same way PB-226 was for `dubsar-theater` — the pattern is
  reusable, the wiring is not automatic.

## See Also

- `crates/kupru/ARCHITECT_KEY_CEREMONY.md` — full Shamir M-of-N ceremony detail
- `crates/kupru/README.md` — kupru crate module reference
- `docs/17_troubleshooting/` — deeper diagnosis guides for other subsystems
