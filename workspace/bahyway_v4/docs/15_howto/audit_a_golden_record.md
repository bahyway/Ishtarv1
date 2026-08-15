# How To — Audit a Golden Record

> **DubSar Help** | `HowTo > Audit` | How-To

## Steps

1. `nabu probe <kaki>` — confirms Active state and HPS score.
2. `nabu audit <kaki>` — opens the full Jordan Chain (StoryWay journal).
3. Review each Events-KAKI entry: timestamp, source, ADAD/ANU/MARDUK/SHAMASH
   gate decisions, HPS delta, .akk rule firing that caused the transition.
4. If the promotion to Golden Record is disputed, trace the MARDUK gate log
   for the transformation that raised HPS to the threshold.

## Sovereign Guarantee

Every state transition is KAKI-stamped. The audit answer is deterministic —
no inference, no prediction. The recorded truth of the particle's trajectory
through Hepta-space.

## See Also

- `09_observatory/orbital_visualization.md`
- `04_gates/marduk_gate.md`
- `11_tooling/nabu_cli.md`
