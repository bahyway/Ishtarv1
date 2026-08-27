# EN-QDB-001 · Quarantine Facet Law (draft, unsealed)
EnkiQDB · port 7003 · proposed 2026-08-25 · DUB.SAR 𒁾
Depends on: EN-DDB-002 (Simtu Facet Law), GL-AGE-001 (two-witness), GL-UNT-001 §3 (disclosure boundary),
EN-MDB-001 (Masku mask-as-view), GL-ALG-001-A2 (Never-Averaged Theorem), EN-DDB-004 (Eṭemmu)

---

## §1 The spine is identical; the facet set is not
KAKI v4.0 byte layout is locked and applies unchanged in EnkiQDB — κ[0..3] uuid_hash, κ[4..5] tribe_id,
κ[6] kaki_type, κ[7] kaki_role, κ[8..11] reserved, κ[12..13] timestamp, κ[14..15] CRC-16/CCITT.
What differs is the **mandatory EAV facet set**, and it differs for three reasons that are not stylistic.

## §2 Reason one · the subject shifts
In EnkiSDB the EAV describes **arrival**: where a particle came from and in what condition it landed.
In EnkiODB the EAV describes **meaning**: a factor's unit, range, banding rule, provenance of definition.
In EnkiQDB the EAV describes **a judgment about a particle** — the relation between the citizen and the
clause that refused it. The subject is the *dīnu*, not the thing. A facet set built to describe things
cannot carry a judgment without smuggling the judgment into free text.

## §3 Reason two · validity inverts
Every other store may assume its citizens satisfy the invariants — CRC valid, tribe resolvable, facets parseable.
EnkiQDB exists precisely to hold citizens that **fail** those invariants. Therefore no mandatory QDB facet may be
derived from the payload. The quarantine record must be **self-sufficient**: the envelope carries everything,
and the payload is held as **opaque bytes** (`ṭuppu-blob`) with its integrity recorded as received and never recomputed.
This is the clause that most sharply forbids reusing the SDB/ODB facet set.

## §4 Reason three · quarantine is the only store with a mandatory exit
An ontology particle may live forever. A quarantined particle may not. Every citizen of EnkiQDB carries a
**disposition contract** with a deadline; on expiry it escalates, it does not quietly persist. Release, remediation,
refusal to Irkalla, or archival to NUZI — one of the four must happen, and the rite that closes it is named in advance.
No other EnkiDB type needs custody or exit facets, and inventing them for all seven would be waste.

## §5 The seven mandatory facets (Simtu-compliant)
Four groups, seven facets, mirroring the hepta discipline of EN-DDB-002.

| # | facet | group | carries |
|---|-------|-------|---------|
| 1 | `dinu.rule` | DĪNU · judgment | sealed tablet id + clause that refused it (e.g. `GL-NIM-001 §4`) — never prose |
| 2 | `dinu.class` | DĪNU | epistemic class of the refusal: MEASURED / DERIVED / ESTIMATED / ADVISED / INCOHERENT |
| 3 | `mukinnu.witnesses` | MUKINNU · evidence | the witnesses and their epochs; `count < 2` ⇒ status is SUSPICION, never VERDICT |
| 4 | `mukinnu.observed` | MUKINNU | expected vs observed, verbatim (`crc expected 0x1A2B, observed 0x77C4`) |
| 5 | `massartu.custody` | MAṢṢARTU · custody | holder, held-from, and the disclosure boundary `(k, m)` under which it may be shown |
| 6 | `massartu.integrity` | MAṢṢARTU | sha256 of `ṭuppu-blob` **as received**; recorded once, never recomputed |
| 7 | `wasu.disposition` | WAṢÛ · exit | target state ∈ {RELEASE, REMEDIATE, REFUSE→Irkalla, ARCHIVE→NUZI}, deadline, closing rite (PB id) |

## §6 Optional facets (recorded when known, never required)
`origin.stage` (which of the seven ports refused it) · `origin.sensor` · `tribe.hint` (unresolved tribe guess,
always DERIVED) · `payload.sample` (bounded excerpt, subject to §7) · `remediation.attempts[]` ·
`appeal.opened_by` / `appeal.verdict` · `related.kaki[]` (sibling quarantines) · `impact.blocked_count` ·
`kittu.receipt` (notification delivered) · `legal.hold` (suspends the §4 deadline, and only that)

## §7 Rules of the quarantine store
1. **The payload is never mutated.** A correction is a *new* particle in EnkiSDB citing this KAKI; the quarantined
   original stays byte-identical for as long as custody lasts.
2. **The refusing clause is a reference, not a sentence.** `dinu.rule` points at a sealed tablet and section.
   Free-text reasons belong in optional facets, never in the mandatory seven.
3. **No aggregation over quarantine.** Counting or averaging quarantined citizens hides the one that matters
   (GL-ALG-001-A2). Dashboards may show the census; verdicts may never rest on it.
4. **The disclosure boundary travels with the record.** Quarantined data is often the most sensitive data in the
   estate. `massartu.custody` carries `(k, m)` and SUSA honours it exactly as for any BIGRING.
5. **Two witnesses or it is a suspicion.** A single witness may quarantine (that is a protective act), but may not
   dispose. Disposition requires the second witness (GL-AGE-001).
6. **Every citizen has a deadline.** On expiry, escalate to a named human role; silence is a violation, not a default.
7. **Quarantine is a store, not a bin.** EnkiQDB is queried by HeptaScript like any other tribe — ORBIT, WITNESS,
   PRESENT — and the field over it is a Masku view, never the store itself.

## §8 What must NOT be a mandatory QDB facet
Business meaning · derived quality scores · anything that assumes the payload parsed · a human-written
"reason" string · a severity number without its clause · a colour or state byte (deprecated in v4.0; state lives in EAV).
