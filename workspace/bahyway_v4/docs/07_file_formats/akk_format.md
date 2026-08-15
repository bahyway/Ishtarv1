# .akk File Format

> **DubSar Help** | `.akk` | File Formats

## Purpose

The .akk file is the law file for a Tribe or particle family. It declares the
PARZU governance rules that the MARDUK Gate enforces during transformation.

## Grammar (sketch)

```
akk_file     := header rule*
header       := "tribe" tribe_id version
rule         := "on" event "when" condition "do" action
condition    := expr ("&&" | "||" expr)*
action       := "promote" | "demote" | "steward" | "archive"
```

## Sovereign Constraints

Every .akk change must be stamped with a reason context (the documentation
system) so auditors can trace why a governance rule changed.

## See Also

- `06_governance_parzu/parzu_laws.md`
- `07_file_formats/tmpl_format.md`
