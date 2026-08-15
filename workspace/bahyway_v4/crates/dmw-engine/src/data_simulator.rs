//! data_simulator.rs — SQL Data Grid Simulator
//!
//! Generates realistic mock SQL result data for DMW demonstrations.
//! Simulates execution timing differences between Legacy (Nested Loop)
//! and Sovereign (Hash Join) plans — no real database required.
//!
//! The O(n²) vs O(n) contrast is quantitative:
//!   Legacy:   base_ms = row_count² / 10_000  (nested loop scan cost)
//!   Sovereign: exec_ms = row_count × 0.018   (hash join + index seek)
//!
//! At 500 rows: legacy ≈ 27ms, sovereign ≈ 9ms → 3× speedup.
//! At 6069 rows: legacy ≈ 3684ms, sovereign ≈ 109ms → 33× speedup.
//!
//! DUB.SAR 𒁾 — BahyWay.Ecosystem v4.0

// ── Mock Record ───────────────────────────────────────────────────────────────

/// A single row in the simulated SalesOrder result grid.
#[derive(Debug, Clone)]
pub struct SalesOrderRow {
    pub order_id:      u32,
    pub carrier_number: String,
    pub product_color: String,
    pub order_date:    String,
    pub customer_id:   u32,
    pub total_amount:  f32,
    pub status:        RowStatus,
}

/// Order lifecycle status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RowStatus {
    Active,
    Pending,
    Shipped,
    Cancelled,
}

impl std::fmt::Display for RowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RowStatus::Active    => f.write_str("Active"),
            RowStatus::Pending   => f.write_str("Pending"),
            RowStatus::Shipped   => f.write_str("Shipped"),
            RowStatus::Cancelled => f.write_str("Cancelled"),
        }
    }
}

// ── Execution Metrics ─────────────────────────────────────────────────────────

/// Simulated execution metrics for one plan (legacy or sovereign).
#[derive(Debug, Clone)]
pub struct ExecutionMetrics {
    pub plan_name:      String,
    pub execution_ms:   f32,
    pub logical_reads:  u64,
    pub physical_reads: u64,
    pub rows_returned:  usize,
    pub cpu_time_ms:    f32,
    pub is_sovereign:   bool,
}

impl ExecutionMetrics {
    /// Speedup ratio: `other.execution_ms / self.execution_ms`.
    /// Returns 1.0 when `self.execution_ms` is zero to avoid division errors.
    pub fn speedup_vs(&self, other: &ExecutionMetrics) -> f32 {
        if self.execution_ms > 0.0 {
            other.execution_ms / self.execution_ms
        } else { 1.0 }
    }
}

// ── Simulator ─────────────────────────────────────────────────────────────────

/// Stateless simulator for SalesOrder demo data and plan execution metrics.
pub struct DataSimulator;

impl DataSimulator {
    /// Generate `count` mock SalesOrder rows with deterministic, unique data.
    pub fn generate_rows(count: usize) -> Vec<SalesOrderRow> {
        const COLORS:   &[&str]      = &["Red","Black","Silver","Yellow","Blue","Multi","White","Green"];
        const STATUSES: &[RowStatus] = &[RowStatus::Active, RowStatus::Shipped,
                                         RowStatus::Pending, RowStatus::Cancelled];

        (0..count).map(|i| {
            let id = 43659 + i as u32;
            SalesOrderRow {
                order_id:      id,
                carrier_number: format!("4911-{:04X}-{}", id % 9999, (id * 7) % 999),
                product_color: COLORS[i % COLORS.len()].to_string(),
                order_date:    format!("2024-{:02}-{:02}", 1 + i % 12, 1 + i % 28),
                customer_id:   29825 + (i as u32 * 13) % 5000,
                total_amount:  24.99 + (i as f32 * 7.77) % 5000.0,
                status:        STATUSES[i % STATUSES.len()].clone(),
            }
        }).collect()
    }

    /// Simulate legacy plan execution metrics.
    ///
    /// Models a Nested Loop join: O(n²) scan cost with fragmentation penalty.
    pub fn legacy_metrics(row_count: usize, fragmentation_pct: f32) -> ExecutionMetrics {
        let base_ms      = (row_count as f32).powi(2) / 10_000.0;
        let frag_penalty = 1.0 + fragmentation_pct / 100.0;
        let exec_ms      = (base_ms * frag_penalty).min(9_999.0);

        ExecutionMetrics {
            plan_name:      "Legacy Plan (Nested Loop)".into(),
            execution_ms:   exec_ms,
            logical_reads:  row_count as u64 * 1250,
            physical_reads: row_count as u64 * 48,
            rows_returned:  row_count,
            cpu_time_ms:    exec_ms * 0.85,
            is_sovereign:   false,
        }
    }

    /// Simulate sovereign plan execution metrics.
    ///
    /// Models a Hash Join with Index Seeks: O(n) cost, no physical reads.
    pub fn sovereign_metrics(row_count: usize) -> ExecutionMetrics {
        let exec_ms = row_count as f32 * 0.018;
        ExecutionMetrics {
            plan_name:      "DMW Sovereign Plan (Hash Join)".into(),
            execution_ms:   exec_ms,
            logical_reads:  row_count as u64 * 8,
            physical_reads: 0,
            rows_returned:  row_count,
            cpu_time_ms:    exec_ms * 0.4,
            is_sovereign:   true,
        }
    }

    /// The classic DMW demo: 6,069 SalesOrder rows, 38% fragmentation.
    pub fn demo_comparison() -> (ExecutionMetrics, ExecutionMetrics) {
        (
            Self::legacy_metrics(6069, 38.0),
            Self::sovereign_metrics(6069),
        )
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn generates_correct_row_count() {
        assert_eq!(DataSimulator::generate_rows(500).len(), 500);
    }

    #[test]
    fn rows_have_unique_order_ids() {
        let rows = DataSimulator::generate_rows(100);
        let ids: HashSet<u32> = rows.iter().map(|r| r.order_id).collect();
        assert_eq!(ids.len(), 100, "order IDs must be unique");
    }

    #[test]
    fn sovereign_faster_than_legacy() {
        let l = DataSimulator::legacy_metrics(500, 38.0);
        let s = DataSimulator::sovereign_metrics(500);
        assert!(s.execution_ms < l.execution_ms,
            "sovereign {:.1}ms must be faster than legacy {:.1}ms",
            s.execution_ms, l.execution_ms);
    }

    #[test]
    fn sovereign_fewer_logical_reads() {
        let l = DataSimulator::legacy_metrics(500, 0.0);
        let s = DataSimulator::sovereign_metrics(500);
        assert!(s.logical_reads < l.logical_reads);
    }

    #[test]
    fn sovereign_zero_physical_reads() {
        let s = DataSimulator::sovereign_metrics(1000);
        assert_eq!(s.physical_reads, 0);
    }

    #[test]
    fn speedup_meaningful_at_500_rows() {
        let l = DataSimulator::legacy_metrics(500, 38.0);
        let s = DataSimulator::sovereign_metrics(500);
        assert!(s.speedup_vs(&l) > 2.0,
            "expected >2x speedup, got {:.1}x", s.speedup_vs(&l));
    }

    #[test]
    fn demo_comparison_sovereign_wins() {
        let (legacy, sovereign) = DataSimulator::demo_comparison();
        assert!(sovereign.execution_ms < legacy.execution_ms);
        assert!(sovereign.speedup_vs(&legacy) > 10.0,
            "6069 rows demo must show >10x speedup, got {:.1}x",
            sovereign.speedup_vs(&legacy));
    }

    #[test]
    fn fragmentation_makes_legacy_slower() {
        let clean  = DataSimulator::legacy_metrics(500, 0.0);
        let rotted = DataSimulator::legacy_metrics(500, 80.0);
        assert!(rotted.execution_ms > clean.execution_ms);
    }

    #[test]
    fn row_status_display_non_empty() {
        for s in [RowStatus::Active, RowStatus::Pending,
                  RowStatus::Shipped, RowStatus::Cancelled] {
            assert!(!format!("{s}").is_empty());
        }
    }

    #[test]
    fn sovereign_flag_set_correctly() {
        assert!(!DataSimulator::legacy_metrics(100, 0.0).is_sovereign);
        assert!( DataSimulator::sovereign_metrics(100).is_sovereign);
    }
}
