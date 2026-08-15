# BahyWay.Ecosystem v4.0 — Documentation Transparency Standard

**Sealed 2026-07-11. Governs every entry in
`docs/BAHYWAY_V4_ARCHITECTURE_REFERENCE_2026-07-11.md`,
`docs/BAHYWAY_ECOSYSTEM_V4_GLOSSARY.md`, and any document that
supersedes them.**

## Why this exists

This ecosystem's documentation has, across many independently-authored
sessions, repeatedly stated claims with uniform confidence regardless
of whether they were checked against real code, copied from an
earlier document, or invented. The 2026-07-07 28-document review
found dozens of confidently-worded claims that turned out to be
fabricated (SumerEngine, NUZI, AsakkuEngine — never existed) sitting
in the same documents, in the same tone, as claims that were
completely real and correct. A reader with no way to tell the
difference cannot safely build on either.

**No claim enters a governing document without a tag from the table
below and a citation.** A citation is a file path (`crate::module::item`
or `path/to/file.rs:line`), a test count with the command that
produced it, or an explicit statement of which source document made
the claim and when it was last checked against code.

## The tags

| Tag | Meaning | What it requires |
|---|---|---|
| ✅ VERIFIED | Checked against real, compiling code or a passing test this pass | Exact citation: file:line, struct/fn name, or `cargo test` output |
| 🧩 PARTIAL | Real, but narrower in scope than the claim/name implies | Citation of what exists, and an explicit statement of what doesn't |
| 📄 DOCUMENTED | Stated in a sealed source document; not independently re-checked against code this pass | Which document, and its date |
| ⚠ COLLISION | Two or more real/documented things share this name | Both cited, disambiguated |
| ❌ NOT FOUND | Searched for directly in code; does not exist, despite being described elsewhere as if real | The search performed, so the next reader doesn't repeat it |
| 🔒 LAW | An Architect-sealed rule or axiom — not a factual existence claim, not subject to "verification" the way code is | Source and seal date |
| ⏳ UNREACHABLE | Requires `eriduous-vdi` or another environment this checking session cannot reach | What command would check it, and where to run it |

## Standing rules

1. **A tag is not permanent.** ✅ VERIFIED means "true as of the date
   cited," not "true forever." Code changes; re-check before trusting
   an old ✅ on anything you're about to build directly on top of.
2. **📄 is not a demotion, it's an honesty marker.** Most of this
   ecosystem's real design intent lives in sealed documents before it
   lives in code — that's normal and expected. 📄 just means "read the
   source document yourself if this matters to your work; I haven't
   re-derived it from code this pass."
3. **❌ NOT FOUND is not "wrong forever," it's "not built yet, or not
   built the way described."** Several ❌ items in this ecosystem's
   history later became ✅ once someone actually built them (EnkiMDB
   and EnkiDDB, for example — ❌ as of every document through
   2026-07-07, ✅ as of 2026-07-11).
4. **When you find a claim with no tag, or a tag you can't verify,
   treat it as untagged — i.e., don't build on it without checking
   yourself first.** This is the same discipline that caught the
   HEPT-protocol-magic-bytes error and the KibratuCause error on
   2026-07-11 (see the Glossary's H and K sections) — both had been
   copied forward, unverified, across multiple "SEALED" documents for
   weeks before anyone grepped the actual code.
