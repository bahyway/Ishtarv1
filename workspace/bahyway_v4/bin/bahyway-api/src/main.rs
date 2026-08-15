#![forbid(unsafe_code)]
//! bahyway-api — minimal HTTP JSON bridge.
//!
//! Reads the live_export.json written by bee-watchdog and serves it
//! over HTTP with CORS headers so bahyway-web (localhost:8080) can
//! fetch it without a same-origin restriction.
//!
//! No external crates — pure std::net + std::thread.
//!
//! Usage:
//!   bahyway-api --data-dir /var/enkidb [--port 8082]
//!
//! Endpoints:
//!   GET /api/v1/live     → live_export.json   (aggregate totals)
//!   GET /api/v1/batches  → batches_export.json (ring-buffered per-batch history)
//!   GET /health          → "ok"

use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::path::PathBuf;

// ── Arg parsing ────────────────────────────────────────────────────────────────

struct Config {
    data_dir: PathBuf,
    port:     u16,
}

fn parse_args() -> Config {
    let mut data_dir = PathBuf::from("/var/enkidb");
    let mut port     = 8082u16;
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--data-dir" => { i += 1; data_dir = PathBuf::from(&args[i]); }
            "--port"     => { i += 1; port = args[i].parse().unwrap_or(8082); }
            "--help" | "-h" => {
                eprintln!("Usage: bahyway-api [--data-dir <dir>] [--port <port>]");
                std::process::exit(0);
            }
            _ => {}
        }
        i += 1;
    }
    Config { data_dir, port }
}

// ── HTTP handling ──────────────────────────────────────────────────────────────

fn handle(mut stream: std::net::TcpStream, data_dir: PathBuf) {
    let Ok(clone) = stream.try_clone() else { return };
    let mut reader = BufReader::new(clone);

    // Read request line
    let mut req_line = String::new();
    if reader.read_line(&mut req_line).is_err() { return; }

    // Drain headers
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) | Err(_) => break,
            _ => {}
        }
        if line == "\r\n" || line.is_empty() { break; }
    }

    let method = req_line.split_whitespace().next().unwrap_or("GET");
    let path   = req_line.split_whitespace().nth(1).unwrap_or("/");

    // CORS pre-flight
    if method == "OPTIONS" {
        let _ = write_response(&mut stream, "204 No Content", "text/plain", "");
        return;
    }

    let (status, content_type, body): (&str, &str, String) = match path {
        "/api/v1/live" | "/api/v1/live/" => {
            let body = fs::read_to_string(data_dir.join("live_export.json"))
                .unwrap_or_else(|_|
                    r#"{"status":"no_data","msg":"No export yet — run bee-watchdog to process a ZIP batch."}"#
                        .to_string()
                );
            ("200 OK", "application/json", body)
        }
        "/api/v1/batches" | "/api/v1/batches/" => {
            let body = fs::read_to_string(data_dir.join("batches_export.json"))
                .unwrap_or_else(|_|
                    r#"{"status":"no_data","count":0,"batches":[],"msg":"No batches processed yet."}"#
                        .to_string()
                );
            ("200 OK", "application/json", body)
        }
        "/api/v1/tribes" | "/api/v1/tribes/" => {
            let body = fs::read_to_string(data_dir.join("tribes_summary.json"))
                .unwrap_or_else(|_|
                    r#"{"status":"no_data","tribes":[],"msg":"No tribe data yet — run bee-watchdog to process a ZIP batch."}"#
                        .to_string()
                );
            ("200 OK", "application/json", body)
        }
        "/health" => ("200 OK", "text/plain", "ok".to_string()),
        _ => ("404 Not Found", "text/plain", "Not Found".to_string()),
    };

    let _ = write_response(&mut stream, status, content_type, &body);
}

fn write_response(
    stream:       &mut std::net::TcpStream,
    status:       &str,
    content_type: &str,
    body:         &str,
) -> std::io::Result<()> {
    let response = format!(
        "HTTP/1.1 {status}\r\n\
         Content-Type: {content_type}; charset=utf-8\r\n\
         Access-Control-Allow-Origin: *\r\n\
         Access-Control-Allow-Methods: GET, OPTIONS\r\n\
         Access-Control-Allow-Headers: Content-Type\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\
         \r\n\
         {body}",
        body.len()
    );
    stream.write_all(response.as_bytes())
}

// ── Main ───────────────────────────────────────────────────────────────────────

fn main() {
    let cfg = parse_args();
    let addr = format!("0.0.0.0:{}", cfg.port);

    let listener = TcpListener::bind(&addr).unwrap_or_else(|e| {
        eprintln!("[bahyway-api] FATAL: bind {addr} failed — {e}");
        std::process::exit(1);
    });

    eprintln!("[bahyway-api] listening  http://localhost:{}", cfg.port);
    eprintln!("[bahyway-api] data dir : {}", cfg.data_dir.display());
    eprintln!("[bahyway-api] endpoints:");
    eprintln!("[bahyway-api]   GET http://localhost:{}/api/v1/live", cfg.port);
    eprintln!("[bahyway-api]   GET http://localhost:{}/api/v1/batches", cfg.port);
    eprintln!("[bahyway-api]   GET http://localhost:{}/api/v1/tribes", cfg.port);
    eprintln!("[bahyway-api]   GET http://localhost:{}/health", cfg.port);

    for stream in listener.incoming() {
        let Ok(stream) = stream else { continue };
        let data_dir = cfg.data_dir.clone();
        std::thread::spawn(move || handle(stream, data_dir));
    }
}
