//! Storage fragmentation analysis — tracks index health and prescribes
//! REBUILD vs REORGANIZE vs no-action for each index.
//!
//! Thresholds follow standard SQL Server / PostgreSQL guidance:
//!   < 10 %  → Healthy    (no action)
//!   10–20 % → Minor      (monitor)
//!   20–30 % → Moderate   (REORGANIZE candidate)
//!   30–40 % → Severe     (REBUILD threshold)
//!   > 40 %  → Critical   (urgent REBUILD)

/// Health state of a single index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FragmentationState {
    Healthy,
    Minor,
    Moderate,
    Severe,
    Critical,
}

impl FragmentationState {
    pub fn label(&self) -> &'static str {
        match self {
            FragmentationState::Healthy => "Healthy",
            FragmentationState::Minor => "Minor",
            FragmentationState::Moderate => "Moderate",
            FragmentationState::Severe => "Severe",
            FragmentationState::Critical => "Critical",
        }
    }

    pub fn from_pct(pct: f32) -> Self {
        if pct < 10.0 {
            FragmentationState::Healthy
        } else if pct < 20.0 {
            FragmentationState::Minor
        } else if pct < 30.0 {
            FragmentationState::Moderate
        } else if pct < 40.0 {
            FragmentationState::Severe
        } else {
            FragmentationState::Critical
        }
    }
}

/// Fragmentation record for a single index.
#[derive(Debug, Clone)]
pub struct StorageFragment {
    pub index_name: String,
    pub table_name: String,
    /// Logical fragmentation percentage (0–100).
    pub fragmentation_pct: f32,
    /// Number of 8-KB pages in the index.
    pub page_count: u32,
    /// Fill factor at last rebuild (0 = server default, usually 80–90).
    pub fill_factor: u8,
}

impl StorageFragment {
    pub fn new(
        index_name: impl Into<String>,
        table_name: impl Into<String>,
        fragmentation_pct: f32,
        page_count: u32,
        fill_factor: u8,
    ) -> Self {
        StorageFragment {
            index_name: index_name.into(),
            table_name: table_name.into(),
            fragmentation_pct: fragmentation_pct.clamp(0.0, 100.0),
            page_count,
            fill_factor,
        }
    }

    pub fn state(&self) -> FragmentationState {
        FragmentationState::from_pct(self.fragmentation_pct)
    }

    pub fn is_critical(&self) -> bool {
        self.fragmentation_pct >= 30.0
    }

    pub fn recommended_action(&self) -> &'static str {
        match self.state() {
            FragmentationState::Healthy => "None",
            FragmentationState::Minor => "Monitor",
            FragmentationState::Moderate => "REORGANIZE",
            FragmentationState::Severe => "REBUILD",
            FragmentationState::Critical => "REBUILD (Urgent)",
        }
    }

    /// Cost estimate of leaving this fragmentation unaddressed:
    /// pages × fragmentation_pct / 100 — a rough I/O waste proxy.
    pub fn wasted_pages(&self) -> u32 {
        (self.page_count as f32 * self.fragmentation_pct / 100.0) as u32
    }

    /// Whether this index would benefit from a lower fill factor after rebuild.
    /// Rule: if fill_factor > 90 and fragmentation_pct > 25, fill factor is too high.
    pub fn fill_factor_too_high(&self) -> bool {
        self.fill_factor > 90 && self.fragmentation_pct > 25.0
    }
}

/// Map of all indexes scanned in a single fragmentation pass.
#[derive(Debug, Clone)]
pub struct FragmentationMap {
    fragments: Vec<StorageFragment>,
}

impl FragmentationMap {
    pub fn new(fragments: Vec<StorageFragment>) -> Self {
        FragmentationMap { fragments }
    }

    pub fn total_indexes(&self) -> usize {
        self.fragments.len()
    }

    pub fn critical_count(&self) -> usize {
        self.fragments.iter().filter(|f| f.is_critical()).count()
    }

    pub fn moderate_or_above_count(&self) -> usize {
        self.fragments
            .iter()
            .filter(|f| f.fragmentation_pct >= 20.0)
            .count()
    }

    pub fn rebuild_targets(&self) -> Vec<&StorageFragment> {
        self.fragments
            .iter()
            .filter(|f| f.fragmentation_pct >= 30.0)
            .collect()
    }

    pub fn reorganize_targets(&self) -> Vec<&StorageFragment> {
        self.fragments
            .iter()
            .filter(|f| f.fragmentation_pct >= 20.0 && f.fragmentation_pct < 30.0)
            .collect()
    }

    pub fn worst_fragment(&self) -> Option<&StorageFragment> {
        self.fragments.iter().max_by(|a, b| {
            a.fragmentation_pct
                .partial_cmp(&b.fragmentation_pct)
                .unwrap()
        })
    }

    pub fn healthy_count(&self) -> usize {
        self.fragments
            .iter()
            .filter(|f| f.state() == FragmentationState::Healthy)
            .count()
    }

    /// Overall storage health score: 100 − weighted_fragmentation (0–100).
    pub fn overall_health_score(&self) -> f32 {
        if self.fragments.is_empty() {
            return 100.0;
        }
        let total_pages: u32 = self.fragments.iter().map(|f| f.page_count).sum();
        if total_pages == 0 {
            return 100.0;
        }
        let wasted: u32 = self.fragments.iter().map(|f| f.wasted_pages()).sum();
        let frag_ratio = wasted as f32 / total_pages as f32;
        (100.0 - frag_ratio * 100.0).max(0.0)
    }

    pub fn total_wasted_pages(&self) -> u32 {
        self.fragments.iter().map(|f| f.wasted_pages()).sum()
    }

    pub fn iter(&self) -> impl Iterator<Item = &StorageFragment> {
        self.fragments.iter()
    }
}

// ── Fixtures ─────────────────────────────────────────────────────────────────

/// Representative fragmentation scan — mirrors the screenshot showing
/// index fragmentation at 38 % (Critical) for the SalesOrder indexes.
pub fn sales_order_frag_map() -> FragmentationMap {
    FragmentationMap::new(vec![
        StorageFragment::new("PK_BusinessEntity", "Person.BusinessEntity", 5.2, 120, 80),
        StorageFragment::new("IX_Person_LastName", "Person.Person", 38.4, 2800, 95),
        StorageFragment::new(
            "IX_PersonPhone_PhoneNumber",
            "Person.PersonPhone",
            12.3,
            450,
            80,
        ),
        StorageFragment::new(
            "IX_PersonPhone_CountryRegion",
            "Person.PersonPhone",
            22.1,
            110,
            80,
        ),
        StorageFragment::new("PK_SalesOrder", "Sales.SalesOrderHeader", 8.7, 900, 80),
        StorageFragment::new(
            "IX_SalesOrder_CustomerID",
            "Sales.SalesOrderHeader",
            31.0,
            1800,
            90,
        ),
        StorageFragment::new(
            "IX_OrderDetail_ProductID",
            "Sales.SalesOrderDetail",
            7.1,
            600,
            80,
        ),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sales_frag_has_seven_indexes() {
        assert_eq!(sales_order_frag_map().total_indexes(), 7);
    }
    #[test]
    fn critical_count_two() {
        // IX_Person_LastName (38.4%) + IX_SalesOrder_CustomerID (31.0%)
        assert_eq!(sales_order_frag_map().critical_count(), 2);
    }
    #[test]
    fn rebuild_targets_are_critical_indexes() {
        let map = sales_order_frag_map();
        for t in map.rebuild_targets() {
            assert!(t.fragmentation_pct >= 30.0);
        }
    }
    #[test]
    fn reorganize_targets_in_correct_range() {
        let map = sales_order_frag_map();
        for t in map.reorganize_targets() {
            assert!(
                t.fragmentation_pct >= 20.0 && t.fragmentation_pct < 30.0,
                "{} at {:.1}%",
                t.index_name,
                t.fragmentation_pct
            );
        }
    }
    #[test]
    fn worst_fragment_is_person_lastname() {
        let map = sales_order_frag_map();
        let worst = map.worst_fragment().unwrap();
        assert_eq!(worst.index_name, "IX_Person_LastName");
    }
    #[test]
    fn health_score_below_100_when_fragmented() {
        assert!(sales_order_frag_map().overall_health_score() < 100.0);
    }
    #[test]
    fn health_score_100_for_empty_map() {
        assert_eq!(FragmentationMap::new(vec![]).overall_health_score(), 100.0);
    }
    #[test]
    fn fragmentation_state_thresholds() {
        assert_eq!(
            FragmentationState::from_pct(5.0),
            FragmentationState::Healthy
        );
        assert_eq!(
            FragmentationState::from_pct(15.0),
            FragmentationState::Minor
        );
        assert_eq!(
            FragmentationState::from_pct(25.0),
            FragmentationState::Moderate
        );
        assert_eq!(
            FragmentationState::from_pct(35.0),
            FragmentationState::Severe
        );
        assert_eq!(
            FragmentationState::from_pct(45.0),
            FragmentationState::Critical
        );
    }
    #[test]
    fn recommended_action_matches_state() {
        let rebuild = StorageFragment::new("IX_X", "T", 38.0, 100, 80);
        assert_eq!(rebuild.recommended_action(), "REBUILD");
        let reorg = StorageFragment::new("IX_Y", "T", 25.0, 100, 80);
        assert_eq!(reorg.recommended_action(), "REORGANIZE");
        let ok = StorageFragment::new("IX_Z", "T", 5.0, 100, 80);
        assert_eq!(ok.recommended_action(), "None");
    }
    #[test]
    fn fill_factor_flag() {
        let high_fill = StorageFragment::new("IX_A", "T", 30.0, 100, 95);
        assert!(high_fill.fill_factor_too_high());
        let low_fill = StorageFragment::new("IX_B", "T", 30.0, 100, 80);
        assert!(!low_fill.fill_factor_too_high());
    }
    #[test]
    fn state_ordering() {
        assert!(FragmentationState::Critical > FragmentationState::Severe);
        assert!(FragmentationState::Severe > FragmentationState::Moderate);
        assert!(FragmentationState::Moderate > FragmentationState::Minor);
        assert!(FragmentationState::Minor > FragmentationState::Healthy);
    }
    #[test]
    fn state_labels_non_empty() {
        let states = [
            FragmentationState::Healthy,
            FragmentationState::Minor,
            FragmentationState::Moderate,
            FragmentationState::Severe,
            FragmentationState::Critical,
        ];
        for s in &states {
            assert!(!s.label().is_empty());
        }
    }
}
