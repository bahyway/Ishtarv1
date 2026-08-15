//! 𒁾 DubSar IDE v4.0 — BahyWay Sovereign Terminal Environment
//!
//! Named after Dubsar (𒁾), the Sumerian scribe who kept the sovereign tablets.
//!
//! Commands
//! ────────
//!   .naba                     Summon NABA — Sovereign Hologram Agent
//!   .help                     Show this help
//!   .demo                     Load 5 sample particles into the session DB
//!   .ingest <Golden|Fuzzy|Dead>  Add one particle manually
//!   .particles                List all particles with ColorID visualization
//!   .query <hepta-source>     Run an inline HeptaScript query
//!   .open <file>              Load a .akk or .hepta file into the buffer
//!   .check                    Syntax-check the loaded buffer
//!   .run                      Execute the loaded .hepta buffer as a query
//!   .templates                List all registered templates with their fields
//!   .fields <template-name>   Show field-completion hints for a template
//!   .color <r> <g> <b>        Preview a ColorID (values 0–255)
//!   .status                   Session statistics
//!   .quit / .exit             Exit

mod naba;
mod dashboard;

use std::env;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

use bahyway_core::{ParticleState, TribeId};
use enkidb_query::query;
use enkidb_persist::PersistedDb;
use enkidb_storage::FsyncPolicy;
use adad_gate::{AdadGate, ArrivalRecord};
use enkidb_kaki::KakiRole;
use musaru_security::check_sovereignty;
use vgca_validation::validate;
use template_library::{civil_registry_template, load_defaults};
use story_engine::projection::{encode_state, ATTR_STATE};
use score_engine::color_id::ColorRgb;
use dubsar_ide::DubSarIde;
use dubsar_visualizer::render;

// ── ANSI palette ─────────────────────────────────────────────────────────────

const RST:  &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM:  &str = "\x1b[2m";
const YLW:  &str = "\x1b[0;33m";
const CYN:  &str = "\x1b[0;36m";
const BRED: &str = "\x1b[1;31m";
const BGRN: &str = "\x1b[1;32m";
const BYLW: &str = "\x1b[1;33m";
const BCYN: &str = "\x1b[1;36m";
const BWHT: &str = "\x1b[1;37m";

fn fg(r: u8, g: u8, b: u8) -> String { format!("\x1b[38;2;{r};{g};{b}m") }
fn bg(r: u8, g: u8, b: u8) -> String { format!("\x1b[48;2;{r};{g};{b}m") }

fn state_rgb(s: ParticleState) -> (u8, u8, u8) {
    match s {
        ParticleState::Golden => (0xFF, 0xFF, 0x40),
        ParticleState::Fuzzy  => (0xFF, 0xA5, 0x00),
        ParticleState::Dead   => (0x80, 0x80, 0x80),
    }
}

fn fmt_state(s: ParticleState) -> String {
    let (c, sym, name) = match s {
        ParticleState::Golden => (BYLW, "●", "GOLDEN "),
        ParticleState::Fuzzy  => (YLW,  "◐", "FUZZY  "),
        ParticleState::Dead   => (DIM,  "○", "DEAD   "),
    };
    format!("{BOLD}{c}{sym} {name}{RST}")
}

// ── File buffer ───────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
enum FileKind { Akk, Hepta, Other }

#[derive(Clone)]
struct FileBuffer {
    path:    String,
    content: String,
    kind:    FileKind,
}

impl FileBuffer {
    fn load(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Cannot read '{path}': {e}"))?;
        let kind = if path.ends_with(".akk")   { FileKind::Akk }
                   else if path.ends_with(".hepta") { FileKind::Hepta }
                   else { FileKind::Other };
        Ok(FileBuffer { path: path.to_string(), content, kind })
    }

    fn kind_label(&self) -> &'static str {
        match self.kind { FileKind::Akk => "AAOL (.akk)", FileKind::Hepta => "HeptaScript (.hepta)", FileKind::Other => "text" }
    }
}

// ── Session ───────────────────────────────────────────────────────────────────

#[derive(Default)]
struct Stats { ingested: usize, queries: usize, checks: usize, errors: usize }

struct Session {
    pdb:      PersistedDb,
    gate:     AdadGate,
    ide:      DubSarIde,
    stats:    Stats,
    buf:      Option<FileBuffer>,
    tribe:    u16,
    epoch:    u32,
    data_dir: String,
}

impl Session {
    fn new(tribe_id: u16, data_dir: &str) -> Result<Self, String> {
        let tid     = TribeId::from_u16(tribe_id);
        let dir     = PathBuf::from(data_dir);
        let pdb     = PersistedDb::open(&dir, tid, FsyncPolicy::PerCommit)
            .map_err(|e| format!("Cannot open database at '{data_dir}': {e}"))?;
        let mut ide = DubSarIde::new();
        load_defaults(&mut ide.registry);
        Ok(Session {
            pdb,
            gate:     AdadGate::new(tid),
            ide,
            stats:    Stats::default(),
            buf:      None,
            tribe:    tribe_id,
            epoch:    0,
            data_dir: data_dir.to_string(),
        })
    }

    fn add_particle(&mut self, state: ParticleState) -> Result<u32, String> {
        self.epoch += 1;
        let ep = self.epoch;

        let record = ArrivalRecord {
            attrs: vec![(ATTR_STATE, encode_state(state).to_vec())],
            epoch: ep,
            role:  KakiRole::Zikru,
        };
        let gr  = self.gate.ingest(record).map_err(|e| e.to_string())?;
        let sec = check_sovereignty(self.gate.tribe_id(), &gr.particle);
        if !sec.is_approved() {
            return Err(format!("sovereignty rejected: {sec:?}"));
        }
        let tmpl = civil_registry_template();
        let vr   = validate(&tmpl, &gr.eav);
        if !vr.is_valid() {
            return Err(format!("missing required fields: {:?}", vr.missing_required));
        }
        self.pdb.register_particle(&gr.particle).map_err(|e| e.to_string())?;
        self.pdb.commit(gr.event_kaki, gr.particle, gr.epoch, gr.eav)
            .map_err(|e| e.to_string())?;
        self.stats.ingested += 1;
        Ok(ep)
    }
}

// ── Particle drift bar ────────────────────────────────────────────────────────

fn drift_bar(r: u8, g: u8, b: u8) -> String {
    let dr = 255u32.saturating_sub(r as u32) as f32;
    let dg = 255u32.saturating_sub(g as u32) as f32;
    let db = 255u32.saturating_sub(b as u32) as f32;
    let dist   = (dr*dr + dg*dg + db*db).sqrt();
    let max_d  = (3.0f32 * 255.0f32 * 255.0f32).sqrt();
    let health = 1.0 - (dist / max_d);
    let filled = (health * 8.0).round() as usize;
    format!("{}{}{DIM}{}{RST}",
        fg(r, g, b),
        "█".repeat(filled.min(8)),
        "░".repeat(8usize.saturating_sub(filled)),
    )
}

// ── Command handlers ──────────────────────────────────────────────────────────

fn do_naba(s: &Session) {
    let m = naba::NabaMetrics::gather(&s.pdb, s.tribe, s.epoch);
    naba::render(&m);
}

fn do_help() {
    println!();
    println!("  {BCYN}Command           Description{RST}");
    println!("  {DIM}{}{RST}", "─".repeat(54));
    let rows = [
        (".naba",                   "Summon NABA — Sovereign Hologram Agent (live metrics)"),
        (".demo",                   "Load 5 sample particles (Golden×2, Fuzzy×2, Dead×1)"),
        (".ingest <state>",         "Add a particle  Golden | Fuzzy | Dead"),
        (".particles",              "List all particles with ColorID visualization"),
        (".query <hepta>",          "Run an inline HeptaScript query"),
        (".open <file>",            "Load a .akk or .hepta file into the buffer"),
        (".check",                  "Syntax-check the loaded buffer"),
        (".run",                    "Execute the loaded .hepta buffer as a query"),
        (".templates",              "List registered templates with field counts"),
        (".fields <template>",      "Show field-completion hints for a template"),
        (".color <r> <g> <b>",      "Preview a ColorID (three bytes 0-255)"),
        (".dama",                   "Show all 11 DAMA knowledge areas with term counts"),
        (".dama <code>",            "Look up a DAMA term (e.g. .dama DQ.FRESHNESS)"),
        (".dama search <keyword>",  "Search the DAMA dictionary (e.g. .dama search golden)"),
        (".dashboard",              "Generate showway.html — self-contained HTML dashboard"),
        (".status",                 "Session statistics"),
        (".quit / .exit",           "Exit DubSar IDE"),
    ];
    for (cmd, desc) in rows {
        println!("  {BCYN}{cmd:<26}{RST}{DIM}{desc}{RST}");
    }
    println!();
}

fn do_demo(s: &mut Session) {
    let items = [
        (ParticleState::Golden, "Ali_Karim"),
        (ParticleState::Golden, "Fatima_Zahra"),
        (ParticleState::Fuzzy,  "Hassan_Mahdi"),
        (ParticleState::Fuzzy,  "Zainab_Nasir"),
        (ParticleState::Dead,   "Omar_Jawad"),
    ];
    let mut ok = 0usize;
    for (state, name) in items {
        match s.add_particle(state) {
            Ok(ep)  => {
                println!("  {BGRN}+{RST} ep={ep:>2}  {:<14} {}", name, fmt_state(state));
                ok += 1;
            }
            Err(e)  => println!("  {BRED}✗{RST} {name}: {e}"),
        }
    }
    println!("  {DIM}── {ok} demo particles loaded ──{RST}");
}

fn do_ingest(s: &mut Session, arg: &str) {
    let state = match arg.to_lowercase().as_str() {
        "golden" => ParticleState::Golden,
        "fuzzy"  => ParticleState::Fuzzy,
        "dead"   => ParticleState::Dead,
        other    => {
            println!("  {BRED}Unknown state '{other}' — use Golden / Fuzzy / Dead{RST}");
            return;
        }
    };
    match s.add_particle(state) {
        Ok(ep)  => println!("  {BGRN}✓{RST} epoch={ep}  {}", fmt_state(state)),
        Err(e)  => println!("  {BRED}✗ {e}{RST}"),
    }
}

fn do_particles(s: &mut Session) {
    let particles = s.pdb.db().journal().all_particles();
    if particles.is_empty() {
        println!("  {DIM}No particles yet — try .demo or .ingest Golden{RST}");
        return;
    }
    println!();
    println!("  {BOLD}{DIM}  uuid_hash    state         ep   colorid  drift{RST}");
    println!("  {DIM}{}{RST}", "─".repeat(56));
    for p in &particles {
        let proj   = s.pdb.db().project(p);
        let state  = proj.state;
        let evs    = proj.events_seen;
        let (r, g, b) = state_rgb(state);
        let swatch = format!("{}{}{RST}", bg(r, g, b), "  ");
        let bar    = drift_bar(r, g, b);
        println!("  {swatch} {DIM}{:#010x}{RST}  {}  ev={evs:<2}  {bar}",
            p.uuid_hash(), fmt_state(state));
    }
    println!("  {DIM}{}{RST}", "─".repeat(56));
    println!("  {BGRN}{}{RST}{DIM} particle(s){RST}", particles.len());
    println!();
}

fn do_query(s: &mut Session, src: &str) {
    s.stats.queries += 1;
    match query(s.pdb.db(), src) {
        Ok(res) => {
            let pct = if res.evaluated > 0 { res.matched.len() * 100 / res.evaluated } else { 0 };
            println!("  {BGRN}matched={}{RST}  evaluated={}  selectivity={pct}%",
                res.matched.len(), res.evaluated);
            for p in &res.matched {
                let state = s.pdb.db().project(&p.entity).state;
                println!("    {CYN}→{RST} {:#010x}  {}", p.entity.uuid_hash(), fmt_state(state));
            }
        }
        Err(e) => {
            s.stats.errors += 1;
            println!("  {BRED}Query error: {e}{RST}");
        }
    }
}

fn do_open(s: &mut Session, path: &str) {
    match FileBuffer::load(path) {
        Ok(buf) => {
            println!("  {BGRN}✓{RST} {BOLD}{}{RST}  [{CYN}{}{RST}]  {} bytes",
                buf.path, buf.kind_label(), buf.content.len());
            println!();
            for (i, line) in buf.content.lines().enumerate() {
                let n   = i + 1;
                let col = match buf.kind { FileKind::Akk => CYN, FileKind::Hepta => BYLW, _ => RST };
                println!("  {DIM}{n:>3}{RST}  {DIM}│{RST}  {col}{line}{RST}");
            }
            println!();
            s.buf = Some(buf);
        }
        Err(e) => println!("  {BRED}✗ {e}{RST}"),
    }
}

fn do_check(s: &mut Session) {
    s.stats.checks += 1;
    let (content, kind) = match s.buf.as_ref() {
        Some(b) => (b.content.clone(), b.kind.clone()),
        None    => { println!("  {DIM}No file loaded — use .open <file>{RST}"); return; }
    };
    let diags = match kind {
        FileKind::Akk   => s.ide.check_aaol(&content),
        FileKind::Hepta => s.ide.check_hepta(&content),
        FileKind::Other => { println!("  {DIM}Unknown file type — use .akk or .hepta extension{RST}"); return; }
    };
    if diags.is_empty() {
        println!("  {BGRN}✓{RST} No diagnostics — file is valid");
    } else {
        s.stats.errors += diags.len();
        println!("  {BYLW}⚠{RST} {BOLD}{}{RST} diagnostic(s):", diags.len());
        for d in &diags {
            let icon = if d.is_error { format!("{BRED}✗{RST}") } else { format!("{BYLW}⚠{RST}") };
            println!("    {icon} {}", d.message);
        }
    }
}

fn do_run(s: &mut Session) {
    let content = match s.buf.as_ref() {
        Some(b) if matches!(b.kind, FileKind::Hepta) => b.content.clone(),
        Some(_) => { println!("  {DIM}Loaded file is not a .hepta query{RST}"); return; }
        None    => { println!("  {DIM}No file loaded — use .open <file.hepta>{RST}"); return; }
    };
    do_query(s, &content);
}

fn do_templates(s: &Session) {
    println!();
    for name in &["civil.registry", "operational", "sensor.stream"] {
        let fields = s.ide.complete_fields(name);
        println!("  {BCYN}{name}{RST}  {DIM}({} fields){RST}", fields.len());
        for f in &fields {
            println!("    {DIM}·{RST} {f}");
        }
    }
    println!();
}

fn do_fields(s: &Session, name: &str) {
    let fields = s.ide.complete_fields(name);
    if fields.is_empty() {
        println!("  {DIM}Template '{name}' not found — try .templates{RST}");
    } else {
        println!("  {BCYN}{name}{RST}  — {DIM}{} field(s){RST}", fields.len());
        for f in &fields {
            println!("    {CYN}·{RST} {f}");
        }
    }
}

fn do_color(args: &str) {
    let parts: Vec<&str> = args.split_whitespace().collect();
    if parts.len() < 3 {
        println!("  {DIM}Usage: .color <r> <g> <b>   (values 0–255){RST}");
        return;
    }
    let r = parts[0].parse::<u8>().unwrap_or(0);
    let g = parts[1].parse::<u8>().unwrap_or(0);
    let b = parts[2].parse::<u8>().unwrap_or(0);
    let rgb  = ColorRgb::new(r, g, b);
    let disp = render(&rgb);
    println!();
    println!("  {}{}  ColorID  {}  rgb({r},{g},{b})",
        bg(r, g, b), BWHT, RST);
    println!("  {}{BOLD}label  :{RST}  {}", disp.ansi_fg, disp.label);
    println!("  {}{BOLD}drift  :{RST}  {:.2}", disp.ansi_fg, disp.drift);
    println!("  {}{BOLD}bar    :{RST}  {}", disp.ansi_fg, disp.drift_bar);
    println!("  {RST}");
}

fn do_dama(arg: &str) {
    use damadmbok_dictionary::{KnowledgeArea, lookup, search, by_area, ALIGNMENTS};

    let arg = arg.trim();

    if arg.is_empty() {
        // Show all 11 KAs with term counts and alignment counts
        println!();
        println!("  {BCYN}{BOLD}DAMA-DMBOK Knowledge Areas — BahyWay Sovereign Dictionary{RST}");
        println!("  {DIM}{}{RST}", "─".repeat(62));
        for ka in KnowledgeArea::all() {
            let terms: Vec<_> = by_area(*ka).collect();
            let aligns: Vec<_> = ALIGNMENTS.iter().filter(|a| {
                a.dama_code.starts_with(ka.code())
            }).collect();
            println!("  {BCYN}Ch.{:<2}{RST}  {BYLW}{:<4}{RST}  {BWHT}{:<42}{RST}  {DIM}{:>3} terms  {:>2} aligned{RST}",
                ka.chapter_number(), ka.code(), ka.label(),
                terms.len(), aligns.len());
        }
        println!();
        let total_terms = damadmbok_dictionary::DICTIONARY.len();
        let total_aligns = ALIGNMENTS.len();
        println!("  {DIM}Dictionary: {BGRN}{total_terms}{RST}{DIM} terms  ·  Alignments: {BGRN}{total_aligns}{RST}{DIM} BahyWay components{RST}");
        println!("  {DIM}Usage: {BCYN}.dama <CODE>{RST}{DIM}  or  {BCYN}.dama search <keyword>{RST}");
        println!();
        return;
    }

    if let Some(kw) = arg.strip_prefix("search ") {
        // Search by keyword
        let hits = search(kw.trim());
        if hits.is_empty() {
            println!("  {DIM}No terms found for '{kw}'{RST}");
        } else {
            println!();
            println!("  {BCYN}Search: \"{kw}\" — {} result(s){RST}", hits.len());
            println!("  {DIM}{}{RST}", "─".repeat(54));
            for t in &hits {
                println!("  {BYLW}{:<20}{RST}  {BWHT}{:<30}{RST}", t.code, t.label);
                println!("  {DIM}  {}{RST}", t.definition);
            }
            println!();
        }
        return;
    }

    // Look up specific term by code
    match lookup(arg) {
        Some(t) => {
            println!();
            println!("  {BCYN}{BOLD}{}{RST}", t.code);
            println!("  {BWHT}{}{RST}", t.label);
            println!("  {DIM}Area   :{RST}  {CYN}{}{RST}", t.area.label());
            println!("  {DIM}Chapter:{RST}  {DIM}{}{RST}", t.area.chapter_number());
            println!("  {DIM}Def    :{RST}  {}", t.definition);
            // Show BahyWay alignments for this code
            let aligns: Vec<_> = ALIGNMENTS.iter().filter(|a| a.dama_code == t.code).collect();
            if !aligns.is_empty() {
                println!("  {DIM}{}{RST}", "─".repeat(54));
                println!("  {BYLW}BahyWay Implementation:{RST}");
                for a in &aligns {
                    println!("  {DIM}crate  :{RST}  {BCYN}{}{RST}", a.bahyway_crate);
                    println!("  {DIM}concept:{RST}  {BGRN}{}{RST}", a.bahyway_concept);
                    println!("  {DIM}note   :{RST}  {DIM}{}{RST}", a.mapping_note);
                }
            }
            println!();
        }
        None => {
            println!("  {BRED}Term '{arg}' not found — try .dama search <keyword>{RST}");
        }
    }
}

fn do_dashboard(s: &Session) {
    let particles_raw = s.pdb.db().journal().all_particles();
    let mut rows: Vec<dashboard::ParticleRow> = Vec::new();
    for p in &particles_raw {
        let proj  = s.pdb.db().project(p);
        let state = match proj.state {
            ParticleState::Golden => "GOLDEN",
            ParticleState::Fuzzy  => "FUZZY",
            ParticleState::Dead   => "DEAD",
        };
        rows.push(dashboard::ParticleRow {
            uuid_hash: p.uuid_hash(),
            state,
            epoch:     s.epoch.min(255) as u32,
            events:    proj.events_seen,
        });
    }

    let m = naba::NabaMetrics::gather(&s.pdb, s.tribe, s.epoch);
    let html = dashboard::generate_dashboard(
        s.tribe,
        m.golden,
        m.fuzzy,
        m.dead,
        m.epoch,
        &rows,
    );

    let path = format!("{}/showway.html", s.data_dir);
    match std::fs::write(&path, html) {
        Ok(()) => {
            println!("  {BGRN}✓{RST}  Dashboard written → {CYN}{path}{RST}");
            println!("  {DIM}Open in browser: {BCYN}file://{path}{RST}");
            println!("  {DIM}Particles: {BGRN}{}{RST}{DIM}  Golden: {BYLW}{}{RST}{DIM}  Fuzzy: {YLW}{}{RST}{DIM}  Dead: {DIM}{}{RST}",
                m.total, m.golden, m.fuzzy, m.dead);
        }
        Err(e) => println!("  {BRED}✗ Cannot write dashboard: {e}{RST}"),
    }
}

fn do_status(s: &Session) {
    let count    = s.pdb.db().journal().all_particles().len();
    let replayed = s.pdb.stats().entries_replayed;
    let written  = s.pdb.stats().entries_written;
    println!();
    println!("  {BOLD}Session{RST}");
    println!("  {DIM}tribe       :{RST}  {BCYN}{:#06x}{RST}", s.tribe);
    println!("  {DIM}data-dir    :{RST}  {CYN}{}{RST}", s.data_dir);
    println!("  {DIM}particles   :{RST}  {BGRN}{count}{RST}");
    println!("  {DIM}replayed    :{RST}  {DIM}{replayed}{RST}");
    println!("  {DIM}written     :{RST}  {DIM}{written}{RST}");
    println!("  {DIM}ingested    :{RST}  {}", s.stats.ingested);
    println!("  {DIM}queries     :{RST}  {}", s.stats.queries);
    println!("  {DIM}checks      :{RST}  {}", s.stats.checks);
    println!("  {DIM}errors      :{RST}  {BRED}{}{RST}", s.stats.errors);
    match &s.buf {
        Some(b) => println!("  {DIM}buffer      :{RST}  {BYLW}{}{RST}  [{}]", b.path, b.kind_label()),
        None    => println!("  {DIM}buffer      :{RST}  {DIM}(empty){RST}"),
    }
    println!();
}

// ── Banner & prompt ───────────────────────────────────────────────────────────

fn banner(tribe: u16, data_dir: &str, replayed: usize) {
    println!();
    println!("  {BCYN}{BOLD}𒁾  DubSar IDE v4.0{RST}");
    println!("  {DIM}BahyWay Sovereign Ecosystem — Pure Rust, Zero Dependencies{RST}");
    println!("  {DIM}tribe    : {BCYN}{tribe:#06x}{RST}");
    println!("  {DIM}data-dir : {CYN}{data_dir}{RST}");
    if replayed > 0 {
        println!("  {BGRN}✓{RST}  {DIM}Replayed {replayed} entries from disk{RST}");
    }
    println!();
    println!("  {DIM}Type {BCYN}.help{RST}{DIM} for commands  ·  {BCYN}.demo{RST}{DIM} to load sample data  ·  {BCYN}.naba{RST}{DIM} to summon NABA{RST}");
    println!();
}

fn prompt(tribe: u16) {
    print!("{BCYN}𒁾{RST} {DIM}[{tribe:#06x}]{RST} {BOLD}>{RST} ");
    let _ = io::stdout().flush();
}

// ── Main loop ─────────────────────────────────────────────────────────────────

fn default_data_dir() -> String {
    env::var("HOME")
        .map(|h| format!("{h}/.bahyway"))
        .unwrap_or_else(|_| "/tmp/.bahyway".to_string())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut tribe_id = 0x0001u16;
    let mut data_dir = default_data_dir();
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--tribe" && i + 1 < args.len() {
            let hex = args[i+1].trim_start_matches("0x");
            tribe_id = u16::from_str_radix(hex, 16).unwrap_or(0x0001);
            i += 2;
        } else if args[i] == "--data-dir" && i + 1 < args.len() {
            data_dir = args[i+1].clone();
            i += 2;
        } else { i += 1; }
    }

    let mut s = Session::new(tribe_id, &data_dir).unwrap_or_else(|e| {
        eprintln!("error: {e}");
        std::process::exit(1);
    });
    let replayed = s.pdb.stats().entries_replayed;
    banner(tribe_id, &data_dir, replayed);
    let stdin = io::stdin();
    prompt(s.tribe);

    for line in stdin.lock().lines() {
        let line = match line { Ok(l) => l, Err(_) => break };
        let t    = line.trim();
        if t.is_empty() { prompt(s.tribe); continue; }

        if      let Some(a) = t.strip_prefix(".query ")   { do_query(&mut s, a); }
        else if let Some(a) = t.strip_prefix(".ingest ")  { do_ingest(&mut s, a); }
        else if let Some(a) = t.strip_prefix(".open ")    { do_open(&mut s, a); }
        else if let Some(a) = t.strip_prefix(".fields ")  { do_fields(&s, a); }
        else if let Some(a) = t.strip_prefix(".color ")   { do_color(a); }
        else if let Some(a) = t.strip_prefix(".dama ")    { do_dama(a); }
        else {
            match t {
                ".naba"        => do_naba(&s),
                ".help"        => do_help(),
                ".demo"        => { do_demo(&mut s); naba::naba_hint(&naba::NabaMetrics::gather(&s.pdb, s.tribe, s.epoch)); }
                ".particles"   => do_particles(&mut s),
                ".check"       => do_check(&mut s),
                ".run"         => do_run(&mut s),
                ".templates"   => do_templates(&s),
                ".dama"        => do_dama(""),
                ".dashboard"   => do_dashboard(&s),
                ".status"      => do_status(&s),
                ".quit"|".exit"=> { println!("  {BCYN}𒁾  DubSar signing off.{RST}"); break; }
                _              => println!("  {DIM}Unknown command. Type .help{RST}"),
            }
        }

        prompt(s.tribe);
    }
}
