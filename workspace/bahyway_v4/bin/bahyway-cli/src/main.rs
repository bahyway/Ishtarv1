//! bahyway-cli — HeptaScript REPL + AAOL validator (§12).
//!
//! Commands:
//!   .hepta <query>   Run a HeptaScript query against the in-session db
//!   .akk <source>    Validate AAOL source
//!   .help            Show this help
//!   .quit / .exit    Exit
//!
//! Usage: bahyway-cli [--tribe <hex>]

use std::env;
use std::io::{self, BufRead, Write};

use aaol::ast::Parser as AaolParser;
use aaol::tokenize as aaol_lex;
use bahyway_core::TribeId;
use enkidb_engine::EnkiDb;
use enkidb_query::query;
use heptascript::parse_query;

fn main() {
    // Parse --tribe arg
    let args: Vec<String> = env::args().collect();
    let mut tribe_id = 0x0001u16;
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--tribe" && i + 1 < args.len() {
            let hex = args[i + 1].trim_start_matches("0x");
            tribe_id = u16::from_str_radix(hex, 16).unwrap_or(0x0001);
            i += 2;
        } else {
            i += 1;
        }
    }

    let db = EnkiDb::new(TribeId::from_u16(tribe_id));
    let stdin = io::stdin();
    let stdout = io::stdout();

    eprintln!("𒁾 bahyway-cli v4.0  tribe={tribe_id:#06x}");
    eprintln!("Type .help for commands.");

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some(src) = trimmed.strip_prefix(".hepta ") {
            match query(&db, src) {
                Ok(res) => println!("matched={} evaluated={}", res.matched.len(), res.evaluated),
                Err(e) => println!("ERROR: {e}"),
            }
        } else if let Some(src) = trimmed.strip_prefix(".akk ") {
            match aaol_lex(src) {
                Err(e) => println!("Lex error: {e}"),
                Ok(toks) => match AaolParser::new(toks).parse() {
                    Ok(prog) => println!("OK — {} statement(s)", prog.statements.len()),
                    Err(e) => println!("Parse error: {e}"),
                },
            }
        } else if trimmed.strip_prefix(".parse ").is_some() {
            // quick HeptaScript plan dump
            let src = trimmed.strip_prefix(".parse ").unwrap();
            match parse_query(src) {
                Ok(plan) => println!(
                    "plan: at_epoch={:?} conditions={}",
                    plan.when.as_ref().map(|w| format!("{:?}", w)),
                    plan.r#where.len()
                ),
                Err(e) => println!("ERROR: {e}"),
            }
        } else {
            match trimmed {
                ".help" => {
                    println!(".hepta <query>   — HeptaScript query");
                    println!(".akk <source>    — validate AAOL source");
                    println!(".parse <query>   — show query plan");
                    println!(".quit / .exit    — exit");
                }
                ".quit" | ".exit" => {
                    println!("𒁾 bye.");
                    break;
                }
                _ => println!("Unknown command. Type .help"),
            }
        }

        print!("> ");
        let _ = stdout.lock().flush();
    }
}
