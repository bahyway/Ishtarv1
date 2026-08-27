//! 𒁾 EnkiDW CLI — BahyWay Data Warehouse pipeline and analytics.
//!
//! Usage:
//!   enkidw [--tribe <hex>] [--data-dir <dir>] [--landing <dir>] <command>
//!
//! Commands:
//!   ingest              Process all files currently in the landing zone (one shot)
//!   watch [--interval <s>] Poll the landing zone every N seconds (default 5)
//!   analytics [--top <n>]  Display DW analytics report
//!   compile <file.way>     Compile a .way security declaration to .akk

use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use bahyway_core::TribeId;
use enkidb_storage::FsyncPolicy;
use enkidw::{way_compile, DwAlertSeverity, DwAnalytics, EtlPipeline, WayFile};

// ── ANSI palette ──────────────────────────────────────────────────────────────

const RST: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const CYN: &str = "\x1b[0;36m";
const YLW: &str = "\x1b[0;33m";
const BRED: &str = "\x1b[1;31m";
const BGRN: &str = "\x1b[1;32m";
const BYLW: &str = "\x1b[1;33m";
const BCYN: &str = "\x1b[1;36m";
const BWHT: &str = "\x1b[1;37m";

// ── CLI arguments ─────────────────────────────────────────────────────────────

struct Args {
    tribe_id: u16,
    data_dir: String,
    landing_dir: String,
    command: String,
    command_args: Vec<String>,
}

fn parse_args() -> Args {
    let raw: Vec<String> = env::args().collect();
    let mut tribe_id = 0x0001u16;
    let mut data_dir = home_dir(".bahyway");
    let mut landing_dir = home_dir(".bahyway/landing");
    let mut i = 1;
    let mut command = String::new();
    let mut command_args = Vec::new();

    while i < raw.len() {
        match raw[i].as_str() {
            "--tribe" if i + 1 < raw.len() => {
                let hex = raw[i + 1].trim_start_matches("0x");
                tribe_id = u16::from_str_radix(hex, 16).unwrap_or(0x0001);
                i += 2;
            }
            "--data-dir" if i + 1 < raw.len() => {
                data_dir = raw[i + 1].clone();
                i += 2;
            }
            "--landing" if i + 1 < raw.len() => {
                landing_dir = raw[i + 1].clone();
                i += 2;
            }
            other if command.is_empty() && !other.starts_with('-') => {
                command = other.to_string();
                i += 1;
            }
            other => {
                command_args.push(other.to_string());
                i += 1;
            }
        }
    }

    Args {
        tribe_id,
        data_dir,
        landing_dir,
        command,
        command_args,
    }
}

fn home_dir(suffix: &str) -> String {
    env::var("HOME")
        .map(|h| format!("{h}/{suffix}"))
        .unwrap_or_else(|_| format!("/tmp/{suffix}"))
}

// ── Banner ────────────────────────────────────────────────────────────────────

fn banner(args: &Args) {
    println!();
    println!("  {BCYN}{BOLD}𒁾  EnkiDW v4.0{RST}");
    println!("  {DIM}BahyWay Sovereign Data Warehouse — Pure Rust, Zero Dependencies{RST}");
    println!("  {DIM}tribe      : {BCYN}{:#06x}{RST}", args.tribe_id);
    println!("  {DIM}data-dir   : {CYN}{}{RST}", args.data_dir);
    println!("  {DIM}landing    : {CYN}{}{RST}", args.landing_dir);
    println!();
}

// ── Commands ──────────────────────────────────────────────────────────────────

fn cmd_ingest(args: &Args) {
    let mut pipe = open_pipeline(args);
    println!("  {DIM}Polling landing zone…{RST}");
    let stats = pipe.run_once().clone();
    let alerts = pipe.take_alerts();

    for alert in &alerts {
        let (icon, col) = match alert.severity {
            DwAlertSeverity::Info => ("ℹ", CYN),
            DwAlertSeverity::Warning => ("⚠", BYLW),
            DwAlertSeverity::Error => ("✗", BRED),
        };
        println!("  {col}{icon}{RST}  {}", alert.message);
    }

    println!();
    println!("  {BOLD}Ingest summary{RST}");
    println!("  {DIM}files seen    :{RST}  {}", stats.files_seen);
    println!("  {DIM}zips          :{RST}  {}", stats.zips_processed);
    println!("  {DIM}way compiled  :{RST}  {}", stats.way_compiled);
    println!(
        "  {DIM}ingested      :{RST}  {BGRN}{}{RST}",
        stats.records_ingested
    );
    println!(
        "  {DIM}skipped       :{RST}  {BYLW}{}{RST}",
        stats.records_skipped
    );
    println!("  {DIM}alerts        :{RST}  {}", stats.alerts_emitted);
    println!();
}

fn cmd_watch(args: &Args, extra: &[String]) {
    let interval_secs = extra
        .windows(2)
        .find(|w| w[0] == "--interval")
        .and_then(|w| w[1].parse::<u64>().ok())
        .unwrap_or(5);

    println!("  {DIM}Watching landing zone every {interval_secs}s — Ctrl-C to stop{RST}");
    println!();

    let mut pipe = open_pipeline(args);
    let mut cycle = 0usize;

    loop {
        cycle += 1;
        let stats = pipe.run_once().clone();
        if stats.files_seen > 0 || stats.records_ingested > 0 {
            print!("  {DIM}[{cycle}]{RST} ");
            let _ = std::io::stdout().flush();
            for alert in pipe.take_alerts() {
                let (icon, col) = match alert.severity {
                    DwAlertSeverity::Info => ("ℹ", CYN),
                    DwAlertSeverity::Warning => ("⚠", BYLW),
                    DwAlertSeverity::Error => ("✗", BRED),
                };
                println!("  {col}{icon}{RST} {}", alert.message);
            }
            if stats.records_ingested > 0 {
                println!(
                    "  {BGRN}✓{RST} ingested={}  skipped={}",
                    stats.records_ingested, stats.records_skipped
                );
            }
        } else {
            print!("\r  {DIM}[{cycle}] idle…{RST}     ");
            let _ = std::io::stdout().flush();
            pipe.take_alerts(); // drain silently
        }
        thread::sleep(Duration::from_secs(interval_secs));
    }
}

fn cmd_analytics(args: &Args, extra: &[String]) {
    let top_n = extra
        .windows(2)
        .find(|w| w[0] == "--top")
        .and_then(|w| w[1].parse::<usize>().ok())
        .unwrap_or(10);

    let tid = TribeId::from_u16(args.tribe_id);
    let dir = PathBuf::from(&args.data_dir);
    let pdb =
        enkidb_persist::PersistedDb::open(&dir, tid, FsyncPolicy::Never).unwrap_or_else(|e| {
            eprintln!("error: {e}");
            std::process::exit(1);
        });

    let analytics = DwAnalytics::new(&pdb);
    let report = analytics.report(top_n);

    println!();
    println!(
        "  {BOLD}DW Analytics Report{RST}  {DIM}tribe {:#06x}{RST}",
        args.tribe_id
    );
    println!("  {DIM}{}{RST}", "─".repeat(52));
    println!(
        "  {DIM}total particles:{RST}  {BWHT}{}{RST}",
        report.total_particles
    );
    println!(
        "  {DIM}total events   :{RST}  {BWHT}{}{RST}",
        report.total_events
    );
    println!(
        "  {DIM}epoch range    :{RST}  {DIM}{}..{}{RST}",
        report.epoch_min, report.epoch_max
    );
    println!(
        "  {DIM}golden ratio   :{RST}  {BYLW}{:.1}%{RST}",
        report.golden_ratio() * 100.0
    );
    println!();
    println!("  {BOLD}State breakdown{RST}");
    for (state, count) in report.by_state() {
        let (col, sym) = match state {
            bahyway_core::ParticleState::Golden => (BYLW, "●"),
            bahyway_core::ParticleState::Fuzzy => (YLW, "◐"),
            bahyway_core::ParticleState::Dead => (DIM, "○"),
        };
        let pct = (count * 100)
            .checked_div(report.total_particles)
            .unwrap_or(0);
        let bar = "█".repeat((pct / 5).min(20));
        let sname = format!("{:?}", state);
        println!("  {col}{sym} {sname:<8}{RST}  {count:>6}  {col}{bar}{RST}");
    }
    println!();
    if !report.top_particles.is_empty() {
        println!("  {BOLD}Top {top_n} particles by event count{RST}");
        println!("  {DIM}{}{RST}", "─".repeat(40));
        for (rank, ps) in report.top_particles.iter().enumerate() {
            let (col, sym) = match ps.state {
                bahyway_core::ParticleState::Golden => (BYLW, "●"),
                bahyway_core::ParticleState::Fuzzy => (YLW, "◐"),
                bahyway_core::ParticleState::Dead => (DIM, "○"),
            };
            println!(
                "  {DIM}{:>2}.{RST}  {col}{sym}{RST}  {CYN}{:#010x}{RST}  ev={BWHT}{}{RST}",
                rank + 1,
                ps.uuid_hash,
                ps.event_count
            );
        }
        println!();
    }
}

fn cmd_compile(extra: &[String]) {
    let Some(path) = extra.first() else {
        eprintln!("Usage: enkidw compile <file.way>");
        std::process::exit(1);
    };
    let src = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Cannot read {path}: {e}");
            std::process::exit(1);
        }
    };
    let way = match WayFile::parse(&src) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("Parse error: {e}");
            std::process::exit(1);
        }
    };
    match way_compile(&way) {
        Ok(res) => {
            println!();
            println!("{BCYN}# Generated AAOL (.akk){RST}");
            println!("{DIM}{}{RST}", "─".repeat(40));
            println!("{}", res.akk_source);
        }
        Err(e) => {
            eprintln!("{BRED}Compile error: {e}{RST}");
            std::process::exit(1);
        }
    }
}

fn cmd_help() {
    println!();
    println!("  {BCYN}{BOLD}𒁾  EnkiDW v4.0{RST}  — BahyWay Sovereign Data Warehouse");
    println!();
    println!("  {BOLD}Usage:{RST}");
    println!("    enkidw [--tribe <hex>] [--data-dir <dir>] [--landing <dir>] <command>");
    println!();
    println!("  {BOLD}Global options:{RST}");
    println!("    {CYN}--tribe <hex>{RST}       Tribe ID (default 0x0001)");
    println!("    {CYN}--data-dir <dir>{RST}    Data directory (default ~/.bahyway)");
    println!(
        "    {CYN}--landing <dir>{RST}     Landing zone directory (default ~/.bahyway/landing)"
    );
    println!();
    println!("  {BOLD}Commands:{RST}");
    let cmds = [
        ("ingest", "Process all files currently in the landing zone"),
        (
            "watch [--interval <s>]",
            "Poll landing zone continuously (default 5s)",
        ),
        (
            "analytics [--top <n>]",
            "Show DW analytics report (default top 10)",
        ),
        (
            "compile <file.way>",
            "Compile a .way declaration to AAOL (.akk)",
        ),
    ];
    for (cmd, desc) in cmds {
        println!("    {BCYN}{cmd:<32}{RST}{DIM}{desc}{RST}");
    }
    println!();
    println!("  {DIM}Landing zone accepts: .zip (STORE method), .tsv, .csv, .way{RST}");
    println!();
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn open_pipeline(args: &Args) -> EtlPipeline {
    let tid = TribeId::from_u16(args.tribe_id);
    let landing = PathBuf::from(&args.landing_dir);
    let data_dir = PathBuf::from(&args.data_dir);
    EtlPipeline::open(&landing, &data_dir, tid, FsyncPolicy::PerCommit).unwrap_or_else(|e| {
        eprintln!("error: cannot open pipeline: {e}");
        std::process::exit(1);
    })
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() {
    let args = parse_args();

    match args.command.as_str() {
        "ingest" => {
            banner(&args);
            cmd_ingest(&args);
        }
        "watch" => {
            banner(&args);
            cmd_watch(&args, &args.command_args.clone());
        }
        "analytics" => {
            banner(&args);
            cmd_analytics(&args, &args.command_args.clone());
        }
        "compile" => {
            cmd_compile(&args.command_args.clone());
        }
        "help" | "--help" | "-h" | "" => cmd_help(),
        other => {
            eprintln!("Unknown command '{other}'. Run 'enkidw help' for usage.");
            std::process::exit(1);
        }
    }
}
