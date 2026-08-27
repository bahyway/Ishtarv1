//! LAW-HOT-1: the whole u16 domain, materialized. No search exists here.

/// Cache-resident projection of one NUZI snapshot epoch.
pub struct HotIndex {
    /// 64 KB: valid[tribe_id] = 1 when the id resolves in this epoch.
    valid: Vec<u8>,
    /// 512 KB: node_uid per tribe_id (0 = unassigned).
    node: Vec<u64>,
    pub epoch: u64,
    pub seal: u64,
}

impl HotIndex {
    /// Build from a NUZI snapshot's resolution table (tribe_id → node_uid).
    pub fn from_snapshot(table: &[(u16, u64)], epoch: u64, seal: u64) -> Self {
        let mut valid = vec![0u8; 1 << 16];
        let mut node = vec![0u64; 1 << 16];
        for &(t, n) in table {
            valid[t as usize] = 1;
            node[t as usize] = n;
        }
        Self {
            valid,
            node,
            epoch,
            seal,
        }
    }

    /// O(1): one indexed load. This outruns any SIMD *search*.
    #[inline]
    pub fn resolve(&self, tribe_id: u16) -> Option<u64> {
        if self.valid[tribe_id as usize] == 1 {
            Some(self.node[tribe_id as usize])
        } else {
            None
        }
    }

    #[inline]
    pub fn is_valid(&self, tribe_id: u16) -> bool {
        self.valid[tribe_id as usize] == 1
    }

    /// The 64 KB validity plane, exposed for the batch path.
    #[inline]
    pub fn validity_plane(&self) -> &[u8] {
        &self.valid
    }

    pub fn footprint_bytes(&self) -> usize {
        self.valid.len() + self.node.len() * 8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn idx() -> HotIndex {
        let table: Vec<(u16, u64)> = (1..=1000u16).map(|t| (t, 7000 + t as u64)).collect();
        HotIndex::from_snapshot(&table, 0xE7, 0x5EA1)
    }

    #[test]
    fn single_lookup_is_direct_and_exact() {
        let i = idx();
        assert_eq!(i.resolve(500), Some(7500));
        assert_eq!(i.resolve(0), None);
        assert_eq!(i.resolve(1001), None);
        assert_eq!(i.resolve(u16::MAX), None);
        assert!(i.is_valid(1) && !i.is_valid(60_000));
        assert_eq!(i.footprint_bytes(), (1 << 16) + (1 << 16) * 8);
    }
}
