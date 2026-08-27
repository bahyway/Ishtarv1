#![forbid(unsafe_code)]
pub use fuzzy_engine::{
    FuzzyDimensions, QualityTier, ACTIVE_THRESHOLD, GEM_THRESHOLD, TRIBE_THRESHOLD,
};
pub use sla_engine::{GO_LIVE_THRESHOLD, QUALITY_DIVISOR};
use std::collections::VecDeque;
use std::sync::{mpsc, Arc, RwLock};

pub const MAX_ALERTS: usize = 64;
pub const HEPTA_GATE_COUNT: usize = 7;
pub const B11_DIVISOR: u16 = 240;

#[derive(Debug, Clone, Default)]
pub struct EtlTierCounts {
    pub sdb_pending: usize,
    pub sdb_promoted: usize,
    pub sdb_quarantined: usize,
    pub sdb_malware_hits: usize,
    pub sdb_tribal_rgb: [u8; 3],
    pub qdb_total: usize,
    pub qdb_malware_count: usize,
    pub qdb_tribal_rgb: [u8; 3],
    pub odb_total: usize,
    pub odb_tribal_rgb: [u8; 3],
    pub dw_total: usize,
    pub dw_tribal_rgb: [u8; 3],
    pub db_total: usize,
    pub db_tribal_rgb: [u8; 3],
    pub current_tick: u64,
    pub sweep_interval: u64,
    pub ticks_until_sweep: u64,
}

#[derive(Debug, Clone)]
pub struct FuzzySnapshot {
    pub dims: FuzzyDimensions,
    pub b11: u16,
    pub tier: QualityTier,
    pub b11_sum: u64,
    pub scored_count: u64,
}
impl Default for FuzzySnapshot {
    fn default() -> Self {
        FuzzySnapshot {
            dims: FuzzyDimensions {
                d1_name_completeness: 0.0,
                d2_name_consistency: 0.0,
                d3_id_validity: 0.0,
                d4_date_validity: 0.0,
                d5_geo_precision: 0.0,
                d6_source_trust: fuzzy_engine::SourceTrust::Unknown,
                d7_duplicate_proximity: 0.0,
                d8_arabic_density: 0.0,
                d9_orbital_trust_penalty: 0.0,
            },
            b11: 0,
            tier: QualityTier::NonActive,
            b11_sum: 0,
            scored_count: 0,
        }
    }
}
impl FuzzySnapshot {
    pub fn avg_b11(&self) -> u16 {
        self.b11_sum.checked_div(self.scored_count).unwrap_or(0) as u16
    }
}

#[derive(Debug, Clone, Default)]
pub struct GateLoads {
    pub counts: [usize; HEPTA_GATE_COUNT],
}
impl GateLoads {
    pub fn g1_adad(&self) -> usize {
        self.counts[0]
    }
    pub fn g7_nergal(&self) -> usize {
        self.counts[6]
    }
    pub fn total_in_flight(&self) -> usize {
        self.counts.iter().sum()
    }
}

#[derive(Debug, Clone, Default)]
pub struct DiagnosisStats {
    pub watch: usize,
    pub warning: usize,
    pub critical: usize,
}
impl DiagnosisStats {
    pub fn is_healthy(&self) -> bool {
        self.critical == 0 && self.warning == 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlertLevel {
    Watch,
    Warning,
    Critical,
}

#[derive(Debug, Clone)]
pub struct BusAlert {
    pub alert_id: String,
    pub time_display: String,
    pub level: AlertLevel,
    pub message_en: String,
    pub message_ar: String,
    pub b11: Option<u16>,
    pub kaki_hex: Option<String>,
}
impl BusAlert {
    pub fn etl(
        id: impl Into<String>,
        t: impl Into<String>,
        level: AlertLevel,
        en: impl Into<String>,
        ar: impl Into<String>,
        b11: Option<u16>,
        kaki: Option<String>,
    ) -> Self {
        BusAlert {
            alert_id: id.into(),
            time_display: t.into(),
            level,
            message_en: en.into(),
            message_ar: ar.into(),
            b11,
            kaki_hex: kaki,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SlaSnapshot {
    pub overall_b11: u8,
    pub go_live_ready: bool,
    pub domain_scores: Vec<(String, u8)>,
    pub actions: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum BusCmd {
    AccelerateIntake,
    ProcessBadData,
    Pause,
    Resume,
    ForceSweep,
    Shutdown,
}

pub struct BeeMdmBusState {
    pub etl: EtlTierCounts,
    pub fuzzy: FuzzySnapshot,
    pub gates: GateLoads,
    pub diagnosis: DiagnosisStats,
    pub sla: SlaSnapshot,
    pub alerts: VecDeque<BusAlert>,
    pub total_processed: u64,
    pub total_golden: u64,
    pub total_quarantined: u64,
    pub paused: bool,
    pub cmd_tx: mpsc::Sender<BusCmd>,
}

impl BeeMdmBusState {
    pub fn update_fuzzy(&mut self, dims: FuzzyDimensions, b11: u8) {
        self.fuzzy.tier = fuzzy_engine::FuzzyEngine::classify(b11);
        self.fuzzy.dims = dims;
        self.fuzzy.b11 = b11 as u16;
        self.fuzzy.b11_sum += b11 as u64;
        self.fuzzy.scored_count += 1;
    }
    pub fn update_gates(&mut self, c: [usize; HEPTA_GATE_COUNT]) {
        self.gates.counts = c;
    }
    pub fn update_diagnosis(&mut self, w: usize, wa: usize, c: usize) {
        self.diagnosis = DiagnosisStats {
            watch: w,
            warning: wa,
            critical: c,
        };
    }
    pub fn push_alert(&mut self, a: BusAlert) {
        if self.alerts.len() >= MAX_ALERTS {
            self.alerts.pop_back();
        }
        match a.level {
            AlertLevel::Watch => self.diagnosis.watch += 1,
            AlertLevel::Warning => self.diagnosis.warning += 1,
            AlertLevel::Critical => self.diagnosis.critical += 1,
        }
        self.alerts.push_front(a);
    }
    pub fn record_particle(&mut self, golden: bool, quarantined: bool) {
        self.total_processed += 1;
        if golden {
            self.total_golden += 1;
        }
        if quarantined {
            self.total_quarantined += 1;
        }
    }
    pub fn golden_rate_pct(&self) -> f32 {
        if self.total_processed == 0 {
            0.0
        } else {
            self.total_golden as f32 / self.total_processed as f32 * 100.0
        }
    }
    pub fn b11_fraction(&self) -> f32 {
        self.fuzzy.b11 as f32 / B11_DIVISOR as f32
    }
    pub fn send_cmd(&self, cmd: BusCmd) -> bool {
        self.cmd_tx.send(cmd).is_ok()
    }
}

pub type BusHandle = Arc<RwLock<BeeMdmBusState>>;

pub fn new_bus() -> (BusHandle, mpsc::Receiver<BusCmd>) {
    let (tx, rx) = mpsc::channel::<BusCmd>();
    let state = BeeMdmBusState {
        etl: EtlTierCounts::default(),
        fuzzy: FuzzySnapshot::default(),
        gates: GateLoads::default(),
        diagnosis: DiagnosisStats::default(),
        sla: SlaSnapshot::default(),
        alerts: VecDeque::with_capacity(MAX_ALERTS),
        total_processed: 0,
        total_golden: 0,
        total_quarantined: 0,
        paused: false,
        cmd_tx: tx,
    };
    (Arc::new(RwLock::new(state)), rx)
}

pub fn populate_demo(bus: &BusHandle) {
    let mut s = bus.write().unwrap();
    s.etl = EtlTierCounts {
        sdb_pending: 42,
        sdb_promoted: 180,
        sdb_quarantined: 8,
        sdb_malware_hits: 0,
        sdb_tribal_rgb: [100, 180, 220],
        qdb_total: 8,
        qdb_malware_count: 2,
        qdb_tribal_rgb: [200, 60, 60],
        odb_total: 180,
        odb_tribal_rgb: [80, 200, 120],
        dw_total: 520,
        dw_tribal_rgb: [200, 160, 40],
        db_total: 15,
        db_tribal_rgb: [120, 120, 220],
        current_tick: 4320,
        sweep_interval: 900,
        ticks_until_sweep: 213,
    };
    s.gates.counts = [9, 1, 4, 5, 3, 3, 0];
    s.diagnosis = DiagnosisStats {
        watch: 7,
        warning: 3,
        critical: 1,
    };
    s.total_processed = 20_000;
    s.total_golden = 520;
    s.total_quarantined = 31;
    s.alerts.push_front(BusAlert::etl(
        "ETL-4320-001",
        "12:00:31",
        AlertLevel::Critical,
        "Malware detected in batch_003.zip",
        "اكتشاف برمجية خبيثة في الدُفعة 003",
        None,
        None,
    ));
    s.alerts.push_front(BusAlert::etl(
        "ETL-4310-002",
        "11:58:47",
        AlertLevel::Warning,
        "31 staged exceptions awaiting steward review",
        "31 استثناء في الانتظار",
        Some(178),
        None,
    ));
}
