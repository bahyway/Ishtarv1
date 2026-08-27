//! OrbitalDeviationJournal — sovereign evidence record for unexplained orbital events.
//!
//! Parallel to AuditJournal in enkidullm-audit but specific to orbital physics.
//! Entries are append-only, CRC-16 verified, and keyed by (particle_id, epoch).

use crate::cause::DeviationCause;
use bahyway_crc::crc16;

/// One immutable orbital deviation evidence entry.
#[derive(Debug, Clone)]
pub struct OrbitalDeviationEntry {
    /// Opaque particle identifier (first 8 bytes of KAKI, as u64).
    pub particle_id: u64,
    /// Epoch at which the deviation was observed.
    pub observed_epoch: u64,
    /// Epoch of the previous (reference) snapshot.
    pub reference_epoch: u64,
    /// Cartesian distance moved in orbit-space units.
    pub cartesian_delta: f32,
    /// True when the particle crossed an orbital ring boundary.
    pub ring_changed: bool,
    /// Causal attribution — always `Unexplained` for entries in this journal.
    pub cause: DeviationCause,
    /// Trust penalty applied to this particle's SourceTrust dimension.
    pub trust_penalty: f32,
    /// CRC-16 of the evidence payload for structural integrity.
    pub checksum: u16,
}

impl OrbitalDeviationEntry {
    pub fn new(
        particle_id: u64,
        observed_epoch: u64,
        reference_epoch: u64,
        cartesian_delta: f32,
        ring_changed: bool,
        cause: DeviationCause,
        trust_penalty: f32,
    ) -> Self {
        let payload = build_payload(
            particle_id,
            observed_epoch,
            reference_epoch,
            cartesian_delta,
            ring_changed,
            cause,
        );
        let checksum = crc16(&payload);
        Self {
            particle_id,
            observed_epoch,
            reference_epoch,
            cartesian_delta,
            ring_changed,
            cause,
            trust_penalty,
            checksum,
        }
    }

    /// Verify structural integrity of this entry.
    pub fn verify(&self) -> bool {
        let payload = build_payload(
            self.particle_id,
            self.observed_epoch,
            self.reference_epoch,
            self.cartesian_delta,
            self.ring_changed,
            self.cause,
        );
        crc16(&payload) == self.checksum
    }

    /// Render a sovereign evidence tablet in Markdown.
    pub fn to_markdown(&self) -> String {
        format!(
            "# 𒁾 Orbital Deviation Tablet\n\
             **Particle:** `{:#018x}`\n\
             **Reference Epoch:** {}  **Observed Epoch:** {}\n\
             **Cartesian Delta:** {:.4}  **Ring Changed:** {}\n\
             **Cause:** {}  **Trust Penalty:** {:.3}\n\
             **Evidence Seal:** `{:#06x}`\n",
            self.particle_id,
            self.reference_epoch,
            self.observed_epoch,
            self.cartesian_delta,
            self.ring_changed,
            self.cause.label(),
            self.trust_penalty,
            self.checksum,
        )
    }
}

/// Append-only sovereign journal of unexplained orbital deviations.
#[derive(Debug, Default)]
pub struct OrbitalDeviationJournal {
    entries: Vec<OrbitalDeviationEntry>,
}

impl OrbitalDeviationJournal {
    pub fn new() -> Self {
        Self::default()
    }

    /// Append an entry. Deduplicates by (particle_id, observed_epoch, checksum).
    pub fn append(&mut self, entry: OrbitalDeviationEntry) -> bool {
        let is_dup = self.entries.iter().any(|e| {
            e.particle_id == entry.particle_id
                && e.observed_epoch == entry.observed_epoch
                && e.checksum == entry.checksum
        });
        if is_dup {
            return false;
        }
        self.entries.push(entry);
        true
    }

    pub fn entries(&self) -> &[OrbitalDeviationEntry] {
        &self.entries
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// All entries for a specific particle, ordered by observed_epoch.
    pub fn for_particle(&self, particle_id: u64) -> Vec<&OrbitalDeviationEntry> {
        let mut v: Vec<_> = self
            .entries
            .iter()
            .filter(|e| e.particle_id == particle_id)
            .collect();
        v.sort_by_key(|e| e.observed_epoch);
        v
    }

    /// Total accumulated trust penalty across all entries for a particle.
    pub fn accumulated_penalty(&self, particle_id: u64) -> f32 {
        self.for_particle(particle_id)
            .iter()
            .map(|e| e.trust_penalty)
            .sum::<f32>()
            .clamp(0.0, 1.0)
    }
}

// ── Internal helpers ──────────────────────────────────────────────────────────

fn build_payload(
    particle_id: u64,
    observed_epoch: u64,
    reference_epoch: u64,
    cartesian_delta: f32,
    ring_changed: bool,
    cause: DeviationCause,
) -> Vec<u8> {
    let mut v = Vec::new();
    v.extend_from_slice(&particle_id.to_le_bytes());
    v.extend_from_slice(&observed_epoch.to_le_bytes());
    v.extend_from_slice(&reference_epoch.to_le_bytes());
    v.extend_from_slice(&cartesian_delta.to_bits().to_le_bytes());
    v.push(ring_changed as u8);
    v.extend_from_slice(cause.label().as_bytes());
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(pid: u64, obs: u64) -> OrbitalDeviationEntry {
        OrbitalDeviationEntry::new(
            pid,
            obs,
            obs - 1,
            0.42,
            true,
            DeviationCause::Unexplained,
            0.3,
        )
    }

    #[test]
    fn entry_verify_integrity() {
        let e = make_entry(0xABCD_1234, 1_000_000);
        assert!(e.verify());
    }

    #[test]
    fn journal_append_and_dedup() {
        let mut j = OrbitalDeviationJournal::new();
        let e1 = make_entry(1, 100);
        let e2 = make_entry(1, 100); // same — duplicate
        assert!(j.append(e1));
        assert!(!j.append(e2));
        assert_eq!(j.len(), 1);
    }

    #[test]
    fn for_particle_filters_correctly() {
        let mut j = OrbitalDeviationJournal::new();
        j.append(make_entry(1, 100));
        j.append(make_entry(2, 101));
        j.append(make_entry(1, 102));
        assert_eq!(j.for_particle(1).len(), 2);
        assert_eq!(j.for_particle(2).len(), 1);
    }

    #[test]
    fn accumulated_penalty_clamps_at_one() {
        let mut j = OrbitalDeviationJournal::new();
        for epoch in 1..=10 {
            j.append(make_entry(42, epoch)); // 0.3 × 10 = 3.0 → clamped to 1.0
        }
        let penalty = j.accumulated_penalty(42);
        assert!(
            (penalty - 1.0).abs() < 1e-6,
            "penalty must clamp at 1.0, got {penalty}"
        );
    }

    #[test]
    fn markdown_contains_required_fields() {
        let e = make_entry(0xDEAD_BEEF, 999_999);
        let md = e.to_markdown();
        assert!(md.contains("Orbital Deviation Tablet"));
        assert!(md.contains("UNEXPLAINED"));
        assert!(md.contains("0.300") || md.contains("0.3"));
    }
}
