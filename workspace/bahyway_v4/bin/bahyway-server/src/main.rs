//! bahyway-server — BahyWay Sovereign Ecosystem v4.0 daemon.
//!
//! Starts the EriduSupervisor, registers default scheduled jobs, and
//! runs a synchronous tick loop. No async, no tokio — pure Rust.
//!
//! Usage: bahyway-server [--tribe <hex>] [--ticks <n>]

use bahyway_core::TribeId;
use enkidb_engine::EnkiDb;
use eridu_runtime::{Task, TaskResult};
use eridu_scheduler::ScheduledJob;
use eridu_supervisor::EriduSupervisor;
use std::env;

struct HeartbeatTask {
    tick: u64,
}
impl Task for HeartbeatTask {
    fn name(&self) -> &str {
        "heartbeat"
    }
    fn run(&mut self) -> TaskResult {
        eprintln!("[server] heartbeat tick={}", self.tick);
        TaskResult::Ok
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut tribe_id = 0x0001u16;
    let mut ticks = 100u64;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--tribe" if i + 1 < args.len() => {
                let hex = args[i + 1].trim_start_matches("0x");
                tribe_id = u16::from_str_radix(hex, 16).unwrap_or(0x0001);
                i += 2;
            }
            "--ticks" if i + 1 < args.len() => {
                ticks = args[i + 1].parse().unwrap_or(100);
                i += 2;
            }
            "--help" | "-h" => {
                eprintln!("Usage: bahyway-server [--tribe <hex>] [--ticks <n>]");
                std::process::exit(0);
            }
            _ => {
                i += 1;
            }
        }
    }

    eprintln!("𒁾 bahyway-server — BahyWay.Ecosystem v4.0");
    eprintln!("  tribe   : {tribe_id:#06x}");
    eprintln!("  ticks   : {ticks}");

    let tid = TribeId::from_u16(tribe_id);
    let _db = EnkiDb::new(tid);
    let mut sup = EriduSupervisor::new();
    sup.start();
    sup.register_job(ScheduledJob::new("heartbeat", 10));
    sup.register_job(ScheduledJob::new("snapshot", 50));

    let mut tick_no = 0u64;
    while tick_no < ticks {
        let t = tick_no;
        let due = sup.tick(1, |_name| {
            Box::new(HeartbeatTask { tick: t }) as Box<dyn eridu_runtime::Task>
        });
        if !due.is_empty() {
            sup.run_pending();
        }
        tick_no += 1;
    }

    sup.shutdown();
    eprintln!("  ok={} fail={}", sup.runtime.completed, sup.runtime.failed);
    eprintln!("𒁾 shutdown complete.");
}
