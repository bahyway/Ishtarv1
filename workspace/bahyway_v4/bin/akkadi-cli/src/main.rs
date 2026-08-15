use std::str::FromStr;

use akkadi_cli::{
    commands::{
        compare::sovereign_vs_legacy,
        kaki::{inspect_kaki, generate_kaki},
        pipeline::pipeline_stats,
        status::system_status,
        tribe::{list_tribes, show_tribe},
    },
    config::AkkadiConfig,
    notebook::{
        cell::{CellKind, CellState, NotebookCell},
        kernel::AkkadiKernel,
        manifest::ReproManifest,
        renderer::{render_cell, render_notebook_header},
        session::AkkadiNotebook,
    },
    output::format::OutputFormat,
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cfg = AkkadiConfig::load();
    let format = OutputFormat::from_str(&cfg.output_format).unwrap_or_default();

    if args.len() < 2 {
        print_usage();
        return;
    }

    match args[1].as_str() {
        "status" => system_status(format),

        "kaki" => {
            let sub = args.get(2).map(String::as_str).unwrap_or("--help");
            match sub {
                "--inspect" => {
                    let kaki_str = args.get(3).map(String::as_str).unwrap_or("");
                    let fmt = flag_format(&args, format);
                    inspect_kaki(kaki_str, fmt);
                }
                "--generate" => {
                    let domain = args.get(3)
                        .and_then(|s| u8::from_str_radix(s.trim_start_matches("0x"), 16).ok())
                        .unwrap_or(0x00);
                    let quality = args.get(4)
                        .and_then(|s| s.parse::<u8>().ok())
                        .unwrap_or(200);
                    let fmt = flag_format(&args, format);
                    generate_kaki(domain, quality, fmt);
                }
                _ => eprintln!("kaki: unknown sub-command '{sub}'; use --inspect | --generate"),
            }
        }

        "pipeline" => {
            if args.iter().any(|a| a == "--stats") {
                pipeline_stats(flag_format(&args, format));
            } else {
                eprintln!("pipeline: use --stats");
            }
        }

        "tribe" => {
            if let Some(pos) = args.iter().position(|a| a == "--show") {
                let dim = args.get(pos + 1).map(String::as_str).unwrap_or("IZI");
                show_tribe(dim, flag_format(&args, format));
            } else {
                list_tribes(flag_format(&args, format));
            }
        }

        "compare" => {
            let tribe = flag_value(&args, "--tribe").unwrap_or("IZI");
            let limit = flag_value(&args, "--limit")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(7);
            sovereign_vs_legacy(tribe, limit, flag_format(&args, format));
        }

        "notebook" => {
            if let Some(pos) = args.iter().position(|a| a == "--run") {
                let path = args.get(pos + 1).map(String::as_str).unwrap_or("cells.akknb");
                run_notebook(path, cfg);
            } else if args.iter().any(|a| a == "--new") {
                new_notebook(cfg);
            } else {
                eprintln!("notebook: use --run <file.akknb> | --new");
            }
        }

        "debug-capture" => {
            let query = flag_value(&args, "--query").unwrap_or("").to_string();
            if query.is_empty() {
                eprintln!("debug-capture: --query \"<QUERY:...|SEARCH:...>\" is required");
                return;
            }
            let issue = flag_value(&args, "--issue").unwrap_or("(no issue description given)").to_string();
            let title = flag_value(&args, "--title").unwrap_or("Client Debug Session").to_string();
            let out = flag_value(&args, "--out").map(String::from);
            let mut cfg = cfg;
            if let Some(target) = flag_value(&args, "--target") {
                if let Some((host, port)) = target.rsplit_once(':') {
                    if let Ok(port) = port.parse::<u16>() {
                        cfg.enkiddb_read_host = host.to_string();
                        cfg.enkiddb_read_port = port;
                    } else {
                        eprintln!("debug-capture: --target port must be numeric, got '{port}'");
                        return;
                    }
                } else {
                    eprintln!("debug-capture: --target must be host:port, got '{target}'");
                    return;
                }
            }
            debug_capture(&title, &issue, &query, out, cfg);
        }

        "vocab" => {
            for word in akkadi::AKKADI_WORDS {
                println!("{:<12} ({:?}) — {}", word.akkadi, word.domain, word.english);
            }
        }

        "--help" | "-h" | "help" => print_usage(),

        other => eprintln!("unknown command '{other}'; run `akkadi --help`"),
    }
}

fn run_notebook(path: &str, cfg: AkkadiConfig) {
    let p = std::path::Path::new(path);
    let mut nb = match AkkadiNotebook::load(p) {
        Ok(nb) => nb,
        Err(e) => { eprintln!("cannot load notebook: {e}"); return; }
    };
    render_notebook_header(&nb.title, &nb.author, &nb.version);
    let kernel = AkkadiKernel::new(cfg);
    for cell in &mut nb.cells {
        kernel.execute(cell);
        render_cell(cell);
    }
}

fn new_notebook(cfg: AkkadiConfig) {
    let mut nb = AkkadiNotebook::new("Sovereign Notebook", "bahyway");
    nb.add_cell(NotebookCell::new(CellKind::Markdown, "# Sovereign Analysis"));
    nb.add_cell(NotebookCell::new(CellKind::PipelineStatus, ""));
    let path = cfg.sovereign_workspace.join("new_notebook.akknb");
    match nb.save(&path) {
        Ok(())  => println!("created {}", path.display()),
        Err(e)  => eprintln!("error: {e}"),
    }
}

/// Runs `query` for real against `cfg.enkiddb_read_host:enkiddb_read_port`,
/// captures whatever the server actually said (success or a real
/// `ERR:...`, either way), and writes the two files the Architect sends
/// a client instead of environment access: a `.akknb` notebook (the
/// issue description as a Markdown cell, the query and its captured
/// output as an `EnkiddbQuery` cell) and a companion `.repro.json`
/// manifest (`notebook::manifest::ReproManifest`) naming the target,
/// the exact request, and the round-trip time.
fn debug_capture(title: &str, issue: &str, query: &str, out: Option<String>, cfg: AkkadiConfig) {
    let mut nb = AkkadiNotebook::new(title, "bahyway");
    nb.add_cell(NotebookCell::new(CellKind::Markdown, format!("# {title}\n\n**Issue reported:** {issue}")));

    let target = format!("{}:{}", cfg.enkiddb_read_host, cfg.enkiddb_read_port);
    let mut query_cell = NotebookCell::new(CellKind::EnkiddbQuery, query);
    let elapsed_ms = match akkadi_cli::notebook::enkiddb_client::send_query(&cfg.enkiddb_read_host, cfg.enkiddb_read_port, query) {
        Ok(resp) => {
            query_cell.output = Some(resp.response);
            query_cell.state = CellState::Done;
            resp.elapsed_ms
        }
        Err(e) => {
            query_cell.output = Some(e);
            query_cell.state = CellState::Error;
            0
        }
    };
    let failed = query_cell.state == CellState::Error;
    nb.add_cell(query_cell);

    let default_name = format!(
        "debug_{}.akknb",
        chrono::Utc::now().format("%Y%m%d_%H%M%S")
    );
    let out_path = out.map(std::path::PathBuf::from).unwrap_or_else(|| cfg.sovereign_workspace.join(&default_name));
    if let Some(parent) = out_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    match nb.save(&out_path) {
        Ok(()) => println!("notebook written: {}", out_path.display()),
        Err(e) => { eprintln!("error writing notebook: {e}"); return; }
    }

    let manifest = ReproManifest {
        title: title.to_string(),
        issue: issue.to_string(),
        target,
        query: query.to_string(),
        captured_at: chrono::Utc::now().to_rfc3339(),
        elapsed_ms,
        dataset_generation: None,
        notebook_file: out_path.file_name().map(|f| f.to_string_lossy().to_string()).unwrap_or_default(),
    };
    let manifest_path = out_path.with_extension("repro.json");
    match manifest.save(&manifest_path) {
        Ok(()) => println!("config manifest written: {}", manifest_path.display()),
        Err(e) => eprintln!("error writing manifest: {e}"),
    }

    if failed {
        eprintln!("note: the captured query returned an error -- see the notebook's EnkiddbQuery cell output.");
    }
}

fn flag_format(args: &[String], default: OutputFormat) -> OutputFormat {
    if let Some(pos) = args.iter().position(|a| a == "--format" || a == "-f") {
        if let Some(val) = args.get(pos + 1) {
            return OutputFormat::from_str(val).unwrap_or(default);
        }
    }
    default
}

fn flag_value<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1)).map(String::as_str)
}

fn print_usage() {
    println!("𒀭𒆳𒁺  akkadi — Sovereign CLI v4.0.0");
    println!();
    println!("USAGE:");
    println!("  akkadi <command> [options]");
    println!();
    println!("COMMANDS:");
    println!("  status                           sovereign system health");
    println!("  kaki --inspect <KAKI>            decode 16-byte PK");
    println!("  kaki --generate <domain> <qual>  generate KAKI");
    println!("  pipeline --stats                 BeeMDM 4 lanes × 8 stations");
    println!("  tribe [--show <DIM>]             Ibn Wahshiyya tribe metrics");
    println!("  compare [--tribe IZI]            EnkiDB vs PostgreSQL");
    println!("  notebook --run <file.akknb>      execute notebook");
    println!("  notebook --new                   scaffold new notebook");
    println!("  debug-capture --query <Q> --issue <text> [--title <t>] [--target host:port] [--out <file>]");
    println!("                                   run a real query against EnkiDDB, write a client-shareable");
    println!("                                   .akknb notebook + .repro.json config manifest");
    println!("  vocab                            akkadi sovereign vocabulary");
    println!();
    println!("OPTIONS:");
    println!("  --format table|json|cuneiform    output format (default: table)");
}
