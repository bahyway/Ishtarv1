# How To — Add a New Tribe

> **DubSar Help** | `HowTo > New Tribe` | How-To

## Steps

1. Write a `.akk` file defining the Tribe Ideal (ι_T ∈ [0,1]^7) and PARZU rules.
2. Register the `.akk` file in the Template Registry (`06_governance_parzu/`).
3. Submit via `nabu ingest <akk_file>` — this passes all four Pauli Gates.
4. Verify in the Observatory: the new Tribe ring should appear at the outer rim.
5. Seed particles using `nabu ingest <particle_source>`.

## See Also

- `07_file_formats/akk_format.md`
- `06_governance_parzu/parzu_laws.md`
- `11_tooling/nabu_cli.md`
