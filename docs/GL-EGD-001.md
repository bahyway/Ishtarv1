# GL-EGD-001 — Maintenance Horizon Law

Status: SEALED (ratified by this build — playbook_235).

1. Every grid asset's baru residual history is trended on the medium
   cadence (via `trend-core::time_to_threshold`); the GIBIL HORIZON is
   the predicted date its kappa trajectory crosses the defect
   threshold.
2. Horizon <= 90 days => the asset enters the MAINTENANCE WINDOW: a
   prescription (repair or exclude) must be issued while time
   remains.
3. Field disambiguation is by IDENTITY, never appearance: KAKI
   identity + asset passport (EAV) confirm the crew is at the right
   junction/converter. Enforced in code: `dubsar-gridnav-gd`'s
   `add_asset` rejects (and logs via `godot_error!`) any KAKI hex that
   is not exactly 32 valid hex characters, rather than silently
   defaulting the malformed bytes to zero — a silently-mangled
   identity would defeat the entire point of this clause.
4. EXCLUSION requires PROOF: simulate `in_service=false` on a network
   copy and verify the remaining grid serves all load before
   prescribing exclusion (`egd-engine::exclusion::simulate_exclusion`).
5. Tiers on the stage: BIRQU (active strike, red), HORIZON (predicted
   <= 90 d, amber, days shown), SOUND (green).

## Update (playbook_239): exact linear state estimation

`egd-engine::state_estimate::solve_voltages` fills part of the
power-flow gap below: given the feeder-head voltage (or any subset of
node voltages) as known measurements, it solves the linear KCL system
(`Y_bus * V = -injection`, standard nodal analysis -- Gibil's model is
linear in V, unlike full nonlinear AC power flow) for every other
node's voltage by Gaussian elimination. Verified by round-trip: the
estimated voltages, fed back into the independently-tested residual
sweep, produce near-zero residual everywhere.

**This is still not the harder piece.** `solve_voltages` requires
EXACTLY one known value (voltage or injection) per node -- no
redundant measurements, no bad-data rejection. A real deployment with
noisy, occasionally-wrong sparse telemetry needs weighted least squares
over MORE measurements than unknowns, with a genuine
systematic-vs-anomalous classifier (King-plot separation) to reject a
broken meter rather than silently poisoning the estimate. That
estimator remains unbuilt.

## What this build does NOT claim

- No King-plot bad-data separation: horizon trending is a plain
  least-squares fit (`trend-core`), and the new state estimator is an
  exact solve with no redundancy or bad-data rejection. Noisy or
  sparse metering will produce false Horizon/Birqu tiers, or a
  confidently wrong state estimate, without a genuine
  systematic-vs-anomalous classifier on top of the raw trend.
- Single-phase phasor model only; real distribution feeders in
  southern/central Iraq run heavily unbalanced (informal single-phase
  taps). Production needs three-phase or symmetrical-component
  representation.
- No telemetry-ingestion security wiring (Suhrim watch surface is
  planned in BC-MRD-001, not implemented here).

These are the honest next steps, not hidden gaps.
