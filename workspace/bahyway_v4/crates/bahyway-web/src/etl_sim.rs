//! EnkiDB ETL pipeline simulation — pure Rust, no web_sys.
//! DOM/canvas rendering is done in lib.rs.

use bahyway_core::TribeId;
use enkidb_kaki::{IdentityKaki, KakiRole};
use enkisdb::sdb_store::{SdbStatus, StagedParticle};
use eridu_runtime::SchedulerLoop;

pub struct VisParticle {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub rgb: [u8; 3],
    pub vx: f32,
    pub vy: f32,
    pub vz: f32,
}

#[derive(Clone)]
pub enum LogKind {
    Info,
    Sweep,
    Malware,
}

#[derive(Clone)]
pub struct LogEntry {
    pub text: String,
    pub kind: LogKind,
}

pub struct EtlSimState {
    pub lp: SchedulerLoop,
    pub vis: Vec<VisParticle>,
    pub sdb_vis: Vec<[u8; 3]>, // staged particle colors (cleared on sweep)
    pub qdb_vis: Vec<[u8; 3]>, // quarantined particle colors
    pub last_sweep_at: u64,    // tick when the last ValidationSweep ran
    pub auto_tick: bool,
    pub tick_count: u64,
    pub log: Vec<LogEntry>,
    rng: u64,
}

impl EtlSimState {
    pub fn new() -> Self {
        let lp = SchedulerLoop::with_interval(TribeId::from_u16(0x0001), 16, 5);
        let mut s = EtlSimState {
            lp,
            vis: Vec::new(),
            sdb_vis: Vec::new(),
            qdb_vis: Vec::new(),
            last_sweep_at: 0,
            auto_tick: true,
            tick_count: 0,
            log: Vec::new(),
            rng: 0xDEAD_BEEF_1337_u64,
        };
        s.push_log("EnkiDB pipeline ready. Auto-tick on.", LogKind::Info);
        s
    }
}

impl Default for EtlSimState {
    fn default() -> Self {
        Self::new()
    }
}

impl EtlSimState {
    fn rng_next(&mut self) -> u64 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        self.rng
    }

    fn rand_f32(&mut self, lo: f32, hi: f32) -> f32 {
        let r = (self.rng_next() & 0xFFFF) as f32 / 65535.0;
        lo + r * (hi - lo)
    }

    pub fn stage_valid(&mut self) {
        let ik = IdentityKaki::try_from_kaki(self.lp.minter.identity(KakiRole::Zikru)).unwrap();
        let r = self.rand_f32(60.0, 200.0) as u8;
        let g = self.rand_f32(150.0, 255.0) as u8;
        let b = self.rand_f32(180.0, 255.0) as u8;
        self.sdb_vis.push([r, g, b]);
        self.lp.sdb.stage(StagedParticle {
            kaki_bytes: *ik.bytes(),
            tribe_id: self.lp.tribe_id.as_u16(),
            epoch: 1,
            eav: Vec::new(),
            color_rgb: [r, g, b],
            status: SdbStatus::Pending,
            arrived_tick: self.tick_count,
            malware_flag: false,
        });
        let n = self.lp.sdb.stats().pending_count();
        self.push_log(
            format!("Staged valid particle → SDB pending={n}"),
            LogKind::Info,
        );
    }

    pub fn stage_malware(&mut self) {
        let ik = IdentityKaki::try_from_kaki(self.lp.minter.identity(KakiRole::Zikru)).unwrap();
        self.sdb_vis.push([220, 40, 40]);
        self.lp.sdb.stage(StagedParticle {
            kaki_bytes: *ik.bytes(),
            tribe_id: self.lp.tribe_id.as_u16(),
            epoch: 1,
            eav: Vec::new(),
            color_rgb: [220, 40, 40],
            status: SdbStatus::Pending,
            arrived_tick: self.tick_count,
            malware_flag: true,
        });
        let n = self.lp.sdb.stats().pending_count();
        self.push_log(
            format!("Staged MALWARE particle → SDB pending={n}"),
            LogKind::Malware,
        );
    }

    pub fn do_tick(&mut self) {
        self.tick_count += 1;
        let out = self.lp.tick(1);
        if out.sweep_ran {
            self.push_log(
                format!(
                    "Tick {} — sweep ran: +{} ODB  +{} QDB",
                    self.tick_count, out.promoted, out.quarantined
                ),
                LogKind::Sweep,
            );
            self.rebuild_vis();
        }
    }

    fn rebuild_vis(&mut self) {
        self.vis.clear();
        self.sdb_vis.clear();

        let odb_colors: Vec<[u8; 3]> = self.lp.odb.all().iter().map(|p| p.color_rgb).collect();
        for rgb in odb_colors {
            let x = self.rand_f32(-1.0, 1.0);
            let y = self.rand_f32(-1.0, 1.0);
            let z = self.rand_f32(-1.0, 1.0);
            let vx = self.rand_f32(-0.005, 0.005);
            let vy = self.rand_f32(-0.005, 0.005);
            let vz = self.rand_f32(-0.003, 0.003);
            self.vis.push(VisParticle {
                x,
                y,
                z,
                rgb,
                vx,
                vy,
                vz,
            });
        }

        self.qdb_vis = self.lp.qdb.all().iter().map(|r| r.color_rgb).collect();
        self.last_sweep_at = self.tick_count;
    }

    pub fn animate_vis(&mut self) {
        for p in &mut self.vis {
            p.x += p.vx;
            p.y += p.vy;
            p.z += p.vz;
            if p.x.abs() > 1.0 {
                p.vx *= -1.0;
            }
            if p.y.abs() > 1.0 {
                p.vy *= -1.0;
            }
            if p.z.abs() > 1.0 {
                p.vz *= -1.0;
            }
        }
    }

    pub fn sdb_pending(&self) -> usize {
        self.lp.sdb.stats().pending_count()
    }
    pub fn odb_active(&self) -> usize {
        self.lp.odb.stats().active_count
    }
    pub fn qdb_count(&self) -> usize {
        self.lp.qdb.len()
    }
    pub fn journal_entries(&self) -> usize {
        self.lp.journal.entry_count()
    }

    fn push_log(&mut self, text: impl Into<String>, kind: LogKind) {
        self.log.push(LogEntry {
            text: text.into(),
            kind,
        });
        if self.log.len() > 40 {
            self.log.remove(0);
        }
    }
}
