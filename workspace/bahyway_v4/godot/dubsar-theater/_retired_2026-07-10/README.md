# Retired 2026-07-10

`enkidb_wizard.gd`/`.tscn` — superseded by `scenes/wizard.tscn` +
`scripts/wizard.gd` (the WIZ-001-corrected connector wizard). This one
had only 5 engines (missing EnkiMDB/EnkiDDB), no crypto, no passport
model. Its port numbering (EnkiDB=7001..EnkiQDB=7005) was preserved and
extended in the new `enki_engines.gd` rather than discarded.

Kept here for reference, not for reuse — see PB-161 (playbooks/) for the
full reconciliation record.
