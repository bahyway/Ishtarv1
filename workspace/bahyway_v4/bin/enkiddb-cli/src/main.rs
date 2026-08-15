//! 𒁾 EnkiDDB CLI — local directory-ingest bulk loader.
//!
//! Closes EnkiDDB_ROADMAP.md's Phase 2 gap: `WriteNode::ingest_directory_categorized`
//! is real and tested, but nothing exposed it as a one-shot local tool —
//! bulk-loading `docs/` meant either calling the Rust function yourself or
//! sending one document at a time over `enkiddb-write-server`'s TCP
//! protocol. This binary is the "no TCP round-trip needed" option the
//! roadmap described, run in three explicit, always-in-order stages —
//! the Architect's own framing:
//!
//!   1. SCAN        -- walk the directory, list every `.md` file found
//!                     (`enkiddb::scan_markdown_directory`).
//!   2. CATEGORIZE  -- for each file: its inferred `meta.collection`
//!                     (`enkiddb::ingest::infer_collection`) and its git
//!                     authorship check against the team allowlist
//!                     (`enkiddb::check_authorship`) -- reported here so
//!                     you see every rejection *before* anything runs.
//!   3. RUN         -- if every file passed CATEGORIZE, actually journal
//!                     and materialize them
//!                     (`WriteNode::ingest_directory_categorized_checked`,
//!                     which re-checks authorship itself -- the gate is
//!                     enforced in the library, not just this CLI's
//!                     display, so `enkiddb-write-server`'s `INGEST_DIR:`
//!                     command can't bypass it either).
//!
//! ## Usage
//!   enkiddb-cli ingest <dir> [--data-dir <dir>] [--tribe <hex>]
//!                            [--team-allowlist <file>] [--epoch-start <n>]
//!                            [--dry-run] [--remote <host:port>]
//!
//! `--dry-run` stops after CATEGORIZE and reports what *would* happen,
//! without journaling or writing Data Files.
//!
//! `--remote <host:port>` (PB-205): SCAN and CATEGORIZE run exactly as
//! above, but RUN sends each authorized document over the wire to a live
//! `enkiddb-write-server` (`remote::send_document`) instead of journaling
//! locally. This is the VDI-side ingestion half of the "3 Podman + VDI"
//! topology: the git-authorship check runs here, against this host's own
//! real git checkout (correct ownership, git already installed, no
//! container involved) -- the container on the receiving end never sees
//! a directory path or touches `.git` at all, it only ever receives
//! already-verified document bodies.

mod remote;

use std::env;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use bahyway_core::TribeId;
use enkidb_kaki::KakiMinter;
use enkiddb::{check_authorship, ingest, readnode, scan_markdown_directory, TeamAllowlist, WriteNode};

const RST: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const GRN: &str = "\x1b[0;32m";
const BYLW: &str = "\x1b[1;33m";
const BCYN: &str = "\x1b[1;36m";
const BGRN: &str = "\x1b[1;32m";
const BRED: &str = "\x1b[1;31m";

struct Args {
    dir: PathBuf,
    data_dir: PathBuf,
    tribe_id: u16,
    team_allowlist: Option<PathBuf>,
    epoch_start: u32,
    dry_run: bool,
    remote: Option<(String, u16)>,
}

fn main() -> ExitCode {
    let raw: Vec<String> = env::args().collect();

    if raw.len() >= 3 && raw[1] == "topics" {
        return run_topics(Path::new(&raw[2]));
    }

    if raw.len() < 3 || raw[1] != "ingest" {
        print_usage();
        return ExitCode::from(2);
    }

    let args = match parse_args(&raw[2..]) {
        Ok(a) => a,
        Err(msg) => {
            eprintln!("{BRED}error:{RST} {msg}");
            print_usage();
            return ExitCode::from(2);
        }
    };

    println!();
    println!("  {BCYN}{BOLD}𒁾  EnkiDDB CLI — Tigris directory ingest{RST}");
    println!("  {DIM}dir = {}{RST}", args.dir.display());
    println!();

    // ── [1/3] SCAN ──────────────────────────────────────────────────
    println!("{BOLD}[1/3] SCAN{RST}       {DIM}{}{RST}", args.dir.display());
    let files = match scan_markdown_directory(&args.dir) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("  {BRED}error:{RST} {e}");
            return ExitCode::FAILURE;
        }
    };
    if files.is_empty() {
        println!("  {BYLW}no .md files found -- nothing to do.{RST}");
        return ExitCode::SUCCESS;
    }
    println!("  {GRN}{} markdown file(s) found{RST}", files.len());
    println!();

    // PB-210: an empty/whitespace-only file has nothing to ingest and is
    // not a trust or security issue -- treat it as a no-op skip here,
    // not a fail-closed REJECTED that would block every other real
    // document in the same batch (CATEGORIZE's rejection path is
    // all-or-nothing by design for authorship/security, which is right
    // for those two, but wrong for "this file happens to be blank").
    // Found live on 2026-07-19: one empty file in a real 201-document
    // ingest previously slipped past CATEGORIZE and only failed at RUN
    // time against the live server, with a cryptic wire-protocol error
    // ("missing document body after collection line") after 199 other
    // files had already been sent.
    let (files, empty_files): (Vec<_>, Vec<_>) = files.into_iter().partition(|p| {
        std::fs::read_to_string(p).map(|s| !s.trim().is_empty()).unwrap_or(true)
    });
    if !empty_files.is_empty() {
        println!("{BOLD}[1b/3] SKIP (empty){RST}");
        for path in &empty_files {
            let rel = path.strip_prefix(&args.dir).unwrap_or(path);
            println!("  {BYLW}SKIP{RST}     {:<40} -- empty, nothing to ingest", rel.display());
        }
        println!();
    }
    if files.is_empty() {
        println!("  {BYLW}every file found was empty -- nothing to do.{RST}");
        return ExitCode::SUCCESS;
    }

    // ── [2/3] CATEGORIZE ────────────────────────────────────────────
    println!("{BOLD}[2/3] CATEGORIZE{RST}");
    let allowlist = match &args.team_allowlist {
        Some(p) => match TeamAllowlist::from_file(p) {
            Ok(a) => a,
            Err(e) => {
                eprintln!("  {BRED}error reading --team-allowlist {}: {e}{RST}", p.display());
                return ExitCode::FAILURE;
            }
        },
        None => TeamAllowlist::from_env_or_seed(),
    };

    let mut rejected = 0usize;
    for path in &files {
        let collection = ingest::infer_collection(path);
        let check = check_authorship(path, &allowlist);
        let rel = path.strip_prefix(&args.dir).unwrap_or(path);

        // PB-206: a security-scan failure rejects the file regardless of
        // authorship -- checked and reported here so it shows up before
        // RUN, same "see every rejection first" framing as authorship.
        let scan = std::fs::read(path).ok().map(|bytes| enkiddb::scan_document(&bytes));
        let security_ok = scan.as_ref().map(|s| s.clean).unwrap_or(true);

        if check.authorized && security_ok {
            println!(
                "  {GRN}OK{RST}       {:<40} -> {:<20} author={}",
                rel.display(),
                collection,
                check.author.as_deref().unwrap_or("?"),
            );
        } else {
            rejected += 1;
            if !security_ok {
                println!(
                    "  {BRED}REJECTED{RST} {:<40} -> {:<20} security={}",
                    rel.display(),
                    collection,
                    scan.as_ref().map(|s| s.detail).unwrap_or("scan failed"),
                );
            } else {
                println!(
                    "  {BRED}REJECTED{RST} {:<40} -> {:<20} author={}",
                    rel.display(),
                    collection,
                    check.author.as_deref().unwrap_or("<unknown -- no git history>"),
                );
            }
        }
    }
    println!();
    println!(
        "  {} authorized, {} rejected{}",
        files.len() - rejected,
        rejected,
        if rejected == 0 { format!(" {DIM}(allowlist: {} member(s)){RST}", allowlist.len()) } else { String::new() },
    );
    println!();

    if rejected > 0 {
        println!(
            "  {BYLW}RUN skipped -- {rejected} file(s) failed the team-authorship or security-scan check.{RST}"
        );
        println!("  {DIM}Fix authorship (commit as a team member), extend --team-allowlist, or remove the flagged content, then retry.{RST}");
        return ExitCode::FAILURE;
    }

    if args.dry_run {
        println!("  {DIM}--dry-run: stopping before RUN.{RST}");
        return ExitCode::SUCCESS;
    }

    // ── [3/3] RUN ───────────────────────────────────────────────────
    if let Some((host, port)) = &args.remote {
        return run_remote(&files, &args.dir, host, *port);
    }

    println!("{BOLD}[3/3] RUN{RST}");
    let minter = KakiMinter::new(TribeId::from_u16(args.tribe_id));
    let mut write_node = WriteNode::new(minter, 64);

    let kakis = match write_node.ingest_directory_categorized_checked(&args.dir, args.epoch_start, &allowlist) {
        Ok(k) => k,
        Err(e) => {
            // Defense in depth: the library re-checks authorship itself,
            // so this should only trigger on a TOCTOU change between
            // CATEGORIZE and RUN (e.g. a file edited mid-run).
            eprintln!("  {BRED}error:{RST} {e}");
            return ExitCode::FAILURE;
        }
    };
    println!("  {GRN}ingested {} document(s){RST}", kakis.len());

    let current = args.data_dir.join("current");
    if let Err(e) = std::fs::create_dir_all(&current) {
        eprintln!("  {BRED}error creating data dir {}:{RST} {e}", current.display());
        return ExitCode::FAILURE;
    }
    match readnode::materialize_now(&write_node, current.join("entities"), current.join("eav")) {
        Ok(stats) => {
            println!(
                "  {GRN}materialized{RST} -> {} {DIM}({} entities){RST}",
                current.display(),
                stats.entities
            );
        }
        Err(e) => {
            eprintln!("  {BRED}error materializing:{RST} {e}");
            return ExitCode::FAILURE;
        }
    }

    println!();
    println!("  {BGRN}𒁾 done.{RST}");
    ExitCode::SUCCESS
}

/// RUN over the wire instead of locally: for each file already authorized
/// by CATEGORIZE (the same git-authorship check, run here on the VDI's
/// own real git checkout -- correct ownership, git already installed, no
/// container involved), sends it to a live `enkiddb-write-server` using
/// the existing per-document command. This is what actually retires the
/// container's need for `git`/`safe.directory` in the client-facing
/// ingest path: the container only ever receives already-verified
/// document bodies over TCP, never a path to scan itself (see PB-205).
fn run_remote(files: &[PathBuf], base: &Path, host: &str, port: u16) -> ExitCode {
    println!("{BOLD}[3/3] RUN (remote){RST}  {DIM}-> {host}:{port}{RST}");
    let mut ok = 0usize;
    let mut failed = 0usize;

    for path in files {
        let collection = ingest::infer_collection(path);
        let rel = path.strip_prefix(base).unwrap_or(path);
        let body = match std::fs::read_to_string(path) {
            Ok(b) => b,
            Err(e) => {
                println!("  {BRED}ERR{RST}      {:<40} -> {collection:<20} read error: {e}", rel.display());
                failed += 1;
                continue;
            }
        };
        match remote::send_document(host, port, &collection, &body) {
            Ok(resp) if resp.starts_with("OK:") => {
                println!("  {GRN}OK{RST}       {:<40} -> {collection:<20} {DIM}{resp}{RST}", rel.display());
                ok += 1;
            }
            Ok(resp) => {
                println!("  {BRED}ERR{RST}      {:<40} -> {collection:<20} {resp}", rel.display());
                failed += 1;
            }
            Err(e) => {
                println!("  {BRED}ERR{RST}      {:<40} -> {collection:<20} {e}", rel.display());
                failed += 1;
            }
        }
    }

    println!();
    if failed == 0 {
        println!("  {BGRN}𒁾 {ok} document(s) sent to {host}:{port}.{RST}");
        ExitCode::SUCCESS
    } else {
        println!("  {ok} sent, {BRED}{failed} failed{RST} -- see errors above.");
        ExitCode::FAILURE
    }
}

fn parse_args(rest: &[String]) -> Result<Args, String> {
    if rest.is_empty() {
        return Err("missing <dir>".to_string());
    }
    let dir = PathBuf::from(&rest[0]);
    let mut data_dir = PathBuf::from(home_dir(".bahyway/enkiddb"));
    let mut tribe_id = 0x7160u16;
    let mut team_allowlist = None;
    let mut epoch_start = 1u32;
    let mut dry_run = false;
    let mut remote = None;

    let mut i = 1;
    while i < rest.len() {
        match rest[i].as_str() {
            "--data-dir" => {
                i += 1;
                data_dir = PathBuf::from(rest.get(i).ok_or("--data-dir needs a value")?);
            }
            "--tribe" => {
                i += 1;
                let hex = rest.get(i).ok_or("--tribe needs a value")?;
                tribe_id = u16::from_str_radix(hex.trim_start_matches("0x"), 16).map_err(|e| e.to_string())?;
            }
            "--team-allowlist" => {
                i += 1;
                team_allowlist = Some(PathBuf::from(rest.get(i).ok_or("--team-allowlist needs a value")?));
            }
            "--epoch-start" => {
                i += 1;
                epoch_start = rest.get(i).ok_or("--epoch-start needs a value")?.parse().map_err(|e: std::num::ParseIntError| e.to_string())?;
            }
            "--dry-run" => dry_run = true,
            "--remote" => {
                i += 1;
                let val = rest.get(i).ok_or("--remote needs a value (host:port)")?;
                let (host, port) = val.rsplit_once(':').ok_or("--remote must be host:port")?;
                let port: u16 = port.parse().map_err(|e: std::num::ParseIntError| e.to_string())?;
                remote = Some((host.to_string(), port));
            }
            other => return Err(format!("unrecognized flag: {other}")),
        }
        i += 1;
    }

    Ok(Args { dir, data_dir, tribe_id, team_allowlist, epoch_start, dry_run, remote })
}

fn home_dir(suffix: &str) -> String {
    env::var("HOME").map(|h| format!("{h}/{suffix}")).unwrap_or_else(|_| format!("/tmp/{suffix}"))
}

/// `enkiddb-cli topics <data-dir>` -- reads `<data-dir>/current/{entities,eav}`
/// (a Read Node's materialized Data Files -- the same layout `ingest`
/// writes and `enkiddb-read-server` opens) and reports the real citation-
/// affinity topic graph over it: `enkiddb::topics::TopicGraph`, built from
/// `meta.collection` (the topic partition already assigned at ingest
/// time) and real `depends-on` citation edges (markdown links resolved
/// at ingest time by `WriteNode::link_discovered_dependencies`). Every
/// document's home topic, affinity, and any cross-topic bridges are
/// printed; documents with no resolvable citations are reported as
/// unplaced, not silently skipped.
fn run_topics(data_dir: &Path) -> ExitCode {
    let current = data_dir.join("current");
    println!();
    println!("  {BCYN}{BOLD}𒁾  EnkiDDB CLI — topic graph{RST}");
    println!("  {DIM}data dir = {}{RST}", current.display());
    println!();

    let mut rn = match readnode::ReadNode::open(current.join("entities"), current.join("eav")) {
        Ok(rn) => rn,
        Err(e) => {
            eprintln!("  {BRED}error opening Data Files:{RST} {e}");
            return ExitCode::FAILURE;
        }
    };
    let graph = match enkiddb::topics::TopicGraph::build_from_readnode(&mut rn) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("  {BRED}error building topic graph:{RST} {e:?}");
            return ExitCode::FAILURE;
        }
    };

    println!(
        "  {GRN}{} document(s){RST} across {GRN}{} topic(s){RST}: {DIM}{}{RST}",
        graph.len(),
        graph.topics().len(),
        graph.topics().join(", ")
    );
    println!();

    let mut current_topic = String::new();
    let mut bridge_count = 0usize;
    let mut unplaced_count = 0usize;
    for doc in graph.documents() {
        if doc.topic != current_topic {
            current_topic = doc.topic.clone();
            println!("{BOLD}[{current_topic}]{RST}");
        }
        let title = doc.title.as_deref().unwrap_or("(untitled)");
        match &doc.placement {
            Some(p) if p.bridges_to.is_empty() => {
                println!("  {GRN}·{RST} {title:<50} {DIM}affinity={:.2} radius={:.2}{RST}", p.affinity, p.radius);
            }
            Some(p) => {
                bridge_count += 1;
                println!(
                    "  {BCYN}⋈{RST} {title:<50} {DIM}affinity={:.2} radius={:.2}{RST}  {BYLW}bridges -> {}{RST}",
                    p.affinity,
                    p.radius,
                    p.bridges_to.join(", ")
                );
            }
            None => {
                unplaced_count += 1;
                println!("  {DIM}?  {title:<50} unplaced -- no resolvable citations{RST}");
            }
        }
    }

    println!();
    println!(
        "  {BGRN}𒁾 done.{RST} {DIM}{} bridge(s), {} unplaced.{RST}",
        bridge_count, unplaced_count
    );
    ExitCode::SUCCESS
}

fn print_usage() {
    eprintln!();
    eprintln!("{BOLD}enkiddb-cli{RST} -- EnkiDDB (Tigris) local directory-ingest bulk loader");
    eprintln!();
    eprintln!("{BOLD}USAGE{RST}");
    eprintln!("  enkiddb-cli ingest <dir> [options]");
    eprintln!("  enkiddb-cli topics <data-dir>   -- report the citation-affinity topic graph");
    eprintln!();
    eprintln!("{BOLD}OPTIONS{RST}");
    eprintln!("  --data-dir <dir>         where to materialize Data Files (default ~/.bahyway/enkiddb)");
    eprintln!("  --tribe <hex>            KAKI tribe id (default 0x7160, matches enkiddb-write-server)");
    eprintln!("  --team-allowlist <file>  one git author email per line (default: $ENKIDDB_TEAM_ALLOWLIST or a built-in seed)");
    eprintln!("  --epoch-start <n>        first epoch number to journal at (default 1)");
    eprintln!("  --dry-run                stop after CATEGORIZE; ingest and journal nothing");
    eprintln!("  --remote <host:port>     RUN over the wire instead of locally: send each authorized document");
    eprintln!("                           to a live enkiddb-write-server (--data-dir/--tribe/--epoch-start ignored)");
    eprintln!();
}
